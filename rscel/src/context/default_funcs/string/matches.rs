use crate::macros::dispatch;

pub use methods::dispatch as matches;

#[dispatch]
mod methods {
    use crate::{CelResult, CelValue};

    fn matches(this: String, needle: String) -> CelResult<bool> {
        internal::matches(&this, &needle)
    }

    mod internal {
        use regex::Regex;

        use crate::{CelError, CelResult};

        pub fn matches(haystack: &str, needle: &str) -> CelResult<bool> {
            match Regex::new(needle) {
                Ok(re) => return Ok(re.is_match(haystack)),
                Err(err) => {
                    return Err(CelError::value(&format!(
                        "Invalid regular expression: {}",
                        err
                    )))
                }
            }
        }
    }
}
