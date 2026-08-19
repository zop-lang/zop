// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: MIT

//! Multi-Level Intermediate Representation lowering boundary tests.

use zop::{backend::mlir_text, frontend::analyze};

#[test]
fn scalar_hir_emits_verified_mlir() {
    let source = "fn add x: i64, y: i64 -> i64\n    x + y\n";
    let hir = analyze(source).expect("source should type-check");
    let mlir = mlir_text(&hir).expect("MLIR should verify");

    assert!(mlir.contains("func.func @add"));
    assert!(mlir.contains("arith.addi"));
    assert!(mlir.contains("return %0 : i64"));
}

#[test]
fn unsupported_scalar_types_stop_at_the_mlir_boundary() {
    let source = "fn identity x: f64 -> f64\n    x\n";
    let hir = analyze(source).expect("source should type-check");
    let errors = mlir_text(&hir).expect_err("f64 lowering should not exist yet");

    assert_eq!(errors[0].code, "M0001");
}

#[test]
fn kernels_never_fall_back_to_the_cpu_backend() {
    let source = "kn identity x: i64 -> i64\n    x\n";
    let hir = analyze(source).expect("kernel syntax should type-check");
    let errors = mlir_text(&hir).expect_err("a GPU backend is required");

    assert_eq!(errors[0].code, "M0001");
    assert!(errors[0].message.contains("requires a GPU backend"));
}
