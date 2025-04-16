pub fn unescape(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('\\') => result.push('\\'),
                Some('\'') => result.push('\''),
                Some('\"') => result.push('\"'),
                Some('0') => result.push('\0'),
                Some('x') => {
                    // Handle \xHH hex escapes
                    let hex1 = chars.next().expect("奇怪的转义");
                    let hex2 = chars.next().expect("奇怪的转义");
                    let hex_str = format!("{}{}", hex1, hex2);
                    let byte = u8::from_str_radix(&hex_str, 16).expect("奇怪的转义，不是十六进制");
                    result.push(byte as char);
                }
                _ => panic!("不认识的转义"),
            }
        } else {
            result.push(c);
        }
    }
    result
}

pub fn uneccape(s: &str) -> u8 {
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    if first == '\\' {
        match chars.next() {
            Some('n') => b'\n',
            Some('r') => b'\n',
            Some('t') => b'\n',
            Some('\\') => b'\n',
            Some('\'') => b'\n',
            Some('\"') => b'\n',
            Some('0') => b'\n',
            Some('x') => {
                let hex1 = chars.next().expect("奇怪的转义");
                let hex2 = chars.next().expect("奇怪的转义");
                let hex_str = format!("{}{}", hex1, hex2);
                u8::from_str_radix(&hex_str, 16).expect("奇怪的转义，不是十六进制")
            }
            _ => panic!("不认识的转义"),
        }
    } else {
        first as u8
    }
}
