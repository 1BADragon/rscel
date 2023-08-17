use parsel::ast::Lit;

use super::ProgramDetails;
use crate::{
    interp::{ByteCode, JmpWhen},
    parser::grammar::{
        AddOp, Addition, ConditionalAnd, ConditionalOr, Expr, ExprList, Literal, Member,
        MemberPrime, MultOp, Multiplication, NegList, NotList, Primary, Relation, Relop, Unary,
    },
    CelError, CelResult, CelValue, Program,
};

pub struct ProgramCompiler {
    details: ProgramDetails,

    source: String,
}

impl ProgramCompiler {
    pub fn new() -> ProgramCompiler {
        ProgramCompiler {
            details: ProgramDetails::new(),
            source: String::new(),
        }
    }

    pub fn with_source(mut self, source: &str) -> ProgramCompiler {
        self.source = source.to_owned();
        self
    }

    pub fn build(mut self) -> CelResult<Program> {
        let ast: Expr = match parsel::parse_str(&self.source) {
            Ok(expr) => expr,
            Err(err) => {
                let span = err.span();
                return Err(CelError::syntax(&span));
            }
        };

        let bytecode = self.build_expr(&ast)?;

        Ok(Program {
            source: self.source,
            details: self.details,
            bytecode,
        })
    }

    fn build_expr(&mut self, ast: &Expr) -> CelResult<Vec<ByteCode>> {
        let mut bytecode = self.build_or(&ast.cond_or)?;

        if let Some(ternary) = ast.ternary.as_prefix() {
            let true_bc = self.build_or(&ternary.true_clause)?;
            let false_bc = self.build_expr(&ternary.false_clause)?;

            bytecode.push(ByteCode::JmpCond {
                when: JmpWhen::False,
                dist: true_bc.len() as u32,
                leave_val: false,
            });
            bytecode.extend_from_slice(&true_bc);
            bytecode.push(ByteCode::Jmp(false_bc.len() as u32));
            bytecode.extend_from_slice(&false_bc);
        }

        Ok(bytecode)
    }

    fn build_or(&mut self, ast: &ConditionalOr) -> CelResult<Vec<ByteCode>> {
        match ast {
            ConditionalOr::Binary { lhs, op: _, rhs } => {
                let mut bytecode = self.build_or(lhs)?;
                let mut rhs_bc = self.build_and(rhs)?;

                rhs_bc.push(ByteCode::Or);

                bytecode.push(ByteCode::JmpCond {
                    when: JmpWhen::True,
                    dist: rhs_bc.len() as u32,
                    leave_val: true,
                });
                bytecode.extend_from_slice(&rhs_bc);
                Ok(bytecode)
            }
            ConditionalOr::Rhs(child) => self.build_and(child),
        }
    }

    fn build_and(&mut self, ast: &ConditionalAnd) -> CelResult<Vec<ByteCode>> {
        match ast {
            ConditionalAnd::Binary { lhs, op: _, rhs } => {
                let mut bytecode = self.build_and(lhs)?;
                let mut rhs_bc = self.build_relation(rhs)?;

                rhs_bc.push(ByteCode::And);

                bytecode.push(ByteCode::JmpCond {
                    when: JmpWhen::False,
                    dist: rhs_bc.len() as u32,
                    leave_val: true,
                });
                bytecode.append(&mut rhs_bc);
                Ok(bytecode)
            }
            ConditionalAnd::Rhs(child) => self.build_relation(child),
        }
    }
    fn build_relation(&mut self, ast: &Relation) -> CelResult<Vec<ByteCode>> {
        match ast {
            Relation::Binary { lhs, op, rhs } => {
                let mut bytecode = self.build_relation(lhs)?;
                bytecode.append(&mut self.build_addition(rhs)?);

                match op {
                    Relop::Lt(_) => bytecode.push(ByteCode::Lt),
                    Relop::Le(_) => bytecode.push(ByteCode::Le),
                    Relop::Ge(_) => bytecode.push(ByteCode::Ge),
                    Relop::Gt(_) => bytecode.push(ByteCode::Gt),
                    Relop::Eq(_) => bytecode.push(ByteCode::Eq),
                    Relop::Ne(_) => bytecode.push(ByteCode::Ne),
                    Relop::In(_) => bytecode.push(ByteCode::In),
                };

                Ok(bytecode)
            }
            Relation::Rhs(child) => self.build_addition(child),
        }
    }
    fn build_addition(&mut self, ast: &Addition) -> CelResult<Vec<ByteCode>> {
        match ast {
            Addition::Binary { lhs, op, rhs } => {
                let mut bytecode = self.build_addition(lhs)?;
                bytecode.append(&mut self.build_multiplication(rhs)?);

                match op {
                    AddOp::Add(_) => bytecode.push(ByteCode::Add),
                    AddOp::Sub(_) => bytecode.push(ByteCode::Sub),
                };

                Ok(bytecode)
            }
            Addition::Rhs(child) => self.build_multiplication(child),
        }
    }
    fn build_multiplication(&mut self, ast: &Multiplication) -> CelResult<Vec<ByteCode>> {
        match ast {
            Multiplication::Binary { lhs, op, rhs } => {
                let mut bytecode = self.build_multiplication(lhs)?;
                bytecode.append(&mut self.build_unary(rhs)?);

                match op {
                    MultOp::Mult(_) => bytecode.push(ByteCode::Mul),
                    MultOp::Div(_) => bytecode.push(ByteCode::Div),
                    MultOp::Mod(_) => bytecode.push(ByteCode::Mod),
                };

                Ok(bytecode)
            }
            Multiplication::Rhs(child) => self.build_unary(child),
        }
    }
    fn build_unary(&mut self, ast: &Unary) -> CelResult<Vec<ByteCode>> {
        match ast {
            Unary::Member(child) => self.build_member(child),
            Unary::NotMember { nots, member } => {
                let mut bytecode = self.build_member(member)?;

                bytecode.append(&mut self.build_not(nots)?);
                Ok(bytecode)
            }
            Unary::NegMember { negs, member } => {
                let mut bytecode = self.build_member(member)?;
                bytecode.append(&mut self.build_neg(negs)?);

                Ok(bytecode)
            }
        }
    }

    fn build_not(&mut self, ast: &NotList) -> CelResult<Vec<ByteCode>> {
        match ast {
            NotList::List { not: _, tail } => {
                let mut bytecode = vec![ByteCode::Not];

                bytecode.append(&mut self.build_not(tail)?);
                Ok(bytecode)
            }
            NotList::EmptyList(_) => Ok(Vec::new()),
        }
    }

    fn build_neg(&mut self, ast: &NegList) -> CelResult<Vec<ByteCode>> {
        match ast {
            NegList::List { not: _, tail } => {
                let mut bytecode = vec![ByteCode::Neg];

                bytecode.append(&mut self.build_neg(tail)?);
                Ok(bytecode)
            }
            NegList::EmptyList(_) => Ok(Vec::new()),
        }
    }

    fn build_member(&mut self, ast: &Member) -> CelResult<Vec<ByteCode>> {
        let mut bytecode = self.build_primary(&ast.primary)?;

        bytecode.append(&mut self.build_member_prime(&ast.member)?);
        Ok(bytecode)
    }

    fn build_member_prime(&mut self, ast: &MemberPrime) -> CelResult<Vec<ByteCode>> {
        match ast {
            MemberPrime::MemberAccess {
                dot: _,
                ident,
                tail,
            } => {
                let mut bytecode = vec![
                    ByteCode::Push(CelValue::from_ident(&ident.to_string())),
                    ByteCode::Access,
                ];

                bytecode.append(&mut self.build_member_prime(tail)?);
                Ok(bytecode)
            }
            MemberPrime::Call { call, tail } => {
                let mut bytecode = Vec::new();
                let arg_list = match (*call).as_prefix() {
                    Some(expr_list_ast) => self.build_expr_list(expr_list_ast)?,
                    None => Vec::new(),
                };
                let n_args = arg_list.len();

                for arg in arg_list.into_iter().rev() {
                    bytecode.push(ByteCode::Push(arg.into()))
                }

                bytecode.push(ByteCode::Call(n_args as u32));

                bytecode.append(&mut self.build_member_prime(tail)?);

                Ok(bytecode)
            }
            MemberPrime::ArrayAccess { brackets, tail } => {
                let mut bytecode = self.build_expr(brackets)?;
                bytecode.push(ByteCode::Index);
                bytecode.append(&mut self.build_member_prime(tail)?);
                Ok(bytecode)
            }
            MemberPrime::Empty(_) => Ok(vec![]),
        }
    }

    fn build_primary(&mut self, ast: &Primary) -> CelResult<Vec<ByteCode>> {
        match ast {
            Primary::Type(_) => Ok(vec![ByteCode::Push(CelValue::from_ident("type"))]),
            Primary::Ident(child) => {
                self.details.add_param(&child.to_string());
                Ok(vec![ByteCode::Push(CelValue::from_ident(
                    &child.to_string(),
                ))])
            }
            Primary::Parens(child) => self.build_expr(child),
            Primary::ListConstruction(list) => match (*list).as_prefix() {
                Some(exprlist) => {
                    let mut bytecode = Vec::new();

                    let fragments = self.build_expr_list(exprlist)?;
                    let n_frags = fragments.len();

                    for mut fragment in fragments.into_iter() {
                        bytecode.append(&mut fragment);
                    }

                    bytecode.push(ByteCode::MkList(n_frags as u32));
                    Ok(bytecode)
                }
                None => Ok(vec![ByteCode::Push(CelValue::from_list(vec![]))]),
            },
            Primary::ObjectInit(child) => {
                let mut count = 0;
                let mut bytecode = Vec::new();

                if let Some(inner) = (*child).as_prefix() {
                    for pair in inner.into_iter() {
                        bytecode.append(&mut self.build_expr(&pair.value)?);
                        bytecode.append(&mut self.build_expr(&pair.key)?);
                        count += 1;
                    }
                }

                bytecode.push(ByteCode::MkDict(count));
                Ok(bytecode)
            }
            Primary::Literal(literal) => {
                let bytecode = vec![ByteCode::Push(match literal {
                    Literal::Null(_) => CelValue::from_null(),
                    Literal::Lit(lit) => match lit {
                        Lit::Int(val) => CelValue::from_int(val.into_inner()),
                        Lit::Uint(val) => {
                            let source = val.span().source_text().unwrap();

                            if source.ends_with("u") {
                                CelValue::from_uint(val.into_inner())
                            } else {
                                CelValue::from_int(val.into_inner() as i64)
                            }
                        }
                        Lit::Float(val) => CelValue::from_float(*val.into_inner()),
                        Lit::Bool(val) => CelValue::from_bool(val.into_inner()),
                        Lit::Str(val) => CelValue::from_string(val.value()),
                        Lit::ByteStr(val) => CelValue::from_bytes(val.to_vec()),
                        _ => return Err(CelError::misc("Byte and Char literal not allowed")),
                    },
                })];
                Ok(bytecode)
            }
        }
    }

    fn build_expr_list(&mut self, ast: &ExprList) -> CelResult<Vec<Vec<ByteCode>>> {
        let mut fragments = Vec::new();

        fragments.push(self.build_expr(&ast.expr)?);

        for expr in ast.tail.iter() {
            fragments.push(self.build_expr(&expr.expr)?);
        }

        Ok(fragments)
    }
}
