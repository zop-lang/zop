// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: MIT

//! Native object emission tests.

use zop::{backend::compile_object, frontend::analyze};

#[test]
fn cranelift_aot_emits_a_native_object() {
    let source = "fn answer -> i64\n    42\n";
    let hir = analyze(source).expect("source should type-check");
    let object = compile_object(&hir).expect("AOT compilation should succeed");

    assert!(!object.is_empty());
}
