use crate::ast::grammar::{
    Addition, ConditionalAnd, ConditionalOr, Expr, ExprList, ExprListTail, Member, MemberPrime,
    Multiplication, Primary, Relation, Unary,
};
use serde::Serialize;
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize)]
pub struct ProgramDetails {
    params: HashSet<String>,
}

impl ProgramDetails {
    pub fn from_ast(ast: &Expr) -> ProgramDetails {
        let mut dets = ProgramDetails {
            params: HashSet::new(),
        };

        dets.parse_expr(ast);
        dets
    }

    pub fn params<'a>(&'a self) -> Vec<&'a str> {
        self.params.iter().map(|x| x.as_str()).collect()
    }

    fn parse_expr(&mut self, ast: &Expr) {
        self.parse_or(&ast.cond_or);

        if let Some(inner) = ast.ternary.as_prefix() {
            self.parse_or(&inner.true_clause);
            self.parse_expr(&inner.false_clause);
        }
    }

    fn parse_or(&mut self, ast: &ConditionalOr) {
        match ast {
            ConditionalOr::Binary { lhs, op: _, rhs } => {
                self.parse_or(lhs);
                self.parse_and(rhs);
            }
            ConditionalOr::Rhs(child) => self.parse_and(child),
        }
    }

    fn parse_and(&mut self, ast: &ConditionalAnd) {
        match ast {
            ConditionalAnd::Binary { lhs, op: _, rhs } => {
                self.parse_and(lhs);
                self.parse_relation(rhs);
            }
            ConditionalAnd::Rhs(child) => self.parse_relation(child),
        }
    }

    fn parse_relation(&mut self, ast: &Relation) {
        match ast {
            Relation::Binary { lhs, op: _, rhs } => {
                self.parse_relation(lhs);
                self.parse_addition(rhs);
            }
            Relation::Rhs(child) => self.parse_addition(child),
        }
    }

    fn parse_addition(&mut self, ast: &Addition) {
        match ast {
            Addition::Binary { lhs, op: _, rhs } => {
                self.parse_addition(lhs);
                self.parse_multiplication(rhs);
            }
            Addition::Rhs(child) => self.parse_multiplication(child),
        }
    }

    fn parse_multiplication(&mut self, ast: &Multiplication) {
        match ast {
            Multiplication::Binary { lhs, op: _, rhs } => {
                self.parse_multiplication(lhs);
                self.parse_unary(rhs);
            }
            Multiplication::Rhs(child) => self.parse_unary(child),
        }
    }

    fn parse_unary(&mut self, ast: &Unary) {
        match ast {
            Unary::Member(child) => self.parse_member(child),
            Unary::NotMember { nots: _, member } => self.parse_member(member),
            Unary::NegMember { negs: _, member } => self.parse_member(member),
        }
    }

    fn parse_member(&mut self, ast: &Member) {
        self.parse_primary(&ast.primary);
        self.parse_member_prime(&ast.member);
    }

    fn parse_member_prime(&mut self, ast: &MemberPrime) {
        match ast {
            MemberPrime::MemberAccess {
                dot: _,
                ident: _,
                tail,
            } => self.parse_member_prime(tail),
            MemberPrime::Call { call, tail } => {
                match (*call).as_prefix() {
                    Some(exprlist) => self.parse_expr_list(exprlist),
                    None => {}
                };
                self.parse_member_prime(tail);
            }
            MemberPrime::ArrayAccess { brackets, tail } => {
                self.parse_expr(brackets);
                self.parse_member_prime(tail)
            }
            MemberPrime::Empty(_) => {}
        }
    }

    fn parse_primary(&mut self, ast: &Primary) {
        match ast {
            Primary::Ident(child) => {
                println!("{:?}", child);
                self.params.insert(child.to_string());
            }
            Primary::Parens(child) => self.parse_expr(child.as_ref()),
            Primary::ListConstruction(child) => match child.as_ref().as_prefix() {
                Some(child) => self.parse_expr_list(child),
                None => {}
            },
            _ => {}
        }
    }

    fn parse_expr_list(&mut self, ast: &ExprList) {
        self.parse_expr(&ast.expr);

        for expr in ast.tail.iter() {
            self.parse_expr_list_tail(expr)
        }
    }

    fn parse_expr_list_tail(&mut self, ast: &ExprListTail) {
        self.parse_expr(&ast.expr);
    }
}
