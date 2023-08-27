use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    Ternary {
        condition: Box<ConditionalOr>,
        true_clause: Box<ConditionalOr>,
        false_clause: Box<Self>,
    },
    Unary(Box<ConditionalOr>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConditionalOr {
    Binary { lhs: ConditionalAnd, rhs: Box<Self> },
    Unary(ConditionalAnd),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConditionalAnd {
    Binary { lhs: Relation, rhs: Box<Self> },
    Unary(Relation),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Relation {
    Binary {
        lhs: Addition,
        op: Relop,
        rhs: Box<Self>,
    },
    Unary(Addition),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Relop {
    Le,
    Lt,
    Ge,
    Gt,
    Eq,
    Ne,
    In,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Addition {
    Binary {
        lhs: Multiplication,
        op: AddOp,
        rhs: Box<Self>,
    },
    Unary(Multiplication),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AddOp {
    Add,
    Sub,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Multiplication {
    Binary {
        lhs: Unary,
        op: MultOp,
        rhs: Box<Self>,
    },
    Unary(Unary),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MultOp {
    Mult,
    Div,
    Mod,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Unary {
    Member(Member),
    NotMember { nots: NotList, member: Member },
    NegMember { negs: NegList, member: Member },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NotList {
    List { tail: Box<Self> },
    EmptyList,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NegList {
    List { tail: Box<Self> },
    EmptyList,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Member {
    pub primary: Primary,
    pub member: MemberPrime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemberPrime {
    MemberAccess {
        ident: Ident,
        tail: Box<MemberPrime>,
    },
    Call {
        call: ExprList,
        tail: Box<MemberPrime>,
    },
    ArrayAccess {
        access: Expr,
        tail: Box<MemberPrime>,
    },
    Empty,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ident(pub String);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Primary {
    Type,
    Ident(Ident),
    Parens(Expr),
    ListConstruction(ExprList),
    ObjectInit(ObjInits),
    Literal(Literal),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExprList {
    pub exprs: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjInit {
    pub key: Expr,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjInits {
    pub inits: Vec<ObjInit>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    Null,

    Integer(i64),
    Unsigned(u64),
    Floating(f64),
    String(String),
    ByteString(Vec<u8>),
    Boolean(bool),
}
