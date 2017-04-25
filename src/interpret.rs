use std::io::prelude::*;
use std::collections::VecDeque;
use std::thread;
use std::time::Duration;

use super::Instruction;

pub struct Interpreter<I: Read, O: Write, E: Write> {
    inp: I,
    out: O,
    err: E,
    debug: bool,
    delay: u64,
}

impl<I: Read, O: Write, E: Write> Interpreter<I, O, E> {
    pub fn new(inp: I, out: O, err: E, debug: bool, delay: u64) -> Interpreter<I, O, E> {
        Interpreter {inp: inp, out: out, err: err, debug: debug, delay: delay}
    }

    /// Constructs an Interpreter with just streams and using the default values for other
    /// properties
    pub fn from_streams(input: I, output: O, error: E) -> Interpreter<I, O, E> {
        Interpreter::new(input, output, error, false, 0)
    }

    pub fn interpret(mut self, mut program: Vec<Instruction>) {
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
                Instruction::Write => self.out.write_all(&[buffer[p]]).expect("Could not output"),
                Instruction::Read => {
                    let mut inbuffer: [u8; 1] = [0];
                    let res = self.inp.read_exact(&mut inbuffer[0..1]);
                    if res.is_ok() {
                        buffer[p] = inbuffer[0];
                    }
                    else {
                        buffer[p] = 0;
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

            if self.debug {
                writeln!(
                    &mut self.err,
                    "{{\"lastInstructionIndex\": {}, \"lastInstruction\": \"{}\", \"currentPointer\": {}, \"memory\": \"{}\"}}",
                    i-1, instr, p,
                    buffer.iter().fold(String::new(), |acc, v| format!("{} {}", acc, v))
                ).expect("failed printing to stderr");
            }

            thread::sleep(Duration::from_millis(self.delay));
        }
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
        assert_eq!(test_interpret_output(vec![
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
        ]), vec![3, 2, 1]);
    }

    #[test]
    fn wrapping() {
        assert_eq!(test_interpret_output(vec![
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
        ]), vec![0, 255, 1, 1, 2, 1, 0, 255, 0, 1, 255, 1]);
    }

    #[test]
    fn move_left_past_zero() {
        // These movements are designed to cause problems if the move instructions are not
        // implemented correctly. If all goes well, the code should end up back at the same spot
        // in the brainfuck tape.
        assert_eq!(test_interpret_output(vec![
            Increment(1),
            Right(1),
            Left(4),
            Right(3),
            Left(3),
            Right(3),
            Increment(1),
            Write,
        ]), vec![2]);
    }

    #[test]
    fn move_right_past_capacity() {
        assert_eq!(test_interpret_output(vec![
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
        ]), vec![2]);
    }

    #[test]
    fn skip_first_loop() {
        // Any complient brainfuck implementation should always skip the first loop no matter what
        // instructions are in it
        assert_eq!(test_interpret_output(vec![
            JumpForwardIfZero {matching: None},
            Right(1),
            Left(1),
            Increment(1),
            Decrement(2),
            Write,
            Read,
            JumpBackwardUnlessZero {matching: 1},
        ]), vec![]);
    }

    #[test]
    fn reading_input() {
        assert_eq!(test_interpret_with_input(vec![
            Write,
            Read,
            Write,
            Decrement(1),
            Write,
            Read,
            Write,
            Increment(1),
            Write,
            Right(1),
            Read, // reads should overwrite each other
            Read,
            Right(1),
            Read,
            Right(1),
            Read,
            Left(2),
            Write,
            Right(1),
            Write,
            Right(1),
            Write,
            Left(3), // should not have changed
            Write,
            Read, // All reads past the end of input should result in 0
            Write,
            Read,
            Read,
            Read,
            Read,
            Read,
            Read,
            Read,
            Write,
        ], &[ // input
            5,
            0,
            8,
            17,
            32,
            49,
        ]), vec![0, 5, 4, 0, 1, 17, 32, 49, 1, 0, 0]);
    }

    #[test]
    fn basic_looping() {
        // This loop increments cell index 1 using cell index 0 as a loop counter
        // The result is x * y
        let x = 13;
        let y = 15;
        assert_eq!(test_interpret_output(vec![
            Increment(x),
            JumpForwardIfZero {matching: None},
            Right(1),
            Increment(y),
            Left(1),
            Decrement(1),
            JumpBackwardUnlessZero {matching: 2},
            Right(1),
            Write,
        ]), vec![(x * y) as u8]);
    }

    #[test]
    #[should_panic(expected = "Mismatched `[` instruction")]
    fn mismatched_jumps() {
        test_interpret_output(vec![
            JumpForwardIfZero {matching: None},
        ]);
    }

    fn test_interpret_output(program: Vec<Instruction>) -> Vec<u8> {
        let inp: &[u8] = &[];
        test_interpret_with_input(program, inp)
    }

    fn test_interpret_with_input(program: Vec<Instruction>, mut inp: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut err = Vec::new();
        Interpreter::from_streams(&mut inp, &mut out, &mut err).interpret(program);

        // This way errors will get caught in the output testing
        out.extend(err);
        out
    }
}
