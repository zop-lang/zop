//! Logical newlines and indentation tokens layered over raw lexing.
//!
//! Flow:
//!
//! 1. Logos recognizes raw tokens and byte spans.
//! 2. Leading spaces update an indentation stack on logical lines.
//! 3. Newlines inside explicit delimiters are ignored.
//! 4. End of file emits the final newline, dedents, and `Eof`.

use std::{cmp::Ordering, mem};

use logos::Logos;

use crate::{
    diagnostic::{Diagnostic, Diagnostics, Stage},
    source::Span,
};

use super::{Token, TokenKind, raw::RawToken};

/// Tokenize one source file and insert logical layout tokens.
pub fn lex(source: &str) -> Result<Vec<Token>, Diagnostics> {
    LayoutLexer::new(source).lex()
}

struct LayoutLexer<'source> {
    source: &'source str,
    tokens: Vec<Token>,
    errors: Diagnostics,
    indentations: Vec<usize>,
    delimiters: Vec<(TokenKind, Span)>,
    at_line_start: bool,
    line_has_token: bool,
    pending_indentation: usize,
}

impl<'source> LayoutLexer<'source> {
    fn new(source: &'source str) -> Self {
        Self {
            source,
            tokens: Vec::new(),
            errors: Vec::new(),
            indentations: vec![0],
            delimiters: Vec::new(),
            at_line_start: true,
            line_has_token: false,
            pending_indentation: 0,
        }
    }

    fn lex(mut self) -> Result<Vec<Token>, Diagnostics> {
        let mut raw = RawToken::lexer(self.source).spanned();

        for (result, range) in &mut raw {
            let span = Span::from(range);
            match result {
                Ok(RawToken::Whitespace) => self.whitespace(span),
                Ok(RawToken::Comment) => {}
                Ok(RawToken::Newline) => self.newline(span),
                Ok(raw_kind) => self.significant(raw_kind, span),
                Err(()) => self.error("L0001", "unrecognized character", span),
            }
        }

        self.finish();

        if self.errors.is_empty() { Ok(self.tokens) } else { Err(self.errors) }
    }

    fn whitespace(&mut self, span: Span) {
        if !self.at_line_start || !self.delimiters.is_empty() {
            return;
        }

        let whitespace = &self.source[span.start..span.end];
        if whitespace.contains('\t') {
            self.error("L0002", "tabs are not allowed in indentation", span);
        }

        self.pending_indentation = whitespace.bytes().filter(|byte| *byte == b' ').count();
    }

    fn newline(&mut self, span: Span) {
        if self.delimiters.is_empty() && self.line_has_token {
            self.tokens.push(Token::new(TokenKind::Newline, span));
            self.line_has_token = false;
        }

        self.at_line_start = true;
        self.pending_indentation = 0;
    }

    fn significant(&mut self, raw_kind: RawToken, span: Span) {
        if self.at_line_start {
            if self.delimiters.is_empty() {
                self.layout(span.start);
            }
            self.at_line_start = false;
        }

        let kind = raw_kind.kind();
        self.update_delimiters(kind, span);
        self.tokens.push(Token::new(kind, span));
        self.line_has_token = true;
    }

    fn layout(&mut self, offset: usize) {
        let current = *self.indentations.last().expect("root indentation exists");
        match self.pending_indentation.cmp(&current) {
            Ordering::Greater => self.indent(offset),
            Ordering::Equal => {}
            Ordering::Less => self.dedent(offset),
        }
    }

    fn indent(&mut self, offset: usize) {
        self.indentations.push(self.pending_indentation);
        let span = Span::new(offset - self.pending_indentation, offset);
        self.tokens.push(Token::new(TokenKind::Indent, span));
    }

    fn dedent(&mut self, offset: usize) {
        while self.pending_indentation < *self.indentations.last().expect("root indentation exists")
        {
            self.indentations.pop();
            self.tokens.push(Token::new(TokenKind::Dedent, Span::empty(offset)));
        }

        if self.pending_indentation != *self.indentations.last().expect("root indentation exists") {
            let span = Span::new(offset - self.pending_indentation, offset);
            self.error("L0003", "dedent does not match an enclosing block", span);
        }
    }

    fn update_delimiters(&mut self, kind: TokenKind, span: Span) {
        if matches!(kind, TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace) {
            self.delimiters.push((kind, span));
            return;
        }

        let expected = match kind {
            TokenKind::RParen => Some(TokenKind::LParen),
            TokenKind::RBracket => Some(TokenKind::LBracket),
            TokenKind::RBrace => Some(TokenKind::LBrace),
            _ => None,
        };
        let Some(expected) = expected else {
            return;
        };

        match self.delimiters.pop() {
            Some((actual, _)) if actual == expected => {}
            Some((actual, opening_span)) => {
                let message = format!("mismatched delimiter opened by {actual:?}");
                self.error("L0004", message, opening_span);
            }
            None => self.error("L0004", "closing delimiter has no opener", span),
        }
    }

    fn finish(&mut self) {
        let offset = self.source.len();
        if self.line_has_token {
            self.tokens.push(Token::new(TokenKind::Newline, Span::empty(offset)));
        }

        while self.indentations.len() > 1 {
            self.indentations.pop();
            self.tokens.push(Token::new(TokenKind::Dedent, Span::empty(offset)));
        }

        let unclosed = mem::take(&mut self.delimiters);
        for (_, span) in unclosed {
            self.error("L0004", "unclosed delimiter", span);
        }

        self.tokens.push(Token::new(TokenKind::Eof, Span::empty(offset)));
    }

    fn error(&mut self, code: &'static str, message: impl Into<String>, span: Span) {
        self.errors.push(Diagnostic::new(Stage::Lex, code, message, span));
    }
}
