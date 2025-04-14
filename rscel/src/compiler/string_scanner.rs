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
        if self.current.is_none() {
            self.current = self.collect_next();
        }

        self.current
    }

    pub fn next(&mut self) -> Option<char> {
        if self.current.is_none() {
            self.current = self.collect_next();
        }

        match self.current {
            Some(c) => {
                if c == '\n' {
                    self.line += 1;
                    self.column = 0;
                } else {
                    self.column += 1;
                }

                self.current = None;
                Some(c)
            }
            None => None,
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
            Some(val) => Some(val),
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

#[cfg(test)]
mod test {
    use crate::compiler::source_location::SourceLocation;

    use super::StringScanner;

    #[test]
    fn string_scanner_location() {
        let mut scanner = StringScanner::from_input("foo + bar");

        assert_eq!(scanner.location(), SourceLocation::new(0, 0));

        let c = scanner.peek().unwrap();
        assert_eq!(c, 'f');
        assert_eq!(scanner.location(), SourceLocation::new(0, 0));

        let _ = scanner.next().unwrap();
        assert_eq!(scanner.location(), SourceLocation::new(0, 1));
    }
}
