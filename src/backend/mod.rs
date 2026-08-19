// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: Apache-2.0

//! Verified Multi-Level Intermediate Representation (MLIR) and native-code backends.
//!
//! Typed frontend output reaches Cranelift only through the verified MLIR boundary.

mod cranelift;
mod error;
mod ffi;
mod javascript;
mod mlir;
mod scalar;
mod translate;

pub use cranelift::{JitArtifact, compile_jit, compile_object};
pub use javascript::javascript_text;
pub use mlir::mlir_text;
