//! SQL compilation utilities.
//!
//! This module provides a minimal interface for converting CEL
//! expressions into SQL fragments consisting of SQL text and a list
//! of bound parameters.
//!
//! # JSON assumptions
//!
//! The generated SQL targets PostgreSQL `jsonb` values. Field and array
//! accesses are translated into `jsonb` operator chains (`->` and `->>`),
//! with the final segment in a chain extracted using `->>` to produce a
//! text value. Array indices use `jsonb` subscripting. The JSON documents
//! are assumed to contain the referenced structure at runtime.

use regex::Regex;
use std::fmt;

mod functions;

#[cfg(test)]
mod tests;

use crate::compiler::tokens::FStringSegment;
use crate::{
    AddOp, Addition, AstNode, CelValue, ConditionalAnd, ConditionalOr, Expr, ExprList, Ident,
    LiteralsAndKeywords, MatchAnyPattern, MatchCase, MatchCmpOp, MatchPattern, MatchTypePattern,
    Member, MemberPrime, MultOp, Multiplication, NegList, NoAst, NotList, ObjInit, ObjInits,
    Primary, Relation, Relop, Unary,
};

/// SQL text and its associated parameter values.
#[derive(Debug, Clone, PartialEq)]
pub struct SqlFragment {
    /// The SQL text with placeholders for parameters.
    pub sql: String,
    /// Values bound to the placeholders within `sql`.
    pub params: Vec<CelValue>,
}

/// Result type used by SQL compilation routines.
pub type SqlResult<T> = Result<T, SqlError>;

/// Errors that can occur during SQL compilation.
#[derive(Debug, Clone, PartialEq)]
pub enum SqlError {
    /// Feature is not supported by the SQL compiler.
    UnsupportedFeature(String),
    /// Type of an expression is invalid for the requested operation.
    InvalidType(String),
    /// An unexpected internal failure occurred.
    Internal(String),
}

impl fmt::Display for SqlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SqlError::UnsupportedFeature(s) => write!(f, "unsupported feature: {}", s),
            SqlError::InvalidType(s) => write!(f, "invalid type: {}", s),
            SqlError::Internal(s) => write!(f, "internal error: {}", s),
        }
    }
}

impl std::error::Error for SqlError {}

/// Compiler that converts CEL AST nodes into SQL fragments.
pub struct SqlCompiler;

impl SqlCompiler {
    /// Compile a CEL expression AST into a [`SqlFragment`].
    pub fn compile(ast: &AstNode<Expr>) -> SqlResult<SqlFragment> {
        let compiler = SqlCompiler;
        ast.to_sql(&compiler)
    }
}

/// Trait for converting a type into a [`SqlFragment`].
pub trait ToSql {
    /// Convert `self` into a [`SqlFragment`] using the provided compiler.
    fn to_sql(&self, compiler: &SqlCompiler) -> SqlResult<SqlFragment>;
}

/// Shift placeholder numbers in `sql` by `offset`.
pub(super) fn shift_placeholders(sql: &str, offset: usize) -> String {
    if offset == 0 {
        return sql.to_string();
    }
    let re = Regex::new(r"\$(\d+)").unwrap();
    re.replace_all(sql, |caps: &regex::Captures| {
        let n: usize = caps[1].parse().unwrap();
        format!("${}", n + offset)
    })
    .into_owned()
}

fn merge_fragments(lhs: SqlFragment, rhs: SqlFragment, join: &str) -> SqlFragment {
    let offset = lhs.params.len();
    let rhs_sql = shift_placeholders(&rhs.sql, offset);
    let mut params = lhs.params;
    params.extend(rhs.params);
    SqlFragment {
        sql: format!("({} {} {})", lhs.sql, join, rhs_sql),
        params,
    }
}

pub(super) fn join_fragments(args: Vec<SqlFragment>) -> (Vec<String>, Vec<CelValue>) {
    let mut parts = Vec::new();
    let mut params = Vec::new();
    for frag in args {
        let offset = params.len();
        let sql = shift_placeholders(&frag.sql, offset);
        params.extend(frag.params);
        parts.push(sql);
    }
    (parts, params)
}

pub(super) fn default_call(name: &str, args: Vec<SqlFragment>) -> SqlFragment {
    let (parts, params) = join_fragments(args);
    SqlFragment {
        sql: format!("{}({})", name, parts.join(", ")),
        params,
    }
}

impl SqlCompiler {
    fn call_function(&self, name: &str, args: Vec<SqlFragment>) -> SqlResult<SqlFragment> {
        if let Some(func) = functions::FUNCTIONS.get(name) {
            func(args)
        } else {
            Err(SqlError::UnsupportedFeature(format!(
                "function '{}' is not supported",
                name
            )))
        }
    }
}

impl<T: ToSql> ToSql for AstNode<T> {
    fn to_sql(&self, compiler: &SqlCompiler) -> SqlResult<SqlFragment> {
        self.node().to_sql(compiler)
    }
}

impl ToSql for Expr {
    fn to_sql(&self, compiler: &SqlCompiler) -> SqlResult<SqlFragment> {
        match self {
            Expr::Ternary {
                condition,
                true_clause,
                false_clause,
            } => {
                let cond = condition.to_sql(compiler)?;
                let true_frag = true_clause.to_sql(compiler)?;
                let offset_true = cond.params.len();
                let true_sql = shift_placeholders(&true_frag.sql, offset_true);
                let mut params = cond.params.clone();
                params.extend(true_frag.params);

                let false_frag = false_clause.to_sql(compiler)?;
                let offset_false = params.len();
                let false_sql = shift_placeholders(&false_frag.sql, offset_false);
                params.extend(false_frag.params);

                Ok(SqlFragment {
                    sql: format!(
                        "(CASE WHEN {} THEN {} ELSE {} END)",
                        cond.sql, true_sql, false_sql
                    ),
                    params,
                })
            }
            Expr::Match { condition, cases } => {
                let cond = condition.to_sql(compiler)?;
                let mut sql = String::from("CASE");
                let mut params = cond.params.clone();

                for case in cases {
                    let case_node = case.node();
                    let (pattern_sql, mut pat_params) = case_node.pattern.node().to_sql_with_cond(
                        &cond.sql,
                        compiler,
                        params.len(),
                    )?;
                    params.extend(pat_params.drain(..));

                    let expr_frag = case_node.expr.to_sql(compiler)?;
                    let expr_sql = shift_placeholders(&expr_frag.sql, params.len());
                    params.extend(expr_frag.params);

                    sql.push_str(&format!(" WHEN {} THEN {}", pattern_sql, expr_sql));
                }

                sql.push_str(" END");
                Ok(SqlFragment { sql, params })
            }
            Expr::Unary(or) => or.to_sql(compiler),
        }
    }
}

impl MatchPattern {
    fn to_sql_with_cond(
        &self,
        cond_sql: &str,
        compiler: &SqlCompiler,
        offset: usize,
    ) -> SqlResult<(String, Vec<CelValue>)> {
        match self {
            MatchPattern::Cmp { op, or } => {
                let or_frag = or.to_sql(compiler)?;
                let or_sql = shift_placeholders(&or_frag.sql, offset);
                let op_sql = op.to_sql(compiler)?.sql;
                Ok((
                    format!(
                        "{} {} {}",
                        shift_placeholders(cond_sql, offset),
                        op_sql,
                        or_sql
                    ),
                    or_frag.params,
                ))
            }
            MatchPattern::Type(t) => {
                let type_str = t.node().to_pg_type();
                Ok((
                    format!(
                        "pg_typeof({}) = '{}'",
                        shift_placeholders(cond_sql, offset),
                        type_str
                    ),
                    Vec::new(),
                ))
            }
            MatchPattern::Any(_) => Ok(("TRUE".to_string(), Vec::new())),
        }
    }
}

impl ToSql for MatchPattern {
    fn to_sql(&self, compiler: &SqlCompiler) -> SqlResult<SqlFragment> {
        let (sql, params) = self.to_sql_with_cond("$1", compiler, 0)?;
        Ok(SqlFragment { sql, params })
    }
}

impl ToSql for MatchCase {
    fn to_sql(&self, compiler: &SqlCompiler) -> SqlResult<SqlFragment> {
        // Placeholder compilation; actual combination handled in Expr::Match
        let (pattern_sql, params) = self.pattern.node().to_sql_with_cond("$1", compiler, 0)?;
        let expr_frag = self.expr.to_sql(compiler)?;
        Ok(SqlFragment {
            sql: format!(
                "WHEN {} THEN {}",
                pattern_sql,
                shift_placeholders(&expr_frag.sql, params.len())
            ),
            params: [params, expr_frag.params].concat(),
        })
    }
}

impl ToSql for MatchCmpOp {
    fn to_sql(&self, _compiler: &SqlCompiler) -> SqlResult<SqlFragment> {
        let sql = match self {
            MatchCmpOp::Eq => "=",
            MatchCmpOp::Neq => "!=",
            MatchCmpOp::Gt => ">",
            MatchCmpOp::Ge => ">=",
            MatchCmpOp::Lt => "<",
            MatchCmpOp::Le => "<=",
        };
        Ok(SqlFragment {
            sql: sql.to_string(),
            params: Vec::new(),
        })
    }
}

impl MatchTypePattern {
    fn to_pg_type(&self) -> &'static str {
        match self {
            MatchTypePattern::Int => "int8",
            MatchTypePattern::Uint => "int8",
            MatchTypePattern::Float => "float8",
            MatchTypePattern::String => "text",
            MatchTypePattern::Bool => "bool",
            MatchTypePattern::Bytes => "bytea",
            MatchTypePattern::List => "jsonb",
            MatchTypePattern::Object => "jsonb",
            MatchTypePattern::Null => "null",
            MatchTypePattern::Timestamp => "timestamptz",
            MatchTypePattern::Duration => "interval",
        }
    }
}

impl ToSql for MatchTypePattern {
    fn to_sql(&self, _compiler: &SqlCompiler) -> SqlResult<SqlFragment> {
        Ok(SqlFragment {
            sql: format!("'{}'", self.to_pg_type()),
            params: Vec::new(),
        })
    }
}

impl ToSql for MatchAnyPattern {
    fn to_sql(&self, _compiler: &SqlCompiler) -> SqlResult<SqlFragment> {
        Ok(SqlFragment {
            sql: "TRUE".to_string(),
            params: Vec::new(),
        })
    }
}

impl ToSql for ConditionalOr {
    fn to_sql(&self, compiler: &SqlCompiler) -> SqlResult<SqlFragment> {
        match self {
            ConditionalOr::Binary { lhs, rhs } => {
                let lhs_frag = lhs.to_sql(compiler)?;
                let rhs_frag = rhs.to_sql(compiler)?;
                Ok(merge_fragments(lhs_frag, rhs_frag, "OR"))
            }
            ConditionalOr::Unary(inner) => inner.to_sql(compiler),
        }
    }
}

impl ToSql for ConditionalAnd {
    fn to_sql(&self, compiler: &SqlCompiler) -> SqlResult<SqlFragment> {
        match self {
            ConditionalAnd::Binary { lhs, rhs } => {
                let lhs_frag = lhs.to_sql(compiler)?;
                let rhs_frag = rhs.to_sql(compiler)?;
                Ok(merge_fragments(lhs_frag, rhs_frag, "AND"))
            }
            ConditionalAnd::Unary(inner) => inner.to_sql(compiler),
        }
    }
}

impl ToSql for Relation {
    fn to_sql(&self, compiler: &SqlCompiler) -> SqlResult<SqlFragment> {
        match self {
            Relation::Binary { lhs, op, rhs } => {
                let lhs_frag = lhs.to_sql(compiler)?;
                let rhs_frag = rhs.to_sql(compiler)?;
                let offset = lhs_frag.params.len();
                let rhs_sql = shift_placeholders(&rhs_frag.sql, offset);
                let mut params = lhs_frag.params;
                params.extend(rhs_frag.params);
                Ok(SqlFragment {
                    sql: format!(
                        "({} {} {})",
                        lhs_frag.sql,
                        op.to_sql(compiler)?.sql,
                        rhs_sql
                    ),
                    params,
                })
            }
            Relation::Unary(inner) => inner.to_sql(compiler),
        }
    }
}

impl ToSql for Relop {
    fn to_sql(&self, _compiler: &SqlCompiler) -> SqlResult<SqlFragment> {
        let sql = match self {
            Relop::Le => "<=",
            Relop::Lt => "<",
            Relop::Ge => ">=",
            Relop::Gt => ">",
            Relop::Eq => "=",
            Relop::Ne => "!=",
            Relop::In => "IN",
        };
        Ok(SqlFragment {
            sql: sql.to_string(),
            params: Vec::new(),
        })
    }
}

impl ToSql for Addition {
    fn to_sql(&self, compiler: &SqlCompiler) -> SqlResult<SqlFragment> {
        match self {
            Addition::Binary { lhs, op, rhs } => {
                let lhs_frag = lhs.to_sql(compiler)?;
                let rhs_frag = rhs.to_sql(compiler)?;
                Ok(merge_fragments(
                    lhs_frag,
                    rhs_frag,
                    &op.to_sql(compiler)?.sql,
                ))
            }
            Addition::Unary(inner) => inner.to_sql(compiler),
        }
    }
}

impl ToSql for AddOp {
    fn to_sql(&self, _compiler: &SqlCompiler) -> SqlResult<SqlFragment> {
        let sql = match self {
            AddOp::Add => "+",
            AddOp::Sub => "-",
        };
        Ok(SqlFragment {
            sql: sql.to_string(),
            params: Vec::new(),
        })
    }
}

impl ToSql for Multiplication {
    fn to_sql(&self, compiler: &SqlCompiler) -> SqlResult<SqlFragment> {
        match self {
            Multiplication::Binary { lhs, op, rhs } => {
                let lhs_frag = lhs.to_sql(compiler)?;
                let rhs_frag = rhs.to_sql(compiler)?;
                Ok(merge_fragments(
                    lhs_frag,
                    rhs_frag,
                    &op.to_sql(compiler)?.sql,
                ))
            }
            Multiplication::Unary(inner) => inner.to_sql(compiler),
        }
    }
}

impl ToSql for MultOp {
    fn to_sql(&self, _compiler: &SqlCompiler) -> SqlResult<SqlFragment> {
        let sql = match self {
            MultOp::Mult => "*",
            MultOp::Div => "/",
            MultOp::Mod => "%",
        };
        Ok(SqlFragment {
            sql: sql.to_string(),
            params: Vec::new(),
        })
    }
}

impl NotList {
    fn count(&self) -> usize {
        match self {
            NotList::List { tail } => 1 + tail.node().count(),
            NotList::EmptyList => 0,
        }
    }
}

impl ToSql for NotList {
    fn to_sql(&self, _compiler: &SqlCompiler) -> SqlResult<SqlFragment> {
        Ok(SqlFragment {
            sql: String::new(),
            params: Vec::new(),
        })
    }
}

impl NegList {
    fn count(&self) -> usize {
        match self {
            NegList::List { tail } => 1 + tail.node().count(),
            NegList::EmptyList => 0,
        }
    }
}

impl ToSql for NegList {
    fn to_sql(&self, _compiler: &SqlCompiler) -> SqlResult<SqlFragment> {
        Ok(SqlFragment {
            sql: String::new(),
            params: Vec::new(),
        })
    }
}

impl ToSql for Unary {
    fn to_sql(&self, compiler: &SqlCompiler) -> SqlResult<SqlFragment> {
        match self {
            Unary::Member(m) => m.to_sql(compiler),
            Unary::NotMember { nots, member } => {
                let mut frag = member.to_sql(compiler)?;
                for _ in 0..nots.node().count() {
                    frag.sql = format!("(NOT {})", frag.sql);
                }
                Ok(frag)
            }
            Unary::NegMember { negs, member } => {
                let mut frag = member.to_sql(compiler)?;
                for _ in 0..negs.node().count() {
                    frag.sql = format!("(-{})", frag.sql);
                }
                Ok(frag)
            }
        }
    }
}

impl ToSql for Member {
    fn to_sql(&self, compiler: &SqlCompiler) -> SqlResult<SqlFragment> {
        if matches!(self.primary.node(), Primary::Literal(_)) && !self.member.is_empty() {
            return Err(SqlError::InvalidType(
                "cannot access property on literal".to_string(),
            ));
        }

        let mut frag = self.primary.to_sql(compiler)?;
        let mut chain_sql = String::new();
        let mut chain_params: Vec<CelValue> = Vec::new();

        let mut iter = self.member.iter().peekable();
        while let Some(m) = iter.next() {
            match m.node() {
                MemberPrime::MemberAccess { ident } => {
                    if let Some(next) = iter.peek() {
                        if let MemberPrime::Call { call } = next.node() {
                            // method call
                            if !chain_sql.is_empty() {
                                if let Some(pos) = chain_sql.rfind("->") {
                                    chain_sql.replace_range(pos..pos + 2, "->>");
                                }
                                frag.sql.push_str(&chain_sql);
                                frag.params.extend(chain_params.drain(..));
                                chain_sql.clear();
                            }
                            iter.next();
                            let mut args: Vec<SqlFragment> = vec![frag];
                            for expr in call.node().exprs.iter().rev() {
                                args.push(expr.to_sql(compiler)?);
                            }
                            frag = compiler.call_function(&ident.node().0, args)?;
                        } else {
                            let current = m.to_sql(compiler)?;
                            let offset = frag.params.len() + chain_params.len();
                            let sql = shift_placeholders(&current.sql, offset);
                            chain_sql.push_str(&sql);
                            chain_params.extend(current.params);
                        }
                    } else {
                        let current = m.to_sql(compiler)?;
                        let offset = frag.params.len() + chain_params.len();
                        let sql = shift_placeholders(&current.sql, offset);
                        chain_sql.push_str(&sql);
                        chain_params.extend(current.params);
                    }
                }
                MemberPrime::Call { call } => {
                    if !chain_sql.is_empty() {
                        if let Some(pos) = chain_sql.rfind("->") {
                            chain_sql.replace_range(pos..pos + 2, "->>");
                        }
                        frag.sql.push_str(&chain_sql);
                        frag.params.extend(chain_params.drain(..));
                        chain_sql.clear();
                    }
                    let mut args: Vec<SqlFragment> = Vec::new();
                    for expr in call.node().exprs.iter().rev() {
                        args.push(expr.to_sql(compiler)?);
                    }
                    frag = compiler.call_function(&frag.sql, args)?;
                }
                _ => {
                    let current = m.to_sql(compiler)?;
                    let offset = frag.params.len() + chain_params.len();
                    let sql = shift_placeholders(&current.sql, offset);
                    chain_sql.push_str(&sql);
                    chain_params.extend(current.params);
                }
            }
        }

        if !chain_sql.is_empty() {
            if let Some(pos) = chain_sql.rfind("->") {
                chain_sql.replace_range(pos..pos + 2, "->>");
            }
            frag.sql.push_str(&chain_sql);
            frag.params.extend(chain_params);
        }

        Ok(frag)
    }
}

impl ToSql for MemberPrime {
    fn to_sql(&self, compiler: &SqlCompiler) -> SqlResult<SqlFragment> {
        match self {
            MemberPrime::MemberAccess { ident } => Ok(SqlFragment {
                sql: format!(" -> '{}'", ident.node().0),
                params: Vec::new(),
            }),
            MemberPrime::Call { call } => {
                let frag = call.to_sql(compiler)?;
                Ok(SqlFragment {
                    sql: frag.sql,
                    params: frag.params,
                })
            }
            MemberPrime::ArrayAccess { access } => {
                let frag = access.to_sql(compiler)?;
                Ok(SqlFragment {
                    sql: format!(" -> ({})", frag.sql),
                    params: frag.params,
                })
            }
            MemberPrime::Empty => Ok(SqlFragment {
                sql: String::new(),
                params: Vec::new(),
            }),
        }
    }
}

impl ToSql for Ident {
    fn to_sql(&self, _compiler: &SqlCompiler) -> SqlResult<SqlFragment> {
        Ok(SqlFragment {
            sql: self.0.clone(),
            params: Vec::new(),
        })
    }
}

impl ToSql for Primary {
    fn to_sql(&self, compiler: &SqlCompiler) -> SqlResult<SqlFragment> {
        match self {
            Primary::Type => Ok(SqlFragment {
                sql: "type".to_string(),
                params: Vec::new(),
            }),
            Primary::Ident(id) => id.to_sql(compiler),
            Primary::Parens(expr) => {
                let frag = expr.to_sql(compiler)?;
                Ok(SqlFragment {
                    sql: format!("({})", frag.sql),
                    params: frag.params,
                })
            }
            Primary::ListConstruction(list) => {
                let frag = list.to_sql(compiler)?;
                Ok(SqlFragment {
                    sql: format!("ARRAY[{}]", frag.sql),
                    params: frag.params,
                })
            }
            Primary::ObjectInit(obj) => obj.to_sql(compiler),
            Primary::Literal(lit) => lit.to_sql(compiler),
        }
    }
}

impl ToSql for ExprList {
    fn to_sql(&self, compiler: &SqlCompiler) -> SqlResult<SqlFragment> {
        let mut sql_parts = Vec::new();
        let mut params = Vec::new();
        for expr in &self.exprs {
            let frag = expr.to_sql(compiler)?;
            let sql = shift_placeholders(&frag.sql, params.len());
            params.extend(frag.params);
            sql_parts.push(sql);
        }
        Ok(SqlFragment {
            sql: sql_parts.join(", "),
            params,
        })
    }
}

impl ToSql for ObjInit {
    fn to_sql(&self, compiler: &SqlCompiler) -> SqlResult<SqlFragment> {
        let key_frag = self.key.to_sql(compiler)?;
        let val_frag = self.value.to_sql(compiler)?;
        let offset = key_frag.params.len();
        Ok(SqlFragment {
            sql: format!(
                "{}, {}",
                key_frag.sql,
                shift_placeholders(&val_frag.sql, offset)
            ),
            params: [key_frag.params, val_frag.params].concat(),
        })
    }
}

impl ToSql for ObjInits {
    fn to_sql(&self, compiler: &SqlCompiler) -> SqlResult<SqlFragment> {
        let mut sql = String::new();
        let mut params = Vec::new();
        for (i, init) in self.inits.iter().enumerate() {
            let frag = init.to_sql(compiler)?;
            let sql_shift = shift_placeholders(&frag.sql, params.len());
            params.extend(frag.params);
            if i > 0 {
                sql.push_str(", ");
            }
            sql.push_str(&sql_shift);
        }
        Ok(SqlFragment {
            sql: format!("jsonb_build_object({})", sql),
            params,
        })
    }
}

impl ToSql for NoAst {
    fn to_sql(&self, _compiler: &SqlCompiler) -> SqlResult<SqlFragment> {
        Ok(SqlFragment {
            sql: String::new(),
            params: Vec::new(),
        })
    }
}

impl ToSql for LiteralsAndKeywords {
    fn to_sql(&self, _compiler: &SqlCompiler) -> SqlResult<SqlFragment> {
        let frag = match self {
            LiteralsAndKeywords::NullLit => SqlFragment {
                sql: "$1".to_string(),
                params: vec![CelValue::from_null()],
            },
            LiteralsAndKeywords::IntegerLit(i) => SqlFragment {
                sql: "$1".to_string(),
                params: vec![CelValue::from_int(*i)],
            },
            LiteralsAndKeywords::UnsignedLit(u) => SqlFragment {
                sql: "$1".to_string(),
                params: vec![CelValue::from_uint(*u)],
            },
            LiteralsAndKeywords::FloatingLit(f) => SqlFragment {
                sql: "$1".to_string(),
                params: vec![CelValue::from_float(*f)],
            },
            LiteralsAndKeywords::FStringList(segs) => {
                let mut s = String::new();
                for seg in segs {
                    match seg {
                        FStringSegment::Lit(l) => s.push_str(l),
                        FStringSegment::Expr(e) => s.push_str(e),
                    }
                }
                SqlFragment {
                    sql: "$1".to_string(),
                    params: vec![CelValue::from_string(s)],
                }
            }
            LiteralsAndKeywords::StringLit(s) => SqlFragment {
                sql: "$1".to_string(),
                params: vec![CelValue::from_string(s.clone())],
            },
            LiteralsAndKeywords::ByteStringLit(b) => SqlFragment {
                sql: "$1".to_string(),
                params: vec![CelValue::from_bytes(b.clone())],
            },
            LiteralsAndKeywords::BooleanLit(b) => SqlFragment {
                sql: "$1".to_string(),
                params: vec![CelValue::from_bool(*b)],
            },
            _ => SqlFragment {
                sql: String::new(),
                params: Vec::new(),
            },
        };
        Ok(frag)
    }
}
