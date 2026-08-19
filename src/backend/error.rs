// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: Apache-2.0

//! Lowering and backend diagnostic construction.
//!
//! Backend modules use one result type and distinct stage codes.

use crate::{
    diagnostic::{Diagnostic, Diagnostics, Stage},
    source::Span,
};

/// Backend result carrying one or more structured diagnostics.
pub(super) type BackendResult<T> = Result<T, Diagnostics>;

/// Construct one lowering-stage diagnostic list.
pub(super) fn lowering_error(code: &'static str, message: impl Into<String>) -> Diagnostics {
    vec![Diagnostic::new(Stage::Lowering, code, message, Span::default())]
}

/// Construct one target-backend diagnostic list.
pub(super) fn backend_error(code: &'static str, message: impl Into<String>) -> Diagnostics {
    vec![Diagnostic::new(Stage::Backend, code, message, Span::default())]
}
