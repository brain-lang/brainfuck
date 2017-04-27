use std::io::{Read, Write};
use std::collections::VecDeque;

use super::Instruction;

pub struct Interpreter<I: Read, O: Write> {
    // Input stream to read from
    inp: I,
    // Output stream to write to
    out: O,
    // The brainfuck "tape" made of memory "cells" which are one byte in size
    memory: VecDeque<u8>,
    // The current index in the memory known as the "pointer"
    pointer: usize,
    program: Vec<Instruction>,
    // The instruction index in the current program
    next_instruction: usize,
}

impl<I: Read, O: Write> Interpreter<I, O> {
    pub fn new(inp: I, out: O) -> Interpreter<I, O> {
        Interpreter {
            inp: inp,
            out: out,
            memory: {
                let mut m = VecDeque::new();
                // Make sure there is at least one cell to begin with
                m.push_back(0u8);
                m
            },
            pointer: 0,
            program: Vec::new(),
            next_instruction: 0,
        }
    }

    pub fn memory(&self) -> &VecDeque<u8> {
        &self.memory
    }

    pub fn current_pointer(&self) -> usize {
        self.pointer
    }

    pub fn next_instruction(&self) -> usize {
        self.next_instruction
    }

    pub fn load_program(&mut self, program: Vec<Instruction>) {
        self.program.extend(program);
    }

    pub fn interpret(mut self, program: Vec<Instruction>) {
        self.load_program(program);
        // consume the entire program by evaluating it
        while let Some(_) = self.next() {}
    }
}

impl<I: Read, O: Write> Iterator for Interpreter<I, O> {
    type Item = Instruction;

    fn next(&mut self) -> Option<Instruction> {
        if self.next_instruction >= self.program.len() {
            return None;
        }
        let mut instr = self.program[self.next_instruction];
        self.next_instruction += 1;

        match instr {
            Instruction::Right(amount) => {
                self.pointer += amount;
                while self.pointer >= self.memory.len() {
                    self.memory.push_back(0u8);
                }
            },
            Instruction::Left(amount) => {
                if amount > self.pointer {
                    for _ in 0..(amount - self.pointer) {
                        self.memory.push_front(0u8);
                    }
                    self.pointer = 0;
                }
                else {
                    self.pointer -= amount;
                }
            },
            Instruction::Increment(amount) => self.memory[self.pointer] = self.memory[self.pointer].wrapping_add(amount as u8),
            Instruction::Decrement(amount) => self.memory[self.pointer] = self.memory[self.pointer].wrapping_sub(amount as u8),
            Instruction::Write => self.out.write_all(&[self.memory[self.pointer]]).expect("Could not output"),
            Instruction::Read => {
                let mut inmemory: [u8; 1] = [0];
                let res = self.inp.read_exact(&mut inmemory[0..1]);
                if res.is_ok() {
                    self.memory[self.pointer] = inmemory[0];
                }
                else {
                    self.memory[self.pointer] = 0;
                }
            },
            Instruction::JumpForwardIfZero {ref mut matching} => {
                if self.memory[self.pointer] == 0 {
                    self.next_instruction = matching.unwrap_or_else(|| fill_matching(&mut self.program, self.next_instruction - 1));
                }
            },
            Instruction::JumpBackwardUnlessZero {matching} => {
                if self.memory[self.pointer] != 0 {
                    self.next_instruction = matching;
                }
            },
        }

        Some(instr)
    }
}

/// Finds the matching ']' for the given '[' located at `start`
/// Designed to fill in any JumpForwardIfZero instructions found along the way
/// so this function doesn't need to be called needlessly.
///
/// # Panics
/// Panics if a match is not found
fn fill_matching(mut program: &mut Vec<Instruction>, start: usize) -> usize {
    use super::MAX_NESTED_JUMPS;
    let mut jumps = VecDeque::with_capacity(MAX_NESTED_JUMPS);

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
            // If we don't grow the memory properly, this will fail
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
    fn nested_loops() {
        // The result of this is x * y * z
        // Note that 13 * 15 * 2 = 390 which overflows since 390 > 255
        let x = 13;
        let y = 15;
        let z = 2;
        assert_eq!(test_interpret_output(vec![
            Increment(x),
            JumpForwardIfZero {matching: None},
            Right(1),
            Increment(y),
            JumpForwardIfZero {matching: None},
            Right(1),
            Increment(z),
            Left(1),
            Decrement(1),
            JumpBackwardUnlessZero {matching: 5},
            Left(1),
            Decrement(1),
            JumpBackwardUnlessZero {matching: 2},
            Right(2),
            Write,
        ]), vec![(x * y * z) as u8]);
    }

    #[test]
    fn hello_world() {
        use super::super::{precompile, OptimizationLevel};

        // hello world program from examples/hello-world.bf
        let source: Vec<u8> = include_bytes!("../examples/hello-world.bf").to_vec();
        let program = precompile(source.iter(), OptimizationLevel::Off);
        assert_eq!(test_interpret_output(program),
            b"Hello World!\n");

        let program = precompile(source.iter(), OptimizationLevel::Speed);
        assert_eq!(test_interpret_output(program),
            b"Hello World!\n");
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
        Interpreter::new(&mut inp, &mut out).interpret(program);
        out
    }
}
