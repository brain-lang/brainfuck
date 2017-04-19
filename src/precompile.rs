use std::collections::VecDeque;
use std::iter::repeat;

use super::{OptimizationLevel, Instruction};

// We typically don't expect to see more than this many levels of nested jumps
// based on an analysis of some brainfuck programs
// More than this many is okay, we just preallocate this many to avoid the cost of
// allocating later.
const MAX_NESTED_JUMPS: usize = 15;

/// Precompile the program into an appropriate in-memory representation
pub fn precompile<'a, I>(bytes: I, opt: OptimizationLevel) -> Vec<Instruction>
    where I: IntoIterator<Item=&'a u8> {
    use self::Instruction::*;

    let should_group = opt == OptimizationLevel::Speed;

    let mut input = bytes.into_iter().peekable();
    let mut instructions = VecDeque::new();
    while let Some(next_ch) = input.next() {
        let mut count = 1;
        if should_group {
            while let Some(&ch) = input.peek() {
                if ch == next_ch {
                    count += 1;
                    input.next();
                }
                else {
                    break;
                }
            }
        }

        match *next_ch {
            b'>' => instructions.push_back(Instruction::Right(count)),
            b'<' => instructions.push_back(Instruction::Left(count)),
            b'+' => instructions.push_back(Instruction::Increment(count)),
            b'-' => instructions.push_back(Instruction::Decrement(count)),
            b'.' => instructions.extend(repeat(Instruction::Write).take(count)),
            b',' => instructions.extend(repeat(Instruction::Read).take(count)),
            b'[' => instructions.extend(repeat(Instruction::JumpForwardIfZero {matching: None}).take(count)),
            b']' => instructions.extend(repeat(Instruction::JumpBackwardUnlessZero {matching: 0}).take(count)),
            _ => continue,
        };
    }

    // Used to keep track of jump instructions so we can cache where they jump to so that
    // jumping is faster when this code runs
    let mut jump_stack = VecDeque::with_capacity(MAX_NESTED_JUMPS);

    // We have to do this in two separate steps or else the indexes would be all wrong
    let instructions = instructions.into_iter().enumerate().map(|(i, c)|
        match c {
            JumpForwardIfZero { .. } => {
                jump_stack.push_back(i);
                c
            },
            JumpBackwardUnlessZero { .. } => {
                let matching = jump_stack.pop_back()
                    .expect("Mismatched jump instruction");
                // When jumping backward, jump one further than the matching [ instruction
                // This avoids an extra jump test
                JumpBackwardUnlessZero {matching: matching + 1}
            }
            _ => c,
        }
    ).collect();

    if !jump_stack.is_empty() {
        panic!("Mismatched jump instruction");
    }

    instructions
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::Instruction::*;


    #[test]
    fn optimization_levels() {
        // A program with no groups of adjacent instructions
        const NO_GROUPS: &'static [u8] = b"><[.][.][,],+-.>-,+";

        let no_groups_instructions = vec![
            Right(1),
            Left(1),
            JumpForwardIfZero {matching: None},
            Write,
            JumpBackwardUnlessZero {matching: 3},
            JumpForwardIfZero {matching: None},
            Write,
            JumpBackwardUnlessZero {matching: 6},
            JumpForwardIfZero {matching: None},
            Read,
            JumpBackwardUnlessZero {matching: 9},
            Read,
            Increment(1),
            Decrement(1),
            Write,
            Right(1),
            Decrement(1),
            Read,
            Increment(1),
        ];

        // If there are no groups in the instructions, it should be the same with or without
        // optimizations
        test_precompile(NO_GROUPS, OptimizationLevel::Off, no_groups_instructions.clone());
        test_precompile(NO_GROUPS, OptimizationLevel::Speed, no_groups_instructions.clone());

        // A program with some adjacent groups of instructions as well as some single, ungrouped
        // instructions
        // Some of these instructions like writes, reads and jumps are not meant to be grouped
        // This test also tests to make sure that they are left separate when adjacent in the input
        const SOME_GROUPS: &'static [u8] = b"><>>>++-[[[<+,-.>]]]...,,,,++++---<<<>>>[>>.,]";
        // If there are groups, or even a mix of groups/non-groups, the output should be different
        test_precompile(SOME_GROUPS, OptimizationLevel::Off, vec![
            Right(1),
            Left(1),
            Right(1),
            Right(1),
            Right(1),
            Increment(1),
            Increment(1),
            Decrement(1),
            JumpForwardIfZero {matching: None},
            JumpForwardIfZero {matching: None},
            JumpForwardIfZero {matching: None},
            Left(1),
            Increment(1),
            Read,
            Decrement(1),
            Write,
            Right(1),
            JumpBackwardUnlessZero {matching: 11},
            JumpBackwardUnlessZero {matching: 10},
            JumpBackwardUnlessZero {matching: 9},
            Write,
            Write,
            Write,
            Read,
            Read,
            Read,
            Read,
            Increment(1),
            Increment(1),
            Increment(1),
            Increment(1),
            Decrement(1),
            Decrement(1),
            Decrement(1),
            Left(1),
            Left(1),
            Left(1),
            Right(1),
            Right(1),
            Right(1),
            JumpForwardIfZero {matching: None},
            Right(1),
            Right(1),
            Write,
            Read,
            JumpBackwardUnlessZero {matching: 41},
        ]);
        test_precompile(SOME_GROUPS, OptimizationLevel::Speed, vec![
            Right(1),
            Left(1),
            Right(3),
            Increment(2),
            Decrement(1),
            JumpForwardIfZero {matching: None},
            JumpForwardIfZero {matching: None},
            JumpForwardIfZero {matching: None},
            Left(1),
            Increment(1),
            Read,
            Decrement(1),
            Write,
            Right(1),
            JumpBackwardUnlessZero {matching: 8},
            JumpBackwardUnlessZero {matching: 7},
            JumpBackwardUnlessZero {matching: 6},
            Write,
            Write,
            Write,
            Read,
            Read,
            Read,
            Read,
            Increment(4),
            Decrement(3),
            Left(3),
            Right(3),
            JumpForwardIfZero {matching: None},
            Right(2),
            Write,
            Read,
            JumpBackwardUnlessZero {matching: 29},
        ]);
    }

    #[test]
    fn skip_unrecognized() {
        test_precompile(b"
        [all of these should get skipped]

        >>>
        skip me skip me
        >>>
        <<<
        skip me skip me
        >++
        skip me skip me
        <--

        skip me skip me

        [#################################### SKIP ME #########################]
        .hellooooooooooooooooooooooooooo
        ,what's up, dog. did you notice the instructions I just hid
        ", OptimizationLevel::Speed, vec![
            JumpForwardIfZero {matching: None},
            JumpBackwardUnlessZero {matching: 1},
            Right(3),
            Right(3),
            Left(3),
            Right(1),
            Increment(2),
            Left(1),
            Decrement(2),
            JumpForwardIfZero {matching: None},
            JumpBackwardUnlessZero {matching: 10},
            Write,
            Read,
            Read,
            Write,
        ]);
    }

    #[test]
    fn over_max_expected_jumps() {
        let mut program = Vec::new();
        let mut expected = Vec::new();
        let mut program_tail = Vec::new();
        let mut expected_tail = Vec::new();
        for i in 1..(MAX_NESTED_JUMPS+5) {
            program.push(b'[');
            expected.push(JumpForwardIfZero {matching: None});
            program_tail.insert(0, b']');
            expected_tail.insert(0, JumpBackwardUnlessZero {matching: i});
        }

        program.extend(program_tail);
        expected.extend(expected_tail);

        // This should not depend on the OptimizationLevel
        test_precompile(&program, OptimizationLevel::Off, expected.clone());
        test_precompile(&program, OptimizationLevel::Speed, expected.clone());
    }

    #[test]
    #[should_panic(expected = "Mismatched jump instruction")]
    fn mismatched_jumps1() {
        precompile(b"[".iter(), OptimizationLevel::Off);
    }
    #[test]
    #[should_panic(expected = "Mismatched jump instruction")]
    fn mismatched_jumps2() {
        precompile(b"]".iter(), OptimizationLevel::Off);
    }
    #[test]
    #[should_panic(expected = "Mismatched jump instruction")]
    fn mismatched_jumps3() {
        precompile(b"][".iter(), OptimizationLevel::Off);
    }
    #[test]
    #[should_panic(expected = "Mismatched jump instruction")]
    fn mismatched_jumps4() {
        precompile(b"][]".iter(), OptimizationLevel::Off);
    }
    #[test]
    #[should_panic(expected = "Mismatched jump instruction")]
    fn mismatched_jumps5() {
        precompile(b"][][".iter(), OptimizationLevel::Off);
    }
    #[test]
    #[should_panic(expected = "Mismatched jump instruction")]
    fn mismatched_jumps6() {
        precompile(b"[]][]".iter(), OptimizationLevel::Off);
    }
    #[test]
    #[should_panic(expected = "Mismatched jump instruction")]
    fn mismatched_jumps7() {
        precompile(b"[][[]".iter(), OptimizationLevel::Off);
    }
    #[test]
    #[should_panic(expected = "Mismatched jump instruction")]
    fn mismatched_jumps8() {
        precompile(b"[][[]".iter(), OptimizationLevel::Off);
    }
    #[test]
    #[should_panic(expected = "Mismatched jump instruction")]
    fn mismatched_jumps9() {
        precompile(b"[][[]]]".iter(), OptimizationLevel::Off);
    }
    #[test]
    #[should_panic(expected = "Mismatched jump instruction")]
    fn mismatched_jumps10() {
        precompile(b"[[[[[[[[]]][][[[]]]]][[[]]]][[[[]]]]]".iter(), OptimizationLevel::Off);
    }
    #[test]
    #[should_panic(expected = "Mismatched jump instruction")]
    fn mismatched_jumps11() {
        precompile(b"[][]]".iter(), OptimizationLevel::Off);
    }

    fn test_precompile(input: &[u8], opt: OptimizationLevel, expected: Vec<Instruction>) {
        assert_eq!(precompile(input.iter(), opt), expected);
    }
}
