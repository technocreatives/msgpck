//! A light-weight library for serializing/deserializing types as
//! [MessagePack](https://msgpack.org/) ("msgpack").
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
//! let bytes = msgpck::pack_vec(&foo).unwrap();
//! # let foo2: Foo = msgpck::unpack_bytes(&bytes).unwrap();
//! # assert_eq!(foo, foo2);
//! ```
//!
//! ## Custom implementation
//!
//! Here is an example of a custom implementation for a unit struct:
//!
//! ```
//! # use proptest::prelude::*;
//! # use proptest_derive::Arbitrary;
//! use msgpck::{MsgPck, UnMsgPck, MsgWriter, PackError, Marker, UnpackError, slice_take};
//!
//! # #[derive(Arbitrary, Debug, PartialEq)]
//! struct User { id: u32, name: String }
//!
//! impl MsgPck for User {
//!     fn pack(&self, writer: &mut dyn MsgWriter) -> Result<(), PackError> {
//!         // Structs are serialized as arrays of their fields
//!         writer.write(&[Marker::FixArray(2).to_u8()])?;
//!         // serialize the fields
//!         self.id.pack(writer)?;
//!         self.name.pack(writer)?;
//!         Ok(())
//!     }
//! }
//!
//! // The lifetime `'buf` is the lifetime of the buffer we're reading from
//! impl<'buf> UnMsgPck<'buf> for User {
//!     fn unpack(source: &mut &'buf [u8]) -> Result<Self, UnpackError>
//!     where
//!         Self: Sized,
//!     {
//!         // Use helper function to read the first byte
//!         let &[b] = slice_take(source).map_err(|_| dbg!(UnpackError::UnexpectedEof))?;
//!
//!         // Check that it's the marker for a 2-element array
//!         let m = Marker::from_u8(b);
//!         if !matches!(m, Marker::FixArray(2)) {
//!             return Err(UnpackError::WrongMarker(m));
//!         };
//!
//!         // Unpack the fields
//!         Ok(User {
//!             id: u32::unpack(source)?,
//!             name: String::unpack(source)?
//!         })
//!     }
//! }
//! #
//! # proptest! {
//! #     #[test]
//! #     fn roundtrip(s: User) {
//! #         let mut writer: Vec<_> = Vec::new();
//! #         s.pack(&mut writer).unwrap();
//! #         let d = User::unpack(&mut &writer[..]).unwrap();
//! #         assert_eq!(s, d);
//! #     }
//! # }
//! ```
//!
//! # Compatibility with `rmp_serde`
//!
//! We aim to be able to deserialize any value serialized using rmp_serde.
//!
//! This means we use the following patterns for serializing:
//!
//! - `struct`s are serialized as arrays of their fields
//! - An `enum` variants with no fields is serialized as a string
//! - An `enum` variant with one field is a map `{ variant_name: field_value }`
//! - An `enum` variant with multiple fields is a map `{ variant_name: [field_values…] }`
// TODO: - `Option<T>` is serialized as an array of 0 or 1 elements
//!
// *TODO: decide if we're gonna change serialized representation of enums*

#![cfg_attr(not(feature = "std"), no_std)]

mod impls;
mod marker;
mod pack;
mod unpack;
mod utils;

pub use crate::{
    impls::{EnumHeader, Variant},
    marker::Marker,
    pack::{
        errors::PackError,
        helpers::{pack_slice, pack_vec},
        size_hint::SizeHint,
        writers::{BufferWriter, MsgWriter, WriteError},
        MsgPck,
    },
    unpack::{errors::UnpackError, helpers::unpack_bytes, UnMsgPck},
    utils::{pack_array_header, slice_take, unpack_array_header, unpack_enum_header},
};

#[cfg(feature = "derive")]
pub use msgpck_derive::{MsgPck, UnMsgPck};
