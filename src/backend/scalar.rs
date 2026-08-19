// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: Apache-2.0

//! Scalar static single-assignment form accepted by the initial Cranelift backend.
//!
//! Each value is defined once.
//! The Multi-Level Intermediate Representation translator produces this narrow form.
//! Just-in-time and object emission share it.

/// Complete scalar module accepted by both Cranelift emission modes.
#[derive(Debug)]
pub(super) struct Module {
    /// Functions in the same declaration order as verified MLIR.
    pub(super) functions: Vec<Function>,
}

/// One linear scalar function after MLIR operation validation.
#[derive(Debug)]
pub(super) struct Function {
    /// Exported symbol copied from the verified MLIR declaration.
    pub(super) name: String,

    /// Number of `i64` entry-block arguments.
    pub(super) parameter_count: usize,

    /// Whether the bootstrap calling convention returns one `i64` value.
    pub(super) returns_value: bool,

    /// Static single-assignment operations in dependency order.
    pub(super) operations: Vec<Operation>,
}

/// Stable scalar static single-assignment value identity within one function.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ValueId(
    /// Zero-based value slot shared by translation and Cranelift emission.
    pub(super) usize,
);

/// Operations supported by the initial MLIR-to-Cranelift boundary.
#[derive(Debug)]
pub(super) enum Operation {
    /// Materialize one signed 64-bit constant.
    Constant {
        /// New value identity defined by this operation.
        result: ValueId,

        /// Exact constant bits interpreted as signed `i64`.
        value: i64,
    },

    /// Apply one scalar binary operation.
    Binary {
        /// New value identity defined by this operation.
        result: ValueId,

        /// Cranelift-level arithmetic selected by verified MLIR.
        operator: BinaryOperator,

        /// Previously defined left operand.
        left: ValueId,

        /// Previously defined right operand.
        right: ValueId,
    },

    /// Invoke one direct module function.
    Call {
        /// Result identity, or absence for a unit-returning callee.
        result: Option<ValueId>,

        /// Exported callee symbol.
        function: String,

        /// Previously defined arguments in calling-convention order.
        arguments: Vec<ValueId>,
    },

    /// Terminate the function.
    Return(
        /// Returned value, or absence for a unit function.
        Option<ValueId>,
    ),
}

/// Scalar arithmetic accepted by the initial Cranelift translator.
#[derive(Clone, Copy, Debug)]
pub(super) enum BinaryOperator {
    /// Signed integer addition with bootstrap machine semantics.
    Add,

    /// Signed integer subtraction with bootstrap machine semantics.
    Subtract,

    /// Signed integer multiplication with bootstrap machine semantics.
    Multiply,

    /// Signed truncating division retained as bootstrap scaffolding.
    Divide,

    /// Signed truncating remainder retained as bootstrap scaffolding.
    Remainder,
}
