use std::fmt::Debug;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenKind {
    Equals,
    Slash,
    Identifier(String),
    NumberLiteral(String),
    StringLiteral(String),
    End,
    Semicolon,
    DollarSign,
    SingleQuote,
    DoubleQuote,
    Hash,
    Space,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub row: usize,
    pub col: usize,
}
