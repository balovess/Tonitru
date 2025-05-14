// SIMD optimizations for x86_64 architecture
//
// This module contains SIMD-optimized implementations for batch decoding on x86_64 architecture.
// Different submodules implement optimizations for different instruction sets.

// Re-export instruction set specific modules
pub mod sse41;
pub mod avx2;
pub mod avx512;

// Helper function to check if a specific x86_64 SIMD instruction set is available
pub fn is_instruction_set_available(instruction_set: &str) -> bool {
    match instruction_set {
        "sse4.1" => std::is_x86_feature_detected!("sse4.1"),
        "avx2" => std::is_x86_feature_detected!("avx2"),
        "avx512f" => std::is_x86_feature_detected!("avx512f"),
        _ => false,
    }
}
