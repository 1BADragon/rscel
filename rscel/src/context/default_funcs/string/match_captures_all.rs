use crate::macros::dispatch;

pub use methods::dispatch as match_captures_all;

#[dispatch]
mod methods {
    use crate::CelValue;

    fn match_captures_all(this: String, needle: String) -> CelValue {
        internal::match_captures_all(&this, &needle)
    }

    mod internal {
        use super::super::super::regex_cache::with_regex;

        use crate::CelValue;

        pub fn match_captures_all(haystack: &str, needle: &str) -> CelValue {
            with_regex(needle, |re| {
                re.captures_iter(haystack)
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
                    .into()
            })
            .unwrap_or_else(|e| e.into())
        }
    }
}
