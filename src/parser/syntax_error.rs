#[derive(Debug)]
pub struct SyntaxError {
    pub line: usize,
    pub column: usize,
}
