use crate::codec::types::HtlvValue;
use crate::internal::error::{Error, Result};
use crate::codec::decode::batch::BatchDecoder; // Import BatchDecoder trait
use std::slice; // Import slice for unsafe reinterpretation
use std::mem; // Import mem for size_of and align_of

/// Decodes an F32 HtlvValue from bytes.
pub fn decode_f32(length: u64, raw_value_slice: &[u8]) -> Result<HtlvValue> {
    if length as usize != mem::size_of::<f32>() {
        return Err(Error::CodecError(format!(
            "Invalid length for F32 value: {}",
            length
        )));
    }
    if raw_value_slice.len() < mem::size_of::<f32>() {
         return Err(Error::CodecError("Incomplete data for F32 value".to_string()));
    }
    // Use from_le_bytes for standard decoding
    let mut bytes = [0u8; mem::size_of::<f32>()];
    bytes.copy_from_slice(&raw_value_slice[..mem::size_of::<f32>()]);
    Ok(HtlvValue::F32(f32::from_le_bytes(bytes)))
}

/// Decodes an F64 HtlvValue from bytes.
pub fn decode_f64(length: u64, raw_value_slice: &[u8]) -> Result<HtlvValue> {
    if length as usize != mem::size_of::<f64>() {
        return Err(Error::CodecError(format!(
            "Invalid length for F64 value: {}",
            length
        )));
    }
    if raw_value_slice.len() < mem::size_of::<f64>() {
         return Err(Error::CodecError("Incomplete data for F64 value".to_string()));
    }
    // Use from_le_bytes for standard decoding
    let mut bytes = [0u8; mem::size_of::<f64>()];
    bytes.copy_from_slice(&raw_value_slice[..mem::size_of::<f64>()]);
    Ok(HtlvValue::F64(f64::from_le_bytes(bytes)))
}

impl BatchDecoder for f32 {
    type DecodedType = f32;

    /// Decodes a batch of F32 values from bytes.
    /// Returns a slice of the decoded elements and the number of bytes read.
    fn decode_batch(data: &[u8]) -> Result<(&[Self::DecodedType], usize)> {
        let size = mem::size_of::<f32>();
        let align = mem::align_of::<f32>();

        if data.len() % size != 0 {
            return Err(Error::CodecError(format!(
                "Invalid data length for F32 batch decoding. Length ({}) must be a multiple of {}",
                data.len(),
                size
            )));
        }

        // Check alignment
        if data.as_ptr().align_offset(align) != 0 {
            return Err(Error::CodecError(format!(
                "Input data is not aligned for F32 batch decoding. Required alignment: {}",
                align
            )));
        }

        let count = data.len() / size;
        let decoded_slice = unsafe {
            // Safety:
            // 1. The data slice is guaranteed to be valid for reads for `data.len()` bytes.
            // 2. We check that `data.len()` is a multiple of `size_of::<f32>()`,
            //    ensuring the total size is correct for `count` f32 elements.
            // 3. We assume the data is in little-endian format, consistent with `f32::from_le_bytes`.
            //    Reinterpreting assumes the byte order matches the target type's representation.
            // 4. Alignment: We explicitly check that the data pointer is aligned for f32.
            slice::from_raw_parts(data.as_ptr() as *const f32, count)
        };

        Ok((decoded_slice, data.len()))
    }
}

impl BatchDecoder for f64 {
    type DecodedType = f64;

    /// Decodes a batch of F64 values from bytes using zero-copy reinterpretation.
    /// Returns a slice of the decoded elements and the number of bytes read.
    /// Requires the input data slice to be aligned for f64.
    fn decode_batch(data: &[u8]) -> Result<(&[Self::DecodedType], usize)> {
        let size = mem::size_of::<f64>();
        let align = mem::align_of::<f64>();

        if data.len() % size != 0 {
            return Err(Error::CodecError(format!(
                "Invalid data length for F64 batch decoding. Length ({}) must be a multiple of {}",
                data.len(),
                size
            )));
        }

        // Check alignment
        if data.as_ptr().align_offset(align) != 0 {
             return Err(Error::CodecError(format!(
                "Input data is not aligned for F64 batch decoding. Required alignment: {}",
                align
            )));
        }

        let count = data.len() / size;
        let decoded_slice = unsafe {
            // Safety:
            // 1. The data slice is guaranteed to be valid for reads for `data.len()` bytes.
            // 2. We check that `data.len()` is a multiple of `size_of::<f64>()`,
            //    ensuring the total size is correct for `count` f64 elements.
            // 3. We assume the data is in little-endian format, consistent with `f64::from_le_bytes`.
            //    Reinterpreting assumes the byte order matches the target type's representation.
            // 4. Alignment: We explicitly check that the data pointer is aligned for f64.
            slice::from_raw_parts(data.as_ptr() as *const f64, count)
        };

        Ok((decoded_slice, data.len()))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::encode::encode_item;
    use crate::codec::types::HtlvItem;
    use std::slice; // Import slice for unsafe reinterpretation in test
    use std::mem; // Import mem for size_of in test

    #[test]
    fn test_decode_floats() {
        // Test F32
        let encoded_f32 = encode_item(&HtlvItem::new(0, HtlvValue::F32(3.14f32))).unwrap();
        // Assuming encode_item for F32 results in [Tag(varint), Type(u8), Length(varint), Value(f32)]
        // For tag 0 (1 byte), type F32 (1 byte), length 4 (1 byte), the header is 3 bytes.
        let raw_value_slice_f32 = &encoded_f32[3..]; // Length 4
        let decoded_f32 = decode_f32(4, raw_value_slice_f32).unwrap();
        assert_eq!(decoded_f32, HtlvValue::F32(3.14f32));

        // Test with incomplete data for F32
        let result = decode_f32(4, &[0x00, 0x00, 0x00]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Incomplete data for F32 value"
        );


        // Test F64
        let encoded_f64 = encode_item(&HtlvItem::new(0, HtlvValue::F64(3.14))).unwrap();
        // Assuming encode_item for F64 results in [Tag(varint), Type(u8), Length(varint), Value(f64)]
        // For tag 0 (1 byte), type F64 (1 byte), length 8 (1 byte), the header is 3 bytes.
        let raw_value_slice_f64 = &encoded_f64[3..]; // Length 8
        let decoded_f64 = decode_f64(8, raw_value_slice_f64).unwrap();
        assert_eq!(decoded_f64, HtlvValue::F64(3.14));

        // Test with incomplete data for F64
        let result = decode_f64(8, &[0x00; 7]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Incomplete data for F64 value"
        );
    }

    #[test]
    fn test_decode_float_errors() {
        // Invalid length for F32 (expected 4)
        let result = decode_f32(3, &[0x00, 0x00, 0x00, 0x00]); // Provide enough data but wrong length
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Invalid length for F32 value: 3"
        );

        // Invalid length for F64 (expected 8)
        let result = decode_f64(7, &[0x00; 8]); // Provide enough data but wrong length
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Invalid length for F64 value: 7"
        );
    }

    #[test]
    fn test_decode_batch_f32() {
        // Test decoding a batch of F32 values with aligned data
        let original_values: Vec<f32> = vec![
            3.14f32,
            1.0f32,
            0.4f32,
            0.0f32,
            -1.0f32,
        ];
        // Get a byte slice from the f32 vector. This slice is guaranteed to be aligned for f32.
        let data: &[u8] = unsafe {
            slice::from_raw_parts(original_values.as_ptr() as *const u8, original_values.len() * mem::size_of::<f32>())
        };

        let expected: &[f32] = &original_values;
        let (decoded_slice, bytes_consumed) = f32::decode_batch(data).unwrap();
        assert_eq!(decoded_slice, expected);
        assert_eq!(bytes_consumed, data.len());

        // Test with empty data
        let original_values_empty: Vec<f32> = vec![];
        let data_empty: &[u8] = unsafe {
            slice::from_raw_parts(original_values_empty.as_ptr() as *const u8, original_values_empty.len() * mem::size_of::<f32>())
        };
        let expected_empty: &[f32] = &[];
        let (decoded_slice_empty, bytes_consumed_empty) = f32::decode_batch(data_empty).unwrap();
        assert_eq!(decoded_slice_empty, expected_empty);
        assert_eq!(bytes_consumed_empty, 0);

         // Test with incomplete data (not a multiple of size_of::<f32>())
        let incomplete_data: &[u8] = &[0x01, 0x00, 0x00]; // 3 bytes
        let result_incomplete_batch = f32::decode_batch(&incomplete_data);
        assert!(result_incomplete_batch.is_err());
        assert_eq!(
            result_incomplete_batch.unwrap_err().to_string(),
            "Codec Error: Invalid data length for F32 batch decoding. Length (3) must be a multiple of 4"
        );

        // Test with unaligned data (should return an error)
        // Create unaligned data by slicing a Vec<u8> at an offset
        let unaligned_vec: Vec<u8> = vec![0u8, 0xc3, 0xf5, 0x48, 0x40, 0x00, 0x00, 0x80, 0x3f]; // Add a leading byte, include two f32s
        let unaligned_data: &[u8] = &unaligned_vec[1..]; // Slice starts at offset 1
        let result_unaligned = f32::decode_batch(&unaligned_data);
        assert!(result_unaligned.is_err());
        assert_eq!(
            result_unaligned.unwrap_err().to_string(),
            format!("Codec Error: Input data is not aligned for F32 batch decoding. Required alignment: {}", mem::align_of::<f32>())
        );
    }

    #[test]
    fn test_decode_batch_f64() {
        // Test decoding a batch of F64 values with aligned data
        let original_values: Vec<f64> = vec![
            3.14,
            1.0,
            0.4,
            0.0,
            -1.0,
        ];
        // Get a byte slice from the f64 vector. This slice is guaranteed to be aligned for f64.
        let data: &[u8] = unsafe {
            slice::from_raw_parts(original_values.as_ptr() as *const u8, original_values.len() * mem::size_of::<f64>())
        };

        let expected: &[f64] = &original_values;
        let (decoded_slice, bytes_consumed) = f64::decode_batch(&data).unwrap();
        assert_eq!(decoded_slice, expected);
        assert_eq!(bytes_consumed, data.len());

        // Test with empty data
        let original_values_empty: Vec<f64> = vec![];
        let data_empty: &[u8] = unsafe {
            slice::from_raw_parts(original_values_empty.as_ptr() as *const u8, original_values_empty.len() * mem::size_of::<f64>())
        };
        let expected_empty: &[f64] = &[];
        let (decoded_slice_empty, bytes_consumed_empty) = f64::decode_batch(data_empty).unwrap();
        assert_eq!(decoded_slice_empty, expected_empty);
        assert_eq!(bytes_consumed_empty, 0);

         // Test with incomplete data (not a multiple of size_of::<f64>())
        let incomplete_data: &[u8] = &[0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]; // 7 bytes
        let result_incomplete_batch = f64::decode_batch(&incomplete_data);
        assert!(result_incomplete_batch.is_err());
        assert_eq!(
            result_incomplete_batch.unwrap_err().to_string(),
            "Codec Error: Invalid data length for F64 batch decoding. Length (7) must be a multiple of 8"
        );

        // Test with unaligned data (should return an error)
        // Create unaligned data by slicing a Vec<u8> at an offset
        let unaligned_vec: Vec<u8> = vec![0u8, 0x1f, 0x85, 0xeb, 0x51, 0xb8, 0x1e, 0x09, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0x3f]; // Add a leading byte, include two f64s
        let unaligned_data: &[u8] = &unaligned_vec[1..]; // Slice starts at offset 1
        let result_unaligned = f64::decode_batch(&unaligned_data);
        assert!(result_unaligned.is_err());
        assert_eq!(
            result_unaligned.unwrap_err().to_string(),
            format!("Codec Error: Input data is not aligned for F64 batch decoding. Required alignment: {}", mem::align_of::<f64>())
        );
    }
}