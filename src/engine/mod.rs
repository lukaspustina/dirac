use cpython::ObjectProtocol; //for call method
use cpython::{PyBytes, PyDict, PyErr, PyObject, PyString, Python, NoArgs, ToPyObject};
use std::collections::HashMap;
use std::fmt;
use term_painter::ToStyle;
use term_painter::Color::*;
use term_painter::Attr::*;

use super::checks::*;
use super::protocols::*;

pub type Kwargs = HashMap<String, String>;

pub type Results<'a> = HashMap<&'a str, (u16, u16)>;

#[derive(Debug)]
pub enum PropertyError {
    FailedExecution,
    FailedResponseCheck,
    FailedPythonCall(PyErr),
    Unclassified,
}

impl fmt::Display for PropertyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err_name = match self {
            &PropertyError::FailedExecution => "FailedExecution".to_string(),
            &PropertyError::FailedResponseCheck => "FailedResponseCheck".to_string(),
            &PropertyError::FailedPythonCall(ref err) => format!("{:?}", err),
            &PropertyError::Unclassified => "Unclassified".to_string(),
        };
        write!(f, "{}", err_name)
    }
}

impl From<PyErr> for PropertyError {
    fn from(err: PyErr) -> PropertyError {
        PropertyError::FailedPythonCall(err)
    }
}

#[derive(Debug)]
pub struct PropertyResult<'a> {
    pub host: &'a str,
    pub property: &'a Property,
    pub result: Result<(), PropertyError>,
}

#[derive(Debug)]
pub struct CheckResult<'a> {
    pub check: &'a Check,
    pub results: Vec<PropertyResult<'a>>,
}

impl<'a> CheckResult<'a> {
    pub fn new(check: &'a Check) -> CheckResult {
        CheckResult {
            check: check,
            results: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct CheckSuiteResult<'a> {
    pub check_suite: &'a CheckSuite,
    pub results: Vec<CheckResult<'a>>,
}

impl<'a> CheckSuiteResult<'a> {
    pub fn new(check_suite: &'a CheckSuite) -> CheckSuiteResult {
        CheckSuiteResult {
            check_suite: check_suite,
            results: Vec::new(),
        }
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
            println!("  PROPERTY: {} [{}:{}]",
                     property.name,
                     Bold.paint(&property.module),
                     &property.params.get("port").unwrap());
            for host in check_suite.inventory.get(&check.inventory_name).unwrap() {
                debug!("+ Running: '{}' with module '{}' and params '{:?}' for host '{}'.",
                       property.name,
                       property.module,
                       property.params,
                       host);
                let result = execute_module(py, host, &property);

                match result {
                    Ok(_) => println!("    {:>11}: [{}]", Green.paint("Success"), host),
                    Err(ref err) => {
                        match *err {
                            PropertyError::FailedExecution => {
                                println!("    {:>11}: [{}]", Red.paint("Failed (E)"), host)
                            }
                            PropertyError::FailedResponseCheck => {
                                println!("    {:>11}: [{}]", Red.paint("Failed (R)"), host)
                            }
                            PropertyError::FailedPythonCall(ref py_err) => {
                                println!("    {:>11}: [{}]", Red.paint("Failed (P)"), host);
                                // TODO: Make me configurable
                                if true {
                                    // TODO: Destructure and pretty print me
                                    println!("{:?}", py_err);
                                }
                            }
                            PropertyError::Unclassified => {
                                println!("    {:>11}: [{}]", Red.paint("Failed (?)"), host)
                            }
                        }
                    }
                }

                let property_result = PropertyResult {
                    host: host,
                    property: &property,
                    result: result,
                };

                check_result.results.push(property_result);
            }

        }
        check_suite_result.results.push(check_result);
        println!("");
    }

    check_suite_result
}

trait ToData<T> {
    fn to_data(py: Python, po: PyObject) -> Option<T>;
}

impl<'a> ToData<NoData> for TcpConnect<'a> {
    #[allow(unused_variables)]
    fn to_data(py: Python, po: PyObject) -> Option<NoData> {
        None
    }
}

impl<'a> ToData<Vec<u8>> for TcpRaw<'a> {
    fn to_data(py: Python, po: PyObject) -> Option<Vec<u8>> {
        let py_none = py.None();
        if po != py_none {
            let b = po.extract::<PyBytes>(py).unwrap();
            let s = b.as_slice(py);
            let v = From::from(&s[..]);
            Some(v)
        } else {
            None
        }
    }
}

impl<'a> ToData<String> for TcpText<'a> {
    fn to_data(py: Python, po: PyObject) -> Option<String> {
        let py_none = py.None();
        if po != py_none {
            let s = string_from(py, po);
            Some(s)
        } else {
            None
        }
    }
}

impl<'a> ToData<String> for UdpText<'a> {
    fn to_data(py: Python, po: PyObject) -> Option<String> {
        let py_none = py.None();
        if po != py_none {
            let s = string_from(py, po);
            Some(s)
        } else {
            None
        }
    }
}

impl<'a> ToData<String> for TcpHttp<'a> {
    fn to_data(py: Python, po: PyObject) -> Option<String> {
        let py_none = py.None();
        if po != py_none {
            let s = string_from(py, po);
            Some(s)
        } else {
            None
        }
    }
}

impl<'a> ToData<String> for TcpHttps<'a> {
    fn to_data(py: Python, po: PyObject) -> Option<String> {
        let py_none = py.None();
        if po != py_none {
            let s = string_from(py, po);
            Some(s)
        } else {
            None
        }
    }
}


fn string_from(py: Python, po: PyObject) -> String {
    let s = po.extract::<PyString>(py).unwrap().to_string(py).unwrap().to_string();
    s
}

trait ToDict {
    fn to_dict(py: Python, response: Self) -> PyDict;
}

impl ToDict for TcpConnectResponse {
    #[allow(unused_variables)]
    fn to_dict(py: Python, response: TcpConnectResponse) -> PyDict {
        PyDict::new(py)
    }
}

impl ToDict for TcpRawResponse {
    fn to_dict(py: Python, response: TcpRawResponse) -> PyDict {
        let TcpRawResponse(response) = response;
        let py_dict = PyDict::new(py);
        let py_bytes = response.to_py_object(py);
        let _ = py_dict.set_item(py, "response", py_bytes);
        py_dict
    }
}

impl ToDict for TcpTextResponse {
    fn to_dict(py: Python, response: TcpTextResponse) -> PyDict {
        let TcpTextResponse(response) = response;
        let py_dict = PyDict::new(py);
        let py_string = response.to_py_object(py);
        let _ = py_dict.set_item(py, "response", py_string);
        py_dict
    }
}

impl ToDict for UdpTextResponse {
    fn to_dict(py: Python, response: UdpTextResponse) -> PyDict {
        let UdpTextResponse(response) = response;
        let py_dict = PyDict::new(py);
        let py_string = response.to_py_object(py);
        let _ = py_dict.set_item(py, "response", py_string);
        py_dict
    }
}

impl ToDict for TcpHttpTextResponse {
    fn to_dict(py: Python, response: TcpHttpTextResponse) -> PyDict {
        let TcpHttpTextResponse(response) = response;
        let py_dict = PyDict::new(py);
        let py_response_code = response.response_code.to_py_object(py);
        let _ = py_dict.set_item(py, "response_code", py_response_code);
        // TODO: Implement me
        // let py_headers = self.data.headers.to_py_object(py);
        // py_dict.set_item(py, "headers", py_headers);
        let _ = py_dict.set_item(py, "headers", py.None());
        let py_body = response.body.to_py_object(py);
        let _ = py_dict.set_item(py, "body", py_body);
        py_dict
    }
}

impl ToDict for TcpHttpsTextResponse {
    fn to_dict(py: Python, response: TcpHttpsTextResponse) -> PyDict {
        let TcpHttpsTextResponse(response) = response;
        let py_dict = PyDict::new(py);
        let py_response_code = response.response_code.to_py_object(py);
        let _ = py_dict.set_item(py, "response_code", py_response_code);
        // TODO: Implement me
        // let py_headers = self.data.headers.to_py_object(py);
        // py_dict.set_item(py, "headers", py_headers);
        let _ = py_dict.set_item(py, "headers", py.None());
        let py_body = response.body.to_py_object(py);
        let _ = py_dict.set_item(py, "body", py_body);
        py_dict
    }
}

fn execute_module<'a>(py: Python, host: &str, property: &Property) -> Result<(), PropertyError> {
    let import = try!(py.import(&property.module));
    let module: PyObject = try!(import.get(py, "Module"));
    info!("* Loaded module '{}'.", &property.name);

    let protocol_fn = try!(module.getattr(py, "protocol"));
    let protocol: String = try!(protocol_fn.call(py, NoArgs, None)).extract(py).unwrap();
    debug!("- Module protocol is '{}'.", protocol);

    let check_args_fn = try!(module.getattr(py, "check_args"));
    let check_args: bool = try!(check_args_fn.call(py,
                                                   NoArgs,
                                                   Some(&property.params.to_py_object(py))))
                               .extract(py)
                               .unwrap();
    debug!("- Module check args is '{}'.", check_args);

    let instance: PyObject = try!(module.call(py, NoArgs, Some(&property.params.to_py_object(py))))
                                 .extract(py)
                                 .unwrap();
    debug!("- Module instance is '{}'.", instance);

    let py_challenge: PyObject = try!(instance.call_method(py, "challenge", NoArgs, None));
    let port = property.params["port"].parse::<u16>().unwrap();
    let result = match &protocol[..] {
        "connect/tcp" => {
            let p = TcpConnect::new(host, port);
            try!(run_protocol(py, p, instance, None))
        }
        "raw/tcp" => {
            let p = TcpRaw::new(host, port);
            let challenge = TcpRaw::to_data(py, py_challenge);
            try!(run_protocol(py, p, instance, challenge))
        }
        "text/tcp" => {
            let p = TcpText::new(host, port);
            let challenge = TcpText::to_data(py, py_challenge);
            try!(run_protocol(py, p, instance, challenge))
        }
        "text/udp" => {
            let p = UdpText::new(host, port);
            let challenge = UdpText::to_data(py, py_challenge);
            try!(run_protocol(py, p, instance, challenge))
        }
        "http/tcp" => {
            let p = TcpHttp::new(host, port);
            let challenge = TcpHttp::to_data(py, py_challenge);
            try!(run_protocol(py, p, instance, challenge))
        }
        "https/tcp" => {
            let p = TcpHttps::new(host, port);
            let challenge = TcpHttps::to_data(py, py_challenge);
            try!(run_protocol(py, p, instance, challenge))
        }
        unknown => panic!("Unknown protocol '{}'.", unknown),
    };
    debug!("- Module response check is '{}'.", result);

    return if result {
        Ok(())
    } else {
        Err(PropertyError::FailedExecution)
    };
}

fn run_protocol<'a, S, T, V, P>(py: Python,
                                mut p: P,
                                instance: PyObject,
                                data: Option<T>)
                                -> Result<bool, PropertyError>
    where V: ToDict,
          P: Protocol<'a, S, T, V> + ToData<T>
{
    if data.is_some() {
        p.set_data(data.unwrap());
    }
    if let Ok(response) = p.send_challenge() {
        let kwargs = ToDict::to_dict(py, response);
        Ok(try!(check_response(py, &instance, &kwargs)))
    } else {
        Err(PropertyError::FailedExecution)
    }
}

fn check_response(py: Python, instance: &PyObject, response: &PyDict) -> Result<bool, PyErr> {
    let r: bool = try!(instance.call_method(py, "check_response", NoArgs, Some(response)))
                      .extract(py)
                      .unwrap();
    Ok(r)
}
