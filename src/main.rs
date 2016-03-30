extern crate yaml_rust;
extern crate cpython;

use cpython::{PyObject, PyString, Python, NoArgs, ToPyObject};
use cpython::ObjectProtocol; //for call method
use std::collections::HashMap;
use std::io::prelude::*;
use std::fs::File;
use std::net::TcpStream;
use yaml_rust::{YamlLoader, Yaml};

type Kwargs = HashMap<String, String>;

#[derive(Debug)]
struct Property {
    name: String,
    module: String,
    params: HashMap<String, String>
}

#[derive(Debug)]
struct DuckCheck<'a> {
    hosts: &'a Vec<String>,
    properties: Vec<Property>
}

fn main() {
    let mut f = File::open("./examples/esel.ducks").unwrap();
    let mut yaml_str = String::new();
    let _ = f.read_to_string(&mut yaml_str);
    let docs = YamlLoader::load_from_str(&yaml_str).unwrap();
    println!("* {:?}", docs);

/*
[Array([
       Hash({
           String("inventory"): Hash({
               String("raspi"): Array([
                    String("heimbot.fritz.box")])})}),
       Hash({
           String("hosts"): String("raspi"),
           String("properties"): Array([
             Hash({
                 String("name"): String("Checking SSH"),
                 String("ssh"): Hash({
                     String("port"): Integer(22),
                     String("software"): String("OpenSSH.*"),
                     String("version"): Real("2.0")})})])})])]
*/

    // We assume only one document in the file which consists of a list of hashes
    let mut inventory: HashMap<String, Vec<String>> = HashMap::new();
    let INVENTORY = Yaml::from_str("inventory");
    let HOSTS = Yaml::from_str("hosts");
    let PROPERTIES = Yaml::from_str("properties");
    let NAME = Yaml::from_str("name");
    let mut duck_checks = Vec::new();
    for hash in docs[0].as_vec().unwrap() {
        let map = hash.as_hash().unwrap();
        if map.contains_key(&INVENTORY) {
            println!("- Found inventory: {:?}", hash);
            let inventory_yaml = map.get(&INVENTORY).unwrap().as_hash().unwrap();
            for hosts_name_yaml in inventory_yaml.keys() {
                let hosts_name = hosts_name_yaml.as_str().unwrap().to_string();
                let mut hosts: Vec<String> = Vec::new();
                for host in inventory_yaml.values().next().unwrap().as_vec().unwrap() {
                    let h = host.as_str().unwrap().to_string();
                    hosts.push(h);
                }
                println!("- - Inventory name: '{:?}'", hosts_name);
                println!("- - Inventory hosts: '{:?}'", hosts);
                inventory.insert(hosts_name.to_string(), hosts);
            }
        }
    }
    for hash in docs[0].as_vec().unwrap() {
        let map = hash.as_hash().unwrap();
        if map.contains_key(&INVENTORY) {
        } else {
            if map.contains_key(&HOSTS) && map.contains_key(&PROPERTIES) {
                println!("- Found duck check: {:?}", hash);
                let hosts = inventory.get(map.get(&HOSTS).unwrap().as_str().unwrap()).unwrap();
                let mut properties = Vec::new();

                let properties_yaml = map.get(&PROPERTIES).unwrap().as_vec().unwrap();
                for property_yml in properties_yaml {
                    let mut name: Option<String> = None;
                    let mut params = HashMap::new();
                    let mut module: Option<String> = None;
                    for elem in property_yml.as_hash().unwrap() {
                        if elem.0 == &NAME {
                            name = Some(elem.1.as_str().unwrap().to_string());
                        } else {
                            module = Some(elem.0.as_str().unwrap().to_string());
                        }
                    }
                    if module.is_some() {
                        let params_yaml = property_yml.as_hash().unwrap().get(&Yaml::from_str(module.as_ref().unwrap())).unwrap().as_hash().unwrap();
                        for kv in params_yaml {
                            let value: String = match *kv.1 {
                                Yaml::Integer(i) => i.to_string(),
                                Yaml::Real(ref r) => r.to_string(),
                                Yaml::String(ref string) => string.to_string(),
                                _ => "<could not translate YAML value>".to_string(),
                            };
                            params.insert(kv.0.as_str().unwrap().to_string(), value);
                        }
                    }
                    properties.push( Property { name: name.unwrap(), module: module.unwrap(), params: params } );
                }

                let duck_check = DuckCheck { hosts: hosts, properties: properties };
                println!("- Created a duck check: {:?}", duck_check);
                duck_checks.push(duck_check);
            }

        }
    }
    println!("* Inventory: {:?}", inventory);
    println!("* DuckChecks: {:?}", duck_checks);

    let gil = Python::acquire_gil();
    let py = gil.python();

    let sys = py.import("sys").unwrap();
    let version: String = sys.get(py, "version").unwrap().extract(py).unwrap();
    println!("* Running Pythion '{}'.", version);

    for duck_check in duck_checks {
        for property in duck_check.properties {
            for host in duck_check.hosts {
                println!("-> Running: '{}' with module '{}' and params '{:?}'.", property.name, property.module, property.params);
                execute_module(py, host, &property.module, &property.params);
            }
        }
    }

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
    kwargs.insert("response".to_string(), response.to_string());
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

