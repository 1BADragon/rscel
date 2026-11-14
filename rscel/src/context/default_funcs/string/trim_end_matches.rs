use rscel_macro::dispatch;

pub use trim_end_matches_methods::dispatch as trim_end_matches;

#[dispatch]
mod trim_end_matches_methods {
    use crate::CelValue;

    fn trim_end_matches(this: String, pattern: String) -> String {
        this.trim_end_matches(&pattern).to_owned()
    }
}
