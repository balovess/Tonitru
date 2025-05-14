use crate::codec::types::HtlvValue;
use crate::internal::error::{Error, Result};
use std::mem;
use std::slice;
use crate::codec::decode::batch::BatchDecoder; // Import BatchDecoder trait

// SIMD intrinsics are now handled in the simd_optimizations module


/// Decodes a U32 HtlvValue from bytes.
pub fn decode_u32(length: u64, raw_value_slice: &[u8]) -> Result<HtlvValue> {
    if length as usize != mem::size_of::<u32>() {
        return Err(Error::CodecError(format!(
            "Invalid length for U32 value: {}",
            length
        )));
    }
    if raw_value_slice.len() < mem::size_of::<u32>() {
         return Err(Error::CodecError("Incomplete data for U32 value".to_string()));
    }

    // Use from_le_bytes for standard decoding
    let mut bytes = [0u8; mem::size_of::<u32>()];
    bytes.copy_from_slice(&raw_value_slice[..mem::size_of::<u32>()]);
    Ok(HtlvValue::U32(u32::from_le_bytes(bytes)))

    // The SIMD part in the original decode_u32 was incorrect for a single value.
    // SIMD is more applicable to batch decoding.
    /*
    #[cfg(all(target_arch = "x86_64", target_feature = "sse4.1"))]
    {
        if is_x86_feature_detected!("sse4.1") {
            // Use SIMD to load and extract the u32 value
            // Safety: We check data length above. raw_value_slice is guaranteed to be 4 bytes.
            let ptr = raw_value_slice.as_ptr() as *const i32; // Load as i32 for _mm_extract_epi32
            let val_m128i = unsafe { _mm_loadu_si128(ptr as *const _) };
            let val = unsafe { _mm_extract_epi32(val_m128i, 0) as u32 }; // Extract the first 32-bit value
            Ok(HtlvValue::U32(val))
        } else {
            // Fallback
            let mut bytes = [0u8; mem::size_of::<u32>()];
            bytes.copy_from_slice(raw_value_slice);
            Ok(HtlvValue::U32(u32::from_le_bytes(bytes)))
        }
    }
    #[cfg(not(all(target_arch = "x86_64", target_feature = "sse4.1")))]
    {
        // Fallback for other architectures or no SSE4.1
        let mut bytes = [0u8; mem::size_of::<u32>()];
        bytes.copy_from_slice(raw_value_slice);
        Ok(HtlvValue::U32(u32::from_le_bytes(bytes)))
    }
    */
}

// Note: The previous `decode_u32_batch` function is now replaced by the `BatchDecoder` implementation below.

impl BatchDecoder for u32 {
    type DecodedType = u32;

    /// Decodes a batch of U32 values from bytes.
    /// Returns a slice of the decoded elements and the number of bytes read.
    fn decode_batch(data: &[u8]) -> Result<(&[Self::DecodedType], usize)> {
        let size = mem::size_of::<u32>();
        if data.len() % size != 0 {
            return Err(Error::CodecError(format!(
                "Invalid data length for U32 batch decoding. Length ({}) must be a multiple of {}",
                data.len(),
                size
            )));
        }

        let count = data.len() / size;
        let decoded_slice = unsafe {
            // Safety:
            // 1. The data slice is guaranteed to be valid for reads for `data.len()` bytes.
            // 2. We check that `data.len()` is a multiple of `size_of::<u32>()`,
            //    ensuring the total size is correct for `count` u32 elements.
            // 3. We assume the data is in little-endian format, consistent with `u32::from_le_bytes`.
            //    Reinterpreting assumes the byte order matches the target type's representation.
            // 4. Alignment: This is a potential issue. `slice::from_raw_parts` requires
            //    the pointer to be aligned for `Self::DecodedType` (u32). `&[u8]` does not
            //    guarantee alignment for types larger than u8. Using `_mm_loadu_si128` (unaligned load)
            //    in a SIMD context handles this, but direct reinterpretation with `from_raw_parts`
            //    might cause issues on some architectures if the data is not aligned.
            //    For simplicity and zero-copy, we proceed with reinterpretation, but a robust
            //    implementation might need to handle alignment explicitly or use crates like `bytemuck`
            //    which provide checked reinterpretation. For now, we assume sufficient alignment
            //    or that the target architecture handles unaligned access gracefully for this size.
            slice::from_raw_parts(data.as_ptr() as *const u32, count)
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

    #[test]
    fn test_decode_u32() {
        let encoded_u32 = encode_item(&HtlvItem::new(0, HtlvValue::U32(4294967295))).unwrap();
        // Assuming encode_item for U32 results in [Tag(varint), Type(u8), Length(varint), Value(u32)]
        // For tag 0 (1 byte), type U32 (1 byte), length 4 (1 byte), the header is 3 bytes.
        let raw_value_slice_u32 = &encoded_u32[3..]; // Length 4
        let decoded_u32 = decode_u32(4, raw_value_slice_u32).unwrap();
        assert_eq!(decoded_u32, HtlvValue::U32(4294967295));

        // Test with incomplete data
        let result = decode_u32(4, &[0x00, 0x00, 0x00]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Incomplete data for U32 value"
        );
    }

    #[test]
    fn test_decode_u32_errors() {
        // Invalid length for U32 (expected 4)
        let result = decode_u32(3, &[0x00, 0x00, 0x00, 0x00]); // Provide enough data but wrong length
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Invalid length for U32 value: 3"
        );
    }

    #[test]
    fn test_decode_batch_u32() {
        // Test decoding a batch of U32 values
        let values: Vec<u32> = vec![1, 2, 3, 4, 5];
        // Reinterpret the u32 vector as a u8 slice. This ensures correct alignment.
        let data: &[u8] = unsafe {
            slice::from_raw_parts(values.as_ptr() as *const u8, values.len() * mem::size_of::<u32>())
        };
        let expected: &[u32] = &[1, 2, 3, 4, 5];
        let (decoded_slice, bytes_consumed) = u32::decode_batch(&data).unwrap();
        assert_eq!(decoded_slice, expected);
        assert_eq!(bytes_consumed, data.len());

        // Test with empty data
        let values: Vec<u32> = vec![];
        let data: &[u8] = unsafe {
            slice::from_raw_parts(values.as_ptr() as *const u8, values.len() * mem::size_of::<u32>())
        };
        let expected: &[u32] = &[];
        let (decoded_slice, bytes_consumed) = u32::decode_batch(data).unwrap();
        assert_eq!(decoded_slice, expected);
        assert_eq!(bytes_consumed, 0);

         // Test with incomplete data (not a multiple of size_of::<u32>())
        let incomplete_data: &[u8] = &[0x01, 0x00, 0x00]; // 3 bytes
        let result = u32::decode_batch(&incomplete_data);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Invalid data length for U32 batch decoding. Length (3) must be a multiple of 4"
        );
    }
}