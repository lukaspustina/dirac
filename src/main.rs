extern crate yaml_rust;
extern crate cpython;

use cpython::{PyObject, Python, NoArgs};
use cpython::ObjectProtocol; //for call method
use std::io::prelude::*;
use std::fs::File;
use yaml_rust::{YamlLoader};

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

    let text_tcp = py.import("text_tcp").unwrap();
    let module: PyObject = text_tcp.get(py, "Module").unwrap();
    let module_protocol_fn = module.getattr(py, "protocol").unwrap();
    let module_protocol_str: String = module_protocol_fn.call(py, NoArgs, None).unwrap().extract(py).unwrap();

    println!("* Module protocol is '{}'.", module_protocol_str);

    let module_obj: PyObject = module.call(py, NoArgs, None).unwrap().extract(py).unwrap();
    let _ = module_obj.call_method(py, "check_response", NoArgs, None).unwrap();

    // cargo build && cp target/debug/duck_check . && PYTHONPATH=modules ./duck_check
}
