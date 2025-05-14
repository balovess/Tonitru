// SIMD optimizations for integer types
//
// This module contains SIMD-optimized implementations for batch decoding of integer types.

use crate::internal::error::{Error, Result};

/// Decodes a batch of u32 values using SIMD instructions.
pub fn decode_u32_batch_simd(data: &[u8]) -> Result<(super::BatchResult<u32>, usize)> {
    use super::BatchResult;

    // Check if data length is valid
    let size = std::mem::size_of::<u32>();
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

    #[cfg(all(target_arch = "x86_64", feature = "simd"))]
    {
        if std::is_x86_feature_detected!("sse4.1") {
            // Use the re-exported function from the main module
            return super::x86_64::sse41::decode_u32_batch_simd(data);
        }
    }

    // Check alignment
    let aligned = (data.as_ptr() as usize) % std::mem::align_of::<u32>() == 0;

    if aligned {
        // For aligned data, we can simply reinterpret the slice
        let decoded_slice = unsafe {
            std::slice::from_raw_parts(data.as_ptr() as *const u32, count)
        };

        Ok((BatchResult::borrowed(decoded_slice), data.len()))
    } else {
        // For unaligned data, use scalar processing
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

/// Decodes a batch of u8 values using SIMD instructions.
/// For u8, this is mostly a pass-through since no conversion is needed,
/// but we include it for API consistency.
pub fn decode_u8_batch_simd(data: &[u8]) -> Result<(super::BatchResult<u8>, usize)> {
    use super::BatchResult;

    #[cfg(all(target_arch = "x86_64", feature = "simd"))]
    {
        if std::is_x86_feature_detected!("sse4.1") {
            return super::x86_64::sse41::decode_u8_batch_simd(data);
        }
    }

    // Fallback to non-SIMD implementation
    Ok((BatchResult::borrowed(data), data.len()))
}

/// Decodes a batch of i8 values using SIMD instructions.
pub fn decode_i8_batch_simd(data: &[u8]) -> Result<(super::BatchResult<i8>, usize)> {
    use super::BatchResult;

    #[cfg(all(target_arch = "x86_64", feature = "simd"))]
    {
        if std::is_x86_feature_detected!("sse4.1") {
            return super::x86_64::sse41::decode_i8_batch_simd(data);
        }
    }

    // For i8, we can simply reinterpret the slice
    // This is safe because i8 and u8 have the same memory layout
    let decoded_slice = unsafe {
        std::slice::from_raw_parts(data.as_ptr() as *const i8, data.len())
    };

    Ok((BatchResult::borrowed(decoded_slice), data.len()))
}

/// Decodes a batch of u16 values using SIMD instructions.
pub fn decode_u16_batch_simd(data: &[u8]) -> Result<(super::BatchResult<u16>, usize)> {
    use super::BatchResult;

    // Check if data length is valid
    let size = std::mem::size_of::<u16>();
    if data.len() % size != 0 {
        return Err(Error::CodecError(format!(
            "Invalid data length for U16 batch decoding. Length ({}) must be a multiple of {}",
            data.len(),
            size
        )));
    }

    let count = data.len() / size;
    if count == 0 {
        return Ok((BatchResult::borrowed(&[]), 0));
    }

    // Check alignment
    let aligned = (data.as_ptr() as usize) % std::mem::align_of::<u16>() == 0;

    if aligned {
        // For aligned data, we can simply reinterpret the slice
        let decoded_slice = unsafe {
            std::slice::from_raw_parts(data.as_ptr() as *const u16, count)
        };

        Ok((BatchResult::borrowed(decoded_slice), data.len()))
    } else {
        // For unaligned data, use scalar processing
        let mut values = Vec::with_capacity(count);
        for i in 0..count {
            let offset = i * size;
            let value = u16::from_le_bytes([
                data[offset],
                data[offset + 1],
            ]);
            values.push(value);
        }

        // Return an owned BatchResult, which will properly manage the memory
        Ok((BatchResult::owned(values), data.len()))
    }
}

/// Decodes a batch of i16 values using SIMD instructions.
pub fn decode_i16_batch_simd(data: &[u8]) -> Result<(super::BatchResult<i16>, usize)> {
    use super::BatchResult;

    // Implementation similar to u16, but for i16
    // Check if data length is valid
    let size = std::mem::size_of::<i16>();
    if data.len() % size != 0 {
        return Err(Error::CodecError(format!(
            "Invalid data length for I16 batch decoding. Length ({}) must be a multiple of {}",
            data.len(),
            size
        )));
    }

    let count = data.len() / size;
    if count == 0 {
        return Ok((BatchResult::borrowed(&[]), 0));
    }

    // Check alignment
    let aligned = (data.as_ptr() as usize) % std::mem::align_of::<i16>() == 0;

    if aligned {
        // For aligned data, we can simply reinterpret the slice
        let decoded_slice = unsafe {
            std::slice::from_raw_parts(data.as_ptr() as *const i16, count)
        };

        Ok((BatchResult::borrowed(decoded_slice), data.len()))
    } else {
        // For unaligned data, use scalar processing
        let mut values = Vec::with_capacity(count);
        for i in 0..count {
            let offset = i * size;
            let value = i16::from_le_bytes([
                data[offset],
                data[offset + 1],
            ]);
            values.push(value);
        }

        // Return an owned BatchResult, which will properly manage the memory
        Ok((BatchResult::owned(values), data.len()))
    }
}
