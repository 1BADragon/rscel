use crate::macros::dispatch;

pub use methods::dispatch as replace_i;

#[dispatch]
mod methods {
    use crate::CelValue;

    fn replace_i(this: String, needle: String, to: String) -> CelValue {
        internal::replace_i(&this, &needle, &to)
    }

    mod internal {
        use regex::{NoExpand, Regex};

        use crate::{CelError, CelValue};

        pub fn replace_i(haystack: &str, needle: &str, to: &str) -> CelValue {
            let pattern = format!("(?i){}", regex::escape(needle));
            match Regex::new(&pattern) {
                Ok(re) => re.replace_all(haystack, NoExpand(to)).into_owned().into(),
                Err(err) => {
                    CelError::value(&format!("Invalid pattern: {}", err)).into()
                }
            }
        }
    }
}
