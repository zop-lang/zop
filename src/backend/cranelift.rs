// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: Apache-2.0

//! Restricted scalar intermediate representation to Cranelift executable and object code.
//!
//! Just-in-time and object modules share declaration and function construction.
//! Both modes declare every symbol first, define every function second, and
//! reject the artifact before executable memory or object bytes escape.

use std::collections::HashMap;

use cranelift_codegen::{
    Context as CodegenContext,
    ir::{AbiParam, InstBuilder, Signature, UserFuncName, types},
    settings,
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, FuncOrDataId, Linkage, Module as _, default_libcall_names};
use cranelift_object::{ObjectBuilder, ObjectModule};

use crate::{hir, mlir::with_verified_module};

use super::{
    error::{BackendResult, backend_error},
    ffi, scalar,
    translate::translate,
};

/// Executable module plus the verified signatures allowed through its safe API.
pub struct JitArtifact {
    /// Cranelift owner that keeps finalized executable memory alive.
    module: JITModule,

    /// Exported symbols mapped to their checked invocation contracts.
    functions: HashMap<String, JitFunction>,
}

/// Cranelift symbol and scalar calling convention for one exported function.
struct JitFunction {
    /// Module-local function handle used to obtain finalized code.
    id: FuncId,

    /// Number of `i64` arguments accepted by the bootstrap invocation API.
    parameter_count: usize,

    /// Whether the function produces the required `i64` result.
    returns_value: bool,
}

impl JitArtifact {
    /// Invoke an exported function with the initial scalar calling convention.
    ///
    /// # Errors
    ///
    /// Returns a backend diagnostic when the symbol is unknown or its checked
    /// signature is not an `i64`-returning function of the supplied arity.
    #[expect(
        unsafe_code,
        reason = "the safe wrapper proves the private JIT signature metadata before invocation"
    )]
    pub fn invoke_i64(&self, name: &str, arguments: &[i64]) -> BackendResult<i64> {
        let function = self
            .functions
            .get(name)
            .ok_or_else(|| backend_error("B0005", format!("unknown JIT function `{name}`")))?;
        if arguments.len() != function.parameter_count || !function.returns_value {
            return Err(backend_error(
                "B0006",
                "JIT invocation does not match an i64-returning function signature",
            ));
        }
        let pointer = self.module.get_finalized_function(function.id);
        // SAFETY: `functions` is built from the same verified scalar module as
        // `module`. The checks above prove this symbol returns `i64` and accepts
        // exactly the supplied number of `i64` arguments.
        unsafe { ffi::invoke_i64(pointer, arguments) }
    }
}

/// Compile typed Zop code into executable memory owned by the returned artifact.
///
/// # Errors
///
/// Returns a lowering or backend diagnostic when MLIR translation, target
/// construction, function definition, or finalization fails.
pub fn compile_jit(hir: &hir::Module) -> BackendResult<JitArtifact> {
    with_verified_module(hir, |module| emit_jit(&translate(module)?))
}

/// Compile typed Zop code into a native object for the current host.
///
/// # Errors
///
/// Returns a lowering or backend diagnostic when the host target, translation,
/// function definition, or object emission fails.
pub fn compile_object(hir: &hir::Module) -> BackendResult<Vec<u8>> {
    with_verified_module(hir, |module| emit_object(&translate(module)?))
}

fn emit_jit(input: &scalar::Module) -> BackendResult<JitArtifact> {
    let builder = JITBuilder::new(default_libcall_names())
        .map_err(|error| backend_error("B0003", error.to_string()))?;
    let mut module = JITModule::new(builder);
    let declared = declare_functions(&mut module, input)?;
    define_functions(&mut module, input, &declared)?;
    module.finalize_definitions().map_err(|error| backend_error("B0003", error.to_string()))?;
    let functions = input
        .functions
        .iter()
        .zip(declared)
        .map(|(function, id)| {
            (
                function.name.clone(),
                JitFunction {
                    id,
                    parameter_count: function.parameter_count,
                    returns_value: function.returns_value,
                },
            )
        })
        .collect();
    Ok(JitArtifact { module, functions })
}

fn emit_object(input: &scalar::Module) -> BackendResult<Vec<u8>> {
    let isa = cranelift_native::builder()
        .map_err(|error| backend_error("B0004", error))?
        .finish(settings::Flags::new(settings::builder()))
        .map_err(|error| backend_error("B0004", error.to_string()))?;
    let builder = ObjectBuilder::new(isa, "zop", default_libcall_names())
        .map_err(|error| backend_error("B0004", error.to_string()))?;
    let mut module = ObjectModule::new(builder);
    let declared = declare_functions(&mut module, input)?;
    define_functions(&mut module, input, &declared)?;
    module.finish().emit().map_err(|error| backend_error("B0004", error.to_string()))
}

fn declare_functions<M: cranelift_module::Module>(
    module: &mut M,
    input: &scalar::Module,
) -> BackendResult<Vec<FuncId>> {
    input
        .functions
        .iter()
        .map(|function| {
            module
                .declare_function(
                    &function.name,
                    Linkage::Export,
                    &signature(module, function.parameter_count, function.returns_value),
                )
                .map_err(|error| backend_error("B0003", error.to_string()))
        })
        .collect()
}

fn define_functions<M: cranelift_module::Module>(
    module: &mut M,
    input: &scalar::Module,
    declared: &[FuncId],
) -> BackendResult<()> {
    for (function, id) in input.functions.iter().zip(declared) {
        let mut context = module.make_context();
        context.func.signature =
            signature(module, function.parameter_count, function.returns_value);
        context.func.name = UserFuncName::user(0, id.as_u32());
        build_function(module, &mut context, function)?;
        module
            .define_function(*id, &mut context)
            .map_err(|error| backend_error("B0003", error.to_string()))?;
    }
    Ok(())
}

fn build_function<M: cranelift_module::Module>(
    module: &mut M,
    context: &mut CodegenContext,
    function: &scalar::Function,
) -> BackendResult<()> {
    let mut builder_context = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut context.func, &mut builder_context);
    let entry = builder.create_block();
    builder.switch_to_block(entry);
    builder.append_block_params_for_function_params(entry);
    let mut values = builder.block_params(entry).to_vec();

    for operation in &function.operations {
        emit_operation(module, &mut builder, operation, &mut values)?;
    }
    builder.seal_all_blocks();
    builder.finalize(module.target_config());
    Ok(())
}

fn emit_operation<M: cranelift_module::Module>(
    module: &mut M,
    builder: &mut FunctionBuilder<'_>,
    operation: &scalar::Operation,
    values: &mut Vec<cranelift_codegen::ir::Value>,
) -> BackendResult<()> {
    use scalar::Operation as O;
    match operation {
        O::Constant { result, value } => {
            define_value(values, *result, builder.ins().iconst(types::I64, *value))?;
        }
        O::Binary { result, operator, left, right } => {
            let left = value(values, *left)?;
            let right = value(values, *right)?;
            let result_value = match operator {
                scalar::BinaryOperator::Add => builder.ins().iadd(left, right),
                scalar::BinaryOperator::Subtract => builder.ins().isub(left, right),
                scalar::BinaryOperator::Multiply => builder.ins().imul(left, right),
                scalar::BinaryOperator::Divide => builder.ins().sdiv(left, right),
                scalar::BinaryOperator::Remainder => builder.ins().srem(left, right),
            };
            define_value(values, *result, result_value)?;
        }
        O::Call { result, function, arguments } => {
            let id = match module.get_name(function) {
                Some(FuncOrDataId::Func(id)) => id,
                _ => {
                    return Err(backend_error(
                        "B0002",
                        format!("unknown call target `{function}`"),
                    ));
                }
            };
            let reference = module.declare_func_in_func(id, builder.func);
            let arguments =
                arguments.iter().map(|id| value(values, *id)).collect::<BackendResult<Vec<_>>>()?;
            let instruction = builder.ins().call(reference, &arguments);
            if let Some(result) = result {
                let result_value = *builder
                    .inst_results(instruction)
                    .first()
                    .ok_or_else(|| backend_error("B0002", "call result is missing"))?;
                define_value(values, *result, result_value)?;
            }
        }
        O::Return(result) => {
            let results =
                result.map(|id| value(values, id)).transpose()?.into_iter().collect::<Vec<_>>();
            builder.ins().return_(&results);
        }
    }
    Ok(())
}

fn signature<M: cranelift_module::Module>(
    module: &M,
    parameter_count: usize,
    returns_value: bool,
) -> Signature {
    let mut signature = module.make_signature();
    signature.params = vec![AbiParam::new(types::I64); parameter_count];
    if returns_value {
        signature.returns.push(AbiParam::new(types::I64));
    }
    signature
}

fn value(
    values: &[cranelift_codegen::ir::Value],
    id: scalar::ValueId,
) -> BackendResult<cranelift_codegen::ir::Value> {
    values
        .get(id.0)
        .copied()
        .ok_or_else(|| backend_error("B0002", "scalar IR uses an undefined value"))
}

fn define_value(
    values: &mut Vec<cranelift_codegen::ir::Value>,
    id: scalar::ValueId,
    value: cranelift_codegen::ir::Value,
) -> BackendResult<()> {
    if values.len() != id.0 {
        return Err(backend_error("B0002", "scalar SSA values are out of order"));
    }
    values.push(value);
    Ok(())
}
