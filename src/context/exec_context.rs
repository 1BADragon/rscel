use std::collections::HashMap;

use serde_json::Value;

use crate::{
    value_cell::{ValueCell, ValueCellResult},
    ExecError,
};

use super::default_funcs::load_default_funcs;

pub type RsCellCallback = fn(this: ValueCell, args: ValueCell) -> ValueCellResult<ValueCell>;

pub struct ExecContext {
    params: HashMap<String, Value>,
    funcs: HashMap<String, RsCellCallback>,
}

impl ExecContext {
    pub fn new() -> ExecContext {
        let mut ctx = ExecContext {
            params: HashMap::new(),
            funcs: HashMap::new(),
        };

        load_default_funcs(&mut ctx);
        ctx
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

    pub fn bind_func(&mut self, name: &str, func: RsCellCallback) {
        self.funcs.insert(name.to_owned(), func);
    }

    pub fn param(&self, name: &str) -> Option<&Value> {
        self.params.get(name)
    }

    pub fn func(&self, name: &str) -> Option<&RsCellCallback> {
        self.funcs.get(name)
    }
}
