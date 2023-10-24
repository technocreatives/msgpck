use core::iter;

use crate::{
    impl_integers::pack_u64, util::unpack_map_header, Marker, MsgPack, MsgUnpack, Piece, UnpackErr,
};

/// The header/key of a msgpack-encoded enum value.
pub struct EnumHeader<'a> {
    pub variant: Variant<'a>,

    pub unit: bool,
}

/// The discriminant or name of an enum variant.
pub enum Variant<'a> {
    Discriminant(u64),
    Name(&'a str),
}

impl<'a> From<&'a str> for Variant<'a> {
    fn from(name: &'a str) -> Self {
        Variant::Name(name)
    }
}

impl<'a> From<u64> for Variant<'a> {
    fn from(discriminant: u64) -> Self {
        Variant::Discriminant(discriminant)
    }
}

/// Pack an enum header.
///
/// Note that this function does not necessarily pack a complete msgpack value.
/// In the case of an enum with fields, the next value packed must be the fields of the enum.
pub fn pack_enum_header(header: EnumHeader<'_>) -> impl Iterator<Item = Piece<'_>> {
    iter::from_generator(move || {
        if !header.unit {
            yield Marker::FixMap(1).into();
        }

        match header.variant {
            Variant::Discriminant(n) => {
                for p in pack_u64(n) {
                    yield p;
                }
            }
            Variant::Name(s) => {
                for p in s.pack() {
                    yield p;
                }
            }
        }
    })
}

/// Unpack an enum header.
///
/// Note that this function does not necessarily unpack a complete msgpack value.
/// In the case of an enum with fields, the next value unpacked must be the fields of the enum.
pub fn unpack_enum_header<'a>(bytes: &mut &'a [u8]) -> Result<EnumHeader<'a>, UnpackErr> {
    match Marker::from_u8(bytes[0]) {
        // if the enum is just a string or an int, it doesn't have any fields.
        // decode the discriminant/name and return early.
        Marker::FixStr(..) | Marker::Str8 | Marker::Str16 | Marker::Str32 => {
            return Ok(EnumHeader {
                variant: Variant::Name(MsgUnpack::unpack(bytes)?),
                unit: true,
            });
        }
        Marker::FixPos(..) | Marker::U8 | Marker::U16 | Marker::U32 | Marker::U64 => {
            return Ok(EnumHeader {
                variant: Variant::Discriminant(MsgUnpack::unpack(bytes)?),
                unit: true,
            });
        }

        // if the enum is a map, it has at least 1 field.
        Marker::FixMap(_) | Marker::Map16 | Marker::Map32 => {
            let len = unpack_map_header(bytes)?;
            if len != 1 {
                todo!("error on invalid enum map")
            }
        }
        m => return Err(UnpackErr::WrongMarker(m)),
    }

    // read the discriminant/name from the map key
    let variant = match Marker::from_u8(bytes[0]) {
        Marker::FixPos(..) | Marker::U8 | Marker::U16 | Marker::U32 | Marker::U64 => {
            Variant::Discriminant(MsgUnpack::unpack(bytes)?)
        }
        Marker::FixStr(..) | Marker::Str8 | Marker::Str16 | Marker::Str32 => {
            Variant::Name(MsgUnpack::unpack(bytes)?)
        }
        m => return Err(UnpackErr::WrongMarker(m)),
    };

    Ok(EnumHeader {
        variant,
        unit: false,
    })
}
