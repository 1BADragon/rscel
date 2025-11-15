use crate::interp::Interpreter;
use crate::{BindContext, CelContext};

pub(super) fn setup_context<'a>(ctx: &'a Interpreter<'a>) -> (CelContext, BindContext<'a>) {
    (
        ctx.cel_copy().unwrap_or_else(CelContext::new),
        ctx.bindings_copy().unwrap_or_else(BindContext::new),
    )
}
