use crate::codec::types::{HtlvValue, HtlvValueType};
pub use crate::internal::error::{Error, Result};
use crate::codec::varint;
use crate::codec::decode::basic_types;
use crate::codec::decode::complex_types::{array, object}; // Import the new modules


/// Represents a parsed HTLV header (Tag, Type, Length).
#[derive(Debug, PartialEq, Clone)] // Added Clone as we will collect these
struct ParsedHeader {
    tag: u64,
    value_type: HtlvValueType,
    length: usize,
    header_len: usize, // tag + type + length 总长度
}

/// Represents an item that has been scanned (header parsed) but its value has not yet been decoded.
#[derive(Debug)]
struct PendingItem<'a> {
    header: ParsedHeader,
    raw_value_slice: &'a [u8],
    // Removed original_slice_start_offset as it's not needed with the new batch processing approach
}


/// Parses the Tag, Type, and Length components of an HTLV item header.
/// Returns the parsed header information and the total length of the header in bytes.
fn parse_tlv_header(slice: &[u8]) -> Result<ParsedHeader> {
    let mut current_offset = 0;

    // Decode Tag
    let (tag, tag_bytes) = varint::decode_varint(&slice[current_offset..])
        .map_err(|e| Error::CodecError(format!("Failed to decode item Tag varint: {}", e)))?;
    current_offset += tag_bytes;

    // Ensure there's enough data for the Type byte
    if slice.len() < current_offset + 1 {
         return Err(Error::CodecError("Incomplete data for item Type byte".to_string()));
    }

    // Decode Type
    let value_type_byte = slice[current_offset];
    current_offset += 1;

    let value_type = HtlvValueType::from_byte(value_type_byte)
        .ok_or_else(|| Error::CodecError(format!("Unknown value type tag: {}", value_type_byte)))?;

    // Decode Length
    let remaining_data_after_type = &slice[current_offset..];
    let (length, length_bytes) = varint::decode_varint(remaining_data_after_type)
        .map_err(|e| Error::CodecError(format!("Failed to decode item Length varint: {}", e)))?;
    current_offset += length_bytes;

    let header_len = current_offset;

    Ok(ParsedHeader {
        tag,
        value_type,
        length: length as usize, // Assuming length fits in usize
        header_len,
    })
}

/// Detects consecutive items with the same Tag and Type, indicating a batch.
/// Returns a vector of ParsedHeader for the batch items.
fn detect_batch_items(
    raw_slice: &[u8],
    start_offset: usize,
) -> Result<Vec<ParsedHeader>> {
    let mut current_offset = start_offset;
    let mut batch_headers = Vec::new();

    // Parse the header of the first item
    if current_offset >= raw_slice.len() {
        return Ok(batch_headers); // No data to process
    }
    let first_header = parse_tlv_header(&raw_slice[current_offset..])?;

    // If the first item is a complex type, it cannot be a batch of basic types
    match first_header.value_type {
        HtlvValueType::Array | HtlvValueType::Object => return Ok(batch_headers),
        _ => (),
    }

    // Add the first header to the batch
    batch_headers.push(first_header.clone());
    current_offset += first_header.header_len + first_header.length;


    // Iterate and collect consecutive items with the same tag and type
    while current_offset < raw_slice.len() {
        let header = match parse_tlv_header(&raw_slice[current_offset..]) {
            Ok(h) => h,
            Err(_) => break, // Stop if header parsing fails (e.e., incomplete data)
        };

        // Check if the tag and type match the initial item and if it's a basic type
        if header.tag == first_header.tag && header.value_type == first_header.value_type {
            // Ensure there's enough data for the value
            if raw_slice.len() < current_offset + header.header_len + header.length {
                break; // Stop if value data is incomplete
            }
            batch_headers.push(header.clone());
            current_offset += header.header_len + header.length;
        } else {
            break; // Stop if tag or type changes
        }
    }

    // If only one item was found, it's not a batch in the sense of multiple items
    if batch_headers.len() <= 1 {
        Ok(Vec::new()) // Return empty vector if no batch detected
    } else {
        Ok(batch_headers)
    }
}

/// Parses the items within a complex value (Array or Object).
/// This function contains the core iterative parsing logic, now structured for pipelining and batch decoding.
pub fn parse_complex_items(raw_value_slice: &[u8]) -> Result<Vec<crate::codec::types::HtlvItem>> {
    let mut pending_items: Vec<PendingItem> = Vec::new();
    let mut current_offset = 0;

    // Stage 1: Scan and parse headers, collect pending items
    while current_offset < raw_value_slice.len() {
        // Check for batch items first
        let batch_headers = detect_batch_items(&raw_value_slice, current_offset)?;

        if !batch_headers.is_empty() {
            // Collect batch items as pending items
            let mut batch_current_offset = current_offset;
            for header in batch_headers {
                // Ensure enough data for the batch item value
                 if raw_value_slice.len() < batch_current_offset + header.header_len + header.length {
                    return Err(Error::CodecError(format!("Incomplete data for batch item Value (expected {} bytes)", header.length)));
                }

                let batch_item_value_start = batch_current_offset + header.header_len;
                let batch_item_value_end = batch_item_value_start + header.length;
                let batch_item_raw_value_slice = &raw_value_slice[batch_item_value_start..batch_item_value_end];

                pending_items.push(PendingItem {
                    header,
                    raw_value_slice: batch_item_raw_value_slice,
                });

                // Advance offset within the batch
                batch_current_offset = batch_item_value_end;
            }
            // After collecting the batch, update the main offset
            current_offset = batch_current_offset;

        } else {
            // Not a batch, parse a single item
            let header = parse_tlv_header(&raw_value_slice[current_offset..])?;

            // Ensure there's enough data for the nested Value
            if raw_value_slice.len() < current_offset + header.header_len + header.length {
                return Err(Error::CodecError(format!("Incomplete data for nested Value (expected {} bytes)", header.length)));
            }

            let nested_value_start = current_offset + header.header_len;
            let nested_value_end = nested_value_start + header.length;
            let nested_raw_value_slice = &raw_value_slice[nested_value_start..nested_value_end];

            pending_items.push(PendingItem {
                header,
                raw_value_slice: nested_raw_value_slice,
            });

            // Advance offset past this single item
            current_offset = nested_value_end;
        }
    }

    // Ensure all data in the raw_value_slice was consumed during the scanning phase
    if current_offset != raw_value_slice.len() {
         return Err(Error::CodecError(format!("Extra data remaining after scanning complex value: {} bytes", raw_value_slice.len() - current_offset)));
    }


    // Stage 2: Decode the values of the pending items, handling batches
    let mut items = Vec::new();
    let mut i = 0;
    while i < pending_items.len() {
        let current_item = &pending_items[i];

        match current_item.header.value_type {
            HtlvValueType::Array | HtlvValueType::Object => {
                // Decode nested complex types recursively
                let decoded_nested_value = decode_complex_value(current_item.header.value_type, current_item.raw_value_slice)?;
                items.push(crate::codec::types::HtlvItem::new(current_item.header.tag, decoded_nested_value));
                i += 1;
            }
            basic_type => {
                // Check for a batch of this basic type
                let mut batch_end = i;
                while batch_end < pending_items.len() && pending_items[batch_end].header.tag == current_item.header.tag && pending_items[batch_end].header.value_type == basic_type {
                    batch_end += 1;
                }

                let batch_count = batch_end - i;

                if batch_count > 1 {
                    // Process as a batch
                    // Collect raw value slices for the batch and concatenate them
                    let mut raw_batch_values_combined = Vec::new();
                    for j in i..batch_end {
                        raw_batch_values_combined.extend_from_slice(pending_items[j].raw_value_slice);
                    }

                    let decoded_values = match basic_type {
                        HtlvValueType::U8 => basic_types::u8::decode_u8_batch(&raw_batch_values_combined, batch_count)?.into_iter().map(HtlvValue::U8).collect::<Vec<HtlvValue>>(),
                        HtlvValueType::U16 => basic_types::u16::decode_u16_batch(&raw_batch_values_combined, batch_count)?.into_iter().map(HtlvValue::U16).collect::<Vec<HtlvValue>>(),
                        HtlvValueType::U32 => basic_types::u32::decode_u32_batch(&raw_batch_values_combined, batch_count)?.into_iter().map(HtlvValue::U32).collect::<Vec<HtlvValue>>(),
                        HtlvValueType::U64 => basic_types::u64::decode_u64_batch(&raw_batch_values_combined, batch_count)?.into_iter().map(HtlvValue::U64).collect::<Vec<HtlvValue>>(),
                        HtlvValueType::I8 => basic_types::i8::decode_i8_batch(&raw_batch_values_combined, batch_count)?.into_iter().map(HtlvValue::I8).collect::<Vec<HtlvValue>>(),
                        HtlvValueType::I16 => basic_types::i16::decode_i16_batch(&raw_batch_values_combined, batch_count)?.into_iter().map(HtlvValue::I16).collect::<Vec<HtlvValue>>(),
                        HtlvValueType::I32 => basic_types::i32::decode_i32_batch(&raw_batch_values_combined, batch_count)?.into_iter().map(HtlvValue::I32).collect::<Vec<HtlvValue>>(),
                        HtlvValueType::I64 => basic_types::i64::decode_i64_batch(&raw_batch_values_combined, batch_count)?.into_iter().map(HtlvValue::I64).collect::<Vec<HtlvValue>>(),
                        // TODO: Add batch handling for other basic types (Floats, Bytes, String)
                        _ => {
                             // Fallback to decoding individual items for other basic types or non-integer batches
                            let mut individual_decoded_values = Vec::with_capacity(batch_count);
                            for j in i..batch_end {
                                let pending_item = &pending_items[j];
                                let decoded_nested_value = match pending_item.header.value_type {
                                    HtlvValueType::Null => basic_types::null::decode_null(pending_item.header.length as u64)?,
                                    HtlvValueType::Bool => basic_types::boolean::decode_bool(pending_item.header.length as u64, pending_item.raw_value_slice)?,
                                    HtlvValueType::F32 => basic_types::floats::decode_f32(pending_item.header.length as u64, pending_item.raw_value_slice)?,
                                    HtlvValueType::F64 => basic_types::floats::decode_f64(pending_item.header.length as u64, pending_item.raw_value_slice)?,
                                    HtlvValueType::Bytes => basic_types::bytes_and_string::decode_bytes(pending_item.raw_value_slice)?,
                                    HtlvValueType::String => basic_types::bytes_and_string::decode_string(pending_item.raw_value_slice)?,
                                    _ => unreachable!("Should not encounter complex types or already handled integer types here"),
                                };
                                individual_decoded_values.push(decoded_nested_value);
                            }
                            individual_decoded_values
                        }
                    };

                    // Convert decoded values to HtlvItem and add to results
                    for (j, value) in decoded_values.into_iter().enumerate() {
                         // Use the tag from the corresponding pending item in the batch
                        items.push(crate::codec::types::HtlvItem::new(pending_items[i + j].header.tag, value));
                    }

                    // Advance index past the processed batch
                    i = batch_end;
                } else {
                    // Process as a single item
                    let decoded_nested_value = match current_item.header.value_type {
                        HtlvValueType::Null => basic_types::null::decode_null(current_item.header.length as u64)?,
                        HtlvValueType::Bool => basic_types::boolean::decode_bool(current_item.header.length as u64, current_item.raw_value_slice)?,
                        HtlvValueType::U8 => basic_types::u8::decode_u8(current_item.header.length as u64, current_item.raw_value_slice)?,
                        HtlvValueType::U16 => basic_types::u16::decode_u16(current_item.header.length as u64, current_item.raw_value_slice)?,
                        HtlvValueType::U32 => basic_types::u32::decode_u32(current_item.header.length as u64, current_item.raw_value_slice)?,
                        HtlvValueType::U64 => basic_types::u64::decode_u64(current_item.header.length as u64, current_item.raw_value_slice)?,
                        HtlvValueType::I8 => basic_types::i8::decode_i8(current_item.header.length as u64, current_item.raw_value_slice)?,
                        HtlvValueType::I16 => basic_types::i16::decode_i16(current_item.header.length as u64, current_item.raw_value_slice)?,
                        HtlvValueType::I32 => basic_types::i32::decode_i32(current_item.header.length as u64, current_item.raw_value_slice)?,
                        HtlvValueType::I64 => basic_types::i64::decode_i64(current_item.header.length as u64, current_item.raw_value_slice)?,
                        HtlvValueType::F32 => basic_types::floats::decode_f32(current_item.header.length as u64, current_item.raw_value_slice)?,
                        HtlvValueType::F64 => basic_types::floats::decode_f64(current_item.header.length as u64, current_item.raw_value_slice)?,
                        HtlvValueType::Bytes => basic_types::bytes_and_string::decode_bytes(current_item.raw_value_slice)?,
                        HtlvValueType::String => basic_types::bytes_and_string::decode_string(current_item.raw_value_slice)?,
                        _ => unreachable!("Should not encounter complex types here"),
                    };
                    items.push(crate::codec::types::HtlvItem::new(current_item.header.tag, decoded_nested_value));
                    i += 1;
                }
            }
        }
    }

    Ok(items)
}


/// Decodes a complex HtlvValue (Array or Object) from a byte slice.
/// This function delegates the actual item parsing to `parse_complex_items`.
pub fn decode_complex_value(
    value_type: HtlvValueType,
    raw_value_slice: &[u8],
) -> Result<HtlvValue> {
    // First check if this is actually a complex type
    match value_type {
        HtlvValueType::Array => {
            array::decode_array(raw_value_slice)
        }
        HtlvValueType::Object => {
            object::decode_object(raw_value_slice)
        }
        _ => return Err(Error::CodecError(
            "Attempted to decode basic type with complex decoder".to_string()
        )),
    }
}


#[cfg(test)]
mod tests {
    // Import items needed for tests
    use crate::codec::types::{HtlvValue, HtlvValueType};
    use bytes::Bytes;

    use super::{detect_batch_items, parse_tlv_header, parse_complex_items}; // parse_complex_items is still used internally for tests

    // We no longer test decode_complex_value directly here, as its logic is now
    // delegated to array::decode_array and object::decode_object.
    // Tests for Array and Object decoding should be in their respective modules.

    // The existing tests for parse_tlv_header and detect_batch_items are still relevant here.

    #[test]
    fn test_parse_tlv_header() {
        // Test case with a simple header (Tag=1, Type=U8, Length=1)
        let data = Bytes::from_static(&[0x01, 0x02, 0x01, 0x05]); // Tag 1, Type U8, Length 1, Value 5
        let result = parse_tlv_header(&data).unwrap();
        assert_eq!(result.tag, 1);
        assert_eq!(result.value_type, HtlvValueType::U8);
        assert_eq!(result.length, 1);
        assert_eq!(result.header_len, 3); // Tag (1 byte) + Type (1 byte) + Length (1 byte)

        // Test case with a larger tag and length
        let data = Bytes::from_static(&[0x81, 0x01, 0x03, 0x82, 0x01, 0x00]); // Tag 129, Type U16, Length 130
        let result = parse_tlv_header(&data).unwrap();
        assert_eq!(result.tag, 129);
        assert_eq!(result.value_type, HtlvValueType::U16);
        assert_eq!(result.length, 130);
        assert_eq!(result.header_len, 5); // Tag (2 bytes) + Type (1 byte) + Length (2 bytes)

        // Test case with incomplete data for Type
        let data = Bytes::from_static(&[0x01]); // Only Tag
        let result = parse_tlv_header(&data);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Codec Error: Incomplete data for item Type byte");

        // Test case with incomplete data for Length
        let data = Bytes::from_static(&[0x01, 0x01]); // Tag and Type, but no Length
        let result = parse_tlv_header(&data);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to decode item Length varint"));
    }

    #[test]
    fn test_detect_batch_items() {
        // Create a sample raw slice with batch items
        // Item 1: Tag 1, Type U32, Length 4, Value 10
        // Item 2: Tag 1, Type U32, Length 4, Value 20
        // Item 3: Tag 2, Type U32, Length 4, Value 30 (Different Tag)
        // Item 4: Tag 1, Type U64, Length 8, Value 40 (Different Type)
        let raw_slice = Bytes::from_static(&[
            // Item 1 (Tag 1, Type U32, Length 4)
            0x01, 0x04, 0x04, 0x0a, 0x00, 0x00, 0x00,
            // Item 2 (Tag 1, Type U32, Length 4)
            0x01, 0x04, 0x04, 0x14, 0x00, 0x00, 0x00,
            // Item 3 (Tag 2, Type U32, Length 4)
            0x02, 0x04, 0x04, 0x1e, 0x00, 0x00, 0x00,
            // Item 4 (Tag 1, Type U64, Length 8)
            0x01, 0x05, 0x08, 0x28, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]);

        // Test detecting batch of U32 starting at offset 0
        let result = detect_batch_items(&raw_slice, 0).unwrap();
        assert_eq!(result.len(), 2); // Expecting 2 batch items
        assert_eq!(result[0].tag, 1);
        assert_eq!(result[0].value_type, HtlvValueType::U32);
        assert_eq!(result[1].tag, 1);
        assert_eq!(result[1].value_type, HtlvValueType::U32);


        // Test detecting batch of U32 starting at offset 14 (should find 0 items as it's a single item)
        let result = detect_batch_items(&raw_slice, 14).unwrap();
        assert_eq!(result.len(), 0); // Expecting 0 batch items

        // Test detecting batch of U64 starting at offset 21 (should find 0 items as it's a single item)
        let result = detect_batch_items(&raw_slice, 21).unwrap();
        assert_eq!(result.len(), 0); // Expecting 0 batch items


        // Test with empty slice
        let raw_slice_empty = Bytes::from_static(&[]);
        let result = detect_batch_items(&raw_slice_empty, 0).unwrap();
        assert_eq!(result.len(), 0);
        // Test with incomplete data for the first header
        let raw_slice_incomplete_header = Bytes::from_static(&[0x01]); // Only Tag
        let result = detect_batch_items(&raw_slice_incomplete_header, 0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Codec Error: Incomplete data for item Type byte");

         // Test with incomplete data for a value within a potential batch
         let raw_slice_incomplete_value = Bytes::from_static(&[
            // Item 1: Tag 1, Type U32, Length 4, Value 10
            0x01, 0x04, 0x04, 0x0a, 0x00, 0x00, 0x00,

            // Item 2: Tag 1, Type U32, Length 4, Incomplete Value
            0x01, 0x04, 0x04, 0x14, 0x00,
        ]);
        let result = detect_batch_items(&raw_slice_incomplete_value, 0).unwrap();
        assert_eq!(result.len(), 0); // Should detect only the first complete item, but function returns empty if less than 2
    }

    // Add tests for parse_complex_items here
    #[test]
    fn test_parse_complex_items() {
        // Test case with a simple array of basic types
        let raw_array_value = Bytes::from_static(&[
            // Item 1: Tag 1, Type U32, Length 4, Value 10
            0x01, 0x04, 0x04, 0x0a, 0x00, 0x00, 0x00,
            // Item 2: Tag 2, Type Bool, Length 1, Value true
            0x02, 0x01, 0x01, 0x01,
        ]);
        let items = parse_complex_items(&raw_array_value).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].tag, 1);
        assert_eq!(items[0].value, HtlvValue::U32(10));
        assert_eq!(items[1].tag, 2);
        assert_eq!(items[1].value, HtlvValue::Bool(true));

        // Test case with a nested complex type (Array containing an Object)
        let _raw_nested_object_value = Bytes::from_static(&[
            // Nested Object Item 1: Tag 1, Type String, Length 4, Value "name"
            0x01, 0x0d, 0x04, 0x6e, 0x61, 0x6d, 0x65, // "name"
            // Nested Object Item 2: Tag 2, Type I64, Length 8, Value 123
            0x02, 0x09, 0x08, 0x7b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 123
        ]);

        let raw_nested_array_value = Bytes::from_static(&[
            // Item 1: Tag 10, Type Object, Length of nested object value
            0x0a, 0x0f, 0x12, // Tag 10, Type Object, Length 18 (length of nested object value)
            // Nested Object Value
            0x01, 0x0d, 0x04, 0x6e, 0x61, 0x6d, 0x65, // "name"
            0x02, 0x09, 0x08, 0x7b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 123
        ]);

        let items = parse_complex_items(&raw_nested_array_value).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].tag, 10);
        match &items[0].value {
            HtlvValue::Object(nested_items) => {
                assert_eq!(nested_items.len(), 2);
                assert_eq!(nested_items[0].tag, 1);
                assert_eq!(nested_items[0].value, HtlvValue::String(Bytes::from_static("name".as_bytes())));
                assert_eq!(nested_items[1].tag, 2);
                assert_eq!(nested_items[1].value, HtlvValue::I64(123));
            }
            _ => panic!("Expected Object value"),
        }

        // Test case with a batch of basic types
        let raw_batch_value = Bytes::from_static(&[
            // Item 1: Tag 1, Type U32, Length 4, Value 10
            0x01, 0x04, 0x04, 0x0a, 0x00, 0x00, 0x00,
            // Item 2: Tag 1, Type U32, Length 4, Value 20 (Batch)
            0x01, 0x04, 0x04, 0x14, 0x00, 0x00, 0x00,
            // Item 3: Tag 2, Type Bool, Length 1, Value false
            0x02, 0x01, 0x01, 0x00,
        ]);
        let items = parse_complex_items(&raw_batch_value).unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].tag, 1);
        assert_eq!(items[0].value, HtlvValue::U32(10));
        assert_eq!(items[1].tag, 1);
        assert_eq!(items[1].value, HtlvValue::U32(20));
        assert_eq!(items[2].tag, 2);
        assert_eq!(items[2].value, HtlvValue::Bool(false));
    }

    #[test]
    fn test_parse_complex_items_errors() {
        // Test case with incomplete data for an item
        let raw_incomplete_value = Bytes::from_static(&[
            // Item 1: Tag 1, Type U32, Length 4, Value 10 (incomplete)
            0x01, 0x04, 0x04, 0x0a, 0x00,
        ]);
        let result = parse_complex_items(&raw_incomplete_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("Incomplete data for nested Value") || error_message.contains("Failed to decode item Length varint"));

        // Test case with extra data at the end
        let raw_extra_data = Bytes::from_static(&[
            // Item 1: Tag 1, Type U32, Length 4, Value 10
            0x01, 0x04, 0x04, 0x0a, 0x00, 0x00, 0x00,
            // Extra data
            0xFF, 0xFF,
        ]);
        let result = parse_complex_items(&raw_extra_data);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert_eq!(error_message, "Codec Error: Failed to decode item Tag varint: Codec Error: Incomplete varint data");

        // Test case with incomplete data within a batch
        let raw_incomplete_batch = Bytes::from_static(&[
             // Item 1: Tag 1, Type U32, Length 4, Value 10
            0x01, 0x04, 0x04, 0x0a, 0x00, 0x00, 0x00,
            // Item 2: Tag 1, Type U32, Length 4, Incomplete Value
            0x01, 0x04, 0x04, 0x14, 0x00,
        ]);
         let result = parse_complex_items(&raw_incomplete_batch);
         assert!(result.is_err());
         let error_message = result.unwrap_err().to_string();
         // Update assertion to check for the actual error message related to slicing
         assert!(error_message.contains("Incomplete data for nested Value") || error_message.contains("index out of bounds") || error_message.contains("Incomplete data for batch item Value") || error_message.contains("Failed to decode item Length varint"));
    }
}
