extern crate env_logger;
#[macro_use]
extern crate log;
extern crate term_painter;

extern crate dirac;

use std::collections::HashMap;
use std::env;
use term_painter::ToStyle;
use term_painter::Color::*;
use term_painter::Attr::*;

use dirac::checks::CheckSuite;

fn main() {
    // PYTHONPATH=modules cargo run -- examples/pdt.yml

    if env_logger::init().is_err() {
        panic!("Could not initiliaze logger");
    }
    let args: Vec<_> = env::args().collect();

    let check_suite = CheckSuite::read_from_file(&args[1]).unwrap();

    let results_by_host = dirac::engine::run(&check_suite);

    println!("SUMMARY");
    for kv in results_by_host.iter() {
        println!("{:<20} Success {:>4}, Failed {:>4}", Bold.paint(kv.0), Green.paint((kv.1).0), Red.paint((kv.1).1));
    }
}

