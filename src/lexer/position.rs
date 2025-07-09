// source https://github.com/jDomantas/plank/blob/master/plank-syntax/src/position.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Serialize, Deserialize, Hash)]
pub struct Position {
    pub abs: usize,
    pub row: usize,
    pub column: usize,
}

impl Position {
    pub fn new(abs: usize, row: usize, column: usize) -> Self {
        Position { abs, row, column }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Serialize, Deserialize, Hash)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Default for Span {
    fn default() -> Self {
        Span {
            start: Position::new(0, 0, 0),
            end: Position::new(1, 1, 1),
        }
    }
}

impl Span {
    pub fn new(start: Position, end: Position) -> Self {
        // if start <= end {
        Span { start, end }
        // }
        // panic!("start > end")
    }

    pub fn extend(&self, span: &Span) -> Self {
        // if span.start <= self.start || span.end <= self.end {
        //     panic!("invalid span")
        // }

        Span::new(self.start, span.end)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Serialize, Deserialize, Hash)]
pub struct Spanned<T> {
    pub value: T,
    // #[serde(skip_serializing)]
    pub span: Span,
}
