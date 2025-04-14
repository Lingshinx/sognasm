use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};

pub struct Arguments(ArgMatches);

fn argus() -> ArgMatches {
    Command::new("Sognasm")
        .version("v0.2.0")
        .about("Sognac的汇编解释器（有这种东西吗？）")
        .arg(Arg::new("source").required(true))
        .arg(
            Arg::new("print")
                .long("print")
                .short('p')
                .action(ArgAction::SetTrue)
                .required(false),
        )
        .arg(
            Arg::new("speed")
                .long("speed")
                .short('s')
                .value_parser(value_parser!(u64))
                .default_value("100"),
        )
        .arg(
            Arg::new("code")
                .long("code")
                .short('c')
                .action(ArgAction::SetTrue)
                .required(false),
        )
        .arg(Arg::new("output").long("output").short('o').required(false))
        .get_matches()
}

impl Arguments {
    pub fn new() -> Self {
        Arguments(argus())
    }

    pub fn speed(&self) -> u64 {
        *self.0.get_one("speed").unwrap()
    }
    pub fn is_print(&self) -> bool {
        *self.0.get_one("print").unwrap()
    }
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
