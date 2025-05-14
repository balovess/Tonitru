use crate::internal::error::{Error, Result};
use super::{Compressor, CompressionStrategy, get_compressor};
use std::fmt::Debug;

/// Maximum size of a single shard in bytes.
/// This is set to 1MB by default, which is a good balance between compression efficiency and memory usage.
pub const DEFAULT_SHARD_SIZE: usize = 1024 * 1024; // 1MB

/// Metadata for a compressed shard.
#[derive(Debug, Clone)]
pub struct ShardMetadata {
    /// The compression strategy used for this shard.
    pub strategy: CompressionStrategy,
    /// The original size of the data before compression.
    pub original_size: u32,
    /// The compressed size of the data.
    pub compressed_size: u32,
}

/// A compressed shard, containing metadata and compressed data.
#[derive(Debug, Clone)]
pub struct CompressedShard {
    /// Metadata for the shard.
    pub metadata: ShardMetadata,
    /// The compressed data.
    pub data: Vec<u8>,
}

/// Compressor that supports sharded compression.
///
/// This compressor divides large data into smaller shards and compresses each shard independently.
/// This allows for:
/// - Parallel decompression of shards
/// - Better memory usage when working with large data
/// - Improved resilience (corruption in one shard doesn't affect others)
#[derive(Debug, Clone)]
pub struct ShardedCompressor {
    /// The maximum size of a single shard in bytes.
    pub shard_size: usize,
    /// The compression strategy to use for each shard.
    pub strategy: CompressionStrategy,
}

impl Default for ShardedCompressor {
    fn default() -> Self {
        ShardedCompressor {
            shard_size: DEFAULT_SHARD_SIZE,
            strategy: CompressionStrategy::Zstd, // Default to Zstd
        }
    }
}

impl ShardedCompressor {
    /// Creates a new ShardedCompressor with the specified strategy and default shard size.
    pub fn new(strategy: CompressionStrategy) -> Self {
        ShardedCompressor {
            shard_size: DEFAULT_SHARD_SIZE,
            strategy,
        }
    }

    /// Creates a new ShardedCompressor with the specified strategy and shard size.
    pub fn with_shard_size(strategy: CompressionStrategy, shard_size: usize) -> Self {
        ShardedCompressor {
            shard_size,
            strategy,
        }
    }

    /// Compresses data into shards.
    ///
    /// Returns a vector of CompressedShard objects.
    pub fn compress_to_shards(&self, data: &[u8]) -> Result<Vec<CompressedShard>> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        let mut shards = Vec::new();
        let mut offset = 0;

        // Get the appropriate compressor for the strategy
        let compressor = get_compressor(self.strategy)?;

        while offset < data.len() {
            // Calculate the end of this shard
            let end = std::cmp::min(offset + self.shard_size, data.len());
            let shard_data = &data[offset..end];

            // Compress the shard
            let compressed_data = compressor.compress(shard_data)?;

            // Create metadata for the shard
            let metadata = ShardMetadata {
                strategy: self.strategy,
                original_size: shard_data.len() as u32,
                compressed_size: compressed_data.len() as u32,
            };

            // Add the compressed shard to the list
            shards.push(CompressedShard {
                metadata,
                data: compressed_data,
            });

            // Move to the next shard
            offset = end;
        }

        Ok(shards)
    }

    /// Decompresses shards back into the original data.
    pub fn decompress_from_shards(&self, shards: &[CompressedShard]) -> Result<Vec<u8>> {
        if shards.is_empty() {
            return Ok(Vec::new());
        }

        // Pre-allocate the result buffer based on the sum of original sizes
        let total_size: usize = shards.iter()
            .map(|shard| shard.metadata.original_size as usize)
            .sum();
        let mut result = Vec::with_capacity(total_size);

        for shard in shards {
            // Get the appropriate compressor for this shard
            let compressor = get_compressor(shard.metadata.strategy)?;

            // Decompress the shard
            let decompressed_data = compressor.decompress(&shard.data)?;

            // Verify the decompressed size matches the original size
            if decompressed_data.len() != shard.metadata.original_size as usize {
                return Err(Error::CompressionError(format!(
                    "Decompressed size mismatch: expected {}, got {}",
                    shard.metadata.original_size,
                    decompressed_data.len()
                )));
            }

            // Append the decompressed data to the result
            result.extend_from_slice(&decompressed_data);
        }

        Ok(result)
    }
}

impl Compressor for ShardedCompressor {
    /// Compresses the given data using sharded compression.
    ///
    /// The compressed format includes:
    /// - Number of shards (4 bytes)
    /// - For each shard:
    ///   - Compression strategy (1 byte)
    ///   - Original size (4 bytes)
    ///   - Compressed size (4 bytes)
    ///   - Compressed data (variable length)
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Compress the data into shards
        let shards = self.compress_to_shards(data)?;

        // Calculate the total size needed for the compressed data
        let metadata_size = 4 + (shards.len() * 9); // 4 bytes for shard count + 9 bytes per shard metadata
        let data_size: usize = shards.iter().map(|shard| shard.data.len()).sum();
        let total_size = metadata_size + data_size;

        // Create the result buffer
        let mut result = Vec::with_capacity(total_size);

        // Write the number of shards
        result.extend_from_slice(&(shards.len() as u32).to_le_bytes());

        // Write each shard
        for shard in &shards {
            // Write the compression strategy
            result.push(shard.metadata.strategy as u8);

            // Write the original size
            result.extend_from_slice(&shard.metadata.original_size.to_le_bytes());

            // Write the compressed size
            result.extend_from_slice(&shard.metadata.compressed_size.to_le_bytes());

            // Write the compressed data
            result.extend_from_slice(&shard.data);
        }

        Ok(result)
    }

    /// Decompresses the given data that was compressed using sharded compression.
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        // Ensure we have at least 4 bytes for the shard count
        if data.len() < 4 {
            return Err(Error::CompressionError("Invalid sharded compression data: too short".to_string()));
        }

        // Read the number of shards
        let mut shard_count_bytes = [0u8; 4];
        shard_count_bytes.copy_from_slice(&data[0..4]);
        let shard_count = u32::from_le_bytes(shard_count_bytes) as usize;

        // Parse the shards
        let mut shards = Vec::with_capacity(shard_count);
        let mut offset = 4; // Start after the shard count

        for _ in 0..shard_count {
            // Ensure we have enough data for the shard metadata
            if offset + 9 > data.len() {
                return Err(Error::CompressionError("Invalid sharded compression data: truncated metadata".to_string()));
            }

            // Read the compression strategy
            let strategy_byte = data[offset];
            offset += 1;

            // Convert the strategy byte to a CompressionStrategy
            let strategy = match strategy_byte {
                0 => CompressionStrategy::NoCompression,
                1 => CompressionStrategy::Zstd,
                3 => CompressionStrategy::Brotli,
                _ => return Err(Error::CompressionError(format!("Unknown compression strategy: {}", strategy_byte))),
            };

            // Read the original size
            let mut original_size_bytes = [0u8; 4];
            original_size_bytes.copy_from_slice(&data[offset..offset+4]);
            let original_size = u32::from_le_bytes(original_size_bytes);
            offset += 4;

            // Read the compressed size
            let mut compressed_size_bytes = [0u8; 4];
            compressed_size_bytes.copy_from_slice(&data[offset..offset+4]);
            let compressed_size = u32::from_le_bytes(compressed_size_bytes);
            offset += 4;

            // Ensure we have enough data for the compressed data
            if offset + compressed_size as usize > data.len() {
                return Err(Error::CompressionError("Invalid sharded compression data: truncated shard data".to_string()));
            }

            // Read the compressed data
            let shard_data = data[offset..offset+compressed_size as usize].to_vec();
            offset += compressed_size as usize;

            // Create the shard metadata
            let metadata = ShardMetadata {
                strategy,
                original_size,
                compressed_size,
            };

            // Add the shard to the list
            shards.push(CompressedShard {
                metadata,
                data: shard_data,
            });
        }

        // Decompress the shards
        self.decompress_from_shards(&shards)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sharded_compression_single_shard() {
        // Create a test data smaller than the default shard size
        let original_data = b"This is a test string for sharded compression.";

        // Create a sharded compressor with Zstd strategy
        let compressor = ShardedCompressor::new(CompressionStrategy::Zstd);

        // Compress the data
        let compressed_data = compressor.compress(original_data).unwrap();

        // Decompress the data
        let decompressed_data = compressor.decompress(&compressed_data).unwrap();

        // Verify the decompressed data matches the original
        assert_eq!(decompressed_data, original_data.to_vec());
    }

    #[test]
    fn test_sharded_compression_multiple_shards() {
        // Create a test data larger than a small shard size
        let original_data: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();

        // Create a sharded compressor with a small shard size
        let compressor = ShardedCompressor::with_shard_size(CompressionStrategy::Zstd, 1000);

        // Compress the data
        let compressed_data = compressor.compress(&original_data).unwrap();

        // Decompress the data
        let decompressed_data = compressor.decompress(&compressed_data).unwrap();

        // Verify the decompressed data matches the original
        assert_eq!(decompressed_data, original_data);
    }

    #[test]
    fn test_sharded_compression_empty_data() {
        let original_data = b"";
        let compressor = ShardedCompressor::default();

        // Compress empty data
        let compressed_data = compressor.compress(original_data).unwrap();

        // Decompress the empty data
        let decompressed_data = compressor.decompress(&compressed_data).unwrap();

        // Verify the decompressed data is an empty vector
        assert_eq!(decompressed_data, Vec::<u8>::new());
    }

    #[test]
    fn test_compress_to_shards() {
        // Create test data larger than a small shard size
        let original_data: Vec<u8> = (0..5000).map(|i| (i % 256) as u8).collect();

        // Create a sharded compressor with a small shard size
        let compressor = ShardedCompressor::with_shard_size(CompressionStrategy::Zstd, 1000);

        // Compress the data into shards
        let shards = compressor.compress_to_shards(&original_data).unwrap();

        // Verify we have the expected number of shards
        assert_eq!(shards.len(), 5); // 5000 bytes / 1000 bytes per shard = 5 shards

        // Verify each shard has the correct metadata
        for (i, shard) in shards.iter().enumerate() {
            assert_eq!(shard.metadata.strategy, CompressionStrategy::Zstd);

            // All shards except the last one should have original_size = shard_size
            if i < 4 {
                assert_eq!(shard.metadata.original_size, 1000);
            } else {
                assert_eq!(shard.metadata.original_size, 1000); // Last shard is also 1000 in this case
            }
        }

        // Decompress the shards
        let decompressed_data = compressor.decompress_from_shards(&shards).unwrap();

        // Verify the decompressed data matches the original
        assert_eq!(decompressed_data, original_data);
    }

    #[test]
    fn test_different_compression_strategies() {
        // Create test data
        let original_data: Vec<u8> = (0..5000).map(|i| (i % 256) as u8).collect();

        // Test with different compression strategies
        let strategies = [
            CompressionStrategy::NoCompression,
            CompressionStrategy::Zstd,
            CompressionStrategy::Brotli,
        ];

        for strategy in &strategies {
            // Create a sharded compressor with the current strategy
            let compressor = ShardedCompressor::new(*strategy);

            // Compress and decompress
            let compressed_data = compressor.compress(&original_data).unwrap();
            let decompressed_data = compressor.decompress(&compressed_data).unwrap();

            // Verify the decompressed data matches the original
            assert_eq!(decompressed_data, original_data);
        }
    }

    #[test]
    fn test_invalid_compressed_data() {
        let compressor = ShardedCompressor::default();

        // Test with empty data
        let result = compressor.decompress(&[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Vec::<u8>::new());

        // Test with data that's too short
        let result = compressor.decompress(&[1, 2, 3]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too short"));

        // Test with invalid shard count
        let mut invalid_data = Vec::new();
        invalid_data.extend_from_slice(&(10u32).to_le_bytes()); // Claim 10 shards
        invalid_data.extend_from_slice(&[1, 2, 3]); // But provide only 3 bytes of data

        let result = compressor.decompress(&invalid_data);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("truncated metadata"));
    }
}
