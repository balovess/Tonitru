use crate::codec::types::HtlvValue;
use crate::internal::error::{Error, Result};
use std::mem;

// Enable necessary features for SIMD intrinsics (requires Rust nightly or specific configuration)
#[cfg(target_arch = "x86_64")]
use std::is_x86_feature_detected;

// Import specific SIMD intrinsics based on target features
#[cfg(target_arch = "x86_64")]
#[cfg(target_feature = "sse2")]
use std::arch::x86_64::{_mm_loadu_pd, _mm_cvtsd_f64, _mm_cvtss_f32}; // SSE2 for f64, SSE for f32

#[cfg(target_arch = "x86_64")]
#[cfg(target_feature = "sse")]
use std::arch::x86_64::_mm_loadu_ps; // SSE for f32

/// Decodes an F32 HtlvValue from bytes.
pub fn decode_f32(length: u64, raw_value_slice: &[u8]) -> Result<HtlvValue> {
    if length as usize != mem::size_of::<f32>() {
        return Err(Error::CodecError(format!(
            "Invalid length for F32 value: {}",
            length
        )));
    }
    #[cfg(all(target_arch = "x86_64", target_feature = "sse2"))] // SSE2 includes SSE
    {
        if is_x86_feature_detected!("sse2") {
            // Use SIMD to load and extract the f32 value
            // Safety: We check data length above. raw_value_slice is guaranteed to be 4 bytes.
            let ptr = raw_value_slice.as_ptr() as *const f32;
            let val_m128 = unsafe { _mm_loadu_ps(ptr) }; // Load 4 f32s
            let val = unsafe { _mm_cvtss_f32(val_m128) }; // Extract the lowest single-precision float
            Ok(HtlvValue::F32(val))
        } else {
            // Fallback
            let mut bytes = [0u8; mem::size_of::<f32>()];
            bytes.copy_from_slice(raw_value_slice);
            Ok(HtlvValue::F32(f32::from_le_bytes(bytes)))
        }
    }
    #[cfg(not(all(target_arch = "x86_64", target_feature = "sse2")))]
    {
        // Fallback for other architectures or no SSE2
        let mut bytes = [0u8; mem::size_of::<f32>()];
        bytes.copy_from_slice(raw_value_slice);
        Ok(HtlvValue::F32(f32::from_le_bytes(bytes)))
    }
}

/// Decodes an F64 HtlvValue from bytes.
pub fn decode_f64(length: u64, raw_value_slice: &[u8]) -> Result<HtlvValue> {
    if length as usize != mem::size_of::<f64>() {
        return Err(Error::CodecError(format!(
            "Invalid length for F64 value: {}",
            length
        )));
    }
    #[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
    {
        if is_x86_feature_detected!("sse2") {
            // Use SIMD to load and extract the f64 value
            // Safety: We check data length above. raw_value_slice is guaranteed to be 8 bytes.
            let ptr = raw_value_slice.as_ptr() as *const f64;
            let val_m128d = unsafe { _mm_loadu_pd(ptr) }; // Load 2 f64s
            let val = unsafe { _mm_cvtsd_f64(val_m128d) }; // Extract the lowest double-precision float
            Ok(HtlvValue::F64(val))
        } else {
            // Fallback
            let mut bytes = [0u8; mem::size_of::<f64>()];
            bytes.copy_from_slice(raw_value_slice);
            Ok(HtlvValue::F64(f64::from_le_bytes(bytes)))
        }
    }
    #[cfg(not(all(target_arch = "x86_64", target_feature = "sse2")))]
    {
        // Fallback for other architectures or no SSE2
        let mut bytes = [0u8; mem::size_of::<f64>()];
        bytes.copy_from_slice(raw_value_slice);
        Ok(HtlvValue::F64(f64::from_le_bytes(bytes)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::encode::encode_item;
    use crate::codec::types::HtlvItem;

    #[test]
    fn test_decode_floats() {
        // Test F32
        let encoded_f32 = encode_item(&HtlvItem::new(0, HtlvValue::F32(3.14f32))).unwrap();
        let raw_value_slice_f32 = &encoded_f32[encoded_f32.len().checked_sub(4).unwrap()..]; // Length 4
        let decoded_f32 = decode_f32(4, raw_value_slice_f32).unwrap();
        assert_eq!(decoded_f32, HtlvValue::F32(3.14f32));

        // Test F64
        let encoded_f64 = encode_item(&HtlvItem::new(0, HtlvValue::F64(3.14))).unwrap();
        let raw_value_slice_f64 = &encoded_f64[encoded_f64.len().checked_sub(8).unwrap()..]; // Length 8
        let decoded_f64 = decode_f64(8, raw_value_slice_f64).unwrap();
        assert_eq!(decoded_f64, HtlvValue::F64(3.14));
    }

    #[test]
    fn test_decode_float_errors() {
        // Invalid length for F32 (expected 4)
        let result = decode_f32(3, &[0x00, 0x00, 0x00]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Invalid length for F32 value: 3"
        );

        // Invalid length for F64 (expected 8)
        let result = decode_f64(7, &[0x00; 7]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Invalid length for F64 value: 7"
        );
    }
}