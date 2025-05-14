// Codec module for Tonitru network native data format (HyperNova)

pub mod encode;
pub mod decode;
pub mod rcu;
pub mod varint;
pub mod types;

use crate::internal::error::Result;
use bytes::BytesMut;

/// Trait for types that can be encoded into HTLV format.
///
/// Types implementing this trait can be serialized into a byte stream
/// according to the HTLV specification.
pub trait Encode {
    /// Encodes the type into a mutable BytesMut buffer.
    ///
    /// The encoded data is appended to the provided buffer.
    ///
    /// # Arguments
    ///
    /// * `buf` - A mutable reference to the BytesMut buffer to write the encoded data to.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on successful encoding, or an `Error` if encoding fails.
    fn encode(&self, buf: &mut BytesMut) -> Result<()>;
}

/// Trait for types that can be decoded from HTLV format.
///
/// Types implementing this trait can be deserialized from a byte stream
/// according to the HTLV specification.
pub trait Decode
where
    Self: Sized,
{
    /// Decodes the type from a byte slice.
    ///
    /// Reads data from the beginning of the provided byte slice and attempts
    /// to deserialize it into an instance of the implementing type.
    ///
    /// # Arguments
    ///
    /// * `data` - The byte slice containing the encoded data.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a tuple of the decoded value and the
    /// number of bytes consumed from the input slice on success, or an
    /// `Error` if decoding fails.
    fn decode(data: &[u8]) -> Result<(Self, usize)>;
}