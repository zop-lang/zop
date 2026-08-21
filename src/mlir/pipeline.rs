// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: Apache-2.0

//! Verified passes over emitted Multi-Level Intermediate Representation.

use melior::{
    Context,
    ir::Module,
    pass::{PassManager, transform},
    utility::register_all_passes,
};

use crate::backend::error::{BackendResult, lowering_error};

/// Canonicalize and deduplicate the initial pure scalar module.
pub(super) fn run_scalar(context: &Context, module: &mut Module<'_>) -> BackendResult<()> {
    register_all_passes();
    let manager = PassManager::new(context);
    manager.enable_verifier(true);
    manager.add_pass(transform::create_canonicalizer());
    manager.add_pass(transform::create_cse());
    manager
        .run(module)
        .map_err(|error| lowering_error("M0006", format!("MLIR scalar pipeline failed: {error}")))
}
