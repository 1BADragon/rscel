use super::ast_node::AstNode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    Ternary {
        condition: Box<AstNode<ConditionalOr>>,
        true_clause: Box<AstNode<ConditionalOr>>,
        false_clause: Box<AstNode<Self>>,
    },
    Unary(Box<AstNode<ConditionalOr>>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConditionalOr {
    Binary {
        lhs: AstNode<ConditionalAnd>,
        rhs: Box<AstNode<Self>>,
    },
    Unary(AstNode<ConditionalAnd>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConditionalAnd {
    Binary {
        lhs: AstNode<Relation>,
        rhs: Box<AstNode<Self>>,
    },
    Unary(AstNode<Relation>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Relation {
    Binary {
        lhs: AstNode<Addition>,
        op: Relop,
        rhs: Box<AstNode<Self>>,
    },
    Unary(AstNode<Addition>),
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
        lhs: AstNode<Multiplication>,
        op: AddOp,
        rhs: Box<AstNode<Self>>,
    },
    Unary(AstNode<Multiplication>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AddOp {
    Add,
    Sub,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Multiplication {
    Binary {
        lhs: AstNode<Unary>,
        op: MultOp,
        rhs: Box<AstNode<Self>>,
    },
    Unary(AstNode<Unary>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MultOp {
    Mult,
    Div,
    Mod,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Unary {
    Member(AstNode<Member>),
    NotMember {
        nots: AstNode<NotList>,
        member: AstNode<Member>,
    },
    NegMember {
        negs: AstNode<NegList>,
        member: AstNode<Member>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NotList {
    List { tail: Box<AstNode<Self>> },
    EmptyList,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NegList {
    List { tail: Box<AstNode<Self>> },
    EmptyList,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Member {
    pub primary: AstNode<Primary>,
    pub member: AstNode<MemberPrime>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemberPrime {
    MemberAccess {
        ident: AstNode<Ident>,
        tail: Box<AstNode<MemberPrime>>,
    },
    Call {
        call: AstNode<ExprList>,
        tail: Box<AstNode<MemberPrime>>,
    },
    ArrayAccess {
        access: AstNode<Expr>,
        tail: Box<AstNode<MemberPrime>>,
    },
    Empty,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ident(pub String);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Primary {
    Type,
    Ident(Ident),
    Parens(AstNode<Expr>),
    ListConstruction(AstNode<ExprList>),
    ObjectInit(AstNode<ObjInits>),
    Literal(Literal),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExprList {
    pub exprs: Vec<AstNode<Expr>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjInit {
    pub key: AstNode<Expr>,
    pub value: AstNode<Expr>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjInits {
    pub inits: Vec<AstNode<ObjInit>>,
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
