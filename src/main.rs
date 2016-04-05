extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate term_painter;

extern crate dirac;

use clap::{Arg, ArgMatches, App};
use std::collections::HashMap;
use std::env;
use term_painter::ToStyle;
use term_painter::Color::*;
use term_painter::Attr::*;

use dirac::checks::CheckSuite;

static VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    if env_logger::init().is_err() {
        panic!("Could not initiliaze logger");
    }
    let cli_args = App::new("Dirac Host Properties Checker")
                   .version(VERSION)
                   .arg(Arg::with_name("check_suite")
                            .takes_value(true)
                            .value_name("FILE")
                            .min_values(1)
                            .help("Check suites to run"))
                   .get_matches();

    let check_suite_filenames = cli_args.values_of("check_suite").unwrap();
    for filename in check_suite_filenames {
        run_check_suite(filename);
    }
}

fn run_check_suite(file_name: &str) {
    let check_suite = CheckSuite::read_from_file(file_name).unwrap();
    let results_by_host = dirac::engine::run(&check_suite);

    println!("SUMMARY");
    for kv in results_by_host.iter() {
        println!("{:<20} Success {:>4}, Failed {:>4}", Bold.paint(kv.0), Green.paint((kv.1).0), Red.paint((kv.1).1));
    }
}

