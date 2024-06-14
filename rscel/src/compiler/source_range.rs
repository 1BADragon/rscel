use serde::{Deserialize, Serialize};

use super::source_location::SourceLocation;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRange {
    start: SourceLocation,
    end: SourceLocation,
}

impl SourceRange {
    pub fn new(start: SourceLocation, end: SourceLocation) -> SourceRange {
        SourceRange { start, end }
    }

    pub fn start(&self) -> SourceLocation {
        self.start
    }

    pub fn end(&self) -> SourceLocation {
        self.end
    }

    pub fn surrounding(self, other: SourceRange) -> SourceRange {
        SourceRange::new(self.start.min(other.start), self.end.max(other.end))
    }
}

#[cfg(test)]
mod test {
    use crate::compiler::source_location::SourceLocation;

    use super::SourceRange;

    #[test]
    fn test_surrounding() {
        let p = SourceRange::new(SourceLocation::new(0, 3), SourceLocation::new(0, 5)).surrounding(
            SourceRange::new(SourceLocation::new(0, 4), SourceLocation::new(0, 7)),
        );

        assert_eq!(
            p,
            SourceRange::new(SourceLocation::new(0, 3), SourceLocation::new(0, 7))
        );
    }
}
