use std::{num::TryFromIntError, str::Utf8Error};

use crate::{marker::Marker, utils::UnexpectedEofError};

pub trait UnMsgPck<'buf> {
    fn unpack(source: &mut &'buf [u8]) -> Result<Self, UnpackError>
    where
        Self: Sized;
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub enum UnpackError {
    WrongMarker(Marker),
    UnexpectedEof,
    IntTooBig(TryFromIntError),

    #[cfg(feature = "debug")]
    Utf8Error(Utf8Error),
    #[cfg(not(feature = "debug"))]
    Utf8Error,

    // TODO: Make this work with derive
    // #[cfg(feature = "debug")]
    // MissingFields {
    //     got: usize,
    //     expected: usize,
    // },
    // #[cfg(not(feature = "debug"))]
    MissingFields,

    // TODO: Make this work with derive
    // #[cfg(feature = "debug")]
    // TooManyFields {
    //     got: usize,
    //     expected: usize,
    // },
    // #[cfg(not(feature = "debug"))]
    TooManyFields,

    UnexpectedUnitVariant,
    ExpectedUnitVariant,
    UnknownVariant,
}

impl From<TryFromIntError> for UnpackError {
    fn from(e: TryFromIntError) -> Self {
        Self::IntTooBig(e)
    }
}

impl From<UnexpectedEofError> for UnpackError {
    fn from(_: UnexpectedEofError) -> Self {
        Self::UnexpectedEof
    }
}

impl From<Utf8Error> for UnpackError {
    #[cfg(feature = "debug")]
    fn from(e: Utf8Error) -> Self {
        Self::Utf8Error(e)
    }

    #[cfg(not(feature = "debug"))]
    fn from(e: Utf8Error) -> Self {
        Self::Utf8Error
    }
}
