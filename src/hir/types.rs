//! Typed high-level intermediate representation nodes shared by lowering stages.
//!
//! Stable identifiers replace source names before target-specific work begins.

use crate::source::Span;

#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    pub functions: Vec<Function>,
}

impl Module {
    #[must_use]
    pub fn function(&self, name: &str) -> Option<&Function> {
        self.functions.iter().find(|function| function.name == name)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    pub id: FunctionId,
    pub kind: FunctionKind,
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub result: Type,
    pub body: Block,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FunctionId(pub usize);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FunctionKind {
    Host,
    Kernel,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Parameter {
    pub id: LocalId,
    pub mode: ParameterMode,
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParameterMode {
    Borrow,
    Mut,
    Give,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Type {
    I32,
    I64,
    F32,
    F64,
    Bool,
    String,
    Unit,
    Never,
}

impl Type {
    #[must_use]
    pub const fn is_numeric(self) -> bool {
        matches!(self, Self::I32 | Self::I64 | Self::F32 | Self::F64)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub expressions: Vec<Expression>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub ty: Type,
    pub span: Span,
}

impl Expression {
    pub(crate) const fn new(kind: ExpressionKind, ty: Type, span: Span) -> Self {
        Self { kind, ty, span }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExpressionKind {
    Local(LocalId),
    Integer(i128),
    Float(f64),
    Bool(bool),
    String(String),
    Assign { local: LocalId, value: Box<Expression> },
    Unary { operator: UnaryOperator, operand: Box<Expression> },
    Binary { operator: BinaryOperator, left: Box<Expression>, right: Box<Expression> },
    Call { function: FunctionId, arguments: Vec<Expression> },
    Return(Option<Box<Expression>>),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct LocalId(pub usize);

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
    Multiply,
    Divide,
    Remainder,
}
