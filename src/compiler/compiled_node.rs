use crate::{
    interp::JmpWhen, program::ProgramDetails, ByteCode, CelError, CelValue, CelValueDyn, Program,
};

use super::{
    ast_node::AstNode,
    grammar::{ConditionalOr, Expr},
};

#[derive(Debug)]
pub enum NodeValue {
    Bytecode(Vec<ByteCode>),
    ConstExpr(CelValue),
}

#[derive(Debug)]
pub struct CompiledNode<T: Clone> {
    inner: NodeValue,
    details: ProgramDetails,
    ast: Option<AstNode<T>>,
}

impl<T: Clone> CompiledNode<T> {
    pub fn empty() -> CompiledNode<T> {
        CompiledNode {
            inner: NodeValue::Bytecode(Vec::new()),
            details: ProgramDetails::new(),
            ast: None,
        }
    }

    pub fn from_node<I: Clone>(other: CompiledNode<I>) -> Self {
        CompiledNode {
            inner: other.inner,
            details: other.details,
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

    pub fn from_children_w_bytecode<T1: Clone, F>(
        children: Vec<CompiledNode<T1>>,
        bytecode: Vec<ByteCode>,
        resolve: F,
    ) -> CompiledNode<T>
    where
        F: FnOnce(Vec<CelValue>) -> CelValue,
    {
        let mut all_const = true;
        let mut details = ProgramDetails::new();

        for c in children.iter() {
            all_const &= c.is_const();
            details.union_from(c.details.clone());
        }

        let inner = if all_const {
            NodeValue::ConstExpr(resolve(
                children.into_iter().map(|c| c.const_val()).collect(),
            ))
        } else {
            NodeValue::Bytecode(
                children
                    .into_iter()
                    .map(|c| c.inner.into_bytecode())
                    .flatten()
                    .chain(bytecode.into_iter())
                    .collect(),
            )
        };

        CompiledNode {
            inner,
            details,
            ast: None,
        }
    }

    pub fn from_children2_w_bytecode<T1: Clone, T2: Clone, F>(
        child1: CompiledNode<T1>,
        child2: CompiledNode<T2>,
        bytecode: Vec<ByteCode>,
        resolve: F,
    ) -> CompiledNode<T>
    where
        F: FnOnce(CelValue, CelValue) -> CelValue,
    {
        let new_details = ProgramDetails::joined2(child1.details, child2.details);

        match (child1.inner, child2.inner) {
            (NodeValue::ConstExpr(c1), NodeValue::ConstExpr(c2)) => CompiledNode {
                inner: NodeValue::ConstExpr(resolve(c1, c2)),
                details: new_details,
                ast: None,
            },
            (c1, c2) => CompiledNode {
                inner: NodeValue::Bytecode(
                    c1.into_bytecode()
                        .into_iter()
                        .chain(c2.into_bytecode().into_iter())
                        .chain(bytecode.into_iter())
                        .collect(),
                ),
                details: new_details,
                ast: None,
            },
        }
    }

    pub fn from_children2_w_bytecode_cannone<T1: Clone, T2: Clone, F>(
        child1: CompiledNode<T1>,
        child2: CompiledNode<T2>,
        bytecode: Vec<ByteCode>,
        resolve: F,
    ) -> CompiledNode<T>
    where
        F: FnOnce(&CelValue, &CelValue) -> Option<CelValue>,
    {
        let new_details = ProgramDetails::joined2(child1.details, child2.details);

        match (child1.inner, child2.inner) {
            (NodeValue::ConstExpr(c1), NodeValue::ConstExpr(c2)) => match resolve(&c1, &c2) {
                Some(res) => CompiledNode {
                    inner: NodeValue::ConstExpr(res),
                    details: new_details,
                    ast: None,
                },
                None => CompiledNode {
                    inner: NodeValue::Bytecode(
                        [ByteCode::Push(c1), ByteCode::Push(c2)]
                            .into_iter()
                            .chain(bytecode.into_iter())
                            .collect(),
                    ),
                    details: new_details,
                    ast: None,
                },
            },
            (c1, c2) => CompiledNode {
                inner: NodeValue::Bytecode(
                    c1.into_bytecode()
                        .into_iter()
                        .chain(c2.into_bytecode().into_iter())
                        .chain(bytecode.into_iter())
                        .collect(),
                ),
                details: new_details,
                ast: None,
            },
        }
    }

    pub fn convert_with_ast<F, O: Clone>(self, ast_builder: F) -> CompiledNode<O>
    where
        F: FnOnce(Option<AstNode<T>>) -> AstNode<O>,
    {
        CompiledNode {
            inner: self.inner,
            details: self.details,
            ast: Some(ast_builder(self.ast)),
        }
    }

    pub fn add_ast(mut self, ast: AstNode<T>) -> Self {
        self.ast = Some(ast);
        self
    }

    pub fn into_program(node: CompiledNode<Expr>, source: String) -> Program {
        let mut details = node.details;
        details.add_source(source);

        if let Some(ast) = node.ast {
            details.add_ast(ast);
        }

        Program::new(details, node.inner.into_bytecode())
    }

    pub fn add_ident(mut self, ident: &str) -> CompiledNode<T> {
        self.details.add_param(ident);
        self
    }

    pub fn append_result<T1: Clone, O: Clone>(self, other: CompiledNode<T1>) -> CompiledNode<O> {
        let my_bytecode = self.inner.into_bytecode();
        let other_bytecode = other.inner.into_bytecode();

        let mut new_details = self.details;
        new_details.union_from(other.details);

        let new_bytecode = my_bytecode
            .into_iter()
            .chain(other_bytecode.into_iter())
            .collect();

        CompiledNode {
            inner: NodeValue::Bytecode(new_bytecode),
            details: new_details,
            ast: None,
        }
    }

    pub fn consume_child<T1: Clone>(mut self, child: CompiledNode<T1>) -> CompiledNode<T> {
        let mut ast = None;
        std::mem::swap(&mut ast, &mut self.ast);

        let mut r = self.append_result(child);

        r.ast = ast;
        r
    }

    pub fn consume_children2<T1: Clone, T2: Clone, F>(
        self,
        child1: CompiledNode<T1>,
        child2: CompiledNode<T2>,
        resolve_const: F,
    ) -> CompiledNode<T>
    where
        F: FnOnce(CelValue, CelValue) -> CelValue,
    {
        let mut new_details = self.details;

        new_details.union_from(child1.details);
        new_details.union_from(child2.details);

        match (child1.inner, child2.inner) {
            (NodeValue::ConstExpr(c1), NodeValue::ConstExpr(c2)) => CompiledNode {
                inner: NodeValue::ConstExpr(resolve_const(c1, c2)),
                details: new_details,
                ast: None,
            },
            (c1, c2) => {
                let mut new_bytecode = Vec::new();

                new_bytecode.append(&mut c1.into_bytecode());
                new_bytecode.append(&mut c2.into_bytecode());
                new_bytecode.append(&mut self.inner.into_bytecode());

                CompiledNode {
                    inner: NodeValue::Bytecode(new_bytecode),
                    details: new_details,
                    ast: None,
                }
            }
        }
    }

    pub fn consume_children3<T1: Clone, T2: Clone, T3: Clone, F>(
        self,
        lhs: CompiledNode<T1>,
        jmp: CompiledNode<T2>,
        rhs: CompiledNode<T3>,
        resolve_const: F,
    ) -> CompiledNode<T>
    where
        F: FnOnce(CelValue, CelValue) -> CelValue,
    {
        let mut new_details = self.details;

        new_details.union_from(lhs.details);
        new_details.union_from(jmp.details);
        new_details.union_from(rhs.details);

        match (lhs.inner, rhs.inner) {
            (NodeValue::ConstExpr(c1), NodeValue::ConstExpr(c2)) => CompiledNode {
                inner: NodeValue::ConstExpr(resolve_const(c1, c2)),
                details: new_details,
                ast: None,
            },
            (li, ri) => {
                let mut new_bytecode = Vec::new();

                new_bytecode.append(&mut li.into_bytecode());
                new_bytecode.append(&mut jmp.inner.into_bytecode());
                new_bytecode.append(&mut ri.into_bytecode());
                new_bytecode.append(&mut self.inner.into_bytecode());

                CompiledNode {
                    inner: NodeValue::Bytecode(new_bytecode),
                    details: new_details,
                    ast: None,
                }
            }
        }
    }

    pub fn into_turnary(
        mut self,
        true_clause: CompiledNode<ConditionalOr>,
        false_clause: CompiledNode<Expr>,
    ) -> CompiledNode<Expr> {
        self.details.union_from(true_clause.details);
        self.details.union_from(false_clause.details);

        if let NodeValue::ConstExpr(i) = self.inner {
            if cfg!(feature = "type_prop") {
                if i.is_truthy() {
                    CompiledNode {
                        inner: true_clause.inner,
                        details: self.details,
                        ast: None,
                    }
                } else {
                    CompiledNode {
                        inner: false_clause.inner,
                        details: self.details,
                        ast: None,
                    }
                }
            } else {
                if let CelValue::Bool(b) = i {
                    if b {
                        CompiledNode {
                            inner: true_clause.inner,
                            details: self.details,
                            ast: None,
                        }
                    } else {
                        CompiledNode {
                            inner: false_clause.inner,
                            details: self.details,
                            ast: None,
                        }
                    }
                } else {
                    CompiledNode {
                        inner: NodeValue::ConstExpr(CelValue::from_err(CelError::Value(format!(
                            "{} cannot be converted to bool",
                            i.as_type()
                        )))),
                        details: self.details,
                        ast: None,
                    }
                }
            }
        } else {
            let true_clause_bytecode = true_clause.inner.into_bytecode();
            let false_clause_bytecode = false_clause.inner.into_bytecode();
            CompiledNode {
                inner: NodeValue::Bytecode(
                    self.inner
                        .into_bytecode()
                        .into_iter()
                        .chain(
                            [ByteCode::JmpCond {
                                when: JmpWhen::False,
                                dist: (true_clause_bytecode.len() as u32) + 1, // +1 to jmp over the next jump
                                leave_val: false,
                            }]
                            .into_iter(),
                        )
                        .chain(true_clause_bytecode.into_iter())
                        .chain([ByteCode::Jmp(false_clause_bytecode.len() as u32)].into_iter())
                        .chain(false_clause_bytecode.into_iter())
                        .collect(),
                ),
                details: self.details,
                ast: None,
            }
        }
    }

    #[inline]
    pub fn yank_ast(&mut self) -> AstNode<T> {
        let mut moved_ast = None;
        std::mem::swap(&mut moved_ast, &mut self.ast);

        moved_ast.expect("Internal error, no ast")
    }

    #[inline]
    pub fn bytecode_len(&self) -> usize {
        match self.inner {
            NodeValue::Bytecode(ref b) => b.len(),
            NodeValue::ConstExpr(_) => 1,
        }
    }

    pub fn into_bytecode(self) -> Vec<ByteCode> {
        self.inner.into_bytecode()
    }

    pub fn is_const(&self) -> bool {
        self.inner.is_const()
    }

    pub fn const_val(self) -> CelValue {
        match self.inner {
            NodeValue::ConstExpr(c) => c,
            _ => panic!("Internal Error: not const"),
        }
    }
}

impl NodeValue {
    fn is_const(&self) -> bool {
        match self {
            NodeValue::ConstExpr(_) => true,
            NodeValue::Bytecode(_) => false,
        }
    }

    fn into_bytecode(self) -> Vec<ByteCode> {
        match self {
            NodeValue::Bytecode(b) => b,
            NodeValue::ConstExpr(c) => [ByteCode::Push(c)].to_vec(),
        }
    }
}
