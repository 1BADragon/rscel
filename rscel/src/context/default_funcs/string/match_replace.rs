use crate::macros::dispatch;

pub use methods::dispatch as match_replace;

#[dispatch]
mod methods {
    use crate::CelValue;

    fn match_replace(this: String, needle: String, rep: String) -> CelValue {
        internal::match_replace(&this, &needle, &rep)
    }

    mod internal {
        use regex::Regex;

        use crate::{CelError, CelValue};

        pub fn match_replace(haystack: &str, needle: &str, rep: &str) -> CelValue {
            match Regex::new(needle) {
                Ok(re) => re.replace_all(haystack, rep).into_owned().into(),
                Err(err) => {
                    return CelError::value(&format!("Invalid regular expression: {}", err)).into()
                }
            }
        }
    }
}
