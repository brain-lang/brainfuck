//! Brainfuck language interpreter

#[macro_use]
extern crate clap;

#[macro_use]
extern crate brainfuck;

use std::path::{Path};
use std::fs::File;
use std::io::prelude::*;

use clap::{Arg, App};

use brainfuck::{precompile, interpret};

fn main() {
    let args = App::new(crate_name!())
        .version(crate_version!())
        .version_short("v")
        .author(crate_authors!())
        .about(crate_description!())
        .arg(Arg::with_name("input-file")
            .help("The brainfuck file to process. Should contain brainfuck instructions")
            .value_name("file")
            .takes_value(true)
            .required(true)
        )
        .arg(Arg::with_name("debug-enabled")
            .short("D")
            .long("debug")
            .help("Enables debug mode which outputs debugging information to stderr")
        )
        .arg(Arg::with_name("delay")
            .long("delay")
            .takes_value(true)
            .help("Delays execution of each instruction by this amount in ms")
        )
        .get_matches();

    let source_path = Path::new(args.value_of("input-file").unwrap());
    if !source_path.exists() || !source_path.is_file() {
        exit_with_error!("Not a valid file: '{}'", source_path.display());
    }

    let debug_mode = args.is_present("debug-enabled");

    let delay: u64 = if let Some(delay_str) = args.value_of("delay") {
        delay_str.parse().unwrap_or_else(|e: std::num::ParseIntError| exit_with_error!("Invalid delay: {}", e))
    } else {
        0
    };

    let mut f = File::open(source_path).unwrap_or_else(|e| {
        exit_with_error!("Could not open source file: {}", e);
    });

    let mut bytes = Vec::new();
    f.read_to_end(&mut bytes).expect("Fatal: Could not read source file");
    let program = precompile(bytes.iter());
    interpret(program, ::std::io::stdout(), debug_mode, delay);
}
