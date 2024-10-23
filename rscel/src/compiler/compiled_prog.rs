use crate::{
    interp::JmpWhen, program::ProgramDetails, ByteCode, CelError, CelValue, CelValueDyn, Program,
};

#[derive(Debug, Clone)]
pub enum NodeValue {
    Bytecode(Vec<ByteCode>),
    ConstExpr(CelValue),
}

#[derive(Debug, Clone)]
pub struct CompiledProg {
    pub inner: NodeValue,
    pub details: ProgramDetails,
}

#[macro_export]
macro_rules! compile {
    ($bytecode:expr, $const_expr:expr, $( $child : ident),+) => {
        {
            use crate::compiler::compiled_prog::NodeValue;
            use crate::program::ProgramDetails;

            let mut new_details = ProgramDetails::new();

            $(
                new_details.union_from($child.details);
            )+

            match ($($child.inner,)+) {
                #[allow(unused_variables)]
                ($(NodeValue::ConstExpr($child),)+) => {
                    let resolved_const = $const_expr;

                    CompiledProg {
                        inner: NodeValue::ConstExpr(resolved_const),
                        details: new_details,
                    }
                }
                ($($child,)+) => {
                let mut new_bytecode = Vec::new();

                $(
                    new_bytecode.append(&mut $child.into_bytecode());
                )+

                new_bytecode.extend_from_slice(&$bytecode);

                CompiledProg {
                    inner: NodeValue::Bytecode(new_bytecode),
                    details: new_details,
                }

                }

            }
        }
    };
}

impl CompiledProg {
    pub fn empty() -> CompiledProg {
        CompiledProg {
            inner: NodeValue::Bytecode(Vec::new()),
            details: ProgramDetails::new(),
        }
    }

    pub fn from_node(other: CompiledProg) -> Self {
        CompiledProg {
            inner: other.inner,
            details: other.details,
        }
    }

    pub fn with_bytecode(bytecode: Vec<ByteCode>) -> CompiledProg {
        CompiledProg {
            inner: NodeValue::Bytecode(bytecode),
            details: ProgramDetails::new(),
        }
    }

    pub fn with_const(val: CelValue) -> CompiledProg {
        CompiledProg {
            inner: NodeValue::ConstExpr(val),
            details: ProgramDetails::new(),
        }
    }

    pub fn from_children_w_bytecode<F>(
        children: Vec<CompiledProg>,
        bytecode: Vec<ByteCode>,
        resolve: F,
    ) -> CompiledProg
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

        CompiledProg { inner, details }
    }

    pub fn from_children2_w_bytecode_cannone<F>(
        child1: CompiledProg,
        child2: CompiledProg,
        bytecode: Vec<ByteCode>,
        resolve: F,
    ) -> CompiledProg
    where
        F: FnOnce(&CelValue, &CelValue) -> Option<CelValue>,
    {
        let new_details = ProgramDetails::joined2(child1.details, child2.details);

        match (child1.inner, child2.inner) {
            (NodeValue::ConstExpr(c1), NodeValue::ConstExpr(c2)) => match resolve(&c1, &c2) {
                Some(res) => CompiledProg {
                    inner: NodeValue::ConstExpr(res),
                    details: new_details,
                },
                None => CompiledProg {
                    inner: NodeValue::Bytecode(
                        [ByteCode::Push(c1), ByteCode::Push(c2)]
                            .into_iter()
                            .chain(bytecode.into_iter())
                            .collect(),
                    ),
                    details: new_details,
                },
            },
            (c1, c2) => CompiledProg {
                inner: NodeValue::Bytecode(
                    c1.into_bytecode()
                        .into_iter()
                        .chain(c2.into_bytecode().into_iter())
                        .chain(bytecode.into_iter())
                        .collect(),
                ),
                details: new_details,
            },
        }
    }

    pub fn into_program(self, source: String) -> Program {
        let mut details = self.details;
        details.add_source(source);

        Program::new(details, self.inner.into_bytecode())
    }

    pub fn add_ident(mut self, ident: &str) -> CompiledProg {
        self.details.add_param(ident);
        self
    }

    pub fn append_result(self, other: CompiledProg) -> CompiledProg {
        let my_bytecode = self.inner.into_bytecode();
        let other_bytecode = other.inner.into_bytecode();

        let mut new_details = self.details;
        new_details.union_from(other.details);

        let new_bytecode = my_bytecode
            .into_iter()
            .chain(other_bytecode.into_iter())
            .collect();

        CompiledProg {
            inner: NodeValue::Bytecode(new_bytecode),
            details: new_details,
        }
    }

    pub fn consume_child(self, child: CompiledProg) -> CompiledProg {
        let r = self.append_result(child);
        r
    }

    pub fn into_turnary(
        mut self,
        true_clause: CompiledProg,
        false_clause: CompiledProg,
    ) -> CompiledProg {
        self.details.union_from(true_clause.details);
        self.details.union_from(false_clause.details);

        if let NodeValue::ConstExpr(i) = self.inner {
            if i.is_err() {
                CompiledProg {
                    inner: NodeValue::ConstExpr(i),
                    details: self.details,
                }
            } else {
                if cfg!(feature = "type_prop") {
                    if i.is_truthy() {
                        CompiledProg {
                            inner: true_clause.inner,
                            details: self.details,
                        }
                    } else {
                        CompiledProg {
                            inner: false_clause.inner,
                            details: self.details,
                        }
                    }
                } else {
                    if let CelValue::Bool(b) = i {
                        if b {
                            CompiledProg {
                                inner: true_clause.inner,
                                details: self.details,
                            }
                        } else {
                            CompiledProg {
                                inner: false_clause.inner,
                                details: self.details,
                            }
                        }
                    } else {
                        CompiledProg {
                            inner: NodeValue::ConstExpr(CelValue::from_err(CelError::Value(
                                format!("{} cannot be converted to bool", i.as_type()),
                            ))),
                            details: self.details,
                        }
                    }
                }
            }
        } else {
            let true_clause_bytecode = true_clause.inner.into_bytecode();
            let false_clause_bytecode = false_clause.inner.into_bytecode();
            CompiledProg {
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
            }
        }
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
    pub fn is_const(&self) -> bool {
        matches!(*self, NodeValue::ConstExpr(_))
    }

    pub fn into_bytecode(self) -> Vec<ByteCode> {
        match self {
            NodeValue::Bytecode(b) => b,
            NodeValue::ConstExpr(c) => [ByteCode::Push(c)].to_vec(),
        }
    }
}
