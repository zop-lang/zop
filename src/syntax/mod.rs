//! Surface syntax and parsing.
//!
//! The frontend consumes this lossless-enough structure for name and type checking.

mod ast;
mod expression;
mod parser;

pub use ast::{
    Argument, BinaryOperator, Block, CatchArm, Dimension, Expression, ExpressionKind, FailureType,
    Function, FunctionKind, Module, Parameter, ParameterMode, Pattern, TypeExpression,
    UnaryOperator,
};
pub use parser::parse;
