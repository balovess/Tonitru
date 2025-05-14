// Signed integer type pipeline processors

use crate::internal::error::{Error, Result};
use crate::codec::types::HtlvValue;
use bytes::{Bytes, BytesMut};
use std::mem;
use bytemuck;

use super::PipelineProcessor;

/// Implementation of PipelineProcessor for i16
impl PipelineProcessor for i16 {
    type DecodedType = i16;

    fn prefetch(data: &[u8]) -> Result<(Bytes, usize)> {
        let size = mem::size_of::<i16>();
        if data.len() % size != 0 {
            return Err(Error::CodecError(format!(
                "Invalid data length for I16 batch decoding. Length ({}) must be a multiple of {}",
                data.len(),
                size
            )));
        }

        let align = mem::align_of::<i16>();
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
        if (data.as_ptr() as usize) % mem::align_of::<i16>() == 0 {
            let decoded_slice = bytemuck::cast_slice(data);
            return Ok((decoded_slice.to_vec(), data.len()));
        }

        // Fallback to scalar processing
        let count = data.len() / mem::size_of::<i16>();
        if count == 0 {
            return Ok((Vec::new(), 0));
        }

        // For unaligned data, we need to copy it to a new buffer
        // This is inefficient but safe
        let mut values = Vec::with_capacity(count);
        for i in 0..count {
            let offset = i * mem::size_of::<i16>();
            let value = i16::from_le_bytes([data[offset], data[offset + 1]]);
            values.push(value);
        }

        Ok((values, data.len()))
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

    fn prefetch(data: &[u8]) -> Result<(Bytes, usize)> {
        let size = mem::size_of::<i32>();
        if data.len() % size != 0 {
            return Err(Error::CodecError(format!(
                "Invalid data length for I32 batch decoding. Length ({}) must be a multiple of {}",
                data.len(),
                size
            )));
        }

        let align = mem::align_of::<i32>();
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
        if (data.as_ptr() as usize) % mem::align_of::<i32>() == 0 {
            let decoded_slice = bytemuck::cast_slice(data);
            return Ok((decoded_slice.to_vec(), data.len()));
        }

        // Fallback to scalar processing
        let count = data.len() / mem::size_of::<i32>();
        if count == 0 {
            return Ok((Vec::new(), 0));
        }

        // For unaligned data, we need to copy it to a new buffer
        // This is inefficient but safe
        let mut values = Vec::with_capacity(count);
        for i in 0..count {
            let offset = i * mem::size_of::<i32>();
            let value = i32::from_le_bytes([
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

    fn prefetch(data: &[u8]) -> Result<(Bytes, usize)> {
        let size = mem::size_of::<i64>();
        if data.len() % size != 0 {
            return Err(Error::CodecError(format!(
                "Invalid data length for I64 batch decoding. Length ({}) must be a multiple of {}",
                data.len(),
                size
            )));
        }

        let align = mem::align_of::<i64>();
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
        if (data.as_ptr() as usize) % mem::align_of::<i64>() == 0 {
            let decoded_slice = bytemuck::cast_slice(data);
            return Ok((decoded_slice.to_vec(), data.len()));
        }

        // Fallback to scalar processing
        let count = data.len() / mem::size_of::<i64>();
        if count == 0 {
            return Ok((Vec::new(), 0));
        }

        // For unaligned data, we need to copy it to a new buffer
        // This is inefficient but safe
        let mut values = Vec::with_capacity(count);
        for i in 0..count {
            let offset = i * mem::size_of::<i64>();
            let value = i64::from_le_bytes([
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
        decoded_values.iter().map(|&v| HtlvValue::I64(v)).collect()
    }

    fn verify(decoded_values: &[Self::DecodedType], original_data: &[u8], bytes_consumed: usize) -> bool {
        // Verify that we consumed the expected number of bytes
        bytes_consumed == original_data.len() &&
        bytes_consumed == decoded_values.len() * mem::size_of::<i64>()
    }
}