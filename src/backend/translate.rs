// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: Apache-2.0

//! Verified Multi-Level Intermediate Representation (MLIR) to the restricted
//! Cranelift input intermediate representation.
//!
//! Unknown operations stop here before machine-code construction.
//! Translation admits one verified scalar block, assigns stable values, and
//! converts each allowed operation exactly once.

use melior::ir::{
    Attribute, BlockRef, Module as MlirModule, RegionLike, Value,
    attribute::{FlatSymbolRefAttribute, IntegerAttribute, StringAttribute, TypeAttribute},
    block::BlockLike,
    operation::{OperationLike, OperationRef},
    r#type::FunctionType,
};

use super::{
    error::{BackendResult, backend_error},
    scalar,
};

/// Translate one verified MLIR module into the restricted scalar boundary.
pub(super) fn translate(module: &MlirModule<'_>) -> BackendResult<scalar::Module> {
    let mut functions = Vec::new();
    let mut operation = module.body().first_operation();
    while let Some(current) = operation {
        if current.name().as_string_ref().as_str() != Ok("func.func") {
            let identifier = current.name();
            let name = identifier.as_string_ref().as_str().unwrap_or("");
            return Err(backend_error(
                "B0001",
                format!("unsupported top-level MLIR operation `{name}`"),
            ));
        }
        functions.push(translate_function(current)?);
        operation = current.next_in_block();
    }
    Ok(scalar::Module { functions })
}

fn translate_function(function: OperationRef<'_, '_>) -> BackendResult<scalar::Function> {
    let name = string_attribute(function, "sym_name")?.value().to_owned();
    let function_type = TypeAttribute::try_from(attribute(function, "function_type")?)
        .map_err(|error| backend_error("B0002", error.to_string()))?;
    let function_type = FunctionType::try_from(function_type.value())
        .map_err(|error| backend_error("B0002", error.to_string()))?;
    if function_type.result_count() > 1 {
        return Err(backend_error("B0002", "multiple MLIR results are unsupported"));
    }
    let block = function
        .region(0)
        .map_err(|error| backend_error("B0002", error.to_string()))?
        .first_block()
        .ok_or_else(|| backend_error("B0002", "function has no entry block"))?;
    if block.argument_count() != function_type.input_count() {
        return Err(backend_error("B0002", "function signature and block disagree"));
    }

    let mut translator = FunctionTranslator::new(block)?;
    translator.translate_operations()?;
    Ok(scalar::Function {
        name,
        parameter_count: function_type.input_count(),
        returns_value: function_type.result_count() == 1,
        operations: translator.operations,
    })
}

/// Verified MLIR entry block translated into the restricted scalar form.
struct FunctionTranslator<'context, 'operation> {
    /// Only block admitted by the current scalar function contract.
    block: BlockRef<'context, 'operation>,

    /// MLIR values paired with stable scalar identities.
    values: Vec<(Value<'context, 'operation>, scalar::ValueId)>,

    /// Scalar operations emitted in dependency order.
    operations: Vec<scalar::Operation>,

    /// First unused scalar value identity.
    next_value: usize,
}

impl<'context, 'operation> FunctionTranslator<'context, 'operation> {
    fn new(block: BlockRef<'context, 'operation>) -> BackendResult<Self> {
        let values = (0..block.argument_count())
            .map(|index| {
                block
                    .argument(index)
                    .map(|value| (Value::from(value), scalar::ValueId(index)))
                    .map_err(|error| backend_error("B0002", error.to_string()))
            })
            .collect::<BackendResult<Vec<_>>>()?;
        Ok(Self { block, next_value: values.len(), values, operations: Vec::new() })
    }

    fn translate_operations(&mut self) -> BackendResult<()> {
        let mut operation = self.block.first_operation();
        while let Some(current) = operation {
            self.translate_operation(current)?;
            operation = current.next_in_block();
        }
        if !matches!(self.operations.last(), Some(scalar::Operation::Return(_))) {
            return Err(backend_error("B0002", "function has no return terminator"));
        }
        Ok(())
    }

    fn translate_operation(
        &mut self,
        operation: OperationRef<'context, 'operation>,
    ) -> BackendResult<()> {
        let identifier = operation.name();
        let name = identifier
            .as_string_ref()
            .as_str()
            .map_err(|error| backend_error("B0002", error.to_string()))?;
        match name {
            "arith.constant" => self.translate_constant(operation),
            "arith.addi" => self.translate_binary(operation, scalar::BinaryOperator::Add),
            "arith.subi" => self.translate_binary(operation, scalar::BinaryOperator::Subtract),
            "arith.muli" => self.translate_binary(operation, scalar::BinaryOperator::Multiply),
            "arith.divsi" => self.translate_binary(operation, scalar::BinaryOperator::Divide),
            "arith.remsi" => self.translate_binary(operation, scalar::BinaryOperator::Remainder),
            "func.call" => self.translate_call(operation),
            "func.return" => self.translate_return(operation),
            _ => Err(backend_error("B0001", format!("unsupported MLIR operation `{name}`"))),
        }
    }

    fn translate_constant(
        &mut self,
        operation: OperationRef<'context, 'operation>,
    ) -> BackendResult<()> {
        let value = IntegerAttribute::try_from(attribute(operation, "value")?)
            .map_err(|error| backend_error("B0002", error.to_string()))?
            .signed_value();
        let result = self.define_result(operation)?;
        self.operations.push(scalar::Operation::Constant { result, value });
        Ok(())
    }

    fn translate_binary(
        &mut self,
        operation: OperationRef<'context, 'operation>,
        operator: scalar::BinaryOperator,
    ) -> BackendResult<()> {
        let left = self.operand(operation, 0)?;
        let right = self.operand(operation, 1)?;
        let result = self.define_result(operation)?;
        self.operations.push(scalar::Operation::Binary { result, operator, left, right });
        Ok(())
    }

    fn translate_call(
        &mut self,
        operation: OperationRef<'context, 'operation>,
    ) -> BackendResult<()> {
        let function = FlatSymbolRefAttribute::try_from(attribute(operation, "callee")?)
            .map_err(|error| backend_error("B0002", error.to_string()))?
            .value()
            .to_owned();
        let arguments = (0..operation.operand_count())
            .map(|index| self.operand(operation, index))
            .collect::<BackendResult<Vec<_>>>()?;
        let result = match operation.result_count() {
            0 => None,
            1 => Some(self.define_result(operation)?),
            _ => return Err(backend_error("B0002", "call has multiple results")),
        };
        self.operations.push(scalar::Operation::Call { result, function, arguments });
        Ok(())
    }

    fn translate_return(
        &mut self,
        operation: OperationRef<'context, 'operation>,
    ) -> BackendResult<()> {
        let value = match operation.operand_count() {
            0 => None,
            1 => Some(self.operand(operation, 0)?),
            _ => return Err(backend_error("B0002", "return has multiple values")),
        };
        self.operations.push(scalar::Operation::Return(value));
        Ok(())
    }

    fn define_result(
        &mut self,
        operation: OperationRef<'context, 'operation>,
    ) -> BackendResult<scalar::ValueId> {
        if operation.result_count() != 1 {
            return Err(backend_error("B0002", "operation must have one result"));
        }
        let value = operation
            .result(0)
            .map(Value::from)
            .map_err(|error| backend_error("B0002", error.to_string()))?;
        let id = scalar::ValueId(self.next_value);
        self.next_value += 1;
        self.values.push((value, id));
        Ok(id)
    }

    fn operand(
        &self,
        operation: OperationRef<'context, 'operation>,
        index: usize,
    ) -> BackendResult<scalar::ValueId> {
        let operand =
            operation.operand(index).map_err(|error| backend_error("B0002", error.to_string()))?;
        self.values
            .iter()
            .find_map(|(value, id)| (*value == operand).then_some(*id))
            .ok_or_else(|| backend_error("B0002", "operand has no translated definition"))
    }
}

fn attribute<'context>(
    operation: OperationRef<'context, '_>,
    name: &str,
) -> BackendResult<Attribute<'context>> {
    operation.attribute(name).map_err(|error| backend_error("B0002", error.to_string()))
}

fn string_attribute<'context>(
    operation: OperationRef<'context, '_>,
    name: &str,
) -> BackendResult<StringAttribute<'context>> {
    StringAttribute::try_from(attribute(operation, name)?)
        .map_err(|error| backend_error("B0002", error.to_string()))
}
