//! Scalar static single-assignment form accepted by the initial Cranelift backend.
//!
//! Each value is defined once.
//! The Multi-Level Intermediate Representation translator produces this narrow form.
//! Just-in-time and object emission share it.

#[derive(Debug)]
pub(super) struct Module {
    pub(super) functions: Vec<Function>,
}

#[derive(Debug)]
pub(super) struct Function {
    pub(super) name: String,
    pub(super) parameter_count: usize,
    pub(super) returns_value: bool,
    pub(super) operations: Vec<Operation>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ValueId(pub(super) usize);

#[derive(Debug)]
pub(super) enum Operation {
    Constant { result: ValueId, value: i64 },
    Binary { result: ValueId, operator: BinaryOperator, left: ValueId, right: ValueId },
    Call { result: Option<ValueId>, function: String, arguments: Vec<ValueId> },
    Return(Option<ValueId>),
}

#[derive(Clone, Copy, Debug)]
pub(super) enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
}
