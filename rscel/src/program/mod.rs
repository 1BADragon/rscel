mod program_details;

use crate::{
    compiler::{ast_node::AstNode, grammar::Expr},
    types::CelByteCode,
    CelCompiler, CelResult, StringTokenizer,
};
pub use program_details::ProgramDetails;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Program {
    details: ProgramDetails,
    bytecode: CelByteCode,
}

impl Program {
    pub fn new(details: ProgramDetails, bytecode: CelByteCode) -> Program {
        Program { details, bytecode }
    }

    pub fn from_source(source: &str) -> CelResult<Program> {
        CelCompiler::with_tokenizer(&mut StringTokenizer::with_input(source)).compile()
    }

    pub fn params<'a>(&'a self) -> Vec<&'a str> {
        self.details.params()
    }

    pub fn source<'a>(&'a self) -> Option<&'a str> {
        self.details.source()
    }

    pub fn into_details(self) -> ProgramDetails {
        self.details
    }

    pub fn details<'a>(&'a self) -> &'a ProgramDetails {
        &self.details
    }

    pub fn details_mut<'a>(&'a mut self) -> &'a mut ProgramDetails {
        &mut self.details
    }

    pub fn bytecode<'a>(&'a self) -> &'a CelByteCode {
        &self.bytecode
    }

    pub fn dumps_bc(&self) -> String {
        let mut lines = Vec::new();

        for code in self.bytecode.iter() {
            lines.push(format!("{:?}", code))
        }

        lines.join("\n")
    }

    pub fn ast<'a>(&'a self) -> Option<&'a AstNode<Expr>> {
        self.details.ast()
    }
}

impl Clone for Program {
    fn clone(&self) -> Self {
        Program {
            details: self.details.clone(),
            bytecode: self.bytecode.clone(),
        }
    }
}
