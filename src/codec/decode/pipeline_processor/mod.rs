// Four-stage pipeline processor for batch decoding
//
// This module implements a complete four-stage pipeline for batch decoding:
// 1. Prefetch: Prepare data for efficient processing
// 2. Decode: Convert raw bytes to typed values
// 3. Dispatch: Process decoded values
// 4. Verify: Validate decoded data

use crate::internal::error::{Error, Result};
use crate::codec::types::HtlvValue;
use bytes::Bytes;

// Sub-modules
pub mod integer_processors;
pub mod signed_integer_processors;
pub mod float_processors;
pub mod batch_processor;

// Re-export the main function
pub use batch_processor::process_batch_value;

/// Trait for types that can be processed through the four-stage pipeline
pub trait PipelineProcessor: Sized {
    /// The type of the decoded elements
    type DecodedType;

    /// Stage 1: Prefetch data for efficient processing
    ///
    /// This stage prepares data for efficient processing, including:
    /// - Ensuring proper memory alignment
    /// - Prefetching data into CPU cache
    /// - Preparing data structures for batch processing
    ///
    /// Returns a tuple of:
    /// - The prepared data
    /// - Metadata needed for subsequent stages
    fn prefetch(data: &[u8]) -> Result<(Bytes, usize)>;

    /// Stage 2: Decode raw bytes to typed values
    ///
    /// This stage converts raw bytes to typed values, using:
    /// - SIMD instructions when available
    /// - Zero-copy techniques when possible
    /// - Fallback to scalar processing when necessary
    ///
    /// Returns a vector of decoded values and the number of bytes consumed
    fn decode(prepared_data: &Bytes) -> Result<(Vec<Self::DecodedType>, usize)>;

    /// Stage 3: Dispatch decoded values
    ///
    /// This stage processes decoded values, including:
    /// - Converting to HtlvValue types
    /// - Applying any necessary transformations
    ///
    /// Returns a vector of HtlvValue items
    fn dispatch(decoded_values: &[Self::DecodedType]) -> Vec<HtlvValue>;

    /// Stage 4: Verify decoded data
    ///
    /// This stage validates the decoded data, including:
    /// - Checking for data consistency
    /// - Validating against expected constraints
    ///
    /// Returns true if verification passes, false otherwise
    fn verify(decoded_values: &[Self::DecodedType], original_data: &[u8], bytes_consumed: usize) -> bool;

    /// Process data through the complete pipeline
    fn process_pipeline(data: &[u8]) -> Result<(Vec<HtlvValue>, usize)> {
        // Stage 1: Prefetch
        let (prepared_data, _expected_size) = Self::prefetch(data)?;

        // Stage 2: Decode
        let (decoded_values, bytes_consumed) = Self::decode(&prepared_data)?;

        // Stage 3: Dispatch
        let htlv_values = Self::dispatch(&decoded_values);

        // Stage 4: Verify
        if !Self::verify(&decoded_values, data, bytes_consumed) {
            return Err(Error::CodecError(format!(
                "Verification failed for {} batch decoding",
                std::any::type_name::<Self>()
            )));
        }

        Ok((htlv_values, bytes_consumed))
    }
}
