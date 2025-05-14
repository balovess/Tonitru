// Batch result types for SIMD optimizations
//
// This module contains result types for batch decoding operations.
// These types allow us to return either borrowed or owned data without leaking memory.

use std::ops::Deref;

/// A batch of decoded values, either borrowed or owned.
/// This enum allows us to return either a borrowed slice or an owned vector,
/// depending on whether the data was aligned or not.
#[derive(Debug)]
pub enum BatchResult<'a, T: 'a> {
    /// A borrowed slice of values, used when the data was already aligned.
    Borrowed(&'a [T]),
    /// An owned vector of values, used when the data needed to be copied and aligned.
    Owned(Vec<T>),
}

impl<'a, T> PartialEq<[T]> for BatchResult<'a, T>
where
    T: PartialEq,
{
    fn eq(&self, other: &[T]) -> bool {
        match self {
            BatchResult::Borrowed(slice) => slice == &other,
            BatchResult::Owned(vec) => vec.as_slice() == other,
        }
    }
}

impl<'a, T, const N: usize> PartialEq<[T; N]> for BatchResult<'a, T>
where
    T: PartialEq,
{
    fn eq(&self, other: &[T; N]) -> bool {
        self.eq(other.as_slice())
    }
}

impl<'a, T> PartialEq<&[T]> for BatchResult<'a, T>
where
    T: PartialEq,
{
    fn eq(&self, other: &&[T]) -> bool {
        self.eq(*other)
    }
}

impl<'a, T> Deref for BatchResult<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        match self {
            BatchResult::Borrowed(slice) => slice,
            BatchResult::Owned(vec) => vec.as_slice(),
        }
    }
}

impl<'a, T> BatchResult<'a, T> {
    /// Get a reference to the underlying slice.
    pub fn as_slice(&self) -> &[T] {
        match self {
            BatchResult::Borrowed(slice) => slice,
            BatchResult::Owned(vec) => vec.as_slice(),
        }
    }

    /// Convert the batch result to a vector, potentially cloning the data.
    pub fn to_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        match self {
            BatchResult::Borrowed(slice) => slice.to_vec(),
            BatchResult::Owned(vec) => vec.clone(),
        }
    }

    /// Convert the batch result into a vector, taking ownership if possible.
    pub fn into_vec(self) -> Vec<T>
    where
        T: Clone,
    {
        match self {
            BatchResult::Borrowed(slice) => slice.to_vec(),
            BatchResult::Owned(vec) => vec,
        }
    }

    /// Create a new borrowed batch result.
    pub fn borrowed(slice: &'a [T]) -> Self {
        BatchResult::Borrowed(slice)
    }

    /// Create a new owned batch result.
    pub fn owned(vec: Vec<T>) -> Self {
        BatchResult::Owned(vec)
    }
}
