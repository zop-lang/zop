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

    /// Swizzle bit fields do not fit in a nonnegative signed index.
    InvalidSwizzle {
        /// Number of bits requested in each field.
        bits: u32,
        /// Number of low-order bits below the destination field.
        base: u32,
        /// Signed distance between the interacting fields.
        shift: i32,
    },

    /// A swizzle received a negative intermediate index.
    NegativeSwizzleInput {
        /// Negative value that cannot enter a CuTe swizzle.
        input: i64,
    },

    /// Internal composition offset exceeded the signed 64-bit index.
    CompositionOffsetOverflow,
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
            Self::InvalidSwizzle { bits, base, shift } => write!(
                formatter,
                "swizzle bits={bits}, base={base}, shift={shift} exceeds nonnegative i64"
            ),
            Self::NegativeSwizzleInput { input } => {
                write!(formatter, "swizzle input {input} must be nonnegative")
            }
            Self::CompositionOffsetOverflow => {
                formatter.write_str("layout composition offset overflowed i64")
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

/// CuTe-style exclusive-or permutation over two fixed-width bit fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Swizzle {
    /// Number of bits in each interacting field.
    bits: u32,

    /// Number of low-order bits left unchanged.
    base: u32,

    /// Signed distance from the destination field to the source field.
    shift: i32,
}

/// Named CuTe bit-field parameters that cannot be transposed at a call site.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SwizzleSpec {
    /// Number of bits in each interacting field.
    pub bits: u32,

    /// Number of low-order bits left unchanged.
    pub base: u32,

    /// Signed distance from the destination field to the source field.
    pub shift: i32,
}

impl Swizzle {
    /// Construct a swizzle whose fields fit in a nonnegative signed index.
    ///
    /// # Errors
    ///
    /// Returns [`LayoutError::InvalidSwizzle`] when either field extends past
    /// bit 62.
    pub fn new(spec: SwizzleSpec) -> Result<Self, LayoutError> {
        let SwizzleSpec { bits, base, shift } = spec;
        let highest = base
            .checked_add(shift.unsigned_abs())
            .and_then(|position| position.checked_add(bits))
            .ok_or(LayoutError::InvalidSwizzle { bits, base, shift })?;
        if highest > i64::BITS - 1 {
            return Err(LayoutError::InvalidSwizzle { bits, base, shift });
        }
        Ok(Self { bits, base, shift })
    }

    fn apply(self, index: i64) -> Result<i64, LayoutError> {
        let mut index =
            u64::try_from(index).map_err(|_| LayoutError::NegativeSwizzleInput { input: index })?;
        let field = if self.bits == 0 { 0 } else { (1_u64 << self.bits) - 1 };
        let mask = field << self.base;
        if self.shift >= 0 {
            index ^= (index >> self.shift.unsigned_abs()) & mask;
        } else {
            index ^= (index & mask) << self.shift.unsigned_abs();
        }
        i64::try_from(index).map_err(|_| LayoutError::InvalidSwizzle {
            bits: self.bits,
            base: self.base,
            shift: self.shift,
        })
    }
}

/// Exact layout function implemented by the Rust bootstrap reference model.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LayoutExpr {
    /// Integer inner-product coordinate map.
    Affine(AffineLayout),

    /// Swizzle applied after an internal offset and inner layout evaluation.
    Compose {
        /// Nonlinear outer coordinate map.
        outer: Swizzle,

        /// Signed contribution applied before the outer map.
        offset: i64,

        /// Layout that owns the logical coordinate domain.
        inner: Box<LayoutExpr>,
    },
}

impl LayoutExpr {
    /// Compose one swizzle outside an internal offset and inner layout.
    #[must_use]
    pub fn compose(outer: Swizzle, offset: i64, inner: Self) -> Self {
        Self::Compose { outer, offset, inner: Box::new(inner) }
    }

    /// Evaluate one natural flat coordinate without dereferencing storage.
    ///
    /// # Errors
    ///
    /// Returns a typed layout error for rank mismatch or arithmetic overflow.
    pub fn evaluate(&self, coordinate: &[i64]) -> Result<i64, LayoutError> {
        match self {
            Self::Affine(layout) => layout.evaluate(coordinate),
            Self::Compose { outer, offset, inner } => {
                let inner = inner.evaluate(coordinate)?;
                let input =
                    offset.checked_add(inner).ok_or(LayoutError::CompositionOffsetOverflow)?;
                outer.apply(input)
            }
        }
    }
}

impl From<AffineLayout> for LayoutExpr {
    fn from(layout: AffineLayout) -> Self {
        Self::Affine(layout)
    }
}
