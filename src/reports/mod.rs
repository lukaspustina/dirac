pub mod json;

use std::collections::HashMap;
use std::io;
use term_painter::ToStyle;
use term_painter::Color::*;
use term_painter::Attr::*;

use super::engine::*;
use self::json::*;


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

pub struct SummaryReport<'a> {
    check_suite_result: &'a CheckSuiteResult<'a>,
}

impl<'a> Report<'a> for SummaryReport<'a> {
    // TODO: Farben werden nicht dargestellt
    fn as_string(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("{}\n", Bold.paint("SUMMARY")));
        let summary = SummaryReport::create_summary(self.check_suite_result);
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

    fn create_summary(check_suite_result: &'a CheckSuiteResult) -> HashMap<&'a str, (u16, u16)> {
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
