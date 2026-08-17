//! The only unsafe boundary in the initial just-in-time compiler.
//!
//! Safe artifact code checks names, result types, and arity before entering this module.

#![allow(unsafe_code)]

use super::error::{BackendResult, backend_error};

pub(super) fn invoke_i64(pointer: *const u8, arguments: &[i64]) -> BackendResult<i64> {
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
