// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: MIT

//! Typed high-level intermediate representation.
//!
//! The frontend produces this form. Target-independent lowering consumes it.

mod types;

pub use types::{
    BinaryOperator, Block, CallArgument, Expression, ExpressionKind, Function, FunctionId,
    FunctionKind, LocalId, Module, Parameter, ParameterId, ParameterMode, Type, UnaryOperator,
};
