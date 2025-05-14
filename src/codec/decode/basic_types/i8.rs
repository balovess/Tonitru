use crate::codec::types::HtlvValue;
use crate::internal::error::{Error, Result};
use std::mem;
use crate::codec::decode::batch::BatchDecoder; // Import BatchDecoder trait
use std::slice; // Import slice for unsafe reinterpretation

/// Decodes an I8 HtlvValue from bytes.
pub fn decode_i8(length: u64, raw_value_slice: &[u8]) -> Result<HtlvValue> {
    if length as usize != mem::size_of::<i8>() {
        return Err(Error::CodecError(format!(
            "Invalid length for I8 value: {}",
            length
        )));
    }
    if raw_value_slice.len() < mem::size_of::<i8>() {
         return Err(Error::CodecError("Incomplete data for I8 value".to_string()));
    }
    Ok(HtlvValue::I8(raw_value_slice[0] as i8)) // Assuming two's complement
}

// Note: The previous `decode_i8_batch` function is now replaced by the `BatchDecoder` implementation below.

impl BatchDecoder for i8 {
    type DecodedType = i8;

    /// Decodes a batch of I8 values from bytes using zero-copy reinterpretation.
    /// Returns a slice of the decoded elements and the number of bytes read.
    fn decode_batch(data: &[u8]) -> Result<(&[Self::DecodedType], usize)> {
        let size = mem::size_of::<i8>();
        if data.len() % size != 0 {
            return Err(Error::CodecError(format!(
                "Invalid data length for I8 batch decoding. Length ({}) must be a multiple of {}",
                data.len(),
                size
            )));
        }

        let count = data.len() / size;
        let decoded_slice = unsafe {
            // Safety:
            // 1. The data slice is guaranteed to be valid for reads for `data.len()` bytes.
            // 2. We check that `data.len()` is a multiple of `size_of::<i8>()`,
            //    ensuring the total size is correct for `count` i8 elements.
            // 3. Reinterpreting &[u8] as &[i8] is safe because i8 has the same size and alignment as u8,
            //    and the bit patterns are interpreted correctly for signed integers in two's complement.
            slice::from_raw_parts(data.as_ptr() as *const i8, count)
        };

        Ok((decoded_slice, data.len()))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::encode::encode_item;
    use crate::codec::types::HtlvItem;

    #[test]
    fn test_decode_i8() {
        let encoded_i8 = encode_item(&HtlvItem::new(0, HtlvValue::I8(-128))).unwrap();
        // Assuming encode_item for I8 results in [Tag(varint), Type(u8), Length(varint), Value(i8)]
        // For tag 0 (1 byte), type I8 (1 byte), length 1 (1 byte), the header is 3 bytes.
        let raw_value_slice_i8 = &encoded_i8[3..]; // Length 1
        let decoded_i8 = decode_i8(1, raw_value_slice_i8).unwrap();
        assert_eq!(decoded_i8, HtlvValue::I8(-128));

        // Test with incomplete data
        let result = decode_i8(1, &[]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Incomplete data for I8 value"
        );
    }

    #[test]
    fn test_decode_i8_errors() {
        // Invalid length for I8 (expected 1)
        let result = decode_i8(0, &[0x00]); // Provide some data but wrong length
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Invalid length for I8 value: 0"
        );
    }

    #[test]
    fn test_decode_batch_i8() {
        // Test decoding a batch of I8 values
        let data: &[u8] = &[
            0xFF, // -1
            0xFE, // -2
            0x01, // 1
            0x02, // 2
            0x00, // 0
        ];
        let expected: &[i8] = &[-1, -2, 1, 2, 0];
        let (decoded_slice, bytes_consumed) = i8::decode_batch(data).unwrap();
        assert_eq!(decoded_slice, expected);
        assert_eq!(bytes_consumed, data.len());

        // Test with empty data
        let data: &[u8] = &[];
        let expected: &[i8] = &[];
        let (decoded_slice, bytes_consumed) = i8::decode_batch(data).unwrap();
        assert_eq!(decoded_slice, expected);
        assert_eq!(bytes_consumed, 0);

         // Test with incomplete data (not a multiple of size_of::<i8>())
        let incomplete_data: &[u8] = &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06]; // 6 bytes, expecting 5 i8 values (5 bytes)
        let result = i8::decode_batch(&incomplete_data);
        assert!(result.is_ok()); // i8 batch decoding is zero-copy, so it will return the slice
        let (decoded_slice, bytes_consumed) = result.unwrap();
        assert_eq!(decoded_slice, &[0x01 as i8, 0x02 as i8, 0x03 as i8, 0x04 as i8, 0x05 as i8, 0x06 as i8]);
        assert_eq!(bytes_consumed, 6); // Consumed all 6 bytes

        // Note: The BatchDecoder for i8 currently doesn't enforce the count from the Array header.
        // This will be handled in the higher-level Array decoding logic.
    }
}