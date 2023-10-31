use core::{num::TryFromIntError, str::Utf8Error};

use crate::{marker::Marker, utils::UnexpectedEofError};

#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum UnpackError {
    #[cfg_attr(feature = "std", error("Wrong marker, got {0:?}"))]
    WrongMarker(Marker),
    #[cfg_attr(feature = "std", error("Unexpected end of data"))]
    UnexpectedEof,
    #[cfg_attr(feature = "std", error("Integer too big"))]
    IntTooBig { source: TryFromIntError },

    #[cfg(feature = "debug")]
    #[cfg_attr(feature = "std", error("Invalid UTF-8"))]
    Utf8Error {
        #[cfg_attr(feature = "defmt", defmt(Display2Format))]
        source: Utf8Error,
    },
    #[cfg(not(feature = "debug"))]
    #[cfg_attr(feature = "std", error("Invalid UTF-8"))]
    Utf8Error,

    // TODO: Make this work with derive
    // #[cfg(feature = "debug")]
    // MissingFields {
    //     got: usize,
    //     expected: usize,
    // },
    // #[cfg(not(feature = "debug"))]
    #[cfg_attr(feature = "std", error("Missing fields"))]
    MissingFields,

    // TODO: Make this work with derive
    // #[cfg(feature = "debug")]
    // TooManyFields {
    //     got: usize,
    //     expected: usize,
    // },
    // #[cfg(not(feature = "debug"))]
    #[cfg_attr(feature = "std", error("Too many fields"))]
    TooManyFields,

    #[cfg_attr(feature = "std", error("Unexpected unit variant"))]
    UnexpectedUnitVariant,
    #[cfg_attr(feature = "std", error("Expected unit variant"))]
    ExpectedUnitVariant,
    #[cfg_attr(feature = "std", error("Unknown variant"))]
    UnknownVariant,
}

impl From<TryFromIntError> for UnpackError {
    fn from(source: TryFromIntError) -> Self {
        Self::IntTooBig { source }
    }
}

impl From<UnexpectedEofError> for UnpackError {
    fn from(_: UnexpectedEofError) -> Self {
        Self::UnexpectedEof
    }
}

impl From<Utf8Error> for UnpackError {
    #[cfg(feature = "debug")]
    fn from(source: Utf8Error) -> Self {
        Self::Utf8Error { source }
    }

    #[cfg(not(feature = "debug"))]
    fn from(_: Utf8Error) -> Self {
        Self::Utf8Error
    }
}
