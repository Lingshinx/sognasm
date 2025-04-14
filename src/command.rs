#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Oper {
    __ = 0,
    Add,
    Sub,
    SubBy,
    Div,
    DivBy,
    Mul,
    Mod,
    ModBy,

    Xor,
    BitOr,
    BitAnd,

    And,
    Or,
    Not,

    Lt,
    Gt,
    Eq,
    Le,
    Ge,

    If,

    Type, // 将value转换成type

    Local, // 后接一个u8，将对应局部变量放到栈上, 如果是函数就调用
    Push,  // 后接一个u8，将对应局部变量放到栈上
    Pop,   // 将栈顶元素放到局部变量
    Drop,  // 移除栈顶元素
    Call,  // 后接一个usize，调用对应函数
    Ret,   // 退出函数, 销毁局部变量

    Capture,    // 将栈顶的函数变成闭包，后接一个u8, 表示数组长度，接下来的数组表示捕获列表
    CapFromCap, // 后接一个u8, 表示数组长度，接下来的数组表示捕获列表, 从捕获列表中捕获
    Capped,     // 后接一个u8, 将对应捕获变量放到栈上，如果是函数就调用
    PushCapped, // 后接一个u8, 将对应捕获变量放到栈上

    NewList, // 切换到新栈, 并调用栈顶函数
    Collect, // 收集栈转换成列表, 回到旧栈
    Insert,  // 将元素放在列表前
    Append,  // 将元素放在列表后
    Concat,  // 连接两个列表
    Length,  // 查看列表长度
    Empty,   // 查看列表是否为空
    Head,    // 获取头部
    Rest,    // 获取去掉头部后的元素

    Input,  // 从输入流读取一个字符放到栈顶
    Output, // 将栈顶元素输出
    Print,  // 打印字符或字符串
    Flush,  // 刷新输出流

    Byte,
    Num,
    Func,
    Str,

    True,
    False,

    End,
}

#[derive(Copy, Clone)]
pub struct Cmd(pub u8);

impl std::fmt::Debug for Cmd {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}:{}", Oper::from(self), self.0)
    }
}

impl From<Oper> for Cmd {
    fn from(value: Oper) -> Self {
        Cmd(value as u8)
    }
}

impl From<&Cmd> for Oper {
    fn from(value: &Cmd) -> Self {
        use Oper::*;
        match value.0 {
            1 => Add,
            2 => Sub,
            3 => SubBy,
            4 => Div,
            5 => DivBy,
            6 => Mul,
            7 => Mod,
            8 => ModBy,

            9 => Xor,
            10 => BitOr,
            11 => BitAnd,

            12 => And,
            13 => Or,
            14 => Not,

            15 => Lt,
            16 => Gt,
            17 => Eq,
            18 => Le,
            19 => Ge,

            20 => If,

            21 => Type,

            22 => Local,
            23 => Push,
            24 => Pop,
            25 => Drop,
            26 => Call,
            27 => Ret,

            28 => Capture,
            29 => CapFromCap,
            30 => Capped,
            31 => PushCapped,

            32 => NewList,
            33 => Collect,
            34 => Insert,
            35 => Append,
            36 => Concat,
            37 => Length,
            38 => Empty,
            39 => Head,
            40 => Rest,

            41 => Input,
            42 => Output,
            43 => Print,
            44 => Flush,

            45 => Byte,
            46 => Num,
            47 => Func,
            48 => Str,

            49 => True,
            50 => False,

            51 => End,

            _ => __,
        }
    }
}
