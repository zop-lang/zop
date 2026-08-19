// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: Apache-2.0

//! Typed high-level intermediate representation nodes shared by lowering stages.
//!
//! Stable identifiers replace source names before target-specific work begins.

use crate::source::Span;

/// One semantically valid source module ready for target lowering.
#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    /// Checked functions in stable source identity order.
    pub functions: Vec<Function>,
}

impl Module {
    /// Find a checked function by its source declaration name.
    #[must_use]
    pub fn function(&self, name: &str) -> Option<&Function> {
        self.functions.iter().find(|function| function.name == name)
    }
}

/// One function after name, type, and current-subset semantic checking.
#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    /// Stable module identity used by calls and backend symbol tables.
    pub id: FunctionId,

    /// Host or device execution domain.
    pub kind: FunctionKind,

    /// Source declaration name retained for diagnostics and emitted symbols.
    pub name: String,

    /// Parameters in calling-convention order.
    pub parameters: Vec<Parameter>,

    /// Type yielded by every successful function exit.
    pub result: Type,

    /// Checked expression body.
    pub body: Block,

    /// Source range covering the declaration.
    pub span: Span,
}

/// Stable source-order function identity within one [`Module`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FunctionId(
    /// Zero-based slot in [`Module::functions`].
    pub usize,
);

/// Checked execution domain for a function.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FunctionKind {
    /// Central processing unit host function.
    Host,

    /// Graphics processing unit kernel.
    Kernel,
}

/// One checked function parameter and its local binding.
#[derive(Clone, Debug, PartialEq)]
pub struct Parameter {
    /// Local identity introduced at function entry.
    pub id: LocalId,

    /// Borrowing or ownership-transfer contract at the call boundary.
    pub mode: ParameterMode,

    /// Source name retained for diagnostics and named calls.
    pub name: String,

    /// Concrete parameter type for this HIR body.
    pub ty: Type,

    /// Source range covering the parameter declaration.
    pub span: Span,
}

/// Stable calling-convention position within one callee signature.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ParameterId(
    /// Zero-based slot in [`Function::parameters`].
    pub usize,
);

/// Checked ownership access for one parameter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParameterMode {
    /// Immutable borrow for the duration of the call.
    Borrow,

    /// Exclusive mutable borrow for the duration of the call.
    Mut,

    /// Ownership transfer into the callee.
    Give,
}

/// Concrete scalar types implemented by the bootstrap HIR.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Type {
    /// Signed 32-bit integer.
    I32,

    /// Signed 64-bit integer.
    I64,

    /// IEEE 754 binary32 floating-point value.
    F32,

    /// IEEE 754 binary64 floating-point value.
    F64,

    /// Boolean truth value.
    Bool,

    /// Unicode string value.
    String,

    /// Successful expression with no payload.
    Unit,

    /// Expression that cannot complete its current control-flow path.
    Never,
}

impl Type {
    /// Return whether this scalar type admits numeric operators.
    #[must_use]
    pub const fn is_numeric(self) -> bool {
        matches!(self, Self::I32 | Self::I64 | Self::F32 | Self::F64)
    }
}

/// Checked expression sequence with one result type determined by context.
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    /// Expressions in source evaluation order.
    pub expressions: Vec<Expression>,

    /// Source range from indentation start through matching dedent.
    pub span: Span,
}

/// One checked expression with a concrete type and source origin.
#[derive(Clone, Debug, PartialEq)]
pub struct Expression {
    /// Target-independent operation represented by this expression.
    pub kind: ExpressionKind,

    /// Concrete result type before target lowering.
    pub ty: Type,

    /// Source range used by later diagnostics and debug information.
    pub span: Span,
}

impl Expression {
    /// Construct one typed expression after semantic validation.
    pub(crate) const fn new(kind: ExpressionKind, ty: Type, span: Span) -> Self {
        Self { kind, ty, span }
    }
}

/// Target-independent operations implemented by the scalar bootstrap.
#[derive(Clone, Debug, PartialEq)]
pub enum ExpressionKind {
    /// Read one local binding.
    Local(
        /// Stable function-local identity.
        LocalId,
    ),

    /// Contextually typed integer constant.
    Integer(
        /// Mathematical literal value before backend-width encoding.
        i128,
    ),

    /// Contextually typed floating-point constant.
    Float(
        /// Binary64 storage used by the bootstrap HIR container.
        f64,
    ),

    /// Boolean constant.
    Bool(
        /// Truth value.
        bool,
    ),

    /// Decoded string constant.
    String(
        /// Unicode scalar sequence.
        String,
    ),

    /// Definition or update of one local binding.
    Assign {
        /// Stable binding written exactly once by this operation.
        local: LocalId,

        /// Checked value evaluated before the write.
        value: Box<Expression>,
    },

    /// Prefix scalar operation.
    Unary {
        /// Operation with source semantics already fixed.
        operator: UnaryOperator,

        /// Checked operand evaluated once.
        operand: Box<Expression>,
    },

    /// Infix scalar operation.
    Binary {
        /// Operation with numeric meaning already fixed.
        operator: BinaryOperator,

        /// Left operand evaluated before the right operand.
        left: Box<Expression>,

        /// Right operand evaluated after the left operand.
        right: Box<Expression>,
    },

    /// Direct call to one module function.
    Call {
        /// Stable callee identity.
        function: FunctionId,

        /// Checked arguments in source evaluation order with explicit parameter placement.
        arguments: Vec<CallArgument>,
    },

    /// Early exit from the current function.
    Return(
        /// Returned value, or absence for a unit result.
        Option<Box<Expression>>,
    ),
}

/// One checked call argument before target-specific evaluation and placement.
#[derive(Clone, Debug, PartialEq)]
pub struct CallArgument {
    /// Callee parameter that receives the evaluated value.
    pub parameter: ParameterId,

    /// Value evaluated at this position in the source argument list.
    pub value: Expression,

    /// Source range covering the optional label and value.
    pub span: Span,
}

/// Stable local-binding identity within one [`Function`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct LocalId(
    /// Zero-based slot assigned during body checking.
    pub usize,
);

/// Checked prefix scalar operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnaryOperator {
    /// Numeric identity.
    Positive,

    /// Numeric negation.
    Negative,

    /// Boolean negation.
    Not,
}

/// Checked infix scalar operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BinaryOperator {
    /// Short-circuit boolean disjunction.
    Or,

    /// Short-circuit boolean conjunction.
    And,

    /// Equality comparison.
    Equal,

    /// Inequality comparison.
    NotEqual,

    /// Strict less-than comparison.
    Less,

    /// Less-than-or-equal comparison.
    LessEqual,

    /// Strict greater-than comparison.
    Greater,

    /// Greater-than-or-equal comparison.
    GreaterEqual,

    /// Trapping numeric addition in the target contract.
    Add,

    /// Trapping numeric subtraction in the target contract.
    Subtract,

    /// Trapping numeric multiplication in the target contract.
    Multiply,

    /// Fractional division after operand typing.
    Divide,

    /// Integer remainder after operand typing.
    Remainder,
}
