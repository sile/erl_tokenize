use std;
use num;
use trackable::error::{TrackableError, IntoTrackableError};
use trackable::error::{ErrorKind as TrackableErrorKind, ErrorKindExt};

pub type Error = TrackableError<ErrorKind>;

#[derive(Debug, Clone)]
pub enum ErrorKind {
    InvalidInput,
    UnexpectedEof,
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
