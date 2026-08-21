// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: Apache-2.0

//! Layout-expression evaluation and slicing invariants.

use zop::layout::{AffineLayout, LayoutError, LayoutExpr, SliceCoordinate, Swizzle, SwizzleSpec};

#[test]
fn invariant_slicing_preserves_every_layout_expression_address() {
    let affine = AffineLayout::new(vec![4, 4], vec![4, 1]).expect("layout should be valid");
    let affine = LayoutExpr::from(affine);
    let swizzled = LayoutExpr::compose(
        Swizzle::new(SwizzleSpec { bits: 2, base: 0, shift: 2 }).expect("swizzle should be valid"),
        0,
        affine.clone(),
    );
    let nested = LayoutExpr::compose(
        Swizzle::new(SwizzleSpec { bits: 1, base: 0, shift: 1 })
            .expect("outer swizzle should be valid"),
        0,
        swizzled.clone(),
    );

    for parent in [affine, swizzled, nested] {
        let sliced = parent
            .slice(&[SliceCoordinate::Fixed(1), SliceCoordinate::Free])
            .expect("slice should be valid");

        for column in 0..4_i64 {
            let parent_address = parent.evaluate(&[1, column]).expect("parent should evaluate");
            let residual = sliced.layout.evaluate(&[column]).expect("residual should evaluate");

            assert_eq!(sliced.engine_delta + residual, parent_address);
        }
    }
}

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

#[test]
fn invariant_invalid_slices_never_produce_a_residual_layout() {
    let affine = AffineLayout::new(vec![3], vec![i64::MAX]).expect("layout should be valid");
    let affine = LayoutExpr::from(affine);

    assert!(matches!(
        affine.slice(&[SliceCoordinate::Fixed(3)]),
        Err(LayoutError::FixedCoordinateOutOfBounds { .. })
    ));
    assert!(matches!(
        affine.slice(&[SliceCoordinate::Fixed(2)]),
        Err(LayoutError::SliceOffsetOverflow)
    ));
}
