use crate::macros::dispatch;

pub use methods::dispatch as matches;

#[dispatch]
mod methods {
    use crate::{CelResult, CelValue};

    fn matches(this: String, needle: String) -> CelResult<bool> {
        internal::matches(&this, &needle)
    }

    mod internal {
        use super::super::super::regex_cache::with_regex;

        use crate::CelResult;

        pub fn matches(haystack: &str, needle: &str) -> CelResult<bool> {
            with_regex(needle, |re| re.is_match(haystack))
        }
    }
}
