use rscel_macro::dispatch;

pub use trim_start_matches_methods::dispatch as trim_start_matches;

#[dispatch]
mod trim_start_matches_methods {
    use crate::CelValue;

    fn trim_start_matches(this: String, pattern: String) -> String {
        this.trim_start_matches(&pattern).to_owned()
    }
}
