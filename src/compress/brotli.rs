use crate::internal::error::{Error, Result};
use super::Compressor; // Import the Compressor trait
use brotli; // Import the brotli crate
use std::io::{Read, Write};

/// Compresses data using Brotli algorithm.
pub fn compress(data: &[u8]) -> Result<Vec<u8>> {
    let mut writer = brotli::CompressorWriter::new(Vec::new(), 4096, 11, 22); // Adjust parameters as needed
    writer.write_all(data).map_err(|e| Error::CompressionError(format!("Brotli compression failed: {}", e)))?;
    writer.flush().map_err(|e| Error::CompressionError(format!("Brotli compression flush failed: {}", e)))?;
    Ok(writer.into_inner()) // Corrected to return Ok(Vec<u8>)
}

/// Decompresses data using Brotli algorithm.
pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    let mut reader = brotli::Decompressor::new(data, 4096);
    let mut decompressed_data = Vec::new();
    reader.read_to_end(&mut decompressed_data).map_err(|e| Error::CompressionError(format!("Brotli decompression failed: {}", e)))?;
    Ok(decompressed_data)
}

/// Brotli Compressor implementation.
#[derive(Debug)]
pub struct BrotliCompressor;

impl Compressor for BrotliCompressor {
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
    fn test_brotli_compression() {
        let original_data = b"This is a test string for Brotli compression. This is a test string for Brotli compression. This is a test string for Brotli compression.";
        let compressor = BrotliCompressor;
        let compressed_data = compressor.compress(original_data).unwrap();
        assert_ne!(compressed_data, original_data.to_vec()); // Expect compression to change data
        let decompressed_data = compressor.decompress(&compressed_data).unwrap();
        assert_eq!(decompressed_data, original_data.to_vec());
    }

    #[test]
    fn test_brotli_empty_data() {
        let original_data = b"";
        let compressor = BrotliCompressor;
        let compressed_data = compressor.compress(original_data).unwrap();
        let decompressed_data = compressor.decompress(&compressed_data).unwrap();
        assert_eq!(decompressed_data, original_data.to_vec());
    }

    #[test]
    fn test_brotli_uncompressible_data() {
        let original_data = (0..255).collect::<Vec<u8>>(); // Data with high entropy
        let compressor = BrotliCompressor;
        let compressed_data = compressor.compress(&original_data).unwrap();
        // For uncompressible data, compressed size might be slightly larger or similar
        let decompressed_data = compressor.decompress(&compressed_data).unwrap();
        assert_eq!(decompressed_data, original_data);
    }

     #[test]
    fn test_brotli_invalid_data() {
        let invalid_data = vec![0xFF, 0xFF, 0xFF]; // Invalid brotli data
        let compressor = BrotliCompressor;
        let decompressed_result = compressor.decompress(&invalid_data);
        assert!(decompressed_result.is_err());
        assert!(decompressed_result.unwrap_err().to_string().contains("Brotli decompression failed"));
    }
}