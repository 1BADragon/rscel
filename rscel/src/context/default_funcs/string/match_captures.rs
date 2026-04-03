use crate::macros::dispatch;

pub use methods::dispatch as match_captures;

#[dispatch]
mod methods {
    use crate::CelValue;

    fn match_captures(this: String, needle: String) -> CelValue {
        internal::matches(&this, &needle)
    }

    mod internal {
        use super::super::super::regex_cache::with_regex;

        use crate::CelValue;

        pub fn matches(haystack: &str, needle: &str) -> CelValue {
            with_regex(needle, |re| match re.captures(haystack) {
                Some(c) => c
                    .iter()
                    .map(|s| match s {
                        Some(s) => s.as_str().into(),
                        None => CelValue::Null,
                    })
                    .collect::<Vec<_>>()
                    .into(),
                None => CelValue::Null,
            })
            .unwrap_or_else(|e| e.into())
        }
    }
}
