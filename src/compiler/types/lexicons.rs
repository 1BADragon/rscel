use crate::enum_str;

enum_str! {
pub enum Lexicons {
    Mismatch,
    Ident,
    Literal,
    IntLit,
    UintLit,
    FloatLit,
    Digit,
    HexDitit,
    Exponent,
    StringLit,
    BytesLit,
    Question,
    Colon,
    OrOp,
    AndOp,
    LtOp,
    GtOp,
    LeOp,
    GeOp,
    EqOp,
    NeOp,
    InOp,
    AddOp,
    SubOp,
    MulOp,
    DivOp,
    ModOp,
    BangOp,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Period,
    Comma,
    Escape,
    NewLine,
    BoolLit,
    NullLit,
    Reserved,
    Whitespace,
    Comment,
}
}

#[cfg(test)]
mod test {
    use super::Lexicons;

    #[test]
    fn test_works() {
        let r = Lexicons::Comment;

        assert!(r.to_str() == "Comment");
        assert!(Lexicons::from_str("Comment").unwrap() == Lexicons::Comment);
    }
}
