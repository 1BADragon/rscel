use super::{
    input_scanner::StringScanner, syntax_error::SyntaxError, tokenizer::Tokenizer, tokens::Token,
};

pub struct StringTokenizer<'l> {
    scanner: StringScanner<'l>,

    current: Option<Token>,

    eof: bool,
}

impl<'l> StringTokenizer<'l> {
    pub fn with_input(input: &'l str) -> StringTokenizer<'l> {
        StringTokenizer {
            scanner: StringScanner::from_input(input),
            current: None,
            eof: false,
        }
    }

    fn collect_next_token(&mut self) -> Result<Option<Token>, SyntaxError> {
        let mut tmp = [0; 4];
        let mut curr_char = self.scanner.next();

        if self.eof {
            return Ok(None);
        }

        'outer: loop {
            match curr_char {
                Some(' ') | Some('\t') | Some('\n') => {
                    curr_char = self.scanner.next();
                }
                _ => break 'outer,
            };
        }

        let res = if let Some(input_char) = curr_char {
            match input_char {
                '?' => Ok(Some(Token::Question)),
                ':' => Ok(Some(Token::Colon)),
                '+' => Ok(Some(Token::Add)),
                '-' => Ok(Some(Token::Minus)),
                '*' => Ok(Some(Token::Multiply)),
                '/' => Ok(Some(Token::Divide)),
                '%' => Ok(Some(Token::Mod)),
                '!' => match self.scanner.peek() {
                    Some('=') => {
                        self.scanner.next();
                        Ok(Some(Token::NotEqual))
                    }
                    _ => Ok(Some(Token::Not)),
                },
                '.' => {
                    if let Some(v) = self.scanner.peek() {
                        if v >= '0' && v <= '9' {
                            self.parse_number_or_token(input_char.encode_utf8(&mut tmp), Token::Dot)
                        } else {
                            Ok(Some(Token::Dot))
                        }
                    } else {
                        Ok(Some(Token::Dot))
                    }
                }
                ',' => Ok(Some(Token::Comma)),
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
                    _ => Err(SyntaxError::from_location(self.scanner.location())
                        .with_message("Token = is not supported".to_string())),
                },
                '|' => match self.scanner.peek() {
                    Some('|') => {
                        self.scanner.next();
                        Ok(Some(Token::OrOr))
                    }
                    _ => Err(SyntaxError::from_location(self.scanner.location())
                        .with_message("Token | is not supported".to_string())),
                },
                '&' => match self.scanner.peek() {
                    Some('&') => {
                        self.scanner.next();
                        Ok(Some(Token::AndAnd))
                    }
                    _ => Err(SyntaxError::from_location(self.scanner.location())
                        .with_message("Token & is not supported".to_string())),
                },
                'f' => self.parse_keywords_or_ident("f", &[("false", Token::BoolLit(false))]),
                'i' => self.parse_keywords_or_ident("i", &[("in", Token::In)]),
                'n' => self.parse_keywords_or_ident("n", &[("null", Token::Null)]),
                'r' => {
                    if let Some('\'') = self.scanner.peek() {
                        self.scanner.next();
                        self.parse_string_literal('\'', true)
                    } else if let Some('"') = self.scanner.peek() {
                        self.scanner.next();
                        self.parse_string_literal('"', true)
                    } else {
                        self.parse_keywords_or_ident("r", &[])
                    }
                }
                't' => self.parse_keywords_or_ident("t", &[("true", Token::BoolLit(true))]),
                '0'..='9' => self.parse_number_or_token(
                    input_char.encode_utf8(&mut tmp),
                    Token::IntLit(input_char as i64 - '0' as i64),
                ),
                '\'' | '"' => self.parse_string_literal(input_char, false),
                '_' | 'A'..='Z' | 'a'..='z' => {
                    return self.parse_keywords_or_ident(&input_char.to_string(), &[]);
                }
                other => {
                    return Err(SyntaxError::from_location(self.scanner.location())
                        .with_message(format!("Unexpected symbol: '{}'", other)));
                }
            }
        } else {
            self.eof = true;
            Ok(None)
        };

        #[cfg(feature = "debug_output")]
        {
            if let Ok(Some(ref val)) = res {
                println!("[tokenizer]: collect {:?}", val);
            } else if let Ok(None) = res {
                println!("[tokenizer]: EOF");
            } else if let Err(ref err) = res {
                println!("[tokenizer]: {:?}", err);
            }
        }

        res
    }

    fn parse_string_literal(
        &mut self,
        starting: char,
        is_raw: bool,
    ) -> Result<Option<Token>, SyntaxError> {
        let mut working = String::new();

        'outer: loop {
            let curr = if let Some(curr) = self.scanner.next() {
                curr
            } else {
                return Err(SyntaxError::from_location(self.scanner.location()));
            };

            if curr == starting {
                break 'outer;
            } else if curr == '\\' && !is_raw {
                let escaped = if let Some(curr) = self.scanner.next() {
                    curr
                } else {
                    let (_line, _column) = self.scanner.location();
                    return Err(SyntaxError::from_location(self.scanner.location()));
                };

                match escaped {
                    'a' => working.push(0x07 as char),
                    'b' => working.push(0x08 as char),
                    'f' => working.push(0x0c as char),
                    'n' => working.push('\n'),
                    'r' => working.push('\r'),
                    't' => working.push('\t'),
                    'v' => working.push(0x0b as char),
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
        _starting_token: Token,
    ) -> Result<Option<Token>, SyntaxError> {
        let mut working = starting.to_owned();
        let mut is_float = starting.contains(".");
        let mut is_exp = false;
        let mut is_unsigned = false;
        let mut base = 10;

        'outer: loop {
            if let Some(next) = self.scanner.peek() {
                match next {
                    '0' => {
                        working.push(next);
                        self.scanner.next();
                    }
                    '1'..='9' => {
                        working.push(next);
                        self.scanner.next();
                    }
                    'e' | 'E' | '.' => {
                        if next == '.' && is_float {
                            break 'outer;
                        } else if is_exp {
                            break 'outer;
                        }

                        is_float = true;

                        if next == 'e' || next == 'E' {
                            is_exp = true;
                        }
                        working.push(next);

                        self.scanner.next();
                        if let Some(p) = self.scanner.peek() {
                            match p {
                                '+' | '-' => {
                                    self.scanner.next();
                                    working.push(p);
                                }
                                _ => {
                                    continue 'outer;
                                }
                            }
                        }
                    }
                    'u' | 'U' => {
                        if is_float {
                            break 'outer;
                        }

                        is_unsigned = true;
                        self.scanner.next();
                        break 'outer;
                    }
                    'x' | 'X' => {
                        if working == "0" && base == 10 {
                            working.push('x');
                            self.scanner.next();
                            base = 16;
                        } else {
                            break 'outer;
                        }
                    }
                    _ => break 'outer,
                };
            } else {
                break 'outer;
            }
        }

        let fixedup_str = match base {
            10 => &working,
            16 => working.trim_start_matches("0x"),
            _ => return Err(SyntaxError::from_location(self.scanner.location())),
        };

        if is_unsigned {
            match u64::from_str_radix(fixedup_str, base) {
                Ok(value) => Ok(Some(Token::UIntLit(value))),
                Err(_) => Err(SyntaxError::from_location(self.scanner.location())
                    .with_message(format!("Failed to parse unsigned value: {}", working))),
            }
        } else if is_float {
            match working.parse::<f64>() {
                Ok(value) => Ok(Some(Token::FloatLit(value))),
                Err(_) => Err(SyntaxError::from_location(self.scanner.location())
                    .with_message(format!("Failed to parse floating point value: {}", working))),
            }
        } else {
            match i64::from_str_radix(fixedup_str, base) {
                Ok(value) => Ok(Some(Token::IntLit(value))),
                Err(_) => Err(SyntaxError::from_location(self.scanner.location())
                    .with_message(format!("Failed to parse integer: {}", working))),
            }
        }
    }
}

impl Tokenizer for StringTokenizer<'_> {
    fn peek(&mut self) -> Result<Option<Token>, SyntaxError> {
        if let None = self.current {
            match self.collect_next_token() {
                Ok(token) => self.current = token,
                Err(err) => return Err(err),
            };
        }
        Ok(self.current.clone())
    }

    fn next(&mut self) -> Result<Option<Token>, SyntaxError> {
        if let None = self.current {
            self.collect_next_token()
        } else {
            let tmp = std::mem::replace(&mut self.current, None);
            Ok(tmp)
        }
    }

    fn source<'a>(&'a self) -> &'a str {
        self.scanner.input()
    }

    fn location(&self) -> (usize, usize) {
        self.scanner.location()
    }
}

#[cfg(test)]
mod test {
    use super::{StringTokenizer, SyntaxError, Tokenizer};
    use crate::compiler::tokens::Token;
    use test_case::test_case;

    fn _tokenize(input: &str) -> Result<Vec<Token>, SyntaxError> {
        let mut tokenizer = StringTokenizer::with_input(input);
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
    #[test_case(".4", vec![Token::FloatLit(0.4)]; "parse float 2")]
    #[test_case(r#""test\"123""#, vec![Token::StringLit("test\"123".to_string())]; "string literal")]
    #[test_case("-0.4", vec![Token::Minus, Token::FloatLit(0.4)]; "parse neg float")]
    #[test_case("-.4", vec![Token::Minus, Token::FloatLit(0.4)]; "parse neg float 2")]
    #[test_case("0e+0", vec![Token::FloatLit(0.0)]; "parse exp with pos sign")]
    #[test_case("0x0", vec![Token::IntLit(0)]; "parse hex value")]
    #[test_case("-2.3e+1", vec![Token::Minus, Token::FloatLit(23.0)]; "parse neg float exp")]
    fn test_tokenizer(input: &str, expected: Vec<Token>) {
        let tokens = _tokenize(input).unwrap();

        assert_eq!(tokens, expected);
    }
}
