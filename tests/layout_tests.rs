// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: Apache-2.0

//! Layout-expression evaluation invariants.

use zop::layout::{AffineLayout, LayoutError, LayoutExpr};

#[test]
fn affine_layout_evaluates_its_shape_stride_map() {
    let affine = AffineLayout::new(vec![4, 4], vec![4, 1]).expect("layout should be valid");
    let layout = LayoutExpr::from(affine);

    assert_eq!(layout.evaluate(&[2, 3]).expect("layout should evaluate"), 11);
}

#[test]
fn invariant_invalid_affine_layouts_never_produce_an_index() {
    let rank_mismatch = AffineLayout::new(vec![4, 4], vec![1]);
    assert!(matches!(rank_mismatch, Err(LayoutError::ShapeStrideRankMismatch { .. })));

    let affine = AffineLayout::new(vec![4], vec![i64::MAX]).expect("layout should be valid");
    let affine = LayoutExpr::from(affine);
    assert!(matches!(affine.evaluate(&[2]), Err(LayoutError::AffineEvaluationOverflow)));
}
