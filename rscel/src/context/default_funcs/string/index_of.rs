use crate::macros::dispatch;

pub use index_of_methods::dispatch as index_of;
pub use last_index_of_methods::dispatch as last_index_of;

#[dispatch]
mod index_of_methods {
    use crate::CelValue;

    fn index_of(this: String, needle: String) -> i64 {
        match this.find(needle.as_str()) {
            Some(byte_idx) => this[..byte_idx].chars().count() as i64,
            None => -1,
        }
    }
}

#[dispatch]
mod last_index_of_methods {
    use crate::CelValue;

    fn last_index_of(this: String, needle: String) -> i64 {
        match this.rfind(needle.as_str()) {
            Some(byte_idx) => this[..byte_idx].chars().count() as i64,
            None => -1,
        }
    }
}
