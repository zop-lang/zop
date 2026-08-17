//! Source-level syntax tree nodes.
//!
//! Nodes preserve spelling-level structure and spans without assigning semantic identities.

use crate::source::Span;

#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    pub functions: Vec<Function>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    pub kind: FunctionKind,
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<TypeExpression>,
    pub failure: FailureType,
    pub body: Block,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FunctionKind {
    Host,
    Kernel,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Parameter {
    pub mode: ParameterMode,
    pub name: String,
    pub ty: TypeExpression,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParameterMode {
    Borrow,
    Mut,
    Give,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypeExpression {
    pub name: String,
    pub dimensions: Vec<Dimension>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Dimension {
    Name(String),
    Integer(u64),
}

#[derive(Clone, Debug, PartialEq)]
pub enum FailureType {
    None,
    Infer,
    Named(TypeExpression),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub expressions: Vec<Expression>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub span: Span,
}

impl Expression {
    pub(super) const fn new(kind: ExpressionKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExpressionKind {
    Name(String),
    Integer(i128),
    Float(f64),
    Bool(bool),
    String(String),
    Assign { target: Box<Expression>, value: Box<Expression> },
    Unary { operator: UnaryOperator, operand: Box<Expression> },
    Binary { operator: BinaryOperator, left: Box<Expression>, right: Box<Expression> },
    Member { object: Box<Expression>, name: String },
    Call { callee: Box<Expression>, arguments: Vec<Argument> },
    If { condition: Box<Expression>, then_block: Block, else_block: Option<Block> },
    Return(Option<Box<Expression>>),
    Fail(Box<Expression>),
    Try(Box<Expression>),
    Catch { expression: Box<Expression>, arms: Vec<CatchArm> },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Argument {
    pub label: Option<String>,
    pub value: Expression,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CatchArm {
    pub pattern: Pattern,
    pub body: Block,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pattern {
    pub name: String,
    pub bindings: Vec<String>,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnaryOperator {
    Positive,
    Negative,
    Not,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BinaryOperator {
    Or,
    And,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Add,
    Subtract,
    Matmul,
    Multiply,
    Divide,
    Remainder,
}
