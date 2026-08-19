// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: MIT

//! Name resolution, typing, and semantic rejection tests.

use zop::{
    frontend::analyze,
    hir::{BinaryOperator, ExpressionKind, Type},
};

#[test]
fn scalar_function_is_resolved_and_typed() {
    let source = "fn add x: i64, y: i64 -> i64\n    x + y\n";
    let module = analyze(source).expect("function should type-check");
    let function = module.function("add").expect("function should exist");

    assert_eq!(function.result, Type::I64);
    assert!(matches!(
        function.body.expressions[0].kind,
        ExpressionKind::Binary { operator: BinaryOperator::Add, .. }
    ));
}

#[test]
fn local_assignment_cannot_change_type() {
    let source = "fn main -> i64\n    value = 1\n    value = true\n    value\n";
    let errors = analyze(source).expect_err("type-changing assignment must fail");

    assert!(errors.iter().any(|error| error.code == "S0005"));
}

#[test]
fn unknown_names_are_rejected_before_lowering() {
    let source = "fn main -> i64\n    missing + 1\n";
    let errors = analyze(source).expect_err("unknown name must fail");

    assert!(errors.iter().any(|error| error.code == "S0002"));
}

#[test]
fn named_arguments_are_arranged_in_parameter_order() {
    let source = concat!(
        "fn subtract left: i64, right: i64 -> i64\n",
        "    left - right\n",
        "fn main -> i64\n",
        "    subtract right=2, left=5\n",
    );
    let module = analyze(source).expect("named call should type-check");
    let main = module.function("main").expect("main should exist");
    let ExpressionKind::Call { arguments, .. } = &main.body.expressions[0].kind else {
        panic!("expected call expression");
    };

    assert!(matches!(arguments[0].kind, ExpressionKind::Integer(5)));
    assert!(matches!(arguments[1].kind, ExpressionKind::Integer(2)));
}

#[test]
fn function_result_must_match_its_annotation() {
    let source = "fn main -> i64\n    true\n";
    let errors = analyze(source).expect_err("wrong result type must fail");

    assert!(errors.iter().any(|error| error.code == "S0004"));
}

#[test]
fn ownership_modes_stop_before_hir_until_the_checker_exists() {
    let source = "fn update mut value: i64 -> i64\n    value\n";
    let errors = analyze(source).expect_err("unchecked ownership must not reach HIR");

    assert!(errors.iter().any(|error| error.code == "S0001"));
}

#[test]
fn error_channels_stop_before_hir_until_the_checker_exists() {
    let source = "fn load -> i64 or fails with LoadError\n    42\n";
    let errors = analyze(source).expect_err("unchecked errors must not reach HIR");

    assert!(errors.iter().any(|error| error.code == "S0001"));
}

#[test]
fn parameter_names_are_unique_within_a_signature() {
    let source = "fn choose value: i64, value: i64 -> i64\n    value\n";
    let errors = analyze(source).expect_err("duplicate parameters must fail");

    assert!(errors.iter().any(|error| error.code == "S0003"));
}

#[test]
fn invariant_forward_calls_resolve_from_signatures() {
    let source = concat!(
        "fn first value: i64 -> i64\n",
        "    second value\n",
        "fn second value: i64 -> i64\n",
        "    value\n",
    );
    let module = analyze(source).expect("peer signatures must resolve before bodies");
    let first = module.function("first").expect("first function should exist");

    assert!(matches!(first.body.expressions[0].kind, ExpressionKind::Call { .. }));
}

#[test]
fn invariant_recursive_functions_require_explicit_result_types() {
    let cases = [
        ("direct", "fn recurse\n    recurse()\n", 1),
        ("mutual", concat!("fn first\n", "    second()\n", "fn second\n", "    first()\n"), 2),
        (
            "caller outside cycle",
            concat!(
                "fn entry\n",
                "    first()\n",
                "fn first\n",
                "    second()\n",
                "fn second\n",
                "    first()\n",
            ),
            2,
        ),
    ];

    for (name, source, expected) in cases {
        let errors = analyze(source).expect_err(name);
        assert_eq!(errors.iter().filter(|error| error.code == "S0010").count(), expected, "{name}");
    }
}

#[test]
fn invariant_explicit_result_types_close_recursive_signatures() {
    let source =
        concat!("fn first -> i64\n", "    second()\n", "fn second -> i64\n", "    first()\n",);

    analyze(source).expect("explicit recursive signatures should type-check");
}

#[test]
fn expressions_cannot_follow_return() {
    let source = "fn answer -> i64\n    return 42\n    0\n";
    let errors = analyze(source).expect_err("unreachable expression must fail");

    assert!(errors.iter().any(|error| error.code == "S0008"));
}

#[test]
fn numeric_literals_adopt_the_surrounding_concrete_type() {
    let source = concat!(
        "fn add_one value: f32 -> f32\n",
        "    value + 1\n",
        "fn identity value: f32 -> f32\n",
        "    value\n",
        "fn main -> f32\n",
        "    identity 1.1\n",
    );
    let module = analyze(source).expect("context should type numeric literals");
    let add_one = module.function("add_one").expect("function should exist");
    let ExpressionKind::Binary { right, .. } = &add_one.body.expressions[0].kind else {
        panic!("expected binary expression");
    };
    let main = module.function("main").expect("function should exist");
    let ExpressionKind::Call { arguments, .. } = &main.body.expressions[0].kind else {
        panic!("expected call expression");
    };

    assert_eq!(right.ty, Type::F32);
    assert_eq!(arguments[0].ty, Type::F32);
}

#[test]
fn concrete_numeric_values_never_promote_implicitly() {
    let source = "fn add left: f32, right: i32 -> f32\n    left + right\n";
    let errors = analyze(source).expect_err("concrete types must remain distinct");

    assert!(errors.iter().any(|error| error.code == "S0004"));
}

#[test]
fn contextual_literals_must_fit_the_selected_type() {
    let source = concat!(
        "fn integer value: i32 -> i32\n",
        "    value + 2147483648\n",
        "fn float value: f32 -> f32\n",
        "    value + 16777217\n",
        "fn overflow value: f32 -> f32\n",
        "    value + 1.0e100\n",
        "fn underflow value: f32 -> f32\n",
        "    value + 1.0e-100\n",
    );
    let errors = analyze(source).expect_err("lossy literals must fail");

    assert_eq!(errors.iter().filter(|error| error.code == "S0009").count(), 4);
}

#[test]
fn minimum_signed_literals_fit_their_context() {
    let source = concat!(
        "fn minimum_i32 -> i32\n",
        "    -2147483648\n",
        "fn minimum_i64 -> i64\n",
        "    -9223372036854775808\n",
    );
    let module = analyze(source).expect("signed minimum literals should type-check");
    let minimum_i32 = module.function("minimum_i32").expect("function should exist");
    let minimum_i64 = module.function("minimum_i64").expect("function should exist");

    assert!(
        matches!(minimum_i32.body.expressions[0].kind, ExpressionKind::Integer(value) if value == i32::MIN as i128)
    );
    assert!(
        matches!(minimum_i64.body.expressions[0].kind, ExpressionKind::Integer(value) if value == i64::MIN as i128)
    );
}
