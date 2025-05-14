// State machine and context for the decoding pipeline

use crate::internal::error::{Error, Result};
use crate::codec::varint; // Import varint for decoding tag and length
use crate::codec::types::{HtlvItem, HtlvValueType};
use bytes::BytesMut;
// Removed unused import: use bytes::Bytes; // Import Bytes for batch decoding alignment
use crate::codec::decode::basic_value_decoder; // Import the new basic value decoder module
use crate::codec::decode::batch_value_decoder; // Import the batch value decoder module
use crate::codec::decode::complex_value_handler::ComplexValueHandler; // Import the new complex value handler
use crate::codec::decode::large_field_handler::{LargeFieldHandler, LargeFieldProcessingResult}; // Import the new large field handler and its result enum
// Removed unused import: use std::mem; // Import std::mem


// Maximum allowed nesting depth to prevent DoS attacks
pub const MAX_NESTING_DEPTH: usize = 32;

/// Represents the state of the decoding pipeline.
#[derive(Debug, PartialEq)]
pub enum DecodeState {
    Scan, // Scan for the next item header (Tag + Type + Length)
    PrepareValue, // Prepare to decode the value based on type and length
    DecodeValue, // Decode the actual single value (basic type)
    DecodeBatchValue, // Decode a batch of values (batch decodable basic type)
    ProcessComplex, // Process a fully decoded complex item
    Done, // Decoding is complete
}

/// Represents a complex item being decoded on the stack.
#[derive(Debug)]
pub struct ComplexDecodeContext {
    pub tag: u64,
    pub value_type: HtlvValueType,
    pub end_offset: usize, // The offset in the original data where this complex value ends
    pub items: Vec<HtlvItem>,
    pub depth: usize, // Current nesting depth
}

/// Represents the context and state of the decoding process.
#[derive(Debug)]
pub struct DecodeContext {
    pub data: BytesMut, // The data being decoded
    pub current_offset: usize,
    pub state: DecodeState,
    pub complex_stack: Vec<ComplexDecodeContext>,
    pub root_item: Option<HtlvItem>,
    pub bytes_read_for_root_item: usize,

    // Information about the current item being processed
    pub current_item_tag: u64,
    pub current_item_type: Option<HtlvValueType>,
    pub current_item_length: u64, // Store the length of the current item (shard or regular)

    // State for decoding large fields
    pub decoding_large_field: bool,
    pub large_field_tag: u64,
    pub large_field_value_type: Option<HtlvValueType>,
    pub large_field_total_length: u64,
    pub large_field_buffer: BytesMut,
}

impl DecodeContext {
    /// Creates a new decoding context.
    pub fn new(data: &[u8]) -> Self {
        DecodeContext {
            data: BytesMut::from(data),
            current_offset: 0,
            state: DecodeState::Scan,
            complex_stack: Vec::new(),
            root_item: None,
            bytes_read_for_root_item: 0,
            current_item_tag: 0, // Initialize new field
            current_item_type: None, // Initialize new field
            current_item_length: 0,
            decoding_large_field: false,
            large_field_tag: 0,
            large_field_value_type: None,
            large_field_total_length: 0,
            large_field_buffer: BytesMut::new(),
        }
    }

    /// Handles the Scan state of the decoding process.
    pub fn handle_scan_state(&mut self) -> Result<()> {
        // Check if we have processed all data for the current complex item on top of the stack.
        if let Some(parent_context) = self.complex_stack.last() {
            if self.current_offset >= parent_context.end_offset {
                // Current complex item is fully processed, move to ProcessComplex state
                self.state = DecodeState::ProcessComplex;
                // println!("decode_item state transition: Scan -> ProcessComplex"); // Debug print
                return Ok(()); // Skip the rest of the Scan logic for this iteration
            }
        }

        // If stack is empty or current complex item is not done, scan for the next item header.
        if self.current_offset < self.data.len() {
            // --- Stage 1: Type Identification & Tag/Length Extraction ---
            // Decode Tag
            let (tag, tag_bytes) = varint::decode_varint(&self.data[self.current_offset..])
                .map_err(|e| Error::CodecError(format!("Failed to decode item Tag varint: {}", e)))?;
            let offset_after_tag = self.current_offset + tag_bytes;

            // Ensure there's enough data for the Type byte
            if self.data.len() < offset_after_tag + 1 {
                 return Err(Error::CodecError("Incomplete data for Type byte".to_string()));
            }

            // Decode Type
            let value_type_byte = self.data[offset_after_tag];
            let offset_after_type = offset_after_tag + 1;

            let value_type = HtlvValueType::from_byte(value_type_byte)
                .ok_or_else(|| Error::CodecError(format!("Unknown value type tag: {}", value_type_byte)))?;

            // Decode Length
            let (length, length_bytes) = varint::decode_varint(&self.data[offset_after_type..])
                .map_err(|e| Error::CodecError(format!("Failed to decode Length varint: {}", e)))?;
            let offset_after_length = offset_after_type + length_bytes;

            // Ensure there's enough data for the Value
            if self.data.len() < offset_after_length + length as usize {
                 return Err(Error::CodecError(format!("Incomplete data for Value (expected {} bytes)", length)));
            }

            // Store extracted info and transition to PrepareValue
            self.current_item_tag = tag; // Store the tag
            self.current_item_type = Some(value_type); // Store the type
            self.current_item_length = length; // Store the length
            self.current_offset = offset_after_length; // Advance offset past header
            self.state = DecodeState::PrepareValue; // Transition to prepare for value decoding
            // println!("decode_item state transition: Scan -> PrepareValue"); // Debug print

        } else {
            // If we are at the end of the data and the stack is empty, we are done.
            if self.complex_stack.is_empty() && !self.decoding_large_field {
                 self.state = DecodeState::Done;
                 // println!("decode_item state transition: Scan -> Done (stack empty)"); // Debug print
            } else if self.decoding_large_field {
                 // If we are at the end of the data but still decoding a large field, it's incomplete.
                 return Err(Error::CodecError(format!("Incomplete large field data. Expected {} bytes, got {}", self.large_field_total_length, self.large_field_buffer.len())));
            }
            else {
                // If we are at the end of the data but the stack is not empty, it means
                // a complex item was not fully decoded.
                 return Err(Error::CodecError("Incomplete data: Complex item not fully decoded".to_string()));
            }
        }
        Ok(())
    }

    /// Handles the PrepareValue state of the decoding process.
    pub fn handle_prepare_value_state(&mut self) -> Result<()> {
        // --- Stage 2: Prepare for Value Decoding ---
        let tag = self.current_item_tag;
        let value_type = self.current_item_type.unwrap();
        let length = self.current_item_length;
        let value_start = self.current_offset;
        let value_end = value_start + length as usize;
        let raw_value_slice = &self.data[value_start..value_end];

        if self.decoding_large_field {
            // If decoding a large field, use the large field handler.
            let result = LargeFieldHandler::process_shard(
                self.large_field_tag,
                self.large_field_value_type.unwrap(),
                self.large_field_total_length,
                &mut self.large_field_buffer,
                raw_value_slice,
                value_end, // Pass the offset after processing this shard
                &mut self.complex_stack, // Pass complex stack
            )?;

            match result {
                LargeFieldProcessingResult::Completed(decoded_item, bytes_read) => {
                    if self.complex_stack.is_empty() {
                        // This is the root large item
                        self.root_item = Some(decoded_item);
                        self.bytes_read_for_root_item = bytes_read;
                        self.state = DecodeState::Done; // Root large item decoded
                        // println!("decode_item state transition: PrepareValue -> Done (Root Large Field)"); // Debug print
                    } else {
                         // This was a nested large item, it's already added to the parent in LargeFieldHandler
                         self.state = DecodeState::Scan; // Continue decoding items at the current level
                         // println!("decode_item state transition: PrepareValue -> Scan (Nested Large Field)"); // Debug print
                    }

                    // Reset large field decoding state
                    self.decoding_large_field = false;
                    self.large_field_tag = 0;
                    self.large_field_value_type = None;
                    self.large_field_total_length = 0;
                    self.large_field_buffer = BytesMut::new();
                    self.current_item_length = 0; // Reset current item length
                }
                LargeFieldProcessingResult::Incomplete => {
                    // Still expecting more shards, stay in Scan state to read the next shard item
                    self.state = DecodeState::Scan; // Go back to scan for the next shard header
                    // println!("decode_item state transition: PrepareValue -> Scan (Expecting Shard)"); // Debug print
                }
            }

        } else {
            // Not decoding a large field, determine how to decode the value
            match value_type {
                HtlvValueType::Array | HtlvValueType::Object => {
                    // It's a complex type, use the complex value handler
                    ComplexValueHandler::handle_prepare_complex_value(self, tag, value_type, value_end)?;
                    self.state = DecodeState::Scan; // Transition to scan for nested items
                    // println!("decode_item state transition: PrepareValue -> Scan (Complex)"); // Debug print
                }
                HtlvValueType::U16 | HtlvValueType::U32 | HtlvValueType::U64 |
                HtlvValueType::I16 | HtlvValueType::I32 | HtlvValueType::I64 |
                HtlvValueType::F32 | HtlvValueType::F64 => {
                    // It's a batch decodable basic type
                    // self.current_offset = value_end; // Removed incorrect offset advance
                    self.state = DecodeState::DecodeBatchValue; // Transition to decode batch value
                    // println!("decode_item state transition: PrepareValue -> DecodeBatchValue (Batch)"); // Debug print
                }
                _ => {
                    // It's a single basic type
                    // self.current_offset = value_end; // Removed incorrect offset advance
                    self.state = DecodeState::DecodeValue; // Transition to decode the single value
                    // println!("decode_item state transition: PrepareValue -> DecodeValue (Single Basic)"); // Debug print
                }
            }
        }
        Ok(())
    }

    /// Handles the DecodeValue state of the decoding process.
    pub fn handle_decode_value_state(&mut self) -> Result<()> {
        // --- Stage 3: Single Value Decoding ---
        let tag = self.current_item_tag;
        let value_type = self.current_item_type.unwrap();
        let length = self.current_item_length;
        let value_start = self.current_offset; // Corrected value_start calculation
        let value_end = value_start + length as usize;
        let raw_value_slice = &self.data[value_start..value_end];

        // Use the new basic_value_decoder function
        let decoded_value = basic_value_decoder::decode_basic_value(value_type, length, raw_value_slice)?;

        self.current_offset = value_end; // Advance offset past the basic value

        if self.complex_stack.is_empty() {
            // This is the root item and it's basic
            self.root_item = Some(HtlvItem::new(tag, decoded_value));
            self.bytes_read_for_root_item = self.current_offset; // Record bytes read for this root item
            self.state = DecodeState::Done; // Root basic item decoded
            // println!("decode_item state transition: DecodeValue -> Done (Root Basic)"); // Debug print
        } else {
            // This is a nested basic item, add it to the current complex item on the stack
            let parent_context = self.complex_stack.last_mut().unwrap();
            parent_context.items.push(HtlvItem::new(tag, decoded_value));
            self.state = DecodeState::Scan; // Continue scanning for the next item at the current level
            // println!("decode_item state transition: DecodeValue -> Scan (Nested Basic)"); // Debug print
        }
        Ok(())
    }

    /// Handles the DecodeBatchValue state of the decoding process.
    pub fn handle_decode_batch_value_state(&mut self) -> Result<()> {
        // --- Stage 3 (Batch): Batch Value Decoding (4-stage pipeline) ---
        let tag = self.current_item_tag;
        let value_type = self.current_item_type.unwrap(); // This is the element type (e.g., U32)
        let length = self.current_item_length; // This is the total length of the batch value
        let value_start = self.current_offset; // Corrected value_start calculation
        let value_end = value_start + length as usize;
        let raw_value_slice = &self.data[value_start..value_end]; // Slice for the entire batch value

        // Use the new batch_value_decoder function
        let decoded_value = batch_value_decoder::decode_batch_value(value_type, length, raw_value_slice)?;

        self.current_offset = value_end; // Advance offset past the batch value

        if self.complex_stack.is_empty() {
            // This is the root item and it's a batch
            self.root_item = Some(HtlvItem::new(tag, decoded_value));
            self.bytes_read_for_root_item = self.current_offset; // Record bytes read for this root item
            self.state = DecodeState::Done; // Root batch item decoded
            // println!("decode_item state transition: DecodeBatchValue -> Done (Root Batch)"); // Debug print
        } else {
            // This is a nested batch item, add it to the current complex item on the stack
            let parent_context = self.complex_stack.last_mut().unwrap();
            parent_context.items.push(HtlvItem::new(tag, decoded_value));
            self.state = DecodeState::Scan; // Continue scanning for the next item at the current level
            // println!("decode_item state transition: DecodeBatchValue -> Scan (Nested Batch)"); // Debug print
        }
        Ok(())
    }

    /// Handles the ProcessComplex state of the decoding process.
    pub fn handle_process_complex_state(&mut self) -> Result<()> {
        // Use the complex value handler
        ComplexValueHandler::handle_process_complex_state(self)
    }

    // TODO: Add methods for scanning header, handling complex items, handling large fields, etc.
}
