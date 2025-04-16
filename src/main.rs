mod argus;
mod assemble;
mod command;
mod error;
mod machine;
mod parser;
mod record;
mod runtime;
mod test;
mod util;
mod value;
use argus::Arguments;
use assemble::Asm;
use colored::Colorize;
use parser::AsmBuilder;
use runtime::Runtime;
use std::fs;

fn main() {
    let arguments = Arguments::new();
    let file = arguments.source();
    let content = match fs::read_to_string(file) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("{} {}", "[error]".red(), err);
            std::process::exit(1);
        }
    };

    let builder = match AsmBuilder::from_str(&content) {
        Ok(builder) => builder,
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    };

    match (arguments.is_print(), arguments.is_code()) {
        (true, true) => {
            let (asm, labels) = Asm::from_builder(builder);
            Runtime::run_printing_code(asm, arguments.speed(), labels);
        }
        (true, false) => {
            let asm = Asm::from(builder);
            Runtime::run_printing(asm, arguments.speed());
        }
        (false, true) => {
            builder.display(0);
        }
        (false, false) => {
            let asm = Asm::from(builder);
            Runtime::run(asm);
        }
    }
}
