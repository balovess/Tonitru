use crate::internal::error::Result;

/// Trait for types that can be decoded in batches from a byte slice.
///
/// This trait is intended for basic numerical types within Arrays to enable
/// efficient decoding using techniques like SIMD and pipelining.
pub trait BatchDecoder {
    /// The type of the decoded elements.
    type DecodedType;

    /// Decodes a batch of elements from the beginning of the provided byte slice.
    ///
    /// Returns a `Result` containing a tuple of:
    /// 1. A slice of the decoded elements (`&[Self::DecodedType]`). This should ideally be
    ///    a zero-copy view into the input `data` slice if possible.
    /// 2. The number of bytes consumed from the input `data` slice.
    ///
    /// # Arguments
    ///
    /// * `data` - The byte slice containing the encoded batch data.
    ///
    /// # Returns
    ///
    /// Returns `Ok((decoded_slice, bytes_consumed))` on successful decoding,
    /// or an `Error` if decoding fails.
    fn decode_batch(data: &[u8]) -> Result<(&[Self::DecodedType], usize)>;
}