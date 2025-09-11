use serde::{Deserialize, Serialize};

use super::{source_location::SourceLocation, source_range::SourceRange};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AstNode<T> {
    pub loc: SourceRange,

    pub node: T,
}

impl<T> AstNode<T> {
    pub fn new(node: T, loc: SourceRange) -> AstNode<T> {
        AstNode::<T> { loc, node }
    }

    pub fn into_parts(self) -> (T, SourceRange) {
        (self.node, self.loc)
    }

    pub fn start(&self) -> SourceLocation {
        self.loc.start()
    }

    pub fn end(&self) -> SourceLocation {
        self.loc.end()
    }

    pub fn range(&self) -> SourceRange {
        self.loc
    }

    pub fn node(&self) -> &T {
        &self.node
    }
}

#[cfg(test)]
mod test {
    use super::AstNode;
    use crate::{SourceLocation, SourceRange};

    #[derive(Debug, PartialEq)]
    struct FakeNode {
        v: u32,
    }

    #[test]
    fn basic() {
        let a = AstNode::new(
            FakeNode { v: 42 },
            SourceRange::new(SourceLocation::new(0, 0), SourceLocation::new(0, 20)),
        );

        assert_eq!(a.start(), SourceLocation::new(0, 0));
        assert_eq!(a.end(), SourceLocation::new(0, 20));

        assert_eq!(
            a.range(),
            SourceRange::new(SourceLocation::new(0, 0), SourceLocation::new(0, 20))
        );

        assert_eq!(*a.node(), FakeNode { v: 42 });

        let (node, range) = a.into_parts();
        assert_eq!(
            range,
            SourceRange::new(SourceLocation::new(0, 0), SourceLocation::new(0, 20))
        );

        assert_eq!(node.v, 42);
    }
}
