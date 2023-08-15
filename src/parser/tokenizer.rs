use super::{input_scanner::InputScanner, tokens::Token};

struct Tokenizer<'l> {
    scanner: InputScanner<'l>,

    current: Option<Token>,
}

#[derive(Debug)]
pub struct SyntaxError {
    pub line: usize,
    pub column: usize,
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
            self.collect_next_token()
        } else {
            Ok(self.current)
        }
    }

    fn collect_next_token(&mut self) -> Result<Option<Token>, SyntaxError> {
        let start_char = self.scanner.next();
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
                'i' => self.parse_keywords_or_ident("i", ["in"]),
                't' => self.parse_keywords_or_ident("t", ["type", "true"]),
                _ => {
                    let (line, column) = self.scanner.location();

                    return Err(SyntaxError { line, column });
                }
            }
        } else {
            Ok(None)
        }
    }

    fn parse_keywords_or_ident(&self, starting: &str, options: &[str]) -> Result<Option<Token>> {
        
    }
    
    fn parse_ident(&self, starting: String) -> Result<Option<
}
