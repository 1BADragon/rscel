use super::{
    source_location::SourceLocation,
    source_range::SourceRange,
    string_scanner::StringScanner,
    syntax_error::SyntaxError,
    tokenizer::{TokenWithLoc, Tokenizer},
    tokens::{FStringSegment, Token},
};

pub struct StringTokenizer<'l> {
    scanner: StringScanner<'l>,

    current: Option<TokenWithLoc>,

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

    fn collect_next_token(&mut self) -> Result<Option<TokenWithLoc>, SyntaxError> {
        let mut tmp = [0; 4];
        let mut token_start = self.location();
        let mut curr_char = self.scanner.next();

        if self.eof {
            return Ok(None);
        }

        'outer: loop {
            match curr_char {
                Some(' ') | Some('\t') | Some('\n') => {
                    token_start = self.location();
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
                'c' => self.parse_keywords_or_ident("c", &[("case", Token::Case)]),
                'f' => {
                    if let Some('\'') = self.scanner.peek() {
                        self.scanner.next();
                        self.parse_string_literal('\'', false, true)
                    } else if let Some('"') = self.scanner.peek() {
                        self.scanner.next();
                        self.parse_string_literal('"', false, true)
                    } else {
                        self.parse_keywords_or_ident("f", &[("false", Token::BoolLit(false))])
                    }
                }
                'i' => self.parse_keywords_or_ident("i", &[("in", Token::In)]),
                'm' => self.parse_keywords_or_ident("m", &[("match", Token::Match)]),
                'n' => self.parse_keywords_or_ident("n", &[("null", Token::Null)]),
                'r' => {
                    if let Some('\'') = self.scanner.peek() {
                        self.scanner.next();
                        self.parse_string_literal('\'', true, false)
                    } else if let Some('"') = self.scanner.peek() {
                        self.scanner.next();
                        self.parse_string_literal('"', true, false)
                    } else {
                        self.parse_keywords_or_ident("r", &[])
                    }
                }
                't' => self.parse_keywords_or_ident("t", &[("true", Token::BoolLit(true))]),
                '0'..='9' => self.parse_number_or_token(input_char.encode_utf8(&mut tmp)),
                '\'' | '"' => self.parse_string_literal(input_char, false, false),
                '_' | 'A'..='Z' | 'a'..='z' => {
                    self.parse_keywords_or_ident(&input_char.to_string(), &[])
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

        res.map(|o| o.map(|t| TokenWithLoc::new(t, SourceRange::new(token_start, self.location()))))
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
                    'x' => working.push(self.extract_hex_val(2)? as u8),
                    'X' => working.push(self.extract_hex_val(2)? as u8),
                    '\\' => working.push('\\' as u8),
                    '\'' => working.push('\'' as u8),
                    '"' => working.push('"' as u8),
                    '0'..='9' => {
                        let mut oct: String = [escaped].into_iter().collect();
                        for _ in 0..2 {
                            match self.scanner.next() {
                                Some(c) => oct.push(c),
                                None => {
                                    return Err(SyntaxError::from_location(self.scanner.location())
                                        .with_message(format!("Octal number requires 3 digits")))
                                }
                            }
                        }

                        let val = match u8::from_str_radix(&oct, 8) {
                            Ok(v) => v,
                            Err(_) => {
                                return Err(SyntaxError::from_location(self.scanner.location())
                                    .with_message(format!("{} is not a valid octal number", oct)))
                            }
                        };

                        working.push(val)
                    }
                    other => {
                        other.encode_utf8(&mut buf);
                        working.extend_from_slice(&buf[..other.len_utf8()]);
                    }
                }
            } else {
                curr.encode_utf8(&mut buf);
                working.extend_from_slice(&buf[..curr.len_utf8()]);
            }
        }

        Ok(Some(Token::ByteStringLit(working.into())))
    }

    fn parse_string_literal(
        &mut self,
        starting: char,
        is_raw: bool,
        is_format: bool,
    ) -> Result<Option<Token>, SyntaxError> {
        let mut working = String::new();
        let mut segments = Vec::new();

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
                    return Err(SyntaxError::from_location(self.scanner.location()));
                };

                match escaped {
                    'a' => working.push(0x07 as char),
                    'b' => working.push(0x08 as char),
                    'f' => working.push(0x0c as char),
                    'n' => working.push('\n'),
                    'r' => working.push('\r'),
                    't' => working.push('\t'),
                    'u' => working.push(self.extract_hex_val(4)?),
                    'U' => working.push(self.extract_hex_val(8)?),
                    'v' => working.push(0x0b as char),
                    'x' => working.push(self.extract_hex_val(2)?),
                    'X' => working.push(self.extract_hex_val(2)?),
                    '\\' => working.push('\\'),
                    '\'' => working.push('\''),
                    '"' => working.push('"'),
                    '0'..='9' => {
                        let mut oct: String = [escaped].into_iter().collect();
                        for _ in 0..2 {
                            match self.scanner.next() {
                                Some(c) => oct.push(c),
                                None => {
                                    return Err(SyntaxError::from_location(self.scanner.location())
                                        .with_message(format!("Octal number requires 3 digits")))
                                }
                            }
                        }

                        let val = match u32::from_str_radix(&oct, 8) {
                            Ok(v) => v,
                            Err(_) => {
                                return Err(SyntaxError::from_location(self.scanner.location())
                                    .with_message(format!("{} is not a valid octal number", oct)))
                            }
                        };

                        working.push(match char::from_u32(val) {
                            Some(c) => c,
                            None => {
                                return Err(SyntaxError::from_location(self.scanner.location())
                                    .with_message(format!("Invalid code point {}", val)))
                            }
                        })
                    }
                    other => working.push(other),
                }
            } else if curr == '{' && is_format {
                let escaped = if let Some(curr) = self.scanner.next() {
                    curr
                } else {
                    return Err(SyntaxError::from_location(self.scanner.location()));
                };

                match escaped {
                    '{' => working.push('{'),
                    c => {
                        if !working.is_empty() {
                            segments.push(FStringSegment::Lit(working));
                            working = String::new();
                        }

                        if c == '}' {
                            return Err(SyntaxError::from_location(self.scanner.location())
                                .with_message("Empty format specifier".to_string()));
                        }

                        working.push(c);
                        let mut bracket_count = 1;
                        while bracket_count > 0 {
                            if let Some(c) = self.scanner.next() {
                                match c {
                                    '}' => {
                                        bracket_count -= 1;
                                        if bracket_count > 0 {
                                            working.push('}');
                                        }
                                    }
                                    '{' => {
                                        bracket_count += 1;
                                        working.push('{');
                                    }
                                    other => working.push(other),
                                }
                            } else {
                                return Err(SyntaxError::from_location(self.scanner.location()));
                            }
                        }

                        if working.is_empty() {
                            return Err(SyntaxError::from_location(self.scanner.location()));
                        }

                        segments.push(FStringSegment::Expr(working));
                        working = String::new();
                    }
                }
            } else if curr == '}' && is_format {
                let escaped = if let Some(curr) = self.scanner.next() {
                    curr
                } else {
                    return Err(SyntaxError::from_location(self.scanner.location()));
                };

                if escaped == '}' {
                    working.push('}');
                } else {
                    return Err(SyntaxError::from_location(self.scanner.location())
                        .with_message("Single } not allowed".to_string()));
                }
            } else {
                working.push(curr);
            }
        }

        if segments.is_empty() {
            Ok(Some(Token::StringLit(working)))
        } else {
            if !working.is_empty() {
                segments.push(FStringSegment::Lit(working))
            }
            Ok(Some(Token::FStringLit(segments)))
        }
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
            match working.parse::<f64>() {
                Ok(v) => Ok(Some(Token::FloatLit(v))),
                Err(_) => Err(SyntaxError::from_location(self.scanner.location())
                    .with_message(format!("Failed to parse float {}", orig))),
            }
        } else {
            match u64::from_str_radix(fixedup_str, base) {
                Ok(val) => Ok(Some(Token::IntLit(val))),
                Err(_) => Err(SyntaxError::from_location(self.scanner.location())
                    .with_message(format!("Failed to parse unsigned int {}", orig))),
            }
        }
    }

    fn extract_hex_val(&mut self, len: usize) -> Result<char, SyntaxError> {
        let mut code_str = String::new();
        for _ in 0..len {
            code_str.push(match self.scanner.next() {
                Some(c) => {
                    if c.is_digit(16) {
                        c
                    } else {
                        return Err(SyntaxError::from_location(self.scanner.location())
                            .with_message(format!(
                                "{} is not a valid unicode code point value",
                                c
                            )));
                    }
                }
                None => {
                    return Err(SyntaxError::from_location(self.scanner.location())
                        .with_message(format!("Expected {} hex digits after unicode escape", len)))
                }
            });
        }

        let unicode_value: u32 = u32::from_str_radix(&code_str, 16).unwrap();

        match char::from_u32(unicode_value) {
            Some(c) => Ok(c),
            None => Err(
                SyntaxError::from_location(self.scanner.location()).with_message(format!(
                    "{:x} is not a valid unicode code point",
                    unicode_value
                )),
            ),
        }
    }
}

impl Tokenizer for StringTokenizer<'_> {
    fn peek(&mut self) -> Result<Option<&TokenWithLoc>, SyntaxError> {
        if let None = self.current {
            match self.collect_next_token() {
                Ok(token) => self.current = token,
                Err(err) => return Err(err),
            };
        }
        Ok(self.current.as_ref())
    }

    fn next(&mut self) -> Result<Option<TokenWithLoc>, SyntaxError> {
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

    fn location(&self) -> SourceLocation {
        self.scanner.location()
    }
}

#[cfg(test)]
mod test {
    use crate::{
        compiler::{
            source_location::SourceLocation, source_range::SourceRange, tokenizer::TokenWithLoc,
            tokens::Token,
        },
        StringTokenizer, Tokenizer,
    };

    #[test]
    fn tokens_locations() {
        let src = "foo + 3";

        let mut tokenizer = StringTokenizer::with_input(src);

        let t = tokenizer.next().unwrap().unwrap();
        assert_eq!(
            t,
            TokenWithLoc {
                token: Token::Ident("foo".to_owned()),
                loc: SourceRange::new(SourceLocation::new(0, 0), SourceLocation::new(0, 3))
            }
        );

        assert_eq!(&src[t.loc.start().col()..t.loc.end().col()], "foo");
        assert_eq!(
            tokenizer.next().unwrap().unwrap(),
            TokenWithLoc {
                token: Token::Add,
                loc: SourceRange::new(SourceLocation::new(0, 4), SourceLocation::new(0, 5))
            }
        );

        assert_eq!(
            tokenizer.next().unwrap().unwrap(),
            TokenWithLoc {
                token: Token::IntLit(3),
                loc: SourceRange::new(SourceLocation::new(0, 6), SourceLocation::new(0, 7))
            }
        )
    }

    #[test]
    fn string_literal_loc() {
        let src = "\"this is a test string\"";

        let mut tokenozer = StringTokenizer::with_input(src);

        assert_eq!(
            tokenozer.next().unwrap().unwrap(),
            TokenWithLoc {
                token: Token::StringLit("this is a test string".to_owned()),
                loc: SourceRange::new(SourceLocation::new(0, 0), SourceLocation::new(0, src.len()))
            }
        );
    }
}
