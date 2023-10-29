//! A light-weight library for serializing/deserializing types as MsgPack.
//!
//! The goal of this library is to be as light-weight as possible, while still
//! being easy to use. This is especially useful for embedded systems, where
//! binary size is important.
//!
//! To limit binary bloat, this library doesn't use serde. Instead, we provide
//! the [MsgPck] and [UnMsgPck] traits, which can be derived for most types.
//!
//! # Usage
//!
//! The easiest way to use this library is by deriving the [MsgPck] and
//! [UnMsgPck] traits on your types. This will automatically generate the
//! necessary code to serialize and deserialize your types.
//!
//! ```rust
//! use msgpck::{MsgPck, UnMsgPck};
//!
//! # #[derive(Debug, PartialEq)]
//! #[derive(MsgPck, UnMsgPck)]
//! struct Foo {
//!     a: bool,
//!     b: String,
//! }
//!
//! let foo = Foo { a: true, b: "hello".to_owned() };
//! let bytes = foo.pack_vec().unwrap();
//! # let foo2 = Foo::unpack_bytes(&bytes).unwrap();
//! # assert_eq!(foo, foo2);
//! ```
//!
//! # Compatibility with `rmp_serde`
//! We aim to be able to deserialize any value serialized using rmp_serde.
//!
//! *TODO: decide if we're gonna change serialized representation of enums*

mod impls;
mod marker;
mod pack;
mod unpack;
mod utils;
mod writers;

pub use crate::{
    impls::{EnumHeader, Variant},
    marker::Marker,
    pack::{MsgPck, PackError, SizeHint},
    unpack::{UnMsgPck, UnpackError},
    utils::{pack_array_header, slice_take, unpack_array_header, unpack_enum_header},
    writers::MsgWriter,
};

#[cfg(feature = "derive")]
pub use msgpck_derive::{MsgPck, UnMsgPck};
