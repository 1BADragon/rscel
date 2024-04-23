use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AstNode<T> {
    start: (usize, usize),
    end: (usize, usize),

    node: T,
}

impl<T> AstNode<T> {
    pub fn new(node: T, start: (usize, usize), end: (usize, usize)) -> AstNode<T> {
        AstNode::<T> { start, end, node }
    }

    pub fn start(&self) -> (usize, usize) {
        self.start
    }

    pub fn end(&self) -> (usize, usize) {
        self.end
    }
}
