//! SQL compilation utilities.
//!
//! This module provides a minimal interface for converting CEL
//! expressions into SQL fragments consisting of SQL text and a list
//! of bound parameters.

use crate::{AstNode, CelValue, Expr};

/// SQL text and its associated parameter values.
#[derive(Debug, Clone, PartialEq)]
pub struct SqlFragment {
    /// The SQL text with placeholders for parameters.
    pub sql: String,
    /// Values bound to the placeholders within `sql`.
    pub params: Vec<CelValue>,
}

/// Compiler that converts CEL AST nodes into SQL fragments.
pub struct SqlCompiler;

impl SqlCompiler {
    /// Compile a CEL expression AST into a [`SqlFragment`].
    pub fn compile(ast: &AstNode<Expr>) -> SqlFragment {
        let compiler = SqlCompiler;
        ast.to_sql(&compiler)
    }
}

/// Trait for converting a type into a [`SqlFragment`].
pub trait ToSql {
    /// Convert `self` into a [`SqlFragment`] using the provided compiler.
    fn to_sql(&self, compiler: &SqlCompiler) -> SqlFragment;
}

impl ToSql for AstNode<Expr> {
    fn to_sql(&self, _compiler: &SqlCompiler) -> SqlFragment {
        SqlFragment {
            sql: String::new(),
            params: Vec::new(),
        }
    }
}
