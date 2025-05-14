// Default value strategies for Tonitru schema system
//
// This module defines strategies for handling default values in schemas,
// particularly for nested objects and complex types.

use std::collections::HashMap;

use crate::internal::error::{Error, Result};
use crate::codec::types::{HtlvItem, HtlvValue};
use crate::schema::types::{SchemaType, SchemaField, Schema};

/// Represents different strategies for applying default values
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DefaultValueStrategy {
    /// No default values are applied
    None,
    
    /// Apply defaults only for required fields
    RequiredOnly,
    
    /// Apply defaults for all fields that have default values defined
    AllFields,
    
    /// Apply defaults recursively for nested objects
    Recursive,
    
    /// Custom strategy with specific fields to apply defaults
    Custom(Vec<String>),
}

impl DefaultValueStrategy {
    /// Applies default values to an HtlvValue based on the schema and strategy
    pub fn apply_defaults(
        &self,
        schema_type: &SchemaType,
        value: Option<HtlvValue>,
    ) -> Result<HtlvValue> {
        match value {
            // If a value is provided, use it (but still apply defaults to nested objects)
            Some(mut val) => {
                if let (SchemaType::Object(fields), HtlvValue::Object(ref mut items)) = (schema_type, &mut val) {
                    if matches!(self, DefaultValueStrategy::Recursive) {
                        self.apply_defaults_to_object(fields, items)?;
                    }
                } else if let (SchemaType::Array(elem_type), HtlvValue::Array(ref mut items)) = (schema_type, &mut val) {
                    if matches!(self, DefaultValueStrategy::Recursive) {
                        for item in items {
                            item.value = self.apply_defaults(elem_type, Some(item.value.clone()))?;
                        }
                    }
                }
                Ok(val)
            },
            
            // If no value is provided, create a default value based on the schema
            None => match schema_type {
                SchemaType::Null => Ok(HtlvValue::Null),
                SchemaType::Boolean => Ok(HtlvValue::Bool(false)),
                SchemaType::UInt8 => Ok(HtlvValue::U8(0)),
                SchemaType::UInt16 => Ok(HtlvValue::U16(0)),
                SchemaType::UInt32 => Ok(HtlvValue::U32(0)),
                SchemaType::UInt64 => Ok(HtlvValue::U64(0)),
                SchemaType::Int8 => Ok(HtlvValue::I8(0)),
                SchemaType::Int16 => Ok(HtlvValue::I16(0)),
                SchemaType::Int32 => Ok(HtlvValue::I32(0)),
                SchemaType::Int64 => Ok(HtlvValue::I64(0)),
                SchemaType::Float32 => Ok(HtlvValue::F32(0.0)),
                SchemaType::Float64 => Ok(HtlvValue::F64(0.0)),
                SchemaType::Binary => Ok(HtlvValue::Bytes(bytes::Bytes::new())),
                SchemaType::String => Ok(HtlvValue::String(bytes::Bytes::new())),
                SchemaType::Array(_) => Ok(HtlvValue::Array(Vec::new())),
                SchemaType::Object(fields) => {
                    let mut items = Vec::new();
                    
                    // Apply defaults based on strategy
                    match self {
                        DefaultValueStrategy::None => {},
                        DefaultValueStrategy::RequiredOnly => {
                            for field in fields {
                                if field.required {
                                    if let Some(default) = &field.default_value {
                                        items.push(HtlvItem {
                                            tag: field.tag,
                                            value: default.clone(),
                                        });
                                    } else {
                                        // Create default value for required field
                                        let default_value = self.apply_defaults(&field.field_type, None)?;
                                        items.push(HtlvItem {
                                            tag: field.tag,
                                            value: default_value,
                                        });
                                    }
                                }
                            }
                        },
                        DefaultValueStrategy::AllFields | DefaultValueStrategy::Recursive => {
                            for field in fields {
                                if let Some(default) = &field.default_value {
                                    items.push(HtlvItem {
                                        tag: field.tag,
                                        value: default.clone(),
                                    });
                                } else if field.required {
                                    // Create default value for required field
                                    let default_value = self.apply_defaults(&field.field_type, None)?;
                                    items.push(HtlvItem {
                                        tag: field.tag,
                                        value: default_value,
                                    });
                                }
                            }
                        },
                        DefaultValueStrategy::Custom(field_names) => {
                            let field_name_set: std::collections::HashSet<&String> = field_names.iter().collect();
                            
                            for field in fields {
                                if field_name_set.contains(&field.name) {
                                    if let Some(default) = &field.default_value {
                                        items.push(HtlvItem {
                                            tag: field.tag,
                                            value: default.clone(),
                                        });
                                    } else {
                                        // Create default value for specified field
                                        let default_value = self.apply_defaults(&field.field_type, None)?;
                                        items.push(HtlvItem {
                                            tag: field.tag,
                                            value: default_value,
                                        });
                                    }
                                } else if field.required {
                                    // Create default value for required field
                                    let default_value = self.apply_defaults(&field.field_type, None)?;
                                    items.push(HtlvItem {
                                        tag: field.tag,
                                        value: default_value,
                                    });
                                }
                            }
                        },
                    }
                    
                    Ok(HtlvValue::Object(items))
                },
                SchemaType::Map(_, _) => Ok(HtlvValue::Object(Vec::new())), // Maps are represented as empty objects by default
                SchemaType::Union(types) => {
                    if types.is_empty() {
                        return Err(Error::SchemaError("Cannot create default for empty union".to_string()));
                    }
                    // Use the first type in the union as the default
                    self.apply_defaults(&types[0], None)
                },
            },
        }
    }
    
    /// Applies default values to an object's fields
    fn apply_defaults_to_object(
        &self,
        fields: &[SchemaField],
        items: &mut Vec<HtlvItem>,
    ) -> Result<()> {
        // Create a map of existing field tags
        let mut existing_fields = HashMap::new();
        for (i, item) in items.iter_mut().enumerate() {
            existing_fields.insert(item.tag, i);
            
            // Find the corresponding field definition
            if let Some(field) = fields.iter().find(|f| f.tag == item.tag) {
                // Recursively apply defaults to nested objects
                if let SchemaType::Object(_) = &field.field_type {
                    if let HtlvValue::Object(_) = &item.value {
                        item.value = self.apply_defaults(&field.field_type, Some(item.value.clone()))?;
                    }
                } else if let SchemaType::Array(elem_type) = &field.field_type {
                    if let HtlvValue::Array(array_items) = &item.value {
                        let mut new_array_items = Vec::new();
                        for array_item in array_items {
                            let new_value = self.apply_defaults(elem_type, Some(array_item.value.clone()))?;
                            new_array_items.push(HtlvItem {
                                tag: array_item.tag,
                                value: new_value,
                            });
                        }
                        item.value = HtlvValue::Array(new_array_items);
                    }
                }
            }
        }
        
        // Add missing fields with default values based on strategy
        match self {
            DefaultValueStrategy::None => {},
            DefaultValueStrategy::RequiredOnly => {
                for field in fields {
                    if field.required && !existing_fields.contains_key(&field.tag) {
                        if let Some(default) = &field.default_value {
                            items.push(HtlvItem {
                                tag: field.tag,
                                value: default.clone(),
                            });
                        } else {
                            // Create default value for required field
                            let default_value = self.apply_defaults(&field.field_type, None)?;
                            items.push(HtlvItem {
                                tag: field.tag,
                                value: default_value,
                            });
                        }
                    }
                }
            },
            DefaultValueStrategy::AllFields | DefaultValueStrategy::Recursive => {
                for field in fields {
                    if !existing_fields.contains_key(&field.tag) {
                        if let Some(default) = &field.default_value {
                            items.push(HtlvItem {
                                tag: field.tag,
                                value: default.clone(),
                            });
                        } else if field.required {
                            // Create default value for required field
                            let default_value = self.apply_defaults(&field.field_type, None)?;
                            items.push(HtlvItem {
                                tag: field.tag,
                                value: default_value,
                            });
                        }
                    }
                }
            },
            DefaultValueStrategy::Custom(field_names) => {
                let field_name_set: std::collections::HashSet<&String> = field_names.iter().collect();
                
                for field in fields {
                    if !existing_fields.contains_key(&field.tag) {
                        if field_name_set.contains(&field.name) {
                            if let Some(default) = &field.default_value {
                                items.push(HtlvItem {
                                    tag: field.tag,
                                    value: default.clone(),
                                });
                            } else {
                                // Create default value for specified field
                                let default_value = self.apply_defaults(&field.field_type, None)?;
                                items.push(HtlvItem {
                                    tag: field.tag,
                                    value: default_value,
                                });
                            }
                        } else if field.required {
                            // Create default value for required field
                            let default_value = self.apply_defaults(&field.field_type, None)?;
                            items.push(HtlvItem {
                                tag: field.tag,
                                value: default_value,
                            });
                        }
                    }
                }
            },
        }
        
        Ok(())
    }
}
