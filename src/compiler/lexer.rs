use std::{fmt, iter, str::FromStr};

use super::types::Lexicons;
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
        r#"[bB][rR]?(?:"[^"\r\n]*"|\'[^\'\r\n]*\'|""".*""")"#,
    ),
    (
        Lexicons::StringLit,
        r#"[rR]?(?:"[^"\r\n]*"|\'[^\'\r\n]*\'|""".*""")"#,
    ),
    (Lexicons::FloatLit, r#"-?[0-9]*\.[0-9]+(?:[eE][+-]?[0-9]+)"#),
    (Lexicons::UintLit, r#"(?:-?[0-9]+|-?0x[0-9a-fA-F]+)[uU]"#),
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
    (Lexicons::BangOp, r#"!"#),
    (Lexicons::Ident, r#"[_a-zA-Z][_a-zA-Z0-9]*"#),
    (Lexicons::Mismatch, r#"."#),
];

#[derive(Debug)]
pub struct Token {
    kind: Lexicons,
    value: String,
}

#[derive(Debug)]
pub struct ParseError {
    line_no: usize,
    line_start: usize,
    token: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Parse Error on line {} column {}, \"{}\"",
            self.line_no, self.line_start, self.token
        )
    }
}

#[derive(Debug, Clone)]
pub struct Parser<'a> {
    input: &'a str,
    pos: usize,
    line_no: usize,
    line_start: usize,
    pattern: Regex,
}

pub struct ParserIterator<'a> {
    parser: &'a mut Parser<'a>,
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
        'outer: while self.pos < self.input.len() {
            match self.pattern.captures_at(self.input, self.pos) {
                Some(capture) => {
                    for pattern in PATTERNS.iter() {
                        if let Some(group) = capture.name(pattern.0.to_str()) {
                            self.pos += group.as_str().len();
                            self.line_start += group.as_str().len();
                            match pattern.0 {
                                Lexicons::Comment => continue 'outer,
                                Lexicons::NewLine => {
                                    self.line_no += 1;
                                    self.line_start = 0;
                                }
                                Lexicons::Whitespace => continue 'outer,
                                Lexicons::Mismatch => {
                                    return Err(ParseError {
                                        line_no: self.line_no,
                                        line_start: self.line_start,
                                        token: String::from_str(group.as_str()).unwrap(),
                                    })
                                }
                                _ => {
                                    return Ok(Some(Token {
                                        kind: pattern.0,
                                        value: String::from_str(group.as_str()).unwrap(),
                                    }))
                                }
                            }
                        }
                    }
                }
                None => return Ok(None),
            }
        }
        Ok(None)
    }

    pub fn iter(&'a mut self) -> ParserIterator<'a> {
        ParserIterator { parser: self }
    }
}

impl<'a> iter::Iterator for ParserIterator<'a> {
    type Item = Result<Token, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.parser.get_token() {
            Ok(value_opt) => match value_opt {
                Some(value) => return Some(Ok(value)),
                None => return None,
            },
            Err(err) => return Some(Err(err)),
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
    use super::super::types::Lexicons;
    use super::super::types::Lexicons::*;
    use test_case::test_case;

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

        let tokens = parser
            .iter()
            .map(|x| x.unwrap().kind())
            .collect::<Vec<Lexicons>>();
        assert!(tokens == vec![IntLit, AddOp, IntLit]);
    }

    #[test_case("-1", IntLit ; "int lit negative int lit")]
    #[test_case("1u", UintLit)]
    #[test_case("1", IntLit)]
    #[test_case(r#""""hello""""#, StringLit)]
    #[test_case("\"hello\"", StringLit)]
    #[test_case("'hello'", StringLit; "stringlit single quote")]
    fn assert_patterns(input: &str, kind: Lexicons) {
        let mut parser = Parser::with_input(input);

        let token = parser.get_token().unwrap().unwrap();
        assert!(token.kind() == kind);
        assert!(token.value() == input);
    }
}
