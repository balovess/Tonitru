use crate::internal::error::Result;
use crate::internal::error::Error; // Import Error for the default case
use std::fmt::Debug; // Import Debug trait

pub mod zstd;
// Removed lz4 module: pub mod lz4;
pub mod brotli;

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
        CompressionStrategy::NoCompression => Err(Error::CompressionError("No compression strategy selected".to_string())), // Or return a NoOpCompressor
        CompressionStrategy::Zstd => Ok(Box::new(zstd::ZstdCompressor)),
        // Removed Lz4 match arm: CompressionStrategy::Lz4 => Ok(Box::new(lz4::Lz4Compressor)),
        CompressionStrategy::Brotli => Ok(Box::new(brotli::BrotliCompressor)),
    }
}


// TODO: Implement dynamic compression strategy logic (partially done with get_compressor)
// TODO: Implement sharded compression logic
// TODO: Implement incremental compression logic

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
        let result = get_compressor(CompressionStrategy::NoCompression);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Compression Error: No compression strategy selected"); // Corrected expected error string
    }
}