// Encode module for HTLV (HyperNova) data format

pub mod basic;
pub mod complex;

use crate::internal::error::Result;
use crate::codec::varint;
use crate::codec::types::HtlvItem;

/// Encodes an HtlvItem into bytes (Tag + Type + Length + Value).
pub fn encode_item(item: &HtlvItem) -> Result<Vec<u8>> {
    let mut encoded_item = Vec::new();

    // Encode Tag (Variable-length)
    encoded_item.extend_from_slice(&varint::encode_varint(item.tag));

    // Encode Type (1 byte) and Value
    let (value_type_byte, encoded_value) = match &item.value {
        // Basic types handled by basic encoder
        crate::codec::types::HtlvValue::Null |
        crate::codec::types::HtlvValue::Bool(_) |
        crate::codec::types::HtlvValue::U8(_) |
        crate::codec::types::HtlvValue::U16(_) |
        crate::codec::types::HtlvValue::U32(_) |
        crate::codec::types::HtlvValue::U64(_) |
        crate::codec::types::HtlvValue::I8(_) |
        crate::codec::types::HtlvValue::I16(_) |
        crate::codec::types::HtlvValue::I32(_) |
        crate::codec::types::HtlvValue::I64(_) |
        crate::codec::types::HtlvValue::F32(_) |
        crate::codec::types::HtlvValue::F64(_) |
        crate::codec::types::HtlvValue::Bytes(_) |
        crate::codec::types::HtlvValue::String(_) => {
            basic::encode_basic_value(&item.value)?
        }
        // Complex types handled by complex encoder
        crate::codec::types::HtlvValue::Array(_) |
        crate::codec::types::HtlvValue::Object(_) => {
            complex::encode_complex_value(&item.value)?
        }
    };
    encoded_item.push(value_type_byte);

    // Encode Length (Variable-length)
    let length = encoded_value.len() as u64;
    encoded_item.extend_from_slice(&varint::encode_varint(length));

    // Append Value
    encoded_item.extend_from_slice(&encoded_value);

    Ok(encoded_item)
}

// Re-export encode_h_tlv from basic for now, if it's intended to be public
pub use basic::encode_h_tlv;

// TODO: Add more encoding related functions and structures