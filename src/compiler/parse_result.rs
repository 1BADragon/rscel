use crate::{interp::JmpWhen, program::ProgramDetails, ByteCode, CelValue, CelValueDyn, Program};

use super::{ast_node::AstNode, grammar::Expr, tokenizer::SyntaxError, tokens::Token};

enum ParseConstExpr {
    CelValue(CelValue),
    ParseNode(Token),
}

enum ParseResultInner {
    Bytecode(Vec<ByteCode>),
    ConstExpr(ParseConstExpr),
}

pub struct ParseResult {
    inner: ParseResultInner,
    details: ProgramDetails,
}

impl ParseResult {
    pub fn empty() -> ParseResult {
        ParseResult {
            inner: ParseResultInner::Bytecode(Vec::new()),
            details: ProgramDetails::new(),
        }
    }

    pub fn is_const(&self) -> bool {
        match self.inner {
            ParseResultInner::Bytecode(_) => false,
            ParseResultInner::ConstExpr(_) => true,
        }
    }

    pub fn with_bytecode(bytecode: Vec<ByteCode>) -> ParseResult {
        ParseResult {
            inner: ParseResultInner::Bytecode(bytecode),
            details: ProgramDetails::new(),
        }
    }

    pub fn into_program(self, source: String, ast: AstNode<Expr>) -> Program {
        let mut details = self.details;
        details.add_source(source);
        details.add_ast(ast);

        Program::new(details, self.bytecode)
    }

    pub fn add_ident(mut self, ident: &str) -> ParseResult {
        self.details.add_param(ident);
        self
    }

    pub fn append_result(mut self, other: ParseResult) -> ParseResult {
        self.bytecode.extend(other.bytecode.into_iter());
        self.details.union_from(other.details);

        self
    }

    pub fn bytecode<'a>(&'a self) -> &'a [ByteCode] {
        &self.bytecode
    }

    pub fn consume_children(mut self, children: Vec<ParseResult>) -> ParseResult {
        let mut new_bytecode: Vec<ByteCode> = Vec::new();
        let mut new_details = self.details;

        for child in children.into_iter() {
            new_bytecode.extend(child.bytecode.into_iter());
            new_details.union_from(child.details);
        }

        new_bytecode.append(&mut self.bytecode);

        ParseResult {
            inner: ParseResultInner::Bytecode(new_bytecode),
            details: new_details,
        }
    }

    pub fn consume_call_children(mut self, children: Vec<ParseResult>) -> ParseResult {
        let mut new_bytecode: Vec<ByteCode> = Vec::new();
        let mut new_details = self.details;

        for child in children.into_iter().rev() {
            new_bytecode.push(ByteCode::Push(child.bytecode.into()));
            new_details.union_from(child.details);
        }

        new_bytecode.append(&mut self.bytecode);

        ParseResult {
            inner: ParseResultInner::Bytecode(new_bytecode),
            details: new_details,
        }
    }

    pub fn resolve(&self) -> Option<Result<CelValue, SyntaxError>> {
        if let Some(ParseResultInner::ConstExpr(c)) = self.inner {
            match c {
                ParseConstExpr::CelValue(v) => Some(Ok(v)),
                ParseConstExpr::ParseNode(n) => n.resolve(false),
            }
        }
    }

    pub fn into_turnary(
        mut self,
        true_clause: ParseResult,
        false_clause: ParseResult,
    ) -> Result<ParseResult, SyntaxError> {
        self.details.union_from(true_clause.details);
        self.details.union_from(false_clause.details);

        if self.is_const() {
            if self.resolve().unwrap()?.is_truthy() {}
        }

        self.bytecode.push(ByteCode::JmpCond {
            when: JmpWhen::False,
            dist: (true_clause.bytecode().len() as u32) + 1, // +1 to jmp over the next jump
            leave_val: false,
        });

        self.bytecode.extend(true_clause.bytecode.into_iter());
        self.bytecode
            .push(ByteCode::Jmp(false_clause.bytecode.len() as u32));
        self.bytecode.extend(false_clause.bytecode.into_iter());

        self
    }
}
