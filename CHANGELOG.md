# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]
### Added
- Nothing...yet!

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

[Unreleased]: https://github.com/brain-lang/brainfuck/compare/v1.1.1...HEAD
[1.1.1]: https://github.com/brain-lang/brainfuck/compare/v1.1.0...v1.1.1
[1.1.0]: https://github.com/brain-lang/brainfuck/compare/v1.0.0...v1.1.0
