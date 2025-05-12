use crate::internal::error::Result;
use crate::codec::types::HtlvValue;
use crate::codec::decode::complex::parse_complex_items;

/// Decodes an Object HtlvValue from a byte slice.
pub fn decode_object(raw_value_slice: &[u8]) -> Result<HtlvValue> {
    let items = parse_complex_items(raw_value_slice)?;
    // For Object, we might add validation here later (e.g., checking for duplicate tags)
    Ok(HtlvValue::Object(items))
}