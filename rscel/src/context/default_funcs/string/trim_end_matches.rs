use rscel_macro::dispatch;

pub use trim_end_matches::dispatch as trim_end_matches;

#[dispatch]
mod trim_end_matches {
    use crate::CelValue;

    fn trim_end_matches(this: String, pattern: String) -> String {
        this.trim_end_matches(&pattern).to_owned()
    }
}
