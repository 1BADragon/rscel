use std::{cell::RefCell, collections::HashMap, rc::Rc};

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

struct ExecContextInternal {
    params: HashMap<String, ValueCell>,
    funcs: HashMap<String, RsCellFunction>,
    macros: HashMap<String, RsCellMacro>,
}

#[derive(Clone)]
pub struct ExecContext {
    inner: Rc<RefCell<ExecContextInternal>>,
}

impl ExecContext {
    pub fn new() -> ExecContext {
        let mut ctx = ExecContext {
            inner: Rc::new(RefCell::new(ExecContextInternal {
                params: HashMap::new(),
                funcs: HashMap::new(),
                macros: HashMap::new(),
            })),
        };

        load_default_macros(&mut ctx);
        load_default_funcs(&mut ctx);
        ctx
    }

    pub fn bind_param(&mut self, name: &str, value: ValueCell) {
        self.inner
            .borrow_mut()
            .params
            .insert(name.to_owned(), value);
    }

    pub fn bind_params_from_json_obj(&mut self, values: Value) -> Result<(), ExecError> {
        let obj = if let Value::Object(o) = values {
            o
        } else {
            return Err(ExecError::new("Binding must be an object"));
        };

        for (key, value) in obj.into_iter() {
            self.inner
                .borrow_mut()
                .params
                .insert(key, ValueCell::from(value));
        }
        Ok(())
    }

    pub fn bind_func(&mut self, name: &str, func: RsCellFunction) {
        self.inner.borrow_mut().funcs.insert(name.to_owned(), func);
    }

    pub fn bind_macro(&mut self, name: &str, macro_: RsCellMacro) {
        self.inner
            .borrow_mut()
            .macros
            .insert(name.to_owned(), macro_);
    }

    pub fn get_param(&self, name: &str) -> Option<ValueCell> {
        Some(self.inner.borrow().params.get(name)?.clone())
    }

    pub fn get_func(&self, name: &str) -> Option<RsCellFunction> {
        Some(self.inner.borrow().funcs.get(name)?.clone())
    }

    pub fn get_macro(&self, name: &str) -> Option<RsCellMacro> {
        Some(self.inner.borrow().macros.get(name)?.clone())
    }
}
