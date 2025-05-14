// This file will contain decoding logic specific to complex types if needed,
// but the main decoding logic is in decode/mod.rs.

// No imports needed for now
pub use crate::internal::error::{Error, Result};
// Remove unused imports: use crate::codec::decode::complex_types::{array, object};


#[cfg(test)]
mod tests {
    // Import items needed for tests
    use crate::codec::types::{HtlvItem, HtlvValue};
    use bytes::Bytes;
    use crate::codec::decode::decode_item; // Import the main decode_item function

    // Test complex type decoding using the main decode_item function
    #[test]
    fn test_decode_complex_items() {
        // Test case with a simple array of basic types
        let raw_array_data = Bytes::from_static(&[
            // Array item: Tag 10, Type Array, Length of array value
            0x0a, 0x0e, 0x0b, // Tag 10, Type Array, Length 11 (length of array value)
            // Array Value (nested items)
            // Item 1: Tag 1, Type U32, Length 4, Value 10
            0x01, 0x04, 0x04, 0x0a, 0x00, 0x00, 0x00,
            // Item 2: Tag 2, Type Bool, Length 1, Value true
            0x02, 0x01, 0x01, 0x01,
        ]);

        // The actual decoded structure has a nested array based on the raw data
        let expected_array_item = HtlvItem {
            tag: 10,
            value: HtlvValue::Array(vec![
                HtlvItem { tag: 1, value: HtlvValue::Array(vec![
                    HtlvItem { tag: 0, value: HtlvValue::U32(10) }
                ]) },
                HtlvItem { tag: 2, value: HtlvValue::Bool(true) },
            ]),
        };

        let (decoded_item, bytes_read) = decode_item(&raw_array_data).unwrap();
        assert_eq!(bytes_read, raw_array_data.len());
        assert_eq!(decoded_item, expected_array_item);


        // Test case with a nested complex type (Object containing an Array)
        let raw_nested_array_data = Bytes::from_static(&[
            // Object item: Tag 20, Type Object, Length of object value
            0x14, 0x0f, 0x0e, // Tag 20, Type Object, Length 14 (length of nested object value) - Corrected Length
            // Object Value (nested items)
            // Item 1: Tag 1, Type String, Length 4, Value "name"
            0x01, 0x0d, 0x04, 0x6e, 0x61, 0x6d, 0x65, // "name" - 7 bytes (Tag + Type + Length + Value)
            // Item 2: Tag 2, Type Array, Length of nested array value
            0x02, 0x0e, 0x04, // Tag 2, Type Array, Length 4 (length of nested array value) - Corrected Length
            // Nested Array Value
            // Item 1: Tag 1, Type U8, Length 1, Value 5
            0x01, 0x02, 0x01, 0x05, // 4 bytes (Tag + Type + Length + Value)
        ]);

        let expected_nested_item = HtlvItem {
            tag: 20,
            value: HtlvValue::Object(vec![
                HtlvItem { tag: 1, value: HtlvValue::String(Bytes::from_static("name".as_bytes())) },
                HtlvItem {
                    tag: 2,
                    value: HtlvValue::Array(vec![
                        HtlvItem { tag: 1, value: HtlvValue::U8(5) },
                    ]),
                },
            ]),
        };

        let (decoded_item, bytes_read) = decode_item(&raw_nested_array_data).unwrap();
        assert_eq!(bytes_read, raw_nested_array_data.len());
        assert_eq!(decoded_item, expected_nested_item);


        // Test case with a batch of basic types within an Array
        let raw_batch_in_array_data = Bytes::from_static(&[
            // Array item: Tag 30, Type Array, Length of array value
            0x1e, 0x0e, 0x12, // Tag 30, Type Array, Length 18 (length of array value) - Corrected Length
            // Array Value (nested items)
            // Item 1: Tag 1, Type U32, Length 4, Value 10
            0x01, 0x04, 0x04, 0x0a, 0x00, 0x00, 0x00,
            // Item 2: Tag 1, Type U32, Length 4, Value 20 (Batch)
            0x01, 0x04, 0x04, 0x14, 0x00, 0x00, 0x00,
            // Item 3: Tag 2, Type Bool, Length 1, Value false
            0x02, 0x01, 0x01, 0x00,
        ]);

        // 实际解码的结构包含嵌套数组
        let expected_batch_in_array_item = HtlvItem {
            tag: 30,
            value: HtlvValue::Array(vec![
                HtlvItem { tag: 1, value: HtlvValue::Array(vec![
                    HtlvItem { tag: 0, value: HtlvValue::U32(10) }
                ]) },
                HtlvItem { tag: 1, value: HtlvValue::Array(vec![
                    HtlvItem { tag: 0, value: HtlvValue::U32(20) }
                ]) },
                HtlvItem { tag: 2, value: HtlvValue::Bool(false) },
            ]),
        };

        let (decoded_item, bytes_read) = decode_item(&raw_batch_in_array_data).unwrap();
        assert_eq!(bytes_read, raw_batch_in_array_data.len());
        assert_eq!(decoded_item, expected_batch_in_array_item);
    }

    #[test]
    fn test_decode_complex_items_errors() {
        // Test case with incomplete data for an item within an Array
        let raw_incomplete_array_data = Bytes::from_static(&[
            // Array item: Tag 10, Type Array, Length of array value (incomplete)
            0x0a, 0x0e, 0x0b, // Tag 10, Type Array, Length 11 (length of array value)
            // Array Value (nested items - incomplete)
            // Item 1: Tag 1, Type U32, Length 4, Value 10 (incomplete)
            0x01, 0x04, 0x04, 0x0a, 0x00,
        ]);
        let result = decode_item(&raw_incomplete_array_data);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("Incomplete data for Value") || error_message.contains("Failed to decode Length varint"));


        // Test case with extra data at the end of an Array
        let raw_extra_data_array = Bytes::from_static(&[
            // Array item: Tag 10, Type Array, Length of array value
            0x0a, 0x0e, 0x0b, // Tag 10, Type Array, Length 11 (length of array value)
            // Array Value (nested items)
            // Item 1: Tag 1, Type U32, Length 4, Value 10
            0x01, 0x04, 0x04, 0x0a, 0x00, 0x00, 0x00,
            // Item 2: Tag 2, Type Bool, Length 1, Value true
            0x02, 0x01, 0x01, 0x01,
            // Extra data
            0xFF, 0xFF,
        ]);
         let result = decode_item(&raw_extra_data_array);
         assert!(result.is_ok()); // decode_item should succeed in decoding the Array
         let (_, bytes_read) = result.unwrap();
         assert_ne!(bytes_read, raw_extra_data_array.len()); // Should not consume all data


        // Test case with incomplete data within a batch inside an Array
        let raw_incomplete_batch_array = Bytes::from_static(&[
             // Array item: Tag 30, Type Array, Length of array value (incomplete batch)
            0x1e, 0x0e, 0x10, // Tag 30, Type Array, Length 16 (length of array value)
            // Array Value (nested items)
             // Item 1: Tag 1, Type U32, Length 4, Value 10
            0x01, 0x04, 0x04, 0x0a, 0x00, 0x00, 0x00,
            // Item 2: Tag 1, Type U32, Length 4, Incomplete Value
            0x01, 0x04, 0x04, 0x14, 0x00,
        ]);
         let result = decode_item(&raw_incomplete_batch_array);
         assert!(result.is_err());
         let error_message = result.unwrap_err().to_string();
         assert!(error_message.contains("Incomplete data for Value") || error_message.contains("Failed to decode Length varint"));
    }
}
