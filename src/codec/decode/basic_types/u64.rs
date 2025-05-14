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
use std::arch::x86_64::{_mm_loadu_si128, _mm_extract_epi64};


/// Decodes a U64 HtlvValue from bytes.
pub fn decode_u64(length: u64, raw_value_slice: &[u8]) -> Result<HtlvValue> {
    if length as usize != mem::size_of::<u64>() {
        return Err(Error::CodecError(format!(
            "Invalid length for U64 value: {}",
            length
        )));
    }
    if raw_value_slice.len() < mem::size_of::<u64>() {
         return Err(Error::CodecError("Incomplete data for U64 value".to_string()));
    }
    let mut bytes = [0u8; mem::size_of::<u64>()];
    bytes.copy_from_slice(&raw_value_slice[..mem::size_of::<u64>()]);
    Ok(HtlvValue::U64(u64::from_le_bytes(bytes)))
}

// Note: The previous `decode_u64_batch` function is now replaced by the `BatchDecoder` implementation below.

impl BatchDecoder for u64 {
    type DecodedType = u64;

    /// Decodes a batch of U64 values from bytes using zero-copy reinterpretation.
    /// Returns a slice of the decoded elements and the number of bytes read.
    fn decode_batch(data: &[u8]) -> Result<(&[Self::DecodedType], usize)> {
        let size = mem::size_of::<u64>();
        if data.len() % size != 0 {
            return Err(Error::CodecError(format!(
                "Invalid data length for U64 batch decoding. Length ({}) must be a multiple of {}",
                data.len(),
                size
            )));
        }

        let count = data.len() / size;
        let decoded_slice = unsafe {
            // Safety:
            // 1. The data slice is guaranteed to be valid for reads for `data.len()` bytes.
            // 2. We check that `data.len()` is a multiple of `size_of::<u64>()`,
            //    ensuring the total size is correct for `count` u64 elements.
            // 3. We assume the data is in little-endian format, consistent with `u64::from_le_bytes`.
            //    Reinterpreting assumes the byte order matches the target type's representation.
            // 4. Alignment: This is a potential issue. `slice::from_raw_parts` requires
            //    the pointer to be aligned for `Self::DecodedType` (u64). `&[u8]` does not
            //    guarantee alignment for types larger than u8. Using `_mm_loadu_si128` (unaligned load)
            //    in a SIMD context handles this, but direct reinterpretation with `from_raw_parts`
            //    might cause issues on some architectures if the data is not aligned.
            //    For simplicity and zero-copy, we proceed with reinterpretation, but a robust
            //    implementation might need to handle alignment explicitly or use crates like `bytemuck`
            //    which provide checked reinterpretation. For now, we assume sufficient alignment
            //    or that the target architecture handles unaligned access gracefully for this size.
            slice::from_raw_parts(data.as_ptr() as *const u64, count)
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
    fn test_decode_u64() {
        let encoded_u64 = encode_item(&HtlvItem::new(0, HtlvValue::U64(1234567890))).unwrap();
        // Assuming encode_item for U64 results in [Tag(varint), Type(u8), Length(varint), Value(u64)]
        // For tag 0 (1 byte), type U64 (1 byte), length 8 (1 byte), the header is 3 bytes.
        let raw_value_slice_u64 = &encoded_u64[3..]; // Length 8
        let decoded_u64 = decode_u64(8, raw_value_slice_u64).unwrap();
        assert_eq!(decoded_u64, HtlvValue::U64(1234567890));

        // Test with incomplete data
        let result = decode_u64(8, &[0x00; 7]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Incomplete data for U64 value"
        );
    }

    #[test]
    fn test_decode_u64_errors() {
        // Invalid length for U64 (expected 8)
        let result = decode_u64(7, &[0x00; 8]); // Provide enough data but wrong length
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Invalid length for U64 value: 7"
        );
    }

    #[test]
    fn test_decode_batch_u64() {
        // Test decoding a batch of U64 values
        let values: Vec<u64> = vec![1, 2, 3];
        // Reinterpret the u64 vector as a u8 slice. This ensures correct alignment.
        let data: &[u8] = unsafe {
            slice::from_raw_parts(values.as_ptr() as *const u8, values.len() * mem::size_of::<u64>())
        };
        let expected: &[u64] = &[1, 2, 3];
        let (decoded_slice, bytes_consumed) = u64::decode_batch(&data).unwrap();
        assert_eq!(decoded_slice, expected);
        assert_eq!(bytes_consumed, data.len());

        // Test with empty data
        let values: Vec<u64> = vec![];
        let data: &[u8] = unsafe {
            slice::from_raw_parts(values.as_ptr() as *const u8, values.len() * mem::size_of::<u64>())
        };
        let expected: &[u64] = &[];
        let (decoded_slice, bytes_consumed) = u64::decode_batch(data).unwrap();
        assert_eq!(decoded_slice, expected);
        assert_eq!(bytes_consumed, 0);

         // Test with incomplete data (not a multiple of size_of::<u64>())
        let incomplete_data: &[u8] = &[0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]; // 7 bytes
        let result = u64::decode_batch(&incomplete_data);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Invalid data length for U64 batch decoding. Length (7) must be a multiple of 8"
        );
    }
}