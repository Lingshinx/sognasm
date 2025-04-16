use crate::assemble::Asm;
use crate::command::{Cmd, Oper};
use crate::record::Record;
use crate::util::{uneccape, unescape};
use colored::Color;
use core::f64;
use pest::error::{Error, ErrorVariant};
use pest::iterators::Pair;
use pest::Parser;
use pest::Span;
use pest_derive::Parser;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

#[derive(Parser)]
#[grammar = "pest/sognasm.pest"]
pub struct Sognasm;

#[derive(Clone)]
enum AsmCmd<'a> {
    Number(Number),
    Str(String),
    Func(Span<'a>),
    Label(Span<'a>),
    Command(Oper),
    Byte(u8),
    List(Vec<u8>),
}

#[derive(Clone, Copy, Debug)]
pub struct Number(pub f64);

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for Number {}

impl Hash for Number {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

#[derive(Clone)]
pub struct AsmBuilder<'a> {
    cmds: Vec<AsmCmd<'a>>,
}

impl<'a> AsmBuilder<'a> {
    pub fn from_str(str: &'a str) -> Result<Self, Box<Error<Rule>>> {
        use Rule::*;
        let mut builder = AsmBuilder::new();
        let pairs = Sognasm::parse(file, str)?;
        for pair in pairs {
            match pair.as_rule() {
                func_name => builder.push_label(pair.as_span()),

                commands => {
                    for pair in pair.into_inner().rev() {
                        builder.push_pair(pair);
                    }
                }

                func_end => builder.push_cmd(Oper::Ret),
                EOI => builder.push_cmd(Oper::End),

                _ => unreachable!(),
            }
        }
        builder.scan_label()?;
        Ok(builder)
    }
}

trait ByteCode {
    fn push_byte(&mut self, byte: u8);
    fn push_oper(&mut self, oper: Oper);
    fn push_offset(&mut self, offset: usize);
}

impl ByteCode for Vec<Cmd> {
    fn push_byte(&mut self, byte: u8) {
        self.push(Cmd(byte));
    }

    fn push_oper(&mut self, oper: Oper) {
        self.push(Cmd(oper as u8));
    }

    fn push_offset(&mut self, offset: usize) {
        let mut offset = offset;
        while offset > 0xff {
            self.push(Cmd(0xff));
            offset -= 0xff;
        }
        self.push(Cmd(offset as u8));
    }
}

impl From<AsmBuilder<'_>> for Asm {
    fn from(builder: AsmBuilder) -> Self {
        use AsmCmd::*;
        let mut string_pool = Record::new();
        let mut number_pool = Record::new();
        let mut function_pool = Record::new();
        let mut label_record = HashMap::new();
        let mut bytes = Vec::<Cmd>::new();
        for cmd in builder.cmds {
            match cmd {
                Number(number) => bytes.push_offset(number_pool.insert(number)),
                Str(string) => bytes.push_offset(string_pool.insert(string)),
                Command(cmd) => bytes.push_oper(cmd),
                Byte(byte) => bytes.push_byte(byte),
                List(vec) => {
                    bytes.push_byte(vec.len() as u8);
                    for offset in vec {
                        bytes.push_byte(offset);
                    }
                }
                Func(lab) => bytes.push_offset(function_pool.insert(lab.as_str())),
                Label(span) => {
                    label_record.insert(span.as_str(), bytes.len());
                }
            }
        }
        Asm::new(
            bytes,
            string_pool.into_vec(),
            number_pool.into_vec(),
            function_pool
                .into_vec()
                .into_iter()
                .map(|x| *label_record.get(&x).unwrap())
                .collect(),
        )
    }
}

impl<'a> AsmBuilder<'a> {
    fn new() -> Self {
        AsmBuilder { cmds: Vec::new() }
    }

    fn push_str(&mut self, str: String) {
        use AsmCmd::*;
        self.cmds.push(Str(str))
    }

    fn push_label(&mut self, label: Span<'a>) {
        use AsmCmd::*;
        self.cmds.push(Label(label));
    }

    fn push_func(&mut self, label: Span<'a>) {
        use AsmCmd::*;
        self.cmds.push(Func(label));
    }
    fn push_list(&mut self, list: Vec<u8>) {
        use AsmCmd::*;
        self.cmds.push(List(list));
    }

    fn push_byte(&mut self, byte: u8) {
        use AsmCmd::*;
        self.cmds.push(Byte(byte))
    }

    fn push_cmd(&mut self, oper: Oper) {
        use AsmCmd::*;
        self.cmds.push(Command(oper));
    }

    fn push_number(&mut self, number: f64) {
        self.cmds.push(AsmCmd::Number(Number(number)))
    }

    fn scan_label(&mut self) -> Result<(), Box<Error<Rule>>> {
        use AsmCmd::*;
        let mut label_record = HashSet::new();
        let mut unknown_record = HashMap::new();
        for command in &self.cmds {
            match command {
                Func(name) => {
                    unknown_record.insert(name.as_str(), name);
                }
                Label(name) => {
                    label_record.insert(name.as_str());
                }
                _ => {}
            }
        }

        for v in label_record {
            unknown_record.remove(v);
        }

        if unknown_record.is_empty() {
            Ok(())
        } else {
            Err(Box::new(Error::new_from_span(
                ErrorVariant::CustomError {
                    message: "未知的标签".to_owned(),
                },
                **unknown_record.iter().next().unwrap().1,
            )))
        }
    }

    fn push_pair(&mut self, pair: Pair<'a, Rule>) {
        fn get_offset(pair: Pair<'_, Rule>) -> u8 {
            pair.into_inner()
                .next()
                .unwrap()
                .as_str()
                .parse::<u8>()
                .unwrap()
        }

        use Rule::*;
        let rule = pair.as_rule();
        match rule {
            Add => self.push_cmd(Oper::Add),
            Sub => self.push_cmd(Oper::Sub),
            SubBy => self.push_cmd(Oper::SubBy),
            Div => self.push_cmd(Oper::Div),
            DivBy => self.push_cmd(Oper::DivBy),
            Mul => self.push_cmd(Oper::Mul),
            Mod => self.push_cmd(Oper::Mod),
            ModBy => self.push_cmd(Oper::ModBy),
            Xor => self.push_cmd(Oper::Xor),
            BitOr => self.push_cmd(Oper::BitOr),
            BitAnd => self.push_cmd(Oper::BitAnd),
            And => self.push_cmd(Oper::And),
            Or => self.push_cmd(Oper::Or),
            Not => self.push_cmd(Oper::Not),
            Lt => self.push_cmd(Oper::Lt),
            Gt => self.push_cmd(Oper::Gt),
            Eq => self.push_cmd(Oper::Eq),
            Le => self.push_cmd(Oper::Le),
            Ge => self.push_cmd(Oper::Ge),
            If => self.push_cmd(Oper::If),
            Type => self.push_cmd(Oper::Type),
            Let => self.push_cmd(Oper::Pop),
            Drop => self.push_cmd(Oper::Drop),
            Ret => self.push_cmd(Oper::Ret),
            Insert => self.push_cmd(Oper::Insert),
            Append => self.push_cmd(Oper::Append),
            Concat => self.push_cmd(Oper::Concat),
            Length => self.push_cmd(Oper::Length),
            Empty => self.push_cmd(Oper::Empty),
            Head => self.push_cmd(Oper::Head),
            Rest => self.push_cmd(Oper::Rest),
            Input => self.push_cmd(Oper::Input),
            Output => self.push_cmd(Oper::Output),
            Print => self.push_cmd(Oper::Print),
            Flush => self.push_cmd(Oper::Flush),
            True => self.push_cmd(Oper::True),
            False => self.push_cmd(Oper::False),

            List => {
                self.push_cmd(Oper::NewList);
                self.push_cmd(Oper::Collect)
            }

            Capture => {
                self.push_cmd(Oper::Capture);
                let caplist = pair.into_inner();
                let mut list = vec![];
                for capped in caplist {
                    list.push(capped.as_str().parse::<u8>().unwrap());
                }
                self.push_list(list);
            }

            CapFromCap => {
                self.push_cmd(Oper::CapFromCap);
                let caplist = pair.into_inner();
                let mut list = vec![];
                for capped in caplist {
                    list.push(capped.as_str().parse::<u8>().unwrap());
                }
                self.push_list(list);
            }

            Local => {
                self.push_cmd(Oper::Local);
                self.push_byte(get_offset(pair))
            }

            Push => {
                self.push_cmd(Oper::Push);
                self.push_byte(get_offset(pair))
            }

            Capped => {
                self.push_cmd(Oper::Capped);
                self.push_byte(get_offset(pair))
            }

            PushCapped => {
                self.push_cmd(Oper::PushCapped);
                self.push_byte(get_offset(pair))
            }

            Num => {
                self.push_cmd(Oper::Num);
                let value: f64 = pair.as_str().parse().expect("不是浮点数");
                self.push_number(value);
            }

            Func => {
                self.push_cmd(Oper::Func);
                let lab = pair.into_inner().next().unwrap().as_span();
                self.push_func(lab);
            }

            Call => {
                self.push_cmd(Oper::Call);
                let lab = pair.into_inner().next().unwrap().as_span();
                self.push_func(lab);
            }

            Byte => {
                self.push_cmd(Oper::Byte);
                let value =
                    u8::from_str_radix(pair.as_str(), 16).expect("16进制字节解析失败, 这不太可能");
                self.push_byte(value);
            }

            Char => {
                self.push_cmd(Oper::Byte);
                self.push_byte(uneccape(pair.into_inner().as_str()));
            }

            Str => {
                self.push_cmd(Oper::Str);
                let str = pair.into_inner().next().expect("解析字符串失败").as_str();
                self.push_str(unescape(str));
            }
            End => self.push_cmd(Oper::End),
            _ => unreachable!(),
        }
    }
}

impl AsmBuilder<'_> {
    #[allow(dead_code)]
    pub fn display(&self, index: usize) {
        let mut counter = 0;
        for cmd in &self.cmds {
            counter += 1;
            if counter % 13 == 0 {
                print!("\n  ");
            } else {
                print!("  ");
            };
            if index == counter {
                print!("\x1b[3;4m")
            }
            match cmd {
                AsmCmd::Number(number) => print!("{}", number.0),
                AsmCmd::Str(str) => {
                    let str = if str.len() > 5 {
                        format!("{}..", &str[..5])
                    } else {
                        str.to_owned()
                    };
                    print!("\"{}\",", str);
                }
                AsmCmd::Command(cmd) => {
                    print!("\x1b[0m{}m", map_color(cmd));
                    print!("{:?}\x1b[1m", cmd);
                }
                AsmCmd::Byte(byte) => {
                    print!("{}:", byte);
                    if byte.is_ascii_control() {
                        print!("np,",);
                    } else {
                        print!("'{}',", *byte as char);
                    };
                }
                AsmCmd::List(list) => {
                    let list = list
                        .iter()
                        .map(u8::to_string)
                        .collect::<Vec<String>>()
                        .join(" ");
                    print!("[{}],", list);
                }
                AsmCmd::Func(name) => print!("{}", name.as_str()),
                AsmCmd::Label(name) => {
                    counter = 0;
                    print!("\n\x1b[0m{}:\n", name.as_str())
                }
            }
        }
        println!("\x1b[0m")
    }
}

fn map_color(cmd: &Oper) -> String {
    use Oper::*;
    let fg = match cmd {
        Call | Add | Sub | SubBy | Div | DivBy | Mul | Mod | ModBy | Xor | BitOr | BitAnd | And
        | Or | Not | Lt | Gt | Eq | Le | Ge => Color::Cyan,
        If | Type | Local | Capped | Push | Pop | Drop | Ret | End => Color::Red,
        Capture | CapFromCap => Color::Yellow,
        PushCapped | NewList | Collect | Insert | Append | Concat | Length | Empty | Head
        | Rest | Input => Color::Blue,
        Output | Print | Flush => Color::Magenta,
        Byte | Num | Func | Str | True | False => Color::Green,
        _ => unreachable!(),
    }
    .to_fg_str();
    format!("\x1b[{}", fg)
}
