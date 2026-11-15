use crate::{CelError, CelValue};

pub fn char_at(this: CelValue, args: Vec<CelValue>) -> CelValue {
    let s = match this {
        CelValue::String(s) => s,
        _ => return CelValue::from_err(CelError::value("charAt() only available on string")),
    };

    let idx = match args.first() {
        Some(CelValue::Int(n)) => *n as usize,
        Some(CelValue::UInt(n)) => *n as usize,
        _ => {
            return CelValue::from_err(CelError::argument(
                "charAt() requires a numeric index argument",
            ))
        }
    };

    match s.chars().nth(idx) {
        Some(c) => CelValue::String(c.to_string()),
        None => CelValue::String(String::new()),
    }
}
