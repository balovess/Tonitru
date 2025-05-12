use crate::internal::error::{Error, Result};
use super::Compressor; // Import the Compressor trait
use zstd; // Import the zstd crate
use std::fmt::Debug; // Import Debug trait

/// Compresses data using Zstandard algorithm.
pub fn compress(data: &[u8]) -> Result<Vec<u8>> {
    zstd::encode_all(data, 0).map_err(|e| Error::CompressionError(format!("Zstd compression failed: {}", e)))
}

/// Decompresses data using Zstandard algorithm.
pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    zstd::decode_all(data).map_err(|e| Error::CompressionError(format!("Zstd decompression failed: {}", e)))
}

/// Zstandard Compressor implementation.
#[derive(Debug)] // Added Debug derive
pub struct ZstdCompressor;

impl Compressor for ZstdCompressor {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Call the specific compression function
        compress(data)
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Call the specific decompression function
        decompress(data)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zstd_compression() {
        let original_data = b"This is a test string for Zstandard compression. This is a test string for Zstandard compression. This is a test string for Zstandard compression.";
        let compressor = ZstdCompressor;
        let compressed_data = compressor.compress(original_data).unwrap();
        assert_ne!(compressed_data, original_data.to_vec()); // Expect compression to change data
        let decompressed_data = compressor.decompress(&compressed_data).unwrap();
        assert_eq!(decompressed_data, original_data.to_vec());
    }

    #[test]
    fn test_zstd_empty_data() {
        let original_data = b"";
        let compressor = ZstdCompressor;
        let compressed_data = compressor.compress(original_data).unwrap();
        let decompressed_data = compressor.decompress(&compressed_data).unwrap();
        assert_eq!(decompressed_data, original_data.to_vec());
    }

    #[test]
    fn test_zstd_uncompressible_data() {
        let original_data = (0..255).collect::<Vec<u8>>(); // Data with high entropy
        let compressor = ZstdCompressor;
        let compressed_data = compressor.compress(&original_data).unwrap();
        // For uncompressible data, compressed size might be slightly larger or similar
        let decompressed_data = compressor.decompress(&compressed_data).unwrap();
        assert_eq!(decompressed_data, original_data);
    }

     #[test]
    fn test_zstd_invalid_data() {
        let invalid_data = vec![0xFF, 0xFF, 0xFF]; // Invalid zstd data
        let compressor = ZstdCompressor;
        let decompressed_result = compressor.decompress(&invalid_data);
        assert!(decompressed_result.is_err());
        assert!(decompressed_result.unwrap_err().to_string().contains("Zstd decompression failed"));
    }
}