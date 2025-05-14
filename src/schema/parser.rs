// JSON-like Schema parser for Tonitru
//
// This module implements a parser for JSON Schema-like definitions,
// converting them to Tonitru Schema objects.

use std::collections::HashMap;

use serde_json::Value;

use crate::internal::error::{Error, Result};
use crate::codec::types::HtlvValue;
use crate::schema::types::{Schema, SchemaType, SchemaField, SchemaOptions, SchemaVersion};

/// Parser for JSON-like Schema definitions
#[derive(Debug, Default)]
pub struct SchemaParser {
    /// Custom type mappings (JSON schema type name -> Tonitru schema type)
    custom_type_mappings: HashMap<String, SchemaType>,
}

impl SchemaParser {
    /// Creates a new schema parser
    pub fn new() -> Self {
        Self {
            custom_type_mappings: HashMap::new(),
        }
    }
    
    /// Adds a custom type mapping
    pub fn add_type_mapping(&mut self, json_type: &str, schema_type: SchemaType) {
        self.custom_type_mappings.insert(json_type.to_string(), schema_type);
    }
    
    /// Parses a JSON schema definition into a Tonitru Schema
    pub fn parse_schema(&self, json: &Value) -> Result<Schema> {
        // Validate that the input is an object
        let obj = match json {
            Value::Object(obj) => obj,
            _ => return Err(Error::SchemaError("Schema must be a JSON object".to_string())),
        };
        
        // Extract required fields
        let id = self.get_string_field(obj, "id")?;
        let name = self.get_string_field(obj, "name")?;
        
        // Parse version
        let version = if let Some(version_value) = obj.get("version") {
            self.parse_version(version_value)?
        } else {
            // Default to version 1.0.0
            SchemaVersion::new(1, 0, 0)
        };
        
        // Parse root type
        let root_type = if let Some(type_value) = obj.get("type") {
            self.parse_type(type_value, obj)?
        } else if let Some(properties) = obj.get("properties") {
            // If no type is specified but properties are present, assume it's an object
            self.parse_object_type(properties)?
        } else {
            return Err(Error::SchemaError("Schema must specify a type or properties".to_string()));
        };
        
        // Create the schema
        let mut schema = Schema::new(id, name, version, root_type);
        
        // Parse optional description
        if let Some(Value::String(desc)) = obj.get("description") {
            schema.description = Some(desc.clone());
        }
        
        // Parse additional metadata
        if let Some(Value::Object(metadata)) = obj.get("metadata") {
            for (key, value) in metadata {
                if let Value::String(val) = value {
                    schema.metadata.insert(key.clone(), val.clone());
                }
            }
        }
        
        Ok(schema)
    }
    
    /// Parses a version string or object into a SchemaVersion
    fn parse_version(&self, value: &Value) -> Result<SchemaVersion> {
        match value {
            Value::String(version_str) => {
                // Parse version string (e.g., "1.2.3")
                let parts: Vec<&str> = version_str.split('.').collect();
                if parts.len() != 3 {
                    return Err(Error::SchemaError(format!(
                        "Invalid version format: {}, expected 'major.minor.patch'", version_str
                    )));
                }
                
                let major = parts[0].parse::<u32>().map_err(|_| {
                    Error::SchemaError(format!("Invalid major version: {}", parts[0]))
                })?;
                
                let minor = parts[1].parse::<u32>().map_err(|_| {
                    Error::SchemaError(format!("Invalid minor version: {}", parts[1]))
                })?;
                
                let patch = parts[2].parse::<u32>().map_err(|_| {
                    Error::SchemaError(format!("Invalid patch version: {}", parts[2]))
                })?;
                
                Ok(SchemaVersion::new(major, minor, patch))
            },
            Value::Object(obj) => {
                // Parse version object (e.g., {"major": 1, "minor": 2, "patch": 3})
                let major = self.get_u32_field(obj, "major")?;
                let minor = self.get_u32_field(obj, "minor")?;
                let patch = self.get_u32_field(obj, "patch")?;
                
                Ok(SchemaVersion::new(major, minor, patch))
            },
            _ => Err(Error::SchemaError(format!(
                "Invalid version format: {:?}, expected string or object", value
            ))),
        }
    }
    
    /// Parses a type definition into a SchemaType
    fn parse_type(&self, type_value: &Value, schema_obj: &serde_json::Map<String, Value>) -> Result<SchemaType> {
        match type_value {
            Value::String(type_name) => {
                // Check for custom type mapping
                if let Some(custom_type) = self.custom_type_mappings.get(type_name) {
                    return Ok(custom_type.clone());
                }
                
                // Parse standard types
                match type_name.as_str() {
                    "null" => Ok(SchemaType::Null),
                    "boolean" => Ok(SchemaType::Boolean),
                    "integer" => {
                        // Check for format to determine integer size
                        if let Some(Value::String(format)) = schema_obj.get("format") {
                            match format.as_str() {
                                "int8" => Ok(SchemaType::Int8),
                                "int16" => Ok(SchemaType::Int16),
                                "int32" => Ok(SchemaType::Int32),
                                "int64" => Ok(SchemaType::Int64),
                                "uint8" => Ok(SchemaType::UInt8),
                                "uint16" => Ok(SchemaType::UInt16),
                                "uint32" => Ok(SchemaType::UInt32),
                                "uint64" => Ok(SchemaType::UInt64),
                                _ => Ok(SchemaType::Int32), // Default to int32
                            }
                        } else {
                            Ok(SchemaType::Int32) // Default to int32
                        }
                    },
                    "number" => {
                        // Check for format to determine float size
                        if let Some(Value::String(format)) = schema_obj.get("format") {
                            match format.as_str() {
                                "float" | "float32" => Ok(SchemaType::Float32),
                                "double" | "float64" => Ok(SchemaType::Float64),
                                _ => Ok(SchemaType::Float64), // Default to float64
                            }
                        } else {
                            Ok(SchemaType::Float64) // Default to float64
                        }
                    },
                    "string" => {
                        // Check for format to determine if it's binary
                        if let Some(Value::String(format)) = schema_obj.get("format") {
                            match format.as_str() {
                                "binary" | "bytes" => Ok(SchemaType::Binary),
                                _ => Ok(SchemaType::String),
                            }
                        } else {
                            Ok(SchemaType::String)
                        }
                    },
                    "array" => {
                        // Parse array items type
                        if let Some(items) = schema_obj.get("items") {
                            let item_type = self.parse_type(items, schema_obj)?;
                            Ok(SchemaType::Array(Box::new(item_type)))
                        } else {
                            Err(Error::SchemaError("Array schema must specify 'items'".to_string()))
                        }
                    },
                    "object" => {
                        // Parse object properties
                        if let Some(properties) = schema_obj.get("properties") {
                            self.parse_object_type(properties)
                        } else {
                            // Empty object
                            Ok(SchemaType::Object(Vec::new()))
                        }
                    },
                    _ => Err(Error::SchemaError(format!("Unknown type: {}", type_name))),
                }
            },
            Value::Array(types) => {
                // Union type (multiple possible types)
                let mut union_types = Vec::new();
                for t in types {
                    let schema_type = self.parse_type(t, schema_obj)?;
                    union_types.push(schema_type);
                }
                Ok(SchemaType::Union(union_types))
            },
            Value::Object(obj) => {
                // Complex type definition
                if let Some(Value::String(type_name)) = obj.get("type") {
                    match type_name.as_str() {
                        "array" => {
                            // Parse array items type
                            if let Some(items) = obj.get("items") {
                                let item_type = self.parse_type(items, obj)?;
                                Ok(SchemaType::Array(Box::new(item_type)))
                            } else {
                                Err(Error::SchemaError("Array schema must specify 'items'".to_string()))
                            }
                        },
                        "object" => {
                            // Parse object properties
                            if let Some(properties) = obj.get("properties") {
                                self.parse_object_type(properties)
                            } else {
                                // Empty object
                                Ok(SchemaType::Object(Vec::new()))
                            }
                        },
                        "map" => {
                            // Parse map key and value types
                            let key_type = if let Some(keys) = obj.get("keys") {
                                self.parse_type(keys, obj)?
                            } else {
                                // Default to string keys
                                SchemaType::String
                            };
                            
                            let value_type = if let Some(values) = obj.get("values") {
                                self.parse_type(values, obj)?
                            } else {
                                return Err(Error::SchemaError("Map schema must specify 'values'".to_string()));
                            };
                            
                            Ok(SchemaType::Map(Box::new(key_type), Box::new(value_type)))
                        },
                        _ => self.parse_type(&Value::String(type_name.clone()), obj),
                    }
                } else {
                    // Assume it's an object with inline properties
                    self.parse_object_type(type_value)
                }
            },
            _ => Err(Error::SchemaError(format!("Invalid type definition: {:?}", type_value))),
        }
    }
    
    /// Parses an object type definition
    fn parse_object_type(&self, properties: &Value) -> Result<SchemaType> {
        let props = match properties {
            Value::Object(obj) => obj,
            _ => return Err(Error::SchemaError("Properties must be an object".to_string())),
        };
        
        let mut fields = Vec::new();
        
        for (name, prop) in props {
            let prop_obj = match prop {
                Value::Object(obj) => obj,
                _ => return Err(Error::SchemaError(format!("Property '{}' must be an object", name))),
            };
            
            // Parse field type
            let field_type = if let Some(type_value) = prop_obj.get("type") {
                self.parse_type(type_value, prop_obj)?
            } else {
                return Err(Error::SchemaError(format!("Property '{}' must specify a type", name)));
            };
            
            // Parse tag (required for HTLV encoding)
            let tag = if let Some(Value::Number(tag_num)) = prop_obj.get("tag") {
                if let Some(tag_u64) = tag_num.as_u64() {
                    tag_u64
                } else {
                    return Err(Error::SchemaError(format!("Invalid tag for property '{}': must be a positive integer", name)));
                }
            } else {
                // If no tag is specified, use a hash of the field name
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                name.hash(&mut hasher);
                hasher.finish()
            };
            
            // Parse required flag
            let required = if let Some(Value::Bool(req)) = prop_obj.get("required") {
                *req
            } else {
                false // Default to not required
            };
            
            // Parse default value
            let default_value = if let Some(default) = prop_obj.get("default") {
                // TODO: Convert JSON default value to HtlvValue
                // For now, just use None
                None
            } else {
                None
            };
            
            // Parse description
            let description = if let Some(Value::String(desc)) = prop_obj.get("description") {
                Some(desc.clone())
            } else {
                None
            };
            
            // Parse additional options
            let mut options = SchemaOptions::default();
            
            // Parse compress flag
            if let Some(Value::Bool(compress)) = prop_obj.get("compress") {
                options.compress = *compress;
            }
            
            // Parse encrypt flag
            if let Some(Value::Bool(encrypt)) = prop_obj.get("encrypt") {
                options.encrypt = *encrypt;
            }
            
            // Parse index flag
            if let Some(Value::Bool(index)) = prop_obj.get("index") {
                options.index = *index;
            }
            
            // Parse min/max value
            if let Some(min_value) = prop_obj.get("minimum") {
                // TODO: Convert JSON min value to HtlvValue
            }
            
            if let Some(max_value) = prop_obj.get("maximum") {
                // TODO: Convert JSON max value to HtlvValue
            }
            
            // Parse pattern
            if let Some(Value::String(pattern)) = prop_obj.get("pattern") {
                options.pattern = Some(pattern.clone());
            }
            
            // Parse min/max length
            if let Some(Value::Number(min_length)) = prop_obj.get("minLength") {
                if let Some(len) = min_length.as_u64() {
                    options.min_length = Some(len as usize);
                }
            }
            
            if let Some(Value::Number(max_length)) = prop_obj.get("maxLength") {
                if let Some(len) = max_length.as_u64() {
                    options.max_length = Some(len as usize);
                }
            }
            
            // Parse custom options
            if let Some(Value::Object(custom)) = prop_obj.get("custom") {
                for (key, value) in custom {
                    if let Value::String(val) = value {
                        options.custom.insert(key.clone(), val.clone());
                    }
                }
            }
            
            // Create the field
            let field = SchemaField {
                name: name.clone(),
                tag,
                field_type,
                required,
                default_value,
                description,
                options,
            };
            
            fields.push(field);
        }
        
        Ok(SchemaType::Object(fields))
    }
    
    /// Helper to get a string field from a JSON object
    fn get_string_field(&self, obj: &serde_json::Map<String, Value>, field: &str) -> Result<String> {
        match obj.get(field) {
            Some(Value::String(s)) => Ok(s.clone()),
            Some(_) => Err(Error::SchemaError(format!("Field '{}' must be a string", field))),
            None => Err(Error::SchemaError(format!("Required field '{}' is missing", field))),
        }
    }
    
    /// Helper to get a u32 field from a JSON object
    fn get_u32_field(&self, obj: &serde_json::Map<String, Value>, field: &str) -> Result<u32> {
        match obj.get(field) {
            Some(Value::Number(n)) => {
                if let Some(u) = n.as_u64() {
                    if u <= u32::MAX as u64 {
                        Ok(u as u32)
                    } else {
                        Err(Error::SchemaError(format!("Field '{}' is too large for u32", field)))
                    }
                } else {
                    Err(Error::SchemaError(format!("Field '{}' must be a positive integer", field)))
                }
            },
            Some(_) => Err(Error::SchemaError(format!("Field '{}' must be a number", field))),
            None => Err(Error::SchemaError(format!("Required field '{}' is missing", field))),
        }
    }
}
