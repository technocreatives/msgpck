const FIXSTR_SIZE: u8 = 0x1f;
const FIXARRAY_SIZE: u8 = 0x0f;
const FIXMAP_SIZE: u8 = 0x0f;

/// Format markers.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum Marker {
    FixPos(u8) = 0x00,
    FixNeg(i8) = 0xe0,
    Null = 0xc0,
    True = 0xc3,
    False = 0xc2,
    U8 = 0xcc,
    U16 = 0xcd,
    U32 = 0xce,
    U64 = 0xcf,
    I8 = 0xd0,
    I16 = 0xd1,
    I32 = 0xd2,
    I64 = 0xd3,
    F32 = 0xca,
    F64 = 0xcb,
    FixStr(u8) = 0xa0,
    Str8 = 0xd9,
    Str16 = 0xda,
    Str32 = 0xdb,
    Bin8 = 0xc4,
    Bin16 = 0xc5,
    Bin32 = 0xc6,
    FixArray(u8) = 0x90,
    Array16 = 0xdc,
    Array32 = 0xdd,
    FixMap(u8) = 0x80,
    Map16 = 0xde,
    Map32 = 0xdf,
    FixExt1 = 0xd4,
    FixExt2 = 0xd5,
    FixExt4 = 0xd6,
    FixExt8 = 0xd7,
    FixExt16 = 0xd8,
    Ext8 = 0xc7,
    Ext16 = 0xc8,
    Ext32 = 0xc9,
    Reserved = 0xc1,
}

impl Marker {
    /// Construct a msgpack marker from a single byte.
    pub const fn from_u8(n: u8) -> Marker {
        match n {
            0x00..=0x7f => Marker::FixPos(n),
            0xe0..=0xff => Marker::FixNeg(n as i8),
            0x80..=0x8f => Marker::FixMap(n & FIXMAP_SIZE),
            0x90..=0x9f => Marker::FixArray(n & FIXARRAY_SIZE),
            0xa0..=0xbf => Marker::FixStr(n & FIXSTR_SIZE),
            0xc0 => Marker::Null,
            // Marked in MessagePack spec as never used.
            0xc1 => Marker::Reserved,
            0xc2 => Marker::False,
            0xc3 => Marker::True,
            0xc4 => Marker::Bin8,
            0xc5 => Marker::Bin16,
            0xc6 => Marker::Bin32,
            0xc7 => Marker::Ext8,
            0xc8 => Marker::Ext16,
            0xc9 => Marker::Ext32,
            0xca => Marker::F32,
            0xcb => Marker::F64,
            0xcc => Marker::U8,
            0xcd => Marker::U16,
            0xce => Marker::U32,
            0xcf => Marker::U64,
            0xd0 => Marker::I8,
            0xd1 => Marker::I16,
            0xd2 => Marker::I32,
            0xd3 => Marker::I64,
            0xd4 => Marker::FixExt1,
            0xd5 => Marker::FixExt2,
            0xd6 => Marker::FixExt4,
            0xd7 => Marker::FixExt8,
            0xd8 => Marker::FixExt16,
            0xd9 => Marker::Str8,
            0xda => Marker::Str16,
            0xdb => Marker::Str32,
            0xdc => Marker::Array16,
            0xdd => Marker::Array32,
            0xde => Marker::Map16,
            0xdf => Marker::Map32,
        }
    }

    /// Converts a marker object into a single-byte representation.
    pub const fn to_u8(self) -> u8 {
        match self {
            Marker::FixPos(val) => val,
            Marker::FixNeg(val) => val as u8,

            Marker::Null => 0xc0,

            Marker::True => 0xc3,
            Marker::False => 0xc2,

            Marker::U8 => 0xcc,
            Marker::U16 => 0xcd,
            Marker::U32 => 0xce,
            Marker::U64 => 0xcf,

            Marker::I8 => 0xd0,
            Marker::I16 => 0xd1,
            Marker::I32 => 0xd2,
            Marker::I64 => 0xd3,

            Marker::F32 => 0xca,
            Marker::F64 => 0xcb,

            Marker::FixStr(len) => 0xa0 | (len & FIXSTR_SIZE),
            Marker::Str8 => 0xd9,
            Marker::Str16 => 0xda,
            Marker::Str32 => 0xdb,

            Marker::Bin8 => 0xc4,
            Marker::Bin16 => 0xc5,
            Marker::Bin32 => 0xc6,

            Marker::FixArray(len) => 0x90 | (len & FIXARRAY_SIZE),
            Marker::Array16 => 0xdc,
            Marker::Array32 => 0xdd,

            Marker::FixMap(len) => 0x80 | (len & FIXMAP_SIZE),
            Marker::Map16 => 0xde,
            Marker::Map32 => 0xdf,

            Marker::FixExt1 => 0xd4,
            Marker::FixExt2 => 0xd5,
            Marker::FixExt4 => 0xd6,
            Marker::FixExt8 => 0xd7,
            Marker::FixExt16 => 0xd8,
            Marker::Ext8 => 0xc7,
            Marker::Ext16 => 0xc8,
            Marker::Ext32 => 0xc9,

            Marker::Reserved => 0xc1,
        }
    }
}

impl From<u8> for Marker {
    #[inline]
    fn from(val: u8) -> Marker {
        Marker::from_u8(val)
    }
}

impl From<Marker> for u8 {
    #[inline]
    fn from(val: Marker) -> Self {
        val.to_u8()
    }
}
