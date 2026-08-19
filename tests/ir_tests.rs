// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: Apache-2.0

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

#[test]
fn named_arguments_evaluate_in_source_order_before_parameter_placement() {
    let source = concat!(
        "fn first -> i64\n",
        "    1\n",
        "fn second -> i64\n",
        "    2\n",
        "fn subtract left: i64, right: i64 -> i64\n",
        "    left - right\n",
        "fn main -> i64\n",
        "    subtract right=second(), left=first()\n",
    );
    let hir = analyze(source).expect("named call should type-check");
    let mlir = mlir_text(&hir).expect("named call should lower");
    let main = mlir.split("func.func @main").nth(1).expect("main function should exist");
    let second = main.find("call @second").expect("second argument should be evaluated");
    let first = main.find("call @first").expect("first argument should be evaluated");
    let subtract = main.find("call @subtract").expect("callee should be invoked");

    assert!(second < first);
    assert!(first < subtract);
    assert!(main.contains("call @subtract(%1, %0)"));
}
