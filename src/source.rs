// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: Apache-2.0

//! Source byte ranges shared by compiler stages.
//!
//! Diagnostics retain these spans as source becomes syntax and typed intermediate forms.

use std::ops::Range;

/// A half-open byte range in one source file.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Span {
    /// Inclusive UTF-8 byte offset from the start of the source file.
    pub start: usize,

    /// Exclusive UTF-8 byte offset from the start of the source file.
    pub end: usize,
}

impl Span {
    /// Construct a half-open byte range.
    #[must_use]
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Construct an empty range at one byte offset.
    #[must_use]
    pub const fn empty(offset: usize) -> Self {
        Self::new(offset, offset)
    }
}

impl From<Range<usize>> for Span {
    fn from(range: Range<usize>) -> Self {
        Self::new(range.start, range.end)
    }
}
