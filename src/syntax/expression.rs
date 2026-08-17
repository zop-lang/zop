//! Expression parsing and precedence.
//!
//! The declaration parser delegates here after it enters a function body.

use crate::{lexer::TokenKind, source::Span};

use super::{
    Argument, BinaryOperator, CatchArm, Expression, ExpressionKind, Pattern, UnaryOperator,
    parser::Parser,
};

impl Parser<'_, '_> {
    pub(super) fn parse_expression(&mut self) -> Option<Expression> {
        let left = self.parse_binary(0, true)?;
        let expression = if self.eat(TokenKind::Equal).is_some() {
            let value = self.parse_expression()?;
            let span = Span::new(left.span.start, value.span.end);
            Expression::new(
                ExpressionKind::Assign { target: Box::new(left), value: Box::new(value) },
                span,
            )
        } else {
            left
        };

        self.parse_catches(expression)
    }

    fn parse_binary(&mut self, minimum: u8, allow_command_call: bool) -> Option<Expression> {
        let mut left = self.parse_prefix()?;
        left = self.parse_postfix(left)?;

        if allow_command_call && self.argument_starts_here() {
            left = self.parse_command_call(left)?;
        }

        while let Some((operator, precedence)) = binary_operator(self.current().kind) {
            if precedence < minimum {
                break;
            }

            self.bump();
            let right = self.parse_binary(precedence + 1, allow_command_call)?;
            let span = Span::new(left.span.start, right.span.end);
            left = Expression::new(
                ExpressionKind::Binary { operator, left: Box::new(left), right: Box::new(right) },
                span,
            );
        }

        Some(left)
    }

    fn parse_prefix(&mut self) -> Option<Expression> {
        match self.current().kind {
            TokenKind::Return => self.parse_return(),
            TokenKind::Fail => self.parse_fail(),
            TokenKind::Try => self.parse_try(),
            TokenKind::If => self.parse_if(),
            TokenKind::Plus | TokenKind::Minus | TokenKind::Not => self.parse_unary(),
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Option<Expression> {
        let token = self.current();
        let kind = match token.kind {
            TokenKind::Identifier => ExpressionKind::Name(token.text(self.source).to_owned()),
            TokenKind::Integer => ExpressionKind::Integer(self.parse_integer(token)?),
            TokenKind::Float => ExpressionKind::Float(self.parse_float(token)?),
            TokenKind::String => ExpressionKind::String(self.parse_string(token)?),
            TokenKind::True => ExpressionKind::Bool(true),
            TokenKind::False => ExpressionKind::Bool(false),
            TokenKind::LParen => return self.parse_group(),
            _ => {
                self.error_current("P0011", "expected an expression");
                return None;
            }
        };
        self.bump();
        Some(Expression::new(kind, token.span))
    }

    fn parse_group(&mut self) -> Option<Expression> {
        let opening = self.bump();
        let mut expression = self.parse_expression()?;
        let closing = self.expect(TokenKind::RParen, "P0011", "expected `)`")?;
        expression.span = Span::new(opening.span.start, closing.span.end);
        Some(expression)
    }

    fn parse_postfix(&mut self, mut expression: Expression) -> Option<Expression> {
        loop {
            if self.eat(TokenKind::Dot).is_some() {
                let name = self.expect_identifier("P0011", "expected a member name after `.`")?;
                let end = self.previous().span.end;
                let start = expression.span.start;
                expression = Expression::new(
                    ExpressionKind::Member { object: Box::new(expression), name },
                    Span::new(start, end),
                );
                continue;
            }

            if self.eat(TokenKind::LParen).is_some() {
                let start = expression.span.start;
                let arguments = self.parse_arguments(TokenKind::RParen, true)?;
                let end = self
                    .expect(TokenKind::RParen, "P0011", "expected `)` after arguments")?
                    .span
                    .end;
                expression = Expression::new(
                    ExpressionKind::Call { callee: Box::new(expression), arguments },
                    Span::new(start, end),
                );
                continue;
            }
            break;
        }
        Some(expression)
    }

    fn parse_command_call(&mut self, callee: Expression) -> Option<Expression> {
        let start = callee.span.start;
        let arguments = self.parse_arguments(TokenKind::Newline, false)?;
        let end = arguments.last().map_or(callee.span.end, |argument| argument.span.end);
        Some(Expression::new(
            ExpressionKind::Call { callee: Box::new(callee), arguments },
            Span::new(start, end),
        ))
    }

    fn parse_arguments(
        &mut self,
        closing: TokenKind,
        allow_nested_command: bool,
    ) -> Option<Vec<Argument>> {
        let mut arguments = Vec::new();
        if self.at(closing) {
            return Some(arguments);
        }

        loop {
            arguments.push(self.parse_argument(allow_nested_command)?);
            if self.eat(TokenKind::Comma).is_none() {
                break;
            }
            if self.at(closing) || self.at(TokenKind::Newline) {
                break;
            }
        }
        Some(arguments)
    }

    fn parse_argument(&mut self, allow_nested_command: bool) -> Option<Argument> {
        let start = self.current().span.start;
        let label = if self.at(TokenKind::Identifier) && self.nth(1).kind == TokenKind::Colon {
            let label = self.bump().text(self.source).to_owned();
            self.bump();
            Some(label)
        } else {
            None
        };
        let value = self.parse_binary(0, allow_nested_command)?;
        let span = Span::new(start, value.span.end);
        Some(Argument { label, value, span })
    }

    fn parse_unary(&mut self) -> Option<Expression> {
        let token = self.bump();
        let operator = match token.kind {
            TokenKind::Plus => UnaryOperator::Positive,
            TokenKind::Minus => UnaryOperator::Negative,
            TokenKind::Not => UnaryOperator::Not,
            _ => unreachable!(),
        };
        let operand = self.parse_binary(7, true)?;
        let span = Span::new(token.span.start, operand.span.end);
        Some(Expression::new(ExpressionKind::Unary { operator, operand: Box::new(operand) }, span))
    }

    fn parse_return(&mut self) -> Option<Expression> {
        let token = self.bump();
        let value =
            if self.at_line_end() { None } else { Some(Box::new(self.parse_expression()?)) };
        let end = value.as_ref().map_or(token.span.end, |value| value.span.end);
        Some(Expression::new(ExpressionKind::Return(value), Span::new(token.span.start, end)))
    }

    fn parse_fail(&mut self) -> Option<Expression> {
        let token = self.bump();
        if self.eat(TokenKind::With).is_none() {
            self.error_current("P0013", "expected `with` after `fail`");
        }
        if self.at_line_end() {
            self.error_current("P0013", "expected an error value after `fail with`");
            return None;
        }
        let error = self.parse_expression()?;
        let span = Span::new(token.span.start, error.span.end);
        Some(Expression::new(ExpressionKind::Fail(Box::new(error)), span))
    }

    fn parse_try(&mut self) -> Option<Expression> {
        let token = self.bump();
        self.expect(TokenKind::To, "P0011", "expected `to` after `try`");
        let expression = self.parse_expression()?;
        let span = Span::new(token.span.start, expression.span.end);
        Some(Expression::new(ExpressionKind::Try(Box::new(expression)), span))
    }

    fn parse_if(&mut self) -> Option<Expression> {
        let token = self.bump();
        let condition = self.parse_expression()?;
        self.expect(TokenKind::Newline, "P0011", "expected a newline after the condition");
        let then_block = self.parse_block()?;
        let else_block = if self.eat(TokenKind::Else).is_some() {
            self.expect(TokenKind::Newline, "P0011", "expected a newline after `else`");
            self.parse_block()
        } else {
            None
        };
        let end = else_block.as_ref().map_or(then_block.span.end, |block| block.span.end);
        Some(Expression::new(
            ExpressionKind::If { condition: Box::new(condition), then_block, else_block },
            Span::new(token.span.start, end),
        ))
    }

    fn parse_catches(&mut self, expression: Expression) -> Option<Expression> {
        if !self.at(TokenKind::Catch) {
            return Some(expression);
        }

        let start = expression.span.start;
        let mut arms = Vec::new();
        while let Some(catch) = self.eat(TokenKind::Catch) {
            let pattern = self.parse_catch_pattern(catch.span)?;
            self.expect(TokenKind::Newline, "P0012", "expected a newline after catch pattern");
            let body = self.parse_block()?;
            let span = Span::new(catch.span.start, body.span.end);
            arms.push(CatchArm { pattern, body, span });
        }

        let end = arms.last().map_or(expression.span.end, |arm| arm.span.end);
        Some(Expression::new(
            ExpressionKind::Catch { expression: Box::new(expression), arms },
            Span::new(start, end),
        ))
    }

    fn parse_catch_pattern(&mut self, catch_span: Span) -> Option<Pattern> {
        if self.at_line_end() {
            self.error("P0012", "`catch` requires a pattern", catch_span);
            return None;
        }
        let first = self.expect(TokenKind::Identifier, "P0012", "expected a catch pattern")?;
        let mut bindings = Vec::new();
        while self.at(TokenKind::Identifier) {
            bindings.push(self.bump().text(self.source).to_owned());
        }
        let end = self.previous().span.end;
        Some(Pattern {
            name: first.text(self.source).to_owned(),
            bindings,
            span: Span::new(first.span.start, end),
        })
    }

    fn parse_integer(&mut self, token: crate::lexer::Token) -> Option<i128> {
        match token.text(self.source).parse() {
            Ok(value) => Some(value),
            Err(_) => {
                self.error("P0014", "integer literal is too large", token.span);
                None
            }
        }
    }

    fn parse_float(&mut self, token: crate::lexer::Token) -> Option<f64> {
        match token.text(self.source).parse() {
            Ok(value) => Some(value),
            Err(_) => {
                self.error("P0014", "invalid floating-point literal", token.span);
                None
            }
        }
    }

    fn parse_string(&mut self, token: crate::lexer::Token) -> Option<String> {
        match decode_string(token.text(self.source)) {
            Some(value) => Some(value),
            None => {
                self.error("P0015", "unsupported string escape", token.span);
                None
            }
        }
    }

    fn argument_starts_here(&self) -> bool {
        matches!(
            self.current().kind,
            TokenKind::Identifier
                | TokenKind::Integer
                | TokenKind::Float
                | TokenKind::String
                | TokenKind::True
                | TokenKind::False
                | TokenKind::LParen
                | TokenKind::If
                | TokenKind::Try
                | TokenKind::Not
        )
    }

    fn at_line_end(&self) -> bool {
        matches!(self.current().kind, TokenKind::Newline | TokenKind::Dedent | TokenKind::Eof)
    }
}

fn binary_operator(kind: TokenKind) -> Option<(BinaryOperator, u8)> {
    let pair = match kind {
        TokenKind::Or => (BinaryOperator::Or, 1),
        TokenKind::And => (BinaryOperator::And, 2),
        TokenKind::EqualEqual => (BinaryOperator::Equal, 3),
        TokenKind::BangEqual => (BinaryOperator::NotEqual, 3),
        TokenKind::Less => (BinaryOperator::Less, 3),
        TokenKind::LessEqual => (BinaryOperator::LessEqual, 3),
        TokenKind::Greater => (BinaryOperator::Greater, 3),
        TokenKind::GreaterEqual => (BinaryOperator::GreaterEqual, 3),
        TokenKind::Plus => (BinaryOperator::Add, 4),
        TokenKind::Minus => (BinaryOperator::Subtract, 4),
        TokenKind::At => (BinaryOperator::Matmul, 5),
        TokenKind::Star => (BinaryOperator::Multiply, 6),
        TokenKind::Slash => (BinaryOperator::Divide, 6),
        TokenKind::Percent => (BinaryOperator::Remainder, 6),
        _ => return None,
    };
    Some(pair)
}

fn decode_string(source: &str) -> Option<String> {
    let mut value = String::new();
    let mut characters = source[1..source.len() - 1].chars();
    while let Some(character) = characters.next() {
        if character != '\\' {
            value.push(character);
            continue;
        }

        match characters.next() {
            Some('n') => value.push('\n'),
            Some('r') => value.push('\r'),
            Some('t') => value.push('\t'),
            Some('"') => value.push('"'),
            Some('\\') => value.push('\\'),
            Some(_) | None => return None,
        }
    }
    Some(value)
}
