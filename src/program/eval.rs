use std::ops::{Neg, Not};

use parsel::ast::Lit;

use crate::{
    parser::{
        AddOp, Addition, ConditionalAnd, ConditionalOr, Expr, ExprList, Literal, Member,
        MemberPrime, MultOp, Multiplication, NegList, NotList, Primary, Relation, Relop, Unary,
    },
    value_cell::{ValueCell, ValueCellError, ValueCellResult},
    CelContext,
};

pub fn eval_expr(ast: &Expr, ctx: &CelContext) -> ValueCellResult<ValueCell> {
    let expr_res = eval_or(&ast.cond_or, ctx)?;

    if let Some(ternary) = ast.ternary.as_prefix() {
        if let ValueCell::Bool(val) = expr_res {
            if val {
                return eval_or(&ternary.true_clause, ctx);
            } else {
                return eval_expr(&ternary.false_clause, ctx);
            }
        }
        return Err(ValueCellError::with_msg(&format!(
            "Invalid op '?' on type {:?}",
            expr_res.as_type()
        )));
    } else {
        Ok(expr_res)
    }
}

fn eval_or(ast: &ConditionalOr, ctx: &CelContext) -> ValueCellResult<ValueCell> {
    match ast {
        ConditionalOr::Binary { lhs, op: _, rhs } => {
            let left = eval_or(lhs, ctx)?;

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

            let right = eval_and(rhs, ctx)?;
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
        ConditionalOr::Rhs(child) => eval_and(child, ctx),
    }
}

fn eval_and(ast: &ConditionalAnd, ctx: &CelContext) -> ValueCellResult<ValueCell> {
    match ast {
        ConditionalAnd::Binary { lhs, op: _, rhs } => {
            let left = eval_and(lhs, ctx)?;
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

            let right = eval_relation(rhs, ctx)?;
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
        ConditionalAnd::Rhs(child) => eval_relation(child, ctx),
    }
}

fn eval_relation(ast: &Relation, ctx: &CelContext) -> ValueCellResult<ValueCell> {
    match ast {
        Relation::Binary { lhs, op, rhs } => {
            let lhs = eval_relation(lhs, ctx)?;
            let rhs = eval_addition(rhs, ctx)?;

            let lhs_type = lhs.as_type();
            let rhs_type = rhs.as_type();

            match op {
                Relop::Lt(_) => return lhs.lt(&rhs),
                Relop::Le(_) => return lhs.le(&rhs),
                Relop::Ge(_) => return lhs.ge(&rhs),
                Relop::Gt(_) => return lhs.gt(&rhs),
                Relop::Eq(_) => return lhs.eq(&rhs),
                Relop::Ne(_) => return lhs.neq(&rhs),
                Relop::In(_) => {
                    match rhs {
                        ValueCell::List(l) => {
                            for value in l.iter() {
                                if let Ok(ValueCell::Bool(res)) = lhs.eq(value) {
                                    if res {
                                        return Ok(ValueCell::from_bool(true));
                                    }
                                }
                            }

                            return Ok(ValueCell::from_bool(false));
                        }
                        ValueCell::Map(m) => {
                            if let ValueCell::String(r) = lhs {
                                return Ok(ValueCell::from_bool(m.contains_key(&r)));
                            } else {
                                return Err(ValueCellError::with_msg(&format!(
                                    "Op 'in' invalid between {:?} and {:?}",
                                    rhs_type, lhs_type
                                )));
                            }
                        }
                        _ => {}
                    }
                    return Err(ValueCellError::with_msg(&format!(
                        "Op 'in' invalid between {:?} and {:?}",
                        rhs_type, lhs_type
                    )));
                }
            }
        }
        Relation::Rhs(child) => eval_addition(child, ctx),
    }
}

fn eval_addition(ast: &Addition, ctx: &CelContext) -> ValueCellResult<ValueCell> {
    match ast {
        Addition::Binary { lhs, op, rhs } => {
            let lhs = eval_addition(lhs, ctx)?;
            let rhs = eval_multiplication(rhs, ctx)?;

            match op {
                AddOp::Add(_) => lhs + rhs,
                AddOp::Sub(_) => lhs - rhs,
            }
        }
        Addition::Rhs(child) => eval_multiplication(child, ctx),
    }
}

fn eval_multiplication(ast: &Multiplication, ctx: &CelContext) -> ValueCellResult<ValueCell> {
    match ast {
        Multiplication::Binary { lhs, op, rhs } => {
            let lhs = eval_multiplication(lhs, ctx)?;
            let rhs = eval_unary(rhs, ctx)?;

            match op {
                MultOp::Mult(_) => lhs * rhs,
                MultOp::Div(_) => lhs / rhs,
                MultOp::Mod(_) => lhs % rhs,
            }
        }
        Multiplication::Rhs(child) => eval_unary(child, ctx),
    }
}

fn eval_unary(ast: &Unary, ctx: &CelContext) -> ValueCellResult<ValueCell> {
    match ast {
        Unary::Member(child) => eval_member(child, ctx),
        Unary::NotMember { nots, member } => {
            let m = eval_member(member, ctx)?;

            eval_not(nots, m, ctx)
        }
        Unary::NegMember { negs, member } => {
            let m = eval_member(member, ctx)?;

            eval_neg(negs, m, ctx)
        }
    }
}

fn eval_not(ast: &NotList, val: ValueCell, ctx: &CelContext) -> ValueCellResult<ValueCell> {
    match ast {
        NotList::List { not: _, tail } => {
            let new_value = val.not()?;

            eval_not(tail, new_value, ctx)
        }
        NotList::EmptyList(_) => Ok(val),
    }
}

fn eval_neg(ast: &NegList, val: ValueCell, ctx: &CelContext) -> ValueCellResult<ValueCell> {
    match ast {
        NegList::List { not: _, tail } => {
            let new_value = val.neg()?;

            eval_neg(tail, new_value, ctx)
        }
        NegList::EmptyList(_) => Ok(val),
    }
}

fn eval_member(ast: &Member, ctx: &CelContext) -> ValueCellResult<ValueCell> {
    let primary = eval_primary(&ast.primary, ctx)?;

    eval_member_prime(&ast.member, &mut vec![primary], ctx)
}

fn eval_member_prime(
    ast: &MemberPrime,
    fqn: &mut Vec<ValueCell>,
    ctx: &CelContext,
) -> ValueCellResult<ValueCell> {
    match ast {
        MemberPrime::MemberAccess {
            dot: _,
            ident,
            tail,
        } => {
            fqn.push(ValueCell::Ident(ident.to_string()));
            eval_member_prime(tail, fqn, ctx)
        }
        MemberPrime::Call(expr_list) => {
            let func_name = match fqn.pop() {
                Some(ValueCell::Ident(name)) => name,
                None => return Err(ValueCellError::with_msg("Empty function name")),
                other => {
                    return Err(ValueCellError::with_msg(&format!(
                        "{:?} cannot be called",
                        other.unwrap()
                    )))
                }
            };

            let subject = match ctx.resolve_fqn(&fqn) {
                Ok(subject) => subject,
                Err(_) => ValueCell::from_null(),
            };

            if let Some(func) = ctx.get_func_by_name(&func_name) {
                let arg_list = match (*expr_list).as_prefix() {
                    Some(expr_list_ast) => eval_expr_list(expr_list_ast, ctx)?,
                    None => ValueCell::from_list(&[]),
                };
                func(subject, arg_list)
            } else if let Some(macro_) = ctx.get_macro_by_name(&func_name) {
                let arg_list: Vec<&Expr> = match (*expr_list).as_prefix() {
                    Some(expr_list_ast) => {
                        let mut list = Vec::new();
                        list.push(&expr_list_ast.expr);

                        for expr_tail in expr_list_ast.tail.iter() {
                            list.push(&expr_tail.expr);
                        }
                        list
                    }
                    None => Vec::new(),
                };
                macro_(ctx, subject, &arg_list)
            } else {
                Err(ValueCellError::with_msg(&format!(
                    "ident {} not available",
                    func_name
                )))
            }
        }
        MemberPrime::ArrayAccess(expr) => {
            let val = ctx.resolve_fqn(&fqn)?;
            let base_type = val.as_type();
            let expr_res = eval_expr(expr, ctx)?;

            match val {
                ValueCell::List(l) => {
                    let index: usize = match expr_res {
                        ValueCell::Int(v) => {
                            if v < 0 {
                                return Err(ValueCellError::with_msg(&format!(
                                    "Index of {} is invalid",
                                    v
                                )));
                            } else {
                                v as usize
                            }
                        }
                        ValueCell::UInt(v) => v as usize,
                        _ => {
                            return Err(ValueCellError::with_msg(&format!(
                                "Index of array {:?} is invalid for array",
                                base_type
                            )))
                        }
                    };

                    if index >= l.len() {
                        return Err(ValueCellError::with_msg(&format!(
                            "Index {} out of range on list",
                            index
                        )));
                    }

                    Ok(l[index].clone())
                }
                ValueCell::Map(l) => {
                    if let ValueCell::String(i) = expr_res {
                        match l.get(&i) {
                            Some(value) => Ok(value.clone()),
                            None => Err(ValueCellError::with_msg(&format!(
                                "Member '{}' not available for '{:?}'",
                                i, base_type
                            ))),
                        }
                    } else {
                        return Err(ValueCellError::with_msg(&format!(
                            "Object access invalid for type {:?}",
                            expr_res.as_type()
                        )));
                    }
                }
                _ => Err(ValueCellError::with_msg(&format!(
                    "Index operation invalid on type {:?}",
                    base_type
                ))),
            }
        }
        MemberPrime::Empty(_) => ctx.resolve_fqn(&fqn),
    }
}

fn eval_primary(ast: &Primary, ctx: &CelContext) -> ValueCellResult<ValueCell> {
    match ast {
        Primary::Ident(child) => Ok(ValueCell::from_ident(&child.to_string())),
        Primary::Parens(child) => eval_expr(child, ctx),
        Primary::ListConstruction(list) => match (*list).as_prefix() {
            Some(exprlist) => eval_expr_list(exprlist, ctx),
            None => Ok(ValueCell::from_list(&[])),
        },
        Primary::ObjectInit(_child) => Err(ValueCellError::with_msg("Object init not implemented")),
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

fn eval_expr_list(ast: &ExprList, ctx: &CelContext) -> ValueCellResult<ValueCell> {
    let mut exprs: Vec<ValueCell> = Vec::new();
    exprs.push(eval_expr(&ast.expr, ctx)?);

    for expr in ast.tail.iter() {
        exprs.push(eval_expr(&expr.expr, ctx)?);
    }

    Ok(ValueCell::from_list(&exprs))
}
