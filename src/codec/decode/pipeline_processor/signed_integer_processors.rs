// Signed integer type pipeline processors

use crate::internal::error::Result;
use crate::codec::types::HtlvValue;
use std::mem;

use super::{PipelineProcessor, AlignedBatch};

/// Implementation of PipelineProcessor for i16
impl PipelineProcessor for i16 {
    type DecodedType = i16;

    // Use default prefetch implementation from trait

    fn decode(aligned_batch: AlignedBatch<Self::DecodedType>) -> Result<(Vec<Self::DecodedType>, usize)> {
        // The aligned_batch already contains properly aligned data
        // We can simply convert it to a vector
        let slice = aligned_batch.as_slice();
        let bytes_consumed = slice.len() * mem::size_of::<i16>();

        Ok((slice.to_vec(), bytes_consumed))
    }

    fn dispatch(decoded_values: &[Self::DecodedType]) -> Vec<HtlvValue> {
        decoded_values.iter().map(|&v| HtlvValue::I16(v)).collect()
    }

    fn verify(decoded_values: &[Self::DecodedType], original_data: &[u8], bytes_consumed: usize) -> bool {
        // Verify that we consumed the expected number of bytes
        bytes_consumed == original_data.len() &&
        bytes_consumed == decoded_values.len() * mem::size_of::<i16>()
    }
}

/// Implementation of PipelineProcessor for i32
impl PipelineProcessor for i32 {
    type DecodedType = i32;

    // Use default prefetch implementation from trait

    fn decode(aligned_batch: AlignedBatch<Self::DecodedType>) -> Result<(Vec<Self::DecodedType>, usize)> {
        // The aligned_batch already contains properly aligned data
        // We can simply convert it to a vector
        let slice = aligned_batch.as_slice();
        let bytes_consumed = slice.len() * mem::size_of::<i32>();

        Ok((slice.to_vec(), bytes_consumed))
    }

    fn dispatch(decoded_values: &[Self::DecodedType]) -> Vec<HtlvValue> {
        decoded_values.iter().map(|&v| HtlvValue::I32(v)).collect()
    }

    fn verify(decoded_values: &[Self::DecodedType], original_data: &[u8], bytes_consumed: usize) -> bool {
        // Verify that we consumed the expected number of bytes
        bytes_consumed == original_data.len() &&
        bytes_consumed == decoded_values.len() * mem::size_of::<i32>()
    }
}

/// Implementation of PipelineProcessor for i64
impl PipelineProcessor for i64 {
    type DecodedType = i64;

    // Use default prefetch implementation from trait

    fn decode(aligned_batch: AlignedBatch<Self::DecodedType>) -> Result<(Vec<Self::DecodedType>, usize)> {
        // The aligned_batch already contains properly aligned data
        // We can simply convert it to a vector
        let slice = aligned_batch.as_slice();
        let bytes_consumed = slice.len() * mem::size_of::<i64>();

        Ok((slice.to_vec(), bytes_consumed))
    }

    fn dispatch(decoded_values: &[Self::DecodedType]) -> Vec<HtlvValue> {
        decoded_values.iter().map(|&v| HtlvValue::I64(v)).collect()
    }

    fn verify(decoded_values: &[Self::DecodedType], original_data: &[u8], bytes_consumed: usize) -> bool {
        // Verify that we consumed the expected number of bytes
        bytes_consumed == original_data.len() &&
        bytes_consumed == decoded_values.len() * mem::size_of::<i64>()
    }
}