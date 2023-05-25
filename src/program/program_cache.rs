use super::{Program, ProgramResult};

#[cfg(not(feature = "program_cache"))]
pub fn check_cache(_source: &str) -> Option<ProgramResult<Program>> {
    None
}

#[cfg(feature = "program_cache")]
pub fn check_cache(source: &str) -> Option<ProgramResult<Program>> {
    Some(internal::CACHE.get(source))
}

#[cfg(feature = "program_cache")]
mod internal {
    use crate::{program::ProgramResult, Program};
    use once_cell::sync::Lazy;
    use std::{collections::HashMap, sync::Mutex};

    pub struct ProgramCache {
        cache: Mutex<HashMap<String, Program>>,
    }

    pub static CACHE: Lazy<ProgramCache> = Lazy::new(|| ProgramCache::new());

    unsafe impl Send for ProgramCache {}
    unsafe impl Sync for ProgramCache {}

    impl ProgramCache {
        pub fn new() -> ProgramCache {
            ProgramCache {
                cache: Mutex::new(HashMap::new()),
            }
        }

        pub fn get(&self, source: &str) -> ProgramResult<Program> {
            Ok(self
                .cache
                .lock()
                .unwrap()
                .entry(source.to_owned())
                .or_insert(Program::from_source_nocache(source)?)
                .clone())
        }
    }
}
