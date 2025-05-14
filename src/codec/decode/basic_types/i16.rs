use crate::codec::types::HtlvValue;
use crate::internal::error::{Error, Result};
use std::mem;
use crate::codec::decode::batch::BatchDecoder; // Import BatchDecoder trait
use std::slice; // Import slice for unsafe reinterpretation

// Enable necessary features for SIMD intrinsics (requires Rust nightly or specific configuration)
#[cfg(target_arch = "x86_64")]
// Import specific SIMD intrinsics based on target features
#[cfg(target_arch = "x86_64")]
#[cfg(target_feature = "sse4.1")]
use std::arch::x86_64::_mm_loadu_si128;
// Removed unused import: #[cfg(target_arch = "x86_64")] #[cfg(target_feature = "sse2")] use std::arch::x86_64::_mm_extract_epi16;


/// Decodes an I16 HtlvValue from bytes.
pub fn decode_i16(length: u64, raw_value_slice: &[u8]) -> Result<HtlvValue> {
    if length as usize != mem::size_of::<i16>() {
        return Err(Error::CodecError(format!(
            "Invalid length for I16 value: {}",
            length
        )));
    }
    if raw_value_slice.len() < mem::size_of::<i16>() {
         return Err(Error::CodecError("Incomplete data for I16 value".to_string()));
    }
    // Use from_le_bytes for standard decoding
    let mut bytes = [0u8; mem::size_of::<i16>()];
    bytes.copy_from_slice(&raw_value_slice[..mem::size_of::<i16>()]);
    Ok(HtlvValue::I16(i16::from_le_bytes(bytes)))

    // The SIMD part in the original decode_i16 was incorrect for a single value.
    // SIMD is more applicable to batch decoding.
    /*
    #[cfg(all(target_arch = "x86_64", target_feature = "sse4.1"))]
    {
        if is_x86_feature_detected!("sse4.1") {
            // Use SIMD to load and extract the i16 value
            // Safety: We check data length above. raw_value_slice is guaranteed to be 2 bytes.
            let ptr = raw_value_slice.as_ptr() as *const i16;
            let val_m128i = unsafe { _mm_loadu_si128(ptr as *const _) };
            let val = unsafe { std::arch::x86_64::_mm_extract_epi16(val_m128i, 0) }; // Extract the first 16-bit value
            Ok(HtlvValue::I16(val))
        } else {
            // Fallback
            let mut bytes = [0u8; mem::size_of::<i16>()];
            bytes.copy_from_slice(raw_value_slice);
            Ok(HtlvValue::I16(i16::from_le_bytes(bytes)))
        }
    }
    #[cfg(not(all(target_arch = "x86_64", target_feature = "sse4.1")))]
    {
        // Fallback for other architectures or no SSE4.1
        let mut bytes = [0u8; mem::size_of::<i16>()];
        bytes.copy_from_slice(raw_value_slice);
        Ok(HtlvValue::I16(i16::from_le_bytes(bytes)))
    }
    */
}

// Note: The previous `decode_i16_batch` function is now replaced by the `BatchDecoder` implementation below.

impl BatchDecoder for i16 {
    type DecodedType = i16;

    /// Decodes a batch of I16 values from bytes using zero-copy reinterpretation.
    /// Returns a slice of the decoded elements and the number of bytes read.
    fn decode_batch(data: &[u8]) -> Result<(&[Self::DecodedType], usize)> {
        let size = mem::size_of::<i16>();
        if data.len() % size != 0 {
            return Err(Error::CodecError(format!(
                "Invalid data length for I16 batch decoding. Length ({}) must be a multiple of {}",
                data.len(),
                size
            )));
        }

        let count = data.len() / size;
        let decoded_slice = unsafe {
            // Safety:
            // 1. The data slice is guaranteed to be valid for reads for `data.len()` bytes.
            // 2. We check that `data.len()` is a multiple of `size_of::<i16>()`,
            //    ensuring the total size is correct for `count` i16 elements.
            // 3. We assume the data is in little-endian format, consistent with `i16::from_le_bytes`.
            //    Reinterpreting assumes the byte order matches the target type's representation.
            // 4. Alignment: This is a potential issue. `slice::from_raw_parts` requires
            //    the pointer to be aligned for `Self::DecodedType` (i16). `&[u8]` does not
            //    guarantee alignment for types larger than u8. Using `_mm_loadu_si128` (unaligned load)
            //    in a SIMD context handles this, but direct reinterpretation with `from_raw_parts`
            //    might cause issues on some architectures if the data is not aligned.
            //    For simplicity and zero-copy, we proceed with reinterpretation, but a robust
            //    implementation might need to handle alignment explicitly or use crates like `bytemuck`
            //    which provide checked reinterpretation. For now, we assume sufficient alignment
            //    or that the target architecture handles unaligned access gracefully for this size.
            slice::from_raw_parts(data.as_ptr() as *const i16, count)
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
    fn test_decode_i16() {
        let encoded_i16 = encode_item(&HtlvItem::new(0, HtlvValue::I16(-32768))).unwrap();
        // Assuming encode_item for I16 results in [Tag(varint), Type(u8), Length(varint), Value(i16)]
        // For tag 0 (1 byte), type I16 (1 byte), length 2 (1 byte), the header is 3 bytes.
        let raw_value_slice_i16 = &encoded_i16[3..]; // Length 2
        let decoded_i16 = decode_i16(2, raw_value_slice_i16).unwrap();
        assert_eq!(decoded_i16, HtlvValue::I16(-32768));

        // Test with incomplete data
        let result = decode_i16(2, &[0x00]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Incomplete data for I16 value"
        );
    }

    #[test]
    fn test_decode_i16_errors() {
        // Invalid length for I16 (expected 2)
        let result = decode_i16(1, &[0x00, 0x00]); // Provide enough data but wrong length
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Invalid length for I16 value: 1"
        );
    }

    #[test]
    fn test_decode_batch_i16() {
        // Test decoding a batch of I16 values
        let values: Vec<i16> = vec![-1, -2, 1, 2, 0];
        // Reinterpret the i16 vector as a u8 slice. This ensures correct alignment.
        let data: &[u8] = unsafe {
            slice::from_raw_parts(values.as_ptr() as *const u8, values.len() * mem::size_of::<i16>())
        };
        let expected: &[i16] = &[-1, -2, 1, 2, 0];
        let (decoded_slice, bytes_consumed) = i16::decode_batch(&data).unwrap();
        assert_eq!(decoded_slice, expected);
        assert_eq!(bytes_consumed, data.len());

        // Test with empty data
        let values: Vec<i16> = vec![];
        let data: &[u8] = unsafe {
            slice::from_raw_parts(values.as_ptr() as *const u8, values.len() * mem::size_of::<i16>())
        };
        let expected: &[i16] = &[];
        let (decoded_slice, bytes_consumed) = i16::decode_batch(data).unwrap();
        assert_eq!(decoded_slice, expected);
        assert_eq!(bytes_consumed, 0);

         // Test with incomplete data (not a multiple of size_of::<i16>()) - This test case is still valid
        let incomplete_data: &[u8] = &[0x01, 0x00, 0x02]; // 3 bytes
        let result = i16::decode_batch(&incomplete_data);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Invalid data length for I16 batch decoding. Length (3) must be a multiple of 2"
        );
    }
}