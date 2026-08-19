// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: MIT

//! Parser-facing token contract.
//!
//! These tokens include the logical layout added after raw lexing.

use crate::source::Span;

/// A source token with its exact byte range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Token {
    /// Parser-visible category after keyword and layout classification.
    pub kind: TokenKind,

    /// Exact source bytes consumed by this token.
    pub span: Span,
}

impl Token {
    /// Construct one parser-facing token.
    #[must_use]
    pub const fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    /// Borrow the token's original source spelling.
    ///
    /// # Panics
    ///
    /// Panics when the token span did not originate from `source`. Compiler
    /// stages preserve that pairing as an internal invariant.
    #[must_use]
    pub fn text<'source>(&self, source: &'source str) -> &'source str {
        &source[self.span.start..self.span.end]
    }
}

/// Tokens consumed by the parser.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenKind {
    /// Host-function declaration keyword `fn`.
    Fn,

    /// Device-kernel declaration keyword `kn`.
    Kn,

    /// Exclusive-borrow parameter keyword `mut`.
    Mut,

    /// Ownership-transfer parameter keyword `give`.
    Give,

    /// Early function-exit keyword `return`.
    Return,

    /// Conditional-expression keyword `if`.
    If,

    /// False-branch keyword `else`.
    Else,

    /// Conditional loop keyword `while`.
    While,

    /// Iterable loop keyword `for`.
    For,

    /// Iteration membership keyword `in`.
    In,

    /// Boolean literal `true`.
    True,

    /// Boolean literal `false`.
    False,

    /// Boolean conjunction keyword `and`.
    And,

    /// Boolean disjunction and error-clause keyword `or`.
    Or,

    /// Boolean negation keyword `not`.
    Not,

    /// Error-channel keyword `fails`.
    Fails,

    /// Error-type and error-value introducer `with`.
    With,

    /// Error construction keyword `fail`.
    Fail,

    /// Error propagation keyword `try`.
    Try,

    /// Required continuation in `try to`.
    To,

    /// Local error recovery keyword `catch`.
    Catch,

    /// Target or event preposition reserved by the grammar.
    On,

    /// User-defined or context-resolved name.
    Identifier,

    /// Decimal integer literal.
    Integer,

    /// Decimal floating-point literal.
    Float,

    /// Interpreted one-line string literal.
    String,

    /// Member-selection punctuation `.`.
    Dot,

    /// List and argument separator `,`.
    Comma,

    /// Type-annotation and slice punctuation `:`.
    Colon,

    /// Opening call or grouping delimiter `(`.
    LParen,

    /// Closing call or grouping delimiter `)`.
    RParen,

    /// Opening type, tensor, or index delimiter `[`.
    LBracket,

    /// Closing type, tensor, or index delimiter `]`.
    RBracket,

    /// Rejected brace delimiter retained for precise diagnostics.
    LBrace,

    /// Rejected brace delimiter retained for precise diagnostics.
    RBrace,

    /// Numeric identity or addition operator `+`.
    Plus,

    /// Numeric negation or subtraction operator `-`.
    Minus,

    /// Multiplication or pointer punctuation `*`.
    Star,

    /// Fractional division operator `/`.
    Slash,

    /// Integer remainder operator `%`.
    Percent,

    /// Matrix multiplication operator `@`.
    At,

    /// Assignment punctuation `=`.
    Equal,

    /// Equality comparison `==`.
    EqualEqual,

    /// Inequality comparison `!=`.
    BangEqual,

    /// Strict less-than comparison `<`.
    Less,

    /// Less-than-or-equal comparison `<=`.
    LessEqual,

    /// Strict greater-than comparison `>`.
    Greater,

    /// Greater-than-or-equal comparison `>=`.
    GreaterEqual,

    /// Function-result punctuation `->`.
    Arrow,

    /// Logical source-line boundary outside explicit delimiters.
    Newline,

    /// Entry into a more deeply indented source block.
    Indent,

    /// Return to one enclosing indentation depth.
    Dedent,

    /// Synthetic end-of-file marker consumed by the parser.
    Eof,
}
