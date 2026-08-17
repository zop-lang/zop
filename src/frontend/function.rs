//! Per-function name resolution and type checking.
//!
//! Module signatures enter this pass and typed expressions leave it.

use std::collections::HashMap;

use crate::{hir, source::Span, syntax};

use super::{
    checker::{Checker, Signature},
    literal,
};

#[derive(Clone, Copy)]
struct Local {
    id: hir::LocalId,
    ty: hir::Type,
}

pub(super) struct FunctionContext<'checker, 'syntax> {
    checker: &'checker mut Checker<'syntax>,
    signature: Signature,
    locals: HashMap<String, Local>,
    next_local: usize,
}

impl<'checker, 'syntax> FunctionContext<'checker, 'syntax> {
    pub(super) fn new(checker: &'checker mut Checker<'syntax>, signature: Signature) -> Self {
        let locals = signature
            .parameters
            .iter()
            .enumerate()
            .map(|(index, parameter)| {
                (parameter.name.clone(), Local { id: hir::LocalId(index), ty: parameter.ty })
            })
            .collect();
        let next_local = signature.parameters.len();
        Self { checker, signature, locals, next_local }
    }

    pub(super) fn check(&mut self, function: &syntax::Function) -> Option<hir::Function> {
        let body = self.check_block(&function.body, self.signature.result);
        let body_result =
            body.expressions.last().map_or(hir::Type::Unit, |expression| expression.ty);
        if body_result != hir::Type::Never && body_result != self.signature.result {
            let message =
                format!("function returns {body_result:?}, expected {:?}", self.signature.result);
            self.checker.error("S0004", message, function.body.span);
        }

        let parameters = self
            .signature
            .parameters
            .iter()
            .enumerate()
            .map(|(index, parameter)| hir::Parameter {
                id: hir::LocalId(index),
                mode: parameter.mode,
                name: parameter.name.clone(),
                ty: parameter.ty,
                span: parameter.span,
            })
            .collect();

        Some(hir::Function {
            id: self.signature.id,
            kind: self.signature.kind,
            name: self.signature.name.clone(),
            parameters,
            result: self.signature.result,
            body,
            span: function.span,
        })
    }

    fn check_block(&mut self, block: &syntax::Block, expected: hir::Type) -> hir::Block {
        if let Some(expression) = block.expressions.windows(2).find_map(|expressions| {
            matches!(expressions[0].kind, syntax::ExpressionKind::Return(_))
                .then_some(&expressions[1])
        }) {
            self.checker.error("S0008", "expression follows return", expression.span);
        }
        let last = block.expressions.len().saturating_sub(1);
        let expressions = block
            .expressions
            .iter()
            .enumerate()
            .filter_map(|(index, expression)| {
                self.check_expression_as(expression, (index == last).then_some(expected))
            })
            .collect();
        hir::Block { expressions, span: block.span }
    }

    fn check_expression(&mut self, expression: &syntax::Expression) -> Option<hir::Expression> {
        self.check_expression_as(expression, None)
    }

    fn check_expression_as(
        &mut self,
        expression: &syntax::Expression,
        expected: Option<hir::Type>,
    ) -> Option<hir::Expression> {
        match &expression.kind {
            syntax::ExpressionKind::Name(name) => self.check_name(name, expression.span),
            syntax::ExpressionKind::Integer(value) => {
                self.check_integer(*value, expected, expression.span)
            }
            syntax::ExpressionKind::Float(value) => {
                self.check_float(*value, expected, expression.span)
            }
            syntax::ExpressionKind::Bool(value) => Some(hir::Expression::new(
                hir::ExpressionKind::Bool(*value),
                hir::Type::Bool,
                expression.span,
            )),
            syntax::ExpressionKind::String(value) => Some(hir::Expression::new(
                hir::ExpressionKind::String(value.clone()),
                hir::Type::String,
                expression.span,
            )),
            syntax::ExpressionKind::Assign { target, value } => {
                self.check_assignment(target, value, expression.span)
            }
            syntax::ExpressionKind::Unary { operator, operand } => {
                self.check_unary(*operator, operand, expected, expression.span)
            }
            syntax::ExpressionKind::Binary { operator, left, right } => {
                self.check_binary(*operator, left, right, expected, expression.span)
            }
            syntax::ExpressionKind::Call { callee, arguments } => {
                self.check_call(callee, arguments, expression.span)
            }
            syntax::ExpressionKind::Return(value) => {
                self.check_return(value.as_deref(), expression.span)
            }
            _ => {
                self.checker.error(
                    "S0001",
                    "expression is not in the scalar frontend slice",
                    expression.span,
                );
                None
            }
        }
    }

    fn check_integer(
        &mut self,
        value: i128,
        expected: Option<hir::Type>,
        span: Span,
    ) -> Option<hir::Expression> {
        let ty = match literal::integer_type(value, expected) {
            Ok(ty) => ty,
            Err(error) => {
                self.checker.error("S0009", error.message(), span);
                return None;
            }
        };
        Some(hir::Expression::new(hir::ExpressionKind::Integer(value), ty, span))
    }

    fn check_float(
        &mut self,
        value: f64,
        expected: Option<hir::Type>,
        span: Span,
    ) -> Option<hir::Expression> {
        let ty = match literal::float_type(value, expected) {
            Ok(ty) => ty,
            Err(error) => {
                self.checker.error("S0009", error.message(), span);
                return None;
            }
        };
        Some(hir::Expression::new(hir::ExpressionKind::Float(value), ty, span))
    }

    fn check_name(&mut self, name: &str, span: Span) -> Option<hir::Expression> {
        let Some(local) = self.locals.get(name).copied() else {
            self.checker.error("S0002", format!("unknown name `{name}`"), span);
            return None;
        };
        Some(hir::Expression::new(hir::ExpressionKind::Local(local.id), local.ty, span))
    }

    fn check_assignment(
        &mut self,
        target: &syntax::Expression,
        value: &syntax::Expression,
        span: Span,
    ) -> Option<hir::Expression> {
        let syntax::ExpressionKind::Name(name) = &target.kind else {
            self.checker.error("S0001", "assignment target must be a name", target.span);
            return None;
        };
        let existing = self.locals.get(name).copied();
        let value = self.check_expression_as(value, existing.map(|local| local.ty))?;
        let local = if let Some(local) = existing {
            if local.ty != value.ty {
                self.checker.error("S0005", "assignment cannot change a binding's type", span);
            }
            local
        } else {
            let local = Local { id: hir::LocalId(self.next_local), ty: value.ty };
            self.next_local += 1;
            self.locals.insert(name.clone(), local);
            local
        };
        Some(hir::Expression::new(
            hir::ExpressionKind::Assign { local: local.id, value: Box::new(value) },
            hir::Type::Unit,
            span,
        ))
    }

    fn check_unary(
        &mut self,
        operator: syntax::UnaryOperator,
        operand: &syntax::Expression,
        expected: Option<hir::Type>,
        span: Span,
    ) -> Option<hir::Expression> {
        if operator == syntax::UnaryOperator::Negative
            && let syntax::ExpressionKind::Integer(value) = &operand.kind
        {
            let Some(value) = value.checked_neg() else {
                self.checker.error("S0009", "integer literal is outside the supported range", span);
                return None;
            };
            return self.check_integer(value, expected.filter(|ty| ty.is_numeric()), span);
        }
        let operand_expected = match operator {
            syntax::UnaryOperator::Not => Some(hir::Type::Bool),
            syntax::UnaryOperator::Positive | syntax::UnaryOperator::Negative => {
                expected.filter(|ty| ty.is_numeric())
            }
        };
        let operand = self.check_expression_as(operand, operand_expected)?;
        let valid = match operator {
            syntax::UnaryOperator::Not => operand.ty == hir::Type::Bool,
            syntax::UnaryOperator::Positive | syntax::UnaryOperator::Negative => {
                operand.ty.is_numeric()
            }
        };
        if !valid {
            self.checker.error("S0004", "unary operator has an incompatible operand", span);
        }
        let ty = operand.ty;
        Some(hir::Expression::new(
            hir::ExpressionKind::Unary {
                operator: unary_operator(operator),
                operand: Box::new(operand),
            },
            ty,
            span,
        ))
    }

    fn check_binary(
        &mut self,
        operator: syntax::BinaryOperator,
        left: &syntax::Expression,
        right: &syntax::Expression,
        expected: Option<hir::Type>,
        span: Span,
    ) -> Option<hir::Expression> {
        let expected =
            expected.filter(|ty| ty.is_numeric()).filter(|_| arithmetic_result(operator));
        let left_is_literal = is_numeric_literal_expression(left);
        let right_is_literal = is_numeric_literal_expression(right);
        let (left, right) = if let Some(expected) = expected {
            (
                self.check_expression_as(left, Some(expected))?,
                self.check_expression_as(right, Some(expected))?,
            )
        } else if left_is_literal && !right_is_literal {
            let right = self.check_expression(right)?;
            (self.check_expression_as(left, Some(right.ty))?, right)
        } else if !left_is_literal && right_is_literal {
            let left = self.check_expression(left)?;
            let right = self.check_expression_as(right, Some(left.ty))?;
            (left, right)
        } else {
            (self.check_expression(left)?, self.check_expression(right)?)
        };
        let Some((operator, ty)) = binary_type(operator, left.ty, right.ty) else {
            self.checker.error("S0004", "binary operands have incompatible types", span);
            return None;
        };
        Some(hir::Expression::new(
            hir::ExpressionKind::Binary { operator, left: Box::new(left), right: Box::new(right) },
            ty,
            span,
        ))
    }

    fn check_call(
        &mut self,
        callee: &syntax::Expression,
        arguments: &[syntax::Argument],
        span: Span,
    ) -> Option<hir::Expression> {
        let syntax::ExpressionKind::Name(name) = &callee.kind else {
            self.checker.error("S0001", "scalar calls require a known function", callee.span);
            return None;
        };
        let Some(id) = self.checker.functions.get(name).copied() else {
            self.checker.error("S0002", format!("unknown function `{name}`"), callee.span);
            return None;
        };
        let signature = self.checker.signature(id);
        let arguments = self.check_arguments(arguments, &signature)?;
        Some(hir::Expression::new(
            hir::ExpressionKind::Call { function: id, arguments },
            signature.result,
            span,
        ))
    }

    fn check_arguments(
        &mut self,
        arguments: &[syntax::Argument],
        signature: &Signature,
    ) -> Option<Vec<hir::Expression>> {
        let mut ordered = vec![None; signature.parameters.len()];
        let mut positional = 0;
        let mut saw_label = false;

        for argument in arguments {
            let index = self.argument_index(
                argument.label.as_deref(),
                signature,
                &mut positional,
                &mut saw_label,
                argument.span,
            );
            let expected = index
                .filter(|index| *index < signature.parameters.len())
                .map(|index| signature.parameters[index].ty);
            let Some(value) = self.check_expression_as(&argument.value, expected) else {
                continue;
            };
            let Some(index) = index.filter(|index| *index < ordered.len()) else {
                self.checker.error("S0006", "argument does not match a parameter", argument.span);
                continue;
            };
            if ordered[index].is_some() {
                self.checker.error("S0007", "parameter is supplied more than once", argument.span);
                continue;
            }
            if value.ty != signature.parameters[index].ty {
                self.checker.error(
                    "S0006",
                    "argument type does not match parameter",
                    argument.span,
                );
            }
            ordered[index] = Some(value);
        }

        if ordered.iter().any(Option::is_none) {
            self.checker.error("S0006", "call is missing a required argument", Span::default());
            return None;
        }
        Some(ordered.into_iter().flatten().collect())
    }

    fn argument_index(
        &mut self,
        label: Option<&str>,
        signature: &Signature,
        positional: &mut usize,
        saw_label: &mut bool,
        span: Span,
    ) -> Option<usize> {
        match label {
            Some(label) => {
                *saw_label = true;
                signature.parameters.iter().position(|parameter| parameter.name == label)
            }
            None if *saw_label => {
                self.checker.error("S0007", "positional argument follows a named argument", span);
                None
            }
            None => {
                let index = *positional;
                *positional += 1;
                Some(index)
            }
        }
    }

    fn check_return(
        &mut self,
        value: Option<&syntax::Expression>,
        span: Span,
    ) -> Option<hir::Expression> {
        let value = value.and_then(|value| {
            self.check_expression_as(value, Some(self.signature.result)).map(Box::new)
        });
        let ty = value.as_ref().map_or(hir::Type::Unit, |value| value.ty);
        if ty != self.signature.result {
            self.checker.error("S0004", "return value has the wrong type", span);
        }
        Some(hir::Expression::new(hir::ExpressionKind::Return(value), hir::Type::Never, span))
    }
}

fn unary_operator(operator: syntax::UnaryOperator) -> hir::UnaryOperator {
    match operator {
        syntax::UnaryOperator::Positive => hir::UnaryOperator::Positive,
        syntax::UnaryOperator::Negative => hir::UnaryOperator::Negative,
        syntax::UnaryOperator::Not => hir::UnaryOperator::Not,
    }
}

fn arithmetic_result(operator: syntax::BinaryOperator) -> bool {
    matches!(
        operator,
        syntax::BinaryOperator::Add
            | syntax::BinaryOperator::Subtract
            | syntax::BinaryOperator::Multiply
            | syntax::BinaryOperator::Divide
            | syntax::BinaryOperator::Remainder
    )
}

fn is_numeric_literal_expression(expression: &syntax::Expression) -> bool {
    match &expression.kind {
        syntax::ExpressionKind::Integer(_) | syntax::ExpressionKind::Float(_) => true,
        syntax::ExpressionKind::Unary { operator, operand } => {
            matches!(operator, syntax::UnaryOperator::Positive | syntax::UnaryOperator::Negative)
                && is_numeric_literal_expression(operand)
        }
        syntax::ExpressionKind::Binary { operator, left, right } => {
            arithmetic_result(*operator)
                && is_numeric_literal_expression(left)
                && is_numeric_literal_expression(right)
        }
        _ => false,
    }
}

fn binary_type(
    operator: syntax::BinaryOperator,
    left: hir::Type,
    right: hir::Type,
) -> Option<(hir::BinaryOperator, hir::Type)> {
    use syntax::BinaryOperator as S;
    let hir_operator = match operator {
        S::Or => hir::BinaryOperator::Or,
        S::And => hir::BinaryOperator::And,
        S::Equal => hir::BinaryOperator::Equal,
        S::NotEqual => hir::BinaryOperator::NotEqual,
        S::Less => hir::BinaryOperator::Less,
        S::LessEqual => hir::BinaryOperator::LessEqual,
        S::Greater => hir::BinaryOperator::Greater,
        S::GreaterEqual => hir::BinaryOperator::GreaterEqual,
        S::Add => hir::BinaryOperator::Add,
        S::Subtract => hir::BinaryOperator::Subtract,
        S::Multiply => hir::BinaryOperator::Multiply,
        S::Divide => hir::BinaryOperator::Divide,
        S::Remainder => hir::BinaryOperator::Remainder,
        S::Matmul => return None,
    };
    let comparison = matches!(
        operator,
        S::Equal | S::NotEqual | S::Less | S::LessEqual | S::Greater | S::GreaterEqual
    );
    let boolean = matches!(operator, S::And | S::Or);

    if left != right || (boolean && left != hir::Type::Bool) {
        return None;
    }
    if !comparison && !boolean && !left.is_numeric() {
        return None;
    }
    Some((hir_operator, if comparison { hir::Type::Bool } else { left }))
}
