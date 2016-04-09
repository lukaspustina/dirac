use std::io;
use std::io::prelude::*;
use std::fs::File;

use super::super::engine::{CheckResult, CheckSuiteResult, PropertyResult};
use super::Report;

pub struct MarkdownReport<'a> {
    check_suite_result: &'a CheckSuiteResult<'a>,
    filename: &'a str,
}

impl<'a> MarkdownReport<'a> {
    pub fn new(check_suite_result: &'a CheckSuiteResult, filename: &'a str) -> MarkdownReport<'a> {
        MarkdownReport {
            check_suite_result: check_suite_result,
            filename: filename,
        }
    }
}

impl<'a> Report<'a> for MarkdownReport<'a> {
    fn as_string(&self) -> String {
        format!("{}", self.check_suite_result.to_md())
    }

    fn write_to_file(&self) -> io::Result<()> {
        let mut f = try!(File::create(self.filename));
        f.write_all(self.as_string().as_bytes())
    }
}

trait ToMarkdown {
    fn to_md(&self) -> String;
}

impl<'a> ToMarkdown for CheckSuiteResult<'a> {
    fn to_md(&self) -> String {
        "Mööp".to_string()
    }
}

