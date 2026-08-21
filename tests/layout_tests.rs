// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: Apache-2.0

//! Layout-expression evaluation invariants.

use zop::layout::{AffineLayout, LayoutError, LayoutExpr, Swizzle, SwizzleSpec};

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

#[test]
fn swizzle_matches_cute_bit_field_semantics() {
    let affine = AffineLayout::new(vec![64], vec![1]).expect("layout should be valid");
    let swizzle =
        Swizzle::new(SwizzleSpec { bits: 3, base: 0, shift: 3 }).expect("swizzle should be valid");
    let layout = LayoutExpr::compose(swizzle, 0, affine.into());

    assert_eq!(layout.evaluate(&[19]).expect("layout should evaluate"), 17);
}

#[test]
fn invariant_invalid_compositions_never_produce_an_index() {
    let identity = Swizzle::new(SwizzleSpec { bits: 0, base: 0, shift: 0 })
        .expect("identity swizzle should be valid");
    let scalar = AffineLayout::new(vec![], vec![]).expect("scalar layout should be valid");
    let negative = LayoutExpr::compose(identity, -1, scalar.into());
    assert!(matches!(negative.evaluate(&[]), Err(LayoutError::NegativeSwizzleInput { input: -1 })));

    assert!(matches!(
        Swizzle::new(SwizzleSpec { bits: 4, base: 60, shift: 4 }),
        Err(LayoutError::InvalidSwizzle { .. })
    ));
}
