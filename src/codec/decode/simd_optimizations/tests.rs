// Tests for SIMD optimizations
//
// This module contains tests for the SIMD-optimized implementations.

#[cfg(test)]
mod tests {
    use super::super::*;
    use std::mem;
    use std::slice;

    #[test]
    fn test_is_simd_available() {
        // This test just ensures the function runs without errors
        let _ = is_simd_available();
    }

    #[test]
    fn test_get_simd_instruction_set() {
        // This test just ensures the function runs without errors
        let _ = get_simd_instruction_set();
    }

    #[cfg(all(target_arch = "x86_64", feature = "simd"))]
    mod x86_64_tests {
        use super::super::super::{x86_64, BatchResult};
        use std::slice;
        use std::mem;

        #[test]
        fn test_is_instruction_set_available() {
            // This test just ensures the function runs without errors
            let _ = x86_64::is_instruction_set_available("sse4.1");
            let _ = x86_64::is_instruction_set_available("avx2");
            let _ = x86_64::is_instruction_set_available("avx512f");
            let _ = x86_64::is_instruction_set_available("unknown");
        }

        #[test]
        fn test_decode_u32_batch_simd() {
            // Skip the test if SSE4.1 is not available
            if !std::is_x86_feature_detected!("sse4.1") {
                return;
            }

            // Test with aligned data
            let values: Vec<u32> = vec![1, 2, 3, 4, 5, 6, 7, 8];
            let data: &[u8] = unsafe {
                slice::from_raw_parts(values.as_ptr() as *const u8, values.len() * mem::size_of::<u32>())
            };
            let expected: &[u32] = &[1, 2, 3, 4, 5, 6, 7, 8];
            let (batch_result, bytes_consumed) = x86_64::sse41::decode_u32_batch_simd(data).unwrap();
            assert_eq!(batch_result.as_slice(), expected);
            assert_eq!(bytes_consumed, data.len());
        }

        #[test]
        fn test_decode_f32_batch_simd() {
            // Skip the test if SSE4.1 is not available
            if !std::is_x86_feature_detected!("sse4.1") {
                return;
            }

            // Test with aligned data
            let values: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
            let data: &[u8] = unsafe {
                slice::from_raw_parts(values.as_ptr() as *const u8, values.len() * mem::size_of::<f32>())
            };
            let expected: &[f32] = &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
            let (batch_result, bytes_consumed) = x86_64::sse41::decode_f32_batch_simd(data).unwrap();
            assert_eq!(batch_result.as_slice(), expected);
            assert_eq!(bytes_consumed, data.len());
        }
    }

    #[test]
    fn test_integer_decode_u8_batch_simd() {
        let data = vec![1u8, 2, 3, 4, 5];
        let (batch_result, bytes_consumed) = integer::decode_u8_batch_simd(&data).unwrap();
        assert_eq!(batch_result.as_slice(), &[1, 2, 3, 4, 5]);
        assert_eq!(bytes_consumed, data.len());
    }

    #[test]
    fn test_integer_decode_i8_batch_simd() {
        let data = vec![1u8, 2, 3, 4, 5];
        let (batch_result, bytes_consumed) = integer::decode_i8_batch_simd(&data).unwrap();
        assert_eq!(batch_result.as_slice(), &[1i8, 2, 3, 4, 5]);
        assert_eq!(bytes_consumed, data.len());
    }

    #[test]
    fn test_integer_decode_u16_batch_simd() {
        let values = vec![1u16, 2, 3, 4];
        let data: &[u8] = unsafe {
            slice::from_raw_parts(values.as_ptr() as *const u8, values.len() * mem::size_of::<u16>())
        };
        let (batch_result, bytes_consumed) = integer::decode_u16_batch_simd(data).unwrap();
        assert_eq!(batch_result.as_slice(), &[1, 2, 3, 4]);
        assert_eq!(bytes_consumed, data.len());
    }

    #[test]
    fn test_integer_decode_i16_batch_simd() {
        let values = vec![1i16, 2, 3, 4];
        let data: &[u8] = unsafe {
            slice::from_raw_parts(values.as_ptr() as *const u8, values.len() * mem::size_of::<i16>())
        };
        let (batch_result, bytes_consumed) = integer::decode_i16_batch_simd(data).unwrap();
        assert_eq!(batch_result.as_slice(), &[1, 2, 3, 4]);
        assert_eq!(bytes_consumed, data.len());
    }

    #[test]
    fn test_float_decode_f32_batch_simd() {
        let values = vec![1.0f32, 2.0, 3.0, 4.0];
        let data: &[u8] = unsafe {
            slice::from_raw_parts(values.as_ptr() as *const u8, values.len() * mem::size_of::<f32>())
        };
        let (batch_result, bytes_consumed) = float::decode_f32_batch_simd(data).unwrap();
        assert_eq!(batch_result.as_slice(), &[1.0, 2.0, 3.0, 4.0]);
        assert_eq!(bytes_consumed, data.len());
    }

    #[test]
    fn test_float_decode_f64_batch_simd() {
        let values = vec![1.0f64, 2.0, 3.0, 4.0];
        let data: &[u8] = unsafe {
            slice::from_raw_parts(values.as_ptr() as *const u8, values.len() * mem::size_of::<f64>())
        };
        let (batch_result, bytes_consumed) = float::decode_f64_batch_simd(data).unwrap();
        assert_eq!(batch_result.as_slice(), &[1.0, 2.0, 3.0, 4.0]);
        assert_eq!(bytes_consumed, data.len());
    }

    #[test]
    fn test_unaligned_decode_u32_simd() {
        // Create an unaligned data slice (offset by one byte)
        let mut data = vec![0u8];
        for i in 0..8u32 {
            data.extend_from_slice(&i.to_le_bytes());
        }
        let slice = &data[1..];

        let (batch_result, _) = integer::decode_u32_batch_simd(slice).unwrap();
        assert_eq!(batch_result.as_slice(), &[0, 1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn test_unaligned_decode_f32_simd() {
        // Create an unaligned data slice (offset by one byte)
        let mut data = vec![0u8];
        for i in 0..4 {
            let f = i as f32 * 1.5;
            data.extend_from_slice(&f.to_le_bytes());
        }
        let slice = &data[1..];

        let (batch_result, _) = float::decode_f32_batch_simd(slice).unwrap();
        assert_eq!(batch_result.as_slice(), &[0.0, 1.5, 3.0, 4.5]);
    }

    #[cfg(all(target_arch = "x86_64", feature = "simd"))]
    #[test]
    fn test_string_contains_null_byte_simd() {
        let data_with_null = b"Hello\0World";
        let data_without_null = b"HelloWorld";

        assert!(string::contains_null_byte_simd(data_with_null));
        assert!(!string::contains_null_byte_simd(data_without_null));
    }

    #[test]
    fn test_string_count_utf8_chars_simd() {
        let data = "Hello, 世界!".as_bytes();
        let count = string::count_utf8_chars_simd(data).unwrap();
        assert_eq!(count, 10); // "Hello, 世界!" has 10 characters
    }
}
