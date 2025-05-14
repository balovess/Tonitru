// Schema type system for Tonitru data format
//
// This module defines the core types for the Schema system, ensuring
// cross-platform consistency and precise type definitions.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crate::internal::error::{Error, Result};
use crate::codec::types::{HtlvItem, HtlvValue};

/// Represents a schema version
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SchemaVersion {
    /// Major version (incompatible changes)
    pub major: u32,
    /// Minor version (backwards-compatible additions)
    pub minor: u32,
    /// Patch version (backwards-compatible fixes)
    pub patch: u32,
}

impl SchemaVersion {
    /// Creates a new schema version
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }
    
    /// Checks if this version is compatible with another version
    pub fn is_compatible_with(&self, other: &SchemaVersion) -> bool {
        // Major versions must match for compatibility
        self.major == other.major
    }
    
    /// Returns a string representation of the version (e.g., "1.2.3")
    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Represents the data types supported in the schema
#[derive(Debug, Clone, PartialEq)]
pub enum SchemaType {
    /// Null type
    Null,
    /// Boolean type
    Boolean,
    /// 8-bit unsigned integer
    UInt8,
    /// 16-bit unsigned integer
    UInt16,
    /// 32-bit unsigned integer
    UInt32,
    /// 64-bit unsigned integer
    UInt64,
    /// 8-bit signed integer
    Int8,
    /// 16-bit signed integer
    Int16,
    /// 32-bit signed integer
    Int32,
    /// 64-bit signed integer
    Int64,
    /// 32-bit floating point (IEEE 754)
    Float32,
    /// 64-bit floating point (IEEE 754)
    Float64,
    /// Binary data (bytes)
    Binary,
    /// UTF-8 encoded string
    String,
    /// Array of items with the same type
    Array(Box<SchemaType>),
    /// Object with named fields
    Object(Vec<SchemaField>),
    /// Map with keys and values of specified types
    Map(Box<SchemaType>, Box<SchemaType>),
    /// Union of multiple possible types
    Union(Vec<SchemaType>),
}

impl SchemaType {
    /// Returns true if this type is a numeric type
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            SchemaType::UInt8 | SchemaType::UInt16 | SchemaType::UInt32 | SchemaType::UInt64 |
            SchemaType::Int8 | SchemaType::Int16 | SchemaType::Int32 | SchemaType::Int64 |
            SchemaType::Float32 | SchemaType::Float64
        )
    }
    
    /// Returns true if this type is an integer type
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            SchemaType::UInt8 | SchemaType::UInt16 | SchemaType::UInt32 | SchemaType::UInt64 |
            SchemaType::Int8 | SchemaType::Int16 | SchemaType::Int32 | SchemaType::Int64
        )
    }
    
    /// Returns true if this type is a floating point type
    pub fn is_float(&self) -> bool {
        matches!(self, SchemaType::Float32 | SchemaType::Float64)
    }
    
    /// Returns true if this type is a complex type (array, object, map, union)
    pub fn is_complex(&self) -> bool {
        matches!(
            self,
            SchemaType::Array(_) | SchemaType::Object(_) | SchemaType::Map(_, _) | SchemaType::Union(_)
        )
    }
    
    /// Validates that a given HtlvValue matches this schema type
    pub fn validate_value(&self, value: &HtlvValue) -> Result<()> {
        match (self, value) {
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
            (SchemaType::Array(elem_type), HtlvValue::Array(items)) => {
                // Validate each array element
                for item in items {
                    elem_type.validate_value(&item.value)?;
                }
                Ok(())
            },
            (SchemaType::Object(fields), HtlvValue::Object(items)) => {
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
                        field.field_type.validate_value(&item.value)?;
                        seen_fields.insert(field.tag, true);
                    } else {
                        // Unknown field
                        return Err(Error::SchemaError(format!(
                            "Unknown field with tag {} in object", item.tag
                        )));
                    }
                }
                
                // Check that all required fields are present
                for field in fields {
                    if field.required && !seen_fields.contains_key(&field.tag) {
                        return Err(Error::SchemaError(format!(
                            "Required field '{}' (tag {}) is missing", field.name, field.tag
                        )));
                    }
                }
                
                Ok(())
            },
            (SchemaType::Union(types), value) => {
                // Try each possible type
                for t in types {
                    if t.validate_value(value).is_ok() {
                        return Ok(());
                    }
                }
                
                // No matching type found
                Err(Error::SchemaError(format!(
                    "Value does not match any type in union: {:?}", value
                )))
            },
            // Map type validation would go here
            (SchemaType::Map(_, _), _) => {
                // TODO: Implement Map validation
                Err(Error::SchemaError("Map validation not yet implemented".to_string()))
            },
            // Type mismatch
            (expected, actual) => Err(Error::SchemaError(format!(
                "Type mismatch: expected {:?}, got {:?}", expected, actual
            ))),
        }
    }
}

/// Represents a field in an object schema
#[derive(Debug, Clone, PartialEq)]
pub struct SchemaField {
    /// Field name
    pub name: String,
    /// Field tag (used in HTLV encoding)
    pub tag: u64,
    /// Field type
    pub field_type: SchemaType,
    /// Whether the field is required
    pub required: bool,
    /// Default value (if any)
    pub default_value: Option<HtlvValue>,
    /// Field description
    pub description: Option<String>,
    /// Additional field options
    pub options: SchemaOptions,
}

/// Additional options for schema fields
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SchemaOptions {
    /// Whether the field should be compressed
    pub compress: bool,
    /// Whether the field should be encrypted
    pub encrypt: bool,
    /// Whether the field should be indexed
    pub index: bool,
    /// Minimum value (for numeric types)
    pub min_value: Option<HtlvValue>,
    /// Maximum value (for numeric types)
    pub max_value: Option<HtlvValue>,
    /// Pattern (for string types)
    pub pattern: Option<String>,
    /// Minimum length (for string, binary, array types)
    pub min_length: Option<usize>,
    /// Maximum length (for string, binary, array types)
    pub max_length: Option<usize>,
    /// Custom options
    pub custom: HashMap<String, String>,
}

/// Represents a complete schema definition
#[derive(Debug, Clone)]
pub struct Schema {
    /// Schema ID
    pub id: String,
    /// Schema name
    pub name: String,
    /// Schema version
    pub version: SchemaVersion,
    /// Root schema type
    pub root_type: SchemaType,
    /// Schema description
    pub description: Option<String>,
    /// Additional schema metadata
    pub metadata: HashMap<String, String>,
}

impl Schema {
    /// Creates a new schema
    pub fn new(
        id: String,
        name: String,
        version: SchemaVersion,
        root_type: SchemaType,
    ) -> Self {
        Self {
            id,
            name,
            version,
            root_type,
            description: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Validates that a given HtlvItem matches this schema
    pub fn validate(&self, item: &HtlvItem) -> Result<()> {
        self.root_type.validate_value(&item.value)
    }
}

/// A registry of schemas
#[derive(Debug, Default)]
pub struct SchemaRegistry {
    /// Map of schema IDs to schemas
    schemas: HashMap<String, Arc<Schema>>,
    /// Map of schema IDs to schema versions
    versions: HashMap<String, Vec<(SchemaVersion, Arc<Schema>)>>,
}

impl SchemaRegistry {
    /// Creates a new schema registry
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
            versions: HashMap::new(),
        }
    }
    
    /// Registers a schema
    pub fn register_schema(&mut self, schema: Schema) -> Result<()> {
        let schema_id = schema.id.clone();
        let schema_version = schema.version.clone();
        let schema_arc = Arc::new(schema);
        
        // Store the latest version
        self.schemas.insert(schema_id.clone(), schema_arc.clone());
        
        // Store in version history
        let versions = self.versions.entry(schema_id).or_insert_with(Vec::new);
        versions.push((schema_version, schema_arc));
        
        // Sort versions in descending order
        versions.sort_by(|(a, _), (b, _)| {
            let a_key = (a.major, a.minor, a.patch);
            let b_key = (b.major, b.minor, b.patch);
            b_key.cmp(&a_key) // Descending order
        });
        
        Ok(())
    }
    
    /// Gets a schema by ID (latest version)
    pub fn get_schema(&self, id: &str) -> Option<Arc<Schema>> {
        self.schemas.get(id).cloned()
    }
    
    /// Gets a schema by ID and version
    pub fn get_schema_version(&self, id: &str, version: &SchemaVersion) -> Option<Arc<Schema>> {
        if let Some(versions) = self.versions.get(id) {
            for (ver, schema) in versions {
                if ver == version {
                    return Some(schema.clone());
                }
            }
        }
        None
    }
}
