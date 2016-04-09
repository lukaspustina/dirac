use std::collections::HashMap;
use std::collections::BTreeMap;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use rustc_serialize::json::{self, ToJson, Json};
use term_painter::ToStyle;
use term_painter::Color::*;
use term_painter::Attr::*;

use super::checks::*;
use super::engine::*;

pub struct Reporter<'a> {
    check_suite_result: &'a CheckSuiteResult<'a>,
    report_type: ReportType,
    filename: Option<&'a str>,
}

impl<'a> Reporter<'a> {
    pub fn new(check_suite_result: &'a CheckSuiteResult, report_type_str: &'a str) -> Reporter<'a> {
        let report_type = match report_type_str {
            "json" => ReportType::Json,
            _ => panic!("Mööp"),
        };
        Reporter {
            check_suite_result: check_suite_result,
            report_type: report_type,
            filename: None,
        }
    }

    pub fn with_filename(&'a mut self, filename: &'a str) -> &'a mut Reporter<'a> {
        self.filename = Some(filename);
        self
    }

    pub fn create(&self) -> JsonReport<'a> {
        match self.report_type {
            ReportType::Json => JsonReport::new(self.check_suite_result, self.filename.unwrap()),
        }
    }
}

pub enum ReportType {
    Json,
}

pub trait Report<'a> {
    fn as_string(&self) -> String;

    fn write_to_file(&self) -> io::Result<()>;
}

pub struct JsonReport<'a> {
    check_suite_result: &'a CheckSuiteResult<'a>,
    filename: &'a str,
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

impl<'a> JsonReport<'a> {
    fn new(check_suite_result: &'a CheckSuiteResult, filename: &'a str) -> JsonReport<'a> {
        JsonReport {
            check_suite_result: check_suite_result,
            filename: filename,
        }
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


pub struct SummaryReport<'a> {
    check_suite_result: &'a CheckSuiteResult<'a>,
}

impl<'a> Report<'a> for SummaryReport<'a> {
    // TODO: Farben werden nicht dargestellt
    fn as_string(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("{}\n", Bold.paint("SUMMARY")));
        let summary = SummaryReport::createSummary(self.check_suite_result);
        for kv in summary {
            s.push_str(&format!(" * {:<30} Success {:4}, Failed {:4}\n",
                     kv.0,
                     Green.paint((kv.1).0),
                     Red.paint((kv.1).1)));
        }
        s
    }

    fn write_to_file(&self) -> io::Result<()> {
        panic!("Not implemented");
    }
}

impl<'a> SummaryReport<'a> {
    pub fn new(check_suite_result: &'a CheckSuiteResult) -> SummaryReport<'a> {
        SummaryReport { check_suite_result: check_suite_result }
    }

    fn createSummary(check_suite_result: &'a CheckSuiteResult) -> HashMap<&'a str, (u16, u16)> {
        let mut result = HashMap::new();

        for check in &check_suite_result.results {
            for property in &check.results {
                let mut host_result = result.entry(property.host).or_insert((0, 0));
                if property.result.is_ok() {
                    host_result.0 += 1;
                } else {
                    host_result.1 += 1;
                }
            }
        }

        result
    }
}
