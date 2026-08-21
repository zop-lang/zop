// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: Apache-2.0

//! Bootstrap reference semantics for language-native layout expressions.
//!
//! This module proves coordinate-to-index behavior before tensor HIR, storage,
//! or target lowering exists.

mod expression;

pub use expression::{AffineLayout, LayoutError, LayoutExpr, Swizzle, SwizzleSpec};
