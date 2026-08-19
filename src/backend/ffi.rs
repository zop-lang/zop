// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: Apache-2.0

//! The only unsafe boundary in the initial just-in-time compiler.
//!
//! Safe artifact code checks names, result types, and arity before entering this module.

#![expect(
    unsafe_code,
    reason = "executable JIT pointers are called only through this isolated boundary"
)]

use super::error::{BackendResult, backend_error};

/// Invoke executable memory through the bootstrap's scalar calling convention.
///
/// # Safety
///
/// `pointer` must name a live `extern "C"` function emitted by the owning
/// Cranelift module. Its parameters must be exactly `arguments.len()` `i64`
/// values, and its result must be one `i64`.
///
/// # Errors
///
/// Returns a backend diagnostic when the bootstrap does not implement the
/// supplied arity.
pub(super) unsafe fn invoke_i64(pointer: *const u8, arguments: &[i64]) -> BackendResult<i64> {
    match arguments {
        [] => {
            // SAFETY: Cranelift emitted this pointer from a verified `() -> i64` signature,
            // and `JitArtifact` checks the selected function's arity before calling us.
            let function =
                unsafe { std::mem::transmute::<*const u8, extern "C" fn() -> i64>(pointer) };
            Ok(function())
        }
        [first] => {
            // SAFETY: The pointer and arity invariants are established by `JitArtifact`.
            let function =
                unsafe { std::mem::transmute::<*const u8, extern "C" fn(i64) -> i64>(pointer) };
            Ok(function(*first))
        }
        [first, second] => {
            // SAFETY: The pointer and arity invariants are established by `JitArtifact`.
            let function = unsafe {
                std::mem::transmute::<*const u8, extern "C" fn(i64, i64) -> i64>(pointer)
            };
            Ok(function(*first, *second))
        }
        _ => Err(backend_error(
            "B0006",
            "the initial typed JIT invocation API accepts at most two arguments",
        )),
    }
}
