use std::collections::HashMap;

mod pattern_utils;

use pattern_utils::PrefixPattern;

use super::{
    ast_node::AstNode,
    compiled_prog::{CompiledProg, NodeValue, PreResolvedCodePoint},
    grammar::*,
    source_range::SourceRange,
    syntax_error::SyntaxError,
    tokenizer::{TokenWithLoc, Tokenizer},
    tokens::{AsToken, FStringSegment, IntoToken, Token},
};
use crate::{
    interp::{Interpreter, JmpWhen},
    BindContext, ByteCode, CelError, CelResult, CelValue, CelValueDyn, Program, StringTokenizer,
};

use crate::compile;

pub struct CelCompiler<'l> {
    tokenizer: &'l mut dyn Tokenizer,
    bindings: BindContext<'l>,

    next_label: u32,
}

impl<'l> CelCompiler<'l> {
    pub fn with_tokenizer(tokenizer: &'l mut dyn Tokenizer) -> Self {
        CelCompiler {
            tokenizer,
            bindings: BindContext::for_compile(),
            next_label: 0,
        }
    }

    pub fn compile(mut self) -> CelResult<Program> {
        let (cprog, ast) = self.parse_expression()?;

        if !self.tokenizer.peek()?.is_none() {
            return Err(SyntaxError::from_location(self.tokenizer.location())
                .with_message(format!("Unexpected token: {:?}", self.tokenizer.peek()?))
                .into());
        }

        let mut prog = cprog.into_program(self.tokenizer.source().to_owned());
        prog.details_mut().add_ast(ast);

        Ok(prog)
    }

    fn new_label(&mut self) -> u32 {
        let n = self.next_label;
        self.next_label += 1;
        n
    }

    fn parse_expression(&mut self) -> CelResult<(CompiledProg, AstNode<Expr>)> {
        if let Some(Token::Match) = self.tokenizer.peek()?.as_token() {
            self.tokenizer.next()?;
            self.parse_match_expression()
        } else {
            let (lhs_node, lhs_ast) = self.parse_conditional_or()?;

            match self.tokenizer.peek()?.as_token() {
                Some(Token::Walwrus) => {
                    self.tokenizer.next()?;

                    self.parse_walwrus_expression((lhs_node, lhs_ast))
                }
                Some(Token::Question) => {
                    self.tokenizer.next()?;
                    self.parse_turnary_expression(lhs_node, lhs_ast)
                }
                _ => {
                    let range = lhs_ast.range();
                    Ok((
                        CompiledProg::from_node(lhs_node),
                        AstNode::new(Expr::Unary(Box::new(lhs_ast)), range),
                    ))
                }
            }
        }
    }

    fn parse_turnary_expression(
        &mut self,
        or_prog: CompiledProg,
        or_ast: AstNode<ConditionalOr>,
    ) -> CelResult<(CompiledProg, AstNode<Expr>)> {
        let (expr_node, mut details) = or_prog.into_parts();

        let (true_clause_node, true_clause_ast) = self.parse_conditional_or()?;
        let (true_clause_node, true_clause_details) = true_clause_node.into_parts();

        let next = self.tokenizer.next()?;
        if next.as_token() != Some(&Token::Colon) {
            return Err(SyntaxError::from_location(self.tokenizer.location())
                .with_message(format!("Unexpected token {:?}, expected COLON", next))
                .into());
        }

        let (false_clause_node, false_clause_ast) = self.parse_expression()?;
        let (false_clause_node, false_clause_details) = false_clause_node.into_parts();

        let range = or_ast.range().surrounding(false_clause_ast.range());

        details.union_from(true_clause_details);
        details.union_from(false_clause_details);

        let turnary_node = if let NodeValue::ConstExpr(i) = expr_node {
            if i.is_err() {
                CompiledProg {
                    inner: NodeValue::ConstExpr(i),
                    details,
                }
            } else {
                if cfg!(feature = "type_prop") {
                    if i.is_truthy() {
                        CompiledProg {
                            inner: true_clause_node,
                            details,
                        }
                    } else {
                        CompiledProg {
                            inner: false_clause_node,
                            details,
                        }
                    }
                } else {
                    if let CelValue::Bool(b) = i {
                        if b {
                            CompiledProg {
                                inner: true_clause_node,
                                details,
                            }
                        } else {
                            CompiledProg {
                                inner: false_clause_node,
                                details,
                            }
                        }
                    } else {
                        CompiledProg {
                            inner: NodeValue::ConstExpr(CelValue::from_err(CelError::Value(
                                format!("{} cannot be converted to bool", i.as_type()),
                            ))),
                            details,
                        }
                    }
                }
            }
        } else {
            let true_clause_bytecode = true_clause_node.into_bytecode();
            let false_clause_bytecode = false_clause_node.into_bytecode();

            let after_true_clause = self.new_label();
            let end_label = self.new_label();

            CompiledProg {
                inner: NodeValue::Bytecode(
                    expr_node
                        .into_bytecode()
                        .into_iter()
                        .chain(
                            [PreResolvedCodePoint::JmpCond {
                                when: JmpWhen::False,
                                label: after_true_clause,
                            }]
                            .into_iter(),
                        )
                        .chain(true_clause_bytecode.into_iter())
                        .chain(
                            [
                                PreResolvedCodePoint::Jmp { label: end_label },
                                PreResolvedCodePoint::Label(after_true_clause),
                            ]
                            .into_iter(),
                        )
                        .chain(false_clause_bytecode.into_iter())
                        .chain([PreResolvedCodePoint::Label(end_label)].into_iter())
                        .collect(),
                ),
                details,
            }
        };

        Ok((
            turnary_node,
            AstNode::new(
                Expr::Ternary {
                    condition: Box::new(or_ast),
                    true_clause: Box::new(true_clause_ast),
                    false_clause: Box::new(false_clause_ast),
                },
                range,
            ),
        ))
    }

    fn parse_walwrus_expression(
        &mut self,
        lhs: (CompiledProg, AstNode<ConditionalOr>),
    ) -> CelResult<(CompiledProg, AstNode<Expr>)> {
        let (_, lhs_ast) = lhs;

        let lhs_range = lhs_ast.range().clone();

        // lol
        let ident = if let AstNode {
            loc: _,
            node:
                ConditionalOr::Unary(AstNode {
                    loc: _,
                    node:
                        ConditionalAnd::Unary(AstNode {
                            loc: _,
                            node:
                                Relation::Unary(AstNode {
                                    loc: _,
                                    node:
                                        Addition::Unary(AstNode {
                                            loc: _,
                                            node:
                                                Multiplication::Unary(AstNode {
                                                    loc: _,
                                                    node:
                                                        Unary::Member(AstNode {
                                                            loc: _,
                                                            node:
                                                                Member {
                                                                    primary:
                                                                        AstNode {
                                                                            loc: _,
                                                                            node:
                                                                                Primary::Ident(ident),
                                                                        },
                                                                    member,
                                                                },
                                                        }),
                                                }),
                                        }),
                                }),
                        }),
                }),
        } = lhs_ast
        {
            if !member.is_empty() {
                return Err(CelError::Syntax(
                    SyntaxError::from_location(self.tokenizer.location())
                        .with_message("Walwrus op expects ident on lhs".to_owned()),
                ));
            }

            ident.clone()
        } else {
            return Err(CelError::Syntax(
                SyntaxError::from_location(self.tokenizer.location())
                    .with_message("Walwrus op expects ident on lhs".to_owned()),
            ));
        };

        let (rhs_node, rhs_ast) = self.parse_expression()?;

        let range = lhs_range.clone().surrounding(rhs_ast.range());

        let ast = AstNode::new(
            Expr::Walwrus {
                ident: AstNode::new(ident.clone(), lhs_range),
                expr: Box::new(rhs_ast),
            },
            range,
        );

        let mut node = CompiledProg::from_node_as_bytecode(rhs_node);

        node.append_if_bytecode([
            PreResolvedCodePoint::Bytecode(ByteCode::Push(CelValue::Ident(ident.0))),
            PreResolvedCodePoint::Bytecode(ByteCode::Store),
        ]);

        Ok((node, ast))
    }

    fn parse_match_expression(&mut self) -> CelResult<(CompiledProg, AstNode<Expr>)> {
        let (condition_node, condition_ast) = self.parse_expression()?;

        let mut range = condition_ast.range();

        let (node_value, mut node_details) = condition_node.into_parts();
        let mut node_bytecode = node_value.into_bytecode();

        let next = self.tokenizer.next()?;
        if next.as_token() != Some(&Token::LBrace) {
            return Err(SyntaxError::from_location(self.tokenizer.location())
                .with_message(format!("Unexpected token {:?}, expected LBRACE", next))
                .into());
        }

        let mut expressions: Vec<AstNode<MatchCase>> = Vec::new();

        let mut all_parts = Vec::new();

        let mut comma_seen = true;

        loop {
            // the rbrace at the end of the match
            let rbrace = self.tokenizer.peek()?;
            if rbrace.as_token() == Some(&Token::RBrace) {
                range = range.surrounding(rbrace.unwrap().loc);
                break;
            }

            if !comma_seen {
                return Err(SyntaxError::from_location(self.tokenizer.location())
                    .with_message(format!("Expected COMMA"))
                    .into());
            }
            comma_seen = false;

            // case
            let case_token = self.tokenizer.next()?;
            if case_token.as_token() != Some(&Token::Case) {
                return Err(SyntaxError::from_location(self.tokenizer.location())
                    .with_message(format!("Unexpected token {:?}, expected CASE", next))
                    .into());
            }
            //pattern
            let (pattern_prog, pattern_ast) = self.parse_match_pattern()?;
            let (pattern_bytecode, pattern_details) = pattern_prog.into_parts();
            let pattern_bytecode = pattern_bytecode.into_bytecode();

            node_details.union_from(pattern_details);

            let pattern_range = pattern_ast.range();

            // colon after pattern
            let colon_token = self.tokenizer.next()?;
            if colon_token.as_token() != Some(&Token::Colon) {
                return Err(SyntaxError::from_location(self.tokenizer.location())
                    .with_message(format!("Unexpected token {:?}, expected COLON", next))
                    .into());
            }

            // eval expression
            let (expr_prog, expr_ast) = self.parse_expression()?;
            let (expr_bytecode, expr_details) = expr_prog.into_parts();
            let expr_bytecode: Vec<_> = [ByteCode::Pop.into()]
                .into_iter()
                .chain(expr_bytecode.into_bytecode().into_iter())
                .collect();

            node_details.union_from(expr_details);

            let case_range = pattern_range.surrounding(expr_ast.range());

            all_parts.push((pattern_bytecode, expr_bytecode));
            expressions.push(AstNode::new(
                MatchCase {
                    pattern: pattern_ast,
                    expr: Box::new(expr_ast),
                },
                case_range,
            ));
            //
            // comma after pattern
            let comma_token = self.tokenizer.peek()?;
            if comma_token.as_token() == Some(&Token::Comma) {
                comma_seen = true;
                self.tokenizer.next()?;
            }
        }

        // consume the RBRACE
        self.tokenizer.next()?;

        // After match expression label
        let after_match_s_l = self.new_label();

        for (pattern_bytecode, expr_bytecode) in all_parts.into_iter() {
            let after_case_l = self.new_label();

            node_bytecode.push(ByteCode::Dup);
            node_bytecode.extend(pattern_bytecode.into_iter());
            node_bytecode.push(PreResolvedCodePoint::JmpCond {
                when: JmpWhen::False,
                label: after_case_l,
            });

            node_bytecode.extend(expr_bytecode);
            node_bytecode.push(PreResolvedCodePoint::Jmp {
                label: after_match_s_l,
            });
            node_bytecode.push(PreResolvedCodePoint::Label(after_case_l));
        }

        node_bytecode.extend([
            ByteCode::Pop.into(),
            ByteCode::Push(CelValue::from_null()).into(),
            PreResolvedCodePoint::Label(after_match_s_l),
        ]);

        Ok((
            CompiledProg::new(NodeValue::Bytecode(node_bytecode), node_details),
            AstNode::new(
                Expr::Match {
                    condition: Box::new(condition_ast),
                    cases: expressions,
                },
                range,
            ),
        ))
    }

    fn parse_match_pattern(&mut self) -> CelResult<(CompiledProg, AstNode<MatchPattern>)> {
        let start = self.tokenizer.location();
        let mut prefix_pattern = PrefixPattern::Eq;

        if let Some(t) = self.tokenizer.peek()? {
            if let Token::Ident(i) = t.token() {
                let i = i.clone();
                if i == "_" {
                    self.tokenizer.next()?;
                    let range = SourceRange::new(start, self.tokenizer.location());

                    return Ok((
                        CompiledProg::with_bytecode(
                            [
                                ByteCode::Pop,                     // pop off the pattern value
                                ByteCode::Push(CelValue::true_()), // push true
                            ]
                            .into_iter()
                            .collect(),
                        ),
                        AstNode::new(
                            MatchPattern::Any(AstNode::new(MatchAnyPattern {}, range)),
                            range,
                        ),
                    ));
                } else if self.bindings.get_type(&i).is_some() {
                    self.tokenizer.next()?;
                    return Ok((
                        CompiledProg::with_bytecode(
                            [
                                ByteCode::Push(CelValue::Ident("type".to_owned())),
                                ByteCode::Call(1),
                                ByteCode::Push(CelValue::Ident(i.clone())),
                                ByteCode::Eq,
                            ]
                            .into_iter()
                            .collect(),
                        ),
                        AstNode::new(
                            MatchPattern::Type(AstNode::new(
                                MatchTypePattern::from_type_str(&i),
                                SourceRange::new(start, self.tokenizer.location()),
                            )),
                            SourceRange::new(start, self.tokenizer.location()),
                        ),
                    ));
                }
            }

            if let Some(token_prefix_pattern) = PrefixPattern::from_token(t.token()) {
                self.tokenizer.next()?;
                prefix_pattern = token_prefix_pattern;
            }
        }

        let op_range = SourceRange::new(start, self.tokenizer.location());

        let (or_prod, or_ast) = self.parse_conditional_or()?;
        let or_details = or_prod.details().clone();
        let mut or_bc = or_prod.into_unresolved_bytecode();

        or_bc.push(prefix_pattern.as_bytecode());

        Ok((
            CompiledProg::new(NodeValue::Bytecode(or_bc), or_details),
            AstNode::new(
                MatchPattern::Cmp {
                    op: AstNode::new(prefix_pattern.as_ast(), op_range),
                    or: or_ast,
                },
                SourceRange::new(start, self.tokenizer.location()),
            ),
        ))
    }

    fn parse_conditional_or(&mut self) -> CelResult<(CompiledProg, AstNode<ConditionalOr>)> {
        let (mut current_node, mut current_ast) = into_unary(self.parse_conditional_and()?);

        let label = self.new_label();

        loop {
            if let Some(Token::OrOr) = self.tokenizer.peek()?.as_token() {
                self.tokenizer.next()?;
                let (rhs_node, rhs_ast) = self.parse_conditional_and()?;

                let jmp_node = CompiledProg::with_code_points(vec![
                    PreResolvedCodePoint::Bytecode(ByteCode::Test),
                    PreResolvedCodePoint::Bytecode(ByteCode::Dup),
                    PreResolvedCodePoint::JmpCond {
                        when: JmpWhen::True,
                        label,
                    },
                ]);

                let range = current_ast.range().surrounding(rhs_ast.range());

                current_ast = AstNode::new(
                    ConditionalOr::Binary {
                        lhs: Box::new(current_ast),
                        rhs: rhs_ast,
                    },
                    range,
                );
                current_node = compile!(
                    [ByteCode::Or.into()],
                    current_node.or(&rhs_node),
                    current_node,
                    jmp_node,
                    rhs_node
                );
            } else {
                break;
            }
        }

        current_node.append_if_bytecode([PreResolvedCodePoint::Label(label)]);

        Ok((current_node, current_ast))
    }

    fn parse_conditional_and(&mut self) -> CelResult<(CompiledProg, AstNode<ConditionalAnd>)> {
        let (mut current_node, mut current_ast) = into_unary(self.parse_relation()?);

        let label = self.new_label();

        loop {
            if let Some(Token::AndAnd) = self.tokenizer.peek()?.as_token() {
                self.tokenizer.next()?;
                let (rhs_node, rhs_ast) = self.parse_relation()?;

                let jmp_node = CompiledProg::with_code_points(vec![
                    PreResolvedCodePoint::Bytecode(ByteCode::Test),
                    PreResolvedCodePoint::Bytecode(ByteCode::Dup),
                    PreResolvedCodePoint::JmpCond {
                        when: JmpWhen::False,
                        label: label,
                    },
                ]);

                let range = current_ast.range().surrounding(rhs_ast.range());

                current_ast = AstNode::new(
                    ConditionalAnd::Binary {
                        lhs: Box::new(current_ast),
                        rhs: rhs_ast,
                    },
                    range,
                );
                current_node = compile!(
                    [ByteCode::And.into()],
                    current_node.and(rhs_node),
                    current_node,
                    jmp_node,
                    rhs_node
                );
            } else {
                break;
            }
        }
        current_node.append_if_bytecode([PreResolvedCodePoint::Label(label)]);

        Ok((current_node, current_ast))
    }

    fn parse_relation(&mut self) -> CelResult<(CompiledProg, AstNode<Relation>)> {
        let (mut current_node, mut current_ast) = into_unary(self.parse_addition()?);

        loop {
            match self.tokenizer.peek()?.as_token() {
                Some(Token::LessThan) => {
                    self.tokenizer.next()?;

                    let (rhs_node, rhs_ast) = self.parse_addition()?;
                    let range = current_ast.range().surrounding(rhs_ast.range());

                    current_ast = AstNode::new(
                        Relation::Binary {
                            lhs: Box::new(current_ast),
                            op: Relop::Lt,
                            rhs: rhs_ast,
                        },
                        range,
                    );

                    current_node = compile!(
                        [ByteCode::Lt.into()],
                        current_node.lt(rhs_node),
                        current_node,
                        rhs_node
                    );
                }
                Some(Token::LessEqual) => {
                    self.tokenizer.next()?;
                    let (rhs_node, rhs_ast) = self.parse_addition()?;
                    let range = current_ast.range().surrounding(rhs_ast.range());

                    current_ast = AstNode::new(
                        Relation::Binary {
                            lhs: Box::new(current_ast),
                            op: Relop::Le,
                            rhs: rhs_ast,
                        },
                        range,
                    );

                    current_node = compile!(
                        [ByteCode::Le.into()],
                        current_node.le(rhs_node),
                        current_node,
                        rhs_node
                    );
                }
                Some(Token::EqualEqual) => {
                    self.tokenizer.next()?;
                    let (rhs_node, rhs_ast) = self.parse_addition()?;
                    let range = current_ast.range().surrounding(rhs_ast.range());

                    current_ast = AstNode::new(
                        Relation::Binary {
                            lhs: Box::new(current_ast),
                            op: Relop::Eq,
                            rhs: rhs_ast,
                        },
                        range,
                    );

                    current_node = compile!(
                        [ByteCode::Eq.into()],
                        CelValueDyn::eq(&current_node, &rhs_node),
                        current_node,
                        rhs_node
                    );
                }
                Some(Token::NotEqual) => {
                    self.tokenizer.next()?;
                    let (rhs_node, rhs_ast) = self.parse_addition()?;
                    let range = current_ast.range().surrounding(rhs_ast.range());

                    current_ast = AstNode::new(
                        Relation::Binary {
                            lhs: Box::new(current_ast),
                            op: Relop::Ne,
                            rhs: rhs_ast,
                        },
                        range,
                    );

                    current_node = compile!(
                        [ByteCode::Ne.into()],
                        current_node.neq(rhs_node),
                        current_node,
                        rhs_node
                    );
                }
                Some(Token::GreaterEqual) => {
                    self.tokenizer.next()?;
                    let (rhs_node, rhs_ast) = self.parse_addition()?;
                    let range = current_ast.range().surrounding(rhs_ast.range());

                    current_ast = AstNode::new(
                        Relation::Binary {
                            lhs: Box::new(current_ast),
                            op: Relop::Ge,
                            rhs: rhs_ast,
                        },
                        range,
                    );

                    current_node = compile!(
                        [ByteCode::Ge.into()],
                        current_node.ge(rhs_node),
                        current_node,
                        rhs_node
                    );
                }
                Some(Token::GreaterThan) => {
                    self.tokenizer.next()?;
                    let (rhs_node, rhs_ast) = self.parse_addition()?;
                    let range = current_ast.range().surrounding(rhs_ast.range());

                    current_ast = AstNode::new(
                        Relation::Binary {
                            lhs: Box::new(current_ast),
                            op: Relop::Gt,
                            rhs: rhs_ast,
                        },
                        range,
                    );

                    current_node = compile!(
                        [ByteCode::Gt.into()],
                        current_node.gt(rhs_node),
                        current_node,
                        rhs_node
                    );
                }
                Some(Token::In) => {
                    self.tokenizer.next()?;
                    let (rhs_node, rhs_ast) = self.parse_addition()?;
                    let range = current_ast.range().surrounding(rhs_ast.range());

                    current_ast = AstNode::new(
                        Relation::Binary {
                            lhs: Box::new(current_ast),
                            op: Relop::In,
                            rhs: rhs_ast,
                        },
                        range,
                    );
                    current_node = compile!(
                        [ByteCode::In.into()],
                        current_node.in_(rhs_node),
                        current_node,
                        rhs_node
                    )
                }
                _ => break,
            }
        }

        Ok((current_node, current_ast))
    }

    fn parse_addition(&mut self) -> CelResult<(CompiledProg, AstNode<Addition>)> {
        let (mut current_node, mut current_ast) = into_unary(self.parse_multiplication()?);

        loop {
            match self.tokenizer.peek()?.as_token() {
                Some(Token::Add) => {
                    self.tokenizer.next()?;

                    let (rhs_node, rhs_ast) = self.parse_multiplication()?;
                    let range = current_ast.range().surrounding(rhs_ast.range());

                    current_ast = AstNode::new(
                        Addition::Binary {
                            lhs: Box::new(current_ast),
                            op: AddOp::Add,
                            rhs: rhs_ast,
                        },
                        range,
                    );

                    current_node = compile!(
                        [ByteCode::Add.into()],
                        current_node + rhs_node,
                        current_node,
                        rhs_node
                    );
                }
                Some(Token::Minus) => {
                    self.tokenizer.next()?;

                    let (rhs_node, rhs_ast) = self.parse_multiplication()?;
                    let range = current_ast.range().surrounding(rhs_ast.range());

                    current_ast = AstNode::new(
                        Addition::Binary {
                            lhs: Box::new(current_ast),
                            op: AddOp::Sub,
                            rhs: rhs_ast,
                        },
                        range,
                    );

                    current_node = compile!(
                        [ByteCode::Sub.into()],
                        current_node - rhs_node,
                        current_node,
                        rhs_node
                    );
                }
                _ => break,
            }
        }

        Ok((current_node, current_ast))
    }

    fn parse_multiplication(&mut self) -> CelResult<(CompiledProg, AstNode<Multiplication>)> {
        let (mut current_node, mut current_ast) = into_unary(self.parse_unary()?);

        loop {
            match self.tokenizer.peek()?.as_token() {
                Some(Token::Multiply) => {
                    self.tokenizer.next()?;

                    let (rhs_node, rhs_ast) = self.parse_unary()?;
                    let range = current_ast.range().surrounding(rhs_ast.range());

                    current_ast = AstNode::new(
                        Multiplication::Binary {
                            lhs: Box::new(current_ast),
                            op: MultOp::Mult,
                            rhs: rhs_ast,
                        },
                        range,
                    );
                    current_node = compile!(
                        [ByteCode::Mul.into()],
                        current_node * rhs_node,
                        current_node,
                        rhs_node
                    );
                }
                Some(Token::Divide) => {
                    self.tokenizer.next()?;

                    let (rhs_node, rhs_ast) = self.parse_unary()?;
                    let range = current_ast.range().surrounding(rhs_ast.range());

                    current_ast = AstNode::new(
                        Multiplication::Binary {
                            lhs: Box::new(current_ast),
                            op: MultOp::Div,
                            rhs: rhs_ast,
                        },
                        range,
                    );

                    current_node = compile!(
                        [ByteCode::Div.into()],
                        current_node / rhs_node,
                        current_node,
                        rhs_node
                    );
                }
                Some(Token::Mod) => {
                    self.tokenizer.next()?;

                    let (rhs_node, rhs_ast) = self.parse_unary()?;
                    let range = current_ast.range().surrounding(rhs_ast.range());

                    current_ast = AstNode::new(
                        Multiplication::Binary {
                            lhs: Box::new(current_ast),
                            op: MultOp::Mod,
                            rhs: rhs_ast,
                        },
                        range,
                    );

                    current_node = compile!(
                        [ByteCode::Mod.into()],
                        current_node % rhs_node,
                        current_node,
                        rhs_node
                    );
                }
                _ => break,
            }
        }

        Ok((current_node, current_ast))
    }

    fn parse_unary(&mut self) -> CelResult<(CompiledProg, AstNode<Unary>)> {
        match self.tokenizer.peek()?.as_token() {
            Some(Token::Not) => {
                let (not, not_ast) = self.parse_not_list()?;
                let (member, member_ast) = self.parse_member()?;

                let range = not_ast.range().surrounding(member_ast.range());

                Ok((
                    member.append_result(not),
                    AstNode::new(
                        Unary::NotMember {
                            nots: not_ast,
                            member: member_ast,
                        },
                        range,
                    ),
                ))
            }
            Some(Token::Minus) => {
                let (neg, neg_ast) = self.parse_neg_list()?;
                let (member, member_ast) = self.parse_member()?;

                let range = member_ast.range().surrounding(neg_ast.range());

                Ok((
                    member.append_result(neg),
                    AstNode::new(
                        Unary::NegMember {
                            negs: neg_ast,
                            member: member_ast,
                        },
                        range,
                    ),
                ))
            }
            _ => Ok(into_unary(self.parse_member()?)),
        }
    }

    fn parse_not_list(&mut self) -> CelResult<(CompiledProg, AstNode<NotList>)> {
        match self.tokenizer.peek()? {
            Some(&TokenWithLoc {
                token: Token::Not,
                loc,
            }) => {
                self.tokenizer.next()?;

                let (not_list, ast) = self.parse_not_list()?;
                let node = compile!([ByteCode::Not.into()], not_list, not_list);

                let range = ast.range().surrounding(loc);

                Ok((
                    node,
                    AstNode::new(
                        NotList::List {
                            tail: Box::new(ast),
                        },
                        range,
                    ),
                ))
            }
            _ => {
                let start_loc = self.tokenizer.location();
                Ok((
                    CompiledProg::empty(),
                    AstNode::new(NotList::EmptyList, SourceRange::new(start_loc, start_loc)),
                ))
            }
        }
    }

    fn parse_neg_list(&mut self) -> CelResult<(CompiledProg, AstNode<NegList>)> {
        match self.tokenizer.peek()? {
            Some(&TokenWithLoc {
                token: Token::Minus,
                loc,
            }) => {
                self.tokenizer.next()?;

                let (neg_list, ast) = self.parse_neg_list()?;
                let node = compile!([ByteCode::Neg.into()], neg_list, neg_list);

                let range = ast.range().surrounding(loc);

                Ok((
                    node,
                    AstNode::new(
                        NegList::List {
                            tail: Box::new(ast),
                        },
                        range,
                    ),
                ))
            }
            _ => {
                let start_loc = self.tokenizer.location();
                Ok((
                    CompiledProg::empty(),
                    AstNode::new(NegList::EmptyList, SourceRange::new(start_loc, start_loc)),
                ))
            }
        }
    }

    fn parse_member(&mut self) -> CelResult<(CompiledProg, AstNode<Member>)> {
        let (primary_node, primary_ast) = self.parse_primary()?;

        let mut member_prime_node = CompiledProg::from_node(primary_node);
        let mut member_prime_ast: Vec<AstNode<MemberPrime>> = Vec::new();

        loop {
            match self.tokenizer.peek()? {
                Some(&TokenWithLoc {
                    token: Token::Dot,
                    loc: dot_loc,
                }) => {
                    self.tokenizer.next()?;
                    match self.tokenizer.next()? {
                        Some(TokenWithLoc {
                            token: Token::Ident(ident),
                            loc,
                        }) => {
                            let res = CompiledProg::with_const(CelValue::from_ident(&ident));

                            member_prime_node = CompiledProg::from_children2_w_bytecode_cannone(
                                member_prime_node,
                                res,
                                vec![ByteCode::Access],
                                |o, c| {
                                    if let CelValue::Ident(s) = c {
                                        // Allow for const eval for obj members in the
                                        // off chance a user does somthing like this
                                        // `{'a': 3}.a`. Its const value will be 3.
                                        if o.is_obj() {
                                            // So if this fails we should break the const
                                            // status and let the compiler generate some
                                            // bytecode for function discovery and such.
                                            match o.access(&s) {
                                                CelValue::Err(_) => None,
                                                o => Some(o),
                                            }
                                        } else {
                                            None
                                        }
                                    } else {
                                        Some(CelValue::from_err(CelError::value(
                                            "Accessor must be ident",
                                        )))
                                    }
                                },
                            );

                            member_prime_ast.push(AstNode::new(
                                MemberPrime::MemberAccess {
                                    ident: AstNode::new(Ident(ident.clone()), loc),
                                },
                                dot_loc.surrounding(loc),
                            ));
                        }
                        Some(other) => {
                            return Err(SyntaxError::from_location(self.tokenizer.location())
                                .with_message(format!("Expected IDENT got {:?}", other))
                                .into());
                        }
                        None => {
                            return Err(SyntaxError::from_location(self.tokenizer.location())
                                .with_message("Expected IDENT got NOTHING".to_string())
                                .into());
                        }
                    }
                }
                Some(&TokenWithLoc {
                    token: Token::LParen,
                    loc,
                }) => {
                    self.tokenizer.next()?;

                    let args = self.parse_expression_list(Token::RParen)?;

                    let token = self.tokenizer.next()?;
                    if let Some(TokenWithLoc {
                        token: Token::RParen,
                        loc: rparen_loc,
                    }) = token
                    {
                        let args_len = args.len();

                        let mut args_ast = Vec::new();
                        let mut args_node = CompiledProg::empty();
                        // Arguments are evaluated backwards so they get popped off the stack in order
                        for (a, ast) in args.into_iter().rev() {
                            args_ast.push(ast);
                            args_node =
                                args_node.append_result(CompiledProg::with_code_points(vec![
                                    ByteCode::Push(a.into_unresolved_bytecode().resolve().into())
                                        .into(),
                                ]))
                        }

                        member_prime_node = args_node
                            .consume_child(member_prime_node)
                            .consume_child(CompiledProg::with_code_points(vec![ByteCode::Call(
                                args_len as u32,
                            )
                            .into()]));

                        member_prime_node = self.check_for_const(member_prime_node);

                        member_prime_ast.push(AstNode::new(
                            MemberPrime::Call {
                                call: AstNode::new(
                                    ExprList { exprs: args_ast },
                                    loc.surrounding(rparen_loc),
                                ),
                            },
                            loc.surrounding(rparen_loc),
                        ));
                    } else {
                        return Err(SyntaxError::from_location(self.tokenizer.location())
                            .with_message(format!(
                                "Unexpected token {}, expected RPARAN",
                                &token.map_or("NOTHING".to_string(), |x| format!("{:?}", x))
                            ))
                            .into());
                    }
                }
                Some(&TokenWithLoc {
                    token: Token::LBracket,
                    loc,
                }) => {
                    self.tokenizer.next()?;

                    let (index_node, index_ast) = self.parse_expression()?;

                    match self.tokenizer.next()? {
                        Some(TokenWithLoc {
                            token: Token::RBracket,
                            loc: rbracket_loc,
                        }) => {
                            member_prime_node = compile!(
                                [ByteCode::Index.into()],
                                member_prime_node.index(index_node),
                                member_prime_node,
                                index_node
                            );

                            member_prime_ast.push(AstNode::new(
                                MemberPrime::ArrayAccess { access: index_ast },
                                loc.surrounding(rbracket_loc),
                            ));
                        }
                        next_token => {
                            return Err(SyntaxError::from_location(self.tokenizer.location())
                                .with_message(format!(
                                    "Unexpected token {}, expected RBRACKET",
                                    &next_token
                                        .map_or("NOTHING".to_string(), |x| format!("{:?}", x))
                                ))
                                .into());
                        }
                    }
                }
                _ => break,
            }
        }

        let mut range = primary_ast.range();
        for m in member_prime_ast.iter() {
            range = range.surrounding(m.range());
        }

        Ok((
            member_prime_node,
            AstNode::new(
                Member {
                    primary: primary_ast,
                    member: member_prime_ast,
                },
                range,
            ),
        ))
    }

    fn parse_primary(&mut self) -> CelResult<(CompiledProg, AstNode<Primary>)> {
        match self.tokenizer.next()? {
            Some(TokenWithLoc {
                token: Token::Ident(val),
                loc,
            }) => Ok((
                CompiledProg::with_code_points(vec![
                    ByteCode::Push(CelValue::from_ident(&val)).into()
                ])
                .add_ident(&val),
                AstNode::new(Primary::Ident(Ident(val.clone())), loc),
            )),
            Some(TokenWithLoc {
                token: Token::LParen,
                loc,
            }) => {
                let (expr, expr_ast) = self.parse_expression()?;

                let next_token = self.tokenizer.next();
                let rparen_loc = match next_token? {
                    Some(TokenWithLoc {
                        token: Token::RParen,
                        loc,
                    }) => loc,
                    Some(TokenWithLoc { token, loc }) => {
                        return Err(CelError::syntax(
                            SyntaxError::from_location(loc.start())
                                .with_message(format!("Expected RPAREN got {:?}", token)),
                        ))
                    }
                    None => {
                        return Err(CelError::syntax(
                            SyntaxError::from_location(loc.start())
                                .with_message("Open paren!".to_owned()),
                        ))
                    }
                };

                Ok((
                    CompiledProg::from_node(expr),
                    AstNode::new(Primary::Parens(expr_ast), loc.surrounding(rparen_loc)),
                ))
            }
            Some(TokenWithLoc {
                token: Token::LBracket,
                loc,
            }) => {
                // list construction
                let expr_node_list = self.parse_expression_list(Token::RBracket)?;
                let expr_list_len = expr_node_list.len();
                let (expr_list, expr_list_ast): (Vec<_>, Vec<_>) =
                    expr_node_list.into_iter().unzip();

                let range = if let Some(TokenWithLoc {
                    token: Token::RBracket,
                    loc: rbracket_loc,
                }) = self.tokenizer.peek()?
                {
                    loc.surrounding(*rbracket_loc)
                } else {
                    return Err(SyntaxError::from_location(self.tokenizer.location())
                        .with_message(format!("Unexpected token, expected RBRACKET",))
                        .into());
                };

                self.tokenizer.next()?;

                Ok((
                    CompiledProg::from_children_w_bytecode(
                        expr_list,
                        vec![ByteCode::MkList(expr_list_len as u32)],
                        |c| c.into(),
                    ),
                    AstNode::new(
                        Primary::ListConstruction(AstNode::new(
                            ExprList {
                                exprs: expr_list_ast,
                            },
                            range,
                        )),
                        range,
                    ),
                ))
            }
            Some(TokenWithLoc {
                token: Token::LBrace,
                loc,
            }) => {
                // Dictionary construction
                let obj_init = self.parse_obj_inits()?;

                let range = if let Some(&TokenWithLoc {
                    token: Token::RBrace,
                    loc: rbrace_loc,
                }) = self.tokenizer.peek()?
                {
                    self.tokenizer.next()?;

                    loc.surrounding(rbrace_loc)
                } else {
                    return Err(SyntaxError::from_location(self.tokenizer.location())
                        .with_message(format!("Unexpected token, expected RBRACE",))
                        .into());
                };

                let obj_init_len = obj_init.len();
                debug_assert!(obj_init_len % 2 == 0);

                let mut init_asts = Vec::new();

                let (compiled_children, children_ast): (Vec<_>, Vec<_>) =
                    obj_init.into_iter().unzip();

                let mut children_ast_iter = children_ast.into_iter();
                // init is created as value then key for mkdict stack
                while let Some(val_ast) = children_ast_iter.next() {
                    let key_ast = children_ast_iter.next().unwrap();

                    let range = key_ast.range().surrounding(val_ast.range());

                    init_asts.push(AstNode::new(
                        ObjInit {
                            key: key_ast,
                            value: val_ast,
                        },
                        range,
                    ));
                }

                let new_ast = AstNode::new(
                    Primary::ObjectInit(AstNode::new(ObjInits { inits: init_asts }, range)),
                    range,
                );

                Ok((
                    CompiledProg::from_children_w_bytecode(
                        compiled_children,
                        vec![ByteCode::MkDict(obj_init_len as u32 / 2)],
                        |vals| {
                            let mut obj_map = HashMap::new();
                            for i in (0..vals.len()).step_by(2) {
                                let key = if let CelValue::String(ref k) = vals[i + 1] {
                                    k
                                } else {
                                    return CelValue::from_err(CelError::value(
                                        "Only strings can be object keys",
                                    ));
                                };

                                obj_map.insert(key.clone(), vals[i].clone());
                            }

                            obj_map.into()
                        },
                    ),
                    new_ast,
                ))
            }
            Some(TokenWithLoc {
                token: Token::UIntLit(val),
                loc,
            }) => Ok((
                CompiledProg::with_const(val.into()),
                AstNode::new(Primary::Literal(LiteralsAndKeywords::UnsignedLit(val)), loc),
            )),
            Some(TokenWithLoc {
                token: Token::IntLit(val),
                loc,
            }) => Ok((
                CompiledProg::with_const((val as i64).into()),
                AstNode::new(
                    Primary::Literal(LiteralsAndKeywords::IntegerLit(val as i64)),
                    loc,
                ),
            )),
            Some(TokenWithLoc {
                token: Token::FloatLit(val),
                loc,
            }) => Ok((
                CompiledProg::with_const((val).into()),
                AstNode::new(Primary::Literal(LiteralsAndKeywords::FloatingLit(val)), loc),
            )),
            Some(TokenWithLoc {
                token: Token::StringLit(val),
                loc,
            }) => Ok((
                CompiledProg::with_const(val.clone().into()),
                AstNode::new(
                    Primary::Literal(LiteralsAndKeywords::StringLit(val.clone())),
                    loc,
                ),
            )),
            Some(TokenWithLoc {
                token: Token::ByteStringLit(val),
                loc,
            }) => Ok((
                CompiledProg::with_const(val.clone().into()),
                AstNode::new(
                    Primary::Literal(LiteralsAndKeywords::ByteStringLit(val.into())),
                    loc,
                ),
            )),
            Some(TokenWithLoc {
                token: Token::FStringLit(segments),
                loc,
            }) => {
                let mut bytecode = Vec::<PreResolvedCodePoint>::new();

                for segment in segments.iter() {
                    match segment {
                        FStringSegment::Lit(c) => {
                            bytecode.push(ByteCode::Push(CelValue::String(c.clone())).into())
                        }
                        FStringSegment::Expr(e) => {
                            let mut tok = StringTokenizer::with_input(&e);
                            let mut comp = CelCompiler::with_tokenizer(&mut tok);

                            let (e, _) = comp.parse_expression()?;

                            bytecode.push(
                                ByteCode::Push(CelValue::ByteCode(
                                    e.into_unresolved_bytecode().resolve(),
                                ))
                                .into(),
                            );
                        }
                    }
                    bytecode.push(ByteCode::Push(CelValue::Ident("string".to_string())).into());
                    bytecode.push(ByteCode::Call(1).into());
                }

                // Reverse it so its evaluated in order on the stack
                bytecode.push(ByteCode::FmtString(segments.len() as u32).into());

                Ok((
                    CompiledProg::with_code_points(bytecode),
                    AstNode::new(
                        Primary::Literal(LiteralsAndKeywords::FStringList(segments.clone())),
                        loc,
                    ),
                ))
            }
            Some(TokenWithLoc {
                token: Token::BoolLit(val),
                loc,
            }) => Ok((
                CompiledProg::with_const(val.into()),
                AstNode::new(Primary::Literal(LiteralsAndKeywords::BooleanLit(val)), loc),
            )),
            Some(TokenWithLoc {
                token: Token::Null,
                loc,
            }) => Ok((
                CompiledProg::with_const(CelValue::from_null()),
                AstNode::new(Primary::Literal(LiteralsAndKeywords::NullLit), loc),
            )),
            _ => Err(SyntaxError::from_location(self.tokenizer.location())
                .with_message(format!(
                    "unexpected {:?}! expecting PRIMARY",
                    self.tokenizer.peek()
                ))
                .into()),
        }
    }

    fn parse_expression_list(
        &mut self,
        ending: Token,
    ) -> CelResult<Vec<(CompiledProg, AstNode<Expr>)>> {
        let mut exprs = Vec::new();

        'outer: loop {
            match self.tokenizer.peek()?.as_token() {
                Some(val) => {
                    if *val == ending {
                        break 'outer;
                    }
                }
                None => {}
            }

            let compiled = self.parse_expression()?;
            exprs.push(compiled);

            match self.tokenizer.peek()?.as_token() {
                Some(Token::Comma) => {
                    self.tokenizer.next()?;
                    continue;
                }
                _ => break 'outer,
            }
        }

        Ok(exprs)
    }

    fn parse_obj_inits(&mut self) -> CelResult<Vec<(CompiledProg, AstNode<Expr>)>> {
        let mut inits = Vec::new();

        'outer: loop {
            if self.tokenizer.peek()?.as_token() == Some(&Token::RBrace) {
                break 'outer;
            }

            let compiled_key = self.parse_expression()?;

            let next_token = self.tokenizer.next()?.into_token();
            if next_token != Some(Token::Colon) {
                return Err(SyntaxError::from_location(self.tokenizer.location())
                    .with_message(format!("Invalid token: expected ':' got {:?}", next_token))
                    .into());
            }
            // MkDict expects value then key
            let compiled_value = self.parse_expression()?;

            inits.push(compiled_value);
            inits.push(compiled_key);

            match self.tokenizer.peek()?.as_token() {
                Some(Token::Comma) => {
                    self.tokenizer.next()?;
                    continue;
                }
                _ => break 'outer,
            }
        }

        Ok(inits)
    }

    #[inline]
    fn check_for_const(&self, member_prime_node: CompiledProg) -> CompiledProg {
        let mut i = Interpreter::empty();
        i.add_bindings(&self.bindings);
        let bc = member_prime_node.into_unresolved_bytecode().resolve();
        let r = i.run_raw(&bc, true);

        match r {
            Ok(v) => CompiledProg::with_const(v),
            Err(_) => CompiledProg::with_bytecode(bc),
        }
    }
}

#[cfg(test)]
mod test {
    use test_case::test_case;

    use crate::compiler::string_tokenizer::StringTokenizer;

    use super::CelCompiler;

    #[test_case("3+1"; "addition")]
    #[test_case("(1+foo) / 23"; "with literal")]
    #[test_case("(true || false) + 23"; "with boolean")]
    #[test_case("foo.bar"; "member access")]
    #[test_case("foo[3]"; "list access")]
    #[test_case("foo.bar()"; "member call")]
    #[test_case("foo()"; "empty function call")]
    #[test_case("foo(3)"; "function call")]
    #[test_case("1"; "just 1")]
    #[test_case("foo"; "an ident")]
    #[test_case("foo.bar.baz"; "deep member access")]
    #[test_case("--foo"; "double neg")]
    #[test_case("foo || true"; "or")]
    #[test_case("int(foo.bar && foo.baz) + 4 - (8 * 7)"; "complex")]
    #[test_case("true ? 3 : 1"; "ternary")]
    #[test_case("[1, 2, 3 + 3, 4 * 2, \"fish\"]"; "list construction")]
    fn test_parser(input: &str) {
        let mut tokenizer = StringTokenizer::with_input(input);
        CelCompiler::with_tokenizer(&mut tokenizer)
            .compile()
            .unwrap();
    }

    #[test]
    fn syntax_error() {
        let mut tokenizer = StringTokenizer::with_input("3 + 4 ) - 3");

        let e = CelCompiler::with_tokenizer(&mut tokenizer).compile();

        assert!(e.is_err());
        let _ = format!("{}", e.unwrap_err());
    }
}
