use crate::{interp::ByteCode, CelError, CelResult, CelValue};

use super::CelByteCode;

/// A typed argument passed to a macro at runtime.
///
/// The compiler encodes single bare-identifier arguments (e.g. the `x` in
/// `list.map(x, x + 1)`) as [`CelValue::Binder`], which `call_macro` converts
/// into [`MacroArg::Binder`]. All other arguments become [`MacroArg::Expr`]
/// containing the unevaluated bytecode.
///
/// Macros use [`MacroArg::as_binder`] to extract loop-variable names without
/// calling `eval_ident`, and [`MacroArg::eval`] to evaluate any argument
/// uniformly (works for both binders and expressions).
#[derive(Debug)]
pub enum MacroArg<'a> {
    /// A bare identifier that names a loop variable (e.g. `x` in `list.map(x, ...)`).
    Binder(String),
    /// An unevaluated expression encoded as bytecode.
    Expr(&'a CelByteCode),
}

impl<'a> MacroArg<'a> {
    /// Returns the binder name if this is a [`MacroArg::Binder`], or a `CelError` otherwise.
    pub fn as_binder(&self) -> CelResult<&str> {
        match self {
            MacroArg::Binder(name) => Ok(name.as_str()),
            MacroArg::Expr(_) => Err(CelError::argument(
                "expected an identifier binder (e.g. `x`) but got an expression",
            )),
        }
    }

    /// Returns the bytecode if this is a [`MacroArg::Expr`], or a `CelError` otherwise.
    pub fn as_expr(&self) -> CelResult<&'a CelByteCode> {
        match self {
            MacroArg::Expr(bc) => Ok(bc),
            MacroArg::Binder(name) => Err(CelError::argument(&format!(
                "expected an expression but got identifier binder `{}`",
                name
            ))),
        }
    }

    /// Evaluate this argument in the given interpreter context.
    ///
    /// For `Expr`, runs the bytecode directly. For `Binder`, synthesises a
    /// single-instruction lookup so that macros like `has` and `coalesce` can
    /// evaluate any argument uniformly regardless of whether it was encoded as
    /// a binder or an expression.
    pub fn eval(&self, ctx: &crate::interp::Interpreter<'_>) -> CelResult<CelValue> {
        match self {
            MacroArg::Binder(name) => {
                let bc =
                    CelByteCode::from_code_point(ByteCode::Push(CelValue::from_ident(name)));
                ctx.run_raw(&bc, true)
            }
            MacroArg::Expr(bc) => ctx.run_raw(bc, true),
        }
    }
}
