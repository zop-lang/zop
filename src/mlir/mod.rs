// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: Apache-2.0

//! Verified Multi-Level Intermediate Representation (MLIR) compiler layer.
//!
//! Typed high-level intermediate representation enters through emission, then
//! passes through the named scalar pipeline before any backend can consume it.

mod emit;
mod pipeline;

pub use emit::mlir_text;
pub(crate) use emit::with_verified_module;
