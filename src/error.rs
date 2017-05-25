use std;
use num;
use trackable::error::{TrackableError, IntoTrackableError};
use trackable::error::{ErrorKind as TrackableErrorKind, ErrorKindExt};

/// This crate specific error type.
pub type Error = TrackableError<ErrorKind>;

/// The list of the possible error kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// Input text is invalid.
    InvalidInput,

    /// Unexpected End-Of-String.
    UnexpectedEos,
}

impl TrackableErrorKind for ErrorKind {}
impl IntoTrackableError<std::num::ParseIntError> for ErrorKind {
    fn into_trackable_error(e: std::num::ParseIntError) -> Error {
        ErrorKind::InvalidInput.cause(e)
    }
}
impl IntoTrackableError<std::num::ParseFloatError> for ErrorKind {
    fn into_trackable_error(e: std::num::ParseFloatError) -> Error {
        ErrorKind::InvalidInput.cause(e)
    }
}
impl IntoTrackableError<num::bigint::ParseBigIntError> for ErrorKind {
    fn into_trackable_error(e: num::bigint::ParseBigIntError) -> Error {
        ErrorKind::InvalidInput.cause(e)
    }
}
