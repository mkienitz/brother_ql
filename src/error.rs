//! This module provides crate-specific error types.
use thiserror::Error;

/// The crate-level error type
#[derive(Error, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum BQLError {
    /// Returned when the provided image has a size incompatible with the provided media type.
    #[error("media and image are not compatible")]
    DimensionMismatch,
}
