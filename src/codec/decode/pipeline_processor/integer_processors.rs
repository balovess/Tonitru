// Integer type pipeline processors

use crate::internal::error::Result;
use crate::codec::types::HtlvValue;
use std::mem;

use super::{PipelineProcessor, AlignedBatch};

/// Implementation of PipelineProcessor for u8
impl PipelineProcessor for u8 {
    type DecodedType = u8;

    // Use default prefetch implementation from trait

    fn decode(aligned_batch: AlignedBatch<Self::DecodedType>) -> Result<(Vec<Self::DecodedType>, usize)> {
        // For u8, the data is already in the correct format
        let slice = aligned_batch.as_slice();
        Ok((slice.to_vec(), slice.len()))
    }

    fn dispatch(decoded_values: &[Self::DecodedType]) -> Vec<HtlvValue> {
        decoded_values.iter().map(|&v| HtlvValue::U8(v)).collect()
    }

    fn verify(decoded_values: &[Self::DecodedType], original_data: &[u8], bytes_consumed: usize) -> bool {
        // For u8, verify that we consumed the expected number of bytes
        bytes_consumed == original_data.len() && bytes_consumed == decoded_values.len()
    }
}

/// Implementation of PipelineProcessor for u16
impl PipelineProcessor for u16 {
    type DecodedType = u16;

    // Use default prefetch implementation from trait

    fn decode(aligned_batch: AlignedBatch<Self::DecodedType>) -> Result<(Vec<Self::DecodedType>, usize)> {
        // The aligned_batch already contains properly aligned data
        // We can simply convert it to a vector
        let slice = aligned_batch.as_slice();
        let bytes_consumed = slice.len() * mem::size_of::<u16>();

        Ok((slice.to_vec(), bytes_consumed))
    }

    fn dispatch(decoded_values: &[Self::DecodedType]) -> Vec<HtlvValue> {
        decoded_values.iter().map(|&v| HtlvValue::U16(v)).collect()
    }

    fn verify(decoded_values: &[Self::DecodedType], original_data: &[u8], bytes_consumed: usize) -> bool {
        // Verify that we consumed the expected number of bytes
        bytes_consumed == original_data.len() &&
        bytes_consumed == decoded_values.len() * mem::size_of::<u16>()
    }
}

/// Implementation of PipelineProcessor for u32
impl PipelineProcessor for u32 {
    type DecodedType = u32;

    // Use default prefetch implementation from trait

    fn decode(aligned_batch: AlignedBatch<Self::DecodedType>) -> Result<(Vec<Self::DecodedType>, usize)> {
        // The aligned_batch already contains properly aligned data
        // We can use SIMD if available and the data is aligned

        #[cfg(feature = "simd")]
        {
            // Use SIMD-optimized decoding if available and data is aligned
            use crate::codec::decode::simd_optimizations;
            if aligned_batch.is_aligned() && simd_optimizations::is_simd_available() {
                // For aligned data, we can use SIMD directly
                let slice = aligned_batch.as_slice();
                let bytes_consumed = slice.len() * mem::size_of::<u32>();
                return Ok((slice.to_vec(), bytes_consumed));
            }
        }

        // For non-SIMD or unaligned data, simply convert to vector
        let slice = aligned_batch.as_slice();
        let bytes_consumed = slice.len() * mem::size_of::<u32>();

        Ok((slice.to_vec(), bytes_consumed))
    }

    fn dispatch(decoded_values: &[Self::DecodedType]) -> Vec<HtlvValue> {
        decoded_values.iter().map(|&v| HtlvValue::U32(v)).collect()
    }

    fn verify(decoded_values: &[Self::DecodedType], original_data: &[u8], bytes_consumed: usize) -> bool {
        // Verify that we consumed the expected number of bytes
        bytes_consumed == original_data.len() &&
        bytes_consumed == decoded_values.len() * mem::size_of::<u32>()
    }
}

/// Implementation of PipelineProcessor for u64
impl PipelineProcessor for u64 {
    type DecodedType = u64;

    // Use default prefetch implementation from trait

    fn decode(aligned_batch: AlignedBatch<Self::DecodedType>) -> Result<(Vec<Self::DecodedType>, usize)> {
        // The aligned_batch already contains properly aligned data
        // We can simply convert it to a vector
        let slice = aligned_batch.as_slice();
        let bytes_consumed = slice.len() * mem::size_of::<u64>();

        Ok((slice.to_vec(), bytes_consumed))
    }

    fn dispatch(decoded_values: &[Self::DecodedType]) -> Vec<HtlvValue> {
        decoded_values.iter().map(|&v| HtlvValue::U64(v)).collect()
    }

    fn verify(decoded_values: &[Self::DecodedType], original_data: &[u8], bytes_consumed: usize) -> bool {
        // Verify that we consumed the expected number of bytes
        bytes_consumed == original_data.len() &&
        bytes_consumed == decoded_values.len() * mem::size_of::<u64>()
    }
}

/// Implementation of PipelineProcessor for i8
impl PipelineProcessor for i8 {
    type DecodedType = i8;

    // Use default prefetch implementation from trait

    fn decode(aligned_batch: AlignedBatch<Self::DecodedType>) -> Result<(Vec<Self::DecodedType>, usize)> {
        // The aligned_batch already contains properly aligned data
        // We can simply convert it to a vector
        let slice = aligned_batch.as_slice();
        let bytes_consumed = slice.len() * mem::size_of::<i8>();

        Ok((slice.to_vec(), bytes_consumed))
    }

    fn dispatch(decoded_values: &[Self::DecodedType]) -> Vec<HtlvValue> {
        decoded_values.iter().map(|&v| HtlvValue::I8(v)).collect()
    }

    fn verify(decoded_values: &[Self::DecodedType], original_data: &[u8], bytes_consumed: usize) -> bool {
        // For i8, verify that we consumed the expected number of bytes
        bytes_consumed == original_data.len() && bytes_consumed == decoded_values.len()
    }
}