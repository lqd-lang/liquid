# Liquid
A general purpose, programming language that combines powerful macros, with simple, and easy to learn syntax.

## Installing
To install, clone this repository, then run

`cargo install --path .`

This will compile from scratch (~1min), and add binaries to PATH. This will require `cargo` and `clang` or `gcc` to be installed, and added to PATH ([see below](#note)).

### Note
`lqdc` uses `clang` to compile and link by default. To use `gcc` instead, disable default features (`--no-default-features`) and enable the gcc feature flag (`-F gcc`)

## Features
- Link with external functions and export your own
- Function calls, basic math and boolean expressions

## Contributing
It's just plain old cargo, It would also be beneficial to contribute to Codegem.

### Todo
- Strings
- Standard library
- Macros
- Cranelift backend

## Under the hood
The primary code generation backend used by lqdc (the compiler) is [Codegem](https://github.com/code-gem/codegem). To compile the generated assembly, it is just passed to `clang` or `gcc`, over the command line.