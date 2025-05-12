use crate::codec::types::HtlvValue;
use crate::internal::error::{Error, Result};
use bytes::Bytes;

/// Decodes a Bytes HtlvValue from bytes.
pub fn decode_bytes(raw_value_slice: &[u8]) -> Result<HtlvValue> {
    Ok(HtlvValue::Bytes(Bytes::copy_from_slice(raw_value_slice)))
}

/// Decodes a String HtlvValue from bytes.
pub fn decode_string(raw_value_slice: &[u8]) -> Result<HtlvValue> {
    // Validate UTF-8 but keep the Bytes slice for zero-copy
    std::str::from_utf8(raw_value_slice)
        .map_err(|e| Error::CodecError(format!("Invalid UTF-8 string: {}", e)))?;
    Ok(HtlvValue::String(Bytes::copy_from_slice(raw_value_slice)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::encode::encode_item;
    use crate::codec::types::HtlvItem;
    use bytes::Bytes;

    #[test]
    fn test_decode_bytes() {
        // Test Bytes
        let encoded_bytes = encode_item(&HtlvItem::new(
            0,
            HtlvValue::Bytes(Bytes::from_static(b"raw data")),
        ))
        .unwrap();
        let raw_value_slice_bytes = &encoded_bytes[encoded_bytes.len().checked_sub(8).unwrap()..]; // Length 8
        let decoded_bytes = decode_bytes(raw_value_slice_bytes).unwrap();
        assert_eq!(
            decoded_bytes,
            HtlvValue::Bytes(Bytes::from_static(b"raw data"))
        );
    }

    #[test]
    fn test_decode_string() {
        // Test String
        let encoded_string = encode_item(&HtlvItem::new(
            0,
            HtlvValue::String(Bytes::from_static("你好".as_bytes())),
        ))
        .unwrap();
        let raw_value_slice_string =
            &encoded_string[encoded_string.len().checked_sub(6).unwrap()..]; // Length 6
        let decoded_string = decode_string(raw_value_slice_string).unwrap();
        assert_eq!(
            decoded_string,
            HtlvValue::String(Bytes::from_static("你好".as_bytes()))
        );
    }

    #[test]
    fn test_decode_string_errors() {
        // Invalid UTF-8 string
        let invalid_utf8 = vec![0xff, 0xff];
        let result = decode_string(&invalid_utf8);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid UTF-8 string"));
    }
}