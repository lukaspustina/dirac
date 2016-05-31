use hyper::client::{Client, RedirectPolicy};
use std::collections::HashMap;
use std::io::Error;
use std::io::prelude::*;
use std::net::{TcpStream, UdpSocket};
use std::time::Duration;

use super::engine::Kwargs;

pub struct Challenge<'a, T> {
    host: &'a str,
    port: u16,
    data: Option<T>,
}

pub type NoData = ();

pub trait Protocol<'a, S, T, V> {
    fn new(host: &'a str, port: u16) -> S;
    fn set_data(self: &mut Self, data: T);
    fn send_challenge(self: &Self) -> Result<V, Error>;
}

pub struct TcpConnect<'a>(Challenge<'a, NoData>);

pub struct TcpConnectResponse(pub NoData);

impl<'a> Protocol<'a, TcpConnect<'a>, NoData, TcpConnectResponse> for TcpConnect<'a> {
    fn new(host: &'a str, port: u16) -> TcpConnect {
        TcpConnect(Challenge {
            host: host,
            port: port,
            data: None,
        })
    }

    fn set_data(self: &mut Self, _: NoData) {
    }

    fn send_challenge(self: &Self) -> Result<TcpConnectResponse, Error> {
        let TcpConnect(ref challenge) = *self;
        let _ = try!(TcpStream::connect((challenge.host, challenge.port)));

        Ok(TcpConnectResponse(()))
    }
}

pub struct TcpRaw<'a>(Challenge<'a, Vec<u8>>);

pub struct TcpRawResponse(pub Vec<u8>);

impl<'a> Protocol<'a, TcpRaw<'a>, Vec<u8>, TcpRawResponse> for TcpRaw<'a> {
    fn new(host: &'a str, port: u16) -> TcpRaw {
        TcpRaw(Challenge {
            host: host,
            port: port,
            data: None,
        })
    }

    fn set_data(self: &mut Self, data: Vec<u8>) {
        let TcpRaw(ref mut challenge) = *self;
        challenge.data = Some(data);
    }

    fn send_challenge(self: &Self) -> Result<TcpRawResponse, Error> {
        let TcpRaw(ref challenge) = *self;
        let bytes = if let Some(ref data) = challenge.data {
            data.as_slice()
        } else {
            let empty: &[u8] = &[0u8; 0];
            empty
        };
        let response_bytes = try!(tcp_stream_send_recv(challenge.host, challenge.port, bytes));

        Ok(TcpRawResponse(response_bytes))
    }
}

pub struct TcpText<'a>(Challenge<'a, String>);

pub struct TcpTextResponse(pub String);

impl<'a> Protocol<'a, TcpText<'a>, String, TcpTextResponse> for TcpText<'a> {
    fn new(host: &'a str, port: u16) -> TcpText {
        TcpText(Challenge {
            host: host,
            port: port,
            data: None,
        })
    }

    fn set_data(self: &mut Self, data: String) {
        let TcpText(ref mut challenge) = *self;
        challenge.data = Some(data);
    }

    fn send_challenge(self: &Self) -> Result<TcpTextResponse, Error> {
        let TcpText(ref challenge) = *self;
        let bytes = if let Some(ref data) = challenge.data {
            data.as_bytes()
        } else {
            let empty: &[u8] = &[0u8; 0];
            empty
        };
        let response_bytes = try!(tcp_stream_send_recv(challenge.host, challenge.port, bytes));
        let string = String::from_utf8_lossy(&response_bytes.as_slice()).to_string();

        Ok(TcpTextResponse(string))
    }
}

fn tcp_stream_send_recv(host: &str, port: u16, bytes: &[u8]) -> Result<Vec<u8>, Error> {
    let mut stream = try!(TcpStream::connect((host, port)));
    let dur = Duration::new(1, 0);
    stream.set_read_timeout(Some(dur)).unwrap();

    if bytes.len() > 0 {
        let tx_res = try!(stream.write(&bytes));
        // TODO: Assert to real check
        assert_eq!(tx_res, bytes.len());
        debug!("- Sent data '{:?}'.", bytes);
    }

    let mut response_bytes = [0u8; 1024];
    let rx_len = try!(stream.read(&mut response_bytes));
    debug!("- Received result from '{}/{}', '{}' bytes.",
           host,
           port,
           rx_len);

    let v = From::from(&response_bytes[0..rx_len]);
    Ok(v)
}

pub struct UdpText<'a>(Challenge<'a, String>);

pub struct UdpTextResponse(pub String);

impl<'a> Protocol<'a, UdpText<'a>, String, UdpTextResponse> for UdpText<'a> {
    fn new(host: &'a str, port: u16) -> UdpText {
        UdpText(Challenge {
            host: host,
            port: port,
            data: None,
        })
    }

    fn set_data(self: &mut Self, data: String) {
        let UdpText(ref mut challenge) = *self;
        challenge.data = Some(data);
    }

    fn send_challenge(self: &Self) -> Result<UdpTextResponse, Error> {
        let UdpText(ref challenge) = *self;
        let bytes = if let Some(ref data) = challenge.data {
            data.as_bytes()
        } else {
            let empty: &[u8] = &[0u8; 0];
            empty
        };
        let response_bytes = try!(udp_datagram_send_recv(challenge.host, challenge.port, bytes));
        let string = String::from_utf8_lossy(&response_bytes.as_slice()).to_string();

        Ok(UdpTextResponse(string))
    }
}

pub fn udp_datagram_send_recv(host: &str, port: u16, bytes: &[u8]) -> Result<Vec<u8>, Error> {
    let socket = try!(UdpSocket::bind(("0.0.0.0", 18181)));
    let dur = Duration::new(1, 0);
    socket.set_read_timeout(Some(dur)).unwrap();

    if bytes.len() > 0 {
        let tx_res = try!(socket.send_to(&bytes, (host, port)));
        // TODO: Assert to real check
        assert_eq!(tx_res, bytes.len());
        debug!("- Sent data '{:?}'.", bytes);
    }

    let mut rx_buf = [0u8; 1024];
    let (rx_len, _) = try!(socket.recv_from(&mut rx_buf));
    debug!("- Received result from '{}/{}', '{}' bytes.",
           host,
           port,
           rx_len);

    let v = From::from(&rx_buf[0..rx_len]);
    Ok(v)
}


pub struct TcpHttp<'a>(Challenge<'a, String>);

pub struct HttpResponse<T> {
    pub response_code: u16,
    pub headers: HashMap<String, String>,
    pub body: T,
}

pub struct TcpHttpTextResponse(pub HttpResponse<String>);

impl<'a> Protocol<'a, TcpHttp<'a>, String, TcpHttpTextResponse> for TcpHttp<'a> {
    fn new(host: &'a str, port: u16) -> TcpHttp {
        TcpHttp(Challenge {
            host: host,
            port: port,
            data: None,
        })
    }

    fn set_data(self: &mut Self, data: String) {
        let TcpHttp(ref mut challenge) = *self;
        challenge.data = Some(data);
    }

    fn send_challenge(self: &Self) -> Result<TcpHttpTextResponse, Error> {
        let TcpHttp(ref challenge) = *self;
        let mut client = Client::new();

        let data = challenge.data.as_ref().unwrap();
        let data_parts: Vec<&str> = data.split_whitespace().collect();
        let url = format!("http://{}:{}{}",
                          challenge.host,
                          challenge.port,
                          data_parts[1]);
        debug!("- http request '{}'", url);

        client.set_redirect_policy(RedirectPolicy::FollowNone);
        // TODO: use verb instead of hardcoded get
        let res = client.get(&url).send().unwrap();

        let headers = HashMap::new();
        let response_data: HttpResponse<String> = HttpResponse {
            response_code: res.status_raw().0,
            headers: headers,
            body: "<not yet implemented>".to_string(),
        };
        Ok(TcpHttpTextResponse(response_data))
    }
}


pub fn https_tcp(host: &str, port: u16, challenge: Option<String>) -> Result<Kwargs, Error> {
    let mut client = Client::new();

    let c = challenge.unwrap();
    let challenge_parts: Vec<&str> = c.split_whitespace().collect();
    let url = format!("https://{}:{}{}", host, port, challenge_parts[1]);
    debug!("- https request '{}'", url);

    client.set_redirect_policy(RedirectPolicy::FollowNone);
    let res = client.get(&url).send().unwrap();

    let mut kwargs = Kwargs::new();
    kwargs.insert("response_code".to_string(), res.status_raw().0.to_string());
    kwargs.insert("headers".to_string(), "".to_string());
    kwargs.insert("body".to_string(), "".to_string());

    Ok(kwargs)
}
