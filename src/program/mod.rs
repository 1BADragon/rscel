//mod compile;
//mod program_cache;
mod program_details;

use crate::{
    compiler::{ast_node::AstNode, grammar::Expr},
    interp::ByteCode,
};
//use compile::ProgramCompiler;
pub use program_details::ProgramDetails;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Program {
    source: String,
    details: ProgramDetails,

    bytecode: Vec<ByteCode>,
    ast: AstNode<Expr>,
}

impl Program {
    pub fn new(
        source: String,
        details: ProgramDetails,
        bytecode: Vec<ByteCode>,
        ast: AstNode<Expr>,
    ) -> Program {
        Program {
            source,
            details,
            bytecode,
            ast,
        }
    }

    pub fn params<'a>(&'a self) -> Vec<&'a str> {
        self.details.params()
    }

    pub fn source<'a>(&'a self) -> &'a str {
        &self.source
    }

    pub fn details<'a>(&'a self) -> &'a ProgramDetails {
        &self.details
    }

    pub fn details_mut<'a>(&'a mut self) -> &'a mut ProgramDetails {
        &mut self.details
    }

    pub fn bytecode<'a>(&'a self) -> &'a [ByteCode] {
        &self.bytecode
    }

    pub fn dumps_bc(&self) -> String {
        let mut lines = Vec::new();

        for code in self.bytecode.iter() {
            lines.push(format!("{:?}", code))
        }

        lines.join("\n")
    }
}

impl Clone for Program {
    fn clone(&self) -> Self {
        Program {
            source: self.source.clone(),
            details: self.details.clone(),
            bytecode: self.bytecode.clone(),
            ast: self.ast.clone(),
        }
    }
}
