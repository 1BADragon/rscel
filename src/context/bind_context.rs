use std::collections::HashMap;

use serde_json::Value;

use crate::{
    cel_error::CelResult,
    interp::{ByteCode, Interpreter},
    CelError, CelValue,
};

use super::default_funcs::load_default_funcs;
use super::default_macros::load_default_macros;

/// Prototype for a function binding.
///
/// Rust-accelerated functions can be used to extend
/// the base language's functionality. The `this` argument refers to an object the function
/// is being run on (i.e. `foo.bar()`). The `args` argument refers to any arguments passed
/// to the function within the parenthesis, in order passed. The return value of a function
/// should be a single `ValueCell` wrapped in a `ValueCellResult`.
///
/// An example of a function impl that collects the keys of an object and returns them as a
/// list:
/// ```
/// use rscel::*;
///
/// fn keys_impl(this: CelValue, args: &[CelValue]) -> CelResult<CelValue> {
///     if args.len() == 0 {
///         return Err(CelError::misc("keys() expects 0 arguments"));    
///     }
///     
///     if let CelValue::Map(map) = this {
///         Ok(CelValue::from_list(map.keys().map(|x| x.as_str().into()).collect()))
///     } else {
///        Err(CelError::misc("keys() only supported for map type"))
///     }
/// }
/// ```
pub type RsCelFunction = dyn Fn(CelValue, &[CelValue]) -> CelResult<CelValue>;

/// Prototype for a macro binding.
///
/// Rust-accelerated macros can be used to modify and evaluate
/// program bytecode within the macro arguments. Instead of being given values, macros are
/// provided the bytecode generated from the argument expression. Like functions, the this
/// argument is the resolved value the macro is being run on (i.e `my_list.map()`) however
/// all arguents passed to the macro are left unresolved bytecode. An additional argument,
/// the Interpreter context, is provided to the macro for bytecode resolution.
pub type RsCelMacro =
    dyn for<'a, 'b> Fn(&'a Interpreter<'a>, CelValue, &'b [&'b [ByteCode]]) -> CelResult<CelValue>;

/// Bindings context for a cel evaluation.
///
/// This struct houses all of the bindings (dynamic values) for a cel evaluation.
/// Currently the types of identifiers that can be bound to are variables, functions
/// and macros. This context is separate from the contents of the `CelContext` to allow
/// for multiple runs with different bound values on the same programs without the need
/// to maintain multiple copies of the programs.
#[derive(Clone)]
pub struct BindContext<'a> {
    params: HashMap<String, CelValue>,
    funcs: HashMap<String, &'a RsCelFunction>,
    macros: HashMap<String, &'a RsCelMacro>,
}

impl<'a> BindContext<'a> {
    /// Create a new bind context contain default functions and macros.
    pub fn new() -> BindContext<'a> {
        let mut ctx = BindContext {
            params: HashMap::new(),
            funcs: HashMap::new(),
            macros: HashMap::new(),
        };

        load_default_macros(&mut ctx);
        load_default_funcs(&mut ctx);
        ctx
    }

    /// Bind a param with the given name and value.
    pub fn bind_param(&mut self, name: &str, value: CelValue) {
        self.params.insert(name.to_owned(), value);
    }

    /// Cheater function to bind the keys of an JSON object with its values
    pub fn bind_params_from_json_obj(&mut self, values: Value) -> CelResult<()> {
        let obj = if let Value::Object(o) = values {
            o
        } else {
            return Err(CelError::misc("Binding must be an object"));
        };

        for (key, value) in obj.into_iter() {
            self.params.insert(key, CelValue::from(value));
        }
        Ok(())
    }

    /// Bind a function to the bind context, can be new or overwrite an existing (including default)
    pub fn bind_func(&mut self, name: &str, func: &'a RsCelFunction) {
        self.funcs.insert(name.to_owned(), func);
    }

    /// Bind a macro to the bind context.
    pub fn bind_macro(&mut self, name: &str, macro_: &'a RsCelMacro) {
        self.macros.insert(name.to_owned(), macro_);
    }

    /// Get a param by name.
    pub fn get_param<'l>(&'l self, name: &str) -> Option<&'l CelValue> {
        Some(self.params.get(name)?)
    }

    /// Get a function by name.
    pub fn get_func(&self, name: &str) -> Option<&'a RsCelFunction> {
        Some(*self.funcs.get(name)?)
    }

    /// Get a macro by name.
    pub fn get_macro(&self, name: &str) -> Option<&'a RsCelMacro> {
        Some(*self.macros.get(name)?)
    }

    pub fn is_bound(&self, name: &str) -> bool {
        self.params.contains_key(name)
            || self.funcs.contains_key(name)
            || self.macros.contains_key(name)
    }
}
