// Decode module for HTLV (HyperNova) data format

pub mod basic; // Keep basic.rs for now, will modify its content later
pub mod basic_types; // Introduce the new basic_types module
pub mod complex; // complex module will be refactored or removed later
pub mod complex_types; // Declare the new complex_types module

use crate::internal::error::{Error, Result};
use crate::codec::varint;
use crate::codec::types::{HtlvItem, HtlvValueType, HtlvValue};
// Removed unused import: use bytes::Bytes;

/// Represents the state of the decoding pipeline.
#[derive(Debug, PartialEq)]
enum DecodeState {
    Scan,
    DecodeHeader,
    // Removed DecodeValue state as it's no longer used in the iterative approach
    ProcessComplex,
    Done,
}

/// Represents a complex item being decoded on the stack.
#[derive(Debug)]
struct ComplexDecodeContext {
    tag: u64,
    value_type: HtlvValueType,
    end_offset: usize, // The offset in the original data where this complex value ends
    items: Vec<HtlvItem>,
}

/// Decodes bytes into an HTLV item (Tag + Type + Length + Value) using an iterative approach
/// with a state machine to simulate a multi-stage pipeline and handle nested structures.
/// Returns the decoded HtlvItem and the number of bytes read.
pub fn decode_item(data: &[u8]) -> Result<(HtlvItem, usize)> {
    let mut current_offset = 0;
    let mut state = DecodeState::Scan;
    let mut complex_stack: Vec<ComplexDecodeContext> = Vec::new();
    let mut root_item: Option<HtlvItem> = None;

    while state != DecodeState::Done {
        println!("decode_item loop: current_offset = {}, state = {:?}", current_offset, state);
        match state {
            DecodeState::Scan => {
                // Check if we have processed all data for the current complex item on top of the stack.
                if let Some(parent_context) = complex_stack.last() {
                    if current_offset >= parent_context.end_offset {
                        // Current complex item is fully processed, move to ProcessComplex state
                        state = DecodeState::ProcessComplex;
                        println!("decode_item state transition: Scan -> ProcessComplex");
                        continue; // Skip the rest of the Scan logic for this iteration
                    }
                }

                // If stack is empty or current complex item is not done, scan for the next item header.
                if current_offset < data.len() {
                    state = DecodeState::DecodeHeader;
                    println!("decode_item state transition: Scan -> DecodeHeader");
                } else {
                    // If we are at the end of the data and the stack is empty, we are done.
                    if complex_stack.is_empty() {
                         state = DecodeState::Done;
                         println!("decode_item state transition: Scan -> Done (stack empty)");
                    } else {
                        // If we are at the end of the data but the stack is not empty, it means
                        // a complex item was not fully decoded.
                         return Err(Error::CodecError("Incomplete data: Complex item not fully decoded".to_string()));
                    }
                }
            }
            DecodeState::DecodeHeader => {
                // Decode Tag
                let (tag, tag_bytes) = varint::decode_varint(&data[current_offset..])
                    .map_err(|e| Error::CodecError(format!("Failed to decode item Tag varint: {}", e)))?;
                current_offset += tag_bytes;

                // Ensure there's enough data for the Type byte
                if data.len() < current_offset + 1 {
                     return Err(Error::CodecError("Incomplete data for Type byte".to_string()));
                }

                // Decode Type
                let value_type_byte = data[current_offset];
                current_offset += 1;

                let value_type = HtlvValueType::from_byte(value_type_byte)
                    .ok_or_else(|| Error::CodecError(format!("Unknown value type tag: {}", value_type_byte)))?;

                // Decode Length
                let remaining_data_after_type = &data[current_offset..];
                let (length, length_bytes) = varint::decode_varint(remaining_data_after_type)
                    .map_err(|e| Error::CodecError(format!("Failed to decode Length varint: {}", e)))?;
                current_offset += length_bytes;

                // Ensure there's enough data for the Value
                if data.len() < current_offset + length as usize {
                    return Err(Error::CodecError(format!("Incomplete data for Value (expected {} bytes)", length)));
                }

                // Get the Value slice (zero-copy)
                let value_start = current_offset;
                let value_end = current_offset + length as usize;
                // Note: current_offset is NOT advanced past the value bytes here for complex types.
                // It stays at value_start so the next Scan can process the nested items.

                // Based on the type, decide the next state and how to handle the value
                match value_type {
                    HtlvValueType::Array | HtlvValueType::Object => {
                        // It's a complex type, push a new context onto the stack
                        complex_stack.push(ComplexDecodeContext {
                            tag,
                            value_type,
                            end_offset: value_end, // End of the complex value in the original data
                            items: Vec::new(),
                        });
                        // current_offset remains at value_start to process nested items
                        state = DecodeState::Scan; // Start scanning for items within this complex type
                        println!("decode_item state transition: DecodeHeader -> Scan (Complex)");
                    }
                    _ => {
                        // It's a basic type, decode the value immediately
                        let raw_value_slice = &data[value_start..value_end];
                        // Call the appropriate decoding function from basic_types module
                        let decoded_value = match value_type {
                            HtlvValueType::Null => basic_types::null::decode_null(length)?,
                            HtlvValueType::Bool => basic_types::boolean::decode_bool(length, raw_value_slice)?,
                            HtlvValueType::U8 => basic_types::u8::decode_u8(length, raw_value_slice)?,
                            HtlvValueType::U16 => basic_types::u16::decode_u16(length, raw_value_slice)?,
                            HtlvValueType::U32 => basic_types::u32::decode_u32(length, raw_value_slice)?,
                            HtlvValueType::U64 => basic_types::u64::decode_u64(length, raw_value_slice)?,
                            HtlvValueType::I8 => basic_types::i8::decode_i8(length, raw_value_slice)?,
                            HtlvValueType::I16 => basic_types::i16::decode_i16(length, raw_value_slice)?,
                            HtlvValueType::I32 => basic_types::i32::decode_i32(length, raw_value_slice)?,
                            HtlvValueType::I64 => basic_types::i64::decode_i64(length, raw_value_slice)?,
                            HtlvValueType::F32 => basic_types::floats::decode_f32(length, raw_value_slice)?,
                            HtlvValueType::F64 => basic_types::floats::decode_f64(length, raw_value_slice)?,
                            HtlvValueType::Bytes => basic_types::bytes_and_string::decode_bytes(raw_value_slice)?,
                            HtlvValueType::String => basic_types::bytes_and_string::decode_string(raw_value_slice)?,
                            _ => unreachable!("Complex types should be handled in the complex branch"),
                        };

                        current_offset = value_end; // Advance offset past the basic value

                        if complex_stack.is_empty() {
                            // This is the root item and it's basic
                            root_item = Some(HtlvItem::new(tag, decoded_value));
                            state = DecodeState::Done; // Root basic item decoded
                            println!("decode_item state transition: DecodeHeader -> Done (Root Basic)");
                        } else {
                            // This is a nested basic item, add it to the current complex item on the stack
                            let parent_context = complex_stack.last_mut().unwrap();
                            parent_context.items.push(HtlvItem::new(tag, decoded_value));
                            state = DecodeState::Scan; // Continue scanning for the next item at the current level
                            println!("decode_item state transition: DecodeHeader -> Scan (Nested Basic)");
                        }
                    }
                }
            }
            DecodeState::ProcessComplex => {
                // A complex item on top of the stack is finished processing its children.
                let decoded_complex_context = complex_stack.pop().unwrap();
                let complex_value = match decoded_complex_context.value_type {
                    HtlvValueType::Array => HtlvValue::Array(decoded_complex_context.items),
                    HtlvValueType::Object => HtlvValue::Object(decoded_complex_context.items),
                    _ => unreachable!(),
                };

                // Update current_offset to the end of the processed complex value
                current_offset = decoded_complex_context.end_offset;
                println!("decode_item: Updated current_offset to end_offset = {}", current_offset);


                if let Some(grandparent_context) = complex_stack.last_mut() {
                    // Add the fully decoded complex item to its parent
                    grandparent_context.items.push(HtlvItem::new(decoded_complex_context.tag, complex_value));
                    state = DecodeState::Scan; // Continue scanning for the next item at the grandparent level
                    println!("decode_item state transition: ProcessComplex -> Scan (Nested Complex)");
                } else {
                    // The root complex item is fully decoded
                    root_item = Some(HtlvItem::new(decoded_complex_context.tag, complex_value));
                    state = DecodeState::Done;
                    println!("decode_item state transition: ProcessComplex -> Done (Root Complex)");
                }
            } // Added closing brace for ProcessComplex arm
            DecodeState::Done => {
                // Should exit the loop here
            }
        }
    }

    // Ensure all data was consumed for the root item
    if current_offset != data.len() {
         return Err(Error::CodecError(format!("Extra data remaining after decoding: {} bytes", data.len() - current_offset)));
    }


    root_item.ok_or_else(|| Error::CodecError("Decoding failed: No root item decoded".to_string()))
        .map(|item| (item, current_offset))
}


// TODO: Implement full HTLV decoding for various data types and structures
// TODO: Implement zero-copy parsing for complex structures
// TODO: Implement SIMD optimization for decoding (placeholders added in basic.rs)
// TODO: Design and implement a multi-stage pipeline for decoding, especially for Array/Object
// TODO: Add more decoding related functions and structures