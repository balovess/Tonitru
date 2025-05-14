use crate::internal::error::Result;
use super::Compressor;
use std::fmt::Debug;

/// A compressor that performs no compression, simply returning the original data.
/// 
/// This is useful for cases where compression would not be beneficial, such as:
/// - Already compressed data (images, videos, etc.)
/// - Very small data where compression overhead exceeds benefits
/// - Low-latency scenarios where compression time is unacceptable
/// - High-bandwidth scenarios where compression is unnecessary
#[derive(Debug)]
pub struct NoCompressionCompressor;

impl Compressor for NoCompressionCompressor {
    /// "Compresses" the data by simply cloning it.
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        Ok(data.to_vec())
    }

    /// "Decompresses" the data by simply cloning it.
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        Ok(data.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_compression() {
        let original_data = b"This is a test string that should not be compressed.";
        let compressor = NoCompressionCompressor;
        
        // Compress the data
        let compressed_data = compressor.compress(original_data).unwrap();
        
        // Verify the compressed data is identical to the original
        assert_eq!(compressed_data, original_data.to_vec());
        
        // Decompress the data
        let decompressed_data = compressor.decompress(&compressed_data).unwrap();
        
        // Verify the decompressed data is identical to the original
        assert_eq!(decompressed_data, original_data.to_vec());
    }

    #[test]
    fn test_no_compression_empty_data() {
        let original_data = b"";
        let compressor = NoCompressionCompressor;
        
        // Compress empty data
        let compressed_data = compressor.compress(original_data).unwrap();
        
        // Verify the compressed data is an empty vector
        assert_eq!(compressed_data, Vec::<u8>::new());
        
        // Decompress the empty data
        let decompressed_data = compressor.decompress(&compressed_data).unwrap();
        
        // Verify the decompressed data is an empty vector
        assert_eq!(decompressed_data, Vec::<u8>::new());
    }

    #[test]
    fn test_no_compression_large_data() {
        // Create a large data set (100KB)
        let original_data: Vec<u8> = (0..102400).map(|i| (i % 256) as u8).collect();
        let compressor = NoCompressionCompressor;
        
        // Compress the large data
        let compressed_data = compressor.compress(&original_data).unwrap();
        
        // Verify the compressed data is identical to the original
        assert_eq!(compressed_data, original_data);
        
        // Decompress the data
        let decompressed_data = compressor.decompress(&compressed_data).unwrap();
        
        // Verify the decompressed data is identical to the original
        assert_eq!(decompressed_data, original_data);
    }
}
