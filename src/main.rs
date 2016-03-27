extern crate yaml_rust;
extern crate cpython;

use cpython::{PythonObject, Python, NoArgs};
use cpython::ObjectProtocol; //for call method
use std::io::prelude::*;
use std::fs::File;
use yaml_rust::{YamlLoader};

fn main() {
    let mut f = File::open("./examples/esel.ducks").unwrap();
    let mut yaml_str = String::new();
    f.read_to_string(&mut yaml_str);
    let docs = YamlLoader::load_from_str(&yaml_str).unwrap();

    println!("{:?}", docs);

    let gil = Python::acquire_gil();
    let py = gil.python(); // obtain `Python` token

    let telnet = py.import("telnet").unwrap();
    let type_fn = telnet.get(py, "type").unwrap();
    let type_str: String = type_fn.call(py, NoArgs, None).unwrap().extract(py).unwrap();

    let os = py.import("os").unwrap();
    let getenv = os.get(py, "getenv").unwrap();
    let user: String = getenv.call(py, ("USER",), None).unwrap().extract(py).unwrap();

    println!("Module type is '{}'.", type_str);

    // cargo build && cp target/debug/duck_check . && PYTHONPATH=examples ./duck_check
}
