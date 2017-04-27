use std::io::{Read, Write};
use std::collections::VecDeque;

use super::Instruction;

pub struct InterpreterState<'a> {
    /// Index of the next instruction to be run in the program
    pub next_instruction: usize,
    /// The last instruction that was run (only None at the start of the program)
    pub last_instruction: Option<Instruction>,
    /// The current "pointer" value that represents the current cell in memory
    pub current_pointer: usize,
    /// The entire memory buffer (read-only)
    pub memory: &'a VecDeque<u8>,
}

/// callback is called once before any instructions, and after each instruction
pub fn interpret<I, O, F>(mut inp: I, mut out: O, mut program: Vec<Instruction>, mut callback: F)
    where I: Read, O: Write,
          F: FnMut(InterpreterState) {

    let mut buffer: VecDeque<u8> = VecDeque::new();
    // Make sure there is at least one cell to begin with
    buffer.push_back(0u8);

    // pointer is the position "pointer" in the buffer
    let mut pointer: usize = 0;
    // next_instruction is the instruction index in the program
    let mut next_instruction: usize = 0;

    callback(InterpreterState {
        next_instruction: next_instruction,
        last_instruction: None,
        current_pointer: pointer,
        memory: &buffer,
    });
    loop {
        if next_instruction >= program.len() {
            break;
        }
        let mut instr = program[next_instruction];
        next_instruction += 1;

        match instr {
            Instruction::Right(amount) => {
                pointer += amount;
                while pointer >= buffer.len() {
                    buffer.push_back(0u8);
                }
            },
            Instruction::Left(amount) => {
                if amount > pointer {
                    for _ in 0..(amount - pointer) {
                        buffer.push_front(0u8);
                    }
                    pointer = 0;
                }
                else {
                    pointer -= amount;
                }
            },
            Instruction::Increment(amount) => buffer[pointer] = buffer[pointer].wrapping_add(amount as u8),
            Instruction::Decrement(amount) => buffer[pointer] = buffer[pointer].wrapping_sub(amount as u8),
            Instruction::Write => out.write_all(&[buffer[pointer]]).expect("Could not output"),
            Instruction::Read => {
                let mut inbuffer: [u8; 1] = [0];
                let res = inp.read_exact(&mut inbuffer[0..1]);
                if res.is_ok() {
                    buffer[pointer] = inbuffer[0];
                }
                else {
                    buffer[pointer] = 0;
                }
            },
            Instruction::JumpForwardIfZero {ref mut matching} => {
                if buffer[pointer] == 0 {
                    next_instruction = matching.unwrap_or_else(|| fill_matching(&mut program, next_instruction - 1));
                }
            },
            Instruction::JumpBackwardUnlessZero {matching} => {
                if buffer[pointer] != 0 {
                    next_instruction = matching;
                }
            },
        }

        callback(InterpreterState {
            next_instruction: next_instruction,
            last_instruction: Some(instr),
            current_pointer: pointer,
            memory: &buffer,
        });
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
        interpret(&mut inp, &mut out, program, |_| {});
        out
    }
}
