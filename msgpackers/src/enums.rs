use crate::{util::unpack_map_header, Marker, MsgUnpack, Piece, UnpackErr};

/// The header/key of a msgpack-encoded enum value.
pub struct EnumHeader<'a> {
    pub variant: Variant<'a>,

    pub unit: bool,
}

/// The discriminator or name of an enum variant.
pub enum Variant<'a> {
    Discriminator(u64),
    Name(&'a str),
}

pub fn pack_enum_header<'a>(header: EnumHeader<'a>) -> impl Iterator<Item = Piece<'a>> {
    // TODO
    [].into_iter()
}

pub fn unpack_enum_header<'a>(bytes: &mut &'a [u8]) -> Result<EnumHeader<'a>, UnpackErr> {
    match Marker::from_u8(bytes[0]) {
        // if the enum is just a string or an int, it doesn't have any fields.
        // decode the discriminator/name and return early.
        Marker::FixStr(..) | Marker::Str8 | Marker::Str16 | Marker::Str32 => {
            return Ok(EnumHeader {
                variant: Variant::Name(MsgUnpack::unpack(bytes)?),
                unit: true,
            });
        }
        Marker::FixPos(..) | Marker::U8 | Marker::U16 | Marker::U32 | Marker::U64 => {
            return Ok(EnumHeader {
                variant: Variant::Discriminator(MsgUnpack::unpack(bytes)?),
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

    // read the discriminator/name from the map key
    let variant = match Marker::from_u8(bytes[0]) {
        Marker::FixPos(..) | Marker::U8 | Marker::U16 | Marker::U32 | Marker::U64 => {
            Variant::Discriminator(MsgUnpack::unpack(bytes)?)
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
