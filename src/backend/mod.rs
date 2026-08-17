//! Verified Multi-Level Intermediate Representation (MLIR) and native-code backends.
//!
//! Typed frontend output reaches Cranelift only through the verified MLIR boundary.

mod cranelift;
mod error;
mod ffi;
mod mlir;
mod scalar;
mod translate;

pub use cranelift::{JitArtifact, compile_jit, compile_object};
pub use mlir::mlir_text;
