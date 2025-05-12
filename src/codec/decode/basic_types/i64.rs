use crate::codec::types::HtlvValue;
use crate::internal::error::{Error, Result};
use std::mem;

// Enable necessary features for SIMD intrinsics (requires Rust nightly or specific configuration)
#[cfg(target_arch = "x86_64")]
// Import specific SIMD intrinsics based on target features
#[cfg(target_arch = "x86_64")]
#[cfg(target_feature = "sse4.1")]
use std::arch::x86_64::{_mm_loadu_si128, _mm_extract_epi64};


/// Decodes an I64 HtlvValue from bytes.
pub fn decode_i64(length: u64, raw_value_slice: &[u8]) -> Result<HtlvValue> {
    if length as usize != mem::size_of::<i64>() {
        return Err(Error::CodecError(format!(
            "Invalid length for I64 value: {}",
            length
        )));
    }
    let mut bytes = [0u8; mem::size_of::<i64>()];
    bytes.copy_from_slice(raw_value_slice);
    Ok(HtlvValue::I64(i64::from_le_bytes(bytes)))
}

/// Decodes a batch of I64 values from bytes.
pub fn decode_i64_batch(raw_value_slice: &[u8], count: usize) -> Result<Vec<i64>> {
    let required_len = count * mem::size_of::<i64>();
    if raw_value_slice.len() < required_len {
        return Err(Error::CodecError(format!(
            "Incomplete data for I64 batch decoding. Expected at least {} bytes, got {}",
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
                // Use SIMD to load and extract i64 values
                // Safety: We check data length above. raw_value_slice has enough data.
                let ptr = raw_value_slice[current_offset..].as_ptr() as *const i64;
                let val_m128i = unsafe { _mm_loadu_si128(ptr as *const _) };

                // Extract up to 2 i64 values from the 128-bit register
                result.push(unsafe { _mm_extract_epi64(val_m128i, 0) as i64 });
                if count > result.len() {
                    result.push(unsafe { _mm_extract_epi64(val_m128i, 1) as i64 });
                }

                current_offset += 16; // Advance by 16 bytes (size of __m128i)
            }
            // Truncate if we extracted more than 'count' due to SIMD block size
            result.truncate(count);
        } else {
            // Fallback
            for i in 0..count {
                let start = i * mem::size_of::<i64>();
                let end = start + mem::size_of::<i64>();
                let mut bytes = [0u8; mem::size_of::<i64>()];
                bytes.copy_from_slice(&raw_value_slice[start..end]);
                result.push(i64::from_le_bytes(bytes));
            }
        }
    }
    #[cfg(not(all(target_arch = "x86_64", target_feature = "sse4.1")))]
    {
        // Fallback for other architectures or no SSE4.1
        for i in 0..count {
            let start = i * mem::size_of::<i64>();
            let end = start + mem::size_of::<i64>();
            let mut bytes = [0u8; mem::size_of::<i64>()];
            bytes.copy_from_slice(&raw_value_slice[start..end]);
            result.push(i64::from_le_bytes(bytes));
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
    fn test_decode_i64() {
        let encoded_i64 = encode_item(&HtlvItem::new(0, HtlvValue::I64(-987654321))).unwrap();
        let raw_value_slice_i64 = &encoded_i64[encoded_i64.len().checked_sub(8).unwrap()..]; // Length 8
        let decoded_i64 = decode_i64(8, raw_value_slice_i64).unwrap();
        assert_eq!(decoded_i64, HtlvValue::I64(-987654321));
    }

    #[test]
    fn test_decode_i64_errors() {
        // Invalid length for I64 (expected 8)
        let result = decode_i64(7, &[0x00; 7]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Invalid length for I64 value: 7"
        );
    }

    #[test]
    fn test_decode_i64_batch() {
        // Test decoding a batch of I64 values
        let data: Vec<u8> = vec![
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // -1
            0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // -2
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 1
        ];
        let expected = vec![-1, -2, 1];
        let decoded = decode_i64_batch(&data, 3).unwrap();
        assert_eq!(decoded, expected);

        // Test with incomplete data
        let incomplete_data: Vec<u8> = vec![
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // -1
            0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // Incomplete -2
        ];
        let result = decode_i64_batch(&incomplete_data, 2);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Incomplete data for I64 batch decoding. Expected at least 16 bytes, got 15"
        );

        // Test with empty data
        let result = decode_i64_batch(&[], 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![] as Vec<i64>);

         // Test with empty data and count > 0
        let result = decode_i64_batch(&[], 1);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Incomplete data for I64 batch decoding. Expected at least 8 bytes, got 0"
        );
    }
}