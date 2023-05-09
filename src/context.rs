use std::collections::HashMap;

use crate::program::Program;

pub struct ExecContext {
    progs: HashMap<String, Program>,
}
