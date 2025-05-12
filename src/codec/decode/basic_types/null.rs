use crate::codec::types::HtlvValue;
use crate::internal::error::{Error, Result};

/// Decodes a Null HtlvValue from bytes.
pub fn decode_null(length: u64) -> Result<HtlvValue> {
    if length != 0 {
        return Err(Error::CodecError(format!(
            "Invalid length for Null value: {}",
            length
        )));
    }
    Ok(HtlvValue::Null)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_null() {
        let decoded_null = decode_null(0).unwrap();
        assert_eq!(decoded_null, HtlvValue::Null);
    }

    #[test]
    fn test_decode_null_errors() {
        // Invalid length for Null (expected 0)
        let result = decode_null(1);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Invalid length for Null value: 1"
        );
    }
}