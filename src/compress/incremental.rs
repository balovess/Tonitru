use crate::internal::error::Result;
use super::{Compressor, CompressionStrategy, get_compressor};
use std::fmt::Debug;
use std::collections::HashMap;

/// Maximum number of contexts to keep in the cache.
/// This is to prevent memory leaks if many different context IDs are used.
const MAX_CONTEXT_CACHE_SIZE: usize = 100;

/// Represents a compression context for incremental compression.
#[derive(Debug, Clone)]
struct CompressionContext {
    /// The compression strategy used for this context.
    strategy: CompressionStrategy,
    /// The dictionary built from previous data.
    dictionary: Vec<u8>,
    /// The maximum size of the dictionary.
    max_dict_size: usize,
}

impl CompressionContext {
    /// Creates a new compression context with the specified strategy.
    fn new(strategy: CompressionStrategy, max_dict_size: usize) -> Self {
        CompressionContext {
            strategy,
            dictionary: Vec::new(),
            max_dict_size,
        }
    }

    /// Updates the dictionary with new data.
    fn update_dictionary(&mut self, data: &[u8]) {
        // If the new data alone is larger than max_dict_size, just use a portion of it
        if data.len() >= self.max_dict_size {
            // Use only the last max_dict_size bytes of the data
            let start = data.len() - self.max_dict_size;
            self.dictionary = data[start..].to_vec();
            return;
        }

        // If the dictionary plus new data would exceed max_dict_size, remove oldest data
        if self.dictionary.len() + data.len() > self.max_dict_size {
            // Keep only the most recent data that fits within max_dict_size
            let keep_size = self.max_dict_size - data.len();
            if keep_size < self.dictionary.len() {
                let start = self.dictionary.len() - keep_size;
                self.dictionary = self.dictionary[start..].to_vec();
            } else {
                // This shouldn't happen given the checks above, but just in case
                self.dictionary.clear();
            }
        }

        // Append the new data to the dictionary
        self.dictionary.extend_from_slice(data);
    }
}

/// Compressor that supports incremental compression.
///
/// This compressor maintains a dictionary of previously seen data for each context ID,
/// allowing for more efficient compression of similar data over time.
#[derive(Debug)]
pub struct IncrementalCompressor {
    /// The default compression strategy to use.
    default_strategy: CompressionStrategy,
    /// The maximum size of the dictionary for each context.
    max_dict_size: usize,
    /// Cache of compression contexts, keyed by context ID.
    contexts: HashMap<u64, CompressionContext>,
}

impl Default for IncrementalCompressor {
    fn default() -> Self {
        IncrementalCompressor {
            default_strategy: CompressionStrategy::Zstd, // Default to Zstd
            max_dict_size: 64 * 1024, // 64KB default dictionary size
            contexts: HashMap::new(),
        }
    }
}

impl IncrementalCompressor {
    /// Creates a new IncrementalCompressor with the specified strategy and default dictionary size.
    pub fn new(default_strategy: CompressionStrategy) -> Self {
        IncrementalCompressor {
            default_strategy,
            max_dict_size: 64 * 1024, // 64KB default dictionary size
            contexts: HashMap::new(),
        }
    }

    /// Creates a new IncrementalCompressor with the specified strategy and dictionary size.
    pub fn with_dict_size(default_strategy: CompressionStrategy, max_dict_size: usize) -> Self {
        IncrementalCompressor {
            default_strategy,
            max_dict_size,
            contexts: HashMap::new(),
        }
    }

    /// Compresses data incrementally using the specified context ID.
    ///
    /// The context ID is used to identify a stream of related data that can benefit from
    /// incremental compression. For example, all messages in a specific conversation could
    /// use the same context ID.
    ///
    /// Returns the compressed data.
    pub fn compress_with_context(&mut self, data: &[u8], context_id: u64) -> Result<Vec<u8>> {
        // Get or create the context
        let context = self.get_or_create_context(context_id);

        // Get the appropriate compressor for the context's strategy
        let compressor = get_compressor(context.strategy)?;

        // Compress the data
        // Note: In a real implementation with dictionary compression,
        // you would use the dictionary with the compression algorithm.
        // For now, we'll just compress normally for demonstration
        let compressed = compressor.compress(data)?;

        // Update the context's dictionary with the new data
        self.update_context_dictionary(context_id, data);

        // Return the compressed data
        Ok(compressed)
    }

    /// Decompresses data that was compressed incrementally using the specified context ID.
    pub fn decompress_with_context(&mut self, data: &[u8], context_id: u64) -> Result<Vec<u8>> {
        // Get or create the context
        let context = self.get_or_create_context(context_id);

        // Get the appropriate compressor for the context's strategy
        let compressor = get_compressor(context.strategy)?;

        // Decompress the data
        // Note: In a real implementation with dictionary compression,
        // you would use the dictionary with the decompression algorithm.
        // For now, we'll just decompress normally for demonstration
        let decompressed = compressor.decompress(data)?;

        // Update the context's dictionary with the decompressed data
        self.update_context_dictionary(context_id, &decompressed);

        // Return the decompressed data
        Ok(decompressed)
    }

    /// Gets or creates a compression context for the specified context ID.
    fn get_or_create_context(&mut self, context_id: u64) -> &CompressionContext {
        // Clean up the cache if it's too large
        if self.contexts.len() > MAX_CONTEXT_CACHE_SIZE {
            // This is a simple approach: just clear the entire cache
            // A more sophisticated approach would use LRU or similar
            self.contexts.clear();
        }

        // Get or create the context
        self.contexts.entry(context_id).or_insert_with(|| {
            CompressionContext::new(self.default_strategy, self.max_dict_size)
        })
    }

    /// Updates the dictionary of a compression context with new data.
    fn update_context_dictionary(&mut self, context_id: u64, data: &[u8]) {
        // This is a bit awkward due to Rust's borrowing rules
        // We need to clone the data first, then update the context
        let data_clone = data.to_vec();

        if let Some(context) = self.contexts.get_mut(&context_id) {
            context.update_dictionary(&data_clone);
        }
    }

    /// Clears the context for the specified context ID.
    pub fn clear_context(&mut self, context_id: u64) {
        self.contexts.remove(&context_id);
    }

    /// Clears all contexts.
    pub fn clear_all_contexts(&mut self) {
        self.contexts.clear();
    }
}

impl Compressor for IncrementalCompressor {
    /// Compresses the given data.
    ///
    /// Note: This implementation does not use incremental compression,
    /// as it doesn't have a context ID. Use `compress_with_context` for
    /// incremental compression.
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Get the appropriate compressor for the default strategy
        let compressor = get_compressor(self.default_strategy)?;

        // Compress the data normally
        compressor.compress(data)
    }

    /// Decompresses the given data.
    ///
    /// Note: This implementation does not use incremental decompression,
    /// as it doesn't have a context ID. Use `decompress_with_context` for
    /// incremental decompression.
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Get the appropriate compressor for the default strategy
        let compressor = get_compressor(self.default_strategy)?;

        // Decompress the data normally
        compressor.decompress(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_incremental_compression_basic() {
        // Create a test data
        let original_data = b"This is a test string for incremental compression.";

        // Create an incremental compressor with Zstd strategy
        let mut compressor = IncrementalCompressor::new(CompressionStrategy::Zstd);

        // Compress the data with a context ID
        let context_id = 1;
        let compressed_data = compressor.compress_with_context(original_data, context_id).unwrap();

        // Decompress the data with the same context ID
        let decompressed_data = compressor.decompress_with_context(&compressed_data, context_id).unwrap();

        // Verify the decompressed data matches the original
        assert_eq!(decompressed_data, original_data.to_vec());
    }

    #[test]
    fn test_incremental_compression_multiple_contexts() {
        // Create test data for two different contexts
        let data1 = b"This is data for context 1.";
        let data2 = b"This is completely different data for context 2.";

        // Create an incremental compressor
        let mut compressor = IncrementalCompressor::default();

        // Compress data with different context IDs
        let context1_id = 1;
        let context2_id = 2;
        let compressed1 = compressor.compress_with_context(data1, context1_id).unwrap();
        let compressed2 = compressor.compress_with_context(data2, context2_id).unwrap();

        // Decompress data with the same context IDs
        let decompressed1 = compressor.decompress_with_context(&compressed1, context1_id).unwrap();
        let decompressed2 = compressor.decompress_with_context(&compressed2, context2_id).unwrap();

        // Verify the decompressed data matches the original
        assert_eq!(decompressed1, data1.to_vec());
        assert_eq!(decompressed2, data2.to_vec());
    }

    #[test]
    fn test_incremental_compression_sequential_data() {
        // Create an incremental compressor
        let mut compressor = IncrementalCompressor::default();
        let context_id = 1;

        // Compress and decompress a sequence of related data
        let data1 = b"This is the first part of a message.";
        let data2 = b"This is the second part of a message.";
        let data3 = b"This is the third part of a message.";

        // Compress each part
        let compressed1 = compressor.compress_with_context(data1, context_id).unwrap();
        let compressed2 = compressor.compress_with_context(data2, context_id).unwrap();
        let compressed3 = compressor.compress_with_context(data3, context_id).unwrap();

        // Decompress each part
        let decompressed1 = compressor.decompress_with_context(&compressed1, context_id).unwrap();
        let decompressed2 = compressor.decompress_with_context(&compressed2, context_id).unwrap();
        let decompressed3 = compressor.decompress_with_context(&compressed3, context_id).unwrap();

        // Verify the decompressed data matches the original
        assert_eq!(decompressed1, data1.to_vec());
        assert_eq!(decompressed2, data2.to_vec());
        assert_eq!(decompressed3, data3.to_vec());
    }

    #[test]
    fn test_clear_context() {
        // Create an incremental compressor
        let mut compressor = IncrementalCompressor::default();
        let context_id = 1;

        // Compress some data to build up the context
        let data = b"This is some data to build up the context.";
        let _ = compressor.compress_with_context(data, context_id).unwrap();

        // Clear the context
        compressor.clear_context(context_id);

        // Verify the context was cleared by checking if a new context is created
        assert!(compressor.contexts.is_empty());
    }

    #[test]
    fn test_clear_all_contexts() {
        // Create an incremental compressor
        let mut compressor = IncrementalCompressor::default();

        // Compress data with different context IDs
        let _ = compressor.compress_with_context(b"Data for context 1", 1).unwrap();
        let _ = compressor.compress_with_context(b"Data for context 2", 2).unwrap();
        let _ = compressor.compress_with_context(b"Data for context 3", 3).unwrap();

        // Verify we have 3 contexts
        assert_eq!(compressor.contexts.len(), 3);

        // Clear all contexts
        compressor.clear_all_contexts();

        // Verify all contexts were cleared
        assert!(compressor.contexts.is_empty());
    }

    #[test]
    fn test_context_cache_size_limit() {
        // Create an incremental compressor with a small max context cache size
        let mut compressor = IncrementalCompressor::default();

        // Add more contexts than the maximum
        for i in 0..MAX_CONTEXT_CACHE_SIZE + 10 {
            let data = format!("Data for context {}", i).into_bytes();
            let _ = compressor.compress_with_context(&data, i as u64).unwrap();
        }

        // Verify the cache was cleaned up
        assert!(compressor.contexts.len() <= MAX_CONTEXT_CACHE_SIZE);
    }

    #[test]
    fn test_dictionary_size_limit() {
        // Create an incremental compressor with a small dictionary size
        let max_dict_size = 100;
        let mut compressor = IncrementalCompressor::with_dict_size(CompressionStrategy::Zstd, max_dict_size);
        let context_id = 1;

        // Add data larger than the dictionary size
        let large_data = vec![b'a'; 200];
        let _ = compressor.compress_with_context(&large_data, context_id).unwrap();

        // Verify the dictionary size is limited
        if let Some(context) = compressor.contexts.get(&context_id) {
            assert!(context.dictionary.len() <= max_dict_size);
        } else {
            panic!("Context not found");
        }
    }
}
