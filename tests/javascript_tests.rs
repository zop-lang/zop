// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: MIT

//! Deterministic JavaScript lowering and optimization tests.

use zop::{backend::javascript_text, frontend::analyze, hir::FunctionId};

#[test]
fn scalar_hir_emits_a_deterministic_es_module() {
    let source = concat!(
        "fn affine x: f64, y: f64, bias: f64 -> f64\n",
        "    x * y + bias\n",
        "fn apply value: f64 -> f64\n",
        "    affine value, 2, 1\n",
    );
    let hir = analyze(source).expect("source should type-check");

    let javascript = javascript_text(&hir).expect("JavaScript should lower");

    assert_eq!(
        javascript,
        concat!(
            "function b0(l0, l1, l2) {\n",
            "    return l0 * l1 + l2;\n",
            "}\n\n",
            "function b1(l0) {\n",
            "    return b0(l0, 2, 1);\n",
            "}\n\n",
            "export { b0 as affine, b1 as apply };\n",
        )
    );
}

#[test]
fn i32_arithmetic_retains_its_machine_width() {
    let source = "fn step left: i32, right: i32 -> i32\n    left * right + 1\n";
    let hir = analyze(source).expect("source should type-check");

    let javascript = javascript_text(&hir).expect("i32 should lower");

    assert!(javascript.contains("return Math.imul(l0, l1) + 1 | 0;"));
}

#[test]
fn i64_requires_a_webassembly_compute_region() {
    let source = "fn identity value: i64 -> i64\n    value\n";
    let hir = analyze(source).expect("source should type-check");

    let errors = javascript_text(&hir).expect_err("i64 JavaScript must be rejected");

    assert_eq!(errors[0].code, "J0002");
    assert!(errors[0].message.contains("WebAssembly"));
}

#[test]
fn kernels_never_fall_back_to_javascript() {
    let source = "kn kernel value: f64 -> f64\n    value\n";
    let hir = analyze(source).expect("source should type-check");

    let errors = javascript_text(&hir).expect_err("kernel JavaScript must be rejected");

    assert_eq!(errors[0].code, "J0001");
}

#[test]
fn printer_preserves_tokens_and_uses_compact_float_literals() {
    let source = concat!(
        "fn negate value: i32 -> i32\n",
        "    -(-value)\n",
        "fn huge -> f64\n",
        "    1.0e100\n",
    );
    let hir = analyze(source).expect("source should type-check");

    let javascript = javascript_text(&hir).expect("JavaScript should lower");

    assert!(javascript.contains("return -(-l0 | 0) | 0;"));
    assert!(javascript.contains("return 1e+100;"));
}

#[test]
fn unchecked_i32_division_does_not_acquire_javascript_semantics() {
    let source = "fn divide left: i32, right: i32 -> i32\n    left / right\n";
    let hir = analyze(source).expect("source should type-check");

    let errors = javascript_text(&hir).expect_err("unchecked division must stop");

    assert_eq!(errors[0].code, "J0002");
}

#[test]
fn malformed_hir_fails_before_javascript_printing() {
    let source = "fn identity value: f64 -> f64\n    value\n";
    let mut hir = analyze(source).expect("source should type-check");
    hir.functions[0].id = FunctionId(1);

    let errors = javascript_text(&hir).expect_err("invalid identifiers must stop");

    assert_eq!(errors[0].code, "J0003");
}

#[test]
fn pure_scalar_constants_are_folded_before_printing() {
    let source = concat!(
        "fn integer -> i32\n",
        "    20 + 22\n",
        "fn float -> f64\n",
        "    6.0 / 4.0\n",
        "fn negative_zero -> f64\n",
        "    -0.0\n",
    );
    let hir = analyze(source).expect("source should type-check");

    let javascript = javascript_text(&hir).expect("constants should lower");

    assert!(javascript.contains("function b0() {\n    return 42;"));
    assert!(javascript.contains("function b1() {\n    return 1.5;"));
    assert!(javascript.contains("function b2() {\n    return -0;"));
}

#[test]
fn unused_composite_expressions_preserve_nested_calls() {
    let source = concat!(
        "fn observe -> f64\n",
        "    1.0\n",
        "fn main -> f64\n",
        "    observe() + 1\n",
        "    2.0\n",
    );
    let hir = analyze(source).expect("source should type-check");

    let javascript = javascript_text(&hir).expect("calls should lower");

    assert!(javascript.contains("b0() + 1;"));
}
