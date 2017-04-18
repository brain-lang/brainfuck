#![feature(test)]
extern crate test;

#[macro_use]
extern crate lazy_static;

extern crate brainfuck;

use test::Bencher;

use brainfuck::{precompile, interpret};

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

#[bench]
fn compile_trivial(b: &mut Bencher) {
    b.iter(|| precompile(TRIVIAL_SOURCE.iter()));
}

#[bench]
fn compile_large(b: &mut Bencher) {
    b.iter(|| precompile(LARGE_SOURCE.iter()));
}

#[bench]
#[ignore]
fn compile_huge(b: &mut Bencher) {
    b.iter(|| precompile(HUGE_SOURCE.iter()));
}

#[bench]
fn compile_simple(b: &mut Bencher) {
    b.iter(|| precompile(SIMPLE_SOURCE.iter()));
}

#[bench]
fn compile_slow(b: &mut Bencher) {
    b.iter(|| precompile(SLOW_SOURCE.iter()));
}

#[bench]
fn interpret_simple(b: &mut Bencher) {
    b.iter(|| interpret(precompile(SIMPLE_SOURCE.iter()), false, 0));
}

#[bench]
fn interpret_slow(b: &mut Bencher) {
    b.iter(|| interpret(precompile(SLOW_SOURCE.iter()), false, 0));
}
