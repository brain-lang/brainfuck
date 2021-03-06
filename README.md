# brainfuck

[![Crates.io](https://img.shields.io/crates/v/brain-brainfuck.svg)](https://crates.io/crates/brain-brainfuck)
[![Crates.io](https://img.shields.io/crates/l/brain-brainfuck.svg)](https://crates.io/crates/brain-brainfuck)
[![Build Status](https://travis-ci.org/brain-lang/brainfuck.svg?branch=master)](https://travis-ci.org/brain-lang/brainfuck)
[![Build status](https://ci.appveyor.com/api/projects/status/ap7x7tmrwjj4v6xa?svg=true)](https://ci.appveyor.com/project/sunjay/brainfuck)
[![Dependency Status](https://dependencyci.com/github/brain-lang/brainfuck/badge)](https://dependencyci.com/github/brain-lang/brainfuck)
[![codecov](https://codecov.io/gh/brain-lang/brainfuck/branch/master/graph/badge.svg)](https://codecov.io/gh/brain-lang/brainfuck)
[![Gitter](https://img.shields.io/gitter/room/brain-lang/brain.svg)](https://gitter.im/brain-lang/brain)

[brainfuck][brainfuck] is an esoteric programming language with 8 very simple
instructions.

The [brain][brain] compiler only officially targets this brainfuck interpreter.
You may experience varying results with other brainfuck interpreters/compilers.
There really isn't a definitive spec on how brainfuck should behave so it is
just easier to have a static compilation target that won't vary in how it
behaves.

A brainfuck specification for this brainfuck interpreter is available in
the [brainfuck.md](brainfuck.md) file.

## Installation

For people just looking to use brainfuck, the easiest way to get it right now
is to first install the [Cargo package manager][cargo-install] for the
Rust programming language.

Then in your terminal run:

```
cargo install brain-brainfuck
```

If you are upgrading from a previous version, run:

```
cargo install brain-brainfuck --force
```

## Usage

**For anyone just looking to run brainfuck code with the interpreter:**

1. Follow the installation instructions above
2. Run `brainfuck yourfile.bf` to run a brainfuck interpreter which will
   run your generated brainfuck code

**For anyone looking to work with the interpreter source code:**

To run brainfuck programs:
```
cargo run --bin brainfuck -- filename
```
where `filename` is the brainfuck program you want to run

Make sure you have [rust][rust] and cargo (comes with rust) installed.

## Examples

There are various brainfuck examples in the `examples/` directory which you can
run with the brainfuck interpreter using the instructions above.

[rust]: https://www.rust-lang.org/
[brain]: https://github.com/brain-lang/brain/
[brainfuck]: http://www.muppetlabs.com/~breadbox/bf/
[cargo-install]: https://crates.io/install
