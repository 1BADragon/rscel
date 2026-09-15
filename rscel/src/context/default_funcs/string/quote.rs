use crate::{CelError, CelValue};

pub fn quote(this: CelValue, args: Vec<CelValue>) -> CelValue {
    // Support both "str".quote() and strings.quote(str)
    let s = match this {
        CelValue::String(s) if args.is_empty() => s,
        _ => match args.first() {
            Some(CelValue::String(s)) => s.clone(),
            _ => return CelValue::from_err(CelError::argument("quote() requires a string argument")),
        },
    };

    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\x07' => out.push_str("\\a"),
            '\x08' => out.push_str("\\b"),
            '\x0b' => out.push_str("\\v"),
            '\x0c' => out.push_str("\\f"),
            c if c < '\x20' => {
                out.push_str(&format!("\\x{:02x}", c as u32));
            }
            _ => out.push(c),
        }
    }
    out.push('"');
    CelValue::String(out)
}
