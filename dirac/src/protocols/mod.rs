use hyper::client::{Client, RedirectPolicy};
use std::collections::HashMap;
use std::io::Error;
use std::io::prelude::*;
use std::net::{TcpStream, UdpSocket};
use std::time::Duration;

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

macro_rules! create_protocol {
    ($protocol_name:ident, $data_type:ty, $protocol_response_name: ident, $response_type:ty, $sel:ident, $sender: block) => (
        pub struct $protocol_name<'a>(Challenge<'a, $data_type>);

        pub struct $protocol_response_name(pub $response_type);

        impl<'a> Protocol<'a, $protocol_name<'a>, $data_type, $protocol_response_name> for $protocol_name<'a> {
            fn new(host: &'a str, port: u16) -> $protocol_name {
                $protocol_name(Challenge {
                    host: host,
                    port: port,
                    data: None,
                })
            }

            fn set_data(self: &mut Self, data: $data_type) {
                    let $protocol_name(ref mut challenge) = *self;
                    challenge.data = Some(data);
            }

            fn send_challenge($sel: &Self) -> Result<$protocol_response_name, Error> $sender

        }
    )
}

create_protocol!(TcpConnect, NoData, TcpConnectResponse, NoData, self, {
    let TcpConnect(ref challenge) = *self;
    let _ = try!(TcpStream::connect((challenge.host, challenge.port)));

    Ok(TcpConnectResponse(()))
});

create_protocol!(TcpRaw, Vec<u8>, TcpRawResponse, Vec<u8>, self, {
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
);

create_protocol!(TcpText, String, TcpTextResponse, String, self, {
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
});

create_protocol!(UdpText, String, UdpTextResponse, String, self, {
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
});


pub struct HttpResponse<T> {
    pub response_code: u16,
    pub headers: HashMap<String, String>,
    pub body: T,
}

create_protocol!(TcpHttp, String, TcpHttpTextResponse, HttpResponse<String>, self, {
        let TcpHttp(ref challenge) = *self;
        let response_data = try!(http_send_recv("http", challenge));

        Ok(TcpHttpTextResponse(response_data))
    }
);

create_protocol!(TcpHttps, String, TcpHttpsTextResponse, HttpResponse<String>, self, {
        let TcpHttps(ref challenge) = *self;
        let response_data = try!(http_send_recv("https", challenge));

        Ok(TcpHttpsTextResponse(response_data))
    }
);


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

fn udp_datagram_send_recv(host: &str, port: u16, bytes: &[u8]) -> Result<Vec<u8>, Error> {
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

fn http_send_recv<'a>(url_scheme: &str,
                      challenge: &Challenge<'a, String>)
                      -> Result<HttpResponse<String>, Error> {
    let mut client = Client::new();

    let data = challenge.data.as_ref().unwrap();
    let data_parts: Vec<&str> = data.split_whitespace().collect();
    let url = format!("{}://{}:{}{}",
                      url_scheme,
                      challenge.host,
                      challenge.port,
                      data_parts[1]);
    debug!("- {} request '{}'", url_scheme, url);

    client.set_redirect_policy(RedirectPolicy::FollowNone);
    // TODO: use verb instead of hardcoded get and handle hyper Errors
    let res = client.get(&url).send().unwrap();

    let headers = HashMap::new();
    let response_data: HttpResponse<String> = HttpResponse {
        response_code: res.status_raw().0,
        headers: headers,
        body: "<not yet implemented>".to_string(),
    };

    Ok(response_data)
}
