use std::cell::RefCell;
use std::collections::HashMap;

use regex::Regex;

use crate::CelError;

thread_local! {
    static REGEX_CACHE: RefCell<HashMap<String, Regex>> = RefCell::new(HashMap::new());
}

pub fn with_regex<T>(pattern: &str, f: impl FnOnce(&Regex) -> T) -> Result<T, CelError> {
    REGEX_CACHE.with(|cache| {
        let mut map = cache.borrow_mut();
        if !map.contains_key(pattern) {
            match Regex::new(pattern) {
                Ok(re) => {
                    map.insert(pattern.to_owned(), re);
                }
                Err(err) => {
                    return Err(CelError::value(&format!(
                        "Invalid regular expression: {}",
                        err
                    )));
                }
            }
        }
        Ok(f(map.get(pattern).unwrap()))
    })
}
