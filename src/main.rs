extern crate yaml_rust;
extern crate cpython;

use cpython::{PyObject, Python, NoArgs, ToPyObject};
use cpython::ObjectProtocol; //for call method
use std::collections::HashMap;
use std::io::prelude::*;
use std::fs::File;
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
    execute_module(py, "ssh", &params);

    // clear; cargo build && cp target/debug/duck_check . && PYTHONPATH=modules ./duck_check
}

fn execute_module(py: Python, name: &str, params: &Kwargs) -> () {
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

    let response = "SSH-2.0-OpenSSH_6.8p1-hpn14v6";

    let mut kwargs = Kwargs::new();
    kwargs.insert("response", response);
    let result: bool = instance.call_method(py, "check_response", NoArgs, Some(&kwargs.to_py_object(py))).unwrap().extract(py).unwrap();
    println!("- Module response check is '{}'.", result);
}


