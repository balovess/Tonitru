use crate::internal::error::Result;
use std::fmt::Debug; // Import Debug trait

pub mod zstd;
// Removed lz4 module: pub mod lz4;
pub mod brotli;
pub mod no_compression;
pub mod sharded;
pub mod incremental;

/// Trait for compression algorithms.
pub trait Compressor: Debug { // Added Debug bound
    /// Compresses the given data.
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>>;

    /// Decompresses the given data.
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>>;
}

/// Defines the compression strategy to use.
#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)] // Ensure enum variants have a fixed u8 representation
pub enum CompressionStrategy {
    NoCompression = 0,
    Zstd = 1,
    // Removed Lz4 variant: Lz4,
    Brotli = 3, // Explicitly set to 3 to match packet.rs
    // TODO: Add other strategies if needed (e.g., based on data type)
}

/// Returns a Compressor implementation based on the given strategy.
pub fn get_compressor(strategy: CompressionStrategy) -> Result<Box<dyn Compressor>> {
    match strategy {
        CompressionStrategy::NoCompression => Ok(Box::new(no_compression::NoCompressionCompressor)),
        CompressionStrategy::Zstd => Ok(Box::new(zstd::ZstdCompressor)),
        // Removed Lz4 match arm: CompressionStrategy::Lz4 => Ok(Box::new(lz4::Lz4Compressor)),
        CompressionStrategy::Brotli => Ok(Box::new(brotli::BrotliCompressor)),
    }
}


// NOTE: Dynamic compression strategy selector has been permanently shelved.
// Zstd is now used as the default compression method.
//
// The compression module is designed to be easily extensible:
// 1. To add a new compression algorithm, create a new module (e.g., `new_algo.rs`)
// 2. Implement the `Compressor` trait for your algorithm
// 3. Add a new variant to the `CompressionStrategy` enum
// 4. Update the `get_compressor` function to return your compressor for the new strategy
//
// The core architecture allows for easy replacement of compression protocols
// without affecting other parts of the system.

#[cfg(test)]
mod tests {
    use super::*;
    // Removed unused import: use crate::internal::error::Error;

    #[test]
    fn test_get_compressor_zstd() {
        let compressor = get_compressor(CompressionStrategy::Zstd).unwrap();
        // Check if the returned compressor is of the correct type (ZstdCompressor)
        // This is a bit tricky with Box<dyn Trait>, but we can use downcasting or a helper method if needed.
        // For now, we'll rely on the fact that get_compressor explicitly boxes the correct type.
        // A more robust test might involve calling a method specific to ZstdCompressor if one existed,
        // or using `Any` and `downcast_ref`.
        // A simpler approach for this test is to just ensure it's not the NoCompression error.
        assert!(compressor.compress(b"test").is_ok()); // Basic check that the compressor is functional
    }

    // Removed LZ4 test:
    // #[test]
    // fn test_get_compressor_lz4() {
    //     let compressor = get_compressor(CompressionStrategy::Lz4).unwrap();
    //     assert!(compressor.compress(b"test").is_ok()); // Basic check
    // }

    #[test]
    fn test_get_compressor_brotli() {
        let compressor = get_compressor(CompressionStrategy::Brotli).unwrap();
        assert!(compressor.compress(b"test").is_ok()); // Basic check
    }

    #[test]
    fn test_get_compressor_no_compression() {
        let compressor = get_compressor(CompressionStrategy::NoCompression).unwrap();
        assert!(compressor.compress(b"test").is_ok()); // Basic check that the compressor is functional

        // Test that NoCompressionCompressor doesn't actually compress data
        let test_data = b"test data for no compression";
        let compressed = compressor.compress(test_data).unwrap();
        assert_eq!(compressed, test_data.to_vec()); // Data should be unchanged
    }
}