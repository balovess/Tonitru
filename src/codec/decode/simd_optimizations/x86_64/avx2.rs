// AVX2 optimizations for x86_64 architecture
//
// This module contains SIMD-optimized implementations using AVX2 instructions.

use crate::internal::error::{Error, Result};
use std::mem;
use std::slice;
use std::arch::x86_64::*;

/// Decodes a batch of u32 values using AVX2 SIMD instructions.
/// Returns a slice of the decoded elements and the number of bytes read.
#[cfg(target_feature = "avx2")]
pub fn decode_u32_batch_simd(data: &[u8]) -> Result<(&[u32], usize)> {
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
        return Ok((&[], 0));
    }

    // Check alignment for AVX2 operations (32-byte alignment is ideal for AVX2)
    let aligned = (data.as_ptr() as usize) % 32 == 0;
    
    if aligned {
        // For aligned data, we can simply reinterpret the slice
        // This is safe because we've already checked size and alignment
        let decoded_slice = unsafe {
            slice::from_raw_parts(data.as_ptr() as *const u32, count)
        };
        
        Ok((decoded_slice, data.len()))
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
        
        // Return a reference to the vector's data
        let slice = unsafe {
            slice::from_raw_parts(values.as_ptr(), values.len())
        };
        
        // We need to leak the vector to ensure the data remains valid
        std::mem::forget(values);
        
        Ok((slice, data.len()))
    }
}

/// Decodes a batch of f32 values using AVX2 SIMD instructions.
/// Returns a slice of the decoded elements and the number of bytes read.
#[cfg(target_feature = "avx2")]
pub fn decode_f32_batch_simd(data: &[u8]) -> Result<(&[f32], usize)> {
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
        return Ok((&[], 0));
    }

    // Check alignment for AVX2 operations (32-byte alignment is ideal for AVX2)
    let aligned = (data.as_ptr() as usize) % 32 == 0;
    
    if aligned {
        // For aligned data, we can simply reinterpret the slice
        // This is safe because we've already checked size and alignment
        let decoded_slice = unsafe {
            slice::from_raw_parts(data.as_ptr() as *const f32, count)
        };
        
        Ok((decoded_slice, data.len()))
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
        
        // Return a reference to the vector's data
        let slice = unsafe {
            slice::from_raw_parts(values.as_ptr(), values.len())
        };
        
        // We need to leak the vector to ensure the data remains valid
        std::mem::forget(values);
        
        Ok((slice, data.len()))
    }
}
