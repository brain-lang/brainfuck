//! Brainfuck language interpreter

#[macro_use]
extern crate clap;

extern crate brainfuck;

use std::path::{Path};
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::thread;
use std::time::Duration;

use clap::{Arg, App};

use brainfuck::{precompile, interpret};

macro_rules! exit_with_error(
    ($($arg:tt)*) => { {
        use std::process;
        writeln!(&mut ::std::io::stderr(), $($arg)*)
            .expect("Failed while printing to stderr");
        process::exit(1);
    } }
);

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
        .arg(Arg::with_name("optimize")
            .short("O")
            .long("optimize")
            .value_name("opt-level")
            // Do not optimize by default if debug is enabled
            .default_value_if("debug-enabled", None, "0")
            .default_value("1")
            .possible_values(&["0", "1"])
            .help("Optimize for execution speed")
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

    // We can call unwrap() because the validation is already done by clap
    let opt = args.value_of("optimize").unwrap().parse().unwrap();

    let mut f = File::open(source_path).unwrap_or_else(|e| {
        exit_with_error!("Could not open source file: {}", e);
    });

    let mut bytes = Vec::new();
    f.read_to_end(&mut bytes).expect("Fatal: Could not read source file");
    let program = precompile(bytes.iter(), opt);
    interpret(
        io::stdin(),
        io::stdout(),
        program,
        |state| {
            if debug_mode {
                writeln!(
                    &mut io::stderr(),
                    "{{\"nextInstructionIndex\": {}, \"lastInstruction\": {}, \"currentPointer\": {}, \"memory\": \"{}\"}}",
                    state.next_instruction,
                    state.last_instruction.map_or_else(|| "null".to_owned(), |instr| format!("\"{}\"", instr.to_string())),
                    state.current_pointer,
                    state.memory.iter().fold(String::new(), |acc, v| format!("{} {}", acc, v))
                ).expect("failed printing to stderr");
            }

            thread::sleep(Duration::from_millis(delay));
        }
    );
}
