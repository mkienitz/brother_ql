use thiserror::Error;

#[derive(Error, Debug)]
pub enum BQLError {
    #[error("media and image are not compatible")]
    DimensionMismatch,
}
