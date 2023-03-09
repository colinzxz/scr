use std::{cell::RefCell, ops::Deref, rc::Rc, result};

use scr_ast::Span;
use thiserror::Error;

pub type Result<T> = result::Result<T, Diagnostic>;

#[derive(Debug, Clone, Error, miette::Diagnostic)]
pub enum Diagnostic {
    #[error("This file panicked")]
    #[diagnostic()]
    Panic(#[label("")] Span),

    #[error("Unexpected enn of line")]
    UnexpectedEnd(#[label("Unexpected enn of line")] Span),

    #[error("Expected digit")]
    ExpectedDigit(#[label("Unexpected digit")] Span),

    #[error("Invalid character `{0}`")]
    InvalidCharacter(char, #[label("Invalid character `{0}`")] Span),

    #[error("Invalid number")]
    InvalidNumber(&'static str, #[label("{0}")] Span),

    #[error("Unterminated block comment")]
    UnterminatedBlockComment(#[label("Unterminated block comment")] Span),

    #[error("Invalid Unicode code point")]
    InvalidUnicodeCodePoint(#[label("Invalid Unicode code point")] Span),

    #[error("Expected identifier")]
    ExpectedIdentifier(#[label("Expected identifier")] Span),

    #[error("Expected escape sequence.")]
    ExpectedEscapeSequence(#[label("Expected escape sequence")] Span),

    #[error("Syntax Error")]
    #[diagnostic()]
    ExceptedToken,
}

#[derive(Debug, Default, Clone)]
pub struct Diagnostics(Rc<RefCell<Vec<Diagnostic>>>);

impl Deref for Diagnostics {
    type Target = Rc<RefCell<Vec<Diagnostic>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Diagnostics {
    /// # Panics
    #[must_use]
    fn into_inner(self) -> Vec<Diagnostic> {
        Rc::try_unwrap(self.0).unwrap().into_inner()
    }
}
