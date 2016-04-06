extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate term_painter;

extern crate dirac;

use clap::{Arg, ArgMatches, App};
use std::collections::HashMap;
use std::env;


use dirac::checks::CheckSuite;
use dirac::engine::CheckSuiteResult;
use dirac::reports::*;

static VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    if env_logger::init().is_err() {
        panic!("Could not initiliaze logger");
    }
    let cli_args = App::new("Dirac Host Properties Checker")
                       .version(VERSION)
                       .arg(Arg::with_name("report")
                                .takes_value(true)
                                .requires("output")
                                .possible_values(&["json"])
                                .short("r")
                                .long("report")
                                .value_name("REPORT")
                                .help("Enables report"))
                       .arg(Arg::with_name("output")
                                .takes_value(true)
                                .requires("report")
                                .short("o")
                                .long("output")
                                .value_name("FILENAME")
                                .help("Sets output file for report"))
                       .arg(Arg::with_name("check_suite")
                                .takes_value(true)
                                .value_name("FILENAME")
                                .min_values(1)
                                .help("Check suites to run"))
                       .get_matches();

    let check_suite_filenames = cli_args.values_of("check_suite").unwrap();

    for filename in check_suite_filenames {
        let check_suite = CheckSuite::read_from_file(&filename).unwrap();
        let results = dirac::engine::run(&check_suite);

        let report = SummaryReport::new(&results);
        report.print();

        if cli_args.is_present("report") && cli_args.is_present("output") {
            let report_type = cli_args.value_of("report").unwrap().to_string();
            let report_filename = cli_args.value_of("output").unwrap().to_string();

            let mut report_builder = Reporter::new(&results, &report_type);
            let report = report_builder.with_filename(&report_filename).create();
            report.print();
        }
    }
}
