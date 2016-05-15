use cpython::{PyBytes, PyDict, PyObject, PyString, Python, NoArgs, ToPyObject};
use cpython::ObjectProtocol; //for call method
use std::collections::HashMap;
use std::fmt;
use term_painter::ToStyle;
use term_painter::Color::*;
use term_painter::Attr::*;

use super::checks::*;
use super::protocols::*;

pub type Kwargs = HashMap<String, String>;

pub type Results<'a> = HashMap<&'a str, (u16,u16)>;

#[derive(Debug)]
pub enum PropertyError {
    FailedResponseCheck,
    FailedExecution,
    Unclassified,
}

impl fmt::Display for PropertyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err_name = match self {
            &PropertyError::FailedResponseCheck => "FailedResponseCheck",
            &PropertyError::FailedExecution => "FailedExecution",
            &PropertyError::Unclassified => "Unclassified",
        };
        write!(f, "{}", err_name)
    }
}

#[derive(Debug)]
pub struct PropertyResult<'a> {
    pub host: &'a str,
    pub property: &'a Property,
    pub result: Result<(), PropertyError>
}

#[derive(Debug)]
pub struct CheckResult<'a> {
    pub check: &'a Check,
    pub results: Vec<PropertyResult<'a>>
}

impl<'a> CheckResult<'a> {
    pub fn new(check: &'a Check) -> CheckResult {
        CheckResult { check: check, results: Vec::new() }
    }
}

#[derive(Debug)]
pub struct CheckSuiteResult<'a> {
    pub check_suite: &'a CheckSuite,
    pub results: Vec<CheckResult<'a>>,
}

impl<'a> CheckSuiteResult<'a> {
    pub fn new(check_suite: &'a CheckSuite) -> CheckSuiteResult {
        CheckSuiteResult { check_suite: check_suite, results: Vec::new() }
    }
}

pub fn run(check_suite: &CheckSuite) -> CheckSuiteResult {
    let mut check_suite_result = CheckSuiteResult::new(check_suite);

    let gil = Python::acquire_gil();
    let py = gil.python();

    let sys = py.import("sys").unwrap();
    let version: String = sys.get(py, "version").unwrap().extract(py).unwrap();
    info!("* Running Pythion '{}'.", version);

    for check in &check_suite.checks {
        println!("CHECKING [{}]", Bold.paint(&check.inventory_name));
        let mut check_result = CheckResult::new(&check);

        for property in &check.properties {
            println!("  PROPERTY: {} [{}:{}]", property.name, Bold.paint(&property.module), &property.params.get("port").unwrap());
            for host in check_suite.inventory.get(&check.inventory_name).unwrap() {
                debug!("+ Running: '{}' with module '{}' and params '{:?}' for host '{}'.", property.name, property.module, property.params, host);
                let result = execute_module(py, host, &property);
                let property_result = PropertyResult { host: host, property: &property, result: result };

                if property_result.result.is_ok() {
                    println!("    {:>7}: [{}]", Green.paint("Success"), host);
                } else {
                    println!("    {:>7}: [{}]", Red.paint("Failed"), host);
                }

                check_result.results.push(property_result);
            }

        }
        check_suite_result.results.push(check_result);
        println!("");
    }

    check_suite_result
}

fn vec_from(py: Python, po: PyObject) -> Vec<u8> {
    let b = po.extract::<PyBytes>(py).unwrap();
    let s = b.as_slice(py);
    let v = From::from(&s[..]);
    v
}

fn string_from(py: Python, po: PyObject) -> String {
    let s = po.extract::<PyString>(py).unwrap().to_string(py).unwrap().to_string();
    s
}


impl TcpRawResponse {
    fn to_py_object(self: Self, py: Python) -> PyDict {
        let TcpRawResponse(response) = self;
        let py_dict = PyDict::new(py);
        let py_bytes = response.to_py_object(py);
        py_dict.set_item(py, "response", py_bytes);
        py_dict
    }
}

impl TcpTextResponse {
    fn to_py_object(self: Self, py: Python) -> PyDict {
        let TcpTextResponse(response) = self;
        let py_dict = PyDict::new(py);
        let py_string = response.to_py_object(py);
        py_dict.set_item(py, "response", py_string);
        py_dict
    }
}

impl TcpHttpTextResponse {
    fn to_py_object(self: Self, py: Python) -> PyDict {
        let TcpHttpTextResponse(response) = self;
        let py_dict = PyDict::new(py);
        let py_response_code = response.response_code.to_py_object(py);
        py_dict.set_item(py, "response_code", py_response_code);
        // TODO: Implement me
        // let py_headers = self.data.headers.to_py_object(py);
        // py_dict.set_item(py, "headers", py_headers);
        py_dict.set_item(py, "headers", py.None());
        let py_body = response.body.to_py_object(py);
        py_dict.set_item(py, "body", py_body);
        py_dict
    }
}

fn execute_module<'a>(py: Python, host: &str, property: &Property) -> Result<(), PropertyError> {
    let import = py.import(&property.module).unwrap();
    let module: PyObject = import.get(py, "Module").unwrap();
    info!("* Loaded module '{}'.", &property.name);

    let protocol_fn = module.getattr(py, "protocol").unwrap();
    let protocol: String = protocol_fn.call(py, NoArgs, None).unwrap().extract(py).unwrap();
    debug!("- Module protocol is '{}'.", protocol);

    let check_args_fn = module.getattr(py, "check_args").unwrap();
    let check_args: bool = check_args_fn.call(py, NoArgs, Some(&property.params.to_py_object(py))).unwrap().extract(py).unwrap();
    debug!("- Module check args is '{}'.", check_args);

    let instance: PyObject = module.call(py, NoArgs, Some(&property.params.to_py_object(py))).unwrap().extract(py).unwrap();
    debug!("- Module instance is '{}'.", instance);

    let py_challenge: PyObject = instance.call_method(py, "challenge", NoArgs, None).unwrap();
    let py_none = py.None();
    let challenge: Option<String> = if py_challenge == py_none {
        None
    } else {
        Some(py_challenge.extract::<PyString>(py).unwrap().to_string(py).unwrap().to_string())
    };

    let port = property.params["port"].parse::<u16>().unwrap();
    let result = match &protocol[..] {
        "connect/tcp" => {
            let mut p = TcpConnect::new(host, port);
            if let Ok(response) = p.send_challenge() {
                true
            } else {
                return Err(PropertyError::FailedExecution)
            }
        },
        "raw/tcp" => {
            let mut p = TcpRaw::new(host, port);
            if challenge.is_some() {
                p = p.with_data(vec_from(py, py_challenge));
            }
            if let Ok(response) = p.send_challenge() {
                let kwargs = response.to_py_object(py);
                check_response(py, &instance, &kwargs)
            } else {
                return Err(PropertyError::FailedExecution)
            }
        },
        "text/tcp" => {
            let mut p = TcpText::new(host, port);
            if challenge.is_some() {
                p = p.with_data(string_from(py, py_challenge));
            }
            let result = p.send_challenge();
            if result.is_ok() {
                let response: TcpTextResponse = result.unwrap();
                let kwargs = response.to_py_object(py);
                check_response(py, &instance, &kwargs)
            } else {
                return Err(PropertyError::FailedExecution)
            }
        },
        "text/udp" => if let Ok(kwargs) = text_udp( host, property.params["port"].parse::<u16>().unwrap(), challenge) {
            let r: bool = instance.call_method(py, "check_response", NoArgs, Some(&kwargs.to_py_object(py))).unwrap().extract(py).unwrap();
            r
        } else {
            return Err(PropertyError::FailedExecution)
        },
        "http/tcp" => {
            let mut p = TcpHttp::new(host, port).with_data(string_from(py, py_challenge));
            let result = p.send_challenge();
            if result.is_ok() {
                let response = result.unwrap();
                let kwargs = response.to_py_object(py);
                check_response(py, &instance, &kwargs)
            } else {
                return Err(PropertyError::FailedExecution)
            }
        },
        "https/tcp" => if let Ok(kwargs) = https_tcp( host, property.params["port"].parse::<u16>().unwrap(), challenge) {
            let r: bool = instance.call_method(py, "check_response", NoArgs, Some(&kwargs.to_py_object(py))).unwrap().extract(py).unwrap();
            r
        } else {
            return Err(PropertyError::FailedExecution)
        },
        unknown => panic!("Unknown protocol '{}'.", unknown)
    };
    debug!("- Module response check is '{}'.", result);

    return if result {
        Ok(())
    } else {
        Err(PropertyError::FailedExecution)
    }
}

fn check_response(py: Python, instance: &PyObject, response: &PyDict) -> bool {
    let r: bool = instance.call_method(py, "check_response", NoArgs, Some(response)).unwrap().extract(py).unwrap();
    r
}


