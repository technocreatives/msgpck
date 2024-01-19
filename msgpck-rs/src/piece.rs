use crate::marker::Marker;
use core::slice;

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

/// One or two pieces. Usually a [Marker] and maybe some data.
pub struct Pair<'a>(pub Piece<'a>, pub Option<Piece<'a>>);

impl Piece<'_> {
    pub const fn as_bytes(&self) -> &[u8] {
        match self {
            Piece::Bytes8(b) => b,
            Piece::Bytes4(b) => b,
            Piece::Bytes2(b) => b,
            Piece::Bytes(b) => b,
            Piece::Byte(b) => slice::from_ref(b),
        }
    }

    #[inline(always)]
    pub const fn from_marker(m: Marker) -> Self {
        Self::Byte(m.to_u8())
    }

    #[inline(always)]
    pub const fn from_u8(v: u8) -> Self {
        Self::Byte(v)
    }

    #[inline(always)]
    pub const fn from_u16(v: u16) -> Self {
        Self::Bytes2(v.to_be_bytes())
    }

    #[inline(always)]
    pub const fn from_u32(v: u32) -> Self {
        Self::Bytes4(v.to_be_bytes())
    }

    #[inline(always)]
    pub const fn from_u64(v: u64) -> Self {
        Self::Bytes8(v.to_be_bytes())
    }

    #[inline(always)]
    pub const fn from_i8(v: i8) -> Self {
        Self::Byte(v as u8)
    }

    #[inline(always)]
    pub const fn from_i16(v: i16) -> Self {
        Self::Bytes2(v.to_be_bytes())
    }

    #[inline(always)]
    pub const fn from_i32(v: i32) -> Self {
        Self::Bytes4(v.to_be_bytes())
    }

    #[inline(always)]
    pub const fn from_i64(v: i64) -> Self {
        Self::Bytes8(v.to_be_bytes())
    }
}

impl From<Marker> for Piece<'static> {
    #[inline(always)]
    fn from(m: Marker) -> Self {
        Piece::from_marker(m)
    }
}

impl From<u64> for Piece<'static> {
    #[inline(always)]
    fn from(v: u64) -> Self {
        Self::Bytes8(v.to_be_bytes())
    }
}

impl From<u32> for Piece<'static> {
    #[inline(always)]
    fn from(v: u32) -> Self {
        Self::Bytes4(v.to_be_bytes())
    }
}

impl From<u16> for Piece<'static> {
    #[inline(always)]
    fn from(v: u16) -> Self {
        Self::Bytes2(v.to_be_bytes())
    }
}

impl From<i64> for Piece<'static> {
    #[inline(always)]
    fn from(v: i64) -> Self {
        Self::Bytes8(v.to_be_bytes())
    }
}

impl From<i32> for Piece<'static> {
    #[inline(always)]
    fn from(v: i32) -> Self {
        Self::Bytes4(v.to_be_bytes())
    }
}

impl From<i16> for Piece<'static> {
    #[inline(always)]
    fn from(v: i16) -> Self {
        Self::Bytes2(v.to_be_bytes())
    }
}

impl<'a> From<&'a [u8]> for Piece<'a> {
    #[inline(always)]
    fn from(bytes: &'a [u8]) -> Self {
        Self::Bytes(bytes)
    }
}

impl From<u8> for Piece<'static> {
    #[inline(always)]
    fn from(v: u8) -> Self {
        Self::Byte(v)
    }
}

impl<'a> Pair<'a> {
    pub fn pieces(self) -> impl Iterator<Item = Piece<'a>> {
        [Some(self.0), self.1].into_iter().flatten()
    }
}
