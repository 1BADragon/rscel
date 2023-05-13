use std::collections::HashMap;

use serde_json::Value;

use crate::ExecError;

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

    pub fn bind_params_from_json(&mut self, values: Value) -> Result<(), ExecError> {
        let obj = if let Value::Object(o) = values {
            o
        } else {
            return Err(ExecError::new("Binding must be an object"));
        };

        for (key, value) in obj.into_iter() {
            self.params.insert(key, value);
        }
        Ok(())
    }

    pub fn param(&self, name: &str) -> Option<&Value> {
        self.params.get(name)
    }
}
