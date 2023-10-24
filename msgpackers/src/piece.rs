use core::slice;
use rmp::Marker;

/// A piece of msgpack data. Used by the [MsgPack](crate::MsgPack) trait.
#[derive(Debug)]
pub enum Piece<'a> {
    /// Some 8-byte msgpack value
    Bytes8([u8; 8]),

    /// Some 4-byte msgpack value
    Bytes4([u8; 4]),

    /// Some 2-byte msgpack value
    Bytes2([u8; 2]),

    /// Some msgpack bytes
    Bytes(&'a [u8]),

    /// Some msgpack byte
    Byte(u8),
}

impl Piece<'_> {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Piece::Bytes8(b) => b,
            Piece::Bytes4(b) => b,
            Piece::Bytes2(b) => b,
            Piece::Bytes(b) => b,
            Piece::Byte(b) => slice::from_ref(b),
        }
    }
}

impl From<Marker> for Piece<'static> {
    fn from(m: Marker) -> Self {
        Self::Byte(m.to_u8())
    }
}

impl From<u64> for Piece<'static> {
    fn from(v: u64) -> Self {
        Self::Bytes8(v.to_be_bytes())
    }
}

impl From<u32> for Piece<'static> {
    fn from(v: u32) -> Self {
        Self::Bytes4(v.to_be_bytes())
    }
}

impl From<u16> for Piece<'static> {
    fn from(v: u16) -> Self {
        Self::Bytes2(v.to_be_bytes())
    }
}

impl<'a> From<&'a [u8]> for Piece<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        Self::Bytes(bytes)
    }
}

impl From<u8> for Piece<'static> {
    fn from(v: u8) -> Self {
        Self::Byte(v)
    }
}
