//! A dedicated `Span` object for AST.
//! `core::ops::Range` is not copy. That's because `Range` can be iterators.
//! We don't use a span as an iterator itself. Using `Clone` is just awkward.

use core::{hash::Hash, ops::Range};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span<T=usize> {
    pub start: T,
    pub end: T,
}

impl<T> From<Range<T>> for Span<T> {
    fn from(value: Range<T>) -> Self {
        Self { start: value.start, end: value.end }
    }
}

impl<T> From<Span<T>> for Range<T> {
    fn from(value: Span<T>) -> Self {
        Self { start: value.start, end: value.end }
    }
}

impl<T> Span<T> {
    pub fn new(start: T, end: T) -> Self {
        Self { start, end }
    }

    pub fn to_range(self) -> Range<T> {
        self.into()
    }

    pub fn from_range(value: Range<T>) -> Self {
        value.into()
    }
}
