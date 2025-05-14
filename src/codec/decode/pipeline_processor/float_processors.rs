// Float type pipeline processors

use crate::internal::error::Result;
use crate::codec::types::HtlvValue;
use std::mem;

use super::{PipelineProcessor, AlignedBatch};

/// Implementation of PipelineProcessor for f32
impl PipelineProcessor for f32 {
    type DecodedType = f32;

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
                let bytes_consumed = slice.len() * mem::size_of::<f32>();
                return Ok((slice.to_vec(), bytes_consumed));
            }
        }

        // For non-SIMD or unaligned data, simply convert to vector
        let slice = aligned_batch.as_slice();
        let bytes_consumed = slice.len() * mem::size_of::<f32>();

        Ok((slice.to_vec(), bytes_consumed))
    }

    fn dispatch(decoded_values: &[Self::DecodedType]) -> Vec<HtlvValue> {
        decoded_values.iter().map(|&v| HtlvValue::F32(v)).collect()
    }

    fn verify(decoded_values: &[Self::DecodedType], original_data: &[u8], bytes_consumed: usize) -> bool {
        // Verify that we consumed the expected number of bytes
        bytes_consumed == original_data.len() &&
        bytes_consumed == decoded_values.len() * mem::size_of::<f32>()
    }
}

/// Implementation of PipelineProcessor for f64
impl PipelineProcessor for f64 {
    type DecodedType = f64;

    // Use default prefetch implementation from trait

    fn decode(aligned_batch: AlignedBatch<Self::DecodedType>) -> Result<(Vec<Self::DecodedType>, usize)> {
        // The aligned_batch already contains properly aligned data
        // We can simply convert it to a vector
        let slice = aligned_batch.as_slice();
        let bytes_consumed = slice.len() * mem::size_of::<f64>();

        Ok((slice.to_vec(), bytes_consumed))
    }

    fn dispatch(decoded_values: &[Self::DecodedType]) -> Vec<HtlvValue> {
        decoded_values.iter().map(|&v| HtlvValue::F64(v)).collect()
    }

    fn verify(decoded_values: &[Self::DecodedType], original_data: &[u8], bytes_consumed: usize) -> bool {
        // Verify that we consumed the expected number of bytes
        bytes_consumed == original_data.len() &&
        bytes_consumed == decoded_values.len() * mem::size_of::<f64>()
    }
}
