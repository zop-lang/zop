// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: MIT

//! Typed high-level intermediate representation to verified Multi-Level
//! Intermediate Representation (MLIR) lowering.
//!
//! Both native backends consume the in-memory module produced here.
//! Emission constructs functions first, fills verified entry blocks second,
//! then rejects the complete module if MLIR verification fails.

use std::collections::HashMap;

use melior::{
    Context,
    dialect::{DialectRegistry, arith, func},
    ir::{
        Block, Location, Module, Region, RegionLike, Type, Value,
        attribute::{FlatSymbolRefAttribute, IntegerAttribute, StringAttribute, TypeAttribute},
        block::BlockLike,
        operation::OperationLike,
        r#type::{FunctionType, IntegerType},
    },
    utility::register_all_dialects,
};

use crate::hir;

use super::error::{BackendResult, lowering_error};

/// Lower one typed module to verified textual MLIR.
///
/// # Errors
///
/// Returns a lowering diagnostic when the module contains a type, function
/// kind, expression, or invariant outside the implemented native scalar slice.
pub fn mlir_text(hir: &hir::Module) -> BackendResult<String> {
    with_module(hir, |module| Ok(module.as_operation().to_string()))
}

/// Build and verify one in-memory MLIR module before invoking a consumer.
pub(super) fn with_module<T>(
    hir: &hir::Module,
    compile: impl for<'context> FnOnce(&Module<'context>) -> BackendResult<T>,
) -> BackendResult<T> {
    let registry = DialectRegistry::new();
    register_all_dialects(&registry);
    let context = Context::new();
    context.append_dialect_registry(&registry);
    context.load_all_available_dialects();

    let location = Location::unknown(&context);
    let module = Module::new(location);
    for function in &hir.functions {
        module.body().append_operation(lower_function(&context, hir, function, location)?);
    }
    if !module.as_operation().verify() {
        return Err(lowering_error("M0002", "generated MLIR failed verification"));
    }
    compile(&module)
}

fn lower_function<'context>(
    context: &'context Context,
    module: &hir::Module,
    function: &hir::Function,
    location: Location<'context>,
) -> BackendResult<melior::ir::Operation<'context>> {
    require_scalar_signature(function)?;
    let integer = IntegerType::new(context, 64).into();
    let inputs = vec![integer; function.parameters.len()];
    let results = if function.result == hir::Type::Unit { vec![] } else { vec![integer] };
    let function_type = FunctionType::new(context, &inputs, &results);
    let block = Block::new(&inputs.iter().copied().map(|ty| (ty, location)).collect::<Vec<_>>());
    emit_body(context, module, function, &block)?;

    let region = Region::new();
    region.append_block(block);
    Ok(func::func(
        context,
        StringAttribute::new(context, &function.name),
        TypeAttribute::new(function_type.into()),
        region,
        &[],
        location,
    ))
}

fn require_scalar_signature(function: &hir::Function) -> BackendResult<()> {
    if function.kind != hir::FunctionKind::Host {
        return Err(lowering_error(
            "M0001",
            format!("kernel `{}` requires a GPU backend", function.name),
        ));
    }
    if function.parameters.iter().any(|parameter| parameter.ty != hir::Type::I64)
        || !matches!(function.result, hir::Type::I64 | hir::Type::Unit)
    {
        return Err(lowering_error(
            "M0001",
            format!("`{}` is outside the initial i64 backend slice", function.name),
        ));
    }
    Ok(())
}

fn emit_body<'context>(
    context: &'context Context,
    module: &hir::Module,
    function: &hir::Function,
    block: &Block<'context>,
) -> BackendResult<()> {
    let mut emitter = ExpressionEmitter::new(context, module, function, block)?;
    let expressions = &function.body.expressions;

    for (index, expression) in expressions.iter().enumerate() {
        let last = index + 1 == expressions.len();
        if let hir::ExpressionKind::Return(value) = &expression.kind {
            if !last {
                return Err(lowering_error("M0004", "expression follows return"));
            }
            let value = value.as_deref().map(|value| emitter.emit(value)).transpose()?.flatten();
            emitter.append_return(function, value)?;
            return Ok(());
        }

        let value = emitter.emit(expression)?;
        if last {
            emitter.append_return(function, value)?;
            return Ok(());
        }
    }

    emitter.append_return(function, None)
}

/// Per-function state for emitting checked expressions into one MLIR block.
struct ExpressionEmitter<'context, 'block, 'hir> {
    /// MLIR context that owns every emitted type, attribute, and operation.
    context: &'context Context,

    /// HIR module used to resolve direct call signatures.
    module: &'hir hir::Module,

    /// Entry block receiving emitted operations.
    block: &'block Block<'context>,

    /// HIR local identities mapped to current MLIR static single-assignment values.
    locals: HashMap<hir::LocalId, Value<'context, 'block>>,

    /// Bootstrap integer type shared by every native scalar operation.
    integer: Type<'context>,

    /// Placeholder location until source-backed MLIR locations are implemented.
    location: Location<'context>,
}

impl<'context, 'block, 'hir> ExpressionEmitter<'context, 'block, 'hir> {
    fn new(
        context: &'context Context,
        module: &'hir hir::Module,
        function: &hir::Function,
        block: &'block Block<'context>,
    ) -> BackendResult<Self> {
        let locals = function
            .parameters
            .iter()
            .enumerate()
            .map(|(index, parameter)| {
                block
                    .argument(index)
                    .map(|value| (parameter.id, Value::from(value)))
                    .map_err(|error| lowering_error("M0003", error.to_string()))
            })
            .collect::<BackendResult<_>>()?;
        Ok(Self {
            context,
            module,
            block,
            locals,
            integer: IntegerType::new(context, 64).into(),
            location: Location::unknown(context),
        })
    }

    fn emit(
        &mut self,
        expression: &hir::Expression,
    ) -> BackendResult<Option<Value<'context, 'block>>> {
        use hir::ExpressionKind as E;
        match &expression.kind {
            E::Local(id) => self
                .locals
                .get(id)
                .copied()
                .map(Some)
                .ok_or_else(|| lowering_error("M0003", "HIR references an unknown local")),
            E::Integer(value) => self.integer(*value).map(Some),
            E::Assign { local, value } => {
                let value = require_value(self.emit(value)?)?;
                self.locals.insert(*local, value);
                Ok(None)
            }
            E::Unary { operator, operand } => self.emit_unary(*operator, operand),
            E::Binary { operator, left, right } => self.emit_binary(*operator, left, right),
            E::Call { function, arguments } => self.emit_call(*function, arguments),
            E::Return(_) => Err(lowering_error("M0003", "nested return reached lowering")),
            E::Float(_) | E::Bool(_) | E::String(_) => unsupported("non-i64 literal"),
        }
    }

    fn emit_unary(
        &mut self,
        operator: hir::UnaryOperator,
        operand: &hir::Expression,
    ) -> BackendResult<Option<Value<'context, 'block>>> {
        match operator {
            hir::UnaryOperator::Positive => self.emit(operand),
            hir::UnaryOperator::Negative => {
                let operand = require_value(self.emit(operand)?)?;
                let zero = self.constant(0)?;
                self.result(arith::subi(zero, operand, self.location)).map(Some)
            }
            hir::UnaryOperator::Not => unsupported("boolean not"),
        }
    }

    fn emit_binary(
        &mut self,
        operator: hir::BinaryOperator,
        left: &hir::Expression,
        right: &hir::Expression,
    ) -> BackendResult<Option<Value<'context, 'block>>> {
        let left = require_value(self.emit(left)?)?;
        let right = require_value(self.emit(right)?)?;
        let operation = match operator {
            hir::BinaryOperator::Add => arith::addi(left, right, self.location),
            hir::BinaryOperator::Subtract => arith::subi(left, right, self.location),
            hir::BinaryOperator::Multiply => arith::muli(left, right, self.location),
            hir::BinaryOperator::Divide => arith::divsi(left, right, self.location),
            hir::BinaryOperator::Remainder => arith::remsi(left, right, self.location),
            _ => return unsupported("comparison or boolean operation"),
        };
        self.result(operation).map(Some)
    }

    fn emit_call(
        &mut self,
        function: hir::FunctionId,
        arguments: &[hir::Expression],
    ) -> BackendResult<Option<Value<'context, 'block>>> {
        let (name, result) = {
            let target = self
                .module
                .functions
                .get(function.0)
                .ok_or_else(|| lowering_error("M0003", "HIR references an unknown function"))?;
            require_scalar_signature(target)?;
            (target.name.clone(), target.result)
        };
        let arguments = arguments
            .iter()
            .map(|argument| self.emit(argument).and_then(require_value))
            .collect::<BackendResult<Vec<_>>>()?;
        let results = if result == hir::Type::Unit { vec![] } else { vec![self.integer] };
        let call = self.block.append_operation(func::call(
            self.context,
            FlatSymbolRefAttribute::new(self.context, &name),
            &arguments,
            &results,
            self.location,
        ));
        if results.is_empty() {
            return Ok(None);
        }
        call.result(0)
            .map(Value::from)
            .map(Some)
            .map_err(|error| lowering_error("M0003", error.to_string()))
    }

    fn append_return(
        &self,
        function: &hir::Function,
        value: Option<Value<'context, 'block>>,
    ) -> BackendResult<()> {
        let operands = match (function.result, value) {
            (hir::Type::Unit, None) => vec![],
            (hir::Type::I64, Some(value)) => vec![value],
            _ => return Err(lowering_error("M0005", "function tail has no matching value")),
        };
        self.block.append_operation(func::r#return(&operands, self.location));
        Ok(())
    }

    fn constant(&self, value: i64) -> BackendResult<Value<'context, 'block>> {
        self.result(arith::constant(
            self.context,
            IntegerAttribute::new(self.integer, value).into(),
            self.location,
        ))
    }

    fn integer(&self, value: i128) -> BackendResult<Value<'context, 'block>> {
        let value = i64::try_from(value)
            .map_err(|_| lowering_error("M0003", "integer value is outside the i64 backend"))?;
        self.constant(value)
    }

    fn result(
        &self,
        operation: melior::ir::Operation<'context>,
    ) -> BackendResult<Value<'context, 'block>> {
        self.block
            .append_operation(operation)
            .result(0)
            .map(Value::from)
            .map_err(|error| lowering_error("M0003", error.to_string()))
    }
}

fn require_value<'context, 'block>(
    value: Option<Value<'context, 'block>>,
) -> BackendResult<Value<'context, 'block>> {
    value.ok_or_else(|| lowering_error("M0003", "expression does not produce a value"))
}

fn unsupported<T>(feature: &str) -> BackendResult<T> {
    Err(lowering_error("M0001", format!("{feature} is outside the initial i64 backend slice")))
}
