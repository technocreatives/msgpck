//! A light-weight library for serializing/deserializing types as MsgPack.
//!
//! To limit binary bloat, this library doesn't use serde.
//! Insetead, we provide the [MsgPack] and [MsgUnpack] traits, which can be derived for most types.
//!
//! # Usage
//!
//! We also provide functions like [pack_vec] and [unpack_slice] to convert between rust types and
//! msgpack bytes, but it's easy to define your own.
//!
//! Here is a simple example of an async pack function:
//! ```
//! use msgpck::MsgPack;
//!
//! trait AsyncWrite {
//!     async fn write(&mut self, bytes: &[u8]);
//! }
//!
//! async fn async_pack(writer: &mut impl AsyncWrite, msg: &impl MsgPack) {
//!     for piece in msg.pack() {
//!         writer.write(piece.as_bytes()).await;
//!     }
//! }
//! ```
//!
//! # Compatibility with `rmp_serde`
//! We aim to be able to deserialize any value serialized using rmp_serde.
//!
//! *TODO: decide if we're gonna change serialized representation of enums*

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::match_overlapping_arm)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod enums;
mod error;
mod impls;
mod marker;
mod packers;
mod piece;
mod util;
mod write;

pub use enums::{EnumHeader, Variant};
pub use error::{PackErr, UnpackErr};
pub use marker::Marker;
pub use msgpck_derive::{MsgPack, MsgUnpack};
pub use packers::*;
pub use piece::Piece;
pub use write::Write;

/// Trait for serializing a type using msgpack.
pub trait MsgPack {
    /// Returns an iterator of msgpack [Piece]s. Collect them all to produce a valid msgpack value.
    ///
    /// ```
    /// use msgpck::MsgPack;
    /// let mut encoded = vec![];
    /// for m in vec![0xDDu8, 0xEE, 3].pack() {
    ///     encoded.extend_from_slice(m.as_bytes());
    /// }
    /// println!("{encoded:x?}");
    /// assert_eq!(encoded, [0x93, 0xcc, 0xdd, 0xcc, 0xee, 0x03]);
    /// ```
    fn pack(&self) -> impl Iterator<Item = Piece<'_>>;

    /// Pack this value into a [Write], and return how many bytes were packed.
    ///
    /// When async packing is not needed, packing directly into a writer can be faster than calling
    /// [MsgPack::pack]. Note that the default implementation just calls [MsgPack::pack], but
    /// should be overridden if performance is a concern.
    ///
    /// # Errors
    /// TODO: decide on whether to this takes a `impl Write` or a `dyn Write`, and in the latter
    /// case, how we should handle errors.
    fn pack_with_writer(&self, w: &mut dyn Write) -> Result<usize, PackErr> {
        let mut n = 0;
        for piece in self.pack() {
            w.write_all(piece.as_bytes())?;
            n += piece.as_bytes().len();
        }
        Ok(n)
    }
}

/// Trait for deserializing a type using msgpack.
pub trait MsgUnpack<'buf> {
    /// Unpack a value from a msgpack bytes slice
    ///
    /// ```
    /// use msgpck::MsgUnpack;
    /// let encoded = [0x93, 0xCC, 0xDD, 0xCC, 0xEE, 3];
    /// let decoded: Vec<u8> = Vec::unpack(&mut &encoded[..]).unwrap();
    /// assert_eq!(decoded, &[0xDDu8, 0xEE, 3]);
    /// ```
    fn unpack(bytes: &mut &'buf [u8]) -> Result<Self, UnpackErr>
    where
        Self: Sized;
}

/// Helpers for packing/unpacking certain msgpack values.
///
/// This module is used by the derive macros for [MsgPack] and [MsgUnpack].
/// Unless you are implementing those traits by hand, you probably shouldn't be here.
pub mod helpers {
    pub use crate::enums::{pack_enum_header, pack_enum_header_to_writer, unpack_enum_header};
    pub use crate::impls::ints::{pack_i64, unpack_i64};
    pub use crate::impls::uints::{pack_u64, unpack_u64};
    pub use crate::util::{
        pack_array_header, pack_map_header, unpack_array_header, unpack_map_header,
    };
}
