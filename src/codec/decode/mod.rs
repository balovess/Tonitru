// Decode module for HTLV (HyperNova) data format

pub mod basic; // Keep basic.rs for now, will modify its content later
pub mod basic_types; // Introduce the new basic_types module
pub mod complex; // complex module will be refactored or removed later
pub mod complex_types; // Declare the new complex_types module
pub mod htlv; // Export the htlv module
// pub mod context; // Import the context module - This has been moved
pub mod batch; // Declare the new batch module
pub mod decoder_state_machine; // Import the new state machine module

// Publicly re-export modules used by the state machine
pub mod basic_value_decoder;
pub mod batch_value_decoder;
pub mod complex_value_handler;
pub mod large_field_handler;
pub mod simd_optimizations;
pub mod pipeline_processor;


use crate::internal::error::{Error, Result};
use crate::codec::types::HtlvItem;
use decoder_state_machine::{DecodeContext, DecodeState}; // Import from the new state machine module


// Fixed length for the total length encoded in the large field header item value (size of u64)
// This constant is currently unused but kept for future reference
#[allow(dead_code)]
const TOTAL_LENGTH_HEADER_LEN: u64 = 8;


/// Decodes bytes into a single logical HTLV item (Tag + Type + Value) using an iterative approach
/// with a state machine to simulate a multi-stage pipeline and handle nested structures and large fields.
/// Returns the decoded HtlvItem and the number of bytes read for this logical item.
/// Note: For large fields, this function will consume multiple underlying HTLV items (header + shards).
pub fn decode_item(data: &[u8]) -> Result<(HtlvItem, usize)> {
    let mut ctx = DecodeContext::new(data);

    while ctx.state != DecodeState::Done {
        // println!("decode_item loop: current_offset = {}, state = {:?}", ctx.current_offset, ctx.state); // Debug print
        match ctx.state {
            DecodeState::Scan => ctx.handle_scan_state()?,
            DecodeState::PrepareValue => ctx.handle_prepare_value_state()?,
            DecodeState::DecodeValue => ctx.handle_decode_value_state()?,
            DecodeState::DecodeBatchValue => ctx.handle_decode_batch_value_state()?,
            DecodeState::ProcessComplex => ctx.handle_process_complex_state()?,
            DecodeState::Done => break, // Should exit the loop here
        }
    }

    // If we exit the loop while still decoding a large field, it's an error
    if ctx.decoding_large_field {
         return Err(Error::CodecError(format!("Incomplete large field data at end of stream. Expected total length {}, got {}", ctx.large_field_total_length, ctx.large_field_buffer.len())));
    }


    ctx.root_item.ok_or_else(|| Error::CodecError("Decoding failed: No root item decoded".to_string()))
        .map(|item| (item, ctx.bytes_read_for_root_item)) // Return bytes read for the root item
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::varint; // Import varint for tests
    use crate::codec::encode::encode_item; // Import encode_item for tests
    use decoder_state_machine::MAX_NESTING_DEPTH; // Import MAX_NESTING_DEPTH for tests
    use bytes::BytesMut;
    use crate::codec::types::{HtlvValue, HtlvValueType};

    #[test]
    fn test_decode_nested_depth_limit() {
        // Construct a deeply nested structure exceeding the limit (32 levels)
        let mut raw_data = BytesMut::new();
        let tag = 1;
        let value_type = HtlvValueType::Array; // Or HtlvValueType::Object

        // Create a structure with MAX_NESTING_DEPTH + 1 levels
        for _ in 0..MAX_NESTING_DEPTH + 1 {
            // Encode Tag (1), Type (Array/Object), and a placeholder Length (will be updated)
            let tag_bytes = varint::encode_varint(tag);
            raw_data.extend_from_slice(&tag_bytes);
            raw_data.extend_from_slice(&[value_type as u8]);
            // Placeholder for Length (1 byte for now, will be updated later)
            raw_data.extend_from_slice(&[0x00]);
        }

        // The total length of the data is the sum of the sizes of each nested item header.
        // Each header is Tag (varint, min 1 byte) + Type (1 byte) + Length (varint, min 1 byte).
        // For this test, we use tag 1 (1 byte) and a placeholder length 0 (1 byte).
        // So each header is 1 + 1 + 1 = 3 bytes.
        let _item_header_size = varint::encode_varint(tag).len() + 1 + varint::encode_varint(0).len(); // Unused but kept for clarity

        // Now, go back and update the Length fields.
        let mut current_offset = 0;
        for _ in 0..MAX_NESTING_DEPTH + 1 {
            // Skip Tag and Type
            let (_, tag_bytes_len) = varint::decode_varint(&raw_data[current_offset..]).unwrap();
            current_offset += tag_bytes_len + 1; // Skip Tag and Type byte

            // Calculate the remaining length
            let remaining_length = raw_data.len() - current_offset - varint::encode_varint(0).len();

            // Encode the actual length
            let length_bytes = varint::encode_varint(remaining_length as u64);

            // Replace the placeholder length bytes with the actual length bytes
            let mut new_raw_data = BytesMut::new();
            new_raw_data.extend_from_slice(&raw_data[..current_offset]);
            new_raw_data.extend_from_slice(&length_bytes);
            new_raw_data.extend_from_slice(&raw_data[current_offset + varint::encode_varint(0).len()..]);

            raw_data = new_raw_data;
            current_offset += length_bytes.len();
        }


        // Now decode and expect a depth limit error
        let result = decode_item(&raw_data);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            format!("Codec Error: Maximum nesting depth ({}) exceeded", MAX_NESTING_DEPTH)
        );
    }

    #[test]
    fn test_decode_array_batch_u8() {
        // Test decoding an Array containing a batch of U8 values
        // Correctly encode the Array with nested U8 items
        let items_to_encode = vec![
            HtlvItem::new(0, HtlvValue::U8(1)),
            HtlvItem::new(0, HtlvValue::U8(2)),
            HtlvItem::new(0, HtlvValue::U8(3)),
            HtlvItem::new(0, HtlvValue::U8(4)),
            HtlvItem::new(0, HtlvValue::U8(5)),
        ];
        let array_value = HtlvValue::Array(items_to_encode.clone());
        let raw_data = encode_item(&HtlvItem::new(10, array_value)).unwrap();


        let expected_item = HtlvItem::new(
            10,
            HtlvValue::Array(items_to_encode),
        );

        let (decoded_item, bytes_read) = decode_item(&raw_data).unwrap();
        assert_eq!(bytes_read, raw_data.len());
        assert_eq!(decoded_item, expected_item);
    }
}
