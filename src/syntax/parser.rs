//! Module, declaration, type, and block parsing.
//!
//! Expression parsing lives beside this stage so declarations do not absorb precedence logic.

use crate::{
    diagnostic::{Diagnostic, Diagnostics, Stage},
    lexer::{Token, TokenKind, lex},
    source::Span,
};

use super::{
    Block, Dimension, FailureType, Function, FunctionKind, Module, Parameter, ParameterMode,
    TypeExpression,
};

/// Parse one complete Zop source file.
pub fn parse(source: &str) -> Result<Module, Diagnostics> {
    let tokens = lex(source)?;
    Parser::new(source, &tokens).parse_module()
}

pub(super) struct Parser<'source, 'tokens> {
    pub(super) source: &'source str,
    pub(super) tokens: &'tokens [Token],
    pub(super) position: usize,
    pub(super) errors: Diagnostics,
}

impl<'source, 'tokens> Parser<'source, 'tokens> {
    fn new(source: &'source str, tokens: &'tokens [Token]) -> Self {
        Self { source, tokens, position: 0, errors: Vec::new() }
    }

    fn parse_module(mut self) -> Result<Module, Diagnostics> {
        let mut functions = Vec::new();
        while !self.at(TokenKind::Eof) {
            if self.eat(TokenKind::Newline).is_some() {
                continue;
            }

            if self.at(TokenKind::Fn) || self.at(TokenKind::Kn) {
                if let Some(function) = self.parse_function() {
                    functions.push(function);
                }
                continue;
            }

            self.error_current("P0001", "expected a function declaration");
            self.skip_line();
        }

        let module = Module { functions, span: Span::new(0, self.source.len()) };
        if self.errors.is_empty() { Ok(module) } else { Err(self.errors) }
    }

    fn parse_function(&mut self) -> Option<Function> {
        let start = self.current().span.start;
        let kind = if self.eat(TokenKind::Fn).is_some() {
            FunctionKind::Host
        } else {
            self.bump();
            FunctionKind::Kernel
        };
        let name = self.expect_identifier("P0002", "expected a function name")?;
        let parameters = self.parse_parameters();
        let return_type = self
            .eat(TokenKind::Arrow)
            .and_then(|_| self.parse_type("P0006", "expected a return type"));
        let failure = self.parse_failure();

        self.expect(TokenKind::Newline, "P0007", "expected a newline before the function body");
        let body = self.parse_block()?;
        let span = Span::new(start, body.span.end);

        Some(Function { kind, name, parameters, return_type, failure, body, span })
    }

    fn parse_parameters(&mut self) -> Vec<Parameter> {
        let parenthesized = self.eat(TokenKind::LParen).is_some();
        let mut parameters = Vec::new();

        while self.parameter_starts_here() {
            if let Some(parameter) = self.parse_parameter() {
                parameters.push(parameter);
            }

            if self.eat(TokenKind::Comma).is_some() {
                if parenthesized && self.at(TokenKind::RParen) {
                    break;
                }
                continue;
            }

            if self.parameter_starts_here() {
                self.error_current("P0005", "expected `,` between parameters");
                continue;
            }
            break;
        }

        if parenthesized {
            self.expect(TokenKind::RParen, "P0004", "expected `)` after parameters");
        }
        parameters
    }

    fn parse_parameter(&mut self) -> Option<Parameter> {
        let start = self.current().span.start;
        let mode = if self.eat(TokenKind::Mut).is_some() {
            ParameterMode::Mut
        } else if self.eat(TokenKind::Give).is_some() {
            ParameterMode::Give
        } else {
            ParameterMode::Borrow
        };
        let name = self.expect_identifier("P0002", "expected a parameter name")?;
        self.expect(TokenKind::Colon, "P0003", "expected `:` after the parameter name");
        let ty = self.parse_type("P0003", "expected a parameter type")?;

        Some(Parameter { mode, name, span: Span::new(start, ty.span.end), ty })
    }

    fn parse_type(&mut self, code: &'static str, message: &'static str) -> Option<TypeExpression> {
        let start = self.current().span.start;
        let name = self.expect_identifier(code, message)?;
        let mut dimensions = Vec::new();
        let mut end = self.previous().span.end;

        if self.eat(TokenKind::LBracket).is_some() {
            while !self.at(TokenKind::RBracket) && !self.at(TokenKind::Eof) {
                dimensions.push(self.parse_dimension()?);
                if self.eat(TokenKind::Comma).is_none() {
                    break;
                }
            }
            end = self
                .expect(TokenKind::RBracket, "P0003", "expected `]` after dimensions")
                .map_or(end, |token| token.span.end);
        }

        Some(TypeExpression { name, dimensions, span: Span::new(start, end) })
    }

    fn parse_dimension(&mut self) -> Option<Dimension> {
        let token = self.current();
        match token.kind {
            TokenKind::Identifier => {
                self.bump();
                Some(Dimension::Name(token.text(self.source).to_owned()))
            }
            TokenKind::Integer => {
                self.bump();
                match token.text(self.source).parse() {
                    Ok(value) => Some(Dimension::Integer(value)),
                    Err(_) => {
                        self.error("P0014", "tensor dimension is too large", token.span);
                        None
                    }
                }
            }
            _ => {
                self.error_current("P0003", "expected a tensor dimension");
                None
            }
        }
    }

    fn parse_failure(&mut self) -> FailureType {
        if self.eat(TokenKind::Or).is_none() {
            return FailureType::None;
        }
        self.expect(TokenKind::Fails, "P0008", "expected `fails` after `or`");

        if self.eat(TokenKind::With).is_some() {
            self.parse_type("P0008", "expected an error type after `with`")
                .map_or(FailureType::Infer, FailureType::Named)
        } else {
            FailureType::Infer
        }
    }

    pub(super) fn parse_block(&mut self) -> Option<Block> {
        let indent = self.expect(TokenKind::Indent, "P0009", "expected an indented block")?;
        let mut expressions = Vec::new();

        while !self.at(TokenKind::Dedent) && !self.at(TokenKind::Eof) {
            if self.eat(TokenKind::Newline).is_some() {
                continue;
            }

            match self.parse_expression() {
                Some(expression) => expressions.push(expression),
                None => self.skip_line(),
            }
            self.eat(TokenKind::Newline);
        }

        let dedent = self.expect(TokenKind::Dedent, "P0010", "expected the block to end")?;
        Some(Block { expressions, span: Span::new(indent.span.start, dedent.span.end) })
    }

    fn parameter_starts_here(&self) -> bool {
        matches!(self.current().kind, TokenKind::Mut | TokenKind::Give)
            || (self.at(TokenKind::Identifier) && self.nth(1).kind == TokenKind::Colon)
    }

    pub(super) fn current(&self) -> Token {
        self.nth(0)
    }

    pub(super) fn previous(&self) -> Token {
        self.tokens[self.position.saturating_sub(1)]
    }

    pub(super) fn nth(&self, offset: usize) -> Token {
        let index = (self.position + offset).min(self.tokens.len() - 1);
        self.tokens[index]
    }

    pub(super) fn at(&self, kind: TokenKind) -> bool {
        self.current().kind == kind
    }

    pub(super) fn bump(&mut self) -> Token {
        let token = self.current();
        if token.kind != TokenKind::Eof {
            self.position += 1;
        }
        token
    }

    pub(super) fn eat(&mut self, kind: TokenKind) -> Option<Token> {
        self.at(kind).then(|| self.bump())
    }

    pub(super) fn expect(
        &mut self,
        kind: TokenKind,
        code: &'static str,
        message: &'static str,
    ) -> Option<Token> {
        if self.at(kind) {
            Some(self.bump())
        } else {
            self.error_current(code, message);
            None
        }
    }

    pub(super) fn expect_identifier(
        &mut self,
        code: &'static str,
        message: &'static str,
    ) -> Option<String> {
        let token = self.expect(TokenKind::Identifier, code, message)?;
        Some(token.text(self.source).to_owned())
    }

    pub(super) fn error_current(&mut self, code: &'static str, message: impl Into<String>) {
        self.error(code, message, self.current().span);
    }

    pub(super) fn error(&mut self, code: &'static str, message: impl Into<String>, span: Span) {
        self.errors.push(Diagnostic::new(Stage::Parse, code, message, span));
    }

    pub(super) fn skip_line(&mut self) {
        let initial_position = self.position;
        while !matches!(
            self.current().kind,
            TokenKind::Newline | TokenKind::Dedent | TokenKind::Eof
        ) {
            self.bump();
        }
        self.eat(TokenKind::Newline);
        if self.position == initial_position && !self.at(TokenKind::Eof) {
            self.bump();
        }
    }
}
