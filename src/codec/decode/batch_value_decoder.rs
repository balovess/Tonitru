// Decoder for batch HTLV values

use crate::internal::error::{Error, Result};
use crate::codec::types::{HtlvValueType, HtlvValue, HtlvItem};
use crate::codec::decode::batch::BatchDecoder; // Import BatchDecoder trait
use bytes::Bytes; // Import Bytes for batch decoding alignment
// BytesMut import removed as it's not used
use std::mem; // Import std::mem

/// Decodes a batch of HTLV values based on the element type, total length, and raw data.
/// This function encapsulates the decoding logic for batch decodable basic types.
pub fn decode_batch_value(
    element_type: HtlvValueType,
    length: u64,
    raw_value_slice: &[u8],
) -> Result<HtlvValue> {
    // Stage 3.1: Prefetch (Conceptual - handled by CPU cache)
    // In Rust, explicit prefetching is often not necessary or beneficial
    // due to sophisticated CPU cache mechanisms. The slice access below
    // will likely trigger hardware prefetching.

    // Stage 3.2: Decode (Utilize BatchDecoder) & Stage 3.4: Verify (partially handled by decode_batch)
    let decoded_elements: Vec<HtlvValue> = match element_type {
        HtlvValueType::U8 => {
            let (slice, bytes_consumed) = u8::decode_batch(raw_value_slice)?;
            if bytes_consumed != length as usize {
                 return Err(Error::CodecError(format!("U8 batch decoding consumed incorrect number of bytes. Expected {}, got {}", length, bytes_consumed)));
             }
            // Stage 3.3: Dispatch: Convert slice to Vec<HtlvValue>
            slice.iter().map(|&v| HtlvValue::U8(v)).collect()
        }
        HtlvValueType::U16 => {
            let _element_size = mem::size_of::<u16>(); // Unused but kept for clarity
            let align = mem::align_of::<u16>();
            let slice_to_decode = if raw_value_slice.as_ptr().align_offset(align) != 0 {
                // Data is not aligned, copy to an aligned buffer
                let mut buffer = vec![0u8; length as usize];
                buffer.copy_from_slice(raw_value_slice);
                Bytes::from(buffer) // Use Bytes to manage the buffer
            } else {
                Bytes::copy_from_slice(raw_value_slice) // Use Bytes to manage the slice
            };
            let (slice, bytes_consumed) = u16::decode_batch(&slice_to_decode)?;
             if bytes_consumed != length as usize {
                 return Err(Error::CodecError(format!("U16 batch decoding consumed incorrect number of bytes. Expected {}, got {}", length, bytes_consumed)));
             }
            slice.iter().map(|&v| HtlvValue::U16(v)).collect() // Corrected to U16
        }
        HtlvValueType::U32 => {
            let _element_size = mem::size_of::<u32>(); // Unused but kept for clarity
            let align = mem::align_of::<u32>();
            let slice_to_decode = if raw_value_slice.as_ptr().align_offset(align) != 0 {
                // Data is not aligned, copy to an aligned buffer
                let mut buffer = vec![0u8; length as usize];
                buffer.copy_from_slice(raw_value_slice);
                Bytes::from(buffer) // Use Bytes to manage the buffer
            } else {
                Bytes::copy_from_slice(raw_value_slice) // Use Bytes to manage the slice
            };
            let (slice, bytes_consumed) = u32::decode_batch(&slice_to_decode)?;
             if bytes_consumed != length as usize {
                 return Err(Error::CodecError(format!("U32 batch decoding consumed incorrect number of bytes. Expected {}, got {}", length, bytes_consumed)));
             }
            slice.iter().map(|&v| HtlvValue::U32(v)).collect()
        }
        HtlvValueType::U64 => {
            let _element_size = mem::size_of::<u64>(); // Unused but kept for clarity
            let align = mem::align_of::<u64>();
            let slice_to_decode = if raw_value_slice.as_ptr().align_offset(align) != 0 {
                // Data is not aligned, copy to an aligned buffer
                let mut buffer = vec![0u8; length as usize];
                buffer.copy_from_slice(raw_value_slice);
                Bytes::from(buffer) // Use Bytes to manage the buffer
            } else {
                Bytes::copy_from_slice(raw_value_slice) // Use Bytes to manage the slice
            };
            let (slice, bytes_consumed) = u64::decode_batch(&slice_to_decode)?;
             if bytes_consumed != length as usize {
                 return Err(Error::CodecError(format!("U64 batch decoding consumed incorrect number of bytes. Expected {}, got {}", length, bytes_consumed)));
             }
            slice.iter().map(|&v| HtlvValue::U64(v)).collect()
        }
        HtlvValueType::I8 => {
            let (slice, bytes_consumed) = i8::decode_batch(raw_value_slice)?;
             if bytes_consumed != length as usize {
                 return Err(Error::CodecError(format!("I8 batch decoding consumed incorrect number of bytes. Expected {}, got {}", length, bytes_consumed)));
             }
            slice.iter().map(|&v| HtlvValue::I8(v)).collect()
        }
        HtlvValueType::I16 => {
            let _element_size = mem::size_of::<i16>(); // Unused but kept for clarity
            let align = mem::align_of::<i16>();
            let slice_to_decode = if raw_value_slice.as_ptr().align_offset(align) != 0 {
                // Data is not aligned, copy to an aligned buffer
                let mut buffer = vec![0u8; length as usize];
                buffer.copy_from_slice(raw_value_slice);
                Bytes::from(buffer) // Use Bytes to manage the buffer
            } else {
                Bytes::copy_from_slice(raw_value_slice) // Use Bytes to manage the slice
            };
            let (slice, bytes_consumed) = i16::decode_batch(&slice_to_decode)?;
             if bytes_consumed != length as usize {
                 return Err(Error::CodecError(format!("I16 batch decoding consumed incorrect number of bytes. Expected {}, got {}", length, bytes_consumed)));
             }
            slice.iter().map(|&v| HtlvValue::I16(v)).collect()
        }
        HtlvValueType::I32 => {
            let _element_size = mem::size_of::<i32>(); // Unused but kept for clarity
            let align = mem::align_of::<i32>();
            let slice_to_decode = if raw_value_slice.as_ptr().align_offset(align) != 0 {
                // Data is not aligned, copy to an aligned buffer
                let mut buffer = vec![0u8; length as usize];
                buffer.copy_from_slice(raw_value_slice);
                Bytes::from(buffer) // Use Bytes to manage the buffer
            } else {
                Bytes::copy_from_slice(raw_value_slice) // Use Bytes to manage the slice
            };
            let (slice, bytes_consumed) = i32::decode_batch(&slice_to_decode)?;
             if bytes_consumed != length as usize {
                 return Err(Error::CodecError(format!("I32 batch decoding consumed incorrect number of bytes. Expected {}, got {}", length, bytes_consumed)));
             }
            slice.iter().map(|&v| HtlvValue::I32(v)).collect()
        }
        HtlvValueType::I64 => {
            let _element_size = mem::size_of::<i64>(); // Unused but kept for clarity
            let align = mem::align_of::<i64>();
            let slice_to_decode = if raw_value_slice.as_ptr().align_offset(align) != 0 {
                // Data is not aligned, copy to an aligned buffer
                let mut buffer = vec![0u8; length as usize];
                buffer.copy_from_slice(raw_value_slice);
                Bytes::from(buffer) // Use Bytes to manage the buffer
            } else {
                Bytes::copy_from_slice(raw_value_slice) // Use Bytes to manage the slice
            };
            let (slice, bytes_consumed) = i64::decode_batch(&slice_to_decode)?;
             if bytes_consumed != length as usize {
                 return Err(Error::CodecError(format!("I64 batch decoding consumed incorrect number of bytes. Expected {}, got {}", length, bytes_consumed)));
             }
            slice.iter().map(|&v| HtlvValue::I64(v)).collect()
        }
        HtlvValueType::F32 => {
            let _element_size = mem::size_of::<f32>(); // Unused but kept for clarity
            let align = mem::align_of::<f32>();
            let slice_to_decode = if raw_value_slice.as_ptr().align_offset(align) != 0 {
                // Data is not aligned, copy to an aligned buffer
                let mut buffer = vec![0u8; length as usize];
                buffer.copy_from_slice(raw_value_slice);
                Bytes::from(buffer) // Use Bytes to manage the buffer
            } else {
                Bytes::copy_from_slice(raw_value_slice) // Use Bytes to manage the slice
            };
            let (slice, bytes_consumed) = f32::decode_batch(&slice_to_decode)?;
             if bytes_consumed != length as usize {
                 return Err(Error::CodecError(format!("F32 batch decoding consumed incorrect number of bytes. Expected {}, got {}", length, bytes_consumed)));
             }
            slice.iter().map(|&v| HtlvValue::F32(v)).collect()
        }
        HtlvValueType::F64 => {
            let _element_size = mem::size_of::<f64>(); // Unused but kept for clarity
            let align = mem::align_of::<f64>();
            let slice_to_decode = if raw_value_slice.as_ptr().align_offset(align) != 0 {
                // Data is not aligned, copy to an aligned buffer
                let mut buffer = vec![0u8; length as usize];
                buffer.copy_from_slice(raw_value_slice);
                Bytes::from(buffer) // Use Bytes to manage the buffer
            } else {
                Bytes::copy_from_slice(raw_value_slice) // Use Bytes to manage the slice
            };
            let (slice, bytes_consumed) = f64::decode_batch(&slice_to_decode)?;
             if bytes_consumed != length as usize {
                 return Err(Error::CodecError(format!("F64 batch decoding consumed incorrect number of bytes. Expected {}, got {}", length, bytes_consumed)));
             }
            slice.iter().map(|&v| HtlvValue::F64(v)).collect()
        }
        // Other types should not reach here
        _ => {
             return Err(Error::CodecError(format!("Unexpected type ({:?}) in decode_batch_value", element_type)));
        }
    };

    // Stage 3.3: Dispatch: Convert Vec<HtlvValue> to Vec<HtlvItem> and wrap in HtlvValue::Array
    let items: Vec<HtlvItem> = decoded_elements.into_iter().map(|v| HtlvItem::new(0, v)).collect(); // Use tag 0 for array elements for now
    Ok(HtlvValue::Array(items))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut; // Import BytesMut for tests

    #[test]
    fn test_decode_batch_value_u8() {
        let items_to_encode = vec![
            HtlvItem::new(0, HtlvValue::U8(1)),
            HtlvItem::new(0, HtlvValue::U8(2)),
            HtlvItem::new(0, HtlvValue::U8(3)),
        ];
        // Manually encode the batch value (just the raw bytes of the U8 values)
        let mut raw_batch_data = BytesMut::new();
        for item in &items_to_encode {
            if let HtlvValue::U8(v) = item.value {
                raw_batch_data.extend_from_slice(&[v]);
            }
        }
        let raw_batch_slice = raw_batch_data.freeze();

        let decoded_value = decode_batch_value(HtlvValueType::U8, raw_batch_slice.len() as u64, &raw_batch_slice).unwrap();

        if let HtlvValue::Array(decoded_items) = decoded_value {
            assert_eq!(decoded_items.len(), items_to_encode.len());
            for (decoded_item, expected_item) in decoded_items.iter().zip(items_to_encode.iter()) {
                assert_eq!(decoded_item.value, expected_item.value);
            }
        } else {
            panic!("Decoded value is not an Array");
        }
    }

    #[test]
    fn test_decode_batch_value_u32() {
        let items_to_encode = vec![
            HtlvItem::new(0, HtlvValue::U32(100)),
            HtlvItem::new(0, HtlvValue::U32(200)),
            HtlvItem::new(0, HtlvValue::U32(300)),
        ];
         // Manually encode the batch value (just the raw bytes of the U32 values)
        let mut raw_batch_data = BytesMut::new();
        for item in &items_to_encode {
            if let HtlvValue::U32(v) = item.value {
                raw_batch_data.extend_from_slice(&v.to_le_bytes()); // Use little-endian for consistency
            }
        }
        let raw_batch_slice = raw_batch_data.freeze();

        let decoded_value = decode_batch_value(HtlvValueType::U32, raw_batch_slice.len() as u64, &raw_batch_slice).unwrap();

        if let HtlvValue::Array(decoded_items) = decoded_value {
            assert_eq!(decoded_items.len(), items_to_encode.len());
            for (decoded_item, expected_item) in decoded_items.iter().zip(items_to_encode.iter()) {
                assert_eq!(decoded_item.value, expected_item.value);
            }
        } else {
            panic!("Decoded value is not an Array");
        }
    }

    // Add more tests for other batch decodable types (I8, I16, I32, I64, F32, F64)
}