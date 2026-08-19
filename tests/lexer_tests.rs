// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: MIT

//! Raw-token and indentation-layout lexer tests.

use zop::lexer::{TokenKind, lex};

fn kinds(source: &str) -> Vec<TokenKind> {
    let tokens = lex(source).expect("source should lex");
    tokens.into_iter().map(|token| token.kind).collect()
}

#[test]
fn layout_is_part_of_the_token_contract() {
    let source = "fn add x: i64, y: i64 -> i64\n    x + y\n";

    assert_eq!(
        kinds(source),
        vec![
            TokenKind::Fn,
            TokenKind::Identifier,
            TokenKind::Identifier,
            TokenKind::Colon,
            TokenKind::Identifier,
            TokenKind::Comma,
            TokenKind::Identifier,
            TokenKind::Colon,
            TokenKind::Identifier,
            TokenKind::Arrow,
            TokenKind::Identifier,
            TokenKind::Newline,
            TokenKind::Indent,
            TokenKind::Identifier,
            TokenKind::Plus,
            TokenKind::Identifier,
            TokenKind::Newline,
            TokenKind::Dedent,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn delimiters_suppress_layout_until_they_close() {
    let source = "fn add(\n    x: i64,\n    y: i64,\n) -> i64\n    x + y\n";

    let tokens = kinds(source);
    assert_eq!(tokens.iter().filter(|kind| **kind == TokenKind::Newline).count(), 2);
    assert_eq!(tokens.iter().filter(|kind| **kind == TokenKind::Indent).count(), 1);
    assert_eq!(tokens.iter().filter(|kind| **kind == TokenKind::Dedent).count(), 1);
}

#[test]
fn leading_tabs_are_rejected() {
    let errors = lex("fn main\n\t1\n").expect_err("tab indentation must fail");

    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code, "L0002");
}

#[test]
fn dedents_must_match_an_enclosing_block() {
    let source = "fn main\n    if true\n        1\n      2\n";
    let errors = lex(source).expect_err("misaligned dedent must fail");

    assert!(errors.iter().any(|error| error.code == "L0003"));
}

#[test]
fn semicolons_are_not_tokens() {
    let errors = lex("fn main\n    1;\n").expect_err("semicolon must fail");

    assert!(errors.iter().any(|error| error.code == "L0001"));
}

#[test]
fn comments_do_not_change_layout() {
    let source = "fn main\n    # before\n    42 # after\n";

    assert_eq!(
        kinds(source),
        vec![
            TokenKind::Fn,
            TokenKind::Identifier,
            TokenKind::Newline,
            TokenKind::Indent,
            TokenKind::Integer,
            TokenKind::Newline,
            TokenKind::Dedent,
            TokenKind::Eof,
        ]
    );
}
