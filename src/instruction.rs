use std::fmt;

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
        f.write_str(match *self {
            Instruction::Right(n) => format_instruction(">", n),
            Instruction::Left(n) => format_instruction("<", n),
            Instruction::Increment(n) => format_instruction("+", n),
            Instruction::Decrement(n) => format_instruction("-", n),
            Instruction::Write => ".".to_owned(),
            Instruction::Read => ",".to_owned(),
            Instruction::JumpForwardIfZero { .. } => "[".to_owned(),
            Instruction::JumpBackwardUnlessZero { .. } => "]".to_owned(),
        }.as_ref())
    }
}

#[inline]
fn format_instruction(instr: &str, n: usize) -> String {
    format!("{}{}", instr, if n > 1 {format!("{}", n)} else {"".to_owned()})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display() {
        assert_eq!(Instruction::Right(1).to_string(), ">");
        assert_eq!(Instruction::Right(2).to_string(), ">>");
        assert_eq!(Instruction::Right(5).to_string(), ">>>>>");

        assert_eq!(Instruction::Left(1).to_string(), "<");
        assert_eq!(Instruction::Left(2).to_string(), "<<");
        assert_eq!(Instruction::Left(5).to_string(), "<<<<<");

        assert_eq!(Instruction::Increment(1).to_string(), "+");
        assert_eq!(Instruction::Increment(2).to_string(), "++");
        assert_eq!(Instruction::Increment(5).to_string(), "+++++");

        assert_eq!(Instruction::Decrement(1).to_string(), "-");
        assert_eq!(Instruction::Decrement(2).to_string(), "--");
        assert_eq!(Instruction::Decrement(5).to_string(), "-----");

        assert_eq!(Instruction::Write.to_string(), ".");
        assert_eq!(Instruction::Read.to_string(), ",");

        assert_eq!(Instruction::JumpForwardIfZero {matching: None}.to_string(), "[");
        assert_eq!(Instruction::JumpBackwardUnlessZero {matching: 0}.to_string(), "]");
    }
}
