use parsel::{
    ast::{Any, Brace, Bracket, Empty, LeftAssoc, Lit, Maybe, Paren, Punctuated},
    syn::{token::Type, Ident, Token},
    Parse, ToTokens,
};

mod kw {
    parsel::custom_keyword!(null);
}

#[derive(Debug, PartialEq, Eq, Parse, ToTokens)]
pub struct Expr {
    #[parsel(recursive)]
    pub cond_or: ConditionalOr,
    #[parsel(recursive)]
    pub ternary: Maybe<Ternary>,
}

#[derive(Debug, PartialEq, Eq, Parse, ToTokens)]
pub struct Ternary {
    #[parsel(recursive)]
    pub question: Token![?],
    #[parsel(recursive)]
    pub true_clause: ConditionalOr,
    #[parsel(recursive)]
    pub colon: Token![:],
    #[parsel(recursive)]
    pub false_clause: Box<Expr>,
}

pub type ConditionalOr = LeftAssoc<Token![||], ConditionalAnd>;
pub type ConditionalAnd = LeftAssoc<Token![&&], Relation>;
pub type Relation = LeftAssoc<Relop, Addition>;

#[derive(Debug, PartialEq, Eq, Parse, ToTokens)]
pub enum Relop {
    Lt(Token![<]),
    Le(Token![<=]),
    Ge(Token![>=]),
    Gt(Token![>]),
    Eq(Token![==]),
    Ne(Token![!=]),
    In(Token![in]),
}

pub type Addition = LeftAssoc<AddOp, Multiplication>;

#[derive(Debug, PartialEq, Eq, Parse, ToTokens)]
pub enum AddOp {
    Add(Token![+]),
    Sub(Token![-]),
}

pub type Multiplication = LeftAssoc<MultOp, Unary>;

#[derive(Debug, PartialEq, Eq, Parse, ToTokens)]
pub enum MultOp {
    Mult(Token![*]),
    Div(Token![/]),
    Mod(Token![%]),
}

#[derive(Debug, PartialEq, Eq, Parse, ToTokens)]
pub enum Unary {
    #[parsel(recursive)]
    Member(Box<Member>),
    #[parsel(recursive)]
    NotMember {
        #[parsel(recursive)]
        nots: NotList,
        #[parsel(recursive)]
        member: Box<Member>,
    },
    #[parsel(recursive)]
    NegMember {
        #[parsel(recursive)]
        negs: NegList,
        #[parsel(recursive)]
        member: Box<Member>,
    },
}

#[derive(Debug, PartialEq, Eq, Parse, ToTokens)]
pub enum NotList {
    List {
        not: Token![!],
        #[parsel(recursive)]
        tail: Box<NotList>,
    },
    EmptyList(Empty),
}

#[derive(Debug, PartialEq, Eq, Parse, ToTokens)]
pub enum NegList {
    List {
        not: Token![-],
        #[parsel(recursive)]
        tail: Box<NegList>,
    },
    EmptyList(Empty),
}

#[derive(Debug, PartialEq, Eq, Parse, ToTokens)]
pub struct Member {
    #[parsel(recursive)]
    pub primary: Primary,
    #[parsel(recursive)]
    pub member: MemberPrime,
}

#[derive(Debug, PartialEq, Eq, Parse, ToTokens)]
pub enum MemberPrime {
    #[parsel(recursive)]
    MemberAccess {
        #[parsel(recursive)]
        dot: Token![.],
        ident: Ident,
        #[parsel(recursive)]
        tail: Box<MemberPrime>,
    },
    #[parsel(recursive)]
    Call {
        #[parsel(recursive)]
        call: Paren<Maybe<ExprList>>,
        #[parsel(recursive)]
        tail: Box<MemberPrime>,
    },
    #[parsel(recursive)]
    ArrayAccess {
        #[parsel(recursive)]
        brackets: Bracket<Expr>,
        #[parsel(recursive)]
        tail: Box<MemberPrime>,
    },
    Empty(Empty),
}

#[derive(Debug, PartialEq, Eq, Parse, ToTokens)]
pub enum Primary {
    #[parsel(recursive)]
    Type(Type),
    #[parsel(recursive)]
    Ident(Ident),
    #[parsel(recursive)]
    Parens(Paren<Expr>),
    #[parsel(recursive)]
    ListConstruction(Bracket<Maybe<ExprList>>),
    #[parsel(recursive)]
    ObjectInit(Brace<Maybe<MapInits>>),
    #[parsel(recursive)]
    Literal(Literal),
}

#[derive(Debug, PartialEq, Eq, Parse, ToTokens)]
pub struct DotIdentList {
    pub dot: Token![.],
    pub ident: Ident,
}

#[derive(Debug, PartialEq, Eq, Parse, ToTokens)]
pub struct ExprList {
    #[parsel(recursive)]
    pub expr: Expr,
    #[parsel(recursive)]
    pub tail: Any<ExprListTail>,
}

#[derive(Debug, PartialEq, Eq, Parse, ToTokens)]
pub struct ExprListTail {
    pub comma: Token![,],
    #[parsel(recursive)]
    pub expr: Expr,
}

#[derive(Debug, PartialEq, Eq, Parse, ToTokens)]
pub struct FieldInit {
    #[parsel(recursive)]
    pub ident: Ident,
    #[parsel(recursive)]
    pub colon: Token![:],
    #[parsel(recursive)]
    pub expr: Expr,
}
// pub type FieldInits = Punctuated<FieldInit, Token![,]>;

#[derive(Debug, PartialEq, Eq, Parse, ToTokens)]
pub struct MapInit {
    #[parsel(recursive)]
    pub key: Expr,
    #[parsel(recursive)]
    pub colon: Token![:],
    #[parsel(recursive)]
    pub value: Expr,
}
pub type MapInits = Punctuated<MapInit, Token![,]>;

#[derive(Debug, PartialEq, Eq, Parse, ToTokens)]
pub enum Literal {
    Null(kw::null),
    Lit(Lit),
}

#[cfg(test)]
mod test {
    use super::Expr;

    use test_case::test_case;

    #[test_case("3+1"; "addition")]
    #[test_case("(1+foo) / 23"; "with literal")]
    #[test_case("(true || false) + 23"; "with boolean")]
    #[test_case("foo.bar"; "member access")]
    #[test_case("foo[3]"; "list access")]
    #[test_case("foo.bar()"; "member call")]
    #[test_case("foo()"; "empty function call")]
    #[test_case("foo(3)"; "function call")]
    #[test_case("1"; "just 1")]
    #[test_case("foo"; "an ident")]
    #[test_case("foo.bar.baz"; "deep member access")]
    #[test_case("--foo"; "double neg")]
    #[test_case("foo || true"; "or")]
    #[test_case("int(foo.bar && foo.baz) + 4 - (8 * 7)"; "complex")]
    #[test_case("true ? 3 : 1"; "ternary")]
    fn test_parser(input: &str) {
        let expr: Result<Expr, parsel::Error> = parsel::parse_str(input);

        match expr {
            Ok(_) => {}
            Err(err) => {
                let span = err.span();

                panic!(
                    "Error from column {} to column {}",
                    span.start().column,
                    span.end().column
                );
            }
        };
    }
}
