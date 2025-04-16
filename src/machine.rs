use crate::error::ErrorMessage;
use crate::error::ErrorMessage::*;
use crate::value::Closure;
use crate::value::Value;

#[derive(Debug, Default)]
pub struct Machine<'a> {
    pub variable: Vec<Value<'a>>,
    pub stack: Vec<Value<'a>>,
    sp: usize,

    temp_stack: Vec<Value<'a>>,
}

impl<'a> Machine<'a> {
    pub fn new() -> Self {
        Machine::default()
    }

    pub fn pop(&mut self) -> Result<Value<'a>, ErrorMessage> {
        if let Some(value) = self.stack.pop() {
            Ok(value)
        } else {
            Err(UnderFlow)
        }
    }

    pub fn push(&mut self, v: Value<'a>) -> Result<(), ErrorMessage> {
        if self.stack.len() > 255 {
            return Err(OverFlow);
        }
        self.stack.push(v);
        Ok(())
    }

    pub fn local(&self, offset: u8) -> &Value<'a> {
        &self.variable[self.sp + (offset as usize)]
    }

    pub fn push_to_local(&mut self, value: Value<'a>) {
        self.variable.push(value);
    }

    pub fn push_sp(&mut self) {
        self.variable.push(Value::Function(self.sp));
        self.sp = self.variable.len();
    }

    pub fn ret_ip(&mut self) -> usize {
        use Value::Function;
        self.variable.truncate(self.sp);
        let sp = self.variable.pop().unwrap_or(Value::Bool(false));
        let ip = self.variable.pop().unwrap_or(Value::Bool(false));
        if let (Function(sp), Function(ip)) = (sp, ip) {
            self.sp = sp;
            ip
        } else {
            std::process::exit(0)
        }
    }

    pub fn get_closure(&self) -> Result<&Closure<'a>, ErrorMessage> {
        if let Value::Closure(closure) = self.local(0) {
            Ok(closure)
        } else {
            Err(NotaClosure)
        }
    }

    pub fn swap_temp(&mut self) {
        std::mem::swap(&mut self.stack, &mut self.temp_stack);
    }

    pub fn collect_list(&mut self) {
        let temp = std::mem::take(&mut self.stack);
        let list = Value::List(temp.into_iter().rev().collect());
        self.swap_temp();
        self.stack.push(list);
    }
}
impl std::fmt::Display for &Machine<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "stack: {:?}", self.stack)?;
        write!(f, "local: {:?}", self.variable)
    }
}
