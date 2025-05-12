use crate::codec::types::HtlvValue;
use crate::internal::error::{Error, Result};
use std::mem;

/// Decodes a U8 HtlvValue from bytes.
pub fn decode_u8(length: u64, raw_value_slice: &[u8]) -> Result<HtlvValue> {
    if length != 1 {
        return Err(Error::CodecError(format!(
            "Invalid length for U8 value: {}",
            length
        )));
    }
    Ok(HtlvValue::U8(raw_value_slice[0]))
}

/// Decodes a batch of U8 values from bytes.
pub fn decode_u8_batch(raw_value_slice: &[u8], count: usize) -> Result<Vec<u8>> {
    let required_len = count * mem::size_of::<u8>();
    if raw_value_slice.len() < required_len {
        return Err(Error::CodecError(format!(
            "Incomplete data for U8 batch decoding. Expected at least {} bytes, got {}",
            required_len,
            raw_value_slice.len()
        )));
    }

    // U8 batch decoding is straightforward byte copying
    let result = raw_value_slice[..required_len].to_vec();
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::encode::encode_item;
    use crate::codec::types::HtlvItem;

    #[test]
    fn test_decode_u8() {
        let encoded_u8 = encode_item(&HtlvItem::new(0, HtlvValue::U8(255))).unwrap();
        let raw_value_slice_u8 = &encoded_u8[encoded_u8.len().checked_sub(1).unwrap()..]; // Length 1
        let decoded_u8 = decode_u8(1, raw_value_slice_u8).unwrap();
        assert_eq!(decoded_u8, HtlvValue::U8(255));
    }

    #[test]
    fn test_decode_u8_errors() {
        // Invalid length for U8 (expected 1)
        let result = decode_u8(0, &[]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Invalid length for U8 value: 0"
        );
    }

    #[test]
    fn test_decode_u8_batch() {
        // Test decoding a batch of U8 values
        let data: Vec<u8> = vec![1, 2, 3, 4, 5];
        let expected = vec![1, 2, 3, 4, 5];
        let decoded = decode_u8_batch(&data, 5).unwrap();
        assert_eq!(decoded, expected);

        // Test with incomplete data
        let incomplete_data: Vec<u8> = vec![1, 2, 3];
        let result = decode_u8_batch(&incomplete_data, 5);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Incomplete data for U8 batch decoding. Expected at least 5 bytes, got 3"
        );

        // Test with empty data
        let result = decode_u8_batch(&[], 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![] as Vec<u8>);

         // Test with empty data and count > 0
        let result = decode_u8_batch(&[], 1);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Incomplete data for U8 batch decoding. Expected at least 1 bytes, got 0"
        );
    }
}