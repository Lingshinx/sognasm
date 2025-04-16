use core::f64;
use std::{collections::LinkedList, rc::Rc};

use colored::Colorize;

pub struct Closure<'a> {
    pub capture: Vec<Value<'a>>,
    pub ip: usize,
}

#[derive(Clone)]
pub enum Value<'a> {
    Number(f64),
    Function(usize),
    Closure(Rc<Closure<'a>>),
    List(LinkedList<Value<'a>>),
    String(&'a str),
    Byte(u8),
    Bool(bool),
}

impl std::fmt::Debug for Value<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.color_str())
    }
}

impl Value<'_> {
    fn color_str(&self) -> String {
        use Value::*;
        match self {
            Number(number) => number.to_string().bold().to_string(),
            Function(ip) => format!("F{}", ip).bright_green().to_string(),
            Closure(rc) => format!("C{}", rc.ip).yellow().to_string(),
            List(list) => format!("{:?}", list).cyan().to_string(),
            String(str) => {
                let str = if str.len() > 5 { &str[..5] } else { str };
                format!("\x1b[32m\"{str}\"\x1b[0m")
            }

            Byte(byte) => byte.to_string().bright_blue().to_string(),
            Bool(bool) => bool.to_string().red().to_string(),
        }
    }

    pub fn get_type(&self) -> u8 {
        use Value::*;
        match self {
            Number(_) => b'n',
            Function(_) => b'f',
            Closure(_) => b'c',
            List(_) => b'l',
            String(_) => b's',
            Byte(_) => b'x',
            Bool(_) => b'b',
        }
    }
    pub fn into_integer(self) -> i64 {
        use Value::*;
        match self {
            Number(number) => number as i64,
            Function(_) | Closure(_) => 0,
            List(list) => list.len() as i64,
            String(str) => str.parse::<f64>().unwrap_or(f64::NAN) as i64,
            Byte(byte) => byte as i64,
            Bool(cond) => {
                if cond {
                    1
                } else {
                    0
                }
            }
        }
    }

    pub fn into_bool(self) -> bool {
        use Value::*;
        match self {
            Number(number) => number == 0.0,
            Function(_) => false,
            Closure(_) => false,
            List(list) => list.is_empty(),
            String(str) => str.is_empty(),
            Byte(byte) => byte == 0,
            Bool(cond) => cond,
        }
    }

    pub fn into_number(self) -> f64 {
        use Value::*;
        match self {
            Number(number) => number,
            Function(_) | Closure(_) => f64::NAN,
            List(list) => list.len() as f64,
            String(str) => str.parse().unwrap_or(f64::NAN),
            Byte(byte) => byte as f64,
            Bool(cond) => {
                if cond {
                    1.0
                } else {
                    0.0
                }
            }
        }
    }
}
