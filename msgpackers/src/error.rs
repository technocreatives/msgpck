use core::num::TryFromIntError;
use std::str::Utf8Error;

use rmp::Marker;

#[derive(Debug, thiserror::Error)]
pub enum UnpackErr {
    #[error("Unexpected EOF")]
    UnexpectedEof,

    #[error("Wrong marker, got {0:?}")]
    WrongMarker(Marker),

    #[error("Encounted an integer value that was too large. {0}")]
    IntTooBig(#[from] TryFromIntError),

    #[error("There were {0} bytes remaining after unpacking.")]
    TrailingBytes(usize),

    #[error("Invalid UTF-8: {0}")]
    InvalidUtf8(#[from] Utf8Error),

    #[error("Unknown enum variant")]
    UnknownVariant,
}
