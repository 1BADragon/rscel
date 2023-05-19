use std::collections::HashMap;

use serde_json::Value;

use crate::{
    ast::grammar::Expr,
    value_cell::{ValueCell, ValueCellResult},
    CelContext, ExecError,
};

use super::{default_funcs::load_default_funcs, default_macros::load_default_macros};

pub type RsCellFunction = fn(this: ValueCell, args: ValueCell) -> ValueCellResult<ValueCell>;
pub type RsCellMacro =
    fn(ctx: &CelContext, this: ValueCell, inner: &[&Expr]) -> ValueCellResult<ValueCell>;

#[derive(Clone)]
pub struct ExecContext {
    params: HashMap<String, Value>,
    funcs: HashMap<String, RsCellFunction>,
    macros: HashMap<String, RsCellMacro>,
}

impl ExecContext {
    pub fn new() -> ExecContext {
        let mut ctx = ExecContext {
            params: HashMap::new(),
            funcs: HashMap::new(),
            macros: HashMap::new(),
        };

        load_default_macros(&mut ctx);
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

    pub fn bind_func(&mut self, name: &str, func: RsCellFunction) {
        self.funcs.insert(name.to_owned(), func);
    }

    pub fn bind_macro(&mut self, name: &str, macro_: RsCellMacro) {
        self.macros.insert(name.to_owned(), macro_);
    }

    pub fn get_param(&self, name: &str) -> Option<&Value> {
        self.params.get(name)
    }

    pub fn get_func(&self, name: &str) -> Option<&RsCellFunction> {
        self.funcs.get(name)
    }

    pub fn get_macro(&self, name: &str) -> Option<&RsCellMacro> {
        self.macros.get(name)
    }
}
