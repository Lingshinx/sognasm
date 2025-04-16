use crate::error::ErrorMessage;
use crate::error::ErrorMessage::*;
use crate::machine::Machine;
use std::collections::LinkedList;
use std::fmt;
use std::io::Read;
use std::io::Write;
use std::time::Duration;

use crate::assemble::Asm;
use crate::command::Oper;
use crate::value::{Closure, Value};

use std::rc::Rc;

pub struct Runtime<'a> {
    index: usize,
    codes: &'a Asm,
    machine: Machine<'a>,
    writer: Box<dyn Write>,
}

fn read() -> Option<u8> {
    let mut buffer = [0u8; 1];
    match std::io::stdin().read_exact(&mut buffer) {
        Ok(_) => Some(buffer[0]),
        Err(_) => None,
    }
}

impl<'a> Runtime<'a> {
    pub fn new(asm: &'a mut Asm) -> Runtime<'a> {
        Runtime {
            index: 0,
            codes: asm,
            machine: Machine::new(),
            writer: Box::new(std::io::stdout()),
        }
    }

    pub fn new_with_writer(asm: &'a mut Asm, writer: Box<dyn Write>) -> Runtime<'a> {
        Runtime {
            index: 0,
            codes: asm,
            machine: Machine::new(),
            writer,
        }
    }

    fn write(&mut self, fmt: fmt::Arguments<'_>) -> Result<(), ErrorMessage> {
        self.writer
            .write_fmt(fmt)
            .map_err(|_| ErrorMessage::PrintErr)
    }

    pub fn next(&mut self) {
        self.index += 1
    }

    pub fn run(mut asm: Asm) {
        let mut runtime = Runtime::new(&mut asm);
        loop {
            let oper = runtime.oper();
            if let Err(e) = runtime.deal_oper(oper) {
                Runtime::error_print(e);
            }
        }
    }

    pub fn run_printing_code(mut asm: Asm, speed: u64, labels: Vec<String>) {
        let asm_clone = asm.clone();
        let writer = Box::new(std::io::Cursor::new(vec![0b0; 15]));
        let mut runtime = Runtime::new_with_writer(&mut asm, writer);
        loop {
            let oper = runtime.oper();
            println!("\x1bcOper:{:?} \n{}", oper, &runtime.machine);
            asm_clone.display(runtime.index, &labels);
            println!();
            if let Err(e) = runtime.deal_oper(oper) {
                Runtime::error_print(e);
            }
            std::thread::sleep(Duration::from_millis(speed));
        }
    }

    pub fn run_printing(mut asm: Asm, speed: u64) {
        let mut runtime = Runtime::new(&mut asm);
        loop {
            let oper = runtime.oper();
            if let Oper::Ret = oper {
            } else {
                println!("Oper:{:?} \t{:?}", oper, &runtime.machine.stack);
                std::thread::sleep(Duration::from_millis(speed));
            }
            if let Err(e) = runtime.deal_oper(oper) {
                Runtime::error_print(e);
            }
        }
    }

    pub fn error_print(msg: ErrorMessage) {
        println!("\n\t\x1b[1;31m{}\x1b[0m", msg);
        std::process::exit(1);
    }

    fn ret(&mut self) {
        let ip = self.machine.ret_ip();
        self.jmp(ip);
    }

    fn pop(&mut self) -> Result<Value<'a>, ErrorMessage> {
        self.machine.pop()
    }

    fn local(&mut self, value: Value<'a>) -> Result<(), ErrorMessage> {
        match value {
            Value::Function(func) => {
                self.call(func);
                Ok(())
            }
            Value::Closure(clos) => {
                self.callosure(clos);
                Ok(())
            }
            _ => self.machine.push(value),
        }
    }

    fn push(&mut self, value: Value<'a>) -> Result<(), ErrorMessage> {
        self.machine.push(value)
    }

    fn call(&mut self, ip: usize) {
        self.machine.push_to_local(Value::Function(self.index));
        self.machine.push_sp();
        self.jmp(ip);
    }

    fn callosure(&mut self, closure: Rc<Closure<'a>>) {
        let ip = closure.ip;
        self.machine.push_to_local(Value::Function(self.index));
        self.machine.push_sp();
        self.machine.push_to_local(Value::Closure(closure));
        self.jmp(ip);
    }

    pub fn jmp(&mut self, ip: usize) {
        self.index = ip
    }

    pub fn oper(&mut self) -> Oper {
        let oper = self.codes.oper(self.index);
        self.next();
        oper
    }

    pub fn byte(&mut self) -> u8 {
        let byte = self.codes.byte(self.index);
        self.next();
        byte
    }

    pub fn offset(&mut self) -> usize {
        let (offset, index) = self.codes.offset(self.index);
        self.jmp(index);
        offset
    }

    pub fn string(&mut self) -> &'a str {
        let offset = self.offset();
        &self.codes.string_pool[offset]
    }

    pub fn number(&mut self) -> f64 {
        let offset = self.offset();
        self.codes.number_pool[offset].0
    }

    pub fn ptr(&mut self) -> usize {
        let offset = self.offset();
        self.codes.function_pool[offset]
    }

    fn deal_oper(&mut self, oper: Oper) -> Result<(), ErrorMessage> {
        use Oper::*;
        use Value::*;
        match oper {
            Add => self.binary_f(|x, y| x + y)?,
            Sub => self.binary_f(|x, y| x - y)?,
            SubBy => self.binary_f(|x, y| y - x)?,
            Div => self.binary_f(|x, y| x / y)?,
            DivBy => self.binary_f(|x, y| y / x)?,
            Mul => self.binary_f(|x, y| x * y)?,
            Mod => self.binary_i(|x, y| x % y)?,
            ModBy => self.binary_i(|x, y| y % x)?,
            Xor => self.binary_i(|x, y| x ^ y)?,
            BitOr => self.binary_i(|x, y| x | y)?,
            BitAnd => self.binary_i(|x, y| x & y)?,
            And => self.binary_bool(|x, y| x && y)?,
            Or => self.binary_bool(|x, y| x || y)?,
            Lt => self.binary_cmp(|x, y| x < y)?,
            Gt => self.binary_cmp(|x, y| x > y)?,
            Eq => self.binary_cmp(|x, y| x == y)?,
            Le => self.binary_cmp(|x, y| x <= y)?,
            Ge => self.binary_cmp(|x, y| x >= y)?,
            Not => self.unary(|x| Bool(!x.into_bool()))?,

            If => {
                let cond = self.pop()?.into_bool();
                let a = self.pop()?;
                let b = self.pop()?;
                let value = if cond { a } else { b };
                self.push(value)?
            }

            Type => {
                self.unary(|x| Value::Byte(x.get_type()))?;
            }

            Push => {
                let offset = self.byte();
                let value = self.machine.local(offset).clone();
                self.push(value)?
            }

            Local => {
                let offset = self.byte();
                let value = self.machine.local(offset).clone();
                self.local(value)?
            }

            Pop => {
                let top = self.pop()?;
                self.machine.push_to_local(top);
            }

            Drop => {
                self.pop()?;
            }

            Call => {
                let ip = self.ptr();
                self.call(ip);
            }

            Ret => self.ret(),

            Capture => {
                let (list, index) = self.codes.list(self.index);
                let capture: Vec<Value> = list
                    .iter()
                    .map(|x| self.machine.local(x.0).clone())
                    .collect();
                self.jmp(index);
                if let Function(ip) = self.pop()? {
                    let closure = Rc::new(crate::value::Closure { ip, capture });
                    self.push(Closure(closure))?;
                } else {
                    return Err(NotaClosure);
                }
            }

            CapCap => {
                let (list, index) = self.codes.list(self.index);
                let closure = if let Closure(closure) = self.machine.local(0) {
                    closure
                } else {
                    return Err(NotaClosure);
                };
                let capture: Vec<Value> = list
                    .iter()
                    .map(|x| closure.capture[x.0 as usize].clone())
                    .collect();
                if let Closure(closure) = self.pop()? {
                    let capture: Vec<Value> = closure
                        .capture
                        .iter()
                        .chain(capture.iter())
                        .cloned()
                        .collect();
                    let closure = Rc::new(crate::value::Closure { ip: index, capture });
                    self.push(Closure(closure))?;
                }
            }

            PushCap => {
                let index = self.byte();
                let value = self.machine.get_closure()?.capture[index as usize].clone();
                self.push(value)?;
            }

            Capped => {
                let index = self.byte();
                let value = self.machine.get_closure()?.capture[index as usize].clone();
                self.local(value)?;
            }

            NewList => {
                let fun = self.pop()?;
                self.machine.swap_temp();
                self.local(fun)?;
            }

            Collect => {
                self.machine.collect_list();
            }

            Insert => {
                let top = self.pop()?;
                self.update_list(|mut list| {
                    list.push_front(top);
                    Ok(list)
                })?;
            }

            Append => {
                let top = self.pop()?;
                self.update_list(|mut list| {
                    list.push_back(top);
                    Ok(list)
                })?;
            }

            Concat => {
                let second_list = self.pop()?;
                let first_list = self.pop()?;

                if let (Value::List(mut first), Value::List(second)) = (first_list, second_list) {
                    for item in second.iter() {
                        first.push_back(item.clone());
                    }
                    self.local(Value::List(first))?
                } else {
                    return Err(ConcatNotList);
                }
            }

            Length => self.with_list(|list| Ok(Number(list.len() as f64)))?,

            Empty => self.with_list(|list| Ok(Bool(list.is_empty())))?,

            Head => {
                self.with_list(|list| Ok(list.front().ok_or(ErrorMessage::HeadEmpty)?.clone()))?
            }

            Rest => self.update_list(|mut list| {
                list.pop_front().ok_or(ErrorMessage::RestEmpty)?;
                Ok(list)
            })?,

            Input => self.local(Value::Byte(read().expect("读取失败")))?,

            Output => {
                let value = self.pop()?;
                self.write(format_args!("{:?}", value))?
            }

            Print => match self.pop()? {
                String(str) => self.write(format_args!("{}", str))?,
                Value::Byte(byte) => self.write(format_args!("{}", byte as char))?,
                Number(number) => self.write(format_args!("{}", number))?,
                Bool(boolean) => self.write(format_args!("{}", boolean))?,
                _ => {
                    return Err(PrintErr);
                }
            },

            Flush => self.writer.flush().unwrap(),

            Oper::Byte => {
                let byte = self.byte();
                self.machine.push(Value::Byte(byte))?
            }

            Num => {
                let number = self.number();
                self.machine.push(Number(number))?
            }

            Func => {
                let ip = self.ptr();
                self.machine.push(Function(ip))?
            }

            Str => {
                let string = self.string();
                self.machine.push(String(string))?
            }

            True => self.machine.push(Bool(true))?,

            False => self.machine.push(Bool(false))?,
            End => std::process::exit(0),
            _ => unreachable!(),
        };
        Ok(())
    }

    fn unary<T>(&mut self, f: T) -> Result<(), ErrorMessage>
    where
        T: Fn(Value) -> Value,
    {
        let a = self.pop()?;
        let x = f(a);
        self.local(x)
    }

    fn binary_f<T>(&mut self, f: T) -> Result<(), ErrorMessage>
    where
        T: Fn(f64, f64) -> f64,
    {
        use Value::*;
        let a = self.pop()?;
        let b = self.pop()?;
        self.local(Number(f(a.into_number(), b.into_number())))
    }

    fn binary_i<T>(&mut self, f: T) -> Result<(), ErrorMessage>
    where
        T: Fn(i64, i64) -> i64,
    {
        use Value::*;
        let a = self.pop()?;
        let b = self.pop()?;
        self.local(Number(f(a.into_integer(), b.into_integer()) as f64))
    }

    fn binary_bool<T>(&mut self, f: T) -> Result<(), ErrorMessage>
    where
        T: Fn(bool, bool) -> bool,
    {
        use Value::*;
        let a = self.pop()?;
        let b = self.pop()?;
        self.local(Bool(f(a.into_bool(), b.into_bool())))
    }

    fn binary_cmp<T>(&mut self, f: T) -> Result<(), ErrorMessage>
    where
        T: Fn(f64, f64) -> bool,
    {
        use Value::*;
        let a = self.pop()?;
        let b = self.pop()?;
        self.local(Bool(f(a.into_number(), b.into_number())))
    }

    fn with_list<T>(&mut self, f: T) -> Result<(), ErrorMessage>
    where
        T: Fn(LinkedList<Value>) -> Result<Value, ErrorMessage>,
    {
        if let Value::List(list) = self.pop()? {
            self.local(f(list)?)
        } else {
            Err(NotaList)
        }
    }

    fn update_list<T>(&mut self, f: T) -> Result<(), ErrorMessage>
    where
        T: FnOnce(LinkedList<Value<'a>>) -> Result<LinkedList<Value<'a>>, ErrorMessage>,
    {
        if let Value::List(list) = self.pop()? {
            self.local(Value::List(f(list)?))
        } else {
            Err(NotaList)
        }
    }
}
