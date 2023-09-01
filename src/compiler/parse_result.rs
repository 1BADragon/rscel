use crate::{interp::JmpWhen, program::ProgramDetails, ByteCode, Program};

use super::{ast_node::AstNode, grammar::Expr};

pub struct ParseResult {
    bytecode: Vec<ByteCode>,
    details: ProgramDetails,
}

impl ParseResult {
    pub fn new() -> ParseResult {
        ParseResult {
            bytecode: Vec::new(),
            details: ProgramDetails::new(),
        }
    }

    pub fn with_bytecode(bytecode: Vec<ByteCode>) -> ParseResult {
        ParseResult {
            bytecode,
            details: ProgramDetails::new(),
        }
    }

    pub fn into_program(self, source: String, ast: AstNode<Expr>) -> Program {
        Program::new(source, self.details, self.bytecode, ast)
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
            bytecode: new_bytecode,
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
            bytecode: new_bytecode,
            details: new_details,
        }
    }

    pub fn into_turnary(
        mut self,
        true_clause: ParseResult,
        false_clause: ParseResult,
    ) -> ParseResult {
        self.bytecode.push(ByteCode::JmpCond {
            when: JmpWhen::False,
            dist: (true_clause.bytecode().len() as u32) + 1, // +1 to jmp over the next jump
            leave_val: false,
        });

        self.details.union_from(true_clause.details);
        self.details.union_from(false_clause.details);

        self.bytecode.extend(true_clause.bytecode.into_iter());
        self.bytecode
            .push(ByteCode::Jmp(false_clause.bytecode.len() as u32));
        self.bytecode.extend(false_clause.bytecode.into_iter());

        self
    }
}
