// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: Apache-2.0

//! Source-level syntax tree nodes.
//!
//! Nodes preserve spelling-level structure and spans without assigning semantic identities.

use crate::source::Span;

/// One parsed source file before semantic identities are assigned.
#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    /// Function and kernel declarations in source order.
    pub functions: Vec<Function>,

    /// Byte range covering the complete source file.
    pub span: Span,
}

/// Function or kernel declaration with spelling-level types and body syntax.
#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    /// Host or device execution domain selected by the declaration keyword.
    pub kind: FunctionKind,

    /// Unqualified declaration name as written in source.
    pub name: String,

    /// Ordered parameter declarations before semantic type resolution.
    pub parameters: Vec<Parameter>,

    /// Explicit result annotation, or absence for the implicit unit result.
    pub return_type: Option<TypeExpression>,

    /// Parsed error-channel declaration before error-type checking.
    pub failure: FailureType,

    /// Indented expression body.
    pub body: Block,

    /// Byte range from the declaration keyword through the body dedent.
    pub span: Span,
}

/// Execution domain declared for a callable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FunctionKind {
    /// Central processing unit host function declared with `fn`.
    Host,

    /// Graphics processing unit kernel declared with `kn`.
    Kernel,
}

/// One source parameter before type and ownership checking.
#[derive(Clone, Debug, PartialEq)]
pub struct Parameter {
    /// Borrowing or ownership-transfer mode selected at the call boundary.
    pub mode: ParameterMode,

    /// Source binding introduced inside the function body.
    pub name: String,

    /// Spelling-level parameter type.
    pub ty: TypeExpression,

    /// Byte range covering mode, name, and type.
    pub span: Span,
}

/// Ownership access granted to one function parameter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParameterMode {
    /// Immutable borrow used when no mode keyword is present.
    Borrow,

    /// Exclusive mutable borrow declared with `mut`.
    Mut,

    /// Ownership transfer declared with `give`.
    Give,
}

/// Source type name plus optional tensor dimensions.
#[derive(Clone, Debug, PartialEq)]
pub struct TypeExpression {
    /// Unresolved type name.
    pub name: String,

    /// Tensor extent expressions in logical axis order.
    pub dimensions: Vec<Dimension>,

    /// Byte range covering the complete type expression.
    pub span: Span,
}

/// Static or symbolic tensor extent written inside a type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Dimension {
    /// Symbolic extent whose identity is resolved by the frontend.
    Name(
        /// Source spelling of the symbolic extent.
        String,
    ),

    /// Non-negative extent available directly from source.
    Integer(
        /// Exact source value before target-size validation.
        u64,
    ),
}

/// Source-level error-channel declaration.
#[derive(Clone, Debug, PartialEq)]
pub enum FailureType {
    /// Infallible declaration with no `or fails` clause.
    None,

    /// Private inferred channel declared by bare `or fails`.
    Infer,

    /// Explicit exported channel declared by `or fails with`.
    Named(
        /// Source type naming the complete error sum.
        TypeExpression,
    ),
}

/// Indented sequence of expressions with one source result.
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    /// Expressions in source evaluation order.
    pub expressions: Vec<Expression>,

    /// Byte range from the indent token through the matching dedent.
    pub span: Span,
}

/// One source expression before name and type resolution.
#[derive(Clone, Debug, PartialEq)]
pub struct Expression {
    /// Spelling-level expression form.
    pub kind: ExpressionKind,

    /// Exact byte range used for diagnostics and later source mapping.
    pub span: Span,
}

impl Expression {
    /// Construct one syntax expression with its exact source range.
    pub(super) const fn new(kind: ExpressionKind, span: Span) -> Self {
        Self { kind, span }
    }
}

/// Complete expression grammar preserved by the syntax tree.
#[derive(Clone, Debug, PartialEq)]
pub enum ExpressionKind {
    /// Unresolved lexical or module name.
    Name(
        /// Source spelling used during name lookup.
        String,
    ),

    /// Integer literal before contextual type selection.
    Integer(
        /// Mathematical value parsed from source digits.
        i128,
    ),

    /// Floating-point literal before contextual width selection.
    Float(
        /// Parsed binary64 value retained by the bootstrap parser.
        f64,
    ),

    /// Boolean literal.
    Bool(
        /// `true` or `false` source value.
        bool,
    ),

    /// Decoded string literal.
    String(
        /// Unicode scalar sequence after escape processing.
        String,
    ),

    /// Binding or place update.
    Assign {
        /// Source expression that must resolve to a writable place.
        target: Box<Expression>,

        /// Value evaluated before the target is updated.
        value: Box<Expression>,
    },

    /// Prefix operator expression.
    Unary {
        /// Parsed operator spelling.
        operator: UnaryOperator,

        /// Operand expression evaluated once.
        operand: Box<Expression>,
    },

    /// Infix operator expression.
    Binary {
        /// Parsed operator with source precedence already applied.
        operator: BinaryOperator,

        /// Left operand evaluated before the right operand.
        left: Box<Expression>,

        /// Right operand evaluated after the left operand.
        right: Box<Expression>,
    },

    /// Member selection without invocation.
    Member {
        /// Receiver whose type later determines field or callable lookup.
        object: Box<Expression>,

        /// Selected member spelling.
        name: String,
    },

    /// Positional and named callable invocation.
    Call {
        /// Expression that must resolve to one callable value.
        callee: Box<Expression>,

        /// Arguments retained in source evaluation order.
        arguments: Vec<Argument>,
    },

    /// Expression-valued conditional block.
    If {
        /// Boolean condition evaluated before either branch.
        condition: Box<Expression>,

        /// Branch evaluated when the condition is true.
        then_block: Block,

        /// Optional false branch; absence produces the unit value.
        else_block: Option<Block>,
    },

    /// Early function exit.
    Return(
        /// Returned value, or absence for a unit return.
        Option<Box<Expression>>,
    ),

    /// Construction of the current function's error result.
    Fail(
        /// Error value checked against the declared error sum.
        Box<Expression>,
    ),

    /// Propagation of one compatible fallible result.
    Try(
        /// Fallible expression whose error channel is propagated.
        Box<Expression>,
    ),

    /// Local recovery from one fallible expression.
    Catch {
        /// Fallible expression evaluated before matching handlers.
        expression: Box<Expression>,

        /// Ordered exhaustive handlers.
        arms: Vec<CatchArm>,
    },
}

/// One call argument with optional parameter selection.
#[derive(Clone, Debug, PartialEq)]
pub struct Argument {
    /// Parameter label, or absence for the next positional parameter.
    pub label: Option<String>,

    /// Argument expression retained in source evaluation order.
    pub value: Expression,

    /// Byte range covering label, separator, and value.
    pub span: Span,
}

/// One required pattern and handler block after `catch`.
#[derive(Clone, Debug, PartialEq)]
pub struct CatchArm {
    /// Pattern selecting an error-sum case and its bindings.
    pub pattern: Pattern,

    /// Handler body evaluated for the selected case.
    pub body: Block,

    /// Byte range covering the complete arm.
    pub span: Span,
}

/// Case pattern used by error recovery and future matching forms.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pattern {
    /// Sum-case or type name selected by the pattern.
    pub name: String,

    /// Positional payload bindings introduced by the pattern.
    pub bindings: Vec<String>,

    /// Byte range covering the complete pattern.
    pub span: Span,
}

/// Prefix operators with parser-level precedence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnaryOperator {
    /// Numeric identity operator `+`.
    Positive,

    /// Numeric negation operator `-`.
    Negative,

    /// Boolean negation keyword `not`.
    Not,
}

/// Infix operators ordered later by the expression parser's precedence table.
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

    /// Numeric addition.
    Add,

    /// Numeric subtraction.
    Subtract,

    /// Tensor matrix multiplication.
    Matmul,

    /// Numeric multiplication.
    Multiply,

    /// Fractional division in the target language contract.
    Divide,

    /// Integer remainder in the target language contract.
    Remainder,
}
