use crate::codec::types::HtlvValue;
use crate::internal::error::{Error, Result};
use std::mem;

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
}

/// Decodes a batch of I16 values from bytes.
pub fn decode_i16_batch(raw_value_slice: &[u8], count: usize) -> Result<Vec<i16>> {
    let required_len = count * mem::size_of::<i16>();
    if raw_value_slice.len() < required_len {
        return Err(Error::CodecError(format!(
            "Incomplete data for I16 batch decoding. Expected at least {} bytes, got {}",
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
                // Use SIMD to load and extract i16 values
                // Safety: We check data length above. raw_value_slice has enough data.
                let ptr = raw_value_slice[current_offset..].as_ptr() as *const i16;
                let val_m128i = unsafe { _mm_loadu_si128(ptr as *const _) };

                // Extract up to 8 i16 values from the 128-bit register
                result.push(unsafe { std::arch::x86_64::_mm_extract_epi16(val_m128i, 0) });
                if count > result.len() { result.push(unsafe { std::arch::x86_64::_mm_extract_epi16(val_m128i, 1) }); }
                if count > result.len() { result.push(unsafe { std::arch::x86_64::_mm_extract_epi16(val_m128i, 2) }); }
                if count > result.len() { result.push(unsafe { std::arch::x86_64::_mm_extract_epi16(val_m128i, 3) }); }
                if count > result.len() { result.push(unsafe { std::arch::x86_64::_mm_extract_epi16(val_m128i, 4) }); }
                if count > result.len() { result.push(unsafe { std::arch::x86_64::_mm_extract_epi16(val_m128i, 5) }); }
                if count > result.len() { result.push(unsafe { std::arch::x86_64::_mm_extract_epi16(val_m128i, 6) }); }
                if count > result.len() { result.push(unsafe { std::arch::x86_64::_mm_extract_epi16(val_m128i, 7) }); }

                current_offset += 16; // Advance by 16 bytes (size of __m128i)
            }
            // Truncate if we extracted more than 'count' due to SIMD block size
            result.truncate(count);
        } else {
            // Fallback
            for i in 0..count {
                let start = i * mem::size_of::<i16>();
                let end = start + mem::size_of::<i16>();
                let mut bytes = [0u8; mem::size_of::<i16>()];
                bytes.copy_from_slice(&raw_value_slice[start..end]);
                result.push(i16::from_le_bytes(bytes));
            }
        }
    }
    #[cfg(not(all(target_arch = "x86_64", target_feature = "sse4.1")))]
    {
        // Fallback for other architectures or no SSE4.1
        for i in 0..count {
            let start = i * mem::size_of::<i16>();
            let end = start + mem::size_of::<i16>();
            let mut bytes = [0u8; mem::size_of::<i16>()];
            bytes.copy_from_slice(&raw_value_slice[start..end]);
            result.push(i16::from_le_bytes(bytes));
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
    fn test_decode_i16() {
        let encoded_i16 = encode_item(&HtlvItem::new(0, HtlvValue::I16(-32768))).unwrap();
        let raw_value_slice_i16 = &encoded_i16[encoded_i16.len().checked_sub(2).unwrap()..]; // Length 2
        let decoded_i16 = decode_i16(2, raw_value_slice_i16).unwrap();
        assert_eq!(decoded_i16, HtlvValue::I16(-32768));
    }

    #[test]
    fn test_decode_i16_errors() {
        // Invalid length for I16 (expected 2)
        let result = decode_i16(1, &[0x00]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Invalid length for I16 value: 1"
        );
    }

    #[test]
    fn test_decode_i16_batch() {
        // Test decoding a batch of I16 values
        let data: Vec<u8> = vec![
            0xFF, 0xFF, // -1
            0xFE, 0xFF, // -2
            0x01, 0x00, // 1
            0x02, 0x00, // 2
            0x00, 0x00, // 0
        ];
        let expected = vec![-1, -2, 1, 2, 0];
        let decoded = decode_i16_batch(&data, 5).unwrap();
        assert_eq!(decoded, expected);

        // Test with incomplete data
        let incomplete_data: Vec<u8> = vec![
            0xFF, 0xFF, // -1
            0xFE, // Incomplete -2
        ];
        let result = decode_i16_batch(&incomplete_data, 2);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Incomplete data for I16 batch decoding. Expected at least 4 bytes, got 3"
        );

        // Test with empty data
        let result = decode_i16_batch(&[], 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![] as Vec<i16>);

         // Test with empty data and count > 0
        let result = decode_i16_batch(&[], 1);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Incomplete data for I16 batch decoding. Expected at least 2 bytes, got 0"
        );
    }
}