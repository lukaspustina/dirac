extern crate yaml_rust;
extern crate cpython;

use cpython::{PyObject, PyString, Python, NoArgs, ToPyObject};
use cpython::ObjectProtocol; //for call method
use std::collections::HashMap;
use std::io::prelude::*;
use std::fs::File;
use std::net::TcpStream;
use yaml_rust::{YamlLoader};

type Kwargs<'a> = HashMap<&'a str, &'a str>;

fn main() {
    let mut f = File::open("./examples/esel.ducks").unwrap();
    let mut yaml_str = String::new();
    let _ = f.read_to_string(&mut yaml_str);
    let docs = YamlLoader::load_from_str(&yaml_str).unwrap();

    println!("* {:?}", docs);

    let gil = Python::acquire_gil();
    let py = gil.python();

    let sys = py.import("sys").unwrap();
    let version: String = sys.get(py, "version").unwrap().extract(py).unwrap();
    println!("* Running Pythion '{}'.", version);

    let mut params = Kwargs::new();
    params.insert("port", "22");
    params.insert("version", "2.0");
    params.insert("software", "OpenSSH.*");
    execute_module(py, "esel.fritz.box", "ssh", &params);

    // clear; cargo build && cp target/debug/duck_check . && PYTHONPATH=modules ./duck_check
}

fn execute_module(py: Python, host: &str, name: &str, params: &Kwargs) -> () {
    let import = py.import(name).unwrap();
    let module: PyObject = import.get(py, "Module").unwrap();
    println!("* Loaded module '{}'.", name);

    let protocol_fn = module.getattr(py, "protocol").unwrap();
    let protocol: String = protocol_fn.call(py, NoArgs, None).unwrap().extract(py).unwrap();
    println!("- Module protocol is '{}'.", protocol);

    let check_args_fn = module.getattr(py, "check_args").unwrap();
    let check_args: bool = check_args_fn.call(py, NoArgs, Some(&params.to_py_object(py))).unwrap().extract(py).unwrap();
    println!("- Module check args is '{}'.", check_args);

    let instance: PyObject = module.call(py, NoArgs, Some(&params.to_py_object(py))).unwrap().extract(py).unwrap();
    println!("- Module instance is '{}'.", instance);

    let py_challenge: PyObject = instance.call_method(py, "challenge", NoArgs, None).unwrap();
    let py_none = py.None();
    let challenge: Option<String> = if py_challenge == py_none {
        None
    } else {
        Some(py_challenge.extract::<PyString>(py).unwrap().to_string(py).unwrap().to_string())
    };

    let response = text_tcp(
        host,
        params["port"].parse::<u16>().unwrap(),
        challenge).unwrap();

    let mut kwargs = Kwargs::new();
    kwargs.insert("response", &response);
    let result: bool = instance.call_method(py, "check_response", NoArgs, Some(&kwargs.to_py_object(py))).unwrap().extract(py).unwrap();
    println!("- Module response check is '{}'.", result);
}

fn text_tcp(host: &str, port: u16, challenge: Option<String>) -> Result<String, std::io::Error> {
    let mut stream = try!(TcpStream::connect((host, port)));

    if let Some(challenge) = challenge {
        let challenge_bytes = challenge.as_bytes();
        let tx_res = try!(stream.write(&challenge_bytes));
        // TODO: Assert to real check
        assert_eq!(tx_res, challenge_bytes.len());
    }

    let mut response_bytes = [0; 1024];
    let rx_len = try!(stream.read(&mut response_bytes));
    let respone = String::from_utf8_lossy(&response_bytes[0..rx_len]).to_string();
    println!("- Received result from '{}/{}', result: '{:?}'.",
           host,
           port,
           respone);

    Ok(respone)
}

