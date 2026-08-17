//! Raw tokens recognized by Logos before layout processing.
//!
//! This stage has no indentation state. The layout stage consumes its tokens and spans.

use logos::Logos;

use super::TokenKind;

#[derive(Clone, Copy, Debug, Eq, Logos, PartialEq)]
pub(super) enum RawToken {
    #[regex(r"[ \t\f]+")]
    Whitespace,
    #[regex(r"\r\n|\n|\r")]
    Newline,
    #[regex(r"#[^\r\n]*", allow_greedy = true)]
    Comment,
    #[token("fn")]
    Fn,
    #[token("kn")]
    Kn,
    #[token("mut")]
    Mut,
    #[token("give")]
    Give,
    #[token("return")]
    Return,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("while")]
    While,
    #[token("for")]
    For,
    #[token("in")]
    In,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("and")]
    And,
    #[token("or")]
    Or,
    #[token("not")]
    Not,
    #[token("fails")]
    Fails,
    #[token("with")]
    With,
    #[token("fail")]
    Fail,
    #[token("try")]
    Try,
    #[token("to")]
    To,
    #[token("catch")]
    Catch,
    #[token("on")]
    On,
    #[regex(r"[A-Za-z_][A-Za-z0-9_]*")]
    Identifier,
    #[regex(r"[0-9]+\.[0-9]+([eE][+-]?[0-9]+)?")]
    Float,
    #[regex(r"[0-9]+")]
    Integer,
    #[regex(r#""([^"\\]|\\.)*""#)]
    String,
    #[token(".")]
    Dot,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("@")]
    At,
    #[token("=")]
    Equal,
    #[token("==")]
    EqualEqual,
    #[token("!=")]
    BangEqual,
    #[token("<")]
    Less,
    #[token("<=")]
    LessEqual,
    #[token(">")]
    Greater,
    #[token(">=")]
    GreaterEqual,
    #[token("->")]
    Arrow,
}

impl RawToken {
    pub(super) const fn kind(self) -> TokenKind {
        match self {
            Self::Whitespace | Self::Newline | Self::Comment => unreachable!(),
            Self::Fn => TokenKind::Fn,
            Self::Kn => TokenKind::Kn,
            Self::Mut => TokenKind::Mut,
            Self::Give => TokenKind::Give,
            Self::Return => TokenKind::Return,
            Self::If => TokenKind::If,
            Self::Else => TokenKind::Else,
            Self::While => TokenKind::While,
            Self::For => TokenKind::For,
            Self::In => TokenKind::In,
            Self::True => TokenKind::True,
            Self::False => TokenKind::False,
            Self::And => TokenKind::And,
            Self::Or => TokenKind::Or,
            Self::Not => TokenKind::Not,
            Self::Fails => TokenKind::Fails,
            Self::With => TokenKind::With,
            Self::Fail => TokenKind::Fail,
            Self::Try => TokenKind::Try,
            Self::To => TokenKind::To,
            Self::Catch => TokenKind::Catch,
            Self::On => TokenKind::On,
            Self::Identifier => TokenKind::Identifier,
            Self::Integer => TokenKind::Integer,
            Self::Float => TokenKind::Float,
            Self::String => TokenKind::String,
            Self::Dot => TokenKind::Dot,
            Self::Comma => TokenKind::Comma,
            Self::Colon => TokenKind::Colon,
            Self::LParen => TokenKind::LParen,
            Self::RParen => TokenKind::RParen,
            Self::LBracket => TokenKind::LBracket,
            Self::RBracket => TokenKind::RBracket,
            Self::LBrace => TokenKind::LBrace,
            Self::RBrace => TokenKind::RBrace,
            Self::Plus => TokenKind::Plus,
            Self::Minus => TokenKind::Minus,
            Self::Star => TokenKind::Star,
            Self::Slash => TokenKind::Slash,
            Self::Percent => TokenKind::Percent,
            Self::At => TokenKind::At,
            Self::Equal => TokenKind::Equal,
            Self::EqualEqual => TokenKind::EqualEqual,
            Self::BangEqual => TokenKind::BangEqual,
            Self::Less => TokenKind::Less,
            Self::LessEqual => TokenKind::LessEqual,
            Self::Greater => TokenKind::Greater,
            Self::GreaterEqual => TokenKind::GreaterEqual,
            Self::Arrow => TokenKind::Arrow,
        }
    }
}
