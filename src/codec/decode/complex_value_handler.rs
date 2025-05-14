// Handler for complex HTLV values (Array and Object)

use crate::internal::error::{Error, Result};
use crate::codec::types::{HtlvItem, HtlvValueType, HtlvValue};
use crate::codec::decode::decoder_state_machine::{DecodeContext, DecodeState, ComplexDecodeContext, MAX_NESTING_DEPTH};

/// Handles the logic for decoding complex HTLV values (Array and Object).
pub struct ComplexValueHandler;

impl ComplexValueHandler {
    /// Handles the preparation for decoding a complex value.
    /// This involves pushing a new context onto the complex stack.
    pub fn handle_prepare_complex_value(
        ctx: &mut DecodeContext,
        tag: u64,
        value_type: HtlvValueType,
        value_end: usize,
    ) -> Result<()> {
        let next_depth = ctx.complex_stack.len() + 1;
        if next_depth > MAX_NESTING_DEPTH {
            return Err(Error::CodecError(format!("Maximum nesting depth ({}) exceeded", MAX_NESTING_DEPTH)));
        }

        ctx.complex_stack.push(ComplexDecodeContext {
            tag,
            value_type, // This will be Array or Object
            end_offset: value_end, // End of the complex value in the original data
            items: Vec::new(),
            depth: next_depth, // Set the current depth
        });
        ctx.current_offset = ctx.current_offset; // current_offset remains at value_start to process nested items
        ctx.state = DecodeState::Scan; // Start decoding items within this complex type
        // println!("decode_item state transition: PrepareValue -> Scan (Complex Array/Object)"); // Debug print
        Ok(())
    }

    /// Handles the processing of a fully decoded complex item.
    /// This involves popping the context from the stack and adding the decoded item to its parent.
    pub fn handle_process_complex_state(ctx: &mut DecodeContext) -> Result<()> {
        // A complex item on top of the stack is finished processing its children.
        let decoded_complex_context = ctx.complex_stack.pop().unwrap();
        let complex_value = match decoded_complex_context.value_type {
            HtlvValueType::Array => HtlvValue::Array(decoded_complex_context.items),
            HtlvValueType::Object => HtlvValue::Object(decoded_complex_context.items),
            _ => unreachable!(),
        };

        // Update current_offset to the end of the processed complex value
        ctx.current_offset = decoded_complex_context.end_offset;
        // println!("decode_item: Updated current_offset to end_offset = {}", ctx.current_offset); // Debug print


        if let Some(grandparent_context) = ctx.complex_stack.last_mut() {
            // Add the fully decoded complex item to its parent
            grandparent_context.items.push(HtlvItem::new(decoded_complex_context.tag, complex_value));
            ctx.state = DecodeState::Scan; // Continue decoding items at the grandparent level
            // println!("decode_item state transition: ProcessComplex -> Scan (Nested Complex)"); // Debug print
        } else {
            // The root complex item is fully decoded
            ctx.root_item = Some(HtlvItem::new(decoded_complex_context.tag, complex_value));
            ctx.bytes_read_for_root_item = ctx.current_offset; // Record bytes read for this root item
            ctx.state = DecodeState::Done;
            // println!("decode_item state transition: ProcessComplex -> Done (Root Complex)"); // Debug print
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::types::HtlvItem;
    use crate::codec::decode::decoder_state_machine::DecodeContext;
    use bytes::BytesMut;

    #[test]
    fn test_handle_prepare_complex_value() {
        let data = BytesMut::new();
        let mut ctx = DecodeContext::new(&data);
        let tag = 1;
        let value_type = HtlvValueType::Array;
        let value_end = 10;

        let result = ComplexValueHandler::handle_prepare_complex_value(&mut ctx, tag, value_type, value_end);
        assert!(result.is_ok());
        assert_eq!(ctx.complex_stack.len(), 1);
        let complex_ctx = &ctx.complex_stack[0];
        assert_eq!(complex_ctx.tag, tag);
        assert_eq!(complex_ctx.value_type, value_type);
        assert_eq!(complex_ctx.end_offset, value_end);
        assert_eq!(complex_ctx.items.len(), 0);
        assert_eq!(complex_ctx.depth, 1);
        assert_eq!(ctx.state, DecodeState::Scan);
    }

    #[test]
    fn test_handle_process_complex_state_root() {
        let data = BytesMut::new();
        let mut ctx = DecodeContext::new(&data);
        let tag = 1;
        let value_type = HtlvValueType::Array;
        let end_offset = 10;
        let items = vec![HtlvItem::new(2, HtlvValue::U8(1))];

        ctx.complex_stack.push(ComplexDecodeContext {
            tag,
            value_type,
            end_offset,
            items,
            depth: 1,
        });

        let result = ComplexValueHandler::handle_process_complex_state(&mut ctx);
        assert!(result.is_ok());
        assert_eq!(ctx.complex_stack.len(), 0);
        assert!(ctx.root_item.is_some());
        let root_item = ctx.root_item.unwrap();
        assert_eq!(root_item.tag, tag);
        if let HtlvValue::Array(decoded_items) = root_item.value {
            assert_eq!(decoded_items.len(), 1);
            assert_eq!(decoded_items[0].tag, 2);
            assert_eq!(decoded_items[0].value, HtlvValue::U8(1));
        } else {
            panic!("Root item is not an Array");
        }
        assert_eq!(ctx.bytes_read_for_root_item, end_offset);
        assert_eq!(ctx.state, DecodeState::Done);
    }

    #[test]
    fn test_handle_process_complex_state_nested() {
        let data = BytesMut::new();
        let mut ctx = DecodeContext::new(&data);

        // Grandparent context
        let grandparent_tag = 1;
        let grandparent_type = HtlvValueType::Object;
        let grandparent_end_offset = 20;
        ctx.complex_stack.push(ComplexDecodeContext {
            tag: grandparent_tag,
            value_type: grandparent_type,
            end_offset: grandparent_end_offset,
            items: Vec::new(),
            depth: 1,
        });

        // Parent context (being processed)
        let parent_tag = 2;
        let parent_type = HtlvValueType::Array;
        let parent_end_offset = 15;
        let parent_items = vec![HtlvItem::new(3, HtlvValue::U8(10))];
        ctx.complex_stack.push(ComplexDecodeContext {
            tag: parent_tag,
            value_type: parent_type,
            end_offset: parent_end_offset,
            items: parent_items,
            depth: 2,
        });

        let result = ComplexValueHandler::handle_process_complex_state(&mut ctx);
        assert!(result.is_ok());
        assert_eq!(ctx.complex_stack.len(), 1); // Only grandparent remains
        assert!(ctx.root_item.is_none()); // Not done yet

        let grandparent_context = ctx.complex_stack.last().unwrap();
        assert_eq!(grandparent_context.items.len(), 1); // Parent item added
        let parent_item = &grandparent_context.items[0];
        assert_eq!(parent_item.tag, parent_tag);
        if let HtlvValue::Array(decoded_items) = &parent_item.value {
            assert_eq!(decoded_items.len(), 1);
            assert_eq!(decoded_items[0].tag, 3);
            assert_eq!(decoded_items[0].value, HtlvValue::U8(10));
        } else {
            panic!("Parent item is not an Array");
        }
        assert_eq!(ctx.current_offset, parent_end_offset); // Offset updated to end of processed complex item
        assert_eq!(ctx.state, DecodeState::Scan); // Continue scanning at grandparent level
    }

    #[test]
    #[should_panic] // Expect a panic if stack is empty
    fn test_handle_process_complex_state_empty_stack() {
        let data = BytesMut::new();
        let mut ctx = DecodeContext::new(&data);
        // No complex context on the stack
        ComplexValueHandler::handle_process_complex_state(&mut ctx).unwrap();
    }
}