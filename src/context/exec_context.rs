use std::collections::HashMap;

use serde_json::Value;

pub struct ExecContext {
    params: HashMap<String, Value>,
}

impl ExecContext {
    pub fn new() -> ExecContext {
        ExecContext {
            params: HashMap::new(),
        }
    }

    pub fn bind_param(&mut self, name: &str, value: Value) {
        self.params.insert(name.to_owned(), value);
    }

    pub fn param(&self, name: &str) -> Option<&Value> {
        self.params.get(name)
    }
}
