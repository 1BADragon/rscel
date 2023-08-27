use std::str::Chars;

pub struct InputScanner<'l> {
    input: &'l str,
    iterator: Chars<'l>,
    current: Option<char>,
    line: usize,
    column: usize,
}

impl<'l> InputScanner<'l> {
    pub fn from_input(input: &'l str) -> InputScanner<'l> {
        InputScanner {
            input,
            iterator: input.chars(),
            current: None,
            line: 0,
            column: 0,
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
            let ret = self.current;
            self.current = None;
            ret
        }
    }

    pub fn location(&self) -> (usize, usize) {
        (self.line, self.column)
    }

    fn collect_next(&mut self) -> Option<char> {
        loop {
            self.column += 1;
            match self.iterator.next() {
                Some(val) => match val {
                    '\n' => {
                        self.line += 1;
                        self.column = 0;
                        return Some('\n');
                    }
                    val => return Some(val),
                },
                None => return None,
            }
        }
    }

    pub fn input(&self) -> &'l str {
        self.input
    }
}
