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


/// Decodes a U16 HtlvValue from bytes.
pub fn decode_u16(length: u64, raw_value_slice: &[u8]) -> Result<HtlvValue> {
    if length as usize != mem::size_of::<u16>() {
        return Err(Error::CodecError(format!(
            "Invalid length for U16 value: {}",
            length
        )));
    }
    #[cfg(all(target_arch = "x86_64", target_feature = "sse4.1"))]
    {
        if is_x86_feature_detected!("sse4.1") {
            // Use SIMD to load and extract the u16 value
            // Safety: We check data length above. raw_value_slice is guaranteed to be 2 bytes.
            let ptr = raw_value_slice.as_ptr() as *const i16; // Load as i16 for _mm_extract_epi16
            let val_m128i = unsafe { _mm_loadu_si128(ptr as *const _) };
            let val = unsafe { std::arch::x86_64::_mm_extract_epi16(val_m128i, 0) as u16 }; // Extract the first 16-bit value
            Ok(HtlvValue::U16(val))
        } else {
            // Fallback
            let mut bytes = [0u8; mem::size_of::<u16>()];
            bytes.copy_from_slice(raw_value_slice);
            Ok(HtlvValue::U16(u16::from_le_bytes(bytes)))
        }
    }
    #[cfg(not(all(target_arch = "x86_64", target_feature = "sse4.1")))]
    {
        // Fallback for other architectures or no SSE4.1
        let mut bytes = [0u8; mem::size_of::<u16>()];
        bytes.copy_from_slice(raw_value_slice);
        Ok(HtlvValue::U16(u16::from_le_bytes(bytes)))
    }
}

/// Decodes a batch of U16 values from bytes.
pub fn decode_u16_batch(raw_value_slice: &[u8], count: usize) -> Result<Vec<u16>> {
    let required_len = count * mem::size_of::<u16>();
    if raw_value_slice.len() < required_len {
        return Err(Error::CodecError(format!(
            "Incomplete data for U16 batch decoding. Expected at least {} bytes, got {}",
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
                // Use SIMD to load and extract u16 values
                // Safety: We check data length above. raw_value_slice has enough data.
                let ptr = raw_value_slice[current_offset..].as_ptr() as *const i16;
                let val_m128i = unsafe { _mm_loadu_si128(ptr as *const _) };

                // Extract up to 8 u16 values from the 128-bit register
                result.push(unsafe { std::arch::x86_64::_mm_extract_epi16(val_m128i, 0) as u16 });
                if count > result.len() { result.push(unsafe { std::arch::x86_64::_mm_extract_epi16(val_m128i, 1) as u16 }); }
                if count > result.len() { result.push(unsafe { std::arch::x86_64::_mm_extract_epi16(val_m128i, 2) as u16 }); }
                if count > result.len() { result.push(unsafe { std::arch::x86_64::_mm_extract_epi16(val_m128i, 3) as u16 }); }
                if count > result.len() { result.push(unsafe { std::arch::x86_64::_mm_extract_epi16(val_m128i, 4) as u16 }); }
                if count > result.len() { result.push(unsafe { std::arch::x86_64::_mm_extract_epi16(val_m128i, 5) as u16 }); }
                if count > result.len() { result.push(unsafe { std::arch::x86_64::_mm_extract_epi16(val_m128i, 6) as u16 }); }
                if count > result.len() { result.push(unsafe { std::arch::x86_64::_mm_extract_epi16(val_m128i, 7) as u16 }); }


                current_offset += 16; // Advance by 16 bytes (size of __m128i)
            }
            // Truncate if we extracted more than 'count' due to SIMD block size
            result.truncate(count);
        } else {
            // Fallback
            for i in 0..count {
                let start = i * mem::size_of::<u16>();
                let end = start + mem::size_of::<u16>();
                let mut bytes = [0u8; mem::size_of::<u16>()];
                bytes.copy_from_slice(&raw_value_slice[start..end]);
                result.push(u16::from_le_bytes(bytes));
            }
        }
    }
    #[cfg(not(all(target_arch = "x86_64", target_feature = "sse4.1")))]
    {
        // Fallback for other architectures or no SSE4.1
        for i in 0..count {
            let start = i * mem::size_of::<u16>();
            let end = start + mem::size_of::<u16>();
            let mut bytes = [0u8; mem::size_of::<u16>()];
            bytes.copy_from_slice(&raw_value_slice[start..end]);
            result.push(u16::from_le_bytes(bytes));
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
    fn test_decode_u16() {
        let encoded_u16 = encode_item(&HtlvItem::new(0, HtlvValue::U16(65535))).unwrap();
        let raw_value_slice_u16 = &encoded_u16[encoded_u16.len().checked_sub(2).unwrap()..]; // Length 2
        let decoded_u16 = decode_u16(2, raw_value_slice_u16).unwrap();
        assert_eq!(decoded_u16, HtlvValue::U16(65535));
    }

    #[test]
    fn test_decode_u16_errors() {
        // Invalid length for U16 (expected 2)
        let result = decode_u16(1, &[0x00]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Invalid length for U16 value: 1"
        );
    }

    #[test]
    fn test_decode_u16_batch() {
        // Test decoding a batch of U16 values
        let data: Vec<u8> = vec![
            0x01, 0x00, // 1
            0x02, 0x00, // 2
            0x03, 0x00, // 3
            0x04, 0x00, // 4
            0x05, 0x00, // 5
        ];
        let expected = vec![1, 2, 3, 4, 5];
        let decoded = decode_u16_batch(&data, 5).unwrap();
        assert_eq!(decoded, expected);

        // Test with incomplete data
        let incomplete_data: Vec<u8> = vec![
            0x01, 0x00, // 1
            0x02, // Incomplete 2
        ];
        let result = decode_u16_batch(&incomplete_data, 2);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Incomplete data for U16 batch decoding. Expected at least 4 bytes, got 3"
        );

        // Test with empty data
        let result = decode_u16_batch(&[], 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![] as Vec<u16>);

         // Test with empty data and count > 0
        let result = decode_u16_batch(&[], 1);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Incomplete data for U16 batch decoding. Expected at least 2 bytes, got 0"
        );
    }
}