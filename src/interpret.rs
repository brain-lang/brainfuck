use std::io;
use std::io::prelude::*;
use std::collections::VecDeque;
use std::thread;
use std::time::Duration;

use super::Instruction;

pub fn interpret<OutFile>(mut program: Vec<Instruction>, mut out: OutFile, debug: bool, delay: u64)
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
                    p -= amount;
                }
            },
            Instruction::Increment(amount) => buffer[p] = buffer[p].wrapping_add(amount as u8),
            Instruction::Decrement(amount) => buffer[p] = buffer[p].wrapping_sub(amount as u8),
            Instruction::Write => out.write_all(&[buffer[p]]).expect("Could not output"),
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
                    i = matching.unwrap_or_else(|| fill_matching(&mut program, i - 1));
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

/// Finds the matching ']' for the given '[' located at `start`
/// Designed to fill in any JumpForwardIfZero instructions found along the way
/// so this function doesn't need to be called needlessly.
///
/// # Panics
/// Panics if a match is not found
fn fill_matching(mut program: &mut Vec<Instruction>, start: usize) -> usize {
    let mut jumps = VecDeque::new();

    let program_size = program.len();
    let mut current = start;
    loop {
        if current >= program_size {
            panic!("Mismatched `[` instruction");
        }

        match program[current] {
            Instruction::JumpForwardIfZero { .. } => jumps.push_back(current),
            Instruction::JumpBackwardUnlessZero { .. } => {
                let last_forward = jumps.pop_back().expect("Mismatched `]` instruction");
                match program[last_forward] {
                    Instruction::JumpForwardIfZero {ref mut matching} => {
                        debug_assert!(matching.is_none(),
                            "matching was already set which means this function ran needlessly");
                        *matching = Some(current + 1);
                    },
                    _ => unreachable!(),
                }
            },
            _ => {},
        };

        if jumps.is_empty() {
            break;
        }

        current += 1;
    }

    current + 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::Instruction::*;

    #[test]
    fn basic_movements() {
        // Test to make sure basic movements are working
        let mut out = Vec::new();
        interpret(vec![
            Increment(1),
            Right(1),
            Increment(2),
            Right(1),
            Increment(3),
            Write,
            Left(1),
            Write,
            Left(1),
            Write,
        ], &mut out, false, 0);

        assert_eq!(out, vec![3, 2, 1])
    }

    #[test]
    fn wrapping() {
        let mut out = Vec::new();
        interpret(vec![
            Write,
            Decrement(1),
            Write,
            Increment(2),
            Write,
            Increment(256),
            Write,
            Decrement(255),
            Write,
            Decrement(1),
            Write,
            Decrement(1),
            Write,
            Decrement(1),
            Write,
            Increment(1),
            Write,
            Increment(1),
            Write,
            Increment(255 * 2),
            Write,
            Decrement(255 * 2),
            Write,
        ], &mut out, false, 0);

        assert_eq!(out, vec![0, 255, 1, 1, 2, 1, 0, 255, 0, 1, 255, 1])
    }

    #[test]
    fn move_left_past_zero() {
        // These movements are designed to cause problems if the move instructions are not
        // implemented correctly. If all goes well, the code should end up back at the same spot
        // in the brainfuck tape.
        let mut out = Vec::new();
        interpret(vec![
            Increment(1),
            Right(1),
            Left(4),
            Right(3),
            Left(3),
            Right(3),
            Increment(1),
            Write,
        ], &mut out, false, 0);

        assert_eq!(out, vec![2])
    }

    #[test]
    fn move_right_past_capacity() {
        let mut out = Vec::new();
        interpret(vec![
            Increment(1),
            Right(10),
            // If we don't grow the buffer properly, this will fail
            Increment(1),
            Left(20),
            Right(20),
            Left(10),
            Right(15),
            Left(15),
            Increment(1),
            Write,
        ], &mut out, false, 0);

        assert_eq!(out, vec![2])
    }

    #[test]
    fn skip_first_loop() {
        // Any complient brainfuck implementation should always skip the first loop no matter what
        // instructions are in it
        let mut out = Vec::new();
        interpret(vec![
            JumpForwardIfZero {matching: None},
            Right(1),
            Left(1),
            Increment(1),
            Decrement(2),
            Write,
            //Read,
            JumpBackwardUnlessZero {matching: 1},
        ], &mut out, false, 0);

        assert_eq!(out, vec![])
    }
}
