// Schema module for Tonitru data format
//
// This module provides schema definition, validation, and mapping functionality
// for the Tonitru data format. It includes:
//
// 1. Schema type system with cross-platform consistency
// 2. Default value strategies for nested objects
// 3. Schema to HTLV structure mapping rules
// 4. JSON-like Schema parser
// 5. Type inference logic

// Re-export public types and functions
pub use self::types::{Schema, SchemaType, SchemaField, SchemaOptions};
pub use self::defaults::DefaultValueStrategy;
pub use self::mapper::SchemaMapper;
pub use self::parser::SchemaParser;
pub use self::inference::SchemaInference;
pub use self::validator::SchemaValidator;

// Sub-modules
pub mod types;
pub mod defaults;
pub mod mapper;
pub mod parser;
pub mod inference;
pub mod validator;

// Internal module for shared utilities
mod utils;