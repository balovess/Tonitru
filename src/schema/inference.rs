// Type inference for Tonitru schema system
//
// This module provides functionality to automatically infer schema types
// from sample data, supporting both simple and complex nested structures.

use std::collections::{HashMap, HashSet};

use serde_json::Value;

use crate::internal::error::{Error, Result};
use crate::schema::types::{Schema, SchemaType, SchemaField, SchemaVersion};

/// Configuration for schema inference
#[derive(Debug, Clone)]
pub struct InferenceConfig {
    /// Minimum number of samples required to infer a type
    pub min_samples: usize,
    
    /// Whether to use the most specific numeric type possible
    pub use_specific_numeric_types: bool,
    
    /// Whether to infer required fields
    pub infer_required_fields: bool,
    
    /// Threshold for considering a field required (0.0 - 1.0)
    pub required_field_threshold: f64,
    
    /// Whether to infer patterns for string fields
    pub infer_patterns: bool,
    
    /// Whether to infer min/max values for numeric fields
    pub infer_min_max: bool,
    
    /// Whether to infer min/max length for string/array fields
    pub infer_min_max_length: bool,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            min_samples: 1,
            use_specific_numeric_types: true,
            infer_required_fields: true,
            required_field_threshold: 0.9, // 90% presence to be considered required
            infer_patterns: false, // Pattern inference is complex and disabled by default
            infer_min_max: true,
            infer_min_max_length: true,
        }
    }
}

/// Schema inference engine
#[derive(Debug)]
pub struct SchemaInference {
    config: InferenceConfig,
}

impl SchemaInference {
    /// Creates a new schema inference engine with default configuration
    pub fn new() -> Self {
        Self {
            config: InferenceConfig::default(),
        }
    }
    
    /// Creates a new schema inference engine with custom configuration
    pub fn with_config(config: InferenceConfig) -> Self {
        Self { config }
    }
    
    /// Infers a schema from a collection of JSON samples
    pub fn infer_schema(
        &self,
        id: &str,
        name: &str,
        samples: &[Value],
    ) -> Result<Schema> {
        if samples.is_empty() || samples.len() < self.config.min_samples {
            return Err(Error::SchemaError(format!(
                "Not enough samples to infer schema. Need at least {} sample(s).",
                self.config.min_samples
            )));
        }
        
        // Infer the root type
        let root_type = self.infer_type(samples)?;
        
        // Create the schema
        let schema = Schema::new(
            id.to_string(),
            name.to_string(),
            SchemaVersion::new(1, 0, 0), // Default to version 1.0.0
            root_type,
        );
        
        Ok(schema)
    }
    
    /// Infers a schema type from a collection of JSON values
    fn infer_type(&self, values: &[Value]) -> Result<SchemaType> {
        if values.is_empty() {
            return Err(Error::SchemaError("Cannot infer type from empty values".to_string()));
        }
        
        // Check if all values are of the same type
        let first_type = self.get_json_type(&values[0]);
        let all_same_type = values.iter().all(|v| self.get_json_type(v) == first_type);
        
        if all_same_type {
            match first_type {
                "null" => Ok(SchemaType::Null),
                "boolean" => Ok(SchemaType::Boolean),
                "number" => self.infer_numeric_type(values),
                "string" => self.infer_string_type(values),
                "array" => self.infer_array_type(values),
                "object" => self.infer_object_type(values),
                _ => Err(Error::SchemaError(format!("Unknown JSON type: {}", first_type))),
            }
        } else {
            // If values have different types, create a union type
            let mut type_set = HashSet::new();
            let mut type_samples = HashMap::new();
            
            for value in values {
                let type_name = self.get_json_type(value);
                type_set.insert(type_name.clone());
                
                let samples = type_samples.entry(type_name).or_insert_with(Vec::new);
                samples.push(value.clone());
            }
            
            let mut union_types = Vec::new();
            for (type_name, samples) in type_samples {
                let samples_slice: Vec<&Value> = samples.iter().collect();
                match type_name.as_str() {
                    "null" => union_types.push(SchemaType::Null),
                    "boolean" => union_types.push(SchemaType::Boolean),
                    "number" => union_types.push(self.infer_numeric_type(&samples)?),
                    "string" => union_types.push(self.infer_string_type(&samples)?),
                    "array" => union_types.push(self.infer_array_type(&samples)?),
                    "object" => union_types.push(self.infer_object_type(&samples)?),
                    _ => return Err(Error::SchemaError(format!("Unknown JSON type: {}", type_name))),
                }
            }
            
            Ok(SchemaType::Union(union_types))
        }
    }
    
    /// Gets the JSON type name of a value
    fn get_json_type(&self, value: &Value) -> String {
        match value {
            Value::Null => "null".to_string(),
            Value::Bool(_) => "boolean".to_string(),
            Value::Number(_) => "number".to_string(),
            Value::String(_) => "string".to_string(),
            Value::Array(_) => "array".to_string(),
            Value::Object(_) => "object".to_string(),
        }
    }
    
    /// Infers a numeric type from a collection of JSON number values
    fn infer_numeric_type(&self, values: &[Value]) -> Result<SchemaType> {
        if !self.config.use_specific_numeric_types {
            // If not using specific types, default to Float64
            return Ok(SchemaType::Float64);
        }
        
        let mut has_decimal = false;
        let mut has_negative = false;
        let mut min_value = f64::MAX;
        let mut max_value = f64::MIN;
        
        for value in values {
            if let Value::Number(n) = value {
                if n.is_f64() {
                    has_decimal = true;
                    if let Some(f) = n.as_f64() {
                        min_value = min_value.min(f);
                        max_value = max_value.max(f);
                        if f < 0.0 {
                            has_negative = true;
                        }
                    }
                } else if let Some(i) = n.as_i64() {
                    min_value = min_value.min(i as f64);
                    max_value = max_value.max(i as f64);
                    if i < 0 {
                        has_negative = true;
                    }
                } else if let Some(u) = n.as_u64() {
                    min_value = min_value.min(u as f64);
                    max_value = max_value.max(u as f64);
                }
            }
        }
        
        if has_decimal {
            // If any value has a decimal point, use floating point
            if min_value >= f32::MIN as f64 && max_value <= f32::MAX as f64 {
                Ok(SchemaType::Float32)
            } else {
                Ok(SchemaType::Float64)
            }
        } else if has_negative {
            // If any value is negative, use signed integer
            if min_value >= i8::MIN as f64 && max_value <= i8::MAX as f64 {
                Ok(SchemaType::Int8)
            } else if min_value >= i16::MIN as f64 && max_value <= i16::MAX as f64 {
                Ok(SchemaType::Int16)
            } else if min_value >= i32::MIN as f64 && max_value <= i32::MAX as f64 {
                Ok(SchemaType::Int32)
            } else {
                Ok(SchemaType::Int64)
            }
        } else {
            // All values are non-negative integers
            if max_value <= u8::MAX as f64 {
                Ok(SchemaType::UInt8)
            } else if max_value <= u16::MAX as f64 {
                Ok(SchemaType::UInt16)
            } else if max_value <= u32::MAX as f64 {
                Ok(SchemaType::UInt32)
            } else {
                Ok(SchemaType::UInt64)
            }
        }
    }
    
    /// Infers a string type from a collection of JSON string values
    fn infer_string_type(&self, values: &[Value]) -> Result<SchemaType> {
        // Check if all strings look like base64 encoded binary data
        let all_base64 = values.iter().all(|v| {
            if let Value::String(s) = v {
                // Simple heuristic: base64 strings are typically longer and contain only valid base64 characters
                s.len() > 8 && s.chars().all(|c| {
                    c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='
                })
            } else {
                false
            }
        });
        
        if all_base64 {
            Ok(SchemaType::Binary)
        } else {
            Ok(SchemaType::String)
        }
    }
    
    /// Infers an array type from a collection of JSON array values
    fn infer_array_type(&self, values: &[Value]) -> Result<SchemaType> {
        // Collect all array elements
        let mut all_elements = Vec::new();
        
        for value in values {
            if let Value::Array(arr) = value {
                for elem in arr {
                    all_elements.push(elem.clone());
                }
            }
        }
        
        if all_elements.is_empty() {
            // If no elements, default to array of null
            return Ok(SchemaType::Array(Box::new(SchemaType::Null)));
        }
        
        // Infer the element type
        let element_type = self.infer_type(&all_elements)?;
        
        Ok(SchemaType::Array(Box::new(element_type)))
    }
    
    /// Infers an object type from a collection of JSON object values
    fn infer_object_type(&self, values: &[Value]) -> Result<SchemaType> {
        // Collect all field names and their values
        let mut field_values: HashMap<String, Vec<Value>> = HashMap::new();
        let mut field_presence: HashMap<String, usize> = HashMap::new();
        let total_objects = values.len();
        
        for value in values {
            if let Value::Object(obj) = value {
                // Track which fields are present in this object
                let mut seen_fields = HashSet::new();
                
                for (key, val) in obj {
                    field_values.entry(key.clone()).or_default().push(val.clone());
                    seen_fields.insert(key.clone());
                }
                
                // Update field presence count
                for field in seen_fields {
                    *field_presence.entry(field).or_default() += 1;
                }
            }
        }
        
        if field_values.is_empty() {
            // If no fields, return empty object
            return Ok(SchemaType::Object(Vec::new()));
        }
        
        // Create fields for the object type
        let mut fields = Vec::new();
        
        for (name, values) in field_values {
            // Infer field type
            let field_type = self.infer_type(&values)?;
            
            // Determine if field is required
            let presence_count = field_presence.get(&name).cloned().unwrap_or(0);
            let presence_ratio = presence_count as f64 / total_objects as f64;
            let required = self.config.infer_required_fields && presence_ratio >= self.config.required_field_threshold;
            
            // Generate a tag from the field name
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            name.hash(&mut hasher);
            let tag = hasher.finish();
            
            // Create the field
            let field = SchemaField {
                name,
                tag,
                field_type,
                required,
                default_value: None, // Default values are not inferred
                description: None,   // Descriptions are not inferred
                options: Default::default(), // Use default options
            };
            
            fields.push(field);
        }
        
        Ok(SchemaType::Object(fields))
    }
}
