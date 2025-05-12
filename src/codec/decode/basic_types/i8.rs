use crate::codec::types::HtlvValue;
use crate::internal::error::{Error, Result};
use std::mem;

/// Decodes an I8 HtlvValue from bytes.
pub fn decode_i8(length: u64, raw_value_slice: &[u8]) -> Result<HtlvValue> {
    if length != 1 {
        return Err(Error::CodecError(format!(
            "Invalid length for I8 value: {}",
            length
        )));
    }
    Ok(HtlvValue::I8(raw_value_slice[0] as i8)) // Assuming two's complement
}

/// Decodes a batch of I8 values from bytes.
pub fn decode_i8_batch(raw_value_slice: &[u8], count: usize) -> Result<Vec<i8>> {
    let required_len = count * mem::size_of::<i8>();
    if raw_value_slice.len() < required_len {
        return Err(Error::CodecError(format!(
            "Incomplete data for I8 batch decoding. Expected at least {} bytes, got {}",
            required_len,
            raw_value_slice.len()
        )));
    }

    // I8 batch decoding is straightforward byte copying and casting
    let result: Vec<i8> = raw_value_slice[..required_len].iter().map(|&b| b as i8).collect();
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::encode::encode_item;
    use crate::codec::types::HtlvItem;

    #[test]
    fn test_decode_i8() {
        let encoded_i8 = encode_item(&HtlvItem::new(0, HtlvValue::I8(-128))).unwrap();
        let raw_value_slice_i8 = &encoded_i8[encoded_i8.len().checked_sub(1).unwrap()..]; // Length 1
        let decoded_i8 = decode_i8(1, raw_value_slice_i8).unwrap();
        assert_eq!(decoded_i8, HtlvValue::I8(-128));
    }

    #[test]
    fn test_decode_i8_errors() {
        // Invalid length for I8 (expected 1)
        let result = decode_i8(0, &[]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Invalid length for I8 value: 0"
        );
    }

    #[test]
    fn test_decode_i8_batch() {
        // Test decoding a batch of I8 values
        let data: Vec<u8> = vec![
            0xFF, // -1
            0xFE, // -2
            0x01, // 1
            0x02, // 2
            0x00, // 0
        ];
        let expected = vec![-1, -2, 1, 2, 0];
        let decoded = decode_i8_batch(&data, 5).unwrap();
        assert_eq!(decoded, expected);

        // Test with incomplete data
        let incomplete_data: Vec<u8> = vec![
            0xFF, // -1
        ];
        let result = decode_i8_batch(&incomplete_data, 2); // Expecting 2 i8 values, but only 1 byte provided
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Incomplete data for I8 batch decoding. Expected at least 2 bytes, got 1"
        );

        // Test with empty data
        let result = decode_i8_batch(&[], 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![] as Vec<i8>);

         // Test with empty data and count > 0
        let result = decode_i8_batch(&[], 1);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Incomplete data for I8 batch decoding. Expected at least 1 bytes, got 0"
        );
    }
}