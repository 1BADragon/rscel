use std::collections::HashMap;

#[cfg(feature = "protobuf")]
use protobuf::MessageDyn;
use serde_json::Value;

use crate::{interp::Interpreter, CelError, CelResult, CelValue, MacroArg};

use super::default_macros::{load_compile_macros, load_default_macros};
use super::{default_funcs::load_default_funcs, type_funcs::load_default_types};

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
pub type RsCelFunction = dyn Fn(CelValue, Vec<CelValue>) -> CelValue;

/// Prototype for a macro binding.
///
/// Rust-accelerated macros can be used to modify and evaluate
/// program bytecode within the macro arguments. Instead of being given values, macros are
/// provided the bytecode generated from the argument expression. Like functions, the this
/// argument is the resolved value the macro is being run on (i.e `my_list.map()`) however
/// all arguents passed to the macro are left unresolved bytecode. An additional argument,
/// the Interpreter context, is provided to the macro for bytecode resolution.
pub type RsCelMacro =
    dyn for<'a, 'b> Fn(&'a Interpreter<'a>, CelValue, &'b [MacroArg<'b>]) -> CelValue;

/// Bindings context for a cel evaluation.
///
/// This struct houses all of the bindings (dynamic values) for a cel evaluation.
/// Currently the types of identifiers that can be bound to are variables, functions
/// and macros. This context is separate from the contents of the `CelContext` to allow
/// for multiple runs with different bound values on the same programs without the need
/// to maintain multiple copies of the programs.
///
/// A `BindContext` can optionally reference a parent via [`BindContext::child_scope`].
/// Lookups fall through to the parent when not found locally, allowing macro iteration
/// loops to bind loop variables cheaply without cloning the full function/macro/type tables.
#[derive(Clone)]
pub struct BindContext<'a> {
    params: HashMap<String, CelValue>,
    funcs: HashMap<String, &'a RsCelFunction>,
    macros: HashMap<String, &'a RsCelMacro>,
    types: HashMap<String, CelValue>,
    parent: Option<&'a BindContext<'a>>,
}

impl<'a> BindContext<'a> {
    /// Create a new bind context contain default functions and macros.
    pub fn new() -> BindContext<'a> {
        let mut ctx = BindContext {
            params: HashMap::new(),
            funcs: HashMap::new(),
            macros: HashMap::new(),
            types: HashMap::new(),
            parent: None,
        };

        load_default_macros(&mut ctx);
        load_default_funcs(&mut ctx);
        load_default_types(&mut ctx);
        ctx
    }

    pub fn for_compile() -> BindContext<'a> {
        let mut ctx = BindContext {
            params: HashMap::new(),
            funcs: HashMap::new(),
            macros: HashMap::new(),
            types: HashMap::new(),
            parent: None,
        };

        load_compile_macros(&mut ctx);
        load_default_funcs(&mut ctx);
        load_default_types(&mut ctx);
        ctx
    }

    /// Create a child scope that borrows this context's functions, macros, and types
    /// through a parent pointer rather than cloning them. The child starts with empty
    /// local param/func/macro/type tables; lookups fall through to the parent.
    ///
    /// Use this inside macro iteration loops where only the loop variable changes
    /// between iterations, to avoid cloning the full context on each element.
    pub fn child_scope(&'a self) -> BindContext<'a> {
        BindContext {
            params: HashMap::new(),
            funcs: HashMap::new(),
            macros: HashMap::new(),
            types: HashMap::new(),
            parent: Some(self),
        }
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

    #[cfg(feature = "protobuf")]
    pub fn bind_param_proto_msg(&mut self, name: &str, msg: Box<dyn MessageDyn>) {
        self.params
            .insert(name.to_owned(), CelValue::from_proto_msg(msg));
    }

    /// Bind a function to the bind context, can be new or overwrite an existing (including default)
    pub fn bind_func(&mut self, name: &str, func: &'a RsCelFunction) {
        self.funcs.insert(name.to_owned(), func);
    }

    /// Bind a macro to the bind context.
    pub fn bind_macro(&mut self, name: &str, macro_: &'a RsCelMacro) {
        self.macros.insert(name.to_owned(), macro_);
    }

    /// Get a param by name. Checks local params first, then walks up the parent chain.
    pub fn get_param<'l>(&'l self, name: &str) -> Option<&'l CelValue> {
        self.params
            .get(name)
            .or_else(|| self.parent?.get_param(name))
    }

    /// Get a function by name. Checks local funcs first, then walks up the parent chain.
    pub fn get_func(&self, name: &str) -> Option<&'a RsCelFunction> {
        self.funcs
            .get(name)
            .copied()
            .or_else(|| self.parent?.get_func(name))
    }

    /// Get a macro by name. Checks local macros first, then walks up the parent chain.
    pub fn get_macro(&self, name: &str) -> Option<&'a RsCelMacro> {
        self.macros
            .get(name)
            .copied()
            .or_else(|| self.parent?.get_macro(name))
    }

    /// Returns true if the name is bound anywhere in the scope chain.
    pub fn is_bound(&self, name: &str) -> bool {
        self.params.contains_key(name)
            || self.funcs.contains_key(name)
            || self.macros.contains_key(name)
            || self.parent.map_or(false, |p| p.is_bound(name))
    }

    pub(crate) fn add_type(&mut self, name: &str, r#type: CelValue) {
        self.types.insert(name.to_string(), r#type);
    }

    /// Get a type by name. Checks local types first, then walks up the parent chain.
    pub(crate) fn get_type(&self, name: &str) -> Option<&CelValue> {
        self.types
            .get(name)
            .or_else(|| self.parent?.get_type(name))
    }
}

#[cfg(test)]
mod test {
    use super::BindContext;

    #[test]
    fn basic() {
        let mut b = BindContext::new();

        b.bind_param("foo", 4.into());

        assert!(b.is_bound("foo"))
    }

    #[test]
    fn child_scope_inherits_parent_funcs_and_macros() {
        let parent = BindContext::new();
        let mut child = parent.child_scope();

        // Child should see parent's default functions
        assert!(child.get_func("size").is_some());
        assert!(child.get_macro("filter").is_some());

        // Local param on child does not affect parent lookup
        child.bind_param("x", 42.into());
        assert!(child.get_param("x").is_some());
        assert!(parent.get_param("x").is_none());
    }

    #[test]
    fn child_scope_param_shadows_parent() {
        let mut parent = BindContext::new();
        parent.bind_param("x", 1.into());

        let mut child = parent.child_scope();
        child.bind_param("x", 2.into());

        assert_eq!(*child.get_param("x").unwrap(), 2.into());
        assert_eq!(*parent.get_param("x").unwrap(), 1.into());
    }
}
