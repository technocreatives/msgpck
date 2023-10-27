mod impls;
mod marker;
mod pack;
mod unpack;
mod utils;
mod writers;

pub use crate::{
    impls::{EnumHeader, Variant},
    marker::Marker,
    pack::{MsgPck, PackError},
    unpack::{UnMsgPck, UnpackError},
    utils::{pack_array_header, slice_take, unpack_array_header, unpack_enum_header},
    writers::MsgWriter,
};

#[cfg(feature = "derive")]
pub use msgpck_derive::{MsgPck, UnMsgPck};
