use crate::interp::Interpreter;
use crate::{BindContext, CelContext};

/// Returns the CelContext and a child BindContext scoped to the given interpreter.
///
/// The child borrows the interpreter's binding context through a parent pointer rather
/// than cloning it, so functions, macros, and types are shared by reference. Only the
/// loop variable (added via `bind_param` each iteration) lives in the child's local
/// params table.
pub(super) fn child_scope<'a>(ctx: &'a Interpreter<'a>) -> (CelContext, BindContext<'a>) {
    let cel = ctx.cel_copy().unwrap_or_else(CelContext::new);
    let child = match ctx.bindings_ref() {
        Some(b) => b.child_scope(),
        None => BindContext::new(),
    };
    (cel, child)
}
