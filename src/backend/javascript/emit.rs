// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: Apache-2.0

//! Public JavaScript emission boundary.
//!
//! The frontend supplies checked scalar semantics. Lowering selects exact
//! JavaScript representations, then the printer produces deterministic source.

use crate::{diagnostic::Diagnostics, hir};

use super::{lower, printer};

/// Lower one typed module to deterministic, standalone ECMAScript.
///
/// # Errors
///
/// Returns a lowering diagnostic when HIR requires a device target or a value
/// representation without exact JavaScript semantics.
pub fn javascript_text(module: &hir::Module) -> Result<String, Diagnostics> {
    lower::lower(module).map(|module| printer::print(&module))
}
