// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: MIT

//! Cranelift just-in-time compilation and invocation tests.

use zop::{backend::compile_jit, frontend::analyze};

#[test]
fn cranelift_jit_executes_the_verified_scalar_module() {
    let source = "fn add x: i64, y: i64 -> i64\n    x + y\n";
    let hir = analyze(source).expect("source should type-check");
    let jit = compile_jit(&hir).expect("JIT compilation should succeed");

    assert_eq!(jit.invoke_i64("add", &[20, 22]).expect("call should succeed"), 42);
}

#[test]
fn calls_and_local_assignments_survive_the_full_pipeline() {
    let source = concat!(
        "fn twice value: i64 -> i64\n",
        "    value * 2\n",
        "fn answer -> i64\n",
        "    base = twice 20\n",
        "    base + 2\n",
    );
    let hir = analyze(source).expect("source should type-check");
    let jit = compile_jit(&hir).expect("JIT compilation should succeed");

    assert_eq!(jit.invoke_i64("answer", &[]).expect("call should succeed"), 42);
}

#[test]
fn typed_invocation_rejects_the_wrong_arity() {
    let source = "fn identity value: i64 -> i64\n    value\n";
    let hir = analyze(source).expect("source should type-check");
    let jit = compile_jit(&hir).expect("JIT compilation should succeed");
    let errors = jit
        .invoke_i64("identity", &[])
        .expect_err("wrong arity should fail before unsafe invocation");

    assert_eq!(errors[0].code, "B0006");
}

#[test]
fn unit_calls_do_not_invent_result_values() {
    let source = concat!(
        "fn touch\n",
        "    value = 1\n",
        "fn answer -> i64\n",
        "    touch()\n",
        "    42\n",
    );
    let hir = analyze(source).expect("unit call should type-check");
    let jit = compile_jit(&hir).expect("JIT compilation should succeed");

    assert_eq!(jit.invoke_i64("answer", &[]).expect("call should succeed"), 42);
}
