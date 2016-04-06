use std::collections::HashMap;
use term_painter::ToStyle;
use term_painter::Color::*;
use term_painter::Attr::*;

use super::engine::CheckSuiteResult;

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
    fn print(&self);
}

pub struct JsonReport<'a> {
    check_suite_result: &'a CheckSuiteResult<'a>,
    filename: &'a str,
}

impl<'a> Report<'a> for JsonReport<'a> {
    fn print(&self) {
        println!("Mööp: Json Report");
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


pub struct SummaryReport<'a> {
    check_suite_result: &'a CheckSuiteResult<'a>,
}

impl<'a> Report<'a> for SummaryReport<'a> {
    fn print(&self) {
        println!("{}", Bold.paint("SUMMARY"));
        let summary = SummaryReport::createSummary(self.check_suite_result);
        for kv in summary {
            println!(" * {:<30} Success {:4}, Failed {:4}",
                     kv.0,
                     Green.paint((kv.1).0),
                     Red.paint((kv.1).1));
        }
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
