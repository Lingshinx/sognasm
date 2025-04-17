# Sognasm

A simple bytecode interpreter for Sognac, inspired by [Cognac](https://github.com/cognate-lang/cognate)
built using Rust and [pest parser library](https://github.com/pest-parser/pest)

Childish though it is, Sogansm is decided to be shown off to my friends
as my first programming language project.

Thanks for checking out the early version of my project.
Feel free to experiment and share your feedback
I appreciate all forms of constructive suggestion from you.

By the way, I'm a beginner of Rust.
Perhaps there is full of non-standard code that I'm unaware of.
Please let me know so that the next one would not
smash their precious keyboard into their head.

## Usage

```txt
Usage: sognasm [OPTIONS] <source>

Arguments:
  <source>  

Options:
  -p, --print          Print the stack while running
  -s, --speed <speed>  the period of each operation 
                       when print is on (unit:ms)  [default: 100ms]
  -c, --code           Print the code
  -h, --help           Print help
  -V, --version        Print version

```

```bash
sognasm source.sasm
```

## Tutorial

[learn Sognasm in Y minutes](./blob/main/LearnSasmInYminutes.sasm)

## Todo

- [] add more system calls
- [] Implement binary bytecode output
