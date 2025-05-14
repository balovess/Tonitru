// Handler for large HTLV fields

use crate::internal::error::{Error, Result};
use crate::codec::types::{HtlvItem, HtlvValueType, HtlvValue};
use crate::codec::decode::decoder_state_machine::ComplexDecodeContext; // Keep ComplexDecodeContext for nested large fields
use bytes::BytesMut;
// Bytes import removed as it's not used
use std::mem; // Import std::mem for tests

/// Represents the result of processing a large field shard.
#[derive(Debug)] // Add Debug derive
pub enum LargeFieldProcessingResult {
    /// The large field is complete, contains the decoded item and bytes read.
    Completed(HtlvItem, usize),
    /// More shards are expected.
    Incomplete,
}

/// Handles the logic for decoding large HTLV fields.
pub struct LargeFieldHandler;

impl LargeFieldHandler {
    /// Processes a large field shard.
    /// Appends the shard data to the buffer and checks if the large field is complete.
    /// Returns a `Result<LargeFieldProcessingResult>`.
    pub fn process_shard(
        large_field_tag: u64,
        large_field_value_type: HtlvValueType,
        large_field_total_length: u64,
        large_field_buffer: &mut BytesMut,
        raw_value_slice: &[u8],
        current_offset_after_shard: usize, // Pass the offset after processing this shard
        complex_stack: &mut Vec<ComplexDecodeContext>, // Pass complex stack for nested large fields
    ) -> Result<LargeFieldProcessingResult> {
        large_field_buffer.extend_from_slice(raw_value_slice);

        // println!("Processing large field shard: buffer_len = {}, total_length = {}",
        //            large_field_buffer.len(), large_field_total_length); // Debug print

        if large_field_buffer.len() as u64 > large_field_total_length {
             return Err(Error::CodecError(format!("Large field buffer overflow. Expected total length {}, got more than {} bytes", large_field_total_length, large_field_buffer.len())));
        }

        if large_field_buffer.len() as u64 == large_field_total_length {
            // Finished decoding the large field
            let final_value = match large_field_value_type { // Use stored large field type
                HtlvValueType::Bytes => HtlvValue::Bytes(mem::take(large_field_buffer).freeze()), // Take ownership and freeze
                HtlvValueType::String => HtlvValue::String(mem::take(large_field_buffer).freeze()), // Take ownership and freeze
                _ => unreachable!(), // Should be Bytes or String
            };

            let decoded_item = HtlvItem::new(large_field_tag, final_value);

            // Handle nested large fields
            if let Some(parent_context) = complex_stack.last_mut() {
                 parent_context.items.push(decoded_item);
                 // The state transition and offset update will be handled by the caller (DecodeContext)
                 Ok(LargeFieldProcessingResult::Incomplete) // Still within a complex item, but the large field itself is complete
            } else {
                 Ok(LargeFieldProcessingResult::Completed(decoded_item, current_offset_after_shard))
            }

        } else {
            // Still expecting more shards
            Ok(LargeFieldProcessingResult::Incomplete)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::decode::decoder_state_machine::{ComplexDecodeContext};
    use crate::codec::types::{HtlvValueType, HtlvValue};
    use bytes::BytesMut;

    #[test]
    fn test_process_shard_complete_root() {
        let data = BytesMut::from(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10][..]);
        let mut buffer = BytesMut::new();
        let tag = 100;
        let value_type = HtlvValueType::Bytes;
        let total_length = 10;
        let raw_value_slice = &data[0..10];
        let current_offset_after_shard = 10;
        let mut complex_stack = Vec::new();

        let result = LargeFieldHandler::process_shard(
            tag,
            value_type,
            total_length,
            &mut buffer,
            raw_value_slice,
            current_offset_after_shard,
            &mut complex_stack,
        );

        assert!(result.is_ok());
        if let LargeFieldProcessingResult::Completed(decoded_item, bytes_read) = result.unwrap() {
            assert_eq!(decoded_item.tag, tag);
            if let HtlvValue::Bytes(bytes) = decoded_item.value {
                assert_eq!(bytes.as_ref(), &data[0..10][..]); // Compare byte slices
            } else {
                panic!("Decoded value is not Bytes");
            }
            assert_eq!(bytes_read, current_offset_after_shard);
        } else {
            panic!("Result is not Completed");
        }
        assert_eq!(buffer.len(), 0); // Buffer should be empty after freezing
    }

    #[test]
    fn test_process_shard_incomplete() {
        let data = BytesMut::from(&[1, 2, 3, 4, 5][..]);
        let mut buffer = BytesMut::new();
        let tag = 100;
        let value_type = HtlvValueType::Bytes;
        let total_length = 10;
        let raw_value_slice = &data[0..5];
        let current_offset_after_shard = 5;
        let mut complex_stack = Vec::new();

        let result = LargeFieldHandler::process_shard(
            tag,
            value_type,
            total_length,
            &mut buffer,
            raw_value_slice,
            current_offset_after_shard,
            &mut complex_stack,
        );

        assert!(result.is_ok());
        if let LargeFieldProcessingResult::Incomplete = result.unwrap() {
            // Correct state
        } else {
            panic!("Result is not Incomplete");
        }
        assert_eq!(buffer.len(), 5); // Buffer should still contain the data
    }

    #[test]
    fn test_process_shard_buffer_overflow() {
        let data = BytesMut::from(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11][..]);
        let mut buffer = BytesMut::new();
        let tag = 100;
        let value_type = HtlvValueType::Bytes;
        let total_length = 10;
        let raw_value_slice = &data[0..11]; // Slice is larger than total length
        let current_offset_after_shard = 11;
        let mut complex_stack = Vec::new();

        let result = LargeFieldHandler::process_shard(
            tag,
            value_type,
            total_length,
            &mut buffer,
            raw_value_slice,
            current_offset_after_shard,
            &mut complex_stack,
        );

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Large field buffer overflow. Expected total length 10, got more than 11 bytes"
        );
         assert_eq!(buffer.len(), 11); // Buffer should contain the data that caused overflow
    }

    #[test]
    fn test_process_shard_complete_nested() {
        let data = BytesMut::from(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10][..]);
        let mut buffer = BytesMut::new();
        let tag = 100;
        let value_type = HtlvValueType::Bytes;
        let total_length = 10;
        let raw_value_slice = &data[0..10];
        let current_offset_after_shard = 10;
        let mut complex_stack = vec![ComplexDecodeContext {
            tag: 1,
            value_type: HtlvValueType::Array,
            end_offset: 20,
            items: Vec::new(),
            depth: 1,
        }];

        let result = LargeFieldHandler::process_shard(
            tag,
            value_type,
            total_length,
            &mut buffer,
            raw_value_slice,
            current_offset_after_shard,
            &mut complex_stack,
        );

        assert!(result.is_ok());
        if let LargeFieldProcessingResult::Incomplete = result.unwrap() {
            // Correct state, item added to stack
            assert_eq!(complex_stack.len(), 1);
            let parent_context = &complex_stack[0];
            assert_eq!(parent_context.items.len(), 1);
            let decoded_item = &parent_context.items[0];
            assert_eq!(decoded_item.tag, tag);
            if let HtlvValue::Bytes(bytes) = &decoded_item.value {
                assert_eq!(bytes.as_ref(), &data[0..10][..]); // Compare byte slices
            } else {
                panic!("Decoded value is not Bytes");
            }

        } else {
            panic!("Result is not Incomplete");
        }
         assert_eq!(buffer.len(), 0); // Buffer should be empty after freezing
    }
}