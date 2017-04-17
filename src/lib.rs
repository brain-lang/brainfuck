use std::fmt;
use std::io;
use std::io::prelude::*;
use std::collections::VecDeque;
use std::thread;
use std::time::Duration;

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
    Right,
    // "<" - decrement the pointer (move it to the "left")
    Left,
    // "+" - increment the byte at the pointer
    Increment,
    // "-" - decrement the byte at the pointer
    Decrement,
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
            Instruction::Right => '>',
            Instruction::Left => '<',
            Instruction::Increment => '+',
            Instruction::Decrement => '-',
            Instruction::Write => '.',
            Instruction::Read => ',',
            Instruction::JumpForwardIfZero { .. } => '[',
            Instruction::JumpBackwardUnlessZero { .. } => ']',
        })
    }
}

/// Precompile the program into an appropriate in-memory representation
pub fn precompile(bytes: Vec<u8>) -> Vec<Instruction> {
    let mut jump_stack = VecDeque::with_capacity(15);
    bytes.into_iter().map(|c|
        match c {
            b'>' => Some(Instruction::Right),
            b'<' => Some(Instruction::Left),
            b'+' => Some(Instruction::Increment),
            b'-' => Some(Instruction::Decrement),
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

pub fn interpret(program: Vec<Instruction>, debug: bool, delay: u64) {
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
            Instruction::Right => {
                p += 1;
                if p >= buffer.len() {
                    buffer.push_back(0u8);
                }
            },
            Instruction::Left => {
                if p == 0 {
                    buffer.push_front(0u8);
                }
                else {
                    p -= 1;
                }
            },
            Instruction::Increment => buffer[p] = buffer[p].wrapping_add(1),
            Instruction::Decrement => buffer[p] = buffer[p].wrapping_sub(1),
            Instruction::Write => print!("{}", buffer[p] as char),
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
