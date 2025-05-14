// AlignedBatch type for pipeline processor
//
// This module defines the AlignedBatch enum, which encapsulates the result of the prefetch stage
// and provides a clear indication of whether the data is aligned or has been copied.

use std::ops::Deref;

/// A batch of values that may be borrowed from aligned data or owned from copied data.
/// This enum provides a clear indication of whether the data is aligned or has been copied.
#[derive(Debug)]
pub enum AlignedBatch<'a, T: 'a> {
    /// A borrowed slice of values, used when the original data was already aligned.
    /// This variant enables zero-copy parsing.
    Borrowed(&'a [T]),
    
    /// An owned vector of values, used when the original data needed to be copied and aligned.
    /// This variant ensures proper memory management for unaligned data.
    Owned(Vec<T>),
}

impl<'a, T> AlignedBatch<'a, T> {
    /// Creates a new AlignedBatch::Borrowed variant.
    pub fn borrowed(slice: &'a [T]) -> Self {
        AlignedBatch::Borrowed(slice)
    }

    /// Creates a new AlignedBatch::Owned variant.
    pub fn owned(vec: Vec<T>) -> Self {
        AlignedBatch::Owned(vec)
    }

    /// Returns a reference to the underlying slice.
    pub fn as_slice(&self) -> &[T] {
        match self {
            AlignedBatch::Borrowed(slice) => slice,
            AlignedBatch::Owned(vec) => vec.as_slice(),
        }
    }

    /// Returns true if the data is aligned (Borrowed variant).
    pub fn is_aligned(&self) -> bool {
        matches!(self, AlignedBatch::Borrowed(_))
    }

    /// Converts the batch to a vector, potentially cloning the data.
    pub fn to_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        match self {
            AlignedBatch::Borrowed(slice) => slice.to_vec(),
            AlignedBatch::Owned(vec) => vec.clone(),
        }
    }

    /// Converts the batch into a vector, taking ownership if possible.
    pub fn into_vec(self) -> Vec<T>
    where
        T: Clone,
    {
        match self {
            AlignedBatch::Borrowed(slice) => slice.to_vec(),
            AlignedBatch::Owned(vec) => vec,
        }
    }
}

impl<'a, T> Deref for AlignedBatch<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<'a, T> PartialEq<[T]> for AlignedBatch<'a, T>
where
    T: PartialEq,
{
    fn eq(&self, other: &[T]) -> bool {
        self.as_slice() == other
    }
}

impl<'a, T, const N: usize> PartialEq<[T; N]> for AlignedBatch<'a, T>
where
    T: PartialEq,
{
    fn eq(&self, other: &[T; N]) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<'a, T> PartialEq<&[T]> for AlignedBatch<'a, T>
where
    T: PartialEq,
{
    fn eq(&self, other: &&[T]) -> bool {
        self.as_slice() == *other
    }
}
