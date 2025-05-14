use crate::codec::types::HtlvValue;
use crate::internal::error::{Error, Result};
// std::mem import removed as it's not used
use crate::codec::decode::batch::BatchDecoder; // Import BatchDecoder trait

/// Decodes a U8 HtlvValue from bytes.
pub fn decode_u8(length: u64, raw_value_slice: &[u8]) -> Result<HtlvValue> {
    if length != 1 {
        return Err(Error::CodecError(format!(
            "Invalid length for U8 value: {}",
            length
        )));
    }
    if raw_value_slice.is_empty() {
         return Err(Error::CodecError("Incomplete data for U8 value".to_string()));
    }
    Ok(HtlvValue::U8(raw_value_slice[0]))
}

// Note: The previous `decode_u8_batch` function is now replaced by the `BatchDecoder` implementation below.

impl BatchDecoder for u8 {
    type DecodedType = u8;

    /// Decodes a batch of U8 values from bytes.
    /// Returns a slice of the decoded elements and the number of bytes read.
    fn decode_batch(data: &[u8]) -> Result<(&[Self::DecodedType], usize)> {
        // For u8, batch decoding is a zero-copy operation.
        // The number of bytes read is equal to the number of elements.
        let bytes_consumed = data.len();
        Ok((data, bytes_consumed))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::encode::encode_item;
    use crate::codec::types::HtlvItem;

    #[test]
    fn test_decode_u8() {
        let encoded_u8 = encode_item(&HtlvItem::new(0, HtlvValue::U8(255))).unwrap();
        // Assuming encode_item for U8 results in [Tag(varint), Type(u8), Length(varint), Value(u8)]
        // We need to find the start of the Value slice.
        // For tag 0 (1 byte), type U8 (1 byte), length 1 (1 byte), the header is 3 bytes.
        let raw_value_slice_u8 = &encoded_u8[3..]; // Length 1
        let decoded_u8 = decode_u8(1, raw_value_slice_u8).unwrap();
        assert_eq!(decoded_u8, HtlvValue::U8(255));

        // Test with incomplete data
        let result = decode_u8(1, &[]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Incomplete data for U8 value"
        );
    }

    #[test]
    fn test_decode_u8_errors() {
        // Invalid length for U8 (expected 1)
        let result = decode_u8(0, &[10]); // Provide some data to avoid incomplete data error first
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Invalid length for U8 value: 0"
        );
    }

    #[test]
    fn test_decode_batch_u8() {
        // Test decoding a batch of U8 values
        let data: &[u8] = &[1, 2, 3, 4, 5];
        let expected_slice: &[u8] = &[1, 2, 3, 4, 5];
        let (decoded_slice, bytes_consumed) = u8::decode_batch(data).unwrap();
        assert_eq!(decoded_slice, expected_slice);
        assert_eq!(bytes_consumed, 5);

        // Test with empty data
        let data: &[u8] = &[];
        let expected_slice: &[u8] = &[];
        let (decoded_slice, bytes_consumed) = u8::decode_batch(data).unwrap();
        assert_eq!(decoded_slice, expected_slice);
        assert_eq!(bytes_consumed, 0);
    }
}