use super::{Program, ProgramResult};

#[cfg(not(feature = "program_cache"))]
pub fn check_cache(_source: &str) -> Option<ProgramResult<Program>> {
    None
}

#[cfg(feature = "program_cache")]
pub fn check_cache(source: &str) -> Option<ProgramResult<Program>> {
    Some(internal::CACHE.lock().unwrap().get(source))
}

#[cfg(feature = "program_cache")]
mod internal {
    use crate::{program::ProgramResult, Program};
    use once_cell::sync::Lazy;
    use std::{
        collections::{hash_map::Entry, HashMap},
        sync::{Arc, Mutex},
    };

    pub struct ProgramCache {
        cache: HashMap<String, Program>,
    }

    pub static CACHE: Lazy<Arc<Mutex<ProgramCache>>> =
        Lazy::new(|| Arc::new(Mutex::new(ProgramCache::new())));

    // unsafe impl Send for ProgramCache {}
    // unsafe impl Sync for ProgramCache {}

    impl ProgramCache {
        pub fn new() -> ProgramCache {
            ProgramCache {
                cache: HashMap::new(),
            }
        }

        pub fn get(&mut self, source: &str) -> ProgramResult<Program> {
            match self.cache.entry(source.to_string()) {
                Entry::Occupied(o) => Ok(o.get().clone()),
                Entry::Vacant(v) => {
                    let prog = Program::from_source_nocache(source)?;
                    v.insert(prog.clone());
                    Ok(prog)
                }
            }
        }
    }
}
