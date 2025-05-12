use crate::codec::types::HtlvValue;
use crate::internal::error::{Error, Result};

/// Decodes a Boolean HtlvValue from bytes.
pub fn decode_bool(length: u64, raw_value_slice: &[u8]) -> Result<HtlvValue> {
    if length != 1 {
        return Err(Error::CodecError(format!(
            "Invalid length for Bool value: {}",
            length
        )));
    }
    Ok(HtlvValue::Bool(raw_value_slice[0] != 0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::encode::encode_item;
    use crate::codec::types::HtlvItem;
    // Removed unused import: use bytes::Bytes;

    #[test]
    fn test_decode_bool() {
        // Test Bool(true)
        let encoded_bool = encode_item(&HtlvItem::new(0, HtlvValue::Bool(true))).unwrap();
        let raw_value_slice_bool = &encoded_bool[encoded_bool.len().checked_sub(1).unwrap()..]; // Length 1
        let decoded_bool = decode_bool(1, raw_value_slice_bool).unwrap();
        assert_eq!(decoded_bool, HtlvValue::Bool(true));

        // Test Bool(false)
        let encoded_bool_false = encode_item(&HtlvItem::new(0, HtlvValue::Bool(false))).unwrap();
        let raw_value_slice_bool_false = &encoded_bool_false[encoded_bool_false.len().checked_sub(1).unwrap()..]; // Length 1
        let decoded_bool_false = decode_bool(1, raw_value_slice_bool_false).unwrap();
        assert_eq!(decoded_bool_false, HtlvValue::Bool(false));
    }

    #[test]
    fn test_decode_bool_errors() {
        // Invalid length for Bool (expected 1)
        let result = decode_bool(0, &[]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Invalid length for Bool value: 0"
        );
    }
}