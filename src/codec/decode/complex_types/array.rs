use crate::internal::error::Result;
use crate::codec::types::HtlvValue;
use crate::codec::decode::complex::parse_complex_items;

/// Decodes an Array HtlvValue from a byte slice.
pub fn decode_array(raw_value_slice: &[u8]) -> Result<HtlvValue> {
    let items = parse_complex_items(raw_value_slice)?;
    Ok(HtlvValue::Array(items))
}