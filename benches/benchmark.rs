#![feature(test)]
extern crate test;

#[macro_use]
extern crate lazy_static;

extern crate brainfuck;

use test::Bencher;

use brainfuck::{precompile, Instruction, OptimizationLevel};

lazy_static! {
    // This program is trivial to run in both size and speed
    static ref TRIVIAL_SOURCE: Vec<u8> = include_bytes!("../examples/hello-world.bf").to_vec();
    // This is a very large program in size
    static ref LARGE_SOURCE: Vec<u8> = include_bytes!("../examples/hanoi.bf").to_vec();
    // This program is absurdly huge and is mostly a corner case for the time being
    static ref HUGE_SOURCE: Vec<u8> = include_bytes!("../examples/LostKingdom.bf").to_vec();
    // This program is reasonably small, non-trivial to run, and simple overall
    static ref SIMPLE_SOURCE: Vec<u8> = include_bytes!("../examples/99bottles.bf").to_vec();
    // This program, while only relatively large, takes a long time to run
    static ref SLOW_SOURCE: Vec<u8> = include_bytes!("../examples/mandel.bf").to_vec();
}

struct NullWrite;

impl std::io::Write for NullWrite {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn interpret(program: Vec<Instruction>) {
    let mut inp: &[u8] = &[];
    brainfuck::interpret(&mut inp, NullWrite, program, |_| {});
}

#[bench]
fn b01_compile_trivial(b: &mut Bencher) {
    b.iter(|| precompile(TRIVIAL_SOURCE.iter(), OptimizationLevel::Off));
}

#[bench]
fn b01_compile_trivial_opt(b: &mut Bencher) {
    b.iter(|| precompile(TRIVIAL_SOURCE.iter(), OptimizationLevel::Speed));
}

#[bench]
fn b02_compile_large(b: &mut Bencher) {
    b.iter(|| precompile(LARGE_SOURCE.iter(), OptimizationLevel::Off));
}

#[bench]
fn b02_compile_large_opt(b: &mut Bencher) {
    b.iter(|| precompile(LARGE_SOURCE.iter(), OptimizationLevel::Speed));
}

#[bench]
fn b03_compile_huge(b: &mut Bencher) {
    b.iter(|| precompile(HUGE_SOURCE.iter(), OptimizationLevel::Off));
}

#[bench]
fn b03_compile_huge_opt(b: &mut Bencher) {
    b.iter(|| precompile(HUGE_SOURCE.iter(), OptimizationLevel::Speed));
}

#[bench]
fn b04_compile_simple(b: &mut Bencher) {
    b.iter(|| precompile(SIMPLE_SOURCE.iter(), OptimizationLevel::Off));
}

#[bench]
fn b04_compile_simple_opt(b: &mut Bencher) {
    b.iter(|| precompile(SIMPLE_SOURCE.iter(), OptimizationLevel::Speed));
}

#[bench]
fn b05_compile_slow(b: &mut Bencher) {
    b.iter(|| precompile(SLOW_SOURCE.iter(), OptimizationLevel::Off));
}

#[bench]
fn b05_compile_slow_opt(b: &mut Bencher) {
    b.iter(|| precompile(SLOW_SOURCE.iter(), OptimizationLevel::Speed));
}

#[bench]
fn b06_interpret_trivial(b: &mut Bencher) {
    let program = precompile(TRIVIAL_SOURCE.iter(), OptimizationLevel::Off);
    b.iter(|| interpret(program.clone()));
}

#[bench]
fn b06_interpret_trivial_opt(b: &mut Bencher) {
    let program = precompile(TRIVIAL_SOURCE.iter(), OptimizationLevel::Speed);
    b.iter(|| interpret(program.clone()));
}

#[bench]
fn b07_interpret_simple(b: &mut Bencher) {
    let program = precompile(SIMPLE_SOURCE.iter(), OptimizationLevel::Off);
    b.iter(|| interpret(program.clone()));
}

#[bench]
fn b07_interpret_simple_opt(b: &mut Bencher) {
    let program = precompile(SIMPLE_SOURCE.iter(), OptimizationLevel::Speed);
    b.iter(|| interpret(program.clone()));
}

#[bench]
#[ignore]
fn b08_interpret_slow(b: &mut Bencher) {
    let program = precompile(SLOW_SOURCE.iter(), OptimizationLevel::Off);
    b.iter(|| interpret(program.clone()));
}

#[bench]
fn b08_interpret_slow_opt(b: &mut Bencher) {
    let program = precompile(SLOW_SOURCE.iter(), OptimizationLevel::Speed);
    b.iter(|| interpret(program.clone()));
}
