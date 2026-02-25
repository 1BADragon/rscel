use crate::{CelError, CelValue};

pub mod contains;
pub mod ends_with;
pub mod index_of;
pub mod match_captures;
pub mod match_captures_all;
pub mod match_replace;
pub mod match_replace_once;
pub mod matches;
pub mod remove;
pub mod repeat;
pub mod replace;
pub mod replace_i;
pub mod split;
pub mod split_whitespace;
pub mod starts_with;
pub mod trim_end_matches;
pub mod trim_matches;
pub mod trim_start_matches;

macro_rules! string_func {
    ($cel_func_name: ident, $func_name:ident, $str_func:ident) => {
        pub fn $func_name(this: CelValue, args: Vec<CelValue>) -> CelValue {
            if args.len() > 0 {
                return CelValue::from_err(CelError::argument(
                    "$cel_func_name does not take any argments",
                ));
            }

            if let CelValue::String(s) = this {
                CelValue::String(s.$str_func().chars().collect())
            } else {
                return CelValue::from_err(CelError::value(
                    "$cel_func_name only available on string",
                ));
            }
        }
    };
}

string_func!(toLower, to_lower_impl, to_lowercase);
string_func!(toUpper, to_upper_impl, to_uppercase);
string_func!(trim, trim_impl, trim);
string_func!(trimStart, trim_start_impl, trim_start);
string_func!(trimEnd, trim_end_impl, trim_end);

pub fn pad_start_impl(this: CelValue, args: Vec<CelValue>) -> CelValue {
    let s = match this {
        CelValue::String(ref s) => s.clone(),
        _ => {
            return CelValue::from_err(CelError::value("padStart is only available on strings"))
        }
    };

    let width = match args.first() {
        Some(CelValue::Int(n)) => *n as usize,
        Some(CelValue::UInt(n)) => *n as usize,
        _ => {
            return CelValue::from_err(CelError::argument(
                "padStart() requires a numeric width argument",
            ))
        }
    };

    let pad_char = match args.get(1) {
        Some(CelValue::String(p)) => {
            let mut chars = p.chars();
            match chars.next() {
                Some(c) if chars.next().is_none() => c,
                _ => {
                    return CelValue::from_err(CelError::argument(
                        "padStart() pad character must be a single character",
                    ))
                }
            }
        }
        None => ' ',
        _ => {
            return CelValue::from_err(CelError::argument(
                "padStart() pad character must be a string",
            ))
        }
    };

    let char_len = s.chars().count();
    if char_len >= width {
        return this;
    }
    let pad_count = width - char_len;
    let padded = std::iter::repeat(pad_char)
        .take(pad_count)
        .chain(s.chars())
        .collect::<String>();
    padded.into()
}

pub fn pad_end_impl(this: CelValue, args: Vec<CelValue>) -> CelValue {
    let s = match this {
        CelValue::String(ref s) => s.clone(),
        _ => {
            return CelValue::from_err(CelError::value("padEnd is only available on strings"))
        }
    };

    let width = match args.first() {
        Some(CelValue::Int(n)) => *n as usize,
        Some(CelValue::UInt(n)) => *n as usize,
        _ => {
            return CelValue::from_err(CelError::argument(
                "padEnd() requires a numeric width argument",
            ))
        }
    };

    let pad_char = match args.get(1) {
        Some(CelValue::String(p)) => {
            let mut chars = p.chars();
            match chars.next() {
                Some(c) if chars.next().is_none() => c,
                _ => {
                    return CelValue::from_err(CelError::argument(
                        "padEnd() pad character must be a single character",
                    ))
                }
            }
        }
        None => ' ',
        _ => {
            return CelValue::from_err(CelError::argument(
                "padEnd() pad character must be a string",
            ))
        }
    };

    let char_len = s.chars().count();
    if char_len >= width {
        return this;
    }
    let pad_count = width - char_len;
    let padded = s
        .chars()
        .chain(std::iter::repeat(pad_char).take(pad_count))
        .collect::<String>();
    padded.into()
}
