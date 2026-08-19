// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: Apache-2.0

//! Structured errors shared by every compiler stage.
//!
//! Invalid user input returns these values instead of panicking or inventing a result.

use std::fmt::{self, Display, Formatter};

use crate::source::Span;

/// The compiler stage that rejected the input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Stage {
    /// Raw-token or indentation-layout analysis.
    Lex,

    /// Source grammar construction.
    Parse,

    /// Name, type, ownership, effect, or language-rule checking.
    Semantic,

    /// Conversion between verified intermediate representations.
    Lowering,

    /// Target validation or machine-code construction.
    Backend,
}

/// One source-backed compiler error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    /// Compiler boundary that owns the rejected contract.
    pub stage: Stage,

    /// Stable machine-readable identifier for this diagnostic class.
    pub code: &'static str,

    /// Human-readable explanation without terminal presentation markup.
    pub message: String,

    /// Half-open source range responsible for the rejection.
    pub span: Span,
}

impl Diagnostic {
    /// Construct one structured compiler error.
    #[must_use]
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

/// Ordered diagnostics produced by one compiler operation.
pub type Diagnostics = Vec<Diagnostic>;
