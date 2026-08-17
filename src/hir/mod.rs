//! Typed high-level intermediate representation.
//!
//! The frontend produces this form. Target-independent lowering consumes it.

mod types;

pub use types::{
    BinaryOperator, Block, Expression, ExpressionKind, Function, FunctionId, FunctionKind, LocalId,
    Module, Parameter, ParameterMode, Type, UnaryOperator,
};
