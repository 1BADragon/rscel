#[derive(Debug, PartialEq)]
pub enum Expr {
    Ternary {
        condition: Box<ConditionalOr>,
        true_clause: Box<ConditionalOr>,
        false_clause: Box<Self>,
    },
    Unary(Box<ConditionalOr>),
}

#[derive(Debug, PartialEq)]
pub enum ConditionalOr {
    Binary { lhs: ConditionalAnd, rhs: Box<Self> },
    Unary(ConditionalAnd),
}

#[derive(Debug, PartialEq)]
pub enum ConditionalAnd {
    Binary { lhs: Relation, rhs: Box<Self> },
    Unary(Relation),
}

#[derive(Debug, PartialEq)]
pub enum Relation {
    Binary {
        lhs: Addition,
        op: Relop,
        rhs: Box<Self>,
    },
    Unary(Addition),
}

#[derive(Debug, PartialEq)]
pub enum Relop {
    Le,
    Lt,
    Ge,
    Gt,
    Eq,
    Ne,
    In,
}

#[derive(Debug, PartialEq)]
pub enum Addition {
    Binary {
        lhs: Multiplication,
        op: AddOp,
        rhs: Box<Self>,
    },
    Unary(Multiplication),
}

#[derive(Debug, PartialEq)]
pub enum AddOp {
    Add,
    Sub,
}

#[derive(Debug, PartialEq)]
pub enum Multiplication {
    Binary {
        lhs: Unary,
        op: MultOp,
        rhs: Box<Self>,
    },
    Unary(Unary),
}

#[derive(Debug, PartialEq)]
pub enum MultOp {
    Mult,
    Div,
    Mod,
}

#[derive(Debug, PartialEq)]
pub enum Unary {
    Member(Member),
    NotMember { nots: NotList, member: Member },
    NegMember { negs: NegList, member: Member },
}

#[derive(Debug, PartialEq)]
pub enum NotList {
    List { tail: Box<Self> },
    EmptyList,
}

#[derive(Debug, PartialEq)]
pub enum NegList {
    List { tail: Box<Self> },
    EmptyList,
}

#[derive(Debug, PartialEq)]
pub struct Member {
    pub primary: Primary,
    pub member: MemberPrime,
}

#[derive(Debug, PartialEq)]
pub enum MemberPrime {
    MemberAccess {
        ident: Ident,
        tail: Box<MemberPrime>,
    },
    Call {
        call: Option<ExprList>,
        tail: Box<MemberPrime>,
    },
    ArrayAccess {
        access: Expr,
        tail: Box<MemberPrime>,
    },
    Empty,
}

#[derive(Debug, PartialEq)]
struct Ident(String);

#[derive(Debug, PartialEq)]
pub enum Primary {
    Type,
    Ident(Ident),
    Parens(Expr),
    ListConstruction(Option<ExprList>),
    ObjectInit(Option<MapInits>),
    Literal(Literal),
}

#[derive(Debug, PartialEq)]
pub struct ExprList {
    pub expr: Vec<Expr>,
}

#[derive(Debug, PartialEq)]
pub struct MapInit {
    pub key: Expr,
    pub value: Expr,
}

#[derive(Debug, PartialEq)]
pub struct MapInits {
    inits: Vec<MapInit>,
}

#[derive(Debug, PartialEq)]
pub enum Literal {
    Null,

    Integer(i64),
    Unsigned(u64),
    Floating(f64),
    String(String),
    Boolean(bool),
}
