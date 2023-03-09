use std::ops::Range;

use scr_ast::Atom;

use super::Kind;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Token {
    /// Token kind
    pub kind: Kind,
    /// start offset in source file
    pub start: usize,
    /// end offset if source file
    pub end: usize,

    /// Is the origin string escape?
    pub escaped: bool,

    pub value: TokenValue,
}

impl Token {
    #[must_use]
    pub fn range(&self) -> Range<usize> {
        self.start..self.end
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum TokenValue {
    #[default]
    None,
    String(Atom),
    Number(f64),
}

impl TokenValue {
    //
}
