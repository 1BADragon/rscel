use crate::{CelError, CelValue};

pub fn substring(this: CelValue, args: Vec<CelValue>) -> CelValue {
    let s = match this {
        CelValue::String(s) => s,
        _ => return CelValue::from_err(CelError::value("substring() only available on string")),
    };

    let start = match args.first() {
        Some(CelValue::Int(n)) => *n as usize,
        Some(CelValue::UInt(n)) => *n as usize,
        _ => {
            return CelValue::from_err(CelError::argument(
                "substring() requires a numeric start argument",
            ))
        }
    };

    let char_count = s.chars().count();

    let end = match args.get(1) {
        Some(CelValue::Int(n)) => *n as usize,
        Some(CelValue::UInt(n)) => *n as usize,
        None => char_count,
        _ => {
            return CelValue::from_err(CelError::argument(
                "substring() end must be numeric",
            ))
        }
    };

    if start > char_count || end > char_count || start > end {
        return CelValue::from_err(CelError::value("substring() index out of range"));
    }

    let result: String = s.chars().skip(start).take(end - start).collect();
    CelValue::String(result)
}
