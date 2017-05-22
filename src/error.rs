
use trackable::error::{TrackableError, IntoTrackableError};
use trackable::error::{ErrorKind as TrackableErrorKind, ErrorKindExt};

pub type Error = TrackableError<ErrorKind>;

#[derive(Debug, Clone)]
pub enum ErrorKind {
    InvalidInput,
    UnexpectedEof,
    Other,
}

impl TrackableErrorKind for ErrorKind {}
// impl IntoTrackableError<std::io::Error> for ErrorKind {
//     fn into_trackable_error(from: std::io::Error) -> Error {

//         ErrorKind::Other.cause(from)

//     }
// }
