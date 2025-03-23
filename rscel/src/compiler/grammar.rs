use super::{ast_node::AstNode, compiled_prog::CompiledProg, tokens::FStringSegment};
use serde::{Deserialize, Serialize};

pub trait FromUnary {
    type InputType;

    fn from_unary(inner: AstNode<<Self as FromUnary>::InputType>) -> Self;
}

#[inline]
pub fn into_unary<T, U: FromUnary<InputType = T>>(
    v: (CompiledProg, AstNode<T>),
) -> (CompiledProg, AstNode<U>) {
    let (prog, ast) = v;
    let loc = ast.range();

    (prog, AstNode::new(U::from_unary(ast), loc))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    Ternary {
        condition: Box<AstNode<ConditionalOr>>,
        true_clause: Box<AstNode<ConditionalOr>>,
        false_clause: Box<AstNode<Self>>,
    },
    Match {
        condition: Box<AstNode<Expr>>,
        cases: Vec<AstNode<MatchCase>>,
    },
    Unary(Box<AstNode<ConditionalOr>>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchCase {
    pub pattern: AstNode<MatchPattern>,
    pub expr: Box<AstNode<Expr>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MatchPattern {
    Or(AstNode<ConditionalOr>),
    Type(AstNode<MatchTypePattern>),
    Any(AstNode<MatchAnyPattern>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MatchTypePattern {
    Int,
    Uint,
    Float,
    String,
    Bool,
    Bytes,
    List,
    Object,
    Null,
    Timestamp,
    Duration,
}

impl MatchTypePattern {
    pub fn from_type_str(s: &str) -> Self {
        match s {
            "int" => MatchTypePattern::Int,
            "uint" => MatchTypePattern::Uint,
            "float" | "double" => MatchTypePattern::Float,
            "string" => MatchTypePattern::String,
            "bool" => MatchTypePattern::Bool,
            "bytes" => MatchTypePattern::Bytes,
            "list" => MatchTypePattern::List,
            "object" => MatchTypePattern::Object,
            "null" => MatchTypePattern::Null,
            "timestamp" => MatchTypePattern::Timestamp,
            "duration" => MatchTypePattern::Duration,
            _ => panic!("Unknown type"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchAnyPattern;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConditionalOr {
    Binary {
        lhs: Box<AstNode<Self>>,
        rhs: AstNode<ConditionalAnd>,
    },
    Unary(AstNode<ConditionalAnd>),
}

impl FromUnary for ConditionalOr {
    type InputType = ConditionalAnd;

    fn from_unary(inner: AstNode<<Self as FromUnary>::InputType>) -> Self {
        Self::Unary(inner)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConditionalAnd {
    Binary {
        lhs: Box<AstNode<Self>>,
        rhs: AstNode<Relation>,
    },
    Unary(AstNode<Relation>),
}

impl FromUnary for ConditionalAnd {
    type InputType = Relation;

    fn from_unary(inner: AstNode<<Self as FromUnary>::InputType>) -> Self {
        Self::Unary(inner)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Relation {
    Binary {
        lhs: Box<AstNode<Self>>,
        op: Relop,
        rhs: AstNode<Addition>,
    },
    Unary(AstNode<Addition>),
}

impl FromUnary for Relation {
    type InputType = Addition;

    fn from_unary(inner: AstNode<<Self as FromUnary>::InputType>) -> Self {
        Self::Unary(inner)
    }
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
        lhs: Box<AstNode<Self>>,
        op: AddOp,
        rhs: AstNode<Multiplication>,
    },
    Unary(AstNode<Multiplication>),
}

impl FromUnary for Addition {
    type InputType = Multiplication;

    fn from_unary(inner: AstNode<<Self as FromUnary>::InputType>) -> Self {
        Self::Unary(inner)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AddOp {
    Add,
    Sub,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Multiplication {
    Binary {
        lhs: Box<AstNode<Self>>,
        op: MultOp,
        rhs: AstNode<Unary>,
    },
    Unary(AstNode<Unary>),
}

impl FromUnary for Multiplication {
    type InputType = Unary;

    fn from_unary(inner: AstNode<<Self as FromUnary>::InputType>) -> Self {
        Self::Unary(inner)
    }
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

impl FromUnary for Unary {
    type InputType = Member;

    fn from_unary(inner: AstNode<<Self as FromUnary>::InputType>) -> Self {
        Self::Member(inner)
    }
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
    pub member: Vec<AstNode<MemberPrime>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemberPrime {
    MemberAccess { ident: AstNode<Ident> },
    Call { call: AstNode<ExprList> },
    ArrayAccess { access: AstNode<Expr> },
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
    Literal(LiteralsAndKeywords),
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
pub struct NoAst {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LiteralsAndKeywords {
    Type,

    NullType,
    Int,
    Uint,
    Float,
    Bool,
    String,
    Bytes,
    Timestamp,
    Duration,

    NullLit,
    IntegerLit(i64),
    UnsignedLit(u64),
    FloatingLit(f64),
    FStringList(Vec<FStringSegment>),
    StringLit(String),
    ByteStringLit(Vec<u8>),
    BooleanLit(bool),
}
