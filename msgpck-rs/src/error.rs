use crate::marker::Marker;
use core::num::TryFromIntError;
use core::str::Utf8Error;

#[derive(Debug)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
#[non_exhaustive]
pub enum PackErr {
    /// Packed value was too big for target buffer.
    #[cfg_attr(feature = "std", error("Buffer Overflow"))]
    BufferOverflow,

    /// Failed to allocate memory while unpacking.
    #[cfg_attr(feature = "std", error("Out of memory"))]
    OutOfMemory,

    #[cfg(feature = "std")]
    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),

    #[cfg_attr(feature = "std", error("Error: {0}"))]
    Other(&'static str),
}

#[derive(Debug)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum UnpackErr {
    /// Tried to unpack a value into a buffer that was too small.
    #[cfg_attr(feature = "std", error("Buffer overflow"))]
    BufferOverflow,

    /// Reached EOF while trying to unpack a value.
    #[cfg_attr(feature = "std", error("Unexpected EOF"))]
    UnexpectedEof,

    #[cfg_attr(feature = "std", error("Wrong marker, got {0:?}"))]
    WrongMarker(Marker),

    #[cfg_attr(
        feature = "std",
        error("Encounted an integer value that was too large. {0}")
    )]
    IntTooBig(TryFromIntError),

    #[cfg_attr(
        feature = "std",
        error("There were {0} bytes remaining after unpacking.")
    )]
    TrailingBytes(usize),

    #[cfg_attr(feature = "std", error("Invalid UTF-8: {0}"))]
    InvalidUtf8(Utf8Error),

    #[cfg_attr(feature = "std", error("Unknown enum variant"))]
    UnknownVariant,

    #[cfg_attr(feature = "std", error("Not enough fields when deserializing struct or enum variant, got {got}, expected {expected}"))]
    MissingFields { got: usize, expected: usize },

    #[cfg_attr(feature = "std", error("Too many fields when deserializing struct or enum variant, got {got}, expected {expected}"))]
    TooManyFields { got: usize, expected: usize },

    #[cfg_attr(
        feature = "std",
        error(
            "Tried to deserialize an enum unit variant, but found a map instead of a discriminator."
        )
    )]
    ExpectedUnitVariant,

    #[cfg_attr(
        feature = "std",
        error("Tried to deserialize an enum variant with fields, but found a unit discriminator.")
    )]
    UnexpectedUnitVariant,

    #[cfg_attr(feature = "std", error("Error unpacking enum: Invalid header."))]
    InvalidEnumHeader,

    #[cfg_attr(feature = "std", error("{0}"))]
    Other(&'static str),
}

impl From<TryFromIntError> for UnpackErr {
    fn from(e: TryFromIntError) -> Self {
        UnpackErr::IntTooBig(e)
    }
}

impl From<Utf8Error> for UnpackErr {
    fn from(e: Utf8Error) -> Self {
        UnpackErr::InvalidUtf8(e)
    }
}
