use crate::types::Lexicons;
use regex::Regex;

const PATTERNS: &'static [(Lexicons, &'static str)] = &[
    (Lexicons::Comment, r#"//[^\n\r]* \r\n|\r|\n"#),
    (Lexicons::Whitespace, r#"[\t\n\f\r ]+"#),
    (
        Lexicons::Reserved,
        r#"in|as|break|const|continue|else|for|function|if|import|let|loop|package|namespace|return|var|void|while"#,
    ),
    (Lexicons::NullLit, r#"null"#),
    (Lexicons::BoolLit, r#"true|false"#),
    (Lexicons::NewLine, r#"\r\n|\r|\n"#),
    (
        Lexicons::BytesLit,
        r#"[bB][rR]? (?:"[^"\r\n]*"|\'[^\'\r\n]*\'|""".*""")"#,
    ),
    (
        Lexicons::StringLit,
        r#"[rR]? (?:"[^"\r\n]*"|\'[^\'\r\n]*\'|""".*""")"#,
    ),
    (Lexicons::FloatLit, r#"-?[0-9]*\.[0-9]+(?:[eE][+-]?[0-9]+)"#),
    (Lexicons::UintLit, r#"(?:-?DIGIT+|-?0x[0-9a-fA-F]+)[uU]"#),
    (Lexicons::IntLit, r#"-?[0-9]+|-?0X[0-9a-fA-F]+"#),
    (Lexicons::Question, r#"\?"#),
    (Lexicons::Colon, r#":"#),
    (Lexicons::OrOp, r#"\|\|"#),
    (Lexicons::AndOp, r#"&&"#),
    (Lexicons::LeOp, r#"<="#),
    (Lexicons::GeOp, r#">="#),
    (Lexicons::EqOp, r#"=="#),
    (Lexicons::NeOp, r#"!="#),
    (Lexicons::LtOp, r#"<"#),
    (Lexicons::GtOp, r#">"#),
    (Lexicons::InOp, r#"n"#),
    (Lexicons::Period, r#"\."#),
    (Lexicons::Comma, r#","#),
    (Lexicons::LParen, r#"\("#),
    (Lexicons::RParen, r#"\)"#),
    (Lexicons::LBracket, r#"\["#),
    (Lexicons::RBracket, r#"\]"#),
    (Lexicons::LBrace, r#"\{"#),
    (Lexicons::RBrace, r#"\}"#),
    (Lexicons::AddOp, r#"\+"#),
    (Lexicons::SubOp, r#"-"#),
    (Lexicons::MulOp, r#"\*"#),
    (Lexicons::DivOp, r#"/"#),
    (Lexicons::ModOp, r#"%"#),
    (Lexicons::Ident, r#"[_a-zA-Z][_a-zA-Z0-9]*"#),
    (Lexicons::Mismatch, r#"."#),
];

pub struct Token {
    kind: Lexicons,
    value: String,
}

pub struct ParseError {
    line_no: usize,
    line_start: usize,
    expected: Lexicons,
    actual: Token,
}

pub struct Parser<'a> {
    input: &'a str,
    pos: usize,
    line_no: usize,
    line_start: usize,
    pattern: Regex,
}

impl<'a> Parser<'a> {
    pub fn with_input(input: &'a str) -> Parser<'a> {
        let full_pattern = PATTERNS
            .iter()
            .map(|x| format!("(?P<{}>{})", x.0.to_str(), x.1))
            .collect::<Vec<String>>()
            .join("|");

        Parser {
            input,
            pos: 0,
            line_no: 1,
            line_start: 0,
            pattern: Regex::new(&full_pattern).unwrap(),
        }
    }

    pub fn get_token(&mut self) -> Result<Option<Token>, ParseError> {
        match self.pattern.captures_at(self.input, self.pos) {
            Some(capture) => {
                //let kind = Lexicons::from_str(capture.name.unwrap());
                println!("{:?}", capture);
                Ok(None)
            }
            None => Ok(None),
        }
    }
}

impl Token {
    pub fn kind(&self) -> Lexicons {
        self.kind
    }

    pub fn value<'a>(&'a self) -> &'a str {
        &self.value
    }
}

#[cfg(test)]
mod tests {
    use super::{Parser, PATTERNS};
    use regex::Regex;

    #[test]
    fn patterns_compile() {
        for pattern in PATTERNS.iter() {
            Regex::new(pattern.1).unwrap();
        }
    }

    #[test]
    fn test_works() {
        let mut parser = Parser::with_input("1 + 1");

        let token = parser.get_token();
        assert!(token.is_ok());
    }
}
