use hyper::client::{Client, RedirectPolicy};
use std::io::Error;
use std::io::prelude::*;
use std::net::{TcpStream, UdpSocket};
use std::time::Duration;

use super::engine::Kwargs;

pub fn raw_tcp(host: &str, port: u16) -> Result<Kwargs, Error> {
    let mut kwargs = Kwargs::new();

    let mut stream = try!(TcpStream::connect((host, port)));

    Ok(kwargs)
}

pub fn text_udp(host: &str, port: u16, challenge: Option<String>) -> Result<Kwargs, Error> {
    let mut kwargs = Kwargs::new();

    let dur = Duration::new(5, 0);
    let mut socket = try!(UdpSocket::bind(("0.0.0.0", 18181)));
    socket.set_read_timeout(Some(dur)).unwrap();

    if let Some(c) = challenge {
        let tx_buf = c.as_bytes();
        let tx_len = tx_buf.len();
        try!(socket.send_to(tx_buf, (host, port)));
    };

    let mut rx_buf = [0; 1024];
    let (rx_len, _) = try!(socket.recv_from(&mut rx_buf));

    let response = String::from_utf8_lossy(&rx_buf[0..rx_len]).to_string();
    kwargs.insert("response".to_string(), response);
    Ok(kwargs)
}

pub fn text_tcp(host: &str, port: u16, challenge: Option<String>) -> Result<Kwargs, Error> {
    let mut stream = try!(TcpStream::connect((host, port)));

    if let Some(challenge) = challenge {
        let challenge_bytes = challenge.as_bytes();
        let tx_res = try!(stream.write(&challenge_bytes));
        // TODO: Assert to real check
        assert_eq!(tx_res, challenge_bytes.len());
    }

    let mut response_bytes = [0; 1024];
    let rx_len = try!(stream.read(&mut response_bytes));
    let response = String::from_utf8_lossy(&response_bytes[0..rx_len]).to_string();
    debug!("- Received result from '{}/{}', result: '{:?}'.",
           host,
           port,
           response);

    let mut kwargs = Kwargs::new();
    kwargs.insert("response".to_string(), response.to_string());

    Ok(kwargs)
}

pub fn http_tcp(host: &str, port: u16, challenge: Option<String>) -> Result<Kwargs, Error> {
    let mut client = Client::new();

    let c = challenge.unwrap();
    let challenge_parts: Vec<&str> = c.split_whitespace().collect();
    let url = format!("http://{}:{}{}", host, port, challenge_parts[1]);
    debug!("- http request '{}'", url);

    client.set_redirect_policy(RedirectPolicy::FollowNone);
    let res = client.get(&url).send().unwrap();

    let mut kwargs = Kwargs::new();
    kwargs.insert("response_code".to_string(), res.status_raw().0.to_string());
    kwargs.insert("header".to_string(), "".to_string());
    kwargs.insert("body".to_string(), "".to_string());

    Ok(kwargs)
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
    kwargs.insert("header".to_string(), "".to_string());
    kwargs.insert("body".to_string(), "".to_string());

    Ok(kwargs)
}

