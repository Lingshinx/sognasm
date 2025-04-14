mod argus;
mod assemble;
mod command;
mod machine;
mod parser;
mod runtime;
mod test;
mod util;
mod value;
use argus::Arguments;
use assemble::Asm;
use colored::Colorize;
use parser::AsmBuilder;
use runtime::Runtime;
use std::{fs, io, time::Duration};
use text_io::read;

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

    let asm = Asm::from(builder.clone());

    for (k, v) in &builder.label_record {
        println!("{},{}", k, v)
    }

    for (k, v) in asm.cmds.iter().enumerate() {
        println!("{},{:?}", k, v)
    }

    let mut runtime = Runtime::new(asm);

    if arguments.is_print() {
        if arguments.is_code() {
            runtime.run_with_codes(arguments.speed(), builder);
        } else {
            runtime.run_while_printing(arguments.speed());
        }
    } else if arguments.is_code() {
        builder.display(0);
    } else {
        runtime.run();
    }
}
