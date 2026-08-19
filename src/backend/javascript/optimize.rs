// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: MIT

//! Pure scalar folding while high-level intermediate representation still
//! carries exact source types.

use crate::hir;

/// Pure scalar constant retained while its exact HIR type remains known.
#[derive(Debug, PartialEq)]
pub(super) enum Constant {
    /// Exact signed 32-bit integer value.
    I32(
        /// Wrapped bootstrap result with JavaScript `i32` representation.
        i32,
    ),

    /// Finite binary64 value safe to print as one JavaScript number token.
    F64(
        /// Floating-point value after typed constant evaluation.
        f64,
    ),

    /// Boolean value.
    Bool(
        /// Result of literal or comparison folding.
        bool,
    ),

    /// Decoded string value.
    String(
        /// Unicode scalar sequence before JavaScript escaping.
        String,
    ),
}

/// Fold one proven-pure expression without losing its checked scalar type.
pub(super) fn fold(expression: &hir::Expression) -> Option<Constant> {
    use hir::ExpressionKind as E;
    match &expression.kind {
        E::Integer(value) => fold_integer(*value, expression.ty),
        E::Float(value) if expression.ty == hir::Type::F64 && value.is_finite() => {
            Some(Constant::F64(*value))
        }
        E::Bool(value) => Some(Constant::Bool(*value)),
        E::String(value) => Some(Constant::String(value.clone())),
        E::Unary { operator, operand } => fold_unary(*operator, fold(operand)?),
        E::Binary { operator, left, right } => fold_binary(*operator, fold(left)?, fold(right)?),
        E::Local(_) | E::Assign { .. } | E::Call { .. } | E::Return(_) => None,
        E::Float(_) => None,
    }
}

fn fold_integer(value: i128, ty: hir::Type) -> Option<Constant> {
    match ty {
        hir::Type::I32 => i32::try_from(value).ok().map(Constant::I32),
        hir::Type::F64 => {
            let converted = value as f64;
            (converted as i128 == value).then_some(Constant::F64(converted))
        }
        _ => None,
    }
}

fn fold_unary(operator: hir::UnaryOperator, operand: Constant) -> Option<Constant> {
    use hir::UnaryOperator as U;
    match (operator, operand) {
        (U::Positive, value @ (Constant::I32(_) | Constant::F64(_))) => Some(value),
        (U::Negative, Constant::I32(value)) => Some(Constant::I32(value.wrapping_neg())),
        (U::Negative, Constant::F64(value)) => Some(Constant::F64(-value)),
        (U::Not, Constant::Bool(value)) => Some(Constant::Bool(!value)),
        _ => None,
    }
}

fn fold_binary(operator: hir::BinaryOperator, left: Constant, right: Constant) -> Option<Constant> {
    fold_i32(operator, &left, &right)
        .or_else(|| fold_f64(operator, &left, &right))
        .or_else(|| fold_bool(operator, &left, &right))
        .or_else(|| fold_string(operator, &left, &right))
}

fn fold_i32(operator: hir::BinaryOperator, left: &Constant, right: &Constant) -> Option<Constant> {
    let (Constant::I32(left), Constant::I32(right)) = (left, right) else {
        return None;
    };
    use hir::BinaryOperator as B;
    match operator {
        B::Add => Some(Constant::I32(left.wrapping_add(*right))),
        B::Subtract => Some(Constant::I32(left.wrapping_sub(*right))),
        B::Multiply => Some(Constant::I32(left.wrapping_mul(*right))),
        B::Equal => Some(Constant::Bool(left == right)),
        B::NotEqual => Some(Constant::Bool(left != right)),
        B::Less => Some(Constant::Bool(left < right)),
        B::LessEqual => Some(Constant::Bool(left <= right)),
        B::Greater => Some(Constant::Bool(left > right)),
        B::GreaterEqual => Some(Constant::Bool(left >= right)),
        _ => None,
    }
}

fn fold_f64(operator: hir::BinaryOperator, left: &Constant, right: &Constant) -> Option<Constant> {
    let (Constant::F64(left), Constant::F64(right)) = (left, right) else {
        return None;
    };
    use hir::BinaryOperator as B;
    let value = match operator {
        B::Add => Constant::F64(left + right),
        B::Subtract => Constant::F64(left - right),
        B::Multiply => Constant::F64(left * right),
        B::Divide => Constant::F64(left / right),
        B::Remainder => Constant::F64(left % right),
        B::Equal => Constant::Bool(left == right),
        B::NotEqual => Constant::Bool(left != right),
        B::Less => Constant::Bool(left < right),
        B::LessEqual => Constant::Bool(left <= right),
        B::Greater => Constant::Bool(left > right),
        B::GreaterEqual => Constant::Bool(left >= right),
        _ => return None,
    };
    match value {
        Constant::F64(value) if !value.is_finite() => None,
        value => Some(value),
    }
}

fn fold_bool(operator: hir::BinaryOperator, left: &Constant, right: &Constant) -> Option<Constant> {
    let (Constant::Bool(left), Constant::Bool(right)) = (left, right) else {
        return None;
    };
    use hir::BinaryOperator as B;
    match operator {
        B::Or => Some(Constant::Bool(*left || *right)),
        B::And => Some(Constant::Bool(*left && *right)),
        B::Equal => Some(Constant::Bool(left == right)),
        B::NotEqual => Some(Constant::Bool(left != right)),
        _ => None,
    }
}

fn fold_string(
    operator: hir::BinaryOperator,
    left: &Constant,
    right: &Constant,
) -> Option<Constant> {
    let (Constant::String(left), Constant::String(right)) = (left, right) else {
        return None;
    };
    match operator {
        hir::BinaryOperator::Equal => Some(Constant::Bool(left == right)),
        hir::BinaryOperator::NotEqual => Some(Constant::Bool(left != right)),
        _ => None,
    }
}
