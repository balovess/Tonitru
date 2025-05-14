// Schema to HTLV structure mapping rules
//
// This module defines the mapping between Schema types and HTLV structures,
// providing conversion functions in both directions.

use std::collections::HashMap;

use crate::internal::error::{Error, Result};
use crate::codec::types::{HtlvItem, HtlvValue, HtlvValueType};
use crate::schema::types::{Schema, SchemaType, SchemaField};
use crate::schema::defaults::DefaultValueStrategy;

/// Configuration for schema mapping
#[derive(Debug, Clone)]
pub struct MapperConfig {
    /// Default value strategy to use when creating HTLV values from schema
    pub default_value_strategy: DefaultValueStrategy,
    
    /// Whether to validate values during mapping
    pub validate: bool,
    
    /// Whether to preserve unknown fields when mapping from HTLV to schema
    pub preserve_unknown_fields: bool,
    
    /// Custom type mappings (schema type name -> HTLV value type)
    pub custom_type_mappings: HashMap<String, HtlvValueType>,
}

impl Default for MapperConfig {
    fn default() -> Self {
        Self {
            default_value_strategy: DefaultValueStrategy::RequiredOnly,
            validate: true,
            preserve_unknown_fields: false,
            custom_type_mappings: HashMap::new(),
        }
    }
}

/// Schema mapper for converting between Schema and HTLV structures
#[derive(Debug, Clone)]
pub struct SchemaMapper {
    config: MapperConfig,
}

impl SchemaMapper {
    /// Creates a new schema mapper with default configuration
    pub fn new() -> Self {
        Self {
            config: MapperConfig::default(),
        }
    }
    
    /// Creates a new schema mapper with custom configuration
    pub fn with_config(config: MapperConfig) -> Self {
        Self { config }
    }
    
    /// Maps a schema type to an HTLV value type
    pub fn schema_type_to_htlv_type(&self, schema_type: &SchemaType) -> HtlvValueType {
        match schema_type {
            SchemaType::Null => HtlvValueType::Null,
            SchemaType::Boolean => HtlvValueType::Bool,
            SchemaType::UInt8 => HtlvValueType::U8,
            SchemaType::UInt16 => HtlvValueType::U16,
            SchemaType::UInt32 => HtlvValueType::U32,
            SchemaType::UInt64 => HtlvValueType::U64,
            SchemaType::Int8 => HtlvValueType::I8,
            SchemaType::Int16 => HtlvValueType::I16,
            SchemaType::Int32 => HtlvValueType::I32,
            SchemaType::Int64 => HtlvValueType::I64,
            SchemaType::Float32 => HtlvValueType::F32,
            SchemaType::Float64 => HtlvValueType::F64,
            SchemaType::Binary => HtlvValueType::Bytes,
            SchemaType::String => HtlvValueType::String,
            SchemaType::Array(_) => HtlvValueType::Array,
            SchemaType::Object(_) => HtlvValueType::Object,
            SchemaType::Map(_, _) => HtlvValueType::Object, // Maps are represented as objects in HTLV
            SchemaType::Union(_) => {
                // For unions, we can't determine the HTLV type without a value
                // Default to Object as it's the most flexible
                HtlvValueType::Object
            }
        }
    }
    
    /// Creates an HTLV value from a schema type and optional value
    pub fn create_htlv_value(
        &self,
        schema_type: &SchemaType,
        value: Option<serde_json::Value>,
    ) -> Result<HtlvValue> {
        match value {
            Some(val) => self.json_to_htlv(schema_type, &val),
            None => self.config.default_value_strategy.apply_defaults(schema_type, None),
        }
    }
    
    /// Converts a JSON value to an HTLV value based on the schema type
    pub fn json_to_htlv(
        &self,
        schema_type: &SchemaType,
        json: &serde_json::Value,
    ) -> Result<HtlvValue> {
        match (schema_type, json) {
            // Null type
            (SchemaType::Null, serde_json::Value::Null) => Ok(HtlvValue::Null),
            
            // Boolean type
            (SchemaType::Boolean, serde_json::Value::Bool(b)) => Ok(HtlvValue::Bool(*b)),
            
            // Number types
            (SchemaType::UInt8, serde_json::Value::Number(n)) => {
                if let Some(u) = n.as_u64() {
                    if u <= u8::MAX as u64 {
                        Ok(HtlvValue::U8(u as u8))
                    } else {
                        Err(Error::SchemaError(format!("Value {} is too large for UInt8", u)))
                    }
                } else {
                    Err(Error::SchemaError(format!("Cannot convert {} to UInt8", n)))
                }
            },
            (SchemaType::UInt16, serde_json::Value::Number(n)) => {
                if let Some(u) = n.as_u64() {
                    if u <= u16::MAX as u64 {
                        Ok(HtlvValue::U16(u as u16))
                    } else {
                        Err(Error::SchemaError(format!("Value {} is too large for UInt16", u)))
                    }
                } else {
                    Err(Error::SchemaError(format!("Cannot convert {} to UInt16", n)))
                }
            },
            (SchemaType::UInt32, serde_json::Value::Number(n)) => {
                if let Some(u) = n.as_u64() {
                    if u <= u32::MAX as u64 {
                        Ok(HtlvValue::U32(u as u32))
                    } else {
                        Err(Error::SchemaError(format!("Value {} is too large for UInt32", u)))
                    }
                } else {
                    Err(Error::SchemaError(format!("Cannot convert {} to UInt32", n)))
                }
            },
            (SchemaType::UInt64, serde_json::Value::Number(n)) => {
                if let Some(u) = n.as_u64() {
                    Ok(HtlvValue::U64(u))
                } else {
                    Err(Error::SchemaError(format!("Cannot convert {} to UInt64", n)))
                }
            },
            (SchemaType::Int8, serde_json::Value::Number(n)) => {
                if let Some(i) = n.as_i64() {
                    if i >= i8::MIN as i64 && i <= i8::MAX as i64 {
                        Ok(HtlvValue::I8(i as i8))
                    } else {
                        Err(Error::SchemaError(format!("Value {} is out of range for Int8", i)))
                    }
                } else {
                    Err(Error::SchemaError(format!("Cannot convert {} to Int8", n)))
                }
            },
            (SchemaType::Int16, serde_json::Value::Number(n)) => {
                if let Some(i) = n.as_i64() {
                    if i >= i16::MIN as i64 && i <= i16::MAX as i64 {
                        Ok(HtlvValue::I16(i as i16))
                    } else {
                        Err(Error::SchemaError(format!("Value {} is out of range for Int16", i)))
                    }
                } else {
                    Err(Error::SchemaError(format!("Cannot convert {} to Int16", n)))
                }
            },
            (SchemaType::Int32, serde_json::Value::Number(n)) => {
                if let Some(i) = n.as_i64() {
                    if i >= i32::MIN as i64 && i <= i32::MAX as i64 {
                        Ok(HtlvValue::I32(i as i32))
                    } else {
                        Err(Error::SchemaError(format!("Value {} is out of range for Int32", i)))
                    }
                } else {
                    Err(Error::SchemaError(format!("Cannot convert {} to Int32", n)))
                }
            },
            (SchemaType::Int64, serde_json::Value::Number(n)) => {
                if let Some(i) = n.as_i64() {
                    Ok(HtlvValue::I64(i))
                } else {
                    Err(Error::SchemaError(format!("Cannot convert {} to Int64", n)))
                }
            },
            (SchemaType::Float32, serde_json::Value::Number(n)) => {
                if let Some(f) = n.as_f64() {
                    // Check if the value is within the range of f32
                    if f.abs() <= f32::MAX as f64 && f.abs() >= f32::MIN_POSITIVE as f64 {
                        Ok(HtlvValue::F32(f as f32))
                    } else {
                        Err(Error::SchemaError(format!("Value {} is out of range for Float32", f)))
                    }
                } else {
                    Err(Error::SchemaError(format!("Cannot convert {} to Float32", n)))
                }
            },
            (SchemaType::Float64, serde_json::Value::Number(n)) => {
                if let Some(f) = n.as_f64() {
                    Ok(HtlvValue::F64(f))
                } else {
                    Err(Error::SchemaError(format!("Cannot convert {} to Float64", n)))
                }
            },
            
            // String and binary types
            (SchemaType::String, serde_json::Value::String(s)) => {
                Ok(HtlvValue::String(bytes::Bytes::from(s.clone())))
            },
            (SchemaType::Binary, serde_json::Value::String(s)) => {
                // Assume base64 encoding for binary data in JSON
                match base64::decode(s) {
                    Ok(bytes) => Ok(HtlvValue::Bytes(bytes::Bytes::from(bytes))),
                    Err(e) => Err(Error::SchemaError(format!("Invalid base64 data: {}", e))),
                }
            },
            
            // Array type
            (SchemaType::Array(elem_type), serde_json::Value::Array(arr)) => {
                let mut items = Vec::with_capacity(arr.len());
                for (i, item) in arr.iter().enumerate() {
                    let value = self.json_to_htlv(elem_type, item)?;
                    items.push(HtlvItem {
                        tag: i as u64, // Use array index as tag
                        value,
                    });
                }
                Ok(HtlvValue::Array(items))
            },
            
            // Object type
            (SchemaType::Object(fields), serde_json::Value::Object(obj)) => {
                let mut items = Vec::new();
                
                // Create a map of field names to field definitions for quick lookup
                let field_map: HashMap<&str, &SchemaField> = fields
                    .iter()
                    .map(|field| (field.name.as_str(), field))
                    .collect();
                
                // Convert each field in the JSON object
                for (key, value) in obj {
                    if let Some(field) = field_map.get(key.as_str()) {
                        let htlv_value = self.json_to_htlv(&field.field_type, value)?;
                        items.push(HtlvItem {
                            tag: field.tag,
                            value: htlv_value,
                        });
                    } else if self.config.preserve_unknown_fields {
                        // For unknown fields, try to infer the type
                        let inferred_type = self.infer_schema_type(value);
                        let htlv_value = self.json_to_htlv(&inferred_type, value)?;
                        
                        // Use a hash of the field name as the tag for unknown fields
                        use std::hash::{Hash, Hasher};
                        let mut hasher = std::collections::hash_map::DefaultHasher::new();
                        key.hash(&mut hasher);
                        let tag = hasher.finish();
                        
                        items.push(HtlvItem {
                            tag,
                            value: htlv_value,
                        });
                    }
                }
                
                // Add default values for missing required fields
                for field in fields {
                    if field.required && !obj.contains_key(&field.name) {
                        if let Some(default) = &field.default_value {
                            items.push(HtlvItem {
                                tag: field.tag,
                                value: default.clone(),
                            });
                        } else {
                            // Create default value for required field
                            let default_value = self.config.default_value_strategy.apply_defaults(&field.field_type, None)?;
                            items.push(HtlvItem {
                                tag: field.tag,
                                value: default_value,
                            });
                        }
                    }
                }
                
                Ok(HtlvValue::Object(items))
            },
            
            // Union type
            (SchemaType::Union(types), json) => {
                // Try each possible type in the union
                for t in types {
                    if let Ok(value) = self.json_to_htlv(t, json) {
                        return Ok(value);
                    }
                }
                
                // No matching type found
                Err(Error::SchemaError(format!(
                    "JSON value does not match any type in union: {:?}", json
                )))
            },
            
            // Map type
            (SchemaType::Map(key_type, value_type), serde_json::Value::Object(obj)) => {
                let mut items = Vec::new();
                
                for (key, value) in obj {
                    // Convert the key to an HTLV value
                    let key_json = serde_json::Value::String(key.clone());
                    let key_value = self.json_to_htlv(key_type, &key_json)?;
                    
                    // Convert the value to an HTLV value
                    let value_htlv = self.json_to_htlv(value_type, value)?;
                    
                    // Use a hash of the key as the tag
                    use std::hash::{Hash, Hasher};
                    let mut hasher = std::collections::hash_map::DefaultHasher::new();
                    key.hash(&mut hasher);
                    let tag = hasher.finish();
                    
                    // Create a map entry as an object with key and value fields
                    let entry = HtlvValue::Object(vec![
                        HtlvItem { tag: 0, value: key_value },
                        HtlvItem { tag: 1, value: value_htlv },
                    ]);
                    
                    items.push(HtlvItem { tag, value: entry });
                }
                
                Ok(HtlvValue::Object(items))
            },
            
            // Type mismatch
            (expected, actual) => Err(Error::SchemaError(format!(
                "Type mismatch: expected {:?}, got {:?}", expected, actual
            ))),
        }
    }
    
    /// Infers a schema type from a JSON value
    fn infer_schema_type(&self, json: &serde_json::Value) -> SchemaType {
        match json {
            serde_json::Value::Null => SchemaType::Null,
            serde_json::Value::Bool(_) => SchemaType::Boolean,
            serde_json::Value::Number(n) => {
                if n.is_i64() {
                    let i = n.as_i64().unwrap();
                    if i >= i8::MIN as i64 && i <= i8::MAX as i64 {
                        SchemaType::Int8
                    } else if i >= i16::MIN as i64 && i <= i16::MAX as i64 {
                        SchemaType::Int16
                    } else if i >= i32::MIN as i64 && i <= i32::MAX as i64 {
                        SchemaType::Int32
                    } else {
                        SchemaType::Int64
                    }
                } else if n.is_u64() {
                    let u = n.as_u64().unwrap();
                    if u <= u8::MAX as u64 {
                        SchemaType::UInt8
                    } else if u <= u16::MAX as u64 {
                        SchemaType::UInt16
                    } else if u <= u32::MAX as u64 {
                        SchemaType::UInt32
                    } else {
                        SchemaType::UInt64
                    }
                } else {
                    // It's a floating point number
                    SchemaType::Float64
                }
            },
            serde_json::Value::String(_) => SchemaType::String,
            serde_json::Value::Array(arr) => {
                if arr.is_empty() {
                    // Empty array, use a generic type
                    SchemaType::Array(Box::new(SchemaType::Null))
                } else {
                    // Infer type from the first element
                    let elem_type = self.infer_schema_type(&arr[0]);
                    SchemaType::Array(Box::new(elem_type))
                }
            },
            serde_json::Value::Object(_) => SchemaType::Object(Vec::new()), // Empty field list for inferred objects
        }
    }
}
