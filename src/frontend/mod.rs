// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: Apache-2.0

//! Source parsing, name resolution, and type checking.
//!
//! Successful analysis produces typed high-level intermediate representation for lowering.

mod checker;
mod function;
mod literal;

pub use checker::analyze;
