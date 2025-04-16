mod argus;
mod assemble;
mod command;
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

    // builder.display(1);

    let asm = Asm::from(builder.clone());

    Runtime::run(asm);
}
