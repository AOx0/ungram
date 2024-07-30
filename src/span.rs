#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

impl Default for Span {
    fn default() -> Self {
        Span { start: 0, end: 0 }
    }
}

impl std::fmt::Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

impl From<std::ops::Range<usize>> for Span {
    fn from(range: std::ops::Range<usize>) -> Self {
        Span {
            start: range.start,
            end: range.end,
        }
    }
}

impl std::fmt::Debug for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{line}:{col}", line = self.line, col = self.column)
    }
}

impl Span {
    pub fn new(start: usize, end: usize) -> Span {
        Span { start, end }
    }

    pub fn location(&self, source: &str) -> Location {
        let mut line = 1;
        let mut column = 1;
        for (i, c) in source.chars().enumerate() {
            if i == self.start {
                break;
            }
            if c == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }
        Location { line, column }
    }

    pub fn range(&self) -> std::ops::Range<usize> {
        self.start..self.end
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }
}
