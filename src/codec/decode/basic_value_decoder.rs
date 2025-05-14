// Decoder for basic HTLV values

use crate::internal::error::{Error, Result};
use crate::codec::types::{HtlvValueType, HtlvValue};
use crate::codec::decode::basic_types; // Import basic_types module

/// Decodes a basic HTLV value based on its type, length, and raw data.
/// This function encapsulates the decoding logic for non-batch basic types.
pub fn decode_basic_value(
    value_type: HtlvValueType,
    length: u64,
    raw_value_slice: &[u8],
) -> Result<HtlvValue> {
    match value_type {
        HtlvValueType::Null => basic_types::null::decode_null(length),
        HtlvValueType::Bool => basic_types::boolean::decode_bool(length, raw_value_slice),
        HtlvValueType::U8 => basic_types::u8::decode_u8(length, raw_value_slice),
        HtlvValueType::U16 => basic_types::u16::decode_u16(length, raw_value_slice), // Removed .map(HtlvValue::U16)
        HtlvValueType::U32 => basic_types::u32::decode_u32(length, raw_value_slice), // Removed .map(HtlvValue::U32)
        HtlvValueType::U64 => basic_types::u64::decode_u64(length, raw_value_slice), // Removed .map(HtlvValue::U64)
        HtlvValueType::I8 => basic_types::i8::decode_i8(length, raw_value_slice), // Removed .map(HtlvValue::I8)
        HtlvValueType::I16 => basic_types::i16::decode_i16(length, raw_value_slice), // Removed .map(HtlvValue::I16)
        HtlvValueType::I32 => basic_types::i32::decode_i32(length, raw_value_slice), // Removed .map(HtlvValue::I32)
        HtlvValueType::I64 => basic_types::i64::decode_i64(length, raw_value_slice), // Removed .map(HtlvValue::I64)
        HtlvValueType::F32 => basic_types::floats::decode_f32(length, raw_value_slice), // Removed .map(HtlvValue::F32)
        HtlvValueType::F64 => basic_types::floats::decode_f64(length, raw_value_slice), // Removed .map(HtlvValue::F64)
        HtlvValueType::Bytes => basic_types::bytes_and_string::decode_bytes(raw_value_slice),
        HtlvValueType::String => basic_types::bytes_and_string::decode_string(raw_value_slice),
        // Complex types should not be handled here
        HtlvValueType::Array | HtlvValueType::Object => Err(Error::CodecError(format!(
            "Unexpected complex type ({:?}) in decode_basic_value",
            value_type
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::types::HtlvItem;
    use bytes::Bytes;

    #[test]
    fn test_decode_basic_value() {
        // Test Null
        // Corrected null_data to include Type (0x00) and Length (0x00)
        let null_data = Bytes::from_static(&[0x00, HtlvValueType::Null as u8, 0x00]); // Tag 0, Type Null, Length 0
        let (_, tag_bytes) = crate::codec::varint::decode_varint(&null_data).unwrap();
        let value_type_byte = null_data[tag_bytes];
        let (_, length_bytes) = crate::codec::varint::decode_varint(&null_data[tag_bytes + 1..]).unwrap();
        let value_start = tag_bytes + 1 + length_bytes;
        let value_type = HtlvValueType::from_byte(value_type_byte).unwrap();
        let length = crate::codec::varint::decode_varint(&null_data[tag_bytes + 1..]).unwrap().0;
        let raw_value_slice = &null_data[value_start..value_start + length as usize];
        let decoded_null = decode_basic_value(value_type, length, raw_value_slice).unwrap();
        assert_eq!(decoded_null, HtlvValue::Null);

        // Test Bool (true)
        let bool_true_data = Bytes::from_static(&[0x00, HtlvValueType::Bool as u8, 0x01, 0x01]); // Tag 0, Type Bool, Length 1, Value 1
        let (_, tag_bytes) = crate::codec::varint::decode_varint(&bool_true_data).unwrap();
        let value_type_byte = bool_true_data[tag_bytes];
        let (_, length_bytes) = crate::codec::varint::decode_varint(&bool_true_data[tag_bytes + 1..]).unwrap();
        let value_start = tag_bytes + 1 + length_bytes;
        let value_type = HtlvValueType::from_byte(value_type_byte).unwrap();
        let length = crate::codec::varint::decode_varint(&bool_true_data[tag_bytes + 1..]).unwrap().0;
        let raw_value_slice = &bool_true_data[value_start..value_start + length as usize];
        let decoded_bool_true = decode_basic_value(value_type, length, raw_value_slice).unwrap();
        assert_eq!(decoded_bool_true, HtlvValue::Bool(true));

        // Test U8
        let u8_data = Bytes::from_static(&[0x00, HtlvValueType::U8 as u8, 0x01, 0x42]); // Tag 0, Type U8, Length 1, Value 66
        let (_, tag_bytes) = crate::codec::varint::decode_varint(&u8_data).unwrap();
        let value_type_byte = u8_data[tag_bytes];
        let (_, length_bytes) = crate::codec::varint::decode_varint(&u8_data[tag_bytes + 1..]).unwrap();
        let value_start = tag_bytes + 1 + length_bytes;
        let value_type = HtlvValueType::from_byte(value_type_byte).unwrap();
        let length = crate::codec::varint::decode_varint(&u8_data[tag_bytes + 1..]).unwrap().0;
        let raw_value_slice = &u8_data[value_start..value_start + length as usize];
        let decoded_u8 = decode_basic_value(value_type, length, raw_value_slice).unwrap();
        assert_eq!(decoded_u8, HtlvValue::U8(66));


        // Test Bytes
        let bytes_data = Bytes::from_static(&[0x00, HtlvValueType::Bytes as u8, 0x03, 0x01, 0x02, 0x03]); // Tag 0, Type Bytes, Length 3, Value [1, 2, 3]
        let (_, tag_bytes) = crate::codec::varint::decode_varint(&bytes_data).unwrap();
        let value_type_byte = bytes_data[tag_bytes];
        let (_, length_bytes) = crate::codec::varint::decode_varint(&bytes_data[tag_bytes + 1..]).unwrap();
        let value_start = tag_bytes + 1 + length_bytes;
        let value_type = HtlvValueType::from_byte(value_type_byte).unwrap();
        let length = crate::codec::varint::decode_varint(&bytes_data[tag_bytes + 1..]).unwrap().0;
        let raw_value_slice = &bytes_data[value_start..value_start + length as usize];
        let decoded_bytes = decode_basic_value(value_type, length, raw_value_slice).unwrap();
        assert_eq!(decoded_bytes, HtlvValue::Bytes(Bytes::from_static(&[0x01, 0x02, 0x03])));

        // Test String
        let string_data = Bytes::from_static(&[0x00, HtlvValueType::String as u8, 0x05, b'h', b'e', b'l', b'l', b'o']); // Tag 0, Type String, Length 5, Value "hello"
        let (_, tag_bytes) = crate::codec::varint::decode_varint(&string_data).unwrap();
        let value_type_byte = string_data[tag_bytes];
        let (_, length_bytes) = crate::codec::varint::decode_varint(&string_data[tag_bytes + 1..]).unwrap();
        let value_start = tag_bytes + 1 + length_bytes;
        let value_type = HtlvValueType::from_byte(value_type_byte).unwrap();
        let length = crate::codec::varint::decode_varint(&string_data[tag_bytes + 1..]).unwrap().0;
        let raw_value_slice = &string_data[value_start..value_start + length as usize];
        let decoded_string = decode_basic_value(value_type, length, raw_value_slice).unwrap();
        assert_eq!(decoded_string, HtlvValue::String(Bytes::from_static(b"hello")));
    }

    #[test]
    fn test_decode_basic_value_errors() {
        // Test incorrect length for Null
        let result = decode_basic_value(HtlvValueType::Null, 1, &[0x00]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Null value must have length 0, but got 1"
        );

        // Test incorrect length for Bool
        let result = decode_basic_value(HtlvValueType::Bool, 0, &[]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Bool value must have length 1, but got 0"
        );

        // Test invalid bool value
        let result = decode_basic_value(HtlvValueType::Bool, 1, &[0x02]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Invalid boolean value byte: 2"
        );

        // Test invalid UTF-8 string
        let result = decode_basic_value(HtlvValueType::String, 4, &[0xff, 0xff, 0xff, 0xff]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Invalid UTF-8 sequence"
        );

        // Test unexpected type
        let result = decode_basic_value(HtlvValueType::Array, 0, &[]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Codec Error: Unexpected complex type (Array) in decode_basic_value"
        );
    }
}