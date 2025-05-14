// Prefetch module for pipeline processor
//
// This module defines the prefetch stage of the pipeline processor, which is responsible for
// preparing data for efficient processing by ensuring proper alignment and providing a clear
// indication of whether the data is aligned or has been copied.

use std::mem::{align_of, size_of};
use std::ops::Deref;
use crate::internal::error::{Error, Result};

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

/// Trait for types that can be converted from little-endian bytes.
pub trait FromLeBytes: Sized {
    /// Converts a byte array to Self.
    fn from_le_bytes(bytes: &[u8]) -> Self;
}

// Implement FromLeBytes for common types
impl FromLeBytes for u8 {
    fn from_le_bytes(bytes: &[u8]) -> Self {
        bytes[0]
    }
}

impl FromLeBytes for i8 {
    fn from_le_bytes(bytes: &[u8]) -> Self {
        bytes[0] as i8
    }
}

impl FromLeBytes for u16 {
    fn from_le_bytes(bytes: &[u8]) -> Self {
        u16::from_le_bytes([bytes[0], bytes[1]])
    }
}

impl FromLeBytes for i16 {
    fn from_le_bytes(bytes: &[u8]) -> Self {
        i16::from_le_bytes([bytes[0], bytes[1]])
    }
}

impl FromLeBytes for u32 {
    fn from_le_bytes(bytes: &[u8]) -> Self {
        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }
}

impl FromLeBytes for i32 {
    fn from_le_bytes(bytes: &[u8]) -> Self {
        i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }
}

impl FromLeBytes for f32 {
    fn from_le_bytes(bytes: &[u8]) -> Self {
        f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }
}

impl FromLeBytes for u64 {
    fn from_le_bytes(bytes: &[u8]) -> Self {
        u64::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]])
    }
}

impl FromLeBytes for i64 {
    fn from_le_bytes(bytes: &[u8]) -> Self {
        i64::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]])
    }
}

impl FromLeBytes for f64 {
    fn from_le_bytes(bytes: &[u8]) -> Self {
        f64::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]])
    }
}

/// Trait for types that can be safely reinterpreted from bytes.
pub trait Pod: Copy + 'static {}

// Implement Pod for common types
impl Pod for u8 {}
impl Pod for i8 {}
impl Pod for u16 {}
impl Pod for i16 {}
impl Pod for u32 {}
impl Pod for i32 {}
impl Pod for f32 {}
impl Pod for u64 {}
impl Pod for i64 {}
impl Pod for f64 {}

/// Prepares an aligned batch of values from raw bytes.
/// This function is responsible for ensuring proper alignment and providing a clear
/// indication of whether the data is aligned or has been copied.
pub fn prepare_aligned_batch<'a, T: Pod + FromLeBytes>(
    raw: &'a [u8]
) -> Result<(AlignedBatch<'a, T>, usize)> {
    let type_size = size_of::<T>();
    
    // Check if data length is valid
    if raw.len() % type_size != 0 {
        return Err(Error::CodecError(format!(
            "Invalid data length for batch decoding. Length ({}) must be a multiple of {}",
            raw.len(),
            type_size
        )));
    }

    let count = raw.len() / type_size;
    if count == 0 {
        return Ok((AlignedBatch::borrowed(&[]), 0));
    }

    // Check alignment
    let ptr = raw.as_ptr();
    let is_aligned = (ptr as usize) % align_of::<T>() == 0;

    if is_aligned {
        // For aligned data, we can simply reinterpret the slice
        // This is safe because we've already checked size and alignment
        let slice = unsafe {
            std::slice::from_raw_parts(ptr as *const T, count)
        };
        
        Ok((AlignedBatch::borrowed(slice), raw.len()))
    } else {
        // For unaligned data, we need to copy and align
        let mut values = Vec::with_capacity(count);
        
        // Process data in chunks of type_size
        for chunk in raw.chunks_exact(type_size) {
            values.push(T::from_le_bytes(chunk));
        }
        
        Ok((AlignedBatch::owned(values), raw.len()))
    }
}
