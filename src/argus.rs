use clap::{arg, value_parser, ArgAction, ArgMatches, Command};

pub struct Arguments(ArgMatches);

fn argus() -> ArgMatches {
    Command::new("Sognasm")
        .version("v0.2.0")
        .about("Sognac的字节码解释器")
        .arg(arg!([source]).required(true))
        .arg(arg!(-p --print "打印运行栈").action(ArgAction::SetTrue).required(false))
        .arg(
            arg!(-s --speed <speed> "打印周期(单位:ms)")
                .value_parser(value_parser!(u64))
                .default_value("100"),
        )
        .arg(arg!(-c --code "打印字节码").action(ArgAction::SetTrue).required(false))
        // .arg(arg!(-o --output <file> ).required(false))
        .get_matches()
}

impl Arguments {
    pub fn new() -> Self {
        Arguments(argus())
    }

    #[allow(dead_code)]
    pub fn speed(&self) -> u64 {
        *self.0.get_one("speed").unwrap()
    }

    #[allow(dead_code)]
    pub fn is_print(&self) -> bool {
        *self.0.get_one("print").unwrap()
    }

    #[allow(dead_code)]
    pub fn is_code(&self) -> bool {
        *self.0.get_one("code").unwrap()
    }

    pub fn source(&self) -> &String {
        self.0.get_one("source").unwrap()
    }

    #[allow(dead_code)]
    pub fn output(&self) -> &String {
        self.0.get_one("output").unwrap()
    }
}
