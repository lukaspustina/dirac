use std::collections::HashMap;
use std::io;
use std::io::prelude::*;
use std::fs::File;

use super::super::engine::{CheckSuiteResult, PropertyResult};
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
        let mut report = String::new();

        report.push_str("# Dirac Report\n");
        report.push_str("\n");

        report.push_str("## Summary\n");
        report.push_str("\n");
        let summary = create_total_summary(self);
        for kv in summary {
            report.push_str(&format!(" * *{}* Success {}, Failed {}\n", kv.0, (kv.1).0, (kv.1).1));
        }
        report.push_str("\n");

        report.push_str("## Host Checks\n");
        report.push_str("\n");
        let summary = create_host_summary(self);
        for host in summary.keys() {
            report.push_str(&format!("### {}\n", host));
            report.push_str("\n");
            for property_result in summary.get(host).unwrap() {
                report.push_str(&format!("* {} *{}*", property_result.property.name, property_result.property.module));
                let r = if property_result.result.is_ok() {
                    "Success"
                } else {
                    "**Failed**"
                };
                report.push_str(&format!(" {}\n", r));
            report.push_str("\n");
            }
        }

        report.push_str("\n");

        report
    }
}

fn create_total_summary<'a>(check_suite_result: &'a CheckSuiteResult) -> HashMap<&'a str, (u16, u16)> {
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

fn create_host_summary<'a>(check_suite_result: &'a CheckSuiteResult) -> HashMap<&'a str, Vec<&'a PropertyResult<'a>>> {
    let mut result = HashMap::new();

    for check in &check_suite_result.results {
        for property in &check.results {
            let mut host_result = result.entry(property.host).or_insert(Vec::new());
            host_result.push(property);
        }
    }

    result
}

