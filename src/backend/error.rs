//! Lowering and backend diagnostic construction.
//!
//! Backend modules use one result type and distinct stage codes.

use crate::{
    diagnostic::{Diagnostic, Diagnostics, Stage},
    source::Span,
};

pub(super) type BackendResult<T> = Result<T, Diagnostics>;

pub(super) fn lowering_error(code: &'static str, message: impl Into<String>) -> Diagnostics {
    vec![Diagnostic::new(Stage::Lowering, code, message, Span::default())]
}

pub(super) fn backend_error(code: &'static str, message: impl Into<String>) -> Diagnostics {
    vec![Diagnostic::new(Stage::Backend, code, message, Span::default())]
}
