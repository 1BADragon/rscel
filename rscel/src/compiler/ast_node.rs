use serde::{Deserialize, Serialize};

use crate::CelValue;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AstNode<T> {
    start: (usize, usize),
    end: (usize, usize),

    node: T,
    value: Option<CelValue>,
}

impl<T> AstNode<T> {
    pub fn new(node: T, start: (usize, usize), end: (usize, usize)) -> AstNode<T> {
        AstNode::<T> {
            start,
            end,
            node,
            value: None,
        }
    }

    pub fn start(&self) -> (usize, usize) {
        self.start
    }

    pub fn end(&self) -> (usize, usize) {
        self.end
    }

    pub fn node(&self) -> &T {
        &self.node
    }
}
