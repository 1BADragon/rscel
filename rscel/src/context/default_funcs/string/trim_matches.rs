use crate::macros::dispatch;

pub use methods::dispatch as trim_matches;

#[dispatch]
mod methods {
    use crate::CelValue;

    fn trim_matches(this: String, pattern: String) -> String {
        this.trim_matches(|c: char| pattern.contains(c)).to_owned()
    }
}
