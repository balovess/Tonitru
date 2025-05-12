use bytes::Bytes; // Import Bytes

/// Represents a single HTLV (HyperNova) data item.
#[derive(Debug, PartialEq, Clone)] // Added Clone derive
pub struct HtlvItem {
    pub tag: u64,
    pub value: HtlvValue,
}

/// Represents the value part of an HTLV data item.
#[derive(Debug, PartialEq, Clone)] // Added Clone derive
pub enum HtlvValue {
    Null,
    Bool(bool),
    U8(u8),   // Add U8 type
    U16(u16), // Add U16 type
    U32(u32), // Add U32 type
    U64(u64),
    I8(i8),   // Add I8 type
    I16(i16), // Add I16 type
    I32(i32), // Add I32 type
    I64(i64),
    F32(f32), // Add F32 type
    F64(f64),
    Bytes(Bytes), // Use Bytes for zero-copy
    String(Bytes), // Use Bytes for zero-copy string data
    Array(Vec<HtlvItem>), // Add Array type
    Object(Vec<HtlvItem>), // Add Object type (representing fields)
    // TODO: Add support for other complex types like maps
}

impl HtlvValue {
    /// Returns the corresponding HtlvValueType for the HtlvValue.
    pub fn value_type(&self) -> HtlvValueType {
        match self {
            HtlvValue::Null => HtlvValueType::Null,
            HtlvValue::Bool(_) => HtlvValueType::Bool,
            HtlvValue::U8(_) => HtlvValueType::U8,
            HtlvValue::U16(_) => HtlvValueType::U16,
            HtlvValue::U32(_) => HtlvValueType::U32,
            HtlvValue::U64(_) => HtlvValueType::U64,
            HtlvValue::I8(_) => HtlvValueType::I8,
            HtlvValue::I16(_) => HtlvValueType::I16,
            HtlvValue::I32(_) => HtlvValueType::I32,
            HtlvValue::I64(_) => HtlvValueType::I64,
            HtlvValue::F32(_) => HtlvValueType::F32,
            HtlvValue::F64(_) => HtlvValueType::F64,
            HtlvValue::Bytes(_) => HtlvValueType::Bytes,
            HtlvValue::String(_) => HtlvValueType::String,
            HtlvValue::Array(_) => HtlvValueType::Array,
            HtlvValue::Object(_) => HtlvValueType::Object,
        }
    }
}

/// Defines the byte representation for each HtlvValue type.
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum HtlvValueType {
    Null = 0,
    Bool = 1,
    U8 = 2,   // Assign byte for U8
    U16 = 3,  // Assign byte for U16
    U32 = 4,  // Assign byte for U32
    U64 = 5,
    I8 = 6,   // Assign byte for I8
    I16 = 7,  // Assign byte for I16
    I32 = 8,  // Assign byte for I32
    I64 = 9,
    F32 = 10, // Assign byte for F32
    F64 = 11,
    Bytes = 12, // Update byte for Bytes
    String = 13, // Update byte for String
    Array = 14, // Update byte for Array
    Object = 15, // Update byte for Object
    // TODO: Assign type bytes for other complex types if needed
}

impl HtlvValueType {
    /// Converts a byte into an HtlvValueType.
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0 => Some(HtlvValueType::Null),
            1 => Some(HtlvValueType::Bool),
            2 => Some(HtlvValueType::U8),
            3 => Some(HtlvValueType::U16),
            4 => Some(HtlvValueType::U32),
            5 => Some(HtlvValueType::U64),
            6 => Some(HtlvValueType::I8),
            7 => Some(HtlvValueType::I16),
            8 => Some(HtlvValueType::I32),
            9 => Some(HtlvValueType::I64),
            10 => Some(HtlvValueType::F32),
            11 => Some(HtlvValueType::F64),
            12 => Some(HtlvValueType::Bytes),
            13 => Some(HtlvValueType::String),
            14 => Some(HtlvValueType::Array),
            15 => Some(HtlvValueType::Object),
            _ => None, // Unknown type
        }
    }
}


impl HtlvItem {
    /// Creates a new HTLV item.
    pub fn new(tag: u64, value: HtlvValue) -> Self {
        HtlvItem { tag, value }
    }
}

// Note: Encoding and decoding logic for HtlvItem will be implemented in encode.rs and decode.rs