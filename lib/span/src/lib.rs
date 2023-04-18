#![no_std]

use core::ops::Range;
#[cfg(feature = "use_serde")]
use serde::{Serialize, Deserialize};

/// Struct to store a span.
/// A span is composed from a start offset and span length.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
pub struct Span {
    pub offset: usize,
    pub length: usize,
}

impl Span {
    /// Construct a `Span` from raw offset and length value
    #[inline]
    pub fn new(offset: usize, length: usize) -> Self {
        Self {
            offset,
            length,
        }
    }
}

impl From<Range<usize>> for Span {
    fn from(value: Range<usize>) -> Self {
        Self {
            offset: value.start,
            length: value.end - value.start,
        }
    }
}

impl From<Span> for Range<usize> {
    fn from(value: Span) -> Self {
        Self {
            start: value.offset,
            end: value.offset + value.length,
        }
    }
}
