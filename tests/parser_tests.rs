// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: Apache-2.0

//! Surface syntax structure and rejection tests.

use zop::syntax::{BinaryOperator, ExpressionKind, FunctionKind, ParameterMode, parse};

#[test]
fn function_signature_and_tail_expression_are_structured() {
    let source = "fn add x: i64, y: i64 -> i64\n    x + y\n";
    let module = parse(source).expect("function should parse");
    let function = &module.functions[0];

    assert_eq!(function.kind, FunctionKind::Host);
    assert_eq!(function.name, "add");
    assert_eq!(function.parameters.len(), 2);
    assert_eq!(function.parameters[0].mode, ParameterMode::Borrow);
    assert_eq!(function.parameters[0].name, "x");
    assert_eq!(function.parameters[0].ty.name, "i64");
    assert_eq!(function.return_type.as_ref().map(|ty| ty.name.as_str()), Some("i64"));
    assert!(matches!(
        function.body.expressions[0].kind,
        ExpressionKind::Binary { operator: BinaryOperator::Add, .. }
    ));
}

#[test]
fn calls_preserve_labels_and_full_argument_expressions() {
    let source = "fn main -> i64\n    print 1 + 2, radix=10\n";
    let module = parse(source).expect("call should parse");
    let expression = &module.functions[0].body.expressions[0];
    let ExpressionKind::Call { arguments, .. } = &expression.kind else {
        panic!("expected call expression");
    };

    assert_eq!(arguments.len(), 2);
    assert_eq!(arguments[0].label, None);
    assert!(matches!(
        arguments[0].value.kind,
        ExpressionKind::Binary { operator: BinaryOperator::Add, .. }
    ));
    assert_eq!(arguments[1].label.as_deref(), Some("radix"));
}

#[test]
fn named_arguments_require_equals() {
    let source = "fn main -> i64\n    print 1, radix: 10\n";
    let errors = parse(source).expect_err("named arguments use equals");

    assert!(errors.iter().any(|error| error.code == "P0011"));
}

#[test]
fn expressions_require_a_physical_line_boundary() {
    let source = concat!(
        "fn identity value: i64 -> i64\n",
        "    value\n",
        "fn main y: i64 -> i64\n",
        "    identity y y\n",
    );
    let errors = parse(source).expect_err("same-line expressions must not split implicitly");

    assert!(errors.iter().any(|error| error.code == "P0011"));
}

#[test]
fn multiline_parameters_require_commas_and_accept_a_trailing_comma() {
    let source = "fn add(\n    x: i64,\n    mut y: i64,\n) -> i64\n    x + y\n";
    let module = parse(source).expect("multiline declaration should parse");
    let parameters = &module.functions[0].parameters;

    assert_eq!(parameters.len(), 2);
    assert_eq!(parameters[1].mode, ParameterMode::Mut);
}

#[test]
fn bare_catch_cannot_construct_syntax() {
    let source = "fn main -> i64\n    load() catch\n        0\n";
    let errors = parse(source).expect_err("bare catch must fail");

    assert!(errors.iter().any(|error| error.code == "P0012"));
}

#[test]
fn fail_requires_with_and_an_error_value() {
    let source = "fn main -> i64 or fails with LoadError\n    fail Missing\n";
    let errors = parse(source).expect_err("incomplete fail must fail");

    assert!(errors.iter().any(|error| error.code == "P0013"));
}

#[test]
fn string_escapes_are_decoded_once() {
    let source = r#"fn main -> str
    "\\n"
"#;
    let module = parse(source).expect("escaped string should parse");
    let expression = &module.functions[0].body.expressions[0];

    assert!(matches!(&expression.kind, ExpressionKind::String(value) if value == "\\n"));
}

#[test]
fn unknown_string_escapes_are_rejected() {
    let source = r#"fn main -> str
    "\q"
"#;
    let errors = parse(source).expect_err("unknown escape should fail");

    assert!(errors.iter().any(|error| error.code == "P0015"));
}
