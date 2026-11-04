//! Echo normalization for history rows (doc 03 §2, doc 07 §5).
//!
//! Typed input → stored echo: `*` → `×`, `/` → `÷`, `-` → `−` (every
//! hyphen-minus becomes a true minus U+2212); `^ % ! ( ) .` and
//! superscript/subscript tokens stay as-is; whitespace is preserved.
//! Echoes are never scientific-ified. With `group` set, numeric literals
//! get en-US thousands grouping on the integer part (new-row echoes only —
//! the app keeps stored echoes of older rows untouched).

/// Normalize typed input for a history-row echo.
pub fn normalize_echo(expr: &str, group: bool) -> String {
    let mut out = String::with_capacity(expr.len() + 8);
    let chars: Vec<char> = expr.chars().collect();
    let mut i = 0;
    let prev_alnumish = |out: &str| {
        out.chars()
            .next_back()
            .map(|c| c.is_ascii_alphanumeric() || c == '_')
            .unwrap_or(false)
    };
    while i < chars.len() {
        let c = chars[i];
        match c {
            '*' => out.push('×'),
            '/' => out.push('÷'),
            '-' => out.push('−'),
            '0'..='9' => {
                // numeric literal: digits [. digits]; group the integer part
                // when grouping is on and the literal does not continue an
                // identifier (x1234) or hex-letters (FF12 → run starts at F,
                // preceded by a letter — handled by prev check at run start)
                if !group || prev_alnumish(&out) {
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        out.push(chars[i]);
                        i += 1;
                    }
                    continue;
                }
                let start = out.len();
                while i < chars.len() && chars[i].is_ascii_digit() {
                    out.push(chars[i]);
                    i += 1;
                }
                if i < chars.len() && chars[i] == '.' {
                    out.push('.');
                    i += 1;
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        out.push(chars[i]);
                        i += 1;
                    }
                }
                if group {
                    let literal: String = out[start..].to_string();
                    let (int_part, rest) = match literal.split_once('.') {
                        Some((a, b)) => (a.to_string(), format!(".{b}")),
                        None => (literal.clone(), String::new()),
                    };
                    let grouped = if int_part.len() > 3 {
                        group_int(&int_part)
                    } else {
                        int_part.clone()
                    };
                    out.truncate(start);
                    out.push_str(&grouped);
                    out.push_str(&rest);
                }
                continue;
            }
            _ => out.push(c),
        }
        i += 1;
    }
    out
}

fn group_int(digits: &str) -> String {
    let bytes = digits.as_bytes();
    let mut out = String::with_capacity(bytes.len() + bytes.len() / 3);
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(*b as char);
    }
    out
}
