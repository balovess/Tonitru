// Float type pipeline processors

use crate::internal::error::{Error, Result};
use crate::codec::types::HtlvValue;
use bytes::{Bytes, BytesMut};
use std::mem;
use bytemuck;

use super::PipelineProcessor;

/// Implementation of PipelineProcessor for f32
impl PipelineProcessor for f32 {
    type DecodedType = f32;

    fn prefetch(data: &[u8]) -> Result<(Bytes, usize)> {
        let size = mem::size_of::<f32>();
        if data.len() % size != 0 {
            return Err(Error::CodecError(format!(
                "Invalid data length for F32 batch decoding. Length ({}) must be a multiple of {}",
                data.len(),
                size
            )));
        }

        let align = mem::align_of::<f32>();
        let prepared_data = if data.as_ptr().align_offset(align) != 0 {
            // Data is not aligned, copy to an aligned buffer
            let mut buffer = BytesMut::with_capacity(data.len());
            buffer.extend_from_slice(data);
            buffer.freeze()
        } else {
            // Data is already aligned
            Bytes::copy_from_slice(data)
        };

        Ok((prepared_data, data.len()))
    }

    fn decode(prepared_data: &Bytes) -> Result<(Vec<Self::DecodedType>, usize)> {
        let data = prepared_data.as_ref();

        #[cfg(feature = "simd")]
        {
            // Use SIMD-optimized decoding if available
            use crate::codec::decode::simd_optimizations;
            if simd_optimizations::is_simd_available() {
                let (batch_result, bytes_consumed) = simd_optimizations::decode_f32_batch_simd(data)?;
                return Ok((batch_result.to_vec(), bytes_consumed));
            }
        }

        // Use bytemuck for safe casting if data is properly aligned
        if (data.as_ptr() as usize) % mem::align_of::<f32>() == 0 {
            let decoded_slice = bytemuck::cast_slice(data);
            return Ok((decoded_slice.to_vec(), data.len()));
        }

        // Fallback to scalar processing
        let count = data.len() / mem::size_of::<f32>();
        if count == 0 {
            return Ok((Vec::new(), 0));
        }

        // For unaligned data, we need to copy it to a new buffer
        // This is inefficient but safe
        let mut values = Vec::with_capacity(count);
        for i in 0..count {
            let offset = i * mem::size_of::<f32>();
            let value = f32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3]
            ]);
            values.push(value);
        }

        Ok((values, data.len()))
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

    fn prefetch(data: &[u8]) -> Result<(Bytes, usize)> {
        let size = mem::size_of::<f64>();
        if data.len() % size != 0 {
            return Err(Error::CodecError(format!(
                "Invalid data length for F64 batch decoding. Length ({}) must be a multiple of {}",
                data.len(),
                size
            )));
        }

        let align = mem::align_of::<f64>();
        let prepared_data = if data.as_ptr().align_offset(align) != 0 {
            // Data is not aligned, copy to an aligned buffer
            let mut buffer = BytesMut::with_capacity(data.len());
            buffer.extend_from_slice(data);
            buffer.freeze()
        } else {
            // Data is already aligned
            Bytes::copy_from_slice(data)
        };

        Ok((prepared_data, data.len()))
    }

    fn decode(prepared_data: &Bytes) -> Result<(Vec<Self::DecodedType>, usize)> {
        let data = prepared_data.as_ref();

        // Use bytemuck for safe casting if data is properly aligned
        if (data.as_ptr() as usize) % mem::align_of::<f64>() == 0 {
            let decoded_slice = bytemuck::cast_slice(data);
            return Ok((decoded_slice.to_vec(), data.len()));
        }

        // Fallback to scalar processing
        let count = data.len() / mem::size_of::<f64>();
        if count == 0 {
            return Ok((Vec::new(), 0));
        }

        // For unaligned data, we need to copy it to a new buffer
        // This is inefficient but safe
        let mut values = Vec::with_capacity(count);
        for i in 0..count {
            let offset = i * mem::size_of::<f64>();
            let value = f64::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
                data[offset + 4],
                data[offset + 5],
                data[offset + 6],
                data[offset + 7]
            ]);
            values.push(value);
        }

        Ok((values, data.len()))
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
