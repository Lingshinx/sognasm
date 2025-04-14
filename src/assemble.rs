use crate::command::Cmd;
use crate::command::Oper;
use crate::util::align_8;

#[derive(Debug, Clone)]
pub struct Asm {
    pub index: usize,
    pub cmds: Vec<Cmd>,
    pub string_pool: Vec<String>,
}

impl Asm {
    pub fn new(cmds: Vec<Cmd>, string_pool: Vec<String>) -> Self {
        Asm {
            index: 0,
            cmds,
            string_pool,
        }
    }

    pub fn next(&mut self) {
        self.index += 1
    }

    pub fn next_align(&mut self) {
        self.index = align_8(self.index) + 8
    }

    pub fn oper(&mut self) -> Oper {
        let oper = self.cmds[self.index];
        let result = Oper::from(&oper);
        self.next();
        result
    }

    pub fn byte(&mut self) -> u8 {
        let result = self.cmds[self.index].0;
        self.next();
        result
    }

    pub fn number(&mut self) -> f64 {
        let index = align_8(self.index);
        let byte_slice = &self.cmds[index..index + 8];
        let number = unsafe {
            let ptr = byte_slice.as_ptr();
            let bytes_ptr = std::ptr::addr_of!((*ptr)) as *const [u8; 8];
            f64::from_le_bytes(*bytes_ptr)
        };
        self.next_align();
        number
    }

    pub fn ptr(&mut self) -> usize {
        let index = align_8(self.index);
        let byte_slice = &self.cmds[index..index + 8];
        let ptr = unsafe {
            let ptr = byte_slice.as_ptr() as *const [u8; 8];
            usize::from_le_bytes(*ptr)
        };
        self.next_align();
        ptr
    }

    pub fn jmp(&mut self, index: usize) {
        self.index = index;
    }
}
