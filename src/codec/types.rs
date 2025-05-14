use bytes::Bytes;
use bitflags::bitflags;

/// Represents a single HTLV (HyperNova) data item.
/// This struct is used internally for representing parsed HTLV values,
/// especially within complex types like Arrays and Objects.
#[derive(Debug, PartialEq, Clone)]
pub struct HtlvItem {
    pub tag: u64,
    pub value: HtlvValue,
}

/// Represents the value part of an HTLV data item.
/// This enum covers various basic and complex data types supported by HTLV.
#[derive(Debug, PartialEq, Clone)]
pub enum HtlvValue {
    Null,
    Bool(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    Bytes(Bytes),
    String(Bytes),
    Array(Vec<HtlvItem>),
    Object(Vec<HtlvItem>),
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
    U8 = 2,
    U16 = 3,
    U32 = 4,
    U64 = 5,
    I8 = 6,
    I16 = 7,
    I32 = 8,
    I64 = 9,
    F32 = 10,
    F64 = 11,
    Bytes = 12,
    String = 13,
    Array = 14,
    Object = 15,
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

bitflags! {
    /// Flags for HTLV data blocks.
    ///
    /// These flags indicate properties of the Value field, such as whether it is
    /// a nested structure, compressed, or encrypted.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct HTLVFlag: u8 {
        /// Indicates that the Value is a nested HTLV structure.
        const NESTED    = 0b00000001;
        /// Indicates that the Value has been compressed.
        const COMPRESSED = 0b00000010;
        /// Indicates that the Value has been encrypted.
        const ENCRYPTED = 0b00000100;
        // Bits 3-7 are reserved for future use.
    }
}

/// Represents a complete HTLV data block.
///
/// An HTLV block consists of a Tag, Flags, Length, and Value. The Value
/// can either be raw bytes or a sequence of nested HTLV blocks if the
/// `NESTED` flag is set.
#[derive(Debug, PartialEq, Clone)]
pub struct HTLVBlock {
    /// The tag identifying the field.
    pub tag: u16,
    /// Flags indicating properties of the Value (e.g., nested, compressed, encrypted).
    pub flags: HTLVFlag,
    /// The length of the raw Value bytes. This is encoded using Variable Length Integer (VLQ).
    pub length: u64,
    /// The raw byte sequence of the Value. If the `NESTED` flag is set, this
    /// contains the encoded bytes of the nested HTLV blocks.
    pub value: Vec<u8>,
    /// A vector of nested HTLV blocks, present only if the `NESTED` flag is set
    /// and the Value bytes have been successfully decoded into blocks.
    pub nested: Vec<HTLVBlock>,
}

impl HTLVBlock {
    /// Creates a new HTLV block.
    ///
    /// The `length` field is automatically calculated based on the length of the
    /// provided `value` byte vector.
    pub fn new(tag: u16, flags: HTLVFlag, value: Vec<u8>, nested: Vec<HTLVBlock>) -> Self {
        let length = value.len() as u64;
        HTLVBlock {
            tag,
            flags,
            length,
            value,
            nested,
        }
    }
}