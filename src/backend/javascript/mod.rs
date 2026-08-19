// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: Apache-2.0

//! ECMAScript module emission from typed high-level intermediate representation.

mod ast;
mod emit;
mod lower;
mod optimize;
mod printer;

pub use emit::javascript_text;
