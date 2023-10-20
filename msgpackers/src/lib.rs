#![feature(impl_trait_in_assoc_type)]
#![feature(generators)]
#![feature(iter_from_generator)]
#![allow(clippy::match_overlapping_arm)]

mod error;
mod impl_collections;
mod impl_floats;
mod impl_integers;
mod piece;
mod util;

pub use error::UnpackErr;
pub use msgpackers_derive::MsgPack;
pub use piece::Piece;
pub use rmp::Marker;
pub use util::unpack_array_header;

/// Trait for serializing a type using msgpack.
pub trait MsgPack {
    type Iter<'a>: Iterator<Item = Piece<'a>>
    where
        Self: 'a;

    /// Returns an iterator of msgpack [Piece]s. Collect them all to produce a valid msgpack value.
    ///
    /// ```
    /// use msgpackers::MsgPack;
    /// let mut encoded = vec![];
    /// for m in vec![0xDDu8, 0xEE, 3].pack() {
    ///     encoded.extend_from_slice(m.as_bytes());
    /// }
    /// assert_eq!(encoded, [0x93, 0xCC, 0xDD, 0xCC, 0xEE, 3]);
    /// ```
    fn pack(&self) -> Self::Iter<'_>;
}

/// Trait for deserializing a type using msgpack.
pub trait MsgUnpack {
    /// Unpack a value from a msgpack bytes slice
    ///
    /// ```
    /// use msgpackers::MsgUnpack;
    /// let encoded = [0x93, 0xCC, 0xDD, 0xCC, 0xEE, 3];
    /// let decoded: Vec<u8> = Vec::unpack(&mut &encoded[..]).unwrap();
    /// assert_eq!(decoded, &[0xDDu8, 0xEE, 3]);
    /// ```
    fn unpack<'buf>(bytes: &mut &'buf [u8]) -> Result<Self, UnpackErr>
    where
        Self: Sized + 'buf;
}
