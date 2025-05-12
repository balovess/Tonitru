use crate::internal::error::{Error, Result};
use crate::codec::varint; // Use varint for encoding/decoding fields
use blake3; // Used for checksum calculation and verification
use std::mem;
use crate::compress::CompressionStrategy; // Import CompressionStrategy

// Constants for encoding CompressionStrategy in flow_flags
const COMPRESSION_STRATEGY_MASK: u32 = 0b11; // Use the lowest 2 bits for compression strategy
const COMPRESSION_STRATEGY_SHIFT: u32 = 0; // Start from bit 0

/// Represents the metadata header of a Tonitru packet.
#[derive(Debug, PartialEq, Clone)] // Added Clone derive
pub struct MetadataHeader {
    pub schema_id: u64,
    pub timestamp: u64,
    pub shard_id: u64,
    pub flow_flags: u32, // Using u32 for flags
    pub body_type: u8, // Field to indicate the type of DataBody
    // TODO: Add more metadata fields as needed
}

/// Represents the data body of a Tonitru packet.
#[derive(Debug, PartialEq, Clone)] // Added Clone derive
pub enum DataBody {
    Raw(Vec<u8>),
    Compressed(Vec<u8>), // Compressed data
    Encrypted(Vec<u8>), // Encrypted data
    // TODO: Add variants for other data body types (e.g., combined compressed/encrypted)
}

// Helper enum to indicate the type of DataBody being decoded
#[derive(Debug, PartialEq, Clone, Copy)] // Added Copy derive
#[repr(u8)] // Ensure enum variants have a fixed u8 representation
pub enum DataBodyType {
    Raw = 0,
    Compressed = 1,
    Encrypted = 2,
    // TODO: Add variants for other data body types
}

impl DataBodyType {
    /// Converts a u8 value to DataBodyType.
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            0 => Ok(DataBodyType::Raw),
            1 => Ok(DataBodyType::Compressed),
            2 => Ok(DataBodyType::Encrypted),
            _ => Err(Error::CodecError(format!("Unknown DataBodyType value: {}", value))),
        }
    }
}


/// Represents the checksum of a Tonitru packet.
#[derive(Debug, PartialEq, Clone)] // Added Clone derive for completeness, though not strictly needed for the current errors
pub struct Checksum {
    pub blake3_hash: [u8; 32], // BLAKE3 hash (32 bytes)
}

/// Represents a complete Tonitru network packet.
#[derive(Debug, PartialEq, Clone)] // Added Clone derive for completeness
pub struct Packet {
    pub header: MetadataHeader,
    pub body: DataBody,
    pub checksum: Checksum,
}

impl MetadataHeader {
    /// Encodes the MetadataHeader into bytes.
    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut encoded = Vec::new();
        encoded.extend_from_slice(&varint::encode_varint(self.schema_id));
        encoded.extend_from_slice(&varint::encode_varint(self.timestamp));
        encoded.extend_from_slice(&varint::encode_varint(self.shard_id));
        encoded.extend_from_slice(&self.flow_flags.to_le_bytes()); // Fixed size u32 (4 bytes)
        encoded.push(self.body_type); // Encode body_type as a single byte
        // TODO: Encode other metadata fields
        Ok(encoded)
    }

    /// Decodes bytes into a MetadataHeader.
    pub fn decode(data: &[u8]) -> Result<(Self, usize)> {
        let mut bytes_read = 0;

        let (schema_id, len) = varint::decode_varint(data)?;
        bytes_read += len;

        let remaining = &data[bytes_read..];
        let (timestamp, len) = varint::decode_varint(remaining)?;
        bytes_read += len;

        let remaining = &data[bytes_read..];
        let (shard_id, len) = varint::decode_varint(remaining)?;
        bytes_read += len;

        let remaining = &data[bytes_read..];
        if remaining.len() < mem::size_of::<u32>() {
             return Err(Error::CodecError("Incomplete data for flow_flags".to_string()));
         }
        let mut flags_bytes = [0u8; mem::size_of::<u32>()];
        flags_bytes.copy_from_slice(&remaining[..mem::size_of::<u32>()]);
        let flow_flags = u32::from_le_bytes(flags_bytes);
        bytes_read += mem::size_of::<u32>();

        let remaining = &data[bytes_read..];
        if remaining.is_empty() {
             return Err(Error::CodecError("Incomplete data for body_type".to_string()));
         }
        let body_type = remaining[0];
        bytes_read += 1;


        // TODO: Decode other metadata fields

        Ok((MetadataHeader { schema_id, timestamp, shard_id, flow_flags, body_type }, bytes_read))
    }

    /// Sets the compression strategy in flow_flags.
    pub fn set_compression_strategy(&mut self, strategy: CompressionStrategy) {
        // Clear the existing compression bits
        self.flow_flags &= !COMPRESSION_STRATEGY_MASK;
        // Set the new compression bits
        self.flow_flags |= ((strategy as u8) as u32) << COMPRESSION_STRATEGY_SHIFT;
    }

    /// Gets the compression strategy from flow_flags.
    pub fn get_compression_strategy(&self) -> Result<CompressionStrategy> {
        let strategy_bits = (self.flow_flags >> COMPRESSION_STRATEGY_SHIFT) & COMPRESSION_STRATEGY_MASK;
        match strategy_bits as u8 {
            0 => Ok(CompressionStrategy::NoCompression),
            1 => Ok(CompressionStrategy::Zstd),
            // Removed Lz4 case: 2 => Ok(CompressionStrategy::Lz4),
            3 => Ok(CompressionStrategy::Brotli),
            _ => Err(Error::CodecError(format!("Unknown compression strategy bits in flow_flags: {}", strategy_bits))),
        }
    }
}

impl DataBody {
    /// Encodes the DataBody into bytes.
    pub fn encode(&self) -> Result<Vec<u8>> {
        match self {
            DataBody::Raw(data) => Ok(data.clone()), // Simple clone for now
            DataBody::Compressed(data) => Ok(data.clone()),
            DataBody::Encrypted(data) => Ok(data.clone()),
            // TODO: Encode other data body variants
        }
    }

    /// Decodes bytes into a DataBody.
    /// Note: This basic decode assumes the type is known externally or from a header field.
    /// A more complete implementation would need type information.
    pub fn decode(data: &[u8], body_type: DataBodyType) -> Result<Self> {
         match body_type {
            DataBodyType::Raw => Ok(DataBody::Raw(data.to_vec())),
            DataBodyType::Compressed => Ok(DataBody::Compressed(data.to_vec())),
            DataBodyType::Encrypted => Ok(DataBody::Encrypted(data.to_vec())),
            // TODO: Decode other data body variants
         }
    }
}


impl Checksum {
    /// Creates a new Checksum from a BLAKE3 hash.
    pub fn new(blake3_hash: [u8; 32]) -> Self {
        Checksum { blake3_hash }
    }

    /// Encodes the Checksum into bytes.
    pub fn encode(&self) -> Vec<u8> {
        self.blake3_hash.to_vec()
    }

    /// Decodes bytes into a Checksum.
    pub fn decode(data: &[u8]) -> Result<(Self, usize)> {
        if data.len() < 32 {
            return Err(Error::CodecError("Incomplete data for BLAKE3 checksum".to_string()));
        }
        let mut hash_bytes = [0u8; 32];
        hash_bytes.copy_from_slice(&data[..32]);
        Ok((Checksum { blake3_hash: hash_bytes }, 32))
    }

    /// Verifies the checksum against calculated hash.
    pub fn verify(&self, calculated_hash: &[u8; 32]) -> bool {
        &self.blake3_hash == calculated_hash
    }
}

impl Packet {
    /// Builds a new Tonitru packet.
    pub fn build_packet(mut header: MetadataHeader, body: DataBody) -> Result<Self> {
        // Set body type in header based on DataBody variant
        header.body_type = match body {
            DataBody::Raw(_) => DataBodyType::Raw as u8,
            DataBody::Compressed(_) => DataBodyType::Compressed as u8,
            DataBody::Encrypted(_) => DataBodyType::Encrypted as u8,
        };

        let mut hasher = blake3::Hasher::new();
        hasher.update(&header.encode()?);
        hasher.update(&body.encode()?);
        let checksum = Checksum::new(*hasher.finalize().as_bytes());

        Ok(Packet { header, body, checksum })
    }

    /// Parses bytes into a Tonitru packet.
    pub fn parse_packet(data: &[u8]) -> Result<Self> {
        let mut bytes_read = 0;

        // Decode Header
        let (header, header_bytes) = MetadataHeader::decode(data)?;
        bytes_read += header_bytes;

        // Determine body type from header
        let body_type = DataBodyType::from_u8(header.body_type)?;

        // Decode Body
        let remaining_data = &data[bytes_read..];
        let body_length = remaining_data.len().checked_sub(32) // Checksum is the last 32 bytes
            .ok_or_else(|| Error::CodecError("Incomplete data for body and checksum".to_string()))?;

        let body_slice = &remaining_data[..body_length];
        let body = DataBody::decode(body_slice, body_type)?;
        bytes_read += body_length;

        // Decode Checksum
        let remaining_data_after_body = &data[bytes_read..];
        let (_checksum, _checksum_bytes) = Checksum::decode(remaining_data_after_body)?; // Added underscore

        // Verify checksum
        let mut hasher = blake3::Hasher::new();
        hasher.update(&header.encode()?); // Re-encode header to calculate hash
        hasher.update(&body.encode()?);   // Re-encode body to calculate hash
        let calculated_hash = hasher.finalize();

        if !_checksum.verify(calculated_hash.as_bytes()) { // Used _checksum
            return Err(Error::CodecError("Checksum verification failed".to_string()));
        }

        Ok(Packet { header, body, checksum: _checksum }) // Used _checksum
    }
} // Added closing brace for impl Packet

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compress::CompressionStrategy;

    #[test]
    fn test_packet_build_and_parse_raw() {
        let mut header = MetadataHeader {
            schema_id: 1,
            timestamp: 1678886400, // Example timestamp
            shard_id: 10,
            flow_flags: 0b101, // Example flags
            body_type: 0, // Will be set by build_packet
        };
        let body = DataBody::Raw(vec![1, 2, 3, 4, 5]);

        // Set compression strategy in header
        header.set_compression_strategy(CompressionStrategy::NoCompression);

        // Build the packet
        let packet = Packet::build_packet(header.clone(), body.clone()).unwrap();

        // Encode the packet to bytes (Header + Body + Checksum)
        let mut encoded_packet = packet.header.encode().unwrap(); // Use packet.header to get updated flags
        encoded_packet.extend_from_slice(&packet.body.encode().unwrap()); // Use packet.body
        encoded_packet.extend_from_slice(&packet.checksum.encode());

        // Parse the packet from bytes
        let parsed_packet = Packet::parse_packet(&encoded_packet).unwrap();

        // Verify the parsed packet matches the original
        assert_eq!(parsed_packet.header, packet.header);
        assert_eq!(parsed_packet.body, packet.body);
        assert_eq!(parsed_packet.checksum, packet.checksum);

        // Verify compression strategy from parsed header
        assert_eq!(parsed_packet.header.get_compression_strategy().unwrap(), CompressionStrategy::NoCompression);
    }

    #[test]
    fn test_packet_build_and_parse_compressed() {
        let mut header = MetadataHeader {
            schema_id: 2,
            timestamp: 1678886500,
            shard_id: 20,
            flow_flags: 0b110,
            body_type: 0, // Will be set by build_packet
        };
        let body = DataBody::Compressed(vec![6, 7, 8, 9, 10]);

        // Set compression strategy in header
        header.set_compression_strategy(CompressionStrategy::Zstd);


        let packet = Packet::build_packet(header.clone(), body.clone()).unwrap();
        let mut encoded_packet = packet.header.encode().unwrap(); // Use packet.header
        encoded_packet.extend_from_slice(&packet.body.encode().unwrap()); // Use packet.body
        encoded_packet.extend_from_slice(&packet.checksum.encode());

        let parsed_packet = Packet::parse_packet(&encoded_packet).unwrap();

        assert_eq!(parsed_packet.header, packet.header);
        assert_eq!(parsed_packet.body, packet.body);
        assert_eq!(parsed_packet.checksum, packet.checksum);

        // Verify compression strategy from parsed header
        assert_eq!(parsed_packet.header.get_compression_strategy().unwrap(), CompressionStrategy::Zstd);
    }

    #[test]
    fn test_packet_build_and_parse_encrypted() {
        let mut header = MetadataHeader {
            schema_id: 3,
            timestamp: 1678886600,
            shard_id: 30,
            flow_flags: 0b111,
            body_type: 0, // Will be set by build_packet
        };
        let body = DataBody::Encrypted(vec![11, 12, 13, 14, 15]);

        // Set compression strategy in header
        header.set_compression_strategy(CompressionStrategy::Brotli);


        let packet = Packet::build_packet(header.clone(), body.clone()).unwrap();
        let mut encoded_packet = packet.header.encode().unwrap(); // Use packet.header
        encoded_packet.extend_from_slice(&packet.body.encode().unwrap()); // Use packet.body
        encoded_packet.extend_from_slice(&packet.checksum.encode());

        let parsed_packet = Packet::parse_packet(&encoded_packet).unwrap();

        assert_eq!(parsed_packet.header, packet.header);
        assert_eq!(parsed_packet.body, packet.body);
        assert_eq!(parsed_packet.checksum, packet.checksum);

        // Verify compression strategy from parsed header
        assert_eq!(parsed_packet.header.get_compression_strategy().unwrap(), CompressionStrategy::Brotli);
    }


    #[test]
    fn test_packet_parse_checksum_fail() {
         let header = MetadataHeader { // Removed mut
            schema_id: 1,
            timestamp: 1678886400,
            shard_id: 10,
            flow_flags: 0b101,
            body_type: 0, // Will be set by build_packet
        };
        let body = DataBody::Raw(vec![1, 2, 3, 4, 5]);

        // Removed Lz4 compression strategy setting: header.set_compression_strategy(CompressionStrategy::Lz4);

        // Build a valid packet
        let packet = Packet::build_packet(header.clone(), body.clone()).unwrap();

        // Encode the packet to bytes
        let mut encoded_packet = packet.header.encode().unwrap(); // Use packet.header
        encoded_packet.extend_from_slice(&packet.body.encode().unwrap()); // Use packet.body
        encoded_packet.extend_from_slice(&packet.checksum.encode());

        // Tamper with the body bytes
        let tampered_index = encoded_packet.len() - 32 - 1; // Tamper the last byte of the body
        encoded_packet[tampered_index] = encoded_packet[tampered_index].wrapping_add(1);

        // Attempt to parse the tampered packet (should fail checksum verification)
        let parse_result = Packet::parse_packet(&encoded_packet);
        assert!(parse_result.is_err());
        assert_eq!(parse_result.unwrap_err().to_string(), "Codec Error: Checksum verification failed"); // Updated assertion
    }

     #[test]
    fn test_packet_parse_incomplete_data() {
        let mut header = MetadataHeader {
            schema_id: 1,
            timestamp: 1678886400,
            shard_id: 10,
            flow_flags: 0b101,
            body_type: 0, // Will be set by build_packet
        };
        let body = DataBody::Raw(vec![1, 2, 3, 4, 5]);

        // Set compression strategy in header
        header.set_compression_strategy(CompressionStrategy::NoCompression);

        // Build a valid packet
        let packet = Packet::build_packet(header.clone(), body.clone()).unwrap();

        // Encode the packet to bytes
        let mut encoded_packet = packet.header.encode().unwrap(); // Use packet.header
        encoded_packet.extend_from_slice(&packet.body.encode().unwrap()); // Use packet.body
        encoded_packet.extend_from_slice(&packet.checksum.encode());

        // Truncate the encoded packet
        let truncated_packet = &encoded_packet[..encoded_packet.len() - 10]; // Remove last 10 bytes

        // Attempt to parse the truncated packet (should fail due to incomplete data)
        let parse_result = Packet::parse_packet(truncated_packet);
        assert!(parse_result.is_err());
        // The specific error message might vary depending on where the parsing fails first
        // (incomplete checksum, incomplete body, etc.), but it should be a CodecError.
        assert!(parse_result.unwrap_err().to_string().contains("Incomplete data"));
    }

    #[test]
    fn test_packet_parse_unknown_body_type() {
        let mut header = MetadataHeader {
            schema_id: 1,
            timestamp: 1678886400,
            shard_id: 10,
            flow_flags: 0b101,
            body_type: 99, // An unknown body type
        };
        let body = DataBody::Raw(vec![1, 2, 3, 4, 5]);

        // Set compression strategy in header (doesn't matter for this test, but good practice)
        header.set_compression_strategy(CompressionStrategy::NoCompression);

        let packet = Packet::build_packet(header.clone(), body.clone()).unwrap();
        let mut encoded_packet = packet.header.encode().unwrap(); // Use packet.header
        encoded_packet.extend_from_slice(&packet.body.encode().unwrap()); // Use packet.body
        encoded_packet.extend_from_slice(&packet.checksum.encode());

        // Manually set body_type in the encoded packet to the unknown value
        // Find the position of body_type in the encoded header
        let header_bytes = packet.header.encode().unwrap();
        let body_type_pos = header_bytes.len() - 1;
        encoded_packet[body_type_pos] = 99;


        let parse_result = Packet::parse_packet(&encoded_packet);
        assert!(parse_result.is_err());
        assert!(parse_result.unwrap_err().to_string().contains("Unknown DataBodyType value: 99"));
    }

    #[test]
    fn test_metadata_header_compression_flags() {
        let mut header = MetadataHeader {
            schema_id: 1,
            timestamp: 123,
            shard_id: 456,
            flow_flags: 0, // Start with no flags
            body_type: 0,
        };

        // Test setting and getting NoCompression
        header.set_compression_strategy(CompressionStrategy::NoCompression);
        assert_eq!(header.get_compression_strategy().unwrap(), CompressionStrategy::NoCompression);
        assert_eq!(header.flow_flags & COMPRESSION_STRATEGY_MASK, 0);

        // Test setting and getting Zstd
        header.set_compression_strategy(CompressionStrategy::Zstd);
        assert_eq!(header.get_compression_strategy().unwrap(), CompressionStrategy::Zstd);
        assert_eq!(header.flow_flags & COMPRESSION_STRATEGY_MASK, 1);

        // Removed Lz4 test:
        // #[test]
        // fn test_setting_and_getting_lz4() {
        //     let mut header = MetadataHeader {
        //         schema_id: 1,
        //         timestamp: 123,
        //         shard_id: 456,
        //         flow_flags: 0, // Start with no flags
        //         body_type: 0,
        //     };
        //     header.set_compression_strategy(CompressionStrategy::Lz4);
        //     assert_eq!(header.get_compression_strategy().unwrap(), CompressionStrategy::Lz4);
        //     assert_eq!(header.flow_flags & COMPRESSION_STRATEGY_MASK, 2);
        // }


        // Test setting and getting Brotli
        header.set_compression_strategy(CompressionStrategy::Brotli);
        assert_eq!(header.get_compression_strategy().unwrap(), CompressionStrategy::Brotli);
        assert_eq!(header.flow_flags & COMPRESSION_STRATEGY_MASK, 3);

        // Test setting compression flags while other flags are present
        let mut header_with_other_flags = MetadataHeader {
            schema_id: 1,
            timestamp: 123,
            shard_id: 456,
            flow_flags: 0b1111_1100, // Some other flags set
            body_type: 0,
        };
        header_with_other_flags.set_compression_strategy(CompressionStrategy::Zstd);
        assert_eq!(header_with_other_flags.get_compression_strategy().unwrap(), CompressionStrategy::Zstd);
        assert_eq!(header_with_other_flags.flow_flags & COMPRESSION_STRATEGY_MASK, 1);
        assert_eq!(header_with_other_flags.flow_flags & 0b1111_1100, 0b1111_1100); // Other flags should be preserved
    }
}