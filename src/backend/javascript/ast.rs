//! Target JavaScript structure consumed by the deterministic printer.

#[derive(Debug)]
pub(super) struct Module {
    pub(super) functions: Vec<Function>,
    pub(super) exports: Vec<Export>,
}

#[derive(Debug)]
pub(super) struct Function {
    pub(super) name: String,
    pub(super) parameters: Vec<String>,
    pub(super) body: Vec<Statement>,
}

#[derive(Debug)]
pub(super) struct Export {
    pub(super) local: String,
    pub(super) exported: String,
}

#[derive(Debug)]
pub(super) enum Statement {
    Let { name: String, value: Expression },
    Assign { name: String, value: Expression },
    Expression(Expression),
    Return(Option<Expression>),
}

#[derive(Debug)]
pub(super) enum Expression {
    Identifier(String),
    Number(String),
    Bool(bool),
    String(String),
    Call { function: String, arguments: Vec<Expression> },
    Unary { operator: UnaryOperator, operand: Box<Expression> },
    Binary { operator: BinaryOperator, left: Box<Expression>, right: Box<Expression> },
}

#[derive(Clone, Copy, Debug)]
pub(super) enum UnaryOperator {
    Negative,
    Not,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum BinaryOperator {
    Or,
    And,
    BitOr,
    StrictEqual,
    StrictNotEqual,
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
