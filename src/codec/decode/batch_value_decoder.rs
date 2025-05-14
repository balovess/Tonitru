// Decoder for batch HTLV values

use crate::internal::error::Result;
use crate::codec::types::{HtlvValueType, HtlvValue};
use crate::codec::decode::pipeline_processor;

/// Decodes a batch of HTLV values based on the element type, total length, and raw data.
/// This function encapsulates the decoding logic for batch decodable basic types.
///
/// This implementation uses a four-stage pipeline:
/// 1. Prefetch: Prepare data for efficient processing
/// 2. Decode: Convert raw bytes to typed values
/// 3. Dispatch: Process decoded values
/// 4. Verify: Validate decoded data
///
/// The pipeline provides:
/// - Improved memory alignment handling
/// - SIMD acceleration when available
/// - Better error detection and reporting
/// - More efficient memory usage
pub fn decode_batch_value(
    element_type: HtlvValueType,
    length: u64,
    raw_value_slice: &[u8],
) -> Result<HtlvValue> {
    // Use the pipeline processor to handle the batch decoding
    pipeline_processor::process_batch_value(element_type, length, raw_value_slice)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut; // Import BytesMut for tests
    use crate::codec::types::HtlvItem;

    #[test]
    fn test_decode_batch_value_u8() {
        let items_to_encode = vec![
            HtlvItem::new(0, HtlvValue::U8(1)),
            HtlvItem::new(0, HtlvValue::U8(2)),
            HtlvItem::new(0, HtlvValue::U8(3)),
        ];
        // Manually encode the batch value (just the raw bytes of the U8 values)
        let mut raw_batch_data = BytesMut::new();
        for item in &items_to_encode {
            if let HtlvValue::U8(v) = item.value {
                raw_batch_data.extend_from_slice(&[v]);
            }
        }
        let raw_batch_slice = raw_batch_data.freeze();

        let decoded_value = decode_batch_value(HtlvValueType::U8, raw_batch_slice.len() as u64, &raw_batch_slice).unwrap();

        if let HtlvValue::Array(decoded_items) = decoded_value {
            assert_eq!(decoded_items.len(), items_to_encode.len());
            for (decoded_item, expected_item) in decoded_items.iter().zip(items_to_encode.iter()) {
                assert_eq!(decoded_item.value, expected_item.value);
            }
        } else {
            panic!("Decoded value is not an Array");
        }
    }

    #[test]
    fn test_decode_batch_value_u32() {
        let items_to_encode = vec![
            HtlvItem::new(0, HtlvValue::U32(100)),
            HtlvItem::new(0, HtlvValue::U32(200)),
            HtlvItem::new(0, HtlvValue::U32(300)),
        ];
         // Manually encode the batch value (just the raw bytes of the U32 values)
        let mut raw_batch_data = BytesMut::new();
        for item in &items_to_encode {
            if let HtlvValue::U32(v) = item.value {
                raw_batch_data.extend_from_slice(&v.to_le_bytes()); // Use little-endian for consistency
            }
        }
        let raw_batch_slice = raw_batch_data.freeze();

        let decoded_value = decode_batch_value(HtlvValueType::U32, raw_batch_slice.len() as u64, &raw_batch_slice).unwrap();

        if let HtlvValue::Array(decoded_items) = decoded_value {
            assert_eq!(decoded_items.len(), items_to_encode.len());
            for (decoded_item, expected_item) in decoded_items.iter().zip(items_to_encode.iter()) {
                assert_eq!(decoded_item.value, expected_item.value);
            }
        } else {
            panic!("Decoded value is not an Array");
        }
    }

    // Add more tests for other batch decodable types (I8, I16, I32, I64, F32, F64)
}