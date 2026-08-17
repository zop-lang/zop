//! Structured errors shared by every compiler stage.
//!
//! Invalid user input returns these values instead of panicking or inventing a result.

use crate::source::Span;
use std::fmt::{self, Display, Formatter};

/// The compiler stage that rejected the input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Stage {
    Lex,
    Parse,
    Semantic,
    Lowering,
    Backend,
}

/// One source-backed compiler error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub stage: Stage,
    pub code: &'static str,
    pub message: String,
    pub span: Span,
}

impl Diagnostic {
    pub fn new(stage: Stage, code: &'static str, message: impl Into<String>, span: Span) -> Self {
        Self { stage, code, message: message.into(), span }
    }
}

impl Display for Diagnostic {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}: {} at {}..{}",
            self.code, self.message, self.span.start, self.span.end
        )
    }
}

pub type Diagnostics = Vec<Diagnostic>;
