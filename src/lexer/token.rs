//! Parser-facing token contract.
//!
//! These tokens include the logical layout added after raw lexing.

use crate::source::Span;

/// A source token with its exact byte range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    #[must_use]
    pub const fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    #[must_use]
    pub fn text<'source>(&self, source: &'source str) -> &'source str {
        &source[self.span.start..self.span.end]
    }
}

/// Tokens consumed by the parser.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenKind {
    Fn,
    Kn,
    Mut,
    Give,
    Return,
    If,
    Else,
    While,
    For,
    In,
    True,
    False,
    And,
    Or,
    Not,
    Fails,
    With,
    Fail,
    Try,
    To,
    Catch,
    On,
    Identifier,
    Integer,
    Float,
    String,
    Dot,
    Comma,
    Colon,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    At,
    Equal,
    EqualEqual,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Arrow,
    Newline,
    Indent,
    Dedent,
    Eof,
}
