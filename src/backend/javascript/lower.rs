// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: MIT

//! Typed high-level intermediate representation to target JavaScript structure.
//!
//! Target placement stops kernels and integer widths without an exact, cheap
//! JavaScript representation before the printer can invent weaker semantics.

use std::collections::HashSet;

use crate::hir;

use super::{
    super::error::{BackendResult, lowering_error},
    ast,
    optimize::{self, Constant},
};

/// Lower a checked host module into the deterministic JavaScript AST.
pub(super) fn lower(module: &hir::Module) -> BackendResult<ast::Module> {
    let mut functions = Vec::with_capacity(module.functions.len());
    let mut exports = Vec::with_capacity(module.functions.len());

    for (index, function) in module.functions.iter().enumerate() {
        if function.id.0 != index {
            return Err(lowering_error("J0003", "HIR function identifiers are out of order"));
        }
        let local = function_name(function.id);
        functions.push(FunctionLowerer::new(module, function)?.lower()?);
        exports.push(ast::Export { local, exported: function.name.clone() });
    }

    Ok(ast::Module { functions, exports })
}

/// Stateful lowering for one host function and its local bindings.
struct FunctionLowerer<'hir> {
    /// Complete HIR module used to resolve direct callees.
    module: &'hir hir::Module,

    /// Function whose expressions are being lowered.
    function: &'hir hir::Function,

    /// Local identities that already have a JavaScript binding.
    defined: HashSet<hir::LocalId>,

    /// First unused scratch binding for reordered call arguments.
    next_temporary: usize,
}

impl<'hir> FunctionLowerer<'hir> {
    fn new(module: &'hir hir::Module, function: &'hir hir::Function) -> BackendResult<Self> {
        if function.kind != hir::FunctionKind::Host {
            return Err(lowering_error("J0001", "GPU kernels cannot lower to JavaScript"));
        }
        supported_type(function.result)?;
        let mut defined = HashSet::with_capacity(function.parameters.len());
        for parameter in &function.parameters {
            supported_type(parameter.ty)?;
            if !defined.insert(parameter.id) {
                return Err(lowering_error("J0003", "HIR parameter identifiers are not unique"));
            }
        }
        Ok(Self { module, function, defined, next_temporary: 0 })
    }

    fn lower(mut self) -> BackendResult<ast::Function> {
        let parameters =
            self.function.parameters.iter().map(|parameter| local_name(parameter.id)).collect();
        let body = self.lower_body()?;
        Ok(ast::Function {
            name: function_name(self.function.id),
            parameters,
            temporary_count: self.next_temporary,
            body,
        })
    }

    fn lower_body(&mut self) -> BackendResult<Vec<ast::Statement>> {
        let mut statements = Vec::new();
        let last = self.function.body.expressions.len().saturating_sub(1);
        for (index, expression) in self.function.body.expressions.iter().enumerate() {
            if index == last && self.function.result != hir::Type::Unit {
                statements.push(self.lower_return(expression)?);
            } else if let Some(statement) = self.lower_statement(expression)? {
                statements.push(statement);
            }
        }
        Ok(statements)
    }

    fn lower_return(&mut self, expression: &hir::Expression) -> BackendResult<ast::Statement> {
        match &expression.kind {
            hir::ExpressionKind::Return(value) => value
                .as_deref()
                .map(|value| self.lower_expression(value))
                .transpose()
                .map(ast::Statement::Return),
            _ => self.lower_expression(expression).map(|value| ast::Statement::Return(Some(value))),
        }
    }

    fn lower_statement(
        &mut self,
        expression: &hir::Expression,
    ) -> BackendResult<Option<ast::Statement>> {
        match &expression.kind {
            hir::ExpressionKind::Assign { local, value } => {
                let value = self.lower_expression(value)?;
                let name = local_name(*local);
                if self.defined.insert(*local) {
                    Ok(Some(ast::Statement::Let { name, value }))
                } else {
                    Ok(Some(ast::Statement::Assign { name, value }))
                }
            }
            hir::ExpressionKind::Call { .. } => {
                self.lower_expression(expression).map(ast::Statement::Expression).map(Some)
            }
            hir::ExpressionKind::Return(value) => value
                .as_deref()
                .map(|value| self.lower_expression(value))
                .transpose()
                .map(ast::Statement::Return)
                .map(Some),
            _ if may_have_effect(expression) => {
                self.lower_expression(expression).map(ast::Statement::Expression).map(Some)
            }
            _ => Ok(None),
        }
    }

    fn lower_expression(&mut self, expression: &hir::Expression) -> BackendResult<ast::Expression> {
        supported_type(expression.ty)?;
        if let Some(constant) = optimize::fold(expression) {
            return Ok(lower_constant(constant));
        }
        match &expression.kind {
            hir::ExpressionKind::Local(id) => self.lower_local(*id),
            hir::ExpressionKind::Integer(value) => lower_integer(*value, expression.ty),
            hir::ExpressionKind::Float(value) => lower_float(*value, expression.ty),
            hir::ExpressionKind::Bool(value) => Ok(ast::Expression::Bool(*value)),
            hir::ExpressionKind::String(value) => Ok(ast::Expression::String(value.clone())),
            hir::ExpressionKind::Unary { operator, operand } => {
                self.lower_unary(*operator, operand, expression.ty)
            }
            hir::ExpressionKind::Binary { operator, left, right } => {
                self.lower_binary(*operator, left, right, expression.ty)
            }
            hir::ExpressionKind::Call { function, arguments } => {
                self.lower_call(*function, arguments)
            }
            hir::ExpressionKind::Assign { .. } | hir::ExpressionKind::Return(_) => {
                Err(lowering_error("J0003", "statement reached JavaScript expression lowering"))
            }
        }
    }

    fn lower_local(&self, id: hir::LocalId) -> BackendResult<ast::Expression> {
        if !self.defined.contains(&id) {
            return Err(lowering_error("J0003", "HIR references an undefined local"));
        }
        Ok(ast::Expression::Identifier(local_name(id)))
    }

    fn lower_unary(
        &mut self,
        operator: hir::UnaryOperator,
        operand: &hir::Expression,
        ty: hir::Type,
    ) -> BackendResult<ast::Expression> {
        let operand = self.lower_expression(operand)?;
        match operator {
            hir::UnaryOperator::Positive => Ok(operand),
            hir::UnaryOperator::Negative if ty == hir::Type::I32 => {
                Ok(wrap_i32(ast::Expression::Unary {
                    operator: ast::UnaryOperator::Negative,
                    operand: Box::new(operand),
                }))
            }
            hir::UnaryOperator::Negative => Ok(ast::Expression::Unary {
                operator: ast::UnaryOperator::Negative,
                operand: Box::new(operand),
            }),
            hir::UnaryOperator::Not => Ok(ast::Expression::Unary {
                operator: ast::UnaryOperator::Not,
                operand: Box::new(operand),
            }),
        }
    }

    fn lower_binary(
        &mut self,
        operator: hir::BinaryOperator,
        left: &hir::Expression,
        right: &hir::Expression,
        ty: hir::Type,
    ) -> BackendResult<ast::Expression> {
        let left = self.lower_expression(left)?;
        let right = self.lower_expression(right)?;
        if ty == hir::Type::I32 {
            return lower_i32_binary(operator, left, right);
        }
        let operator = binary_operator(operator);
        Ok(ast::Expression::Binary { operator, left: Box::new(left), right: Box::new(right) })
    }

    fn lower_call(
        &mut self,
        function: hir::FunctionId,
        arguments: &[hir::CallArgument],
    ) -> BackendResult<ast::Expression> {
        let Some(callee) = self.module.functions.get(function.0) else {
            return Err(lowering_error("J0003", "HIR references an unknown function"));
        };
        let parameter_count = callee.parameters.len();
        let callee = function_name(callee.id);
        let mut evaluated = Vec::with_capacity(arguments.len());
        let mut supplied = vec![false; parameter_count];
        for argument in arguments {
            let Some(slot) = supplied.get_mut(argument.parameter.0) else {
                return Err(lowering_error(
                    "J0003",
                    "call argument references an unknown parameter",
                ));
            };
            if *slot {
                return Err(lowering_error("J0003", "call supplies one parameter more than once"));
            }
            *slot = true;
            evaluated.push((argument.parameter, self.lower_expression(&argument.value)?));
        }
        if supplied.iter().any(|supplied| !supplied) {
            return Err(lowering_error("J0003", "call is missing a parameter value"));
        }

        if evaluated.iter().enumerate().all(|(index, (parameter, _))| parameter.0 == index) {
            let arguments = evaluated.into_iter().map(|(_, value)| value).collect();
            return Ok(ast::Expression::Call { function: callee, arguments });
        }

        let mut sequence = Vec::with_capacity(evaluated.len() + 1);
        let mut ordered = std::iter::repeat_with(|| None).take(parameter_count).collect::<Vec<_>>();
        for (parameter, value) in evaluated {
            let name = temporary_name(self.next_temporary);
            self.next_temporary += 1;
            sequence.push(ast::Expression::Set { name: name.clone(), value: Box::new(value) });
            ordered[parameter.0] = Some(ast::Expression::Identifier(name));
        }
        let arguments = ordered.into_iter().flatten().collect();
        sequence.push(ast::Expression::Call { function: callee, arguments });
        Ok(ast::Expression::Sequence(sequence))
    }
}

fn lower_integer(value: i128, ty: hir::Type) -> BackendResult<ast::Expression> {
    match ty {
        hir::Type::I32 => {
            let value = i32::try_from(value).map_err(|_| {
                lowering_error("J0003", "i32 literal is outside its represented range")
            })?;
            Ok(ast::Expression::Number(value.to_string()))
        }
        hir::Type::F64 => {
            let converted = value as f64;
            if converted as i128 != value {
                return Err(lowering_error("J0003", "f64 integer literal is not exact"));
            }
            Ok(ast::Expression::Number(format_float(converted)))
        }
        _ => Err(unsupported_type(ty)),
    }
}

fn lower_float(value: f64, ty: hir::Type) -> BackendResult<ast::Expression> {
    if ty != hir::Type::F64 {
        return Err(unsupported_type(ty));
    }
    if !value.is_finite() {
        return Err(lowering_error("J0003", "f64 literal is not finite"));
    }
    Ok(ast::Expression::Number(format_float(value)))
}

fn lower_constant(constant: Constant) -> ast::Expression {
    match constant {
        Constant::I32(value) => ast::Expression::Number(value.to_string()),
        Constant::F64(value) => ast::Expression::Number(format_float(value)),
        Constant::Bool(value) => ast::Expression::Bool(value),
        Constant::String(value) => ast::Expression::String(value),
    }
}

fn format_float(value: f64) -> String {
    if value == 0.0 && value.is_sign_negative() {
        return "-0".to_owned();
    }
    let mut buffer = ryu_js::Buffer::new();
    buffer.format(value).to_owned()
}

fn lower_i32_binary(
    operator: hir::BinaryOperator,
    left: ast::Expression,
    right: ast::Expression,
) -> BackendResult<ast::Expression> {
    use hir::BinaryOperator as H;
    match operator {
        H::Multiply => Ok(ast::Expression::Call {
            function: "Math.imul".to_owned(),
            arguments: vec![left, right],
        }),
        H::Add | H::Subtract => Ok(wrap_i32(ast::Expression::Binary {
            operator: if operator == H::Add {
                ast::BinaryOperator::Add
            } else {
                ast::BinaryOperator::Subtract
            },
            left: Box::new(left),
            right: Box::new(right),
        })),
        H::Divide | H::Remainder => Err(lowering_error(
            "J0002",
            "i32 division semantics require a checked JavaScript lowering",
        )),
        _ => Ok(ast::Expression::Binary {
            operator: binary_operator(operator),
            left: Box::new(left),
            right: Box::new(right),
        }),
    }
}

fn wrap_i32(expression: ast::Expression) -> ast::Expression {
    ast::Expression::Binary {
        operator: ast::BinaryOperator::BitOr,
        left: Box::new(expression),
        right: Box::new(ast::Expression::Number("0".to_owned())),
    }
}

fn binary_operator(operator: hir::BinaryOperator) -> ast::BinaryOperator {
    use hir::BinaryOperator as H;
    match operator {
        H::Or => ast::BinaryOperator::Or,
        H::And => ast::BinaryOperator::And,
        H::Equal => ast::BinaryOperator::StrictEqual,
        H::NotEqual => ast::BinaryOperator::StrictNotEqual,
        H::Less => ast::BinaryOperator::Less,
        H::LessEqual => ast::BinaryOperator::LessEqual,
        H::Greater => ast::BinaryOperator::Greater,
        H::GreaterEqual => ast::BinaryOperator::GreaterEqual,
        H::Add => ast::BinaryOperator::Add,
        H::Subtract => ast::BinaryOperator::Subtract,
        H::Multiply => ast::BinaryOperator::Multiply,
        H::Divide => ast::BinaryOperator::Divide,
        H::Remainder => ast::BinaryOperator::Remainder,
    }
}

fn supported_type(ty: hir::Type) -> BackendResult<()> {
    match ty {
        hir::Type::I32 | hir::Type::F64 | hir::Type::Bool | hir::Type::String | hir::Type::Unit => {
            Ok(())
        }
        _ => Err(unsupported_type(ty)),
    }
}

fn unsupported_type(ty: hir::Type) -> crate::diagnostic::Diagnostics {
    let message = match ty {
        hir::Type::I64 => "i64 requires a WebAssembly compute region",
        hir::Type::F32 => "f32 requires a representation-preserving JavaScript ABI",
        _ => "type is outside the initial JavaScript backend slice",
    };
    lowering_error("J0002", message)
}

fn function_name(id: hir::FunctionId) -> String {
    format!("b{}", id.0)
}

fn local_name(id: hir::LocalId) -> String {
    format!("l{}", id.0)
}

fn temporary_name(index: usize) -> String {
    format!("t{index}")
}

fn may_have_effect(expression: &hir::Expression) -> bool {
    match &expression.kind {
        hir::ExpressionKind::Call { .. }
        | hir::ExpressionKind::Assign { .. }
        | hir::ExpressionKind::Return(_) => true,
        hir::ExpressionKind::Unary { operand, .. } => may_have_effect(operand),
        hir::ExpressionKind::Binary { left, right, .. } => {
            may_have_effect(left) || may_have_effect(right)
        }
        hir::ExpressionKind::Local(_)
        | hir::ExpressionKind::Integer(_)
        | hir::ExpressionKind::Float(_)
        | hir::ExpressionKind::Bool(_)
        | hir::ExpressionKind::String(_) => false,
    }
}
