use std::collections::HashMap;
use term_painter::ToStyle;
use term_painter::Color::*;
use term_painter::Attr::*;

use super::engine::CheckSuiteResult;

pub trait Report<'a> {
    fn new(check_suite_result: &'a CheckSuiteResult) -> Self;
    fn print(&self);
}

pub struct SummaryReport<'a> {
    check_suite_result: &'a CheckSuiteResult<'a>,
}

impl<'a> Report<'a> for SummaryReport<'a> {
    fn new(check_suite_result: &'a CheckSuiteResult) -> SummaryReport<'a> {
        SummaryReport { check_suite_result: check_suite_result }
    }

    fn print(&self) {
        println!("{}", Bold.paint("SUMMARY"));
        let summary = SummaryReport::createSummary(self.check_suite_result);
        for kv in summary {
            println!(" * {:<30} Success {:4}, Failed {:4}", kv.0, Green.paint((kv.1).0), Red.paint((kv.1).1));
        }
    }
}

impl<'a> SummaryReport<'a> {
    fn createSummary(check_suite_result: &'a CheckSuiteResult) -> HashMap<&'a str, (u16, u16)> {
        let mut result = HashMap::new();

        for check in &check_suite_result.results {
            for property in &check.results {
                let mut host_result = result.entry(property.host).or_insert((0,0));
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
