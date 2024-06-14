use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SourceLocation(usize, usize);

impl SourceLocation {
    pub fn new(line: usize, col: usize) -> SourceLocation {
        SourceLocation(line, col)
    }

    pub fn line(&self) -> usize {
        self.0
    }

    pub fn col(&self) -> usize {
        self.1
    }
}

#[cfg(test)]
mod test {
    use super::SourceLocation;

    #[test]
    fn test_source_location() {
        let loc1 = SourceLocation(0, 1);
        let loc2 = SourceLocation(1, 1);
        let loc3 = SourceLocation(0, 1);

        assert!(loc1 < loc2);
        assert_eq!(loc1.line(), 0);
        assert_eq!(loc1.col(), 1);
        assert_eq!(loc1, loc3);
    }
}
