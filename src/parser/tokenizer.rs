use super::{input_scanner::InputScanner, syntax_error::SyntaxError, tokens::Token};

pub struct Tokenizer<'l> {
    scanner: InputScanner<'l>,

    current: Option<Token>,
}

impl<'l> Tokenizer<'l> {
    pub fn with_input(input: &'l str) -> Tokenizer<'l> {
        Tokenizer {
            scanner: InputScanner::from_input(input),
            current: None,
        }
    }

    pub fn peek(&mut self) -> Result<Option<Token>, SyntaxError> {
        if let None = self.current {
            match self.collect_next_token() {
                Ok(token) => self.current = token,
                Err(err) => return Err(err),
            };
        }
        Ok(self.current.clone())
    }

    pub fn next(&mut self) -> Result<Option<Token>, SyntaxError> {
        if let None = self.current {
            self.collect_next_token()
        } else {
            let tmp = std::mem::replace(&mut self.current, None);
            Ok(tmp)
        }
    }

    fn collect_next_token(&mut self) -> Result<Option<Token>, SyntaxError> {
        let start_char = self.scanner.next();
        let mut tmp = [0; 4];

        if let Some(input_char) = start_char {
            match input_char {
                '?' => Ok(Some(Token::Question)),
                ':' => Ok(Some(Token::Colon)),
                '+' => Ok(Some(Token::Add)),
                '-' => Ok(Some(Token::Minus)),
                '*' => Ok(Some(Token::Multiply)),
                '/' => Ok(Some(Token::Divide)),
                '%' => Ok(Some(Token::Mod)),
                '!' => match self.scanner.peek() {
                    Some('=') => Ok(Some(Token::NotEqual)),
                    _ => Ok(Some(Token::Not)),
                },
                '.' => Ok(Some(Token::Dot)),
                '[' => Ok(Some(Token::LBracket)),
                ']' => Ok(Some(Token::RBracket)),
                '{' => Ok(Some(Token::LBrace)),
                '}' => Ok(Some(Token::RBrace)),
                '(' => Ok(Some(Token::LParen)),
                ')' => Ok(Some(Token::RParen)),
                '<' => match self.scanner.peek() {
                    Some('=') => {
                        self.scanner.next();
                        Ok(Some(Token::LessEqual))
                    }
                    _ => Ok(Some(Token::LessThan)),
                },
                '>' => match self.scanner.peek() {
                    Some('=') => {
                        self.scanner.next();
                        Ok(Some(Token::GreaterEqual))
                    }
                    _ => Ok(Some(Token::GreaterThan)),
                },
                '=' => match self.scanner.peek() {
                    Some('=') => {
                        self.scanner.next();
                        Ok(Some(Token::EqualEqual))
                    }
                    _ => {
                        let (line, column) = self.scanner.location();
                        Err(SyntaxError { line, column })
                    }
                },
                'i' => self.parse_keywords_or_ident("i", &[("in", Token::In)]),
                't' => self.parse_keywords_or_ident(
                    "t",
                    &[("type", Token::Type), ("true", Token::BoolLit(true))],
                ),
                'f' => self.parse_keywords_or_ident("f", &[("false", Token::BoolLit(false))]),
                'n' => self.parse_keywords_or_ident("n", &[("null", Token::Null)]),
                '0'..='9' => self.parse_number_or_token(
                    input_char.encode_utf8(&mut tmp),
                    Token::IntLit(input_char as i64 - '0' as i64),
                ),
                '\'' | '"' => self.parse_string_literal(input_char),
                '_' | 'A'..='Z' | 'a'..='z' => {
                    return self.parse_keywords_or_ident(&input_char.to_string(), &[]);
                }
                _ => {
                    let (line, column) = self.scanner.location();

                    return Err(SyntaxError { line, column });
                }
            }
        } else {
            Ok(None)
        }
    }

    fn parse_string_literal(&mut self, starting: char) -> Result<Option<Token>, SyntaxError> {
        let mut working = String::new();

        'outer: loop {
            let curr = if let Some(curr) = self.scanner.next() {
                curr
            } else {
                let (line, column) = self.scanner.location();
                return Err(SyntaxError { line, column });
            };

            if curr == starting {
                break 'outer;
            } else if curr == '\\' {
                let escaped = if let Some(curr) = self.scanner.next() {
                    curr
                } else {
                    let (line, column) = self.scanner.location();
                    return Err(SyntaxError { line, column });
                };

                match escaped {
                    'n' => working.push('\n'),
                    'r' => working.push('\r'),
                    't' => working.push('\t'),
                    '\\' => working.push('\\'),
                    '\'' => working.push('\''),
                    '"' => working.push('"'),
                    other => working.push(other),
                }
            } else {
                working.push(curr);
            }
        }

        return Ok(Some(Token::StringLit(working)));
    }

    fn parse_keywords_or_ident(
        &mut self,
        starting: &str,
        options: &[(&str, Token)],
    ) -> Result<Option<Token>, SyntaxError> {
        let mut working = starting.to_owned();

        'outer: loop {
            if let Some(next) = self.scanner.peek() {
                match next {
                    'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
                        working.push(next);
                        self.scanner.next();
                    }
                    _ => break 'outer,
                }
            } else {
                break 'outer;
            }
        }

        if let Some(ident) = options.iter().find(|x| x.0 == working) {
            return Ok(Some(ident.1.clone()));
        } else {
            return Ok(Some(Token::Ident(working)));
        }
    }

    fn parse_number_or_token(
        &mut self,
        starting: &str,
        starting_token: Token,
    ) -> Result<Option<Token>, SyntaxError> {
        let mut working = starting.to_owned();
        let mut is_float = false;
        let mut is_unsigned = false;

        // a digit always follows a + or -
        if let Some(next) = self.scanner.peek() {
            match next {
                '0'..='9' => {
                    working.push(next);
                    self.scanner.next();
                }
                _ => return Ok(Some(starting_token)),
            };
        }

        'outer: loop {
            if let Some(next) = self.scanner.peek() {
                match next {
                    '0'..='9' => working.push(next),
                    'e' | 'E' | '.' => {
                        if is_float {
                            break 'outer;
                        }

                        is_float = true;
                        working.push(next);
                    }
                    'u' => {
                        if is_float {
                            break 'outer;
                        }

                        is_unsigned = true;
                        self.scanner.next();
                        break 'outer;
                    }
                    _ => break 'outer,
                };

                self.scanner.next();
            } else {
                break 'outer;
            }
        }

        if is_unsigned {
            match working.parse::<u64>() {
                Ok(value) => Ok(Some(Token::UIntLit(value))),
                Err(_) => {
                    let (line, column) = self.scanner.location();
                    Err(SyntaxError { line, column })
                }
            }
        } else if is_float {
            match working.parse::<f64>() {
                Ok(value) => Ok(Some(Token::FloatLit(value))),
                Err(_) => {
                    let (line, column) = self.scanner.location();
                    Err(SyntaxError { line, column })
                }
            }
        } else {
            match working.parse::<i64>() {
                Ok(value) => Ok(Some(Token::IntLit(value))),
                Err(_) => {
                    let (line, column) = self.scanner.location();
                    Err(SyntaxError { line, column })
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::{SyntaxError, Tokenizer};
    use crate::parser::tokens::Token;
    use test_case::test_case;

    fn _tokenize(input: &str) -> Result<Vec<Token>, SyntaxError> {
        let mut tokenizer = Tokenizer::with_input(input);
        let mut vec = Vec::new();

        loop {
            if let Some(token) = tokenizer.next()? {
                vec.push(token);
            } else {
                break;
            }
        }

        Ok(vec)
    }

    #[test_case("in", vec![Token::In]; "keyword in")]
    #[test_case("ident", vec![Token::Ident("ident".to_owned())]; "ident")]
    #[test_case("false", vec![Token::BoolLit(false)]; "keyword false")]
    #[test_case("true", vec![Token::BoolLit(true)]; "keyword true")]
    #[test_case("100", vec![Token::IntLit(100)]; "int literal")]
    #[test_case("3+4", vec![Token::IntLit(3), Token::Add, Token::IntLit(4)]; "parse addition")]
    #[test_case(r#""test\"123""#, vec![Token::StringLit("test\"123".to_string())]; "string literal")]
    fn test_tokenizer(input: &str, expected: Vec<Token>) {
        let tokens = _tokenize(input).unwrap();

        assert_eq!(tokens, expected);
    }
}
