// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: MIT

//! Rust bootstrap compiler for the Zop programming language.
//!
//! The crate exposes phase-specific lexer, syntax, frontend, intermediate
//! representation, and backend modules. Unsupported language features return
//! structured diagnostics before entering a weaker compiler path.

#![warn(unsafe_code)]

pub mod backend;
pub mod diagnostic;
pub mod frontend;
pub mod hir;
pub mod lexer;
pub mod source;
pub mod syntax;
