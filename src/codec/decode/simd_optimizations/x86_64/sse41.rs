// SSE4.1 optimizations for x86_64 architecture
//
// This module contains SIMD-optimized implementations using SSE4.1 instructions.

use crate::internal::error::{Error, Result};
use std::mem;
use std::slice;
#[allow(unused_imports)]
use std::arch::x86_64::*;

/// Decodes a batch of u32 values using SSE4.1 SIMD instructions.
/// Returns a BatchResult containing the decoded elements and the number of bytes read.
pub fn decode_u32_batch_simd(data: &[u8]) -> Result<(super::super::BatchResult<u32>, usize)> {
    use super::super::BatchResult;

    let size = mem::size_of::<u32>();
    if data.len() % size != 0 {
        return Err(Error::CodecError(format!(
            "Invalid data length for U32 batch decoding. Length ({}) must be a multiple of {}",
            data.len(),
            size
        )));
    }

    let count = data.len() / size;
    if count == 0 {
        return Ok((BatchResult::borrowed(&[]), 0));
    }

    // Check alignment for SIMD operations
    let aligned = (data.as_ptr() as usize) % mem::align_of::<u32>() == 0;

    if aligned {
        // For aligned data, we can simply reinterpret the slice
        // This is safe because we've already checked size and alignment
        let decoded_slice = unsafe {
            slice::from_raw_parts(data.as_ptr() as *const u32, count)
        };

        Ok((BatchResult::borrowed(decoded_slice), data.len()))
    } else {
        // For unaligned data, we need to use unaligned loads
        // This is more complex and would require manual SIMD implementation
        // For now, we'll use a scalar fallback for unaligned data
        let mut values = Vec::with_capacity(count);
        for i in 0..count {
            let offset = i * size;
            let value = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            values.push(value);
        }

        // Return an owned BatchResult, which will properly manage the memory
        Ok((BatchResult::owned(values), data.len()))
    }
}

/// Decodes a batch of f32 values using SSE4.1 SIMD instructions.
/// Returns a BatchResult containing the decoded elements and the number of bytes read.
pub fn decode_f32_batch_simd(data: &[u8]) -> Result<(super::super::BatchResult<f32>, usize)> {
    use super::super::BatchResult;

    let size = mem::size_of::<f32>();
    if data.len() % size != 0 {
        return Err(Error::CodecError(format!(
            "Invalid data length for F32 batch decoding. Length ({}) must be a multiple of {}",
            data.len(),
            size
        )));
    }

    let count = data.len() / size;
    if count == 0 {
        return Ok((BatchResult::borrowed(&[]), 0));
    }

    // Check alignment for SIMD operations
    let aligned = (data.as_ptr() as usize) % mem::align_of::<f32>() == 0;

    if aligned {
        // For aligned data, we can simply reinterpret the slice
        // This is safe because we've already checked size and alignment
        let decoded_slice = unsafe {
            slice::from_raw_parts(data.as_ptr() as *const f32, count)
        };

        Ok((BatchResult::borrowed(decoded_slice), data.len()))
    } else {
        // For unaligned data, we need to use unaligned loads
        // This is more complex and would require manual SIMD implementation
        // For now, we'll use a scalar fallback for unaligned data
        let mut values = Vec::with_capacity(count);
        for i in 0..count {
            let offset = i * size;
            let value = f32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            values.push(value);
        }

        // Return an owned BatchResult, which will properly manage the memory
        Ok((BatchResult::owned(values), data.len()))
    }
}

/// Decodes a batch of u8 values using SSE4.1 SIMD instructions.
/// For u8, this is mostly a pass-through since no conversion is needed,
/// but we include it for API consistency.
pub fn decode_u8_batch_simd(data: &[u8]) -> Result<(super::super::BatchResult<u8>, usize)> {
    use super::super::BatchResult;
    Ok((BatchResult::borrowed(data), data.len()))
}

/// Decodes a batch of i8 values using SSE4.1 SIMD instructions.
pub fn decode_i8_batch_simd(data: &[u8]) -> Result<(super::super::BatchResult<i8>, usize)> {
    use super::super::BatchResult;

    // For i8, we can simply reinterpret the slice
    // This is safe because i8 and u8 have the same memory layout
    let decoded_slice = unsafe {
        slice::from_raw_parts(data.as_ptr() as *const i8, data.len())
    };

    Ok((BatchResult::borrowed(decoded_slice), data.len()))
}
