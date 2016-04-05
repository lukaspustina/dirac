extern crate cpython;
extern crate env_logger;
extern crate hyper;
#[macro_use]
extern crate log;
extern crate term_painter;

extern crate dirac;

use cpython::{PyObject, PyString, Python, NoArgs, ToPyObject};
use cpython::ObjectProtocol; //for call method
use hyper::client::{Client, RedirectPolicy};
use std::collections::HashMap;
use std::env;
use std::io::prelude::*;
use std::fs::File;
use std::net::{TcpStream, UdpSocket};
use std::time::Duration;
use term_painter::ToStyle;
use term_painter::Color::*;
use term_painter::Attr::*;

use dirac::checks::CheckSuite;

type Kwargs = HashMap<String, String>;

fn main() {
    // PYTHONPATH=modules cargo run -- examples/pdt.yml

    if env_logger::init().is_err() {
        panic!("Could not initiliaze logger");
    }
    let args: Vec<_> = env::args().collect();
    let check_suite = CheckSuite::read_from_file(&args[1]).unwrap();

    let gil = Python::acquire_gil();
    let py = gil.python();

    let sys = py.import("sys").unwrap();
    let version: String = sys.get(py, "version").unwrap().extract(py).unwrap();
    info!("* Running Pythion '{}'.", version);

    let mut results_by_host = HashMap::<&str, (u16,u16)>::new();
    for check in check_suite.checks {
        println!("CHECKING [{}]", Bold.paint(&check.inventory_name));
        for property in check.properties {
            println!("  PROPERTY: {} [{}:{}]", property.name, Bold.paint(&property.module), &property.params.get("port").unwrap());
            for host in check_suite.inventory.get(&check.inventory_name).unwrap() {
                debug!("+ Running: '{}' with module '{}' and params '{:?}' for host '{}'.", property.name, property.module, property.params, host);
                let result = execute_module(py, host, &property.module, &property.params);
                if result {
                    println!("    {:>7}: [{}]", Green.paint("Success"), host);
                } else {
                    println!("    {:>7}: [{}]", Red.paint("Failed"), host);
                }

                let mut host_results = results_by_host.entry(&host).or_insert((0,0));
                if result {
                    host_results.0 += 1;
                } else {
                    host_results.1 += 1;
                }
            }
        }
        println!("");
    }

    println!("SUMMARY");
    for kv in results_by_host.iter() {
        println!("{:<20} Success {:>4}, Failed {:>4}", Bold.paint(kv.0), Green.paint((kv.1).0), Red.paint((kv.1).1));
    }

}

fn execute_module(py: Python, host: &str, name: &str, params: &Kwargs) -> bool {
    let import = py.import(name).unwrap();
    let module: PyObject = import.get(py, "Module").unwrap();
    info!("* Loaded module '{}'.", name);

    let protocol_fn = module.getattr(py, "protocol").unwrap();
    let protocol: String = protocol_fn.call(py, NoArgs, None).unwrap().extract(py).unwrap();
    debug!("- Module protocol is '{}'.", protocol);

    let check_args_fn = module.getattr(py, "check_args").unwrap();
    let check_args: bool = check_args_fn.call(py, NoArgs, Some(&params.to_py_object(py))).unwrap().extract(py).unwrap();
    debug!("- Module check args is '{}'.", check_args);

    let instance: PyObject = module.call(py, NoArgs, Some(&params.to_py_object(py))).unwrap().extract(py).unwrap();
    debug!("- Module instance is '{}'.", instance);

    let py_challenge: PyObject = instance.call_method(py, "challenge", NoArgs, None).unwrap();
    let py_none = py.None();
    let challenge: Option<String> = if py_challenge == py_none {
        None
    } else {
        Some(py_challenge.extract::<PyString>(py).unwrap().to_string(py).unwrap().to_string())
    };

    let kwargs = match &protocol[..] {
        "raw/tcp" => if let Ok(res) = raw_tcp( host, params["port"].parse::<u16>().unwrap()) {
            res
        } else {
            return false
        },
        "text/tcp" => if let Ok(res) = text_tcp( host, params["port"].parse::<u16>().unwrap(), challenge) {
            res
        } else {
            return false
        },
        "text/udp" => if let Ok(res) = text_udp( host, params["port"].parse::<u16>().unwrap(), challenge) {
            res
        } else {
            return false
        },
        "http/tcp" => if let Ok(res) = http_tcp( host, params["port"].parse::<u16>().unwrap(), challenge) {
            res
        } else {
            return false
        },
        "https/tcp" => if let Ok(res) = https_tcp( host, params["port"].parse::<u16>().unwrap(), challenge) {
            res
        } else {
            return false
        },
        unknown => panic!("Unknown protocol '{}'.", unknown)
    };

    let result: bool = instance.call_method(py, "check_response", NoArgs, Some(&kwargs.to_py_object(py))).unwrap().extract(py).unwrap();
    debug!("- Module response check is '{}'.", result);

    result
}

fn raw_tcp(host: &str, port: u16) -> Result<Kwargs, std::io::Error> {
    let mut kwargs = Kwargs::new();

    let mut stream = try!(TcpStream::connect((host, port)));

    Ok(kwargs)
}

fn text_udp(host: &str, port: u16, challenge: Option<String>) -> Result<Kwargs, std::io::Error> {
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

fn text_tcp(host: &str, port: u16, challenge: Option<String>) -> Result<Kwargs, std::io::Error> {
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

fn http_tcp(host: &str, port: u16, challenge: Option<String>) -> Result<Kwargs, std::io::Error> {
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

fn https_tcp(host: &str, port: u16, challenge: Option<String>) -> Result<Kwargs, std::io::Error> {
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

