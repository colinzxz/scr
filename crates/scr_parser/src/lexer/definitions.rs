// https://drafts.csswg.org/css-syntax-3/
// § 4.2. Definitions

pub const EOF_CHAR: char = '\0';

pub fn is_dight(c: char) -> bool {
    c.is_ascii_digit()
}

pub fn is_hex_digit(c: char) -> bool {
    c.is_ascii_hexdigit()
}

pub fn is_letter(c: char) -> bool {
    c.is_ascii_alphabetic()
}

/// non-Ascii
/// A code point with a value equal to or greater than U+0080 <control>.
pub fn is_non_ascii(c: char) -> bool {
    c > '\x7f'
}

/// ident-start code point
/// a letter, a non-ASCII code point, or U+005F LOW LINE (_).
pub fn is_ident_start(c: char) -> bool {
    is_letter(c) || is_non_ascii(c) || c == '_'
}

/// ident code point
/// an ident-start code point, a digit, or U+002D HYPHEN-MINUS (-).
pub fn is_ident(c: char) -> bool {
    is_ident_start(c) || is_dight(c) || c == '-'
}

/// non-printable code point
/// a code point between U+0000 NULL and U+0008 BACKSPACE inclusive, or U+000B LINE TABULATION,
/// or a code point between U+000E SHIFT OUT and U+001F INFORMATION SEPARATOR ONE inclusive, or U+007F DELETE.
pub fn is_non_printable(c: char) -> bool {
    (c >= '\x00' && c <= '\x08') || c == '\x0b' || (c >= '\x0e' && c <= '\x1f') || c == '\x7f'
}

/// newline
/// U+000A LINE FEED. Note that U+000D CARRIAGE RETURN and U+000C FORM FEED are not included in this definition,
/// as they are converted to U+000A LINE FEED during preprocessing.
// TODO: we doesn't do a preprocessing, so check a code point for U+000D CARRIAGE RETURN and U+000C FORM FEED
pub fn is_newline(c: char) -> bool {
    c == '\x0a' || c == '\x0d' || c == '\x0c'
}

/// whitespace
/// a newline, U+0009 CHARACTER TABULATION, or U+0020 SPACE.
pub fn is_whitespace(c: char) -> bool {
    is_newline(c) || c == '\x09' || c == '\x20'
}
