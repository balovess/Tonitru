use crate::internal::error::Result;
use crate::codec::types::{HtlvValue, HtlvValueType};
use super::encode_item; // Import encode_item from the parent module


/// Encodes a complex HtlvValue (Array or Object) into bytes.
/// Returns the value type byte and the encoded value bytes.
pub fn encode_complex_value(value: &HtlvValue) -> Result<(u8, Vec<u8>)> {
    match value {
        HtlvValue::Array(items) => {
            let mut encoded_array_items = Vec::new();
            for sub_item in items {
                // Recursively call encode_item for nested items
                encoded_array_items.extend_from_slice(&encode_item(sub_item)?);
            }
            Ok((HtlvValueType::Array as u8, encoded_array_items))
        },
        HtlvValue::Object(fields) => {
            let mut encoded_object_fields = Vec::new();
            for field_item in fields {
                // Recursively call encode_item for nested fields
                encoded_object_fields.extend_from_slice(&encode_item(field_item)?);
            }
            Ok((HtlvValueType::Object as u8, encoded_object_fields))
        },
        // Basic types will be handled in basic.rs
        _ => {
            Err(crate::internal::error::Error::CodecError("Attempted to encode basic type with complex encoder".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::types::{HtlvItem, HtlvValue, HtlvValueType};
    // Removed unused import: use bytes::Bytes;

    #[test]
    fn test_encode_complex_value() {
        // Test Array
        let array_items = vec![
            HtlvItem::new(1, HtlvValue::U64(100)),
            HtlvItem::new(2, HtlvValue::Bool(false)),
        ];
        let value_array = HtlvValue::Array(array_items);
        let (type_byte_array, encoded_array) = encode_complex_value(&value_array).unwrap();
        assert_eq!(type_byte_array, HtlvValueType::Array as u8);
        // Expected encoded array items:
        // Item 1 (Tag 1, U64 100): [0x01, 0x05, 0x08, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00] (11 bytes)
        // Item 2 (Tag 2, Bool false): [0x02, 0x01, 0x01, 0x00] (4 bytes)
        // Combined: [0x01, 0x05, 0x08, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x01, 0x01, 0x00]
        assert_eq!(encoded_array, vec![0x01, 0x05, 0x08, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x01, 0x01, 0x00]);
    }

    #[test]
    fn test_encode_complex_value_error() {
        // Attempt to encode a basic type with the complex encoder
        let value_u64 = HtlvValue::U64(100);
        let result = encode_complex_value(&value_u64);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Codec Error: Attempted to encode basic type with complex encoder");
    }
}