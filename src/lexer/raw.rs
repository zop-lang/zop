// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: Apache-2.0

//! Raw tokens recognized by Logos before layout processing.
//!
//! This stage has no indentation state. The layout stage consumes its tokens and spans.

use logos::Logos;

use super::TokenKind;

/// Logos token classes before indentation and keyword-independent layout work.
#[derive(Clone, Copy, Debug, Eq, Logos, PartialEq)]
pub(super) enum RawToken {
    /// Horizontal spacing retained so the layout pass can measure indentation.
    #[regex(r"[ \t\f]+")]
    Whitespace,

    /// Physical line ending before delimiter-aware suppression.
    #[regex(r"\r\n|\n|\r")]
    Newline,

    /// Non-documentation line comment discarded by the bootstrap layout pass.
    #[regex(r"#[^\r\n]*", allow_greedy = true)]
    Comment,

    /// Exact `fn` keyword spelling.
    #[token("fn")]
    Fn,

    /// Exact `kn` keyword spelling.
    #[token("kn")]
    Kn,

    /// Exact `mut` keyword spelling.
    #[token("mut")]
    Mut,

    /// Exact `give` keyword spelling.
    #[token("give")]
    Give,

    /// Exact `return` keyword spelling.
    #[token("return")]
    Return,

    /// Exact `if` keyword spelling.
    #[token("if")]
    If,

    /// Exact `else` keyword spelling.
    #[token("else")]
    Else,

    /// Exact `while` keyword spelling.
    #[token("while")]
    While,

    /// Exact `for` keyword spelling.
    #[token("for")]
    For,

    /// Exact `in` keyword spelling.
    #[token("in")]
    In,

    /// Exact `true` literal spelling.
    #[token("true")]
    True,

    /// Exact `false` literal spelling.
    #[token("false")]
    False,

    /// Exact `and` keyword spelling.
    #[token("and")]
    And,

    /// Exact `or` keyword spelling.
    #[token("or")]
    Or,

    /// Exact `not` keyword spelling.
    #[token("not")]
    Not,

    /// Exact `fails` keyword spelling.
    #[token("fails")]
    Fails,

    /// Exact `with` keyword spelling.
    #[token("with")]
    With,

    /// Exact `fail` keyword spelling.
    #[token("fail")]
    Fail,

    /// Exact `try` keyword spelling.
    #[token("try")]
    Try,

    /// Exact `to` keyword spelling.
    #[token("to")]
    To,

    /// Exact `catch` keyword spelling.
    #[token("catch")]
    Catch,

    /// Exact `on` keyword spelling.
    #[token("on")]
    On,

    /// ASCII identifier before name resolution.
    #[regex(r"[A-Za-z_][A-Za-z0-9_]*")]
    Identifier,

    /// Decimal floating literal with an optional exponent.
    #[regex(r"[0-9]+\.[0-9]+([eE][+-]?[0-9]+)?")]
    Float,

    /// Unsuffixed decimal integer literal.
    #[regex(r"[0-9]+")]
    Integer,

    /// One-line quoted string with bootstrap escape recognition.
    #[regex(r#""([^"\\]|\\.)*""#)]
    String,

    /// Member-selection punctuation.
    #[token(".")]
    Dot,

    /// Comma separator.
    #[token(",")]
    Comma,

    /// Colon punctuation.
    #[token(":")]
    Colon,

    /// Opening parenthesis.
    #[token("(")]
    LParen,

    /// Closing parenthesis.
    #[token(")")]
    RParen,

    /// Opening square bracket.
    #[token("[")]
    LBracket,

    /// Closing square bracket.
    #[token("]")]
    RBracket,

    /// Opening brace retained for rejection after lexing.
    #[token("{")]
    LBrace,

    /// Closing brace retained for rejection after lexing.
    #[token("}")]
    RBrace,

    /// Plus punctuation.
    #[token("+")]
    Plus,

    /// Minus punctuation.
    #[token("-")]
    Minus,

    /// Star punctuation.
    #[token("*")]
    Star,

    /// Slash punctuation.
    #[token("/")]
    Slash,

    /// Percent punctuation.
    #[token("%")]
    Percent,

    /// At-sign punctuation.
    #[token("@")]
    At,

    /// Assignment punctuation.
    #[token("=")]
    Equal,

    /// Equality punctuation recognized before assignment.
    #[token("==")]
    EqualEqual,

    /// Inequality punctuation.
    #[token("!=")]
    BangEqual,

    /// Less-than punctuation.
    #[token("<")]
    Less,

    /// Less-than-or-equal punctuation.
    #[token("<=")]
    LessEqual,

    /// Greater-than punctuation.
    #[token(">")]
    Greater,

    /// Greater-than-or-equal punctuation.
    #[token(">=")]
    GreaterEqual,

    /// Function-result punctuation.
    #[token("->")]
    Arrow,
}

impl RawToken {
    /// Convert one significant raw token to its parser-facing kind.
    ///
    /// # Panics
    ///
    /// Panics for whitespace, comments, or physical newlines. The layout pass
    /// handles those variants before requesting a parser-facing kind.
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
