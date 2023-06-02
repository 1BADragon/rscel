use std::{collections::HashMap, fmt};

use serde_json::Value;

use crate::{
    interp::{ByteCode, Interpreter},
    value_cell::{ValueCell, ValueCellResult},
    ExecError,
};

use super::default_funcs::load_default_funcs;

pub type RsCellFunction = fn(this: ValueCell, args: ValueCell) -> ValueCellResult<ValueCell>;
pub type RsCellMacro = for<'a> fn(
    ctx: &'a Interpreter<'a>,
    this: ValueCell,
    inner: &'a [&'a [ByteCode]],
) -> ValueCellResult<ValueCell>;

#[derive(Clone)]
pub enum RsCallable {
    Function(RsCellFunction),
    Macro(RsCellMacro),
}

impl fmt::Debug for RsCallable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Function(_) => write!(f, "Function"),
            Self::Macro(_) => write!(f, "Macro"),
        }
    }
}

impl PartialEq for RsCallable {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

#[derive(Clone)]
pub struct BindContext {
    params: HashMap<String, ValueCell>,
    funcs: HashMap<String, RsCellFunction>,
    macros: HashMap<String, RsCellMacro>,
}

impl BindContext {
    pub fn new() -> BindContext {
        let mut ctx = BindContext {
            params: HashMap::new(),
            funcs: HashMap::new(),
            macros: HashMap::new(),
        };

        // load_default_macros(&mut ctx);
        load_default_funcs(&mut ctx);
        ctx
    }

    pub fn bind_param(&mut self, name: &str, value: ValueCell) {
        self.params.insert(name.to_owned(), value);
    }

    pub fn bind_params_from_json_obj(&mut self, values: Value) -> Result<(), ExecError> {
        let obj = if let Value::Object(o) = values {
            o
        } else {
            return Err(ExecError::new("Binding must be an object"));
        };

        for (key, value) in obj.into_iter() {
            self.params.insert(key, ValueCell::from(value));
        }
        Ok(())
    }

    pub fn bind_func(&mut self, name: &str, func: RsCellFunction) {
        self.funcs.insert(name.to_owned(), func);
    }

    pub fn bind_macro(&mut self, name: &str, macro_: RsCellMacro) {
        self.macros.insert(name.to_owned(), macro_);
    }

    pub fn get_param<'a>(&'a self, name: &str) -> Option<&'a ValueCell> {
        Some(self.params.get(name)?)
    }

    pub fn get_func(&self, name: &str) -> Option<RsCellFunction> {
        Some(self.funcs.get(name)?.clone())
    }

    pub fn get_macro(&self, name: &str) -> Option<RsCellMacro> {
        Some(self.macros.get(name)?.clone())
    }
}
