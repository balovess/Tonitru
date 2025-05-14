use bytes::{BytesMut, BufMut};
use byteorder::{BigEndian, WriteBytesExt};

use crate::codec::{types::HTLVBlock, Encode, varint::encode_varint};
use crate::internal::error::Result;

/// Implements the `Encode` trait for `HTLVBlock`.
///
/// This implementation encodes an `HTLVBlock` into a byte stream according to
/// the HTLV header format: Tag (u16), Flags (u8), Length (VLQ u64), and Value.
/// If the `NESTED` flag is set, the Value field contains the encoded bytes
/// of the nested `HTLVBlock`s.
impl Encode for HTLVBlock {
    fn encode(&self, buf: &mut BytesMut) -> Result<()> {
        // 1. Encode Tag (u16)
        buf.put_u16(self.tag);

        // 2. Encode Flags (u8)
        buf.put_u8(self.flags.bits());

        // 3. Encode Length (u64) using VLQ
        let length_bytes = encode_varint(self.length);
        buf.extend_from_slice(&length_bytes);

        // 4. Encode Value ([]byte) or Nested HTLV blocks
        if self.flags.contains(crate::codec::types::HTLVFlag::NESTED) {
            // If Nested flag is set, encode the nested blocks
            for block in &self.nested {
                block.encode(buf)?;
            }
        } else {
            // Otherwise, encode the raw value bytes
            buf.extend_from_slice(&self.value);
        }

        Ok(())
    }
}

// TODO: Add unit tests for HTLVBlock encoding