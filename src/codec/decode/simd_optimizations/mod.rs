// SIMD optimizations for batch decoding
//
// This module contains SIMD-optimized implementations for batch decoding of basic types.
// These implementations are used when the target architecture supports the required SIMD features.
//
// The module is organized by architecture and instruction set:
// - x86_64: SSE4.1, AVX2, AVX-512
// - aarch64: NEON

// Import error types
#[allow(unused_imports)]
use crate::internal::error::Result;

// Re-export architecture-specific modules
#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[cfg(target_arch = "aarch64")]
pub mod aarch64;

// Re-export type-specific modules
pub mod integer;
pub mod float;
pub mod string;
pub mod batch_result;

// Tests module
#[cfg(test)]
mod tests;

// Re-export BatchResult
pub use batch_result::BatchResult;

// Re-export the main functions for backward compatibility
// These functions now return BatchResult instead of raw slices
#[cfg(target_arch = "x86_64")]
pub use x86_64::sse41::decode_u32_batch_simd;

#[cfg(target_arch = "x86_64")]
pub use x86_64::sse41::decode_f32_batch_simd;

#[cfg(target_arch = "x86_64")]
pub use x86_64::sse41::decode_u8_batch_simd;

#[cfg(target_arch = "x86_64")]
pub use x86_64::sse41::decode_i8_batch_simd;

// Helper function to check if SIMD is available for the current platform
pub fn is_simd_available() -> bool {
    #[cfg(all(target_arch = "x86_64", feature = "simd"))]
    {
        std::is_x86_feature_detected!("sse4.1")
    }
    #[cfg(all(target_arch = "aarch64", feature = "simd"))]
    {
        // ARM NEON is always available on aarch64
        true
    }
    #[cfg(not(any(
        all(target_arch = "x86_64", feature = "simd"),
        all(target_arch = "aarch64", feature = "simd")
    )))]
    {
        // Default to false for unsupported platforms or when SIMD feature is disabled
        false
    }
}

// Helper function to get the best available SIMD instruction set for the current platform
pub fn get_simd_instruction_set() -> Option<&'static str> {
    #[cfg(all(target_arch = "x86_64", feature = "simd"))]
    {
        if std::is_x86_feature_detected!("avx512f") {
            return Some("avx512f");
        } else if std::is_x86_feature_detected!("avx2") {
            return Some("avx2");
        } else if std::is_x86_feature_detected!("sse4.1") {
            return Some("sse4.1");
        }
    }

    #[cfg(all(target_arch = "aarch64", feature = "simd"))]
    {
        return Some("neon");
    }

    None
}
