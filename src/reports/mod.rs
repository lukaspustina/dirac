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
        println!("Mööp");
    }
}

