use std::collections::HashMap;

use super::{
    ast_node::AstNode,
    compiled_node::CompiledNode,
    grammar::*,
    source_range::SourceRange,
    syntax_error::SyntaxError,
    tokenizer::{TokenWithLoc, Tokenizer},
    tokens::{AsToken, FStringSegment, IntoToken, Token},
};
use crate::{
    interp::JmpWhen, ByteCode, CelError, CelResult, CelValue, CelValueDyn, Program, StringTokenizer,
};

use crate::compile;

pub struct CelCompiler<'l> {
    tokenizer: &'l mut dyn Tokenizer,
}

impl<'l> CelCompiler<'l> {
    pub fn with_tokenizer(tokenizer: &'l mut dyn Tokenizer) -> CelCompiler<'l> {
        CelCompiler { tokenizer }
    }

    pub fn compile(mut self) -> CelResult<Program> {
        let compiled = self.parse_expression()?;

        if !self.tokenizer.peek()?.is_none() {
            return Err(SyntaxError::from_location(self.tokenizer.location())
                .with_message(format!("Unexpected token: {:?}", self.tokenizer.peek()?))
                .into());
        }

        let prog =
            CompiledNode::<Expr>::into_program(compiled, self.tokenizer.source().to_string());

        Ok(prog)
    }

    fn parse_expression(&mut self) -> CelResult<CompiledNode<Expr>> {
        let mut lhs_node = self.parse_conditional_or()?;

        let lhs_ast = lhs_node.yank_ast();

        match self.tokenizer.peek()?.as_token() {
            Some(Token::Question) => {
                self.tokenizer.next()?;
                let mut true_clause_node = self.parse_conditional_or()?;
                let true_clause_ast = true_clause_node.yank_ast();

                let next = self.tokenizer.next()?;
                if next.as_token() != Some(&Token::Colon) {
                    return Err(SyntaxError::from_location(self.tokenizer.location())
                        .with_message(format!("Unexpected token {:?}, expected COLON", next))
                        .into());
                }

                let mut false_clause_node = self.parse_expression()?;
                let false_clause_ast = false_clause_node.yank_ast();

                let range = lhs_ast.range().surrounding(false_clause_ast.range());

                Ok(lhs_node
                    .into_turnary(true_clause_node, false_clause_node)
                    .add_ast(AstNode::new(
                        Expr::Ternary {
                            condition: Box::new(lhs_ast),
                            true_clause: Box::new(true_clause_ast),
                            false_clause: Box::new(false_clause_ast),
                        },
                        range,
                    )))
            }
            _ => {
                let range = lhs_ast.range();
                Ok(CompiledNode::from_node(lhs_node)
                    .add_ast(AstNode::new(Expr::Unary(Box::new(lhs_ast)), range)))
            }
        }
    }

    fn parse_conditional_or(&mut self) -> CelResult<CompiledNode<ConditionalOr>> {
        let mut current_node = self.parse_conditional_and()?.convert_with_ast(|lhs_ast| {
            let ast = lhs_ast.expect("Internal Error: no ast");
            let range = ast.range();

            AstNode::new(ConditionalOr::Unary(ast), range)
        });
        let mut current_ast = current_node.yank_ast();

        loop {
            if let Some(Token::OrOr) = self.tokenizer.peek()?.as_token() {
                self.tokenizer.next()?;
                let mut rhs_node = self.parse_conditional_and()?;

                let jmp_node = CompiledNode::<NoAst>::with_bytecode(vec![ByteCode::JmpCond {
                    when: JmpWhen::True,
                    dist: rhs_node.bytecode_len() as u32 + 1,
                    leave_val: true,
                }]);

                let rhs_ast = rhs_node.yank_ast();
                let range = current_ast.range().surrounding(rhs_ast.range());

                current_ast = AstNode::new(
                    ConditionalOr::Binary {
                        lhs: Box::new(current_ast),
                        rhs: rhs_ast,
                    },
                    range,
                );
                current_node = compile!(
                    [ByteCode::Or],
                    current_node.or(&rhs_node),
                    current_node,
                    jmp_node,
                    rhs_node
                );
            } else {
                break;
            }
        }

        Ok(current_node.add_ast(current_ast))
    }

    fn parse_conditional_and(&mut self) -> CelResult<CompiledNode<ConditionalAnd>> {
        let mut current_node = self.parse_relation()?.convert_with_ast(|ast_opt| {
            let ast = ast_opt.expect("Internal Error: no ast");
            let range = ast.range();

            AstNode::new(ConditionalAnd::Unary(ast), range)
        });

        let mut current_ast = current_node.yank_ast();

        loop {
            if let Some(Token::AndAnd) = self.tokenizer.peek()?.as_token() {
                self.tokenizer.next()?;
                let mut rhs_node = self.parse_relation()?;
                let jmp_node = CompiledNode::<NoAst>::with_bytecode(vec![ByteCode::JmpCond {
                    when: JmpWhen::False,
                    dist: rhs_node.bytecode_len() as u32 + 1,
                    leave_val: true,
                }]);

                let rhs_ast = rhs_node.yank_ast();
                let range = current_ast.range().surrounding(rhs_ast.range());

                current_ast = AstNode::new(
                    ConditionalAnd::Binary {
                        lhs: Box::new(current_ast),
                        rhs: rhs_ast,
                    },
                    range,
                );
                current_node = compile!(
                    [ByteCode::And],
                    current_node.and(&rhs_node),
                    current_node,
                    jmp_node,
                    rhs_node
                );
            } else {
                break;
            }
        }
        Ok(current_node.add_ast(current_ast))
    }

    fn parse_relation(&mut self) -> CelResult<CompiledNode<Relation>> {
        let mut current_node = self.parse_addition()?.convert_with_ast(|ast_opt| {
            let ast = ast_opt.expect("Internal Error: no ast");
            let range = ast.range();
            AstNode::new(Relation::Unary(ast), range)
        });
        let mut current_ast = current_node.yank_ast();

        loop {
            match self.tokenizer.peek()?.as_token() {
                Some(Token::LessThan) => {
                    self.tokenizer.next()?;

                    let mut rhs_node = self.parse_addition()?;
                    let rhs_ast = rhs_node.yank_ast();
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
                        [ByteCode::Lt],
                        current_node.lt(&rhs_node),
                        current_node,
                        rhs_node
                    );
                }
                Some(Token::LessEqual) => {
                    self.tokenizer.next()?;
                    let mut rhs_node = self.parse_addition()?;
                    let rhs_ast = rhs_node.yank_ast();
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
                        [ByteCode::Le],
                        current_node.le(&rhs_node),
                        current_node,
                        rhs_node
                    );
                }
                Some(Token::EqualEqual) => {
                    self.tokenizer.next()?;
                    let mut rhs_node = self.parse_addition()?;
                    let rhs_ast = rhs_node.yank_ast();
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
                        [ByteCode::Eq],
                        CelValueDyn::eq(&current_node, &rhs_node),
                        current_node,
                        rhs_node
                    );
                }
                Some(Token::NotEqual) => {
                    self.tokenizer.next()?;
                    let mut rhs_node = self.parse_addition()?;
                    let rhs_ast = rhs_node.yank_ast();
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
                        [ByteCode::Ne],
                        current_node.neq(&rhs_node),
                        current_node,
                        rhs_node
                    );
                }
                Some(Token::GreaterEqual) => {
                    self.tokenizer.next()?;
                    let mut rhs_node = self.parse_addition()?;
                    let rhs_ast = rhs_node.yank_ast();
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
                        [ByteCode::Ge],
                        current_node.ge(&rhs_node),
                        current_node,
                        rhs_node
                    );
                }
                Some(Token::GreaterThan) => {
                    self.tokenizer.next()?;
                    let mut rhs_node = self.parse_addition()?;
                    let rhs_ast = rhs_node.yank_ast();
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
                        [ByteCode::Gt],
                        current_node.gt(&rhs_node),
                        current_node,
                        rhs_node
                    );
                }
                Some(Token::In) => {
                    self.tokenizer.next()?;
                    let mut rhs_node = self.parse_addition()?;
                    let rhs_ast = rhs_node.yank_ast();
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
                        [ByteCode::In],
                        current_node.in_(&rhs_node),
                        current_node,
                        rhs_node
                    )
                }
                _ => break,
            }
        }

        Ok(current_node.add_ast(current_ast))
    }

    fn parse_addition(&mut self) -> CelResult<CompiledNode<Addition>> {
        let mut current_node = self.parse_multiplication()?.convert_with_ast(|ast_opt| {
            let ast = ast_opt.expect("Internal Error: no ast");
            let range = ast.range();

            AstNode::new(Addition::Unary(ast), range)
        });
        let mut current_ast = current_node.yank_ast();

        loop {
            match self.tokenizer.peek()?.as_token() {
                Some(Token::Add) => {
                    self.tokenizer.next()?;

                    let mut rhs_node = self.parse_multiplication()?;
                    let rhs_ast = rhs_node.yank_ast();
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
                        [ByteCode::Add],
                        current_node + rhs_node,
                        current_node,
                        rhs_node
                    );
                }
                Some(Token::Minus) => {
                    self.tokenizer.next()?;

                    let mut rhs_node = self.parse_multiplication()?;
                    let rhs_ast = rhs_node.yank_ast();
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
                        [ByteCode::Sub],
                        current_node - rhs_node,
                        current_node,
                        rhs_node
                    );
                }
                _ => break,
            }
        }

        Ok(current_node.add_ast(current_ast))
    }

    fn parse_multiplication(&mut self) -> CelResult<CompiledNode<Multiplication>> {
        let mut current_node = self.parse_unary()?.convert_with_ast(|ast_opt| {
            let ast = ast_opt.expect("Internal Error: no ast");
            let range = ast.range();
            AstNode::new(Multiplication::Unary(ast), range)
        });
        let mut current_ast = current_node.yank_ast();

        loop {
            match self.tokenizer.peek()?.as_token() {
                Some(Token::Multiply) => {
                    self.tokenizer.next()?;

                    let mut rhs_node = self.parse_unary()?;
                    let rhs_ast = rhs_node.yank_ast();
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
                        [ByteCode::Mul],
                        current_node * rhs_node,
                        current_node,
                        rhs_node
                    );
                }
                Some(Token::Divide) => {
                    self.tokenizer.next()?;

                    let mut rhs_node = self.parse_unary()?;
                    let rhs_ast = rhs_node.yank_ast();
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
                        [ByteCode::Div],
                        current_node / rhs_node,
                        current_node,
                        rhs_node
                    );
                }
                Some(Token::Mod) => {
                    self.tokenizer.next()?;

                    let mut rhs_node = self.parse_unary()?;
                    let rhs_ast = rhs_node.yank_ast();
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
                        [ByteCode::Mod],
                        current_node % rhs_node,
                        current_node,
                        rhs_node
                    );
                }
                _ => break,
            }
        }

        Ok(current_node.add_ast(current_ast))
    }

    fn parse_unary(&mut self) -> CelResult<CompiledNode<Unary>> {
        match self.tokenizer.peek()?.as_token() {
            Some(Token::Not) => {
                let mut not = self.parse_not_list()?;
                let not_ast = not.yank_ast();
                let mut member = self.parse_member()?;
                let member_ast = member.yank_ast();

                let range = not_ast.range().surrounding(member_ast.range());

                Ok(member.append_result(not).add_ast(AstNode::new(
                    Unary::NotMember {
                        nots: not_ast,
                        member: member_ast,
                    },
                    range,
                )))
            }
            Some(Token::Minus) => {
                let mut neg = self.parse_neg_list()?;
                let neg_ast = neg.yank_ast();
                let member = self.parse_member()?;

                Ok(member.consume_child(neg).convert_with_ast(|ast_opt| {
                    let ast = ast_opt.expect("Internal Error: no ast");
                    let range = ast.range().surrounding(neg_ast.range());
                    AstNode::new(
                        Unary::NegMember {
                            negs: neg_ast,
                            member: ast,
                        },
                        range,
                    )
                }))
            }
            _ => {
                let member = self.parse_member()?;

                Ok(member.convert_with_ast(|ast_opt| {
                    let ast = ast_opt.expect("Internal Error: no ast");
                    let range = ast.range();

                    AstNode::new(Unary::Member(ast), range)
                }))
            }
        }
    }

    fn parse_not_list(&mut self) -> CelResult<CompiledNode<NotList>> {
        match self.tokenizer.peek()? {
            Some(&TokenWithLoc {
                token: Token::Not,
                loc,
            }) => {
                self.tokenizer.next()?;

                Ok(self
                    .parse_not_list()?
                    .consume_child(CompiledNode::<NoAst>::with_bytecode(vec![ByteCode::Not]))
                    .convert_with_ast(|ast_opt| {
                        let ast = ast_opt.expect("Internal Error: no ast");
                        let range = ast.range().surrounding(loc);

                        AstNode::new(
                            NotList::List {
                                tail: Box::new(ast),
                            },
                            range,
                        )
                    }))
            }
            _ => {
                let start_loc = self.tokenizer.location();
                Ok(CompiledNode::empty().add_ast(AstNode::new(
                    NotList::EmptyList,
                    SourceRange::new(start_loc, start_loc),
                )))
            }
        }
    }

    fn parse_neg_list(&mut self) -> CelResult<CompiledNode<NegList>> {
        match self.tokenizer.peek()? {
            Some(&TokenWithLoc {
                token: Token::Minus,
                loc,
            }) => {
                self.tokenizer.next()?;
                Ok(self
                    .parse_neg_list()?
                    .consume_child(CompiledNode::<NoAst>::with_bytecode(vec![ByteCode::Neg]))
                    .convert_with_ast(|ast_opt| {
                        let ast = ast_opt.expect("Internal Error: no ast");
                        let range = ast.range().surrounding(loc);

                        AstNode::new(
                            NegList::List {
                                tail: Box::new(ast),
                            },
                            range,
                        )
                    }))
            }
            _ => {
                let start_loc = self.tokenizer.location();
                Ok(CompiledNode::empty().add_ast(AstNode::new(
                    NegList::EmptyList,
                    SourceRange::new(start_loc, start_loc),
                )))
            }
        }
    }

    fn parse_member(&mut self) -> CelResult<CompiledNode<Member>> {
        let mut primary_node = self.parse_primary()?;
        let primary_ast = primary_node.yank_ast();

        let mut member_prime_node = CompiledNode::<Member>::from_node(primary_node);
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
                            let res =
                                CompiledNode::<NoAst>::with_const(CelValue::from_ident(&ident));

                            member_prime_node = CompiledNode::from_children2_w_bytecode_cannone(
                                member_prime_node,
                                res,
                                vec![ByteCode::Access],
                                |o, c| {
                                    if let CelValue::Ident(s) = c {
                                        if o.is_obj() {
                                            Some(o.access(&s))
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
                        let mut args_node = CompiledNode::<ExprList>::empty();
                        let mut args_ast = Vec::new();
                        let args_len = args.len();

                        // Arguments are evaluated backwards so they get popped off the stack in order
                        for mut a in args.into_iter().rev() {
                            args_ast.push(a.yank_ast());
                            args_node =
                                args_node.append_result(CompiledNode::<NoAst>::with_bytecode(vec![
                                    ByteCode::Push(a.into_bytecode().into()),
                                ]))
                        }

                        member_prime_node = member_prime_node
                            .consume_child(args_node)
                            .consume_child(CompiledNode::<NoAst>::with_bytecode(vec![
                                ByteCode::Call(args_len as u32),
                            ]));
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

                    let mut index_node = self.parse_expression()?;
                    let index_ast = index_node.yank_ast();

                    match self.tokenizer.next()? {
                        Some(TokenWithLoc {
                            token: Token::RBracket,
                            loc: rbracket_loc,
                        }) => {
                            member_prime_node = CompiledNode::from_children2_w_bytecode(
                                member_prime_node,
                                index_node,
                                vec![ByteCode::Index],
                                |p, i| p.index(&i),
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

        Ok(member_prime_node.add_ast(AstNode::new(
            Member {
                primary: primary_ast,
                member: member_prime_ast,
            },
            range,
        )))
    }

    fn parse_primary(&mut self) -> CelResult<CompiledNode<Primary>> {
        match self.tokenizer.next()? {
            Some(TokenWithLoc {
                token: Token::Ident(val),
                loc,
            }) => Ok(
                CompiledNode::with_bytecode(vec![ByteCode::Push(CelValue::from_ident(&val))])
                    .add_ident(&val)
                    .add_ast(AstNode::new(Primary::Ident(Ident(val.clone())), loc)),
            ),
            Some(TokenWithLoc {
                token: Token::LParen,
                loc,
            }) => {
                let expr = self.parse_expression()?;

                // TODO: enforce a closing paran here
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

                Ok(expr.convert_with_ast(|ast| {
                    AstNode::new(
                        Primary::Parens(ast.expect("Internal Error: no ast")),
                        loc.surrounding(rparen_loc),
                    )
                }))
            }
            Some(TokenWithLoc {
                token: Token::LBracket,
                loc,
            }) => {
                // list construction
                let mut expr_list = self.parse_expression_list(Token::RBracket)?;
                let expr_list_len = expr_list.len();
                let expr_list_ast = expr_list.iter_mut().map(|e| e.yank_ast()).collect();

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

                Ok(CompiledNode::from_children_w_bytecode(
                    expr_list,
                    vec![ByteCode::MkList(expr_list_len as u32)],
                    |c| c.into(),
                )
                .add_ast(AstNode::new(
                    Primary::ListConstruction(AstNode::new(
                        ExprList {
                            exprs: expr_list_ast,
                        },
                        range,
                    )),
                    range,
                )))
            }
            Some(TokenWithLoc {
                token: Token::LBrace,
                loc,
            }) => {
                // Dictionary construction
                let mut obj_init = self.parse_obj_inits()?;
                let obj_init_len = obj_init.len();
                let mut init_asts = Vec::new();

                for i in (0..obj_init.len()).step_by(2) {
                    let key_ast = obj_init[i].yank_ast();
                    let val_ast = obj_init[i + 1].yank_ast();

                    let range = key_ast.range().surrounding(val_ast.range());

                    init_asts.push(AstNode::new(
                        ObjInit {
                            key: key_ast,
                            value: val_ast,
                        },
                        range,
                    ));
                }

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

                let new_ast = AstNode::new(
                    Primary::ObjectInit(AstNode::new(ObjInits { inits: init_asts }, range)),
                    range,
                );

                debug_assert!(obj_init_len % 2 == 0);
                Ok(CompiledNode::from_children_w_bytecode(
                    obj_init,
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
                )
                .add_ast(new_ast))
            }
            Some(TokenWithLoc {
                token: Token::UIntLit(val),
                loc,
            }) => Ok(CompiledNode::with_const(val.into()).add_ast(AstNode::new(
                Primary::Literal(LiteralsAndKeywords::UnsignedLit(val)),
                loc,
            ))),
            Some(TokenWithLoc {
                token: Token::IntLit(val),
                loc,
            }) => Ok(
                CompiledNode::with_const((val as i64).into()).add_ast(AstNode::new(
                    Primary::Literal(LiteralsAndKeywords::IntegerLit(val as i64)),
                    loc,
                )),
            ),
            Some(TokenWithLoc {
                token: Token::FloatLit(val),
                loc,
            }) => Ok(CompiledNode::with_const((val).into()).add_ast(AstNode::new(
                Primary::Literal(LiteralsAndKeywords::FloatingLit(val)),
                loc,
            ))),
            Some(TokenWithLoc {
                token: Token::StringLit(val),
                loc,
            }) => Ok(
                CompiledNode::with_const(val.clone().into()).add_ast(AstNode::new(
                    Primary::Literal(LiteralsAndKeywords::StringLit(val.clone())),
                    loc,
                )),
            ),
            Some(TokenWithLoc {
                token: Token::ByteStringLit(val),
                loc,
            }) => Ok(
                CompiledNode::with_const(val.clone().into()).add_ast(AstNode::new(
                    Primary::Literal(LiteralsAndKeywords::ByteStringLit(val.clone())),
                    loc,
                )),
            ),
            Some(TokenWithLoc {
                token: Token::FStringLit(segments),
                loc,
            }) => {
                let mut bytecode = Vec::new();

                for segment in segments.iter() {
                    bytecode.push(ByteCode::Push(CelValue::Ident("string".to_string())));
                    match segment {
                        FStringSegment::Lit(c) => {
                            bytecode.push(ByteCode::Push(CelValue::String(c.clone())))
                        }
                        FStringSegment::Expr(e) => {
                            let mut tok = StringTokenizer::with_input(&e);
                            let mut comp = CelCompiler::with_tokenizer(&mut tok);

                            let e = comp.parse_expression()?;

                            bytecode.push(ByteCode::Push(CelValue::ByteCode(e.into_bytecode())));
                        }
                    }
                    bytecode.push(ByteCode::Call(1));
                }

                // Reverse it so its evaluated in order on the stack
                bytecode.push(ByteCode::FmtString(segments.len() as u32));

                Ok(CompiledNode::with_bytecode(bytecode).add_ast(AstNode::new(
                    Primary::Literal(LiteralsAndKeywords::FStringList(segments.clone())),
                    loc,
                )))
            }
            Some(TokenWithLoc {
                token: Token::BoolLit(val),
                loc,
            }) => Ok(CompiledNode::with_const(val.into()).add_ast(AstNode::new(
                Primary::Literal(LiteralsAndKeywords::BooleanLit(val)),
                loc,
            ))),
            Some(TokenWithLoc {
                token: Token::Null,
                loc,
            }) => Ok(
                CompiledNode::with_const(CelValue::from_null()).add_ast(AstNode::new(
                    Primary::Literal(LiteralsAndKeywords::NullLit),
                    loc,
                )),
            ),
            _ => Err(SyntaxError::from_location(self.tokenizer.location())
                .with_message(format!("unexpected!!! {:?}", self.tokenizer.peek()))
                .into()),
        }
    }

    fn parse_expression_list(&mut self, ending: Token) -> CelResult<Vec<CompiledNode<Expr>>> {
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

    fn parse_obj_inits(&mut self) -> CelResult<Vec<CompiledNode<Expr>>> {
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
}
