pub enum ErrorMessage {
    OverFlow,
    UnderFlow,
    EmptyList,
    NotaList,
    NotaClosure,
    RestEmpty,
    HeadEmpty,
    ConcatNotList,
}

impl std::fmt::Display for ErrorMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl ErrorMessage {
    fn to_str(&self) -> &'static str {
        use ErrorMessage::*;
        match self {
            OverFlow => "栈溢出了! :(",
            UnderFlow => "栈见底了! :(",
            EmptyList => "空列表! :(",
            NotaList => "类型错误，这不是列表! :(",
            NotaClosure => "类型错误，这不是闭包! :(",
            RestEmpty => "不可以从空列表中取尾部! :(",
            HeadEmpty => "不可以从空列表中取头部! :(",
            ConcatNotList => "Concat需要两个列表! :(",
        }
    }
}
