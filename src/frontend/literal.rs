// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: Apache-2.0

//! Contextual numeric-literal materialization.
//!
//! Function checking supplies an expected type. This module accepts only exact integer
//! conversions and finite floating-point conversions before typed HIR construction.

use crate::hir;

/// Reason a source literal cannot adopt its expected concrete type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum LiteralError {
    /// Expected type does not belong to the literal's numeric family.
    Incompatible,

    /// Integer-to-float conversion would change the mathematical value.
    InexactInteger,

    /// Value lies outside the finite range represented by the expected type.
    OutOfRange,
}

impl LiteralError {
    /// Return the stable diagnostic message for this rejection class.
    #[must_use]
    pub(super) const fn message(self) -> &'static str {
        match self {
            Self::Incompatible => "numeric literal is incompatible with the expected type",
            Self::InexactInteger => {
                "integer literal cannot be represented exactly by the expected type"
            }
            Self::OutOfRange => "numeric literal is outside the expected type's range",
        }
    }
}

/// Select an exact concrete type for one integer literal.
pub(super) fn integer_type(
    value: i128,
    expected: Option<hir::Type>,
) -> Result<hir::Type, LiteralError> {
    match expected {
        None | Some(hir::Type::I64) => {
            i64::try_from(value).map(|_| hir::Type::I64).map_err(|_| LiteralError::OutOfRange)
        }
        Some(hir::Type::I32) => {
            i32::try_from(value).map(|_| hir::Type::I32).map_err(|_| LiteralError::OutOfRange)
        }
        Some(hir::Type::F32) if integer_is_exact(value, 24) => Ok(hir::Type::F32),
        Some(hir::Type::F64) if integer_is_exact(value, 53) => Ok(hir::Type::F64),
        Some(hir::Type::F32 | hir::Type::F64) => Err(LiteralError::InexactInteger),
        Some(_) => Err(LiteralError::Incompatible),
    }
}

/// Select a finite concrete type for one floating-point literal.
pub(super) fn float_type(
    value: f64,
    expected: Option<hir::Type>,
) -> Result<hir::Type, LiteralError> {
    if !value.is_finite() {
        return Err(LiteralError::OutOfRange);
    }
    match expected {
        None | Some(hir::Type::F64) => Ok(hir::Type::F64),
        Some(hir::Type::F32) if fits_f32(value) => Ok(hir::Type::F32),
        Some(hir::Type::F32) => Err(LiteralError::OutOfRange),
        _ => Err(LiteralError::Incompatible),
    }
}

fn integer_is_exact(value: i128, precision: u32) -> bool {
    let magnitude = value.unsigned_abs();
    let significant_bits = u128::BITS - magnitude.leading_zeros();
    significant_bits <= precision || magnitude.trailing_zeros() >= significant_bits - precision
}

fn fits_f32(value: f64) -> bool {
    let narrowed = value as f32;
    narrowed.is_finite() && (value == 0.0 || narrowed != 0.0)
}
