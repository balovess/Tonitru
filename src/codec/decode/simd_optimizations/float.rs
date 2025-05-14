// SIMD optimizations for floating-point types
//
// This module contains SIMD-optimized implementations for batch decoding of floating-point types.

use crate::internal::error::{Error, Result};

/// Decodes a batch of f32 values using the best available SIMD instructions.
/// Returns a BatchResult containing the decoded elements and the number of bytes read.
pub fn decode_f32_batch_simd(data: &[u8]) -> Result<(super::BatchResult<f32>, usize)> {
    use super::BatchResult;

    #[cfg(all(target_arch = "x86_64", feature = "simd"))]
    {
        if std::is_x86_feature_detected!("sse4.1") {
            // Use the re-exported function from the main module
            return super::x86_64::sse41::decode_f32_batch_simd(data);
        }
    }

    // Fallback to non-SIMD implementation
    let size = std::mem::size_of::<f32>();
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

    // Check alignment
    let aligned = (data.as_ptr() as usize) % std::mem::align_of::<f32>() == 0;

    if aligned {
        // For aligned data, we can simply reinterpret the slice
        let decoded_slice = unsafe {
            std::slice::from_raw_parts(data.as_ptr() as *const f32, count)
        };

        Ok((BatchResult::borrowed(decoded_slice), data.len()))
    } else {
        // For unaligned data, use scalar processing
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

/// Decodes a batch of f64 values using the best available SIMD instructions.
/// Returns a BatchResult containing the decoded elements and the number of bytes read.
pub fn decode_f64_batch_simd(data: &[u8]) -> Result<(super::BatchResult<f64>, usize)> {
    use super::BatchResult;

    // Check if data length is valid
    let size = std::mem::size_of::<f64>();
    if data.len() % size != 0 {
        return Err(Error::CodecError(format!(
            "Invalid data length for F64 batch decoding. Length ({}) must be a multiple of {}",
            data.len(),
            size
        )));
    }

    let count = data.len() / size;
    if count == 0 {
        return Ok((BatchResult::borrowed(&[]), 0));
    }

    // Check alignment
    let aligned = (data.as_ptr() as usize) % std::mem::align_of::<f64>() == 0;

    if aligned {
        // For aligned data, we can simply reinterpret the slice
        let decoded_slice = unsafe {
            std::slice::from_raw_parts(data.as_ptr() as *const f64, count)
        };

        Ok((BatchResult::borrowed(decoded_slice), data.len()))
    } else {
        // For unaligned data, use scalar processing
        let mut values = Vec::with_capacity(count);
        for i in 0..count {
            let offset = i * size;
            let value = f64::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
                data[offset + 4],
                data[offset + 5],
                data[offset + 6],
                data[offset + 7],
            ]);
            values.push(value);
        }

        // Return an owned BatchResult, which will properly manage the memory
        Ok((BatchResult::owned(values), data.len()))
    }
}
