use crate::codec::types::HtlvValue;
use crate::internal::error::{Error, Result};
use std::mem;

// Enable necessary features for SIMD intrinsics (requires Rust nightly or specific configuration)
#[cfg(target_arch = "x86_64")]
// Import specific SIMD intrinsics based on target features
#[cfg(target_arch = "x86_64")]
#[cfg(target_feature = "sse4.1")]
use std::arch::x86_64::{_mm_loadu_si128, _mm_extract_epi32};


/// Decodes a U32 HtlvValue from bytes.
pub fn decode_u32(length: u64, raw_value_slice: &[u8]) -> Result<HtlvValue> {
    if length as usize != mem::size_of::<u32>() {
        return Err(Error::CodecError(format!(
            "Invalid length for U32 value: {}",
            length
        )));
    }
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
}

/// Decodes a batch of U32 values from bytes.
pub fn decode_u32_batch(raw_value_slice: &[u8], count: usize) -> Result<Vec<u32>> {
    let required_len = count * mem::size_of::<u32>();
    if raw_value_slice.len() < required_len {
        return Err(Error::CodecError(format!(
            "Incomplete data for U32 batch decoding. Expected at least {} bytes, got {}",
            required_len,
            raw_value_slice.len()
        )));
    }

    let mut result = Vec::with_capacity(count);

    #[cfg(all(target_arch = "x86_64", target_feature = "sse4.1"))]
    {
        if is_x86_feature_detected!("sse4.1") {
            let mut current_offset = 0;
            while current_offset < required_len {
                // Use SIMD to load and extract u32 values
                // Safety: We check data length above. raw_value_slice has enough data.
                let ptr = raw_value_slice[current_offset..].as_ptr() as *const i32;
                let val_m128i = unsafe { _mm_loadu_si128(ptr as *const _) };

                // Extract up to 4 u32 values from the 128-bit register
                result.push(unsafe { _mm_extract_epi32(val_m128i, 0) as u32 });
                if count > result.len() {
                    result.push(unsafe { _mm_extract_epi32(val_m128i, 1) as u32 });
                }
                if count > result.len() {
                    result.push(unsafe { _mm_extract_epi32(val_m128i, 2) as u32 });
                }
                if count > result.len() {
                    result.push(unsafe { _mm_extract_epi32(val_m128i, 3) as u32 });
                }

                current_offset += 16; // Advance by 16 bytes (size of __m128i)
            }
            // Truncate if we extracted more than 'count' due to SIMD block size
            result.truncate(count);
        } else {
            // Fallback
            for i in 0..count {
                let start = i * mem::size_of::<u32>();
                let end = start + mem::size_of::<u32>();
                let mut bytes = [0u8; mem::size_of::<u32>()];
                bytes.copy_from_slice(&raw_value_slice[start..end]);
                result.push(u32::from_le_bytes(bytes));
            }
        }
    }
    #[cfg(not(all(target_arch = "x86_64", target_feature = "sse4.1")))]
    {
        // Fallback for other architectures or no SSE4.1
        for i in 0..count {
            let start = i * mem::size_of::<u32>();
            let end = start + mem::size_of::<u32>();
            let mut bytes = [0u8; mem::size_of::<u32>()];
            bytes.copy_from_slice(&raw_value_slice[start..end]);
            result.push(u32::from_le_bytes(bytes));
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::encode::encode_item;
    use crate::codec::types::HtlvItem;

    #[test]
    fn test_decode_u32() {
        let encoded_u32 = encode_item(&HtlvItem::new(0, HtlvValue::U32(4294967295))).unwrap();
        let raw_value_slice_u32 = &encoded_u32[encoded_u32.len().checked_sub(4).unwrap()..]; // Length 4
        let decoded_u32 = decode_u32(4, raw_value_slice_u32).unwrap();
        assert_eq!(decoded_u32, HtlvValue::U32(4294967295));
    }

    #[test]
    fn test_decode_u32_errors() {
        // Invalid length for U32 (expected 4)
        let result = decode_u32(3, &[0x00, 0x00, 0x00]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Invalid length for U32 value: 3"
        );
    }

    #[test]
    fn test_decode_u32_batch() {
        // Test decoding a batch of U32 values
        let data: Vec<u8> = vec![
            0x01, 0x00, 0x00, 0x00, // 1
            0x02, 0x00, 0x00, 0x00, // 2
            0x03, 0x00, 0x00, 0x00, // 3
            0x04, 0x00, 0x00, 0x00, // 4
            0x05, 0x00, 0x00, 0x00, // 5
        ];
        let expected = vec![1, 2, 3, 4, 5];
        let decoded = decode_u32_batch(&data, 5).unwrap();
        assert_eq!(decoded, expected);

        // Test with incomplete data
        let incomplete_data: Vec<u8> = vec![
            0x01, 0x00, 0x00, 0x00, // 1
            0x02, 0x00, 0x00, // Incomplete 2
        ];
        let result = decode_u32_batch(&incomplete_data, 2);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Incomplete data for U32 batch decoding. Expected at least 8 bytes, got 7"
        );

        // Test with empty data
        let result = decode_u32_batch(&[], 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![] as Vec<u32>);

         // Test with empty data and count > 0
        let result = decode_u32_batch(&[], 1);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Incomplete data for U32 batch decoding. Expected at least 4 bytes, got 0"
        );
    }
}