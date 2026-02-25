use crate::macros::dispatch;

pub use methods::dispatch as match_captures_all;

#[dispatch]
mod methods {
    use crate::CelValue;

    fn match_captures_all(this: String, needle: String) -> CelValue {
        internal::match_captures_all(&this, &needle)
    }

    mod internal {
        use regex::Regex;

        use crate::{CelError, CelValue};

        pub fn match_captures_all(haystack: &str, needle: &str) -> CelValue {
            match Regex::new(needle) {
                Ok(re) => re
                    .captures_iter(haystack)
                    .map(|caps| {
                        caps.iter()
                            .map(|s| match s {
                                Some(m) => m.as_str().into(),
                                None => CelValue::Null,
                            })
                            .collect::<Vec<CelValue>>()
                            .into()
                    })
                    .collect::<Vec<CelValue>>()
                    .into(),
                Err(err) => {
                    CelError::value(&format!("Invalid regular expression: {}", err)).into()
                }
            }
        }
    }
}
