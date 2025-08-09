use crate::macros::dispatch;

pub use methods::dispatch as match_replace_once;

#[dispatch]
mod methods {
    use crate::CelValue;

    fn match_replace_once(this: String, needle: String, rep: String) -> CelValue {
        internal::match_replace_once(&this, &needle, &rep)
    }

    fn match_replace_once(haystack: String, needle: String, rep: String) -> CelValue {
        internal::match_replace_once(&haystack, &needle, &rep)
    }

    mod internal {
        use regex::Regex;

        use crate::{CelError, CelValue};

        pub fn match_replace_once(haystack: &str, needle: &str, rep: &str) -> CelValue {
            match Regex::new(needle) {
                Ok(re) => re.replace(haystack, rep).into_owned().into(),
                Err(err) => {
                    return CelError::value(&format!("Invalid regular expression: {}", err)).into()
                }
            }
        }
    }
}
