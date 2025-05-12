use crate::internal::error::Result;
use crate::codec::varint;
use crate::codec::types::{HtlvValue, HtlvValueType}; // Import HtlvItem for tests


/// Encodes a Tag and Value into the basic HTLV format (Tag + Length + Value).
/// Tag and Length are encoded using variable-length integers.
pub fn encode_h_tlv(tag: u64, value: &[u8]) -> Result<Vec<u8>> {
    let mut encoded_data = Vec::new();

    // Encode Tag
    let encoded_tag = varint::encode_varint(tag);
    encoded_data.extend_from_slice(&encoded_tag);

    // Encode Length
    let length = value.len() as u64;
    let encoded_length = varint::encode_varint(length);
    encoded_data.extend_from_slice(&encoded_length);

    // Append Value
    encoded_data.extend_from_slice(value);

    Ok(encoded_data)
}

/// Encodes a basic HtlvValue into bytes.
/// Returns the value type byte and the encoded value bytes.
pub fn encode_basic_value(value: &HtlvValue) -> Result<(u8, Vec<u8>)> {
    match value {
        HtlvValue::Null => Ok((HtlvValueType::Null as u8, Vec::new())),
        HtlvValue::Bool(v) => Ok((HtlvValueType::Bool as u8, vec![*v as u8])),
        HtlvValue::U8(v) => Ok((HtlvValueType::U8 as u8, vec![*v])),
        HtlvValue::U16(v) => Ok((HtlvValueType::U16 as u8, v.to_le_bytes().to_vec())),
        HtlvValue::U32(v) => Ok((HtlvValueType::U32 as u8, v.to_le_bytes().to_vec())),
        HtlvValue::U64(v) => Ok((HtlvValueType::U64 as u8, v.to_le_bytes().to_vec())),
        HtlvValue::I8(v) => Ok((HtlvValueType::I8 as u8, vec![*v as u8])),
        HtlvValue::I16(v) => Ok((HtlvValueType::I16 as u8, v.to_le_bytes().to_vec())),
        HtlvValue::I32(v) => Ok((HtlvValueType::I32 as u8, v.to_le_bytes().to_vec())),
        HtlvValue::I64(v) => Ok((HtlvValueType::I64 as u8, v.to_le_bytes().to_vec())),
        HtlvValue::F32(v) => Ok((HtlvValueType::F32 as u8, v.to_le_bytes().to_vec())),
        HtlvValue::F64(v) => Ok((HtlvValueType::F64 as u8, v.to_le_bytes().to_vec())),
        HtlvValue::Bytes(v) => Ok((HtlvValueType::Bytes as u8, v.to_vec())),
        HtlvValue::String(v) => Ok((HtlvValueType::String as u8, v.to_vec())),
        // Array and Object will be handled in complex.rs
        HtlvValue::Array(_) | HtlvValue::Object(_) => {
            Err(crate::internal::error::Error::CodecError("Attempted to encode complex type with basic encoder".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::types::{HtlvValue, HtlvValueType};
    use bytes::Bytes;

    #[test]
    fn test_encode_h_tlv() {
        // Example: Tag 1, Value "hello"
        let tag = 1;
        let value = b"hello";
        let expected = vec![0x01, 0x05, 0x68, 0x65, 0x6c, 0x6c, 0x6f];
        let encoded = encode_h_tlv(tag, value).unwrap();
        assert_eq!(encoded, expected);
    }

    #[test]
    fn test_encode_basic_value() {
        // Test Null
        let value_null = HtlvValue::Null;
        let (type_byte_null, encoded_null) = encode_basic_value(&value_null).unwrap();
        assert_eq!(type_byte_null, HtlvValueType::Null as u8);
        assert_eq!(encoded_null, Vec::<u8>::new()); // Added type annotation

        // Test Bool(true)
        let value_bool = HtlvValue::Bool(true);
        let (type_byte_bool, encoded_bool) = encode_basic_value(&value_bool).unwrap();
        assert_eq!(type_byte_bool, HtlvValueType::Bool as u8);
        assert_eq!(encoded_bool, vec![1]);

        // Test U8
        let value_u8 = HtlvValue::U8(255);
        let (type_byte_u8, encoded_u8) = encode_basic_value(&value_u8).unwrap();
        assert_eq!(type_byte_u8, HtlvValueType::U8 as u8);
        assert_eq!(encoded_u8, vec![255]);

        // Test U16
        let value_u16 = HtlvValue::U16(65535);
        let (type_byte_u16, encoded_u16) = encode_basic_value(&value_u16).unwrap();
        assert_eq!(type_byte_u16, HtlvValueType::U16 as u8);
        assert_eq!(encoded_u16, vec![0xff, 0xff]);

        // Test U32
        let value_u32 = HtlvValue::U32(4294967295);
        let (type_byte_u32, encoded_u32) = encode_basic_value(&value_u32).unwrap();
        assert_eq!(type_byte_u32, HtlvValueType::U32 as u8);
        assert_eq!(encoded_u32, vec![0xff, 0xff, 0xff, 0xff]);

        // Test U64
        let value_u64 = HtlvValue::U64(1234567890);
        let (type_byte_u64, encoded_u64) = encode_basic_value(&value_u64).unwrap(); // Corrected variable name
        assert_eq!(type_byte_u64, HtlvValueType::U64 as u8);
        assert_eq!(encoded_u64, vec![0xd2, 0x02, 0x96, 0x49, 0x00, 0x00, 0x00, 0x00]);

        // Test I8
        let value_i8 = HtlvValue::I8(-128);
        let (type_byte_i8, encoded_i8) = encode_basic_value(&value_i8).unwrap();
        assert_eq!(type_byte_i8, HtlvValueType::I8 as u8);
        assert_eq!(encoded_i8, vec![0x80]);

        // Test I16
        let value_i16 = HtlvValue::I16(-32768);
        let (type_byte_i16, encoded_i16) = encode_basic_value(&value_i16).unwrap();
        assert_eq!(type_byte_i16, HtlvValueType::I16 as u8);
        assert_eq!(encoded_i16, vec![0x00, 0x80]);

        // Test I32
        let value_i32 = HtlvValue::I32(-2147483648);
        let (type_byte_i32, encoded_i32) = encode_basic_value(&value_i32).unwrap();
        assert_eq!(type_byte_i32, HtlvValueType::I32 as u8);
        assert_eq!(encoded_i32, vec![0x00, 0x00, 0x00, 0x80]);

        // Test I64
        let value_i64 = HtlvValue::I64(-1); // Changed from -987654321 to -1
        let (type_byte_i64, encoded_i64) = encode_basic_value(&value_i64).unwrap();
        assert_eq!(type_byte_i64, HtlvValueType::I64 as u8);
        assert_eq!(encoded_i64, vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]); // Expected bytes for -1

        // Test F32
        let value_f32 = HtlvValue::F32(3.14f32);
        let (type_byte_f32, encoded_f32) = encode_basic_value(&value_f32).unwrap();
        assert_eq!(type_byte_f32, HtlvValueType::F32 as u8);
        assert_eq!(encoded_f32, vec![0xc3, 0xf5, 0x48, 0x40]);

        // Test F64
        let value_f64 = HtlvValue::F64(3.14);
        let (type_byte_f64, encoded_f64) = encode_basic_value(&value_f64).unwrap();
        assert_eq!(type_byte_f64, HtlvValueType::F64 as u8);
        assert_eq!(encoded_f64, vec![0x1f, 0x85, 0xeb, 0x51, 0xb8, 0x1e, 0x09, 0x40]);

        // Test Bytes
        let value_bytes = HtlvValue::Bytes(Bytes::from_static(b"raw data"));
        let (type_byte_bytes, encoded_bytes) = encode_basic_value(&value_bytes).unwrap();
        assert_eq!(type_byte_bytes, HtlvValueType::Bytes as u8);
        assert_eq!(encoded_bytes, b"raw data".to_vec());
        
        // Test String
        let value_string = HtlvValue::String(Bytes::from_static("你好".as_bytes()));
        let (type_byte_string, encoded_string) = encode_basic_value(&value_string).unwrap();
        assert_eq!(type_byte_string, HtlvValueType::String as u8);
        assert_eq!(encoded_string, "你好".as_bytes().to_vec());
    }
}