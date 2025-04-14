use colored::Colorize;
use core::panic;
use std::collections::LinkedList;
use std::io::Read;
use std::io::Write;
use std::thread;
use std::time::Duration;

use crate::assemble::Asm;
use crate::command::Oper;
use crate::parser::AsmBuilder;
use crate::value::{Closure, Value};

use std::rc::Rc;

pub struct Runtime {
    codes: Asm,
    variable: Vec<Value>,
    stack: Vec<Value>,
    sp: usize,
    temp_stack: Vec<Value>,
}

fn read() -> Option<u8> {
    let mut buffer = [0u8; 1];
    match std::io::stdin().read_exact(&mut buffer) {
        Ok(_) => Some(buffer[0]),
        Err(_) => None,
    }
}

impl Runtime {
    pub fn new(asm: Asm) -> Self {
        Runtime {
            codes: asm,
            variable: Vec::new(),
            stack: Vec::new(),
            sp: 0,
            temp_stack: Vec::new(),
        }
    }

    pub fn run_with_codes(&mut self, speed: u64, builder: AsmBuilder) {
        print!("\x1b[?25h"); // ANSI escape sequence to hide cursor
        loop {
            let oper = self.codes.oper();
            if let Oper::End = oper {
                break;
            } else {
                print!("\x1bc");
                self.print(&oper);
                builder.display(self.codes.index);
                thread::sleep(Duration::from_millis(speed));
                self.deal_oper(oper);
            }
        }
        print!("\x1b[?25l"); // ANSI escape sequence to show cursor
    }

    pub fn run_while_printing(&mut self, speed: u64) {
        print!("\x1b[?25h"); // ANSI escape sequence to hide cursor
        loop {
            let oper = self.codes.oper();
            if let Oper::End = oper {
                break;
            } else {
                thread::sleep(Duration::from_millis(speed));
                self.deal_oper(oper);
                self.print(&oper);
                println!();
            }
        }
        print!("\x1b[?25l"); // ANSI escape sequence to show cursor
    }

    pub fn run(&mut self) {
        loop {
            let oper = self.codes.oper();
            if let Oper::End = oper {
                break;
            } else {
                self.deal_oper(oper);
            }
        }
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().expect("栈空了")
    }

    fn push(&mut self, v: Value) {
        match v {
            Value::Function(ip) => self.call(ip),
            Value::Closure(closure) => self.callosure(closure),
            _ => self.stack.push(v),
        }
    }

    fn relative(&self, offset: u8) -> &Value {
        &self.variable[self.sp + (offset as usize)]
    }

    fn call(&mut self, ip: usize) {
        self.variable.push(Value::Function(self.codes.index));
        self.variable.push(Value::Function(self.sp));
        self.sp = self.variable.len();
        self.codes.jmp(ip);
    }

    fn ret(&mut self) {
        use Value::Function;
        self.variable.truncate(self.sp);
        let sp = self.variable.pop().unwrap_or(Value::Bool(false));
        let ip = self.variable.pop().unwrap_or(Value::Bool(false));
        if let (Function(sp), Function(ip)) = (sp, ip) {
            self.sp = sp;
            self.codes.jmp(ip);
        } else {
            std::process::exit(0)
        }
    }
    fn callosure(&mut self, closure: Rc<Closure>) {
        let ip = closure.ip;
        self.variable.push(Value::Function(self.codes.index));
        self.variable.push(Value::Function(self.sp));
        self.sp = self.variable.len();
        self.variable.push(Value::Closure(closure));
        self.codes.jmp(ip);
    }
    fn get_closure(&self) -> &Closure {
        if let Value::Closure(closure) = self.relative(0) {
            closure
        } else {
            panic!("不是闭包")
        }
    }

    fn deal_oper(&mut self, oper: Oper) {
        use Oper::*;
        use Value::*;
        match oper {
            Add => self.binary_f(|x, y| x + y),
            Sub => self.binary_f(|x, y| x - y),
            SubBy => self.binary_f(|x, y| y - x),
            Div => self.binary_f(|x, y| x / y),
            DivBy => self.binary_f(|x, y| y / x),
            Mul => self.binary_f(|x, y| x * y),
            Mod => self.binary_i(|x, y| x % y),
            ModBy => self.binary_i(|x, y| y % x),
            Xor => self.binary_i(|x, y| x ^ y),
            BitOr => self.binary_i(|x, y| x | y),
            BitAnd => self.binary_i(|x, y| x & y),
            And => self.binary_bool(|x, y| x && y),
            Or => self.binary_bool(|x, y| x || y),
            Lt => self.binary_cmp(|x, y| x < y),
            Gt => self.binary_cmp(|x, y| x > y),
            Eq => self.binary_cmp(|x, y| x == y),
            Le => self.binary_cmp(|x, y| x <= y),
            Ge => self.binary_cmp(|x, y| x >= y),
            Not => self.unary(|x| Bool(!x.into_bool())),

            If => {
                let cond = self.pop().into_bool();
                let a = self.pop();
                let b = self.pop();
                let value = if cond { a } else { b };
                self.push(value);
            }

            Type => self.unary(|x| Value::Byte(x.get_type())),

            Push => {
                let offset = self.codes.byte();
                let value = self.relative(offset).clone();
                self.push(value);
            }

            Local => {
                let offset = self.codes.byte();
                let value = self.relative(offset).clone();
                self.stack.push(value);
            }

            Pop => {
                let top = self.pop();
                self.variable.push(top);
            }

            Drop => {
                self.pop();
            }

            Call => {
                let ip = self.codes.ptr();
                self.call(ip);
            }

            Ret => self.ret(),

            Capture => {
                let length = self.codes.byte() as usize;
                let ip = self.codes.index;
                let span = &self.codes.cmds[ip..ip + length];
                let capture: Vec<Value> = span.iter().map(|x| self.relative(x.0).clone()).collect();
                self.codes.jmp(ip + length);
                if let Function(ip) = self.pop() {
                    let closure = Rc::new(crate::value::Closure { ip, capture });
                    self.stack.push(Closure(closure));
                } else {
                    panic!("不是函数");
                }
            }

            CapFromCap => {
                let length = self.codes.byte() as usize;
                let ip = self.codes.index;
                let span = &self.codes.cmds[ip..ip + length];
                let closure = if let Closure(closure) = self.relative(0) {
                    closure
                } else {
                    panic!("不是闭包")
                };
                let capture: Vec<Value> = span
                    .iter()
                    .map(|x| closure.capture[x.0 as usize].clone())
                    .collect();
                if let Closure(closure) = self.pop() {
                    let capture: Vec<Value> = closure
                        .capture
                        .iter()
                        .chain(capture.iter())
                        .cloned()
                        .collect();
                    let closure = Rc::new(crate::value::Closure { ip, capture });
                    self.stack.push(Closure(closure));
                }
            }

            PushCapped => {
                let index = self.codes.byte();
                let value = self.get_closure().capture[index as usize].clone();
                self.push(value);
            }

            Capped => {
                let index = self.codes.byte();
                let value = self.get_closure().capture[index as usize].clone();
                self.stack.push(value);
            }

            NewList => {
                let fun = self.pop();
                std::mem::swap(&mut self.stack, &mut self.temp_stack);
                self.push(fun);
            }

            Collect => {
                let temp = std::mem::take(&mut self.stack);
                let list = temp.into_iter().collect();
                std::mem::swap(&mut self.stack, &mut self.temp_stack);
                self.stack.push(List(list));
            }

            Insert => {
                let top = self.pop();
                self.update_list(|mut list| {
                    list.push_front(top);
                    list
                });
            }

            Append => {
                let top = self.pop();
                self.update_list(|mut list| {
                    list.push_back(top);
                    list
                });
            }

            Concat => {
                let second_list = self.pop();
                let first_list = self.pop();

                if let (Value::List(mut first), Value::List(second)) = (first_list, second_list) {
                    for item in second.iter() {
                        first.push_back(item.clone());
                    }
                    self.stack.push(Value::List(first));
                } else {
                    panic!("Concat 需要两个列表")
                }
            }

            Length => self.with_list(|list| Number(list.len() as f64)),

            Empty => self.with_list(|list| Bool(list.is_empty())),

            Head => self.with_list(|list| list.front().expect("空列表").clone()),

            Rest => self.update_list(|mut list| {
                list.pop_front().expect("空列表");
                list
            }),

            Input => self.stack.push(Value::Byte(read().expect("读取失败"))),

            Output => print!("{:?}", self.pop()),

            Print => match self.pop() {
                String(str) => {
                    print!("{}", str);
                }
                Value::Byte(byte) => {
                    print!("{}", byte as char)
                }
                _ => {
                    panic!("不可打印, 你应该使用Output")
                }
            },

            Flush => std::io::stdout().flush().unwrap(),

            Oper::Byte => {
                let byte = self.codes.byte();
                self.stack.push(Value::Byte(byte));
            }

            Num => {
                let number = self.codes.number();
                self.stack.push(Number(number));
            }

            Func => {
                let ip = self.codes.ptr();
                self.stack.push(Function(ip));
            }

            Str => {
                let index = self.codes.ptr();
                let str = self.codes.string_pool[index].clone();
                let value = Value::String(str);
                self.stack.push(value);
            }

            True => self.stack.push(Bool(true)),
            False => self.stack.push(Bool(false)),
            _ => unreachable!(),
        };
    }

    fn print(&self, oper: &Oper) {
        print!(
            "{}: {:?} {}\t {}: {:?}",
            "Oper".green().bold(),
            oper,
            self.codes.index,
            "Stack".blue().bold(),
            self.stack
        );
        std::io::stdout().flush().unwrap();
    }

    fn unary<T>(&mut self, f: T)
    where
        T: Fn(Value) -> Value,
    {
        let a = self.pop();
        let x = f(a);
        self.stack.push(x);
    }

    fn binary_f<T>(&mut self, f: T)
    where
        T: Fn(f64, f64) -> f64,
    {
        use Value::*;
        let a = self.pop();
        let b = self.pop();
        self.stack.push(Number(f(a.into_number(), b.into_number())));
    }

    fn binary_i<T>(&mut self, f: T)
    where
        T: Fn(i64, i64) -> i64,
    {
        use Value::*;
        let a = self.pop();
        let b = self.pop();
        self.stack
            .push(Number(f(a.into_integer(), b.into_integer()) as f64));
    }

    fn binary_bool<T>(&mut self, f: T)
    where
        T: Fn(bool, bool) -> bool,
    {
        use Value::*;
        let a = self.pop();
        let b = self.pop();
        self.stack.push(Bool(f(a.into_bool(), b.into_bool())));
    }

    fn binary_cmp<T>(&mut self, f: T)
    where
        T: Fn(f64, f64) -> bool,
    {
        use Value::*;
        let a = self.pop();
        let b = self.pop();
        self.stack.push(Bool(f(a.into_number(), b.into_number())));
    }

    fn with_list<T>(&mut self, f: T)
    where
        T: Fn(LinkedList<Value>) -> Value,
    {
        if let Value::List(list) = self.pop() {
            self.stack.push(f(list));
        } else {
            panic!("不是列表")
        }
    }

    fn update_list<T>(&mut self, f: T)
    where
        T: FnOnce(LinkedList<Value>) -> LinkedList<Value>,
    {
        if let Value::List(list) = self.pop() {
            self.stack.push(Value::List(f(list)));
        } else {
            panic!("不是列表")
        }
    }
}
