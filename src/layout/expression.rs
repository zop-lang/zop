// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: Apache-2.0

//! Exact affine layout evaluation over signed 64-bit bootstrap indices.

use std::fmt::{self, Display, Formatter};

/// Failure to construct or evaluate a valid bootstrap layout expression.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutError {
    /// Shape and stride trees have different flat ranks.
    ShapeStrideRankMismatch {
        /// Number of shape axes.
        shape_rank: usize,
        /// Number of stride axes.
        stride_rank: usize,
    },

    /// An evaluation coordinate does not match the layout rank.
    CoordinateRankMismatch {
        /// Rank required by the layout.
        expected: usize,
        /// Number of supplied coordinates.
        actual: usize,
    },

    /// Affine coordinate evaluation exceeded the signed 64-bit index.
    AffineEvaluationOverflow,
}

impl Display for LayoutError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ShapeStrideRankMismatch { shape_rank, stride_rank } => {
                write!(formatter, "shape rank {shape_rank} differs from stride rank {stride_rank}")
            }
            Self::CoordinateRankMismatch { expected, actual } => {
                write!(formatter, "layout expects {expected} coordinates, got {actual}")
            }
            Self::AffineEvaluationOverflow => {
                formatter.write_str("affine layout evaluation overflowed i64")
            }
        }
    }
}

impl std::error::Error for LayoutError {}

/// Flat affine coordinate map retained by the first executable layout slice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AffineLayout {
    /// Logical extent for each flat axis.
    shape: Box<[usize]>,

    /// Signed Engine-index contribution for each corresponding axis.
    stride: Box<[i64]>,
}

impl AffineLayout {
    /// Construct one congruent flat shape and stride map.
    ///
    /// # Errors
    ///
    /// Returns [`LayoutError::ShapeStrideRankMismatch`] when the profiles have
    /// different ranks.
    pub fn new(shape: Vec<usize>, stride: Vec<i64>) -> Result<Self, LayoutError> {
        if shape.len() != stride.len() {
            return Err(LayoutError::ShapeStrideRankMismatch {
                shape_rank: shape.len(),
                stride_rank: stride.len(),
            });
        }
        Ok(Self { shape: shape.into_boxed_slice(), stride: stride.into_boxed_slice() })
    }

    /// Return the logical flat shape.
    #[must_use]
    pub fn shape(&self) -> &[usize] {
        &self.shape
    }

    /// Return the congruent signed stride profile.
    #[must_use]
    pub fn stride(&self) -> &[i64] {
        &self.stride
    }

    fn evaluate(&self, coordinate: &[i64]) -> Result<i64, LayoutError> {
        if coordinate.len() != self.stride.len() {
            return Err(LayoutError::CoordinateRankMismatch {
                expected: self.stride.len(),
                actual: coordinate.len(),
            });
        }
        coordinate.iter().zip(&self.stride).try_fold(0_i64, |offset, (value, stride)| {
            let contribution =
                value.checked_mul(*stride).ok_or(LayoutError::AffineEvaluationOverflow)?;
            offset.checked_add(contribution).ok_or(LayoutError::AffineEvaluationOverflow)
        })
    }
}

/// Exact layout function implemented by the Rust bootstrap reference model.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LayoutExpr {
    /// Integer inner-product coordinate map.
    Affine(AffineLayout),
}

impl LayoutExpr {
    /// Evaluate one natural flat coordinate without dereferencing storage.
    ///
    /// # Errors
    ///
    /// Returns a typed layout error for rank mismatch or arithmetic overflow.
    pub fn evaluate(&self, coordinate: &[i64]) -> Result<i64, LayoutError> {
        match self {
            Self::Affine(layout) => layout.evaluate(coordinate),
        }
    }
}

impl From<AffineLayout> for LayoutExpr {
    fn from(layout: AffineLayout) -> Self {
        Self::Affine(layout)
    }
}
