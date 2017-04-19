use std::fmt;
use std::io;
use std::io::prelude::*;
use std::collections::VecDeque;
use std::thread;
use std::time::Duration;
use std::str::FromStr;
use std::iter::repeat;

#[macro_export]
macro_rules! exit_with_error(
    ($($arg:tt)*) => { {
        use std::process;
        writeln!(&mut ::std::io::stderr(), $($arg)*)
            .expect("Failed while printing to stderr");
        process::exit(1);
    } }
);

#[macro_export]
macro_rules! println_stderr(
    ($($arg:tt)*) => { {
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("failed printing to stderr");
    } }
);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Instruction {
    // ">" - increment the pointer (move it to the "right")
    Right(usize),
    // "<" - decrement the pointer (move it to the "left")
    Left(usize),
    // "+" - increment the byte at the pointer
    Increment(usize),
    // "-" - decrement the byte at the pointer
    Decrement(usize),
    // "." - output the byte at the pointer
    Write,
    // "," - input a byte and store it in the byte at the pointer
    Read,
    // "[" - jump forward past the matching ] if the byte at the pointer is zero
    JumpForwardIfZero {
        // Store the instruction index of the matching parenthesis
        // Lazily initialized as needed
        matching: Option<usize>,
    },
    // "]" - jump backward to the matching [ unless the byte at the pointer is zero
    JumpBackwardUnlessZero {
        // Store the instruction index of the matching parenthesis
        // Strictly initialized when the program is loaded into memory
        matching: usize,
    },
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            Instruction::Right(n) => repeat(">").take(n).collect(),
            Instruction::Left(n) => repeat("<").take(n).collect(),
            Instruction::Increment(n) => repeat("+").take(n).collect(),
            Instruction::Decrement(n) => repeat("-").take(n).collect(),
            Instruction::Write => ".".to_owned(),
            Instruction::Read => ",".to_owned(),
            Instruction::JumpForwardIfZero { .. } => "[".to_owned(),
            Instruction::JumpBackwardUnlessZero { .. } => "]".to_owned(),
        })
    }
}

pub enum OptimizationLevel {
    // Do not optimize
    Off,
    // Optimize for speed
    Speed,
}

impl FromStr for OptimizationLevel {
    type Err = ();

    fn from_str(val: &str) -> Result<Self, Self::Err> {
        match val {
            "0" => Ok(OptimizationLevel::Off),
            "1" => Ok(OptimizationLevel::Speed),
            _ => Err(()),
        }
    }
}

/// Precompile the program into an appropriate in-memory representation
pub fn precompile<'a, I>(bytes: I, opt: OptimizationLevel) -> Vec<Instruction>
    where I: IntoIterator<Item=&'a u8> {
    // We typically don't expect to see more than this many levels of nested jumps
    // based on an analysis of some brainfuck programs
    let mut jump_stack = VecDeque::with_capacity(15);
    bytes.into_iter().map(|&c|
        match c {
            b'>' => Some(Instruction::Right(1)),
            b'<' => Some(Instruction::Left(1)),
            b'+' => Some(Instruction::Increment(1)),
            b'-' => Some(Instruction::Decrement(1)),
            b'.' => Some(Instruction::Write),
            b',' => Some(Instruction::Read),
            b'[' => Some(Instruction::JumpForwardIfZero {matching: None}),
            b']' => Some(Instruction::JumpBackwardUnlessZero {matching: 0}),
            _ => None,
        }
    ).filter(|c| c.is_some()).map(|c| c.unwrap()).enumerate().map(|(i, c)|
        match c {
            Instruction::JumpForwardIfZero { .. } => {
                jump_stack.push_back(i);
                c
            },
            Instruction::JumpBackwardUnlessZero { .. } => {
                let matching = jump_stack.pop_back()
                    .expect(&format!("Mismatched jump instruction at position {}", i + 1));
                Instruction::JumpBackwardUnlessZero {matching: matching + 1}
            }
            _ => c,
        }
    ).collect::<Vec<Instruction>>()
}

pub fn interpret<OutFile>(program: Vec<Instruction>, mut out: OutFile, debug: bool, delay: u64)
    where OutFile: Write {
    let mut buffer: VecDeque<u8> = VecDeque::new();
    // Make sure there is at least one cell to begin with
    buffer.push_back(0u8);

    // p is the position "pointer" in the buffer
    let mut p: usize = 0;
    // i is the instruction index in the program
    let mut i: usize = 0;

    loop {
        if i >= program.len() {
            break;
        }
        let mut instr = program[i];
        i += 1;

        match instr {
            Instruction::Right(amount) => {
                p += amount;
                while p >= buffer.len() {
                    buffer.push_back(0u8);
                }
            },
            Instruction::Left(amount) => {
                if amount > p {
                    for _ in 0..(amount - p) {
                        buffer.push_front(0u8);
                    }
                    p = 0;
                }
                else {
                    p -= 1;
                }
            },
            Instruction::Increment(amount) => buffer[p] = buffer[p].wrapping_add(amount as u8),
            Instruction::Decrement(amount) => buffer[p] = buffer[p].wrapping_sub(amount as u8),
            Instruction::Write => write!(&mut out, "{}", buffer[p] as char).expect("Could not output"),
            Instruction::Read => {
                let chr = io::stdin().bytes().next();
                if chr.is_none() {
                    buffer[p] = 0;
                }
                else {
                    buffer[p] = chr.unwrap().expect("Could not read input");
                }
            },
            Instruction::JumpForwardIfZero {ref mut matching} => {
                if buffer[p] == 0 {
                    i = matching.unwrap_or_else(|| {
                        let m = find_matching(&program, i - 1) + 1;
                        *matching = Some(m);
                        m
                    });
                }
            },
            Instruction::JumpBackwardUnlessZero {matching} => {
                if buffer[p] != 0 {
                    i = matching;
                }
            },
        }

        if debug {
            println_stderr!("{{\"lastInstructionIndex\": {}, \"lastInstruction\": \"{}\", \"currentPointer\": {}, \"memory\": \"{}\"}}", i-1, instr, p,
                buffer.iter().fold(String::new(), |acc, v| format!("{} {}", acc, v)));
        }

        thread::sleep(Duration::from_millis(delay));
    }
}

/// Finds the matching '[' or ']' for the given position within the program
/// panics if a match is not found
fn find_matching(program: &Vec<Instruction>, start: usize) -> usize {
    let direction: isize = match program[start] {
        Instruction::JumpForwardIfZero { .. } => 1,
        Instruction::JumpBackwardUnlessZero { .. } => -1,
        _ => unreachable!(),
    };

    let mut count = direction;
    let mut current = start;
    loop {
        if (direction < 0 && current == 0) || (direction > 0 && current >= program.len() - 1) {
            panic!("Could not find matching parenthesis for instruction {}", start);
        }
        current = (current as isize + direction) as usize;
        let c = program[current];

        count = match c {
            Instruction::JumpForwardIfZero { .. } => count + 1,
            Instruction::JumpBackwardUnlessZero { .. } => count - 1,
            _ => count,
        };

        if count == 0 {
            break;
        }
    }

    current
}
