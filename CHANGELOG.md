# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]
### Added
- Tests! Lots of tests! The code is now nicely split up between modules too.

### Fixed
- Fixed a bug that was resulting in some cases of mismatched jump instructions
  not being reported as an error.

## [1.2.0] - 2017-04-19
### Added
- Optimizations on by default
  - This is an interpreter designed to run code fast, simulating a brainfuck
    turing machine exactly is not necessarily in service of that
  - This change resulted in `examples/mandel.bf` dropping from 1m46s to 0m39s
    on @sunjay's machine. Huge improvement!
  - See more discussion about this in [#4](https://github.com/brain-lang/brainfuck/issues/4)
- Optimizations can be turned off with the `-O0` command line flag
  - Use this to produce the most accurate simulation of a brainfuck turing
    machine
- Optimizations are **off** by default when `--debug` is passed as a command
  line argument
  - To force optimizations in debug mode, use the `-O1` flag

## [1.1.2] - 2017-04-18
### Added
- Benchmark (run with `cargo bench`)
  - This will now become part of the PR process
  - Performance is very important in an interpreter
- Continuous integration with [Travis CI](https://travis-ci.org/brain-lang/brainfuck)

## Changed
- Updated dependencies to latest versions
- Some internal code moved around or became more generic in order to support
  adding the benchmark, but overall nothing notable changed.

## [1.1.1] - 2017-02-26
### Changed
- Updated dependencies to latest version

## [1.1.0] - 2017-02-26
### Performance
- ~23.7% speed up by lazily caching jump positions in instructions list

The `examples/mandel.bf` program went from 2m19s to 1m46s reproducibly on
@sunjay's machine.

## 1.0.0 - 2017-02-12
### Added
- Stable, reasonably performant brainfuck interpreter implementing all aspects
  of the specification

[Unreleased]: https://github.com/brain-lang/brainfuck/compare/v1.2.0...HEAD
[1.2.0]: https://github.com/brain-lang/brainfuck/compare/v1.1.2...v1.2.0
[1.1.2]: https://github.com/brain-lang/brainfuck/compare/v1.1.0...v1.1.2
[1.1.1]: https://github.com/brain-lang/brainfuck/compare/v1.1.0...v1.1.1
[1.1.0]: https://github.com/brain-lang/brainfuck/compare/v1.0.0...v1.1.0
