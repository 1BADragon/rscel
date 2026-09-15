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

    fn index_of(this: String, needle: String, offset: i64) -> i64 {
        let char_offset = offset as usize;
        let byte_offset: usize = this.char_indices().nth(char_offset).map(|(i, _)| i).unwrap_or(this.len());
        match this[byte_offset..].find(needle.as_str()) {
            Some(byte_idx) => this[..byte_offset + byte_idx].chars().count() as i64,
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

    fn last_index_of(this: String, needle: String, offset: i64) -> i64 {
        let max_char_pos = offset as usize;
        let needle_chars = needle.chars().count();
        // Find the last position <= offset where needle starts
        // Search backwards from offset down to 0
        for start in (0..=max_char_pos).rev() {
            let byte_start = this.char_indices().nth(start).map(|(i, _)| i).unwrap_or(this.len());
            let end_char = start + needle_chars;
            let byte_end = this.char_indices().nth(end_char).map(|(i, _)| i).unwrap_or(this.len());
            if byte_end <= this.len() && &this[byte_start..byte_end] == needle.as_str() {
                return start as i64;
            }
        }
        -1
    }
}
