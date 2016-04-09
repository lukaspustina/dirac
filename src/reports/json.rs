use std::collections::BTreeMap;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use rustc_serialize::json::{ToJson, Json};

use super::super::engine::{CheckResult, CheckSuiteResult, PropertyResult};
use super::Report;

pub struct JsonReport<'a> {
    check_suite_result: &'a CheckSuiteResult<'a>,
    filename: &'a str,
}

impl<'a> JsonReport<'a> {
    pub fn new(check_suite_result: &'a CheckSuiteResult, filename: &'a str) -> JsonReport<'a> {
        JsonReport {
            check_suite_result: check_suite_result,
            filename: filename,
        }
    }
}

impl<'a> Report<'a> for JsonReport<'a> {
    fn as_string(&self) -> String {
        format!("{}", self.check_suite_result.to_json().pretty())
    }

    fn write_to_file(&self) -> io::Result<()> {
        let mut f = try!(File::create(self.filename));
        f.write_all(self.as_string().as_bytes())
    }
}

impl<'a > ToJson for PropertyResult<'a> {
    fn to_json(&self) -> Json {
        let mut d = BTreeMap::new();
        d.insert("host".to_string(), self.host.to_json());
        let mut property = BTreeMap::new();
        property.insert("name".to_string(), self.property.name.to_json());
        property.insert("module".to_string(), self.property.module.to_json());
        property.insert("params".to_string(), self.property.params.to_json());
        d.insert("property".to_string(), property.to_json());
        let property_result = match &self.result {
            &Ok(()) => "Success".to_string(),
            &Err(ref err) => err.to_string(),
        };
        d.insert("property_result".to_string(), property_result.to_json());
        Json::Object(d)
    }
}

impl<'a > ToJson for CheckResult<'a> {
    fn to_json(&self) -> Json {
        let mut d = BTreeMap::new();
        d.insert("inventory_name".to_string(), self.check.inventory_name.to_json());
        d.insert("property_results".to_string(), self.results.to_json());
        Json::Object(d)
    }
}

impl<'a > ToJson for CheckSuiteResult<'a> {
    fn to_json(&self) -> Json {
        let mut d = BTreeMap::new();
        d.insert("inventory".to_string(), self.check_suite.inventory.to_json());
        d.insert("check_results".to_string(), self.results.to_json());
        Json::Object(d)
    }
}



