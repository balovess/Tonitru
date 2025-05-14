// basic_value_decoder.rs
//
// This module contains functions for decoding basic HTLV values.

use crate::codec::types::{HtlvValue, HtlvValueType};
use crate::internal::error::{Error, Result};
// Removed unused import: use crate::codec::types::HtlvItem; // Import HtlvItem for tests

/// Decodes a basic HTLV value from a byte slice.
///
/// This function handles the decoding of single basic types (excluding batch decodable types).
///
/// Arguments:
/// * `value_type`: The `HtlvValueType` of the value to decode.
/// * `length`: The length of the value data in bytes.
/// * `data`: The byte slice containing the value data.
///
/// Returns:
/// A `Result` containing the decoded `HtlvValue` or an `Error` if decoding fails.
pub fn decode_basic_value(value_type: HtlvValueType, length: u64, data: &[u8]) -> Result<HtlvValue> {
    match value_type {
        HtlvValueType::Null => {
            if length != 0 {
                return Err(Error::CodecError(format!("Invalid length for Null value: {}", length)));
            }
            Ok(HtlvValue::Null)
        }
        HtlvValueType::Bool => {
            if length != 1 {
                return Err(Error::CodecError(format!("Invalid length for Bool value: {}", length)));
            }
            if data.is_empty() {
                return Err(Error::CodecError("Incomplete data for Bool value".to_string()));
            }
            Ok(HtlvValue::Bool(data[0] != 0))
        }
        HtlvValueType::U8 => {
            if length != 1 {
                return Err(Error::CodecError(format!("Invalid length for U8 value: {}", length)));
            }
            if data.is_empty() {
                return Err(Error::CodecError("Incomplete data for U8 value".to_string()));
            }
            Ok(HtlvValue::U8(data[0]))
        }
        HtlvValueType::I8 => {
            if length != 1 {
                return Err(Error::CodecError(format!("Invalid length for I8 value: {}", length)));
            }
            if data.is_empty() {
                return Err(Error::CodecError("Incomplete data for I8 value".to_string()));
            }
            Ok(HtlvValue::I8(data[0] as i8))
        }
        HtlvValueType::Bytes => {
            // Bytes type can have any length
            Ok(HtlvValue::Bytes(bytes::Bytes::copy_from_slice(data)))
        }
        HtlvValueType::String => {
            // String type can have any length
            let s = String::from_utf8(data.to_vec())
                .map_err(|e| Error::CodecError(format!("Invalid UTF-8 sequence for String value: {}", e)))?;
            Ok(HtlvValue::String(bytes::Bytes::from(s)))
        }
        // Batch decodable types are handled in batch_value_decoder
        HtlvValueType::U16 | HtlvValueType::U32 | HtlvValueType::U64 |
        HtlvValueType::I16 | HtlvValueType::I32 | HtlvValueType::I64 |
        HtlvValueType::F32 | HtlvValueType::F64 => {
             Err(Error::CodecError(format!("Batch decodable type {:?} should be handled by batch_value_decoder", value_type)))
        }
        // Complex types are handled elsewhere
        HtlvValueType::Array | HtlvValueType::Object => {
            Err(Error::CodecError(format!("Complex type {:?} should be handled by complex_value_handler", value_type)))
        }
        // Note: Large field types are handled by large_field_handler.rs
        // They use the same HtlvValueType (Bytes/String) but are processed differently
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::types::HtlvValue;
    use bytes::Bytes;

    #[test]
    fn test_decode_basic_value() {
        // Test Null
        assert_eq!(decode_basic_value(HtlvValueType::Null, 0, &[]).unwrap(), HtlvValue::Null);

        // Test Bool
        assert_eq!(decode_basic_value(HtlvValueType::Bool, 1, &[0x01]).unwrap(), HtlvValue::Bool(true));
        assert_eq!(decode_basic_value(HtlvValueType::Bool, 1, &[0x00]).unwrap(), HtlvValue::Bool(false));

        // Test U8
        assert_eq!(decode_basic_value(HtlvValueType::U8, 1, &[0x42]).unwrap(), HtlvValue::U8(66));

        // Test I8
        assert_eq!(decode_basic_value(HtlvValueType::I8, 1, &[0x42]).unwrap(), HtlvValue::I8(66));
        assert_eq!(decode_basic_value(HtlvValueType::I8, 1, &[0x80]).unwrap(), HtlvValue::I8(-128)); // -128 in two's complement

        // Test Bytes
        let bytes_data = Bytes::from_static(b"hello");
        assert_eq!(decode_basic_value(HtlvValueType::Bytes, bytes_data.len() as u64, &bytes_data).unwrap(), HtlvValue::Bytes(bytes_data));

        // Test String
        let string_data = Bytes::from_static(b"world");
        assert_eq!(decode_basic_value(HtlvValueType::String, string_data.len() as u64, &string_data).unwrap(), HtlvValue::String(string_data));
    }

    #[test]
    fn test_decode_basic_value_errors() {
        // Test Null with incorrect length
        let result = decode_basic_value(HtlvValueType::Null, 1, &[0x00]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Codec Error: Invalid length for Null value: 1"); // Corrected expected error message

        // Test Bool with incorrect length
        let result = decode_basic_value(HtlvValueType::Bool, 2, &[0x01, 0x00]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Codec Error: Invalid length for Bool value: 2");

        // Test Bool with incomplete data
        let result = decode_basic_value(HtlvValueType::Bool, 1, &[]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Codec Error: Incomplete data for Bool value");

        // Test U8 with incorrect length
        let result = decode_basic_value(HtlvValueType::U8, 2, &[0x42, 0x00]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Codec Error: Invalid length for U8 value: 2");

        // Test U8 with incomplete data
        let result = decode_basic_value(HtlvValueType::U8, 1, &[]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Codec Error: Incomplete data for U8 value");

        // Test I8 with incorrect length
        let result = decode_basic_value(HtlvValueType::I8, 2, &[0x42, 0x00]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Codec Error: Invalid length for I8 value: 2");

        // Test I8 with incomplete data
        let result = decode_basic_value(HtlvValueType::I8, 1, &[]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Codec Error: Incomplete data for I8 value");

        // Test String with invalid UTF-8
        let result = decode_basic_value(HtlvValueType::String, 2, &[0xFF, 0xFF]);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("Invalid UTF-8 sequence for String value"));

        // Test with a batch decodable type (should be handled by batch_value_decoder)
         let result = decode_basic_value(HtlvValueType::U32, 4, &[0x01, 0x00, 0x00, 0x00]);
         assert!(result.is_err());
         assert_eq!(result.unwrap_err().to_string(), "Codec Error: Batch decodable type U32 should be handled by batch_value_decoder");

        // Test with a complex type (should be handled by complex_value_handler)
         let result = decode_basic_value(HtlvValueType::Array, 0, &[]);
         assert!(result.is_err());
         assert_eq!(result.unwrap_err().to_string(), "Codec Error: Complex type Array should be handled by complex_value_handler");

        // Note: Large field types test removed as they use the same HtlvValueType (Bytes/String)
        // but are processed differently by large_field_handler.rs
    }
}