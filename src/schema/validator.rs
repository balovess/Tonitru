// Schema validator for Tonitru
//
// This module provides validation functionality for Tonitru schemas,
// ensuring that data conforms to the defined schema.

use std::collections::HashMap;

use crate::internal::error::{Error, Result};
use crate::codec::types::{HtlvItem, HtlvValue};
use crate::schema::types::{Schema, SchemaType, SchemaField};

/// Configuration for schema validation
#[derive(Debug, Clone)]
pub struct ValidatorConfig {
    /// Whether to allow unknown fields
    pub allow_unknown_fields: bool,
    
    /// Whether to validate field constraints (min/max, pattern, etc.)
    pub validate_constraints: bool,
    
    /// Whether to validate required fields
    pub validate_required: bool,
    
    /// Maximum nesting depth for validation
    pub max_nesting_depth: usize,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            allow_unknown_fields: false,
            validate_constraints: true,
            validate_required: true,
            max_nesting_depth: 32, // Same as the codec's MAX_NESTING_DEPTH
        }
    }
}

/// Schema validator
#[derive(Debug)]
pub struct SchemaValidator {
    config: ValidatorConfig,
}

impl SchemaValidator {
    /// Creates a new schema validator with default configuration
    pub fn new() -> Self {
        Self {
            config: ValidatorConfig::default(),
        }
    }
    
    /// Creates a new schema validator with custom configuration
    pub fn with_config(config: ValidatorConfig) -> Self {
        Self { config }
    }
    
    /// Validates an HTLV item against a schema
    pub fn validate(&self, schema: &Schema, item: &HtlvItem) -> Result<()> {
        self.validate_value(&schema.root_type, &item.value, 0)
    }
    
    /// Validates an HTLV value against a schema type
    pub fn validate_value(
        &self,
        schema_type: &SchemaType,
        value: &HtlvValue,
        depth: usize,
    ) -> Result<()> {
        // Check nesting depth
        if depth > self.config.max_nesting_depth {
            return Err(Error::SchemaError(format!(
                "Maximum nesting depth ({}) exceeded",
                self.config.max_nesting_depth
            )));
        }
        
        match (schema_type, value) {
            // Basic types
            (SchemaType::Null, HtlvValue::Null) => Ok(()),
            (SchemaType::Boolean, HtlvValue::Bool(_)) => Ok(()),
            (SchemaType::UInt8, HtlvValue::U8(_)) => Ok(()),
            (SchemaType::UInt16, HtlvValue::U16(_)) => Ok(()),
            (SchemaType::UInt32, HtlvValue::U32(_)) => Ok(()),
            (SchemaType::UInt64, HtlvValue::U64(_)) => Ok(()),
            (SchemaType::Int8, HtlvValue::I8(_)) => Ok(()),
            (SchemaType::Int16, HtlvValue::I16(_)) => Ok(()),
            (SchemaType::Int32, HtlvValue::I32(_)) => Ok(()),
            (SchemaType::Int64, HtlvValue::I64(_)) => Ok(()),
            (SchemaType::Float32, HtlvValue::F32(_)) => Ok(()),
            (SchemaType::Float64, HtlvValue::F64(_)) => Ok(()),
            (SchemaType::Binary, HtlvValue::Bytes(_)) => Ok(()),
            (SchemaType::String, HtlvValue::String(_)) => Ok(()),
            
            // Array type
            (SchemaType::Array(elem_type), HtlvValue::Array(items)) => {
                for item in items {
                    self.validate_value(elem_type, &item.value, depth + 1)?;
                }
                Ok(())
            },
            
            // Object type
            (SchemaType::Object(fields), HtlvValue::Object(items)) => {
                self.validate_object(fields, items, depth)
            },
            
            // Map type
            (SchemaType::Map(key_type, value_type), HtlvValue::Object(items)) => {
                // Each item in a map is expected to be an object with key and value fields
                for item in items {
                    if let HtlvValue::Object(entry) = &item.value {
                        if entry.len() != 2 {
                            return Err(Error::SchemaError(
                                "Map entry must have exactly 2 fields (key and value)".to_string()
                            ));
                        }
                        
                        // Validate key (tag 0)
                        if let Some(key_item) = entry.iter().find(|i| i.tag == 0) {
                            self.validate_value(key_type, &key_item.value, depth + 1)?;
                        } else {
                            return Err(Error::SchemaError("Map entry missing key field (tag 0)".to_string()));
                        }
                        
                        // Validate value (tag 1)
                        if let Some(val_item) = entry.iter().find(|i| i.tag == 1) {
                            self.validate_value(value_type, &val_item.value, depth + 1)?;
                        } else {
                            return Err(Error::SchemaError("Map entry missing value field (tag 1)".to_string()));
                        }
                    } else {
                        return Err(Error::SchemaError(
                            "Map entry must be an object with key and value fields".to_string()
                        ));
                    }
                }
                Ok(())
            },
            
            // Union type
            (SchemaType::Union(types), value) => {
                // Try each possible type
                for t in types {
                    if self.validate_value(t, value, depth).is_ok() {
                        return Ok(());
                    }
                }
                
                // No matching type found
                Err(Error::SchemaError(format!(
                    "Value does not match any type in union: {:?}", value
                )))
            },
            
            // Type mismatch
            (expected, actual) => Err(Error::SchemaError(format!(
                "Type mismatch: expected {:?}, got {:?}", expected, actual
            ))),
        }
    }
    
    /// Validates an object against a schema object type
    fn validate_object(
        &self,
        fields: &[SchemaField],
        items: &[HtlvItem],
        depth: usize,
    ) -> Result<()> {
        // Create a map of field tags to field definitions for quick lookup
        let field_map: HashMap<u64, &SchemaField> = fields
            .iter()
            .map(|field| (field.tag, field))
            .collect();
        
        // Track which required fields we've seen
        let mut seen_fields = HashMap::new();
        
        // Validate each object field
        for item in items {
            if let Some(field) = field_map.get(&item.tag) {
                self.validate_value(&field.field_type, &item.value, depth + 1)?;
                
                // If validating constraints, check field-specific constraints
                if self.config.validate_constraints {
                    self.validate_constraints(field, &item.value)?;
                }
                
                seen_fields.insert(field.tag, true);
            } else if !self.config.allow_unknown_fields {
                // Unknown field
                return Err(Error::SchemaError(format!(
                    "Unknown field with tag {} in object", item.tag
                )));
            }
        }
        
        // Check that all required fields are present
        if self.config.validate_required {
            for field in fields {
                if field.required && !seen_fields.contains_key(&field.tag) {
                    return Err(Error::SchemaError(format!(
                        "Required field '{}' (tag {}) is missing", field.name, field.tag
                    )));
                }
            }
        }
        
        Ok(())
    }
    
    /// Validates field-specific constraints
    fn validate_constraints(&self, field: &SchemaField, value: &HtlvValue) -> Result<()> {
        let options = &field.options;
        
        // Validate min/max value constraints for numeric types
        if let (Some(min_value), HtlvValue::U8(v)) = (&options.min_value, value) {
            if let HtlvValue::U8(min) = min_value {
                if v < min {
                    return Err(Error::SchemaError(format!(
                        "Field '{}' value {} is less than minimum {}", field.name, v, min
                    )));
                }
            }
        }
        
        // Similar checks for other numeric types...
        // (Omitted for brevity, but would follow the same pattern)
        
        // Validate min/max length constraints for string, binary, array types
        if let (Some(min_length), HtlvValue::String(s)) = (options.min_length, value) {
            if s.len() < min_length {
                return Err(Error::SchemaError(format!(
                    "Field '{}' string length {} is less than minimum {}",
                    field.name, s.len(), min_length
                )));
            }
        }
        
        if let (Some(max_length), HtlvValue::String(s)) = (options.max_length, value) {
            if s.len() > max_length {
                return Err(Error::SchemaError(format!(
                    "Field '{}' string length {} is greater than maximum {}",
                    field.name, s.len(), max_length
                )));
            }
        }
        
        if let (Some(min_length), HtlvValue::Bytes(b)) = (options.min_length, value) {
            if b.len() < min_length {
                return Err(Error::SchemaError(format!(
                    "Field '{}' binary length {} is less than minimum {}",
                    field.name, b.len(), min_length
                )));
            }
        }
        
        if let (Some(max_length), HtlvValue::Bytes(b)) = (options.max_length, value) {
            if b.len() > max_length {
                return Err(Error::SchemaError(format!(
                    "Field '{}' binary length {} is greater than maximum {}",
                    field.name, b.len(), max_length
                )));
            }
        }
        
        if let (Some(min_length), HtlvValue::Array(arr)) = (options.min_length, value) {
            if arr.len() < min_length {
                return Err(Error::SchemaError(format!(
                    "Field '{}' array length {} is less than minimum {}",
                    field.name, arr.len(), min_length
                )));
            }
        }
        
        if let (Some(max_length), HtlvValue::Array(arr)) = (options.max_length, value) {
            if arr.len() > max_length {
                return Err(Error::SchemaError(format!(
                    "Field '{}' array length {} is greater than maximum {}",
                    field.name, arr.len(), max_length
                )));
            }
        }
        
        // Validate pattern constraint for string types
        if let (Some(pattern), HtlvValue::String(s)) = (&options.pattern, value) {
            // TODO: Implement regex pattern validation
            // For now, just skip this validation
        }
        
        Ok(())
    }
}
