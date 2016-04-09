pub mod json;

use std::io;

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

