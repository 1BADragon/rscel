use crate::{
    context::CelContext,
    parser::{
        AddOp, Addition, ConditionalAnd, ConditionalOr, Expr, ExprList, Literal, Member,
        MemberPrime, MultOp, Multiplication, NegList, NotList, Primary, Relation, Relop, Unary,
    },
    value_cell::{ValueCell, ValueCellError, ValueCellResult},
};
use std::{
    ops::{Neg, Not},
    str::FromStr,
};

mod program_details;
mod program_error;

use parsel::ast::Lit;
// Re-export
pub use program_error::ProgramError;

use program_details::ProgramDetails;

pub type ProgramResult<T> = Result<T, ProgramError>;

#[derive(Debug)]
pub struct Program {
    source: String,
    details: program_details::ProgramDetails,

    ast: Expr,
}

impl Program {
    pub fn from_source(source: &str) -> ProgramResult<Program> {
        let ast: Expr = match parsel::parse_str(source) {
            Ok(expr) => expr,
            Err(err) => {
                let span = err.span();
                return Err(ProgramError::new(&format!(
                    "Error on {}:{} ending at {}:{}",
                    span.start().line,
                    span.start().column,
                    span.end().line,
                    span.end().column
                )));
            }
        };

        Ok(Program {
            source: String::from_str(source).unwrap(),
            details: ProgramDetails::from_ast(&ast),
            ast,
        })
    }

    pub fn params<'a>(&'a self) -> Vec<&'a str> {
        self.details.params()
    }

    pub fn source<'a>(&'a self) -> &'a str {
        &self.source
    }

    pub fn eval(&self, ctx: &CelContext) -> ProgramResult<ValueCell> {
        match self.eval_expr(&self.ast, ctx) {
            Ok(val) => Ok(val),
            Err(err) => Err(ProgramError::new(err.msg())),
        }
    }

    fn eval_expr(&self, ast: &Expr, ctx: &CelContext) -> ValueCellResult<ValueCell> {
        match ast {
            Expr::CondOr(child) => return self.eval_or(child, ctx),
            Expr::Ternary {
                cond_or,
                question: _,
                true_clase,
                colon: _,
                expr,
            } => {
                let res = self.eval_or(cond_or, ctx)?;

                if let ValueCell::Bool(val) = res {
                    if val {
                        return self.eval_or(true_clase, ctx);
                    } else {
                        return self.eval_expr(expr, ctx);
                    }
                }

                return Err(ValueCellError::with_msg(&format!(
                    "Invalid op '?' on type {:?}",
                    res.as_type()
                )));
            }
        }
    }

    fn eval_or(&self, ast: &ConditionalOr, ctx: &CelContext) -> ValueCellResult<ValueCell> {
        match ast {
            ConditionalOr::Binary { lhs, op: _, rhs } => {
                let left = self.eval_or(lhs, ctx)?;

                let lhs = if let ValueCell::Bool(res) = left {
                    res
                } else {
                    return Err(ValueCellError::with_msg(&format!(
                        "Boolean eval invalid for {:?}",
                        left.as_type()
                    )));
                };

                if lhs {
                    return Ok(ValueCell::from_bool(true));
                }

                let right = self.eval_and(rhs, ctx)?;
                let rhs = if let ValueCell::Bool(res) = right {
                    res
                } else {
                    return Err(ValueCellError::with_msg(&format!(
                        "Boolean eval invalid for {:?}",
                        left.as_type()
                    )));
                };

                return Ok(ValueCell::from_bool(rhs));
            }
            ConditionalOr::Rhs(child) => self.eval_and(child, ctx),
        }
    }

    fn eval_and(&self, ast: &ConditionalAnd, ctx: &CelContext) -> ValueCellResult<ValueCell> {
        match ast {
            ConditionalAnd::Binary { lhs, op: _, rhs } => {
                let left = self.eval_and(lhs, ctx)?;
                let lhs = if let ValueCell::Bool(res) = left {
                    res
                } else {
                    return Err(ValueCellError::with_msg(&format!(
                        "Boolean eval invalid for {:?}",
                        left.as_type()
                    )));
                };

                if !lhs {
                    return Ok(ValueCell::from_bool(false));
                }

                let right = self.eval_relation(rhs, ctx)?;
                let rhs = if let ValueCell::Bool(res) = right {
                    res
                } else {
                    return Err(ValueCellError::with_msg(&format!(
                        "Boolean eval invalid for {:?}",
                        left.as_type()
                    )));
                };
                return Ok(ValueCell::from_bool(rhs));
            }
            ConditionalAnd::Rhs(child) => self.eval_relation(child, ctx),
        }
    }

    fn eval_relation(&self, ast: &Relation, ctx: &CelContext) -> ValueCellResult<ValueCell> {
        match ast {
            Relation::Binary { lhs, op, rhs } => {
                let lhs = self.eval_relation(lhs, ctx)?;
                let rhs = self.eval_addition(rhs, ctx)?;

                match op {
                    Relop::Lt(_) => return lhs.lt(&rhs),
                    Relop::Le(_) => return lhs.le(&rhs),
                    Relop::Ge(_) => return lhs.ge(&rhs),
                    Relop::Gt(_) => return lhs.gt(&rhs),
                    Relop::Eq(_) => return lhs.eq(&rhs),
                    Relop::Ne(_) => return lhs.neq(&rhs),
                    Relop::In(_) => {
                        return Err(ValueCellError::with_msg("Op 'in' not implemented"));
                    }
                }
            }
            Relation::Rhs(child) => self.eval_addition(child, ctx),
        }
    }

    fn eval_addition(&self, ast: &Addition, ctx: &CelContext) -> ValueCellResult<ValueCell> {
        match ast {
            Addition::Binary { lhs, op, rhs } => {
                let lhs = self.eval_addition(lhs, ctx)?;
                let rhs = self.eval_multiplication(rhs, ctx)?;

                match op {
                    AddOp::Add(_) => lhs + rhs,
                    AddOp::Sub(_) => lhs - rhs,
                }
            }
            Addition::Rhs(child) => self.eval_multiplication(child, ctx),
        }
    }

    fn eval_multiplication(
        &self,
        ast: &Multiplication,
        ctx: &CelContext,
    ) -> ValueCellResult<ValueCell> {
        match ast {
            Multiplication::Binary { lhs, op, rhs } => {
                let lhs = self.eval_multiplication(lhs, ctx)?;
                let rhs = self.eval_unary(rhs, ctx)?;

                match op {
                    MultOp::Mult(_) => lhs * rhs,
                    MultOp::Div(_) => lhs / rhs,
                    MultOp::Mod(_) => {
                        return Err(ValueCellError::with_msg("Op 'mod' not implemented"));
                    }
                }
            }
            Multiplication::Rhs(child) => self.eval_unary(child, ctx),
        }
    }

    fn eval_unary(&self, ast: &Unary, ctx: &CelContext) -> ValueCellResult<ValueCell> {
        match ast {
            Unary::Member(child) => self.eval_member(child, ctx),
            Unary::NotMember { nots, member } => {
                let m = self.eval_member(member, ctx)?;

                self.eval_not(nots, m, ctx)
            }
            Unary::NegMember { negs, member } => {
                let m = self.eval_member(member, ctx)?;

                self.eval_neg(negs, m, ctx)
            }
        }
    }

    fn eval_not(
        &self,
        ast: &NotList,
        val: ValueCell,
        ctx: &CelContext,
    ) -> ValueCellResult<ValueCell> {
        match ast {
            NotList::List { not: _, tail } => {
                let new_value = val.not()?;

                self.eval_not(tail, new_value, ctx)
            }
            NotList::EmptyList(_) => Ok(val),
        }
    }

    fn eval_neg(
        &self,
        ast: &NegList,
        val: ValueCell,
        ctx: &CelContext,
    ) -> ValueCellResult<ValueCell> {
        match ast {
            NegList::List { not: _, tail } => {
                let new_value = val.neg()?;

                self.eval_neg(tail, new_value, ctx)
            }
            NegList::EmptyList(_) => Ok(val),
        }
    }

    fn eval_member(&self, ast: &Member, ctx: &CelContext) -> ValueCellResult<ValueCell> {
        let primary = self.eval_primary(&ast.primary, ctx)?;

        self.eval_member_prime(&ast.member, primary, ctx)
    }

    fn eval_member_prime(
        &self,
        ast: &MemberPrime,
        base: ValueCell,
        _ctx: &CelContext,
    ) -> ValueCellResult<ValueCell> {
        match ast {
            MemberPrime::MemberAccess { dot, ident, tail } => {
                Err(ValueCellError::with_msg("Member access not implemented"))
            }
            MemberPrime::Call(_expr_list) => Err(ValueCellError::with_msg("Call not implemented")),
            MemberPrime::ArrayAccess(_exprlist) => {
                Err(ValueCellError::with_msg("Array access not implemented"))
            }
            MemberPrime::Empty(_) => Ok(base),
        }
    }

    fn eval_primary(&self, ast: &Primary, ctx: &CelContext) -> ValueCellResult<ValueCell> {
        match ast {
            Primary::Ident(child) => match ctx.get_param_by_name(&child.to_string()) {
                Some(param) => Ok(param),
                None => Err(ValueCellError::with_msg(&format!(
                    "Ident '{}' does not exist",
                    child.to_string()
                ))),
            },
            Primary::Parens(_child) => Err(ValueCellError::with_msg("Call not implemented")),
            Primary::ListAccess(_child) => {
                Err(ValueCellError::with_msg("Array Access not implemented"))
            }
            Primary::ObjectAccess(_child) => {
                Err(ValueCellError::with_msg("Object access not implemented"))
            }
            Primary::Literal(literal) => match literal {
                Literal::Null(_) => Ok(ValueCell::from_null()),
                Literal::Lit(lit) => match lit {
                    Lit::Int(val) => Ok(ValueCell::from_int(val.into_inner())),
                    Lit::Uint(val) => {
                        let source = val.span().source_text().unwrap();

                        if source.ends_with("u") {
                            Ok(ValueCell::from_uint(val.into_inner()))
                        } else {
                            Ok(ValueCell::from_int(val.into_inner() as i64))
                        }
                    }
                    Lit::Float(val) => Ok(ValueCell::from_float(*val.into_inner())),
                    Lit::Bool(val) => Ok(ValueCell::from_bool(val.into_inner())),
                    Lit::Str(val) => Ok(ValueCell::from_string(val)),
                    Lit::ByteStr(val) => Ok(ValueCell::from_bytes(val)),
                    _ => Err(ValueCellError::with_msg(
                        "Byte and Char literal not allowed",
                    )),
                },
            },
        }
    }

    fn parse_expr_list(&self, ast: &ExprList, ctx: &CelContext) -> ValueCellResult<ValueCell> {
        let mut exprs: Vec<ValueCell> = Vec::new();
        exprs.push(self.eval_expr(&ast.expr, ctx)?);

        for expr in ast.tail.iter() {
            exprs.push(self.eval_expr(&expr.expr, ctx)?);
        }

        Ok(ValueCell::from_list(&exprs))
    }
}

#[cfg(test)]
mod test {
    use super::Program;

    #[test]
    fn test_basic_prog() {
        let prog = Program::from_source("foo + 3").unwrap();

        assert!(prog.params().len() == 1);
        assert!(prog.params()[0] == "foo");
    }

    #[test]
    fn test_complex_prog() {
        let prog = Program::from_source("((foo.bar + 2) * foo.baz) / bam").unwrap();

        assert!(prog.params().len() == 2);
    }
}
