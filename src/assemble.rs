use crate::command::Cmd;
use crate::command::Oper;
use crate::parser::Number;

#[derive(Debug, Clone)]
pub struct Asm {
    pub cmds: Vec<Cmd>,
    pub string_pool: Vec<String>,
    pub number_pool: Vec<Number>,
    pub function_pool: Vec<usize>,
}

impl Asm {
    pub fn new(
        cmds: Vec<Cmd>,
        string_pool: Vec<String>,
        number_pool: Vec<Number>,
        function_pool: Vec<usize>,
    ) -> Self {
        Asm {
            cmds,
            string_pool,
            number_pool,
            function_pool,
        }
    }

    pub fn oper(&self, index: usize) -> Oper {
        let oper = self.cmds[index];
        Oper::from(&oper)
    }

    pub fn byte(&self, index: usize) -> u8 {
        self.cmds[index].0
    }

    pub fn offset(&self, mut index: usize) -> (usize, usize) {
        let mut offset = 0;
        loop {
            let byte = self.cmds[index].0;
            index += 1;
            offset += byte;
            if byte != 0xff {
                break;
            }
        }
        (offset as usize, index)
    }
}
