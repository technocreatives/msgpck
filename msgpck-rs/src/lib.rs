//! A light-weight library for serializing/deserializing types as MsgPack.
//!
//! To limit binary bloat, this library doesn't use serde.
//! Insetead, we provide the [MsgPack] and [MsgUnpack] traits, which can be derived for most types.
//!
//! # Usage
//!
//! We also provide functions like [pack_vec] and [unpack_bytes] to convert between rust types and
//! msgpack bytes, but it's easy to define your own.
//!
//! Here is a simple example of an async pack function:
//! ```
//! #![feature(async_fn_in_trait)]
//! use msgpck_rs::MsgPack;
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
#![feature(impl_trait_in_assoc_type)]
#![feature(generators, iter_from_generator)]
#![feature(slice_take)]
#![allow(clippy::match_overlapping_arm)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod enums;
mod error;
mod impl_bool;
mod impl_bytes;
mod impl_floats;
mod impl_ints;
mod impl_strings;
mod impl_uints;
mod packers;
mod piece;
mod util;

#[cfg(feature = "alloc")]
mod impl_collections;

pub use enums::{EnumHeader, Variant};
pub use error::UnpackErr;
pub use msgpck_rs_derive::{MsgPack, MsgUnpack};
#[cfg(feature = "alloc")]
pub use packers::pack_vec;
pub use packers::unpack_bytes;
pub use piece::Piece;
pub use rmp::Marker;

/// Trait for serializing a type using msgpack.
pub trait MsgPack {
    type Iter<'a>: Iterator<Item = Piece<'a>>
    where
        Self: 'a;

    /// Returns an iterator of msgpack [Piece]s. Collect them all to produce a valid msgpack value.
    ///
    /// ```
    /// use msgpck_rs::MsgPack;
    /// let mut encoded = vec![];
    /// for m in vec![0xDDu8, 0xEE, 3].pack() {
    ///     encoded.extend_from_slice(m.as_bytes());
    /// }
    /// assert_eq!(encoded, [0x93, 0xCC, 0xDD, 0xCC, 0xEE, 3]);
    /// ```
    fn pack(&self) -> Self::Iter<'_>;
}

/// Trait for deserializing a type using msgpack.
pub trait MsgUnpack<'buf> {
    /// Unpack a value from a msgpack bytes slice
    ///
    /// ```
    /// use msgpck_rs::MsgUnpack;
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
    pub use crate::enums::{pack_enum_header, unpack_enum_header};
    pub use crate::util::{
        pack_array_header, pack_map_header, unpack_array_header, unpack_map_header,
    };
}
