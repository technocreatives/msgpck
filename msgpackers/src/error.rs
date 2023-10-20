use core::num::TryFromIntError;

use rmp::Marker;

#[derive(Debug, thiserror::Error)]
pub enum UnpackErr {
    #[error("Unexpected EOF")]
    UnexpectedEof,
    #[error("Wrong marker, got {0:?}")]
    WrongMarker(Marker),
    #[error("Encounted an integer value that was too large. {0}")]
    IntTooBig(#[from] TryFromIntError),
}
