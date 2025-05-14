use crate::internal::error::{Error, Result};
use super::{CompressionStrategy, Compressor, get_compressor};

/// Enum representing different data types for compression strategy selection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataType {
    /// Numeric data (integers, floats)
    Numeric,
    /// Text data (strings, JSON, XML)
    Text,
    /// Binary data (images, audio, video, etc.)
    Binary,
    /// Unknown data type
    Unknown,
}

/// Struct representing network bandwidth information.
#[derive(Debug, Clone, Copy)]
pub struct BandwidthInfo {
    /// Available bandwidth in Mbps
    pub available_mbps: f32,
    /// Latency in milliseconds
    pub latency_ms: f32,
    /// Jitter in milliseconds
    pub jitter_ms: f32,
}

impl Default for BandwidthInfo {
    fn default() -> Self {
        // Default to medium bandwidth, medium latency
        BandwidthInfo {
            available_mbps: 10.0, // Assume 10 Mbps by default
            latency_ms: 50.0,     // Assume 50ms latency by default
            jitter_ms: 5.0,       // Assume 5ms jitter by default
        }
    }
}

/// Struct for selecting the optimal compression strategy based on data type and network conditions.
#[derive(Debug)]
pub struct CompressionSelector {
    /// Default compression strategy to use when no specific strategy is determined
    default_strategy: CompressionStrategy,
    /// Current bandwidth information
    bandwidth_info: BandwidthInfo,
}

impl Default for CompressionSelector {
    fn default() -> Self {
        CompressionSelector {
            default_strategy: CompressionStrategy::Zstd,
            bandwidth_info: BandwidthInfo::default(),
        }
    }
}

impl CompressionSelector {
    /// Creates a new CompressionSelector with the specified default strategy.
    pub fn new(default_strategy: CompressionStrategy) -> Self {
        CompressionSelector {
            default_strategy,
            bandwidth_info: BandwidthInfo::default(),
        }
    }

    /// Updates the bandwidth information.
    pub fn update_bandwidth(&mut self, bandwidth_info: BandwidthInfo) {
        self.bandwidth_info = bandwidth_info;
    }

    /// Detects the data type based on the provided data.
    ///
    /// This is a simple heuristic and may not be accurate for all data.
    /// More sophisticated detection can be implemented as needed.
    pub fn detect_data_type(data: &[u8]) -> DataType {
        if data.is_empty() {
            return DataType::Unknown;
        }

        // Sample the first 1024 bytes or the entire data if smaller
        let sample_size = std::cmp::min(data.len(), 1024);
        let sample = &data[..sample_size];

        // Count different byte patterns
        let mut text_chars = 0;
        let mut binary_chars = 0;
        let mut numeric_chars = 0;

        for &byte in sample {
            if byte >= 48 && byte <= 57 {
                // 0-9
                numeric_chars += 1;
            } else if (byte >= 32 && byte <= 126) || byte == 9 || byte == 10 || byte == 13 {
                // Printable ASCII, tab, newline, carriage return
                text_chars += 1;
            } else {
                // Other bytes (likely binary)
                binary_chars += 1;
            }
        }

        // Determine the dominant type
        if numeric_chars > text_chars && numeric_chars > binary_chars {
            DataType::Numeric
        } else if text_chars > binary_chars {
            DataType::Text
        } else {
            DataType::Binary
        }
    }

    /// Selects the optimal compression strategy based on data type and network conditions.
    pub fn select_strategy(&self, data: &[u8]) -> CompressionStrategy {
        let data_type = Self::detect_data_type(data);
        self.select_strategy_for_type(data_type)
    }

    /// Selects the optimal compression strategy for a specific data type.
    pub fn select_strategy_for_type(&self, data_type: DataType) -> CompressionStrategy {
        match data_type {
            DataType::Numeric => {
                // For numeric data, Zstd is generally efficient
                CompressionStrategy::Zstd
            }
            DataType::Text => {
                // For text data, Brotli often provides better compression
                CompressionStrategy::Brotli
            }
            DataType::Binary => {
                // For binary data, consider network conditions
                if self.bandwidth_info.latency_ms < 10.0 {
                    // For very low latency requirements, prefer no compression
                    CompressionStrategy::NoCompression
                } else if self.bandwidth_info.available_mbps > 100.0 {
                    // For high bandwidth, prefer faster compression
                    CompressionStrategy::Zstd
                } else {
                    // For limited bandwidth, prefer better compression ratio
                    CompressionStrategy::Brotli
                }
            }
            DataType::Unknown => self.default_strategy,
        }
    }

    /// Gets a compressor based on the selected strategy for the given data.
    pub fn get_compressor_for_data(&self, data: &[u8]) -> Result<Box<dyn Compressor>> {
        let strategy = self.select_strategy(data);
        get_compressor(strategy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_data_type_numeric() {
        let data = b"12345678901234567890";
        assert_eq!(CompressionSelector::detect_data_type(data), DataType::Numeric);
    }

    #[test]
    fn test_detect_data_type_text() {
        let data = b"This is a text string with some numbers 123";
        assert_eq!(CompressionSelector::detect_data_type(data), DataType::Text);
    }

    #[test]
    fn test_detect_data_type_binary() {
        // Create some binary-like data
        let mut data = Vec::with_capacity(100);
        for i in 0..100 {
            data.push(i as u8);
        }
        assert_eq!(CompressionSelector::detect_data_type(&data), DataType::Binary);
    }

    #[test]
    fn test_select_strategy_for_type() {
        let selector = CompressionSelector::default();
        
        // Test numeric data
        assert_eq!(selector.select_strategy_for_type(DataType::Numeric), CompressionStrategy::Zstd);
        
        // Test text data
        assert_eq!(selector.select_strategy_for_type(DataType::Text), CompressionStrategy::Brotli);
        
        // Test binary data with default bandwidth
        assert_eq!(selector.select_strategy_for_type(DataType::Binary), CompressionStrategy::Brotli);
        
        // Test unknown data
        assert_eq!(selector.select_strategy_for_type(DataType::Unknown), CompressionStrategy::Zstd);
    }

    #[test]
    fn test_select_strategy_with_different_bandwidth() {
        // Create a selector with high bandwidth and low latency
        let mut selector = CompressionSelector::default();
        selector.update_bandwidth(BandwidthInfo {
            available_mbps: 200.0,
            latency_ms: 5.0,
            jitter_ms: 1.0,
        });
        
        // For binary data with high bandwidth and very low latency, should prefer no compression
        assert_eq!(selector.select_strategy_for_type(DataType::Binary), CompressionStrategy::NoCompression);
        
        // Update to high bandwidth but higher latency
        selector.update_bandwidth(BandwidthInfo {
            available_mbps: 200.0,
            latency_ms: 20.0,
            jitter_ms: 1.0,
        });
        
        // For binary data with high bandwidth and moderate latency, should prefer Zstd
        assert_eq!(selector.select_strategy_for_type(DataType::Binary), CompressionStrategy::Zstd);
    }
}
