mod instruction;
mod optlevel;
mod precompiler;
mod interpreter;
mod debug_format;

pub use instruction::*;
pub use optlevel::*;
pub use precompiler::*;
pub use interpreter::*;
pub use debug_format::*;

// We typically don't expect to see more than this many levels of nested jumps
// based on an analysis of some brainfuck programs
// More than this many is okay, we just preallocate this many to avoid the cost of
// reallocating later.
const MAX_NESTED_JUMPS: usize = 15;
