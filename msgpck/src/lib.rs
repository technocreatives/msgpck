mod impls;
mod marker;
mod pack;
mod unpack;
mod utils;
mod writers;

pub use crate::{
    marker::Marker,
    pack::{MsgPck, PackError},
    unpack::{UnMsgPck, UnpackError},
    utils::{slice_take, unpack_array_header},
    writers::MsgWriter,
};

#[cfg(feature = "derive")]
pub use msgpck_derive::{MsgPck, UnMsgPck};
