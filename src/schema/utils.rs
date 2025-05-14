// Utility functions for the schema module
//
// This module provides shared utility functions used by other schema submodules.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::internal::error::{Error, Result};
use crate::codec::types::{HtlvItem, HtlvValue};
use crate::schema::types::{SchemaType, SchemaField};

/// Generates a tag from a field name
///
/// This function creates a deterministic u64 tag from a field name
/// using a hash function. This is useful when a tag is not explicitly
/// provided in a schema definition.
pub fn generate_tag_from_name(name: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    hasher.finish()
}

/// Checks if a numeric value is within the valid range for a given schema type
///
/// This function is used to validate that numeric values are within the
/// appropriate range for their schema type, preventing overflow or precision loss.
pub fn check_numeric_range(value: f64, schema_type: &SchemaType) -> Result<()> {
    match schema_type {
        SchemaType::UInt8 => {
            if value < 0.0 || value > u8::MAX as f64 {
                return Err(Error::SchemaError(format!(
                    "Value {} is out of range for UInt8", value
                )));
            }
        },
        SchemaType::UInt16 => {
            if value < 0.0 || value > u16::MAX as f64 {
                return Err(Error::SchemaError(format!(
                    "Value {} is out of range for UInt16", value
                )));
            }
        },
        SchemaType::UInt32 => {
            if value < 0.0 || value > u32::MAX as f64 {
                return Err(Error::SchemaError(format!(
                    "Value {} is out of range for UInt32", value
                )));
            }
        },
        SchemaType::UInt64 => {
            if value < 0.0 || value > u64::MAX as f64 {
                return Err(Error::SchemaError(format!(
                    "Value {} is out of range for UInt64", value
                )));
            }
        },
        SchemaType::Int8 => {
            if value < i8::MIN as f64 || value > i8::MAX as f64 {
                return Err(Error::SchemaError(format!(
                    "Value {} is out of range for Int8", value
                )));
            }
        },
        SchemaType::Int16 => {
            if value < i16::MIN as f64 || value > i16::MAX as f64 {
                return Err(Error::SchemaError(format!(
                    "Value {} is out of range for Int16", value
                )));
            }
        },
        SchemaType::Int32 => {
            if value < i32::MIN as f64 || value > i32::MAX as f64 {
                return Err(Error::SchemaError(format!(
                    "Value {} is out of range for Int32", value
                )));
            }
        },
        SchemaType::Int64 => {
            if value < i64::MIN as f64 || value > i64::MAX as f64 {
                return Err(Error::SchemaError(format!(
                    "Value {} is out of range for Int64", value
                )));
            }
        },
        SchemaType::Float32 => {
            if value.abs() > f32::MAX as f64 || (value != 0.0 && value.abs() < f32::MIN_POSITIVE as f64) {
                return Err(Error::SchemaError(format!(
                    "Value {} is out of range for Float32", value
                )));
            }
        },
        _ => {
            // For Float64 or non-numeric types, no range check is needed
        }
    }
    
    Ok(())
}

/// Converts a numeric value to an HTLV value based on the schema type
///
/// This function handles the conversion of numeric values to the appropriate
/// HTLV value type, ensuring proper type conversion and range checking.
pub fn numeric_to_htlv(value: f64, schema_type: &SchemaType) -> Result<HtlvValue> {
    // First check that the value is within range
    check_numeric_range(value, schema_type)?;
    
    // Convert to the appropriate HTLV value type
    match schema_type {
        SchemaType::UInt8 => Ok(HtlvValue::U8(value as u8)),
        SchemaType::UInt16 => Ok(HtlvValue::U16(value as u16)),
        SchemaType::UInt32 => Ok(HtlvValue::U32(value as u32)),
        SchemaType::UInt64 => Ok(HtlvValue::U64(value as u64)),
        SchemaType::Int8 => Ok(HtlvValue::I8(value as i8)),
        SchemaType::Int16 => Ok(HtlvValue::I16(value as i16)),
        SchemaType::Int32 => Ok(HtlvValue::I32(value as i32)),
        SchemaType::Int64 => Ok(HtlvValue::I64(value as i64)),
        SchemaType::Float32 => Ok(HtlvValue::F32(value as f32)),
        SchemaType::Float64 => Ok(HtlvValue::F64(value)),
        _ => Err(Error::SchemaError(format!(
            "Cannot convert numeric value to non-numeric type: {:?}", schema_type
        ))),
    }
}

/// Extracts a field from an object by tag
///
/// This function finds a field in an HTLV object by its tag,
/// returning the field value if found or None if not found.
pub fn find_field_by_tag(object: &[HtlvItem], tag: u64) -> Option<&HtlvValue> {
    object.iter()
        .find(|item| item.tag == tag)
        .map(|item| &item.value)
}

/// Extracts a field from an object by name
///
/// This function finds a field in an HTLV object by its name using the schema fields,
/// returning the field value if found or None if not found.
pub fn find_field_by_name<'a>(
    object: &'a [HtlvItem],
    name: &str,
    fields: &[SchemaField],
) -> Option<&'a HtlvValue> {
    // Find the field definition by name
    let field = fields.iter().find(|f| f.name == name)?;
    
    // Find the field value by tag
    find_field_by_tag(object, field.tag)
}

/// Determines if a schema type is a numeric type
///
/// This function checks if a schema type represents a numeric value
/// (integer or floating point).
pub fn is_numeric_type(schema_type: &SchemaType) -> bool {
    matches!(
        schema_type,
        SchemaType::UInt8 | SchemaType::UInt16 | SchemaType::UInt32 | SchemaType::UInt64 |
        SchemaType::Int8 | SchemaType::Int16 | SchemaType::Int32 | SchemaType::Int64 |
        SchemaType::Float32 | SchemaType::Float64
    )
}

/// Determines if a schema type is an integer type
///
/// This function checks if a schema type represents an integer value
/// (signed or unsigned).
pub fn is_integer_type(schema_type: &SchemaType) -> bool {
    matches!(
        schema_type,
        SchemaType::UInt8 | SchemaType::UInt16 | SchemaType::UInt32 | SchemaType::UInt64 |
        SchemaType::Int8 | SchemaType::Int16 | SchemaType::Int32 | SchemaType::Int64
    )
}

/// Determines if a schema type is a floating point type
///
/// This function checks if a schema type represents a floating point value.
pub fn is_float_type(schema_type: &SchemaType) -> bool {
    matches!(schema_type, SchemaType::Float32 | SchemaType::Float64)
}

/// Determines if a schema type is a complex type
///
/// This function checks if a schema type represents a complex value
/// (array, object, map, or union).
pub fn is_complex_type(schema_type: &SchemaType) -> bool {
    matches!(
        schema_type,
        SchemaType::Array(_) | SchemaType::Object(_) | SchemaType::Map(_, _) | SchemaType::Union(_)
    )
}
