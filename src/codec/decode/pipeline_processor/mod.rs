// Four-stage pipeline processor for batch decoding
//
// This module implements a complete four-stage pipeline for batch decoding:
// 1. Prefetch: Prepare data for efficient processing
// 2. Decode: Convert raw bytes to typed values
// 3. Dispatch: Process decoded values
// 4. Verify: Validate decoded data

use crate::internal::error::{Error, Result};
use crate::codec::types::HtlvValue;

// Sub-modules
pub mod prefetch;
pub mod integer_processors;
pub mod signed_integer_processors;
pub mod float_processors;
pub mod batch_processor;

// Re-export key types and functions
pub use batch_processor::process_batch_value;
pub use prefetch::{AlignedBatch, prepare_aligned_batch, FromLeBytes, Pod};

/// Trait for types that can be processed through the four-stage pipeline
pub trait PipelineProcessor: Sized {
    /// The type of the decoded elements
    type DecodedType: Pod + FromLeBytes;

    /// Stage 1: Prefetch data for efficient processing
    ///
    /// This stage prepares data for efficient processing, including:
    /// - Ensuring proper memory alignment
    /// - Prefetching data into CPU cache
    /// - Preparing data structures for batch processing
    ///
    /// Returns an AlignedBatch containing the prepared data and the number of bytes consumed.
    /// The AlignedBatch provides a clear indication of whether the data is aligned or has been copied.
    fn prefetch(data: &[u8]) -> Result<(AlignedBatch<Self::DecodedType>, usize)> {
        // Default implementation uses the unified prepare_aligned_batch function
        prepare_aligned_batch::<Self::DecodedType>(data)
    }

    /// Stage 2: Decode raw bytes to typed values
    ///
    /// This stage converts raw bytes to typed values, using:
    /// - SIMD instructions when available and data is aligned
    /// - Zero-copy techniques when possible
    /// - Fallback to scalar processing when necessary
    ///
    /// Returns a vector of decoded values and the number of bytes consumed
    fn decode(aligned_batch: AlignedBatch<Self::DecodedType>) -> Result<(Vec<Self::DecodedType>, usize)>;

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
        let (aligned_batch, bytes_consumed) = Self::prefetch(data)?;

        // Stage 2: Decode
        let (decoded_values, _) = Self::decode(aligned_batch)?;

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
