use std::collections::HashMap;

use crate::command::Cmd;
use crate::command::Oper;
use crate::parser::map_color;
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

    pub fn list(&self, index: usize) -> (Vec<Cmd>, usize) {
        let length = self.byte(index) as usize;
        let list = self.cmds[index + 1..index + length + 1].to_vec();
        (list, index + length + 1)
    }

    pub fn display(&self, index: usize, labels: &[String]) {
        let mut counter = 0;
        let map = {
            let mut map = HashMap::<usize, &str>::new();
            for (index, str) in labels.iter().enumerate() {
                map.insert(self.function_pool[index], str);
            }
            map
        };
        let mut cur_index = 0;
        while cur_index != self.cmds.len() {
            use Oper::*;
            counter = if let Some(label) = map.get(&cur_index) {
                print!("\n{}:\n  ", label);
                1
            } else if counter % 13 == 0 {
                print!("\n  ");
                1
            } else {
                print!("  ");
                counter + 1
            };

            let oper = self.oper(cur_index);
            cur_index += 1;
            if index == cur_index {
                print!("\x1b[3;4m")
            }
            print!("{}m{:?}\x1b[1m", map_color(&oper), oper);
            match oper {
                Local | Push | Capped | PushCap => {
                    let byte = self.byte(cur_index);
                    cur_index += 1;
                    print!(" {}", byte);
                }
                Call | Func => {
                    let (offset, index) = self.offset(cur_index);
                    cur_index = index;
                    print!(" {}", labels[offset]);
                }
                Capture | CapCap => {
                    let (list, index) = self.list(cur_index);
                    cur_index = index;
                    print!("{:?}", list.iter().map(|cmd| cmd.0).collect::<Vec<u8>>())
                }
                Byte => {
                    let byte = self.byte(cur_index);
                    cur_index += 1;
                    print!(" {}:", byte);
                    if byte.is_ascii_control() {
                        print!("np",);
                    } else {
                        print!("'{}'", byte as char);
                    };
                }
                Num => {
                    let (offset, index) = self.offset(cur_index);
                    cur_index = index;
                    print!(" {}", self.number_pool[offset].0);
                }
                Str => {
                    let (offset, index) = self.offset(cur_index);
                    cur_index = index;
                    let str = &self.string_pool[offset];
                    let str = if str.len() > 5 {
                        format!("{}..", &str[..5])
                    } else {
                        str.to_owned()
                    };
                    print!(" \"{}\"", str);
                }
                _ => {}
            }
            print!("\x1b[0m");
        }
    }
}
