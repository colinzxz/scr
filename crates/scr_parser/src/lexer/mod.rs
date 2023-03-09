mod definitions;
mod kind;
mod number;
mod string_builder;
mod token;

use std::{ops::Range, str::Chars};

use definitions::{
    is_dight, is_hex_digit, is_ident, is_ident_start, is_newline, is_whitespace, EOF_CHAR,
};
pub use kind::Kind;
use number::parse;
use scr_allocator::Allocator;
use scr_ast::{Atom, SourceType, Span};
use scr_diagnostics::{Diagnostic, Diagnostics};
use string_builder::AutoCow;
use token::{Token, TokenValue};

#[derive(Debug, Clone)]
pub struct LexerCheckpoint<'a> {
    chars: Chars<'a>,

    token: Token,

    #[cfg(debug_assertions)]
    /// For debug assertions only
    prev: char,
}

pub struct Lexer<'a> {
    pub allocator: &'a Allocator,

    source: &'a str,

    source_type: SourceType,

    pub current: LexerCheckpoint<'a>,

    errors: Diagnostics,
}

impl<'a> Lexer<'a> {
    pub fn new(
        allocator: &'a Allocator,
        source: &'a str,
        errors: Diagnostics,
        source_type: SourceType,
    ) -> Self {
        let current =
            LexerCheckpoint { chars: source.chars(), token: Token::default(), prev: EOF_CHAR };

        Self { allocator, source, source_type, current, errors }
    }

    pub fn remaining(&self) -> &'a str {
        self.current.chars.as_str()
    }

    pub fn next_token(&mut self) -> Token {
        let kind = self.read_next_token();
        self.finish_next(kind)
    }

    fn read_next_token(&mut self) -> Kind {
        self.current.token.start = self.offset();
        let builder = AutoCow::new(self);

        if let Some(c) = self.bump() {
            return self.match_char(c, builder);
        } else {
            return Kind::EOF;
        }
    }

    fn finish_next(&mut self, kind: Kind) -> Token {
        self.current.token.kind = kind;
        self.current.token.end = self.offset();
        debug_assert!(self.current.token.start <= self.current.token.end);
        std::mem::take(&mut self.current.token)
    }

    fn is_eof(&self) -> bool {
        self.current.chars.as_str().is_empty()
    }

    #[cfg(debug_assertions)]
    /// For debug assertions only
    fn prev(&self) -> char {
        self.current.prev
    }

    #[inline]
    fn offset(&self) -> usize {
        self.source.len() - self.current.chars.as_str().len()
    }

    #[inline]
    fn peek(&self) -> char {
        self.nth_char(0)
    }

    fn nth_char(&self, n: usize) -> char {
        self.current.chars.clone().nth(n).unwrap_or(EOF_CHAR)
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.current.chars.next()?;

        #[cfg(debug_assertions)]
        {
            self.current.prev = c
        }

        Some(c)
    }

    #[inline]
    fn next_eq(&mut self, c: char) -> bool {
        let eq = self.peek() == c;
        if eq {
            self.bump();
        }
        eq
    }

    fn eat_while(&mut self, mut predicate: impl FnMut(char) -> bool) {
        while predicate(self.peek()) && !self.is_eof() {
            self.bump();
        }
    }

    fn string_to_token_value(&self, s: &'a str) -> TokenValue {
        TokenValue::String(Atom::from(s))
    }

    fn unterminated_range(&self) -> Range<usize> {
        self.current.token.start..self.offset()
    }

    fn error(&mut self, error: Diagnostic) {
        self.errors.borrow_mut().push(error)
    }

    fn current_offset(&self) -> Span {
        self.offset()..self.offset()
    }

    fn unexpected_err(&mut self) {
        let c = self.peek();
        let error = if c == EOF_CHAR {
            Diagnostic::UnexpectedEnd(self.current_offset())
        } else {
            Diagnostic::InvalidCharacter(c, self.current_offset())
        };

        self.error(error)
    }
}

impl<'a> Lexer<'a> {
    fn match_char(&mut self, c: char, mut builder: AutoCow<'a>) -> Kind {
        let kind = match c {
            '/' => match self.peek() {
                '/' => self.read_line_comment(),
                '*' => self.read_block_comment(),
                _ => Kind::Slash,
            },
            c if is_whitespace(c) => self.read_whitespace(),

            c if c.is_ascii_alphabetic() => {
                let (escape, name) = self.identifier_name(builder);
                self.current.token.escaped = escape;
                self.current.token.value = self.string_to_token_value(name);
                Kind::Ident
            }

            c if is_ident_start(c) => {
                let (escape, name) = self.identifier_name(builder);
                self.current.token.escaped = escape;
                self.current.token.value = self.string_to_token_value(name);
                Kind::Ident
            }

            '0'..='9' => {
                let kind = self.read_digit(&mut builder);
                self.set_numeric_value(builder.finish(self));
                kind
            }

            '\\' => {
                builder.force_allocation_without_current_ascii_char(self);
                self.ident_escaped(&mut builder, true);
                let (_, name) = self.identifier(builder);
                self.current.token.value = self.string_to_token_value(name);
                Kind::Ident
            }
            '\'' | '"' => Kind::Str,

            '-' => self.read_minus(builder),
            '+' => Kind::Plus,
            '=' => Kind::Eq,
            '>' => Kind::Gt,
            '<' => self.read_lt(),
            '!' => Kind::Bang,
            '&' => Kind::Amp,
            '@' => Kind::At,
            '[' => Kind::BracketL,
            ']' => Kind::BracketR,
            ':' => Kind::Colon,
            ',' => Kind::Comma,
            '$' => Kind::Dollar,
            '.' => Kind::Dot,
            '{' => Kind::CurlyL,
            '}' => Kind::CurlyR,
            '(' => Kind::ParenL,
            ')' => Kind::ParenR,
            '%' => Kind::Percentage,
            '#' => Kind::Pound,
            ';' => Kind::Semicolon,
            '*' => Kind::Star,
            '^' => Kind::Caret,
            _ => {
                self.error(Diagnostic::InvalidCharacter(c, self.unterminated_range()));
                Kind::Unknown
            }
        };

        kind
    }

    fn read_line_comment(&mut self) -> Kind {
        debug_assert!(self.prev() == '/' && self.peek() == '/');
        self.bump();
        self.eat_while(|c| !is_newline(c));
        Kind::LineComment
    }

    fn read_block_comment(&mut self) -> Kind {
        debug_assert!(self.prev() == '/' && self.peek() == '*');
        self.bump();

        // for nest block comment
        let mut depth = 1usize;
        while let Some(c) = self.bump() {
            match c {
                '/' if self.next_eq('*') => depth += 1,
                '*' if self.next_eq('/') => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => (),
            }
        }

        if depth != 0 {
            self.error(Diagnostic::UnterminatedBlockComment(self.unterminated_range()));
            return Kind::EOF;
        }

        Kind::BlockComment
    }

    fn read_whitespace(&mut self) -> Kind {
        debug_assert!(is_whitespace(self.prev()));
        self.eat_while(is_whitespace);
        Kind::Whitespace
    }

    fn read_lt(&mut self) -> Kind {
        if self.nth_char(0) == '!' && self.nth_char(1) == '-' && self.nth_char(2) == '-' {
            self.bump();
            self.bump();
            self.bump();
            return Kind::CDO;
        }

        Kind::Lt
    }

    fn read_minus(&mut self, mut builder: AutoCow<'a>) -> Kind {
        debug_assert!(self.prev() == '-');

        if is_dight(self.peek()) {
            return self.read_digit(&mut builder);
        } else if self.next_eq('.') {
            builder.push_matching('.');
            return self.read_digits_after_point(&mut builder);
        } else if self.nth_char(0) == '-' && self.nth_char(1) == '>' {
            self.bump();
            self.bump();
            return Kind::CDC;
        } else if is_ident(self.peek()) {
            self.identifier_name(builder);
            return Kind::Ident;
        }

        Kind::Minus
    }

    fn read_digit(&mut self, builder: &mut AutoCow<'a>) -> Kind {
        self.read_digits_possible(builder);
        if self.next_eq('.') {
            builder.push_matching('.');
            return self.read_digits_after_point(builder);
        }
        let has_exponent = self.optional_exponent(builder);
        self.check_numeric_literal(if has_exponent { Kind::Float } else { Kind::Number })
    }

    fn set_numeric_value(&mut self, src: &'a str) {
        match parse(src).map(TokenValue::Number) {
            Ok(val) => self.current.token.value = val,
            Err(err) => {
                self.error(Diagnostic::InvalidNumber(err, self.current.token.start..self.offset()));
                self.current.token.value = TokenValue::Number(std::f64::NAN);
            }
        }
    }

    fn check_numeric_literal(&self, kind: Kind) -> Kind {
        kind
    }

    fn read_digits_after_point(&mut self, builder: &mut AutoCow<'a>) -> Kind {
        self.read_digits(builder);
        self.optional_exponent(builder);
        self.check_numeric_literal(Kind::Float)
    }

    fn read_digits(&mut self, builder: &mut AutoCow<'a>) {
        if self.peek().is_ascii_digit() {
            builder.push_matching(self.bump().unwrap());
        } else {
            self.error(Diagnostic::ExpectedDigit(self.current_offset()));
            return;
        }
        self.read_digits_possible(builder);
    }

    fn read_digits_possible(&mut self, builder: &mut AutoCow<'a>) {
        loop {
            match self.peek() {
                c @ '0'..='9' => {
                    self.bump();
                    builder.push_matching(c);
                }
                _ => break,
            }
        }
    }

    fn optional_exponent(&mut self, builder: &mut AutoCow<'a>) -> bool {
        if matches!(self.peek(), 'e' | 'E') {
            builder.push_matching(self.bump().unwrap());
            self.read_exponent(builder);
            return true;
        }
        false
    }

    fn read_exponent(&mut self, builder: &mut AutoCow<'a>) -> Kind {
        debug_assert!(matches!(self.prev(), 'e' | 'E'));
        let c = self.peek();
        if matches!(c, '+' | '-') {
            self.bump();
            builder.push_matching(c);
        }
        self.read_digits(builder);
        Kind::Float
    }

    fn identifier(&mut self, mut builder: AutoCow<'a>) -> (bool, &'a str) {
        if self.next_eq('-') && self.next_eq('-') {
            return self.identifier_name(builder);
        }

        let first = self.peek();
        if is_ident_start(first) {
            builder.push_matching(first);
            self.bump();
        } else if first == '\\' {
            self.bump();
            builder.force_allocation_without_current_ascii_char(self);
            self.ident_escaped(&mut builder, /* ident_start */ true)
        } else {
            self.error(Diagnostic::ExpectedIdentifier(self.unterminated_range()))
        }
        return self.identifier_name(builder);
    }

    fn identifier_name(&mut self, mut builder: AutoCow<'a>) -> (bool, &'a str) {
        loop {
            let c = self.peek();
            if !is_ident(c) {
                if self.next_eq('\\') {
                    builder.force_allocation_without_current_ascii_char(self);
                    self.ident_escaped(&mut builder, /* ident_start */ true);
                    continue;
                }
                break;
            }
            self.bump();
            builder.push_matching(c)
        }

        (builder.has_escape(), builder.finish(self))
    }

    fn ident_escaped(&mut self, bulder: &mut AutoCow<'a>, ident_start: bool) {
        debug_assert!(self.prev() == '\\');
        let mut escape_char: Option<char> = None;
        let c = self.peek();
        if self.is_eof() || is_newline(c) {
            self.error(Diagnostic::ExpectedEscapeSequence(self.unterminated_range()))
        } else if is_hex_digit(c) {
            let mut counter = 0;
            let mut hex = String::new();

            while counter < 6 {
                let next = self.peek();
                if !is_hex_digit(next) {
                    break;
                }
                counter += 1;
                self.bump();
                hex.push(next);
            }

            // If the next input code point is whitespace, consume it as well.
            if is_whitespace(self.peek()) {
                self.bump();
            }

            // Interpret the hex digits as a hexadecimal number.
            let number = u32::from_str_radix(hex.as_str(), 16).unwrap();

            // If this number is zero, or is for a surrogate, or is greater than the maximum allowed code point,
            if number == 0 || (number >= 0xd800 && number <= 0xdfff) || number > 0x0010_ffff {
                self.error(Diagnostic::InvalidUnicodeCodePoint(self.unterminated_range()));
                escape_char = None
            } else {
                escape_char = char::from_u32(number);
            }
        } else {
            escape_char = self.bump();
        }

        if let Some(c) = escape_char {
            let matched = if ident_start { is_ident_start(c) } else { is_ident(c) };
            if matched {
                bulder.push_matching(c);
            } else if (ident_start && is_dight(c)) || c <= '\x1f' || c == '\x7f' {
                bulder.push_matching('\\');
                let value = c as u32;
                if c > '\x0f' {
                    bulder.push_matching(char::from_digit(value >> 4, 16).unwrap());
                }
                bulder.push_matching(char::from_digit(value & 0xf, 16).unwrap());
                bulder.push_matching(' ');
            } else {
                bulder.push_matching('\\');
                bulder.push_matching(c);
            }
        }
    }
}

#[test]
fn test() {
    let allocator = Allocator::default();
    let errors = Diagnostics::default();
    let source_type = SourceType::default();
    let path = std::path::Path::new(
        &std::env::var("CARGO_MANIFEST_DIR")
            .unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_owned()),
    )
    .ancestors()
    .nth(2)
    .unwrap()
    .join("target/a.scss");
    let source_text = std::fs::read_to_string(path).unwrap();

    let mut lexer = Lexer::new(&allocator, source_text.as_str(), errors, source_type);
    loop {
        let token = lexer.next_token();
        println!("{token:?}");
        if token.kind == Kind::EOF {
            break;
        }
    }

    println!("Lexer errors: {:#?}", lexer.errors);
}
