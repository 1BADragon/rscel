use crate::macros::dispatch;

pub use methods::dispatch as replace_i;

#[dispatch]
mod methods {
    use crate::CelValue;

    fn replace_i(this: String, needle: String, to: String) -> CelValue {
        internal::replace_i(&this, &needle, &to)
    }

    mod internal {
        use regex::NoExpand;

        use super::super::super::regex_cache::with_regex;

        use crate::CelValue;

        pub fn replace_i(haystack: &str, needle: &str, to: &str) -> CelValue {
            let pattern = format!("(?i){}", regex::escape(needle));
            with_regex(&pattern, |re| re.replace_all(haystack, NoExpand(to)).into_owned().into())
                .unwrap_or_else(|e| e.into())
        }
    }
}
