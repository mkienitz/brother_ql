//! This module provides crate-specific error types.
use thiserror::Error;

/// The crate-level error type
#[derive(Error, Clone, Eq, PartialEq, Hash, Debug)]
pub enum BQLError {
    /// Returned when the provided image has a size incompatible with the provided media type.
    #[error("media and image are not compatible")]
    DimensionMismatch,
    /// Returned when there is an issue with a received status information
    #[error("Received status information is malformed: {0}")]
    MalformedStatus(String),
}
