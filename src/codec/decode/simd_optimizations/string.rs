// SIMD optimizations for string processing
//
// This module contains SIMD-optimized implementations for string processing.

use crate::internal::error::{Error, Result};
use std::str::Utf8Error;

// Import architecture-specific modules
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

// Implement From<Utf8Error> for Error
impl From<Utf8Error> for Error {
    fn from(error: Utf8Error) -> Self {
        Error::CodecError(format!("UTF-8 error: {}", error))
    }
}

/// Checks if a byte slice contains any null bytes using SIMD instructions.
/// This is useful for validating strings in the HTLV format.
#[cfg(all(target_arch = "x86_64", feature = "simd"))]
pub fn contains_null_byte_simd(data: &[u8]) -> bool {
    if !std::is_x86_feature_detected!("sse4.1") {
        // Fallback to scalar implementation
        return data.contains(&0);
    }

    let len = data.len();
    let mut i = 0;

    // Process 16 bytes at a time using SSE4.1
    unsafe {
        while i + 16 <= len {
            let chunk = _mm_loadu_si128(data[i..].as_ptr() as *const __m128i);
            let zero_mask = _mm_cmpeq_epi8(chunk, _mm_setzero_si128());
            let mask = _mm_movemask_epi8(zero_mask);

            if mask != 0 {
                return true;
            }

            i += 16;
        }
    }

    // Process remaining bytes
    data[i..].contains(&0)
}

/// Checks if a byte slice contains any null bytes using scalar code.
#[cfg(not(all(target_arch = "x86_64", feature = "simd")))]
pub fn contains_null_byte_simd(data: &[u8]) -> bool {
    data.contains(&0)
}

/// Counts the number of UTF-8 characters in a byte slice using SIMD instructions.
/// This is useful for efficiently processing strings in the HTLV format.
#[cfg(all(target_arch = "x86_64", feature = "simd"))]
pub fn count_utf8_chars_simd(data: &[u8]) -> Result<usize> {
    if !std::is_x86_feature_detected!("sse4.1") {
        // Fallback to scalar implementation
        return Ok(std::str::from_utf8(data)?.chars().count());
    }

    // This is a simplified implementation that counts non-continuation bytes
    // A more accurate implementation would need to validate UTF-8 sequences
    let len = data.len();
    let mut i = 0;
    let mut count = 0;

    // Process 16 bytes at a time using SSE4.1
    unsafe {
        while i + 16 <= len {
            let chunk = _mm_loadu_si128(data[i..].as_ptr() as *const __m128i);

            // Continuation bytes in UTF-8 start with 10xxxxxx (0x80-0xBF)
            // We want to count bytes that are NOT continuation bytes
            let continuation_mask = _mm_set1_epi8(0x80u8 as i8);
            let is_continuation = _mm_and_si128(chunk, _mm_set1_epi8(0xC0u8 as i8));
            let is_continuation = _mm_cmpeq_epi8(is_continuation, continuation_mask);
            let not_continuation = _mm_cmpeq_epi8(is_continuation, _mm_setzero_si128());

            let mask = _mm_movemask_epi8(not_continuation);
            count += mask.count_ones() as usize;

            i += 16;
        }
    }

    // Process remaining bytes
    for &byte in &data[i..] {
        // Count bytes that are not continuation bytes (don't start with 10xxxxxx)
        if (byte & 0xC0) != 0x80 {
            count += 1;
        }
    }

    // Validate that the data is valid UTF-8
    let _ = std::str::from_utf8(data)?;

    Ok(count)
}

/// Counts the number of UTF-8 characters in a byte slice using scalar code.
#[cfg(not(all(target_arch = "x86_64", feature = "simd")))]
pub fn count_utf8_chars_simd(data: &[u8]) -> Result<usize> {
    Ok(std::str::from_utf8(data)?.chars().count())
}
