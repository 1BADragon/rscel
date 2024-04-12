use crate::{
    interp::JmpWhen, program::ProgramDetails, ByteCode, CelResult, CelValue, CelValueDyn, Program,
};

use super::{ast_node::AstNode, grammar::Expr, tokenizer::SyntaxError, tokens::Token};

pub enum NodeValue {
    Bytecode(Vec<ByteCode>),
    ConstExpr(CelValue),
}

pub struct CompiledNode<T> {
    inner: NodeValue,
    details: ProgramDetails,
    ast: Option<AstNode<T>>,
}

pub trait CompiledNodeTrait {
    pub fn value(&self) -> &NodeValue;
    pub fn details(&self) -> &ProgramDetails;
}

impl<T> CompiledNode<T> {
    pub fn empty() -> CompiledNode<T> {
        CompiledNode {
            inner: NodeValue::Bytecode(Vec::new()),
            details: ProgramDetails::new(),
            ast: None,
        }
    }

    pub fn with_bytecode(bytecode: Vec<ByteCode>) -> CompiledNode<T> {
        CompiledNode {
            inner: NodeValue::Bytecode(bytecode),
            details: ProgramDetails::new(),
            ast: None,
        }
    }

    pub fn with_const(val: CelValue) -> CompiledNode<T> {
        CompiledNode {
            inner: NodeValue::ConstExpr(val),
            details: ProgramDetails::new(),
            ast: None,
        }
    }

    pub fn add_ast(mut self, ast: AstNode<T>) -> Self {
        self.ast = Some(ast);
        self
    }

    pub fn into_program(self, source: String) -> Program {
        let mut details = self.details;
        details.add_source(source);

        if let Some(ast) = self.ast {
            details.add_ast(ast);
        }

        Program::new(details, self.bytecode)
    }

    pub fn add_ident(mut self, ident: &str) -> CompiledNode<T> {
        self.details.add_param(ident);
        self
    }

    pub fn append_result(mut self, other: CompiledNode) -> CompiledNode<T> {
        self.bytecode.extend(other.bytecode.into_iter());
        self.details.union_from(other.details);

        self
    }

    pub fn bytecode<'a>(&'a self) -> &'a [ByteCode] {
        &self.bytecode
    }

    pub fn consume_child<O, T>(mut self, child: CompiledNode<T>) -> CompiledNode<O> {
        CompiledNode {
            inner: child.inner,
            details: child.details,
            ast: None,
        }
    }

    pub fn consume_children2<O, T1, T2>(
        mut self,
        child1: CompiledNode<T1>,
        child2: CompiledNode<T2>,
        resolve_const: &dyn Fn(CelValue, CelValue) -> CelResult<CelValue>,
    ) -> CelResult<CompiledNode<O>> {
        let mut new_details = self.details;

        new_details.union_from(child1.details());
        new_details.union_from(child2.details());

        match (child1.value(), child2.value()) {
            (NodeValue::ConstExpr(c1), NodeValue::ConstExpr(c2)) => Ok(CompiledNode {
                inner: NodeValue::ConstExpr(resolve_const(c1, c2)?),
                details: new_details,
                ast: None,
            }),
            _ => {
                let mut new_bytecode = Vec::new();

                new_bytecode.append(&mut child1.value().into_bytecode());
                new_bytecode.append(&mut child2.value().into_bytecode());
                new_bytecode.append(&mut self.value().into_bytecode());

                Ok(CompiledNode {
                    inner: NodeValue::Bytecode(new_bytecode),
                    details: new_details,
                    ast: None,
                })
            }
        }
    }

    pub fn consume_children<O>(
        mut self,
        children: Vec<Box<dyn CompiledNodeTrait>>,
        resolve_expr: Option<&dyn Fn(Vec<CelValue>) -> CelValue>,
    ) -> CelResult<CompiledNode<O>> {
        let mut new_bytecode: Vec<ByteCode> = Vec::new();
        let mut new_details = self.details;
        let mut children_are_all_const = true;

        let node_values: Vec<NodeValue> = children
            .iter()
            .map(|c| {
                new_details.union_from(c.details());
                children_are_all_const &= c.value().is_const();
                c.value()
            })
            .collect();

        if children_are_all_const {
            Ok(CompiledNode {
                inner: NodeValue::ConstExpr(resolve_expr(
                    &node_values
                        .into_iter()
                        .map(|v| match v {
                            NodeValue::Bytecode(_) => panic!("Internal error"),
                            NodeValue::ConstExpr(v) => v,
                        })
                        .collect(),
                )?),
                details: new_details,
                ast: None,
            })
        } else {
            for val in node_values.iter() {
                new_bytecode.append(&mut val.into_bytecode());
            }

            new_bytecode.append(&mut self.bytecode);

            Ok(CompiledNode {
                inner: NodeValue::Bytecode(new_bytecode),
                details: new_details,
                ast: None,
            })
        }
    }

    pub fn consume_call_children(
        mut self,
        children: Vec<Box<dyn CompiledNodeTrait>>,
    ) -> CompiledNode {
        let mut new_bytecode: Vec<ByteCode> = Vec::new();
        let mut new_details = self.details;

        for child in children.into_iter().rev() {
            new_details.union_from(child.details());
            new_bytecode.append(&mut child.value().into_bytecode())
        }

        new_bytecode.append(&mut self.bytecode);

        CompiledNode {
            inner: NodeValue::Bytecode(new_bytecode),
            details: new_details,
            ast: None,
        }
    }

    pub fn into_turnary(
        mut self,
        true_clause: CompiledNode,
        false_clause: CompiledNode,
    ) -> Result<CompiledNode, SyntaxError> {
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

        Ok(self)
    }

    pub fn ast(&self) -> AstNode<T> {
        self.ast.as_ref().expect("Internal error, no ast").clone()
    }
}

impl NodeValue {
    pub fn is_const(&self) -> bool {
        match self {
            NodeValue::Bytecode(_) => false,
            NodeValue::ConstExpr(_) => true,
        }
    }

    pub fn into_bytecode(self) -> Vec<ByteCode> {
        match self {
            NodeValue::Bytecode(b) => b,
            NodeValue::ConstExpr(c) => [ByteCode::Push(c)].to_owned(),
        }
    }
}

impl<T> CompiledNodeTrait for CompiledNode<T> {
    fn value(&self) -> &NodeValue {
        &self.inner
    }

    fn details(&self) -> &ProgramDetails {
        &self.details
    }
}
