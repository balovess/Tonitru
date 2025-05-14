// Batch processing functions for the pipeline processor

use crate::internal::error::{Error, Result};
use crate::codec::types::{HtlvValueType, HtlvValue, HtlvItem};

// Import PipelineProcessor trait
use super::PipelineProcessor;

/// Process batch values using the pipeline processor
///
/// This function selects the appropriate pipeline processor based on the element type
/// and processes the raw data through the four-stage pipeline:
/// 1. Prefetch: Prepare data for efficient processing
/// 2. Decode: Convert raw bytes to typed values
/// 3. Dispatch: Process decoded values
/// 4. Verify: Validate decoded data
///
/// Returns an HtlvValue::Array containing the decoded values
pub fn process_batch_value(
    element_type: HtlvValueType,
    _length: u64,
    raw_value_slice: &[u8],
) -> Result<HtlvValue> {
    let (htlv_values, _) = match element_type {
        HtlvValueType::U8 => u8::process_pipeline(raw_value_slice)?,
        HtlvValueType::U16 => u16::process_pipeline(raw_value_slice)?,
        HtlvValueType::U32 => u32::process_pipeline(raw_value_slice)?,
        HtlvValueType::U64 => u64::process_pipeline(raw_value_slice)?,
        HtlvValueType::I8 => i8::process_pipeline(raw_value_slice)?,
        HtlvValueType::I16 => i16::process_pipeline(raw_value_slice)?,
        HtlvValueType::I32 => i32::process_pipeline(raw_value_slice)?,
        HtlvValueType::I64 => i64::process_pipeline(raw_value_slice)?,
        HtlvValueType::F32 => f32::process_pipeline(raw_value_slice)?,
        HtlvValueType::F64 => f64::process_pipeline(raw_value_slice)?,
        _ => return Err(Error::CodecError(format!("Unsupported type for batch processing: {:?}", element_type))),
    };

    // Convert to HtlvItems and wrap in an Array
    let items: Vec<HtlvItem> = htlv_values.into_iter()
        .map(|v| HtlvItem::new(0, v))
        .collect();

    Ok(HtlvValue::Array(items))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn test_u8_pipeline_processor() {
        let test_data = vec![1u8, 2, 3, 4, 5];
        let result = process_batch_value(HtlvValueType::U8, test_data.len() as u64, &test_data).unwrap();

        if let HtlvValue::Array(items) = result {
            assert_eq!(items.len(), 5);
            assert_eq!(items[0].value, HtlvValue::U8(1));
            assert_eq!(items[1].value, HtlvValue::U8(2));
            assert_eq!(items[2].value, HtlvValue::U8(3));
            assert_eq!(items[3].value, HtlvValue::U8(4));
            assert_eq!(items[4].value, HtlvValue::U8(5));
        } else {
            panic!("Expected Array, got {:?}", result);
        }
    }

    #[test]
    fn test_i32_pipeline_processor() {
        // Create test data with i32 values
        let values = vec![100i32, 200, 300, 400];
        let mut buffer = BytesMut::new();

        // Manually encode the values
        for value in &values {
            buffer.extend_from_slice(&value.to_le_bytes());
        }

        let data = buffer.freeze();

        let result = process_batch_value(HtlvValueType::I32, data.len() as u64, &data).unwrap();

        if let HtlvValue::Array(items) = result {
            assert_eq!(items.len(), 4);
            assert_eq!(items[0].value, HtlvValue::I32(100));
            assert_eq!(items[1].value, HtlvValue::I32(200));
            assert_eq!(items[2].value, HtlvValue::I32(300));
            assert_eq!(items[3].value, HtlvValue::I32(400));
        } else {
            panic!("Expected Array, got {:?}", result);
        }
    }

    #[test]
    fn test_f32_pipeline_processor() {
        // Create test data with f32 values
        let values = vec![1.0f32, 2.5, 3.75, 4.125];
        let mut buffer = BytesMut::new();

        // Manually encode the values
        for value in &values {
            buffer.extend_from_slice(&value.to_le_bytes());
        }

        let data = buffer.freeze();

        let result = process_batch_value(HtlvValueType::F32, data.len() as u64, &data).unwrap();

        if let HtlvValue::Array(items) = result {
            assert_eq!(items.len(), 4);
            assert_eq!(items[0].value, HtlvValue::F32(1.0));
            assert_eq!(items[1].value, HtlvValue::F32(2.5));
            assert_eq!(items[2].value, HtlvValue::F32(3.75));
            assert_eq!(items[3].value, HtlvValue::F32(4.125));
        } else {
            panic!("Expected Array, got {:?}", result);
        }
    }

    #[test]
    fn test_unaligned_data() {
        // Create unaligned data by adding a single byte at the beginning
        let mut buffer = BytesMut::new();
        buffer.extend_from_slice(&[0]); // Unaligned byte

        // Add i32 values
        let values = vec![100i32, 200, 300];
        for value in &values {
            buffer.extend_from_slice(&value.to_le_bytes());
        }

        let data = buffer.freeze();

        // Skip the first byte to create unaligned data
        let unaligned_data = &data[1..];

        let result = process_batch_value(HtlvValueType::I32, unaligned_data.len() as u64, unaligned_data).unwrap();

        if let HtlvValue::Array(items) = result {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0].value, HtlvValue::I32(100));
            assert_eq!(items[1].value, HtlvValue::I32(200));
            assert_eq!(items[2].value, HtlvValue::I32(300));
        } else {
            panic!("Expected Array, got {:?}", result);
        }
    }
}
