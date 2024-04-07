use super::{
    input_scanner::StringScanner,
    syntax_error::SyntaxError,
    tokenizer::Tokenizer,
    tokens::{NumericLiteral, Token},
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
                '-' => {
                    if let Some(token) = self.scanner.peek() {
                        match token {
                            '.' | '0'..='9' => {
                                let mut starting = String::new();
                                self.scanner.next();
                                starting.push('-');
                                starting.push(token);
                                self.parse_number_or_token(&starting)
                            }
                            _ => Ok(Some(Token::Minus)),
                        }
                    } else {
                        Ok(Some(Token::Minus))
                    }
                }
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
                            self.parse_number_or_token(input_char.encode_utf8(&mut tmp))
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
                'b' => {
                    if let Some('\'') = self.scanner.peek() {
                        self.scanner.next();
                        self.parse_bytes_literal('\'')
                    } else if let Some('"') = self.scanner.peek() {
                        self.scanner.next();
                        self.parse_bytes_literal('"')
                    } else {
                        self.parse_keywords_or_ident("b", &[])
                    }
                }
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
                '0'..='9' => self.parse_number_or_token(input_char.encode_utf8(&mut tmp)),
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

    fn parse_bytes_literal(&mut self, starting: char) -> Result<Option<Token>, SyntaxError> {
        let mut buf = [0u8; 4];
        let mut working = Vec::new();

        'outer: loop {
            let curr = if let Some(curr) = self.scanner.next() {
                curr
            } else {
                return Err(SyntaxError::from_location(self.scanner.location()));
            };

            if curr == starting {
                break 'outer;
            } else if curr == '\\' {
                let escaped = if let Some(curr) = self.scanner.next() {
                    curr
                } else {
                    let (_line, _column) = self.scanner.location();
                    return Err(SyntaxError::from_location(self.scanner.location()));
                };

                match escaped {
                    'a' => working.push(0x07u8),
                    'b' => working.push(0x08u8),
                    'f' => working.push(0x0cu8),
                    'n' => working.push('\n' as u8),
                    'r' => working.push('\r' as u8),
                    't' => working.push('\t' as u8),
                    'v' => working.push(0x0bu8),
                    '\\' => working.push('\\' as u8),
                    '\'' => working.push('\'' as u8),
                    '"' => working.push('"' as u8),
                    other => {
                        other.encode_utf8(&mut buf);
                        working.extend_from_slice(&buf[..other.len_utf8()]);
                    }
                }
            } else {
                curr.encode_utf8(&mut buf);
                println!("{:?} {}", buf, curr.len_utf8());
                working.extend_from_slice(&buf[..curr.len_utf8()]);
            }
        }

        Ok(Some(Token::ByteStringLit(working)))
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

        Ok(Some(Token::StringLit(working)))
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

    fn parse_number_or_token(&mut self, starting: &str) -> Result<Option<Token>, SyntaxError> {
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

        let orig = working.clone();
        let fixedup_str = match base {
            10 => &working,
            16 => working.trim_start_matches("0x"),
            _ => return Err(SyntaxError::from_location(self.scanner.location())),
        };

        if is_unsigned {
            // u64 is always positive and doesn't have a neg rep that cannot be represeted in a u64
            match u64::from_str_radix(fixedup_str, base) {
                Ok(val) => Ok(Some(Token::UIntLit(val))),
                Err(_) => Err(SyntaxError::from_location(self.scanner.location())
                    .with_message(format!("Failed to parse unsigned int {}", orig))),
            }
        } else if is_float {
            Ok(Some(Token::FloatLit(NumericLiteral {
                value: working.parse::<f64>().ok(),
                str_value: fixedup_str.to_string(),
                base: 10,
                location: self.scanner.location(),
            })))
        } else {
            Ok(Some(Token::IntLit(NumericLiteral {
                value: i64::from_str_radix(fixedup_str, base).ok(),
                str_value: fixedup_str.to_string(),
                base,
                location: self.scanner.location(),
            })))
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
