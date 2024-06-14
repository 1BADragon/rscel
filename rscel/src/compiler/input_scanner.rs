use std::str::Chars;

use super::source_location::SourceLocation;

pub struct StringScanner<'l> {
    input: &'l str,
    iterator: Chars<'l>,
    current: Option<char>,
    line: usize,
    column: usize,
    eof: bool,
}

impl<'l> StringScanner<'l> {
    pub fn from_input(input: &'l str) -> StringScanner<'l> {
        StringScanner {
            input,
            iterator: input.chars(),
            current: None,
            line: 0,
            column: 0,
            eof: false,
        }
    }

    pub fn peek(&mut self) -> Option<char> {
        if let None = self.current {
            self.current = self.collect_next();
        }

        self.current
    }

    pub fn next(&mut self) -> Option<char> {
        if let None = self.current {
            self.collect_next()
        } else {
            std::mem::replace(&mut self.current, None)
        }
    }

    pub fn location(&self) -> SourceLocation {
        SourceLocation::new(self.line, self.column)
    }

    fn collect_next(&mut self) -> Option<char> {
        if self.eof {
            return None;
        }

        match self.iterator.next() {
            Some(val) => match val {
                '\n' => {
                    self.line += 1;
                    self.column = 0;
                    Some('\n')
                }
                val => {
                    self.column += 1;
                    Some(val)
                }
            },
            None => {
                self.eof = true;
                None
            }
        }
    }

    pub fn input(&self) -> &'l str {
        self.input
    }
}
