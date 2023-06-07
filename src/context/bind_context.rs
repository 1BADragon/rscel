use std::{collections::HashMap, fmt};

use serde_json::Value;

use crate::{
    interp::{ByteCode, Interpreter},
    value_cell::{ValueCell, ValueCellResult},
    ExecError,
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
/// fn keys_impl(this: ValueCell, args: &[ValueCell]) -> ValueCellResult<ValueCell> {
///     if args.len() == 0 {
///         return Err(ValueCellError::with_msg("keys() expects 0 arguments"));    
///     }
///     
///     if let ValueCellInner::Map(map) = this.into_inner() {
///         Ok(ValueCell::from_list(map.keys().map(|x| x.as_str().into()).collect()))
///     } else {
///        Err(ValueCellError::with_msg("keys() only supported for map type"))
///     }
/// }
/// ```
pub type RsCelFunction = fn(this: ValueCell, args: &[ValueCell]) -> ValueCellResult<ValueCell>;

/// Prototype for a macro binding.
///
/// Rust-accelerated macros can be used to modify and evaluate
/// program bytecode within the macro arguments. Instead of being given values, macros are
/// provided the bytecode generated from the argument expression. Like functions, the this
/// argument is the resolved value the macro is being run on (i.e `my_list.map()`) however
/// all arguents passed to the macro are left unresolved bytecode. An additional argument,
/// the Interpreter context, is provided to the macro for bytecode resolution.
pub type RsCelMacro = for<'a> fn(
    ctx: &'a Interpreter<'a>,
    this: ValueCell,
    inner: &'a [&'a [ByteCode]],
) -> ValueCellResult<ValueCell>;

/// Wrapper enum that contains either an RsCelCallable or an RsCelFunction. Used
/// as a ValueCell value.
#[derive(Clone)]
pub enum RsCallable {
    Function(RsCelFunction),
    Macro(RsCelMacro),
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

/// Bindings context for a cel evaluation.
///
/// This struct houses all of the bindings (dynamic values) for a cel evaluation.
/// Currently the types of identifiers that can be bound to are variables, functions
/// and macros. This context is separate from the contents of the `CelContext` to allow
/// for multiple runs with different bound values on the same programs without the need
/// to maintain multiple copies of the programs.
#[derive(Clone)]
pub struct BindContext {
    params: HashMap<String, ValueCell>,
    funcs: HashMap<String, RsCelFunction>,
    macros: HashMap<String, RsCelMacro>,
}

impl BindContext {
    /// Create a new bind context contain default functions and macros.
    pub fn new() -> BindContext {
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
    pub fn bind_param(&mut self, name: &str, value: ValueCell) {
        self.params.insert(name.to_owned(), value);
    }

    /// Cheater function to bind the keys of an JSON object with its values
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

    /// Bind a function to the bind context, can be new or overwrite an existing (including default)
    pub fn bind_func(&mut self, name: &str, func: RsCelFunction) {
        self.funcs.insert(name.to_owned(), func);
    }

    /// Bind a macro to the bind context.
    pub fn bind_macro(&mut self, name: &str, macro_: RsCelMacro) {
        self.macros.insert(name.to_owned(), macro_);
    }

    /// Get a param by name.
    pub fn get_param<'a>(&'a self, name: &str) -> Option<&'a ValueCell> {
        Some(self.params.get(name)?)
    }

    /// Get a function by name.
    pub fn get_func(&self, name: &str) -> Option<RsCelFunction> {
        Some(self.funcs.get(name)?.clone())
    }

    /// Get a macro by name.
    pub fn get_macro(&self, name: &str) -> Option<RsCelMacro> {
        Some(self.macros.get(name)?.clone())
    }
}
