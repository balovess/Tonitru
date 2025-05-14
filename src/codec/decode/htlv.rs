use bytes::Buf;
use byteorder::{BigEndian, ReadBytesExt};

use crate::codec::{types::{HTLVBlock, HTLVFlag}, Decode, varint::decode_varint};
use crate::internal::error::{Result, Error};

/// Implements the `Decode` trait for `HTLVBlock`.
///
/// This implementation decodes a byte stream into an `HTLVBlock` according to
/// the HTLV header format: Tag (u16), Flags (u8), Length (VLQ u64), and Value.
/// If the `NESTED` flag is set, the Value field is recursively decoded into
/// nested `HTLVBlock`s.
impl Decode for HTLVBlock {
    fn decode(data: &[u8]) -> Result<(Self, usize)> {
        let mut reader = data;
        let original_len = reader.len();

        // 1. Read Tag (u16)
        if reader.len() < 2 {
            return Err(Error::CodecError("Incomplete data for HTLV Tag".to_string()));
        }
        let tag = reader.read_u16::<BigEndian>()?;

        // 2. Read Flags (u8)
        if reader.is_empty() {
            return Err(Error::CodecError("Incomplete data for HTLV Flags".to_string()));
        }
        let flags_byte = reader.read_u8()?;
        let flags = HTLVFlag::from_bits(flags_byte)
            .ok_or_else(|| Error::CodecError(format!("Invalid HTLV flags byte: {}", flags_byte)))?;

        // 3. Read Length (u64) using VLQ
        let (length, varint_bytes_read) = decode_varint(reader)?;
        reader = &reader[varint_bytes_read..];

        // 4. Read Value ([]byte)
        if reader.len() < length as usize {
            return Err(Error::CodecError(format!("Incomplete data for HTLV Value. Expected {} bytes, got {}", length, reader.len())));
        }
        let value = reader[..length as usize].to_vec();
        reader = &reader[length as usize..];

        // 5. If Nested flag is set, recursively decode nested blocks
        let mut nested_blocks = Vec::new();
        if flags.contains(HTLVFlag::NESTED) {
            let mut nested_reader = value.as_slice();
            while !nested_reader.is_empty() {
                let (nested_block, nested_bytes_read) = HTLVBlock::decode(nested_reader)?;
                nested_blocks.push(nested_block);
                nested_reader = &nested_reader[nested_bytes_read..];
            }
        }

        let bytes_read = original_len - reader.len();

        Ok((
            HTLVBlock {
                tag,
                flags,
                length,
                value,
                nested: nested_blocks,
            },
            bytes_read,
        ))
    }
}

// TODO: Add unit tests for HTLVBlock decoding