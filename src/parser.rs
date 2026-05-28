pub fn split_into_lines(source: &str) -> Vec<String> {
    source.lines().map(|line| line.trim().to_string()).collect()
}

pub fn tokenize_line(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut chars = line.char_indices().peekable();

    while let Some((i, c)) = chars.next() {
        if c.is_whitespace() {
            continue;
        }
        if c == '"' {
            let mut s = String::from('"');
            let mut closed = false;
            while let Some((_, ch)) = chars.next() {
                s.push(ch);
                if ch == '"' {
                    closed = true;
                    break;
                }
            }
            if !closed {
                s.push('"');
            }
            tokens.push(s);
        } else {
            let start = i;
            let mut end = start + c.len_utf8();
            while let Some(&(j, ch)) = chars.peek() {
                if ch.is_whitespace() { break; }
                end = j + ch.len_utf8();
                chars.next();
            }
            tokens.push(line[start..end].to_string());
        }
    }

    tokens
}

pub fn parse_value(s: &str) -> Result<crate::Variable, Box<dyn std::error::Error>> {
    if s.starts_with("$") {
        let n = u32::from_str_radix(&s[1..], 16)?;
        Ok(crate::Variable {
            name: String::new(),
            value: {
                // strip leading zero bytes but keep at least one
                let bytes = n.to_be_bytes();
                let start = bytes.iter().position(|&b| b != 0).unwrap_or(3);
                bytes[start..].to_vec()
            },
        })
    } else if s.starts_with("#") {
        let n = u32::from_str_radix(&s[1..], 10)?;
        Ok(crate::Variable {
            name: String::new(),
            value: {
                let bytes = n.to_be_bytes();
                let start = bytes.iter().position(|&b| b != 0).unwrap_or(3);
                bytes[start..].to_vec()
            },
        })
    } else if s.starts_with("\"") && s.ends_with("\"") {
        Ok(crate::Variable {
            name: String::new(),
            value: {
                let mut bytes = s[1..s.len()-1].as_bytes().to_vec();
                bytes.push(0);
                bytes
            },
        })
    } else {
        Err(format!("Invalid value format: '{}'", s).into())
    }
}