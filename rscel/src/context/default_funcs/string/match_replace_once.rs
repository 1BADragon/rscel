use crate::macros::dispatch;

pub use methods::dispatch as match_replace_once;

#[dispatch]
mod methods {
    use crate::CelValue;

    fn match_replace_once(this: String, needle: String, rep: String) -> CelValue {
        internal::match_replace_once(&this, &needle, &rep)
    }

    mod internal {
        use super::super::super::regex_cache::with_regex;

        use crate::CelValue;

        pub fn match_replace_once(haystack: &str, needle: &str, rep: &str) -> CelValue {
            with_regex(needle, |re| re.replace(haystack, rep).into_owned().into())
                .unwrap_or_else(|e| e.into())
        }
    }
}
