// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: MIT

//! Target JavaScript structure consumed by the deterministic printer.

/// Complete JavaScript module before deterministic printing.
#[derive(Debug)]
pub(super) struct Module {
    /// Lowered function declarations in stable HIR order.
    pub(super) functions: Vec<Function>,

    /// Explicit public-name mappings emitted after local declarations.
    pub(super) exports: Vec<Export>,
}

/// One JavaScript function with already-selected runtime representations.
#[derive(Debug)]
pub(super) struct Function {
    /// Collision-free local symbol generated from the HIR identity.
    pub(super) name: String,

    /// Local parameter symbols in calling-convention order.
    pub(super) parameters: Vec<String>,

    /// Statements in observable execution order.
    pub(super) body: Vec<Statement>,
}

/// Public source name mapped to one generated local symbol.
#[derive(Debug)]
pub(super) struct Export {
    /// Generated module-local binding.
    pub(super) local: String,

    /// Source declaration name exposed by ECMAScript.
    pub(super) exported: String,
}

/// Statement forms admitted by the deterministic printer.
#[derive(Debug)]
pub(super) enum Statement {
    /// First definition of one mutable local binding.
    Let {
        /// Generated collision-free local name.
        name: String,

        /// Initial value evaluated before the binding becomes visible.
        value: Expression,
    },

    /// Update of one previously defined local binding.
    Assign {
        /// Generated local name selected by HIR identity.
        name: String,

        /// Replacement value evaluated before assignment.
        value: Expression,
    },

    /// Effect-preserving expression whose value is discarded.
    Expression(
        /// Lowered expression retained because it may call user code.
        Expression,
    ),

    /// Function exit.
    Return(
        /// Returned value, or absence for a unit function.
        Option<Expression>,
    ),
}

/// JavaScript expression forms with precedence handled by the printer.
#[derive(Debug)]
pub(super) enum Expression {
    /// Read one generated local binding.
    Identifier(
        /// Collision-free local symbol.
        String,
    ),

    /// Preformatted numeric literal with exact JavaScript spelling.
    Number(
        /// Deterministic token emitted without reparsing a host float.
        String,
    ),

    /// Boolean literal.
    Bool(
        /// JavaScript truth value.
        bool,
    ),

    /// String literal before printer escaping.
    String(
        /// Unicode scalar sequence inherited from HIR.
        String,
    ),

    /// Direct call to one generated function or permitted host primitive.
    Call {
        /// Callee symbol printed without dynamic property lookup.
        function: String,

        /// Arguments in source evaluation order.
        arguments: Vec<Expression>,
    },

    /// Prefix JavaScript operation.
    Unary {
        /// Operation selected from checked HIR semantics.
        operator: UnaryOperator,

        /// Operand evaluated once.
        operand: Box<Expression>,
    },

    /// Infix JavaScript operation.
    Binary {
        /// Operation with precedence known by the printer.
        operator: BinaryOperator,

        /// Left operand evaluated before the right operand.
        left: Box<Expression>,

        /// Right operand evaluated after the left operand.
        right: Box<Expression>,
    },
}

/// Prefix JavaScript operations used by the bootstrap.
#[derive(Clone, Copy, Debug)]
pub(super) enum UnaryOperator {
    /// Arithmetic negation.
    Negative,

    /// Boolean negation.
    Not,
}

/// Infix JavaScript operations used by the bootstrap.
#[derive(Clone, Copy, Debug)]
pub(super) enum BinaryOperator {
    /// Short-circuit boolean disjunction.
    Or,

    /// Short-circuit boolean conjunction.
    And,

    /// Bitwise OR used to normalize exact `i32` results.
    BitOr,

    /// Strict equality without JavaScript coercion.
    StrictEqual,

    /// Strict inequality without JavaScript coercion.
    StrictNotEqual,

    /// Strict less-than comparison.
    Less,

    /// Less-than-or-equal comparison.
    LessEqual,

    /// Strict greater-than comparison.
    Greater,

    /// Greater-than-or-equal comparison.
    GreaterEqual,

    /// Addition after representation selection.
    Add,

    /// Subtraction after representation selection.
    Subtract,

    /// Multiplication for representations with native exact semantics.
    Multiply,

    /// Floating-point division.
    Divide,

    /// Floating-point remainder retained as nonconforming bootstrap scaffolding.
    Remainder,
}
