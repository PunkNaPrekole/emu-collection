#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
    pub start: usize,
    pub end: usize,
}

impl Default for Span {
    fn default() -> Self {
        Self {
            line: 1,
            column: 1,
            start: 0,
            end: 0,
        }
    }
}