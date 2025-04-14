use crate::assemble::Asm;
use crate::command::Cmd;
use crate::command::Oper;
use crate::util::swap_kv;
use crate::util::{align_8, uneccape, unescape};
use colored::Color;
use core::f64;
use pest::error::Error;
use pest::error::ErrorVariant;
use pest::iterators::Pair;
use pest::Parser;
use pest::Span;
use pest_derive::Parser;
use std::collections::HashMap;
use std::collections::HashSet;
use std::str::FromStr;

#[derive(Parser)]
#[grammar = "pest/sognasm.pest"]
pub struct Sognasm;

pub trait CmdVec {
    fn push_byte(&mut self, byte: u8);
    fn push_cmd(&mut self, oper: Oper);
    fn push_number(&mut self, number: f64);
    fn push_ptr(&mut self, ptr: usize);
}

impl CmdVec for Vec<Cmd> {
    fn push_byte(&mut self, byte: u8) {
        self.push(Cmd(byte));
    }

    fn push_cmd(&mut self, oper: Oper) {
        self.push(Cmd::from(oper))
    }

    fn push_number(&mut self, number: f64) {
        let size = self.len();
        self.resize(align_8(size), Cmd(0));
        let bytes: [u8; 8] = number.to_le_bytes();
        for byte in bytes {
            self.push(Cmd(byte));
        }
    }

    fn push_ptr(&mut self, ptr: usize) {
        let size = self.len();
        self.resize(align_8(size), Cmd(0));
        let bytes: [u8; 8] = ptr.to_le_bytes();
        for byte in bytes {
            self.push(Cmd(byte));
        }
    }
}

#[derive(Clone)]
enum AsmCmd<'a> {
    Number(f64),
    Ptr(usize),
    Str(String),
    Label(Span<'a>),
    Command(Oper),
    Byte(u8),
    List(Vec<u8>),
}

#[derive(Clone)]
pub struct AsmBuilder<'a> {
    pub label_record: HashMap<&'a str, usize>,
    index: usize,
    cmds: Vec<AsmCmd<'a>>,
}

impl FromStr for Asm {
    type Err = Box<Error<Rule>>;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let builder = AsmBuilder::from_str(str)?;
        Ok(Asm::from(builder))
    }
}

impl<'a> AsmBuilder<'a> {
    pub fn from_str(str: &'a str) -> Result<Self, Box<Error<Rule>>> {
        use Rule::*;
        let mut builder = AsmBuilder::new();
        let pairs = Sognasm::parse(file, str)?;
        for pair in pairs {
            match pair.as_rule() {
                func_name => {
                    let lab = pair.as_str();
                    builder.record_label(lab);
                }

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

impl From<AsmBuilder<'_>> for Asm {
    fn from(builder: AsmBuilder) -> Self {
        use AsmCmd::*;
        let mut string_pool_set = HashSet::<String>::new();
        let mut string_pool_vec = Vec::<String>::new();
        let mut bytes = Vec::<Cmd>::new();
        for cmd in builder.cmds {
            match cmd {
                Number(number) => bytes.push_number(number),
                Ptr(ptr) => bytes.push_ptr(ptr),
                Str(value) => {
                    if !string_pool_set.contains(&value) {
                        string_pool_set.insert(value.to_owned());
                        string_pool_vec.push(value);
                    }
                    let index = string_pool_vec.len() - 1;
                    bytes.push_ptr(index);
                }
                Command(cmd) => bytes.push(Cmd::from(cmd)),
                Byte(byte) => bytes.push_byte(byte),
                List(vec) => {
                    bytes.push_byte(vec.len() as u8);
                    for offset in vec {
                        bytes.push_byte(offset);
                    }
                }
                Label(_) => unreachable!(),
            }
        }
        // assert_eq!(bytes.len(), builder.index);
        Asm::new(bytes, string_pool_vec)
    }
}

impl<'a> AsmBuilder<'a> {
    fn new() -> Self {
        AsmBuilder {
            label_record: HashMap::new(),
            index: 0,
            cmds: Vec::new(),
        }
    }

    fn next_align(&mut self) {
        self.index = align_8(self.index) + 8;
    }

    fn push_str(&mut self, str: String) {
        use AsmCmd::*;
        self.next_align();
        self.cmds.push(Str(str))
    }

    fn push_label(&mut self, label: Span<'a>) {
        use AsmCmd::*;
        self.next_align();
        if let Some(index) = self.label_record.get(label.as_str()) {
            self.push_ptr(*index);
        } else {
            self.cmds.push(Label(label.to_owned()));
        }
    }

    fn push_list(&mut self, list: Vec<u8>) {
        use AsmCmd::*;
        self.index += list.len() + 1;
        self.cmds.push(List(list));
    }

    fn record_label(&mut self, label: &'a str) {
        self.label_record.insert(label, self.index);
    }

    fn scan_label(&mut self) -> Result<(), Box<Error<Rule>>> {
        use AsmCmd::*;
        for command in &mut self.cmds {
            if let Label(label) = command {
                if let Some(index) = self.label_record.get(label.as_str()) {
                    *command = Ptr(*index);
                } else {
                    return Err(Box::new(Error::new_from_span(
                        ErrorVariant::CustomError {
                            message: "未知的标签".to_owned(),
                        },
                        *label,
                    )));
                }
            }
        }
        Ok(())
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
                self.push_label(lab);
            }

            Call => {
                self.push_cmd(Oper::Call);
                let lab = pair.into_inner().next().unwrap().as_span();
                self.push_label(lab);
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
        println!("{:?},{}", rule, self.index);
    }
}

impl CmdVec for AsmBuilder<'_> {
    fn push_byte(&mut self, byte: u8) {
        use AsmCmd::*;
        self.index += 1;
        self.cmds.push(Byte(byte))
    }

    fn push_cmd(&mut self, oper: Oper) {
        use AsmCmd::*;
        self.index += 1;
        self.cmds.push(Command(oper));
    }

    fn push_number(&mut self, number: f64) {
        use AsmCmd::*;
        self.next_align();
        self.cmds.push(Number(number))
    }

    fn push_ptr(&mut self, ptr: usize) {
        use AsmCmd::*;
        self.next_align();
        self.cmds.push(Ptr(ptr))
    }
}

impl AsmBuilder<'_> {
    #[allow(dead_code)]
    pub fn display(&self, index: usize) {
        let label = swap_kv(self.label_record.clone());
        let mut counter = 14;
        let mut index_counter: usize = 0;
        for cmd in &self.cmds {
            counter = if label.contains_key(&index_counter) {
                print!("\n\x1b[0m{}:\n  ", label.get(&index_counter).unwrap());
                0
            } else if counter > 13 {
                print!("\n  ");
                0
            } else {
                print!("  ");
                counter + 1
            };
            match cmd {
                AsmCmd::Number(number) => {
                    print!("{},", number);
                    index_counter = align_8(index_counter) + 8;
                }
                AsmCmd::Ptr(ptr) => {
                    print!("{},", label.get(ptr).unwrap());
                    index_counter = align_8(index_counter) + 8;
                }
                AsmCmd::Str(str) => {
                    let str = if str.len() > 5 {
                        format!("{}..", &str[..5])
                    } else {
                        str.to_owned()
                    };
                    print!("\"{}\",", str);
                    index_counter = align_8(index_counter) + 8;
                }
                AsmCmd::Command(cmd) => {
                    print!("\x1b[0m{}m", map_color(cmd));
                    if index_counter + 1 == index {
                        print!("\x1b[3;4;47m")
                    }
                    print!("{:?}\x1b[1m", cmd);
                    index_counter += 1;
                }
                AsmCmd::Byte(byte) => {
                    print!("{}:", byte);
                    if byte.is_ascii_control() {
                        print!("np,",);
                    } else {
                        print!("'{}',", *byte as char);
                    };
                    index_counter += 1;
                }
                AsmCmd::List(list) => {
                    let list = list
                        .iter()
                        .map(u8::to_string)
                        .collect::<Vec<String>>()
                        .join(" ");
                    print!("[{}],", list);
                    index_counter += list.len() + 1;
                }
                AsmCmd::Label(_) => unreachable!(),
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
