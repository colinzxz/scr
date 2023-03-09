#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Kind {
    Unknown,

    #[default]
    EOF,
    /// Any whitespace characters sequence, include newline.
    Whitespace,
    LineComment,
    BlockComment,

    Ident,
    Variable,
    Float,
    Number,
    Str,

    Amp, // &
    At,
    Bang,
    BracketL,
    BracketR,
    Caret, // ^
    Colon,
    Comma,
    CurlyL,
    CurlyR,
    Dollar,
    Dot,
    Dot3,
    Eq,
    Eq2,
    Minus,
    ParenL,
    ParenR,
    Percentage,
    Plus,
    Pound, // #
    Semicolon,
    Slash,
    Star,
    Tilde, // ~
    Gt,
    Lt,

    GtEq, // >=
    LtEq, // <=
    Neq,  // `!=`
    CDO,  // <!--
    CDC,  // -->
}

use core::fmt;

use self::Kind::*;

impl Kind {
    pub const fn to_str(self) -> &'static str {
        match self {
            Unknown => "Unknown",
            EOF => "EOF",
            Whitespace => " ",
            LineComment => "//",
            BlockComment => "/* */",
            Ident => "Identifier",
            Variable => "Variable",
            Float => "Float",
            Number => "Number",
            Str => "String",

            Amp => "&",
            At => "@",
            Bang => "!",
            BracketL => "[",
            BracketR => "]",
            Caret => "^",
            Colon => ":",
            Comma => ",",
            CurlyL => "{",
            CurlyR => "}",
            Dollar => "$",
            Dot => ".",
            Dot3 => "...",
            Eq => "=",
            Eq2 => "==",
            Minus => "-",
            ParenL => "(",
            ParenR => ")",
            Percentage => "%",
            Pound => "#",
            Plus => "+",
            Semicolon => ";",
            Slash => "/",
            Star => "*",
            Tilde => "~",

            Gt => ">",
            Lt => "<",
            GtEq => ">=",
            LtEq => "<=",
            Neq => "!=",

            CDC => "-->",
            CDO => "<!--",
        }
    }

    pub const fn is_trivia(self) -> bool {
        matches!(self, Whitespace | LineComment | BlockComment)
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}
