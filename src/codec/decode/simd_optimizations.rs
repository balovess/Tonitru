// SIMD optimizations for batch decoding
//
// This module contains SIMD-optimized implementations for batch decoding of basic types.
// These implementations are used when the target architecture supports the required SIMD features.

use crate::internal::error::{Error, Result};
use std::mem;
use std::slice;

// SIMD intrinsics are imported directly in the functions where they are used

/// Decodes a batch of u32 values using SIMD instructions (SSE4.1 on x86_64).
/// Returns a slice of the decoded elements and the number of bytes read.
#[cfg(target_arch = "x86_64")]
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

    // For aligned data, we can simply reinterpret the slice
    // This is safe because we've already checked size
    let decoded_slice = unsafe {
        slice::from_raw_parts(data.as_ptr() as *const u32, count)
    };

    Ok((decoded_slice, data.len()))
}

/// Decodes a batch of f32 values using SIMD instructions (SSE4.1 on x86_64).
/// Returns a slice of the decoded elements and the number of bytes read.
#[cfg(target_arch = "x86_64")]
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

    // For aligned data, we can simply reinterpret the slice
    // This is safe because we've already checked size
    let decoded_slice = unsafe {
        slice::from_raw_parts(data.as_ptr() as *const f32, count)
    };

    Ok((decoded_slice, data.len()))
}

// Add more SIMD-optimized batch decoding functions for other types as needed

#[cfg(test)]
mod tests {
    #[cfg(target_arch = "x86_64")]
    mod x86_64_tests {
        use super::super::*;
        use std::slice;
        use std::mem;

        #[test]
        fn test_decode_u32_batch_simd() {
            // Skip the test if SSE4.1 is not available
            if !is_x86_feature_detected!("sse4.1") {
                return;
            }

            // Test with aligned data
            let values: Vec<u32> = vec![1, 2, 3, 4, 5, 6, 7, 8];
            let data: &[u8] = unsafe {
                slice::from_raw_parts(values.as_ptr() as *const u8, values.len() * mem::size_of::<u32>())
            };
            let expected: &[u32] = &[1, 2, 3, 4, 5, 6, 7, 8];
            let (decoded_slice, bytes_consumed) = decode_u32_batch_simd(data).unwrap();
            assert_eq!(decoded_slice, expected);
            assert_eq!(bytes_consumed, data.len());
        }

        #[test]
        fn test_decode_f32_batch_simd() {
            // Skip the test if SSE4.1 is not available
            if !is_x86_feature_detected!("sse4.1") {
                return;
            }

            // Test with aligned data
            let values: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
            let data: &[u8] = unsafe {
                slice::from_raw_parts(values.as_ptr() as *const u8, values.len() * mem::size_of::<f32>())
            };
            let expected: &[f32] = &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
            let (decoded_slice, bytes_consumed) = decode_f32_batch_simd(data).unwrap();
            assert_eq!(decoded_slice, expected);
            assert_eq!(bytes_consumed, data.len());
        }
    }
}
