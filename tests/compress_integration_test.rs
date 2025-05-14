use tonitru::compress::{
    CompressionStrategy, get_compressor, Compressor,
    sharded::ShardedCompressor,
    incremental::IncrementalCompressor,
};
use tonitru::internal::packet::{Packet, MetadataHeader, DataBody};

/// Tests the integration of the compression module with the packet module.
#[test]
fn test_compression_with_packet() {
    // Create test data
    let original_data = b"This is a test string for compression integration with packets.";

    // Test with different compression strategies
    let strategies = [
        CompressionStrategy::NoCompression,
        CompressionStrategy::Zstd,
        CompressionStrategy::Brotli,
    ];

    for strategy in &strategies {
        // Create a header with the current compression strategy
        let mut header = MetadataHeader {
            schema_id: 1,
            timestamp: 1678886400,
            shard_id: 10,
            flow_flags: 0,
            body_type: 0, // Will be set by build_packet
        };

        // Set compression strategy in header
        header.set_compression_strategy(*strategy);

        // Get the appropriate compressor
        let compressor = get_compressor(*strategy).unwrap();

        // Compress the data
        let compressed_data = compressor.compress(original_data).unwrap();

        // Create a compressed data body
        let body = DataBody::Compressed(compressed_data);

        // Build a packet with the compressed body
        let packet = Packet::build_packet(header, body).unwrap();

        // Encode the packet
        let mut encoded_packet = packet.header.encode().unwrap();
        encoded_packet.extend_from_slice(&packet.body.encode().unwrap());
        encoded_packet.extend_from_slice(&packet.checksum.encode());

        // Parse the packet
        let parsed_packet = Packet::parse_packet(&encoded_packet).unwrap();

        // Verify the parsed packet's compression strategy matches the original
        assert_eq!(parsed_packet.header.get_compression_strategy().unwrap(), *strategy);

        // Extract the compressed data from the parsed packet
        let compressed_body = match &parsed_packet.body {
            DataBody::Compressed(data) => data,
            _ => panic!("Expected Compressed data body"),
        };

        // Decompress the data
        let decompressed_data = compressor.decompress(compressed_body).unwrap();

        // Verify the decompressed data matches the original
        assert_eq!(decompressed_data, original_data.to_vec());
    }
}

/// Tests the sharded compression with large data.
#[test]
fn test_sharded_compression_with_large_data() {
    // Create a large test data (1MB)
    let original_data: Vec<u8> = (0..1_000_000).map(|i| (i % 256) as u8).collect();

    // Create a sharded compressor with a small shard size
    let shard_size = 100_000; // 100KB shards
    let compressor = ShardedCompressor::with_shard_size(CompressionStrategy::Zstd, shard_size);

    // Compress the data
    let compressed_data = compressor.compress(&original_data).unwrap();

    // Decompress the data
    let decompressed_data = compressor.decompress(&compressed_data).unwrap();

    // Verify the decompressed data matches the original
    assert_eq!(decompressed_data, original_data);
}

/// Tests the incremental compression with sequential data.
#[test]
fn test_incremental_compression_with_sequential_data() {
    // Create an incremental compressor
    let mut compressor = IncrementalCompressor::default();
    let context_id = 1;

    // Create a sequence of related data
    let data_parts = [
        b"This is part 1 of a message with some repeated content.",
        b"This is part 2 of a message with some repeated content.",
        b"This is part 3 of a message with some repeated content.",
        b"This is part 4 of a message with some repeated content.",
        b"This is part 5 of a message with some repeated content.",
    ];

    // Compress each part
    let mut compressed_parts = Vec::new();
    for part in &data_parts {
        let compressed = compressor.compress_with_context(*part, context_id).unwrap();
        compressed_parts.push(compressed);
    }

    // Decompress each part
    let mut decompressed_parts = Vec::new();
    for compressed in &compressed_parts {
        let decompressed = compressor.decompress_with_context(compressed, context_id).unwrap();
        decompressed_parts.push(decompressed);
    }

    // Verify each decompressed part matches the original
    for (i, part) in data_parts.iter().enumerate() {
        assert_eq!(decompressed_parts[i], part.to_vec());
    }
}

/// Tests the combination of sharded and incremental compression.
#[test]
fn test_combined_sharded_and_incremental_compression() {
    // Create a large test data with repeating patterns (500KB)
    let pattern = b"This is a repeating pattern for testing combined compression. ";
    let mut original_data = Vec::with_capacity(500_000);
    while original_data.len() < 500_000 {
        original_data.extend_from_slice(pattern);
    }

    // Create a sharded compressor with a small shard size
    let shard_size = 50_000; // 50KB shards
    let sharded_compressor = ShardedCompressor::with_shard_size(CompressionStrategy::Zstd, shard_size);

    // Compress the data into shards
    let shards = sharded_compressor.compress_to_shards(&original_data).unwrap();

    // Create an incremental compressor
    let mut incremental_compressor = IncrementalCompressor::default();
    let context_id = 1;

    // Compress each shard incrementally
    let mut compressed_shards = Vec::new();
    for shard in &shards {
        let compressed = incremental_compressor.compress_with_context(&shard.data, context_id).unwrap();
        compressed_shards.push(compressed);
    }

    // Decompress each shard incrementally
    let mut decompressed_shards = Vec::new();
    for compressed in &compressed_shards {
        let decompressed = incremental_compressor.decompress_with_context(compressed, context_id).unwrap();
        decompressed_shards.push(decompressed);
    }

    // Reconstruct the original data from decompressed shards
    let mut reconstructed_data = Vec::new();
    for (i, decompressed) in decompressed_shards.iter().enumerate() {
        // Verify the decompressed shard matches the original shard data
        assert_eq!(*decompressed, shards[i].data);

        // Decompress the shard
        let compressor = get_compressor(shards[i].metadata.strategy).unwrap();
        let decompressed_shard = compressor.decompress(decompressed).unwrap();

        // Add to the reconstructed data
        reconstructed_data.extend_from_slice(&decompressed_shard);
    }

    // Verify the reconstructed data matches the original
    assert_eq!(reconstructed_data, original_data);
}
