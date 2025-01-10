use crate::{CelError, CelValue};

pub mod contains;
pub mod ends_with;
pub mod matches;
pub mod split_whitespace;
pub mod starts_with;

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
