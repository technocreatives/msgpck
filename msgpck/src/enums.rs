use crate::{
    helpers::pack_i64,
    marker::Marker,
    util::{unpack_map_header, Either},
    MsgPack, MsgUnpack, PackErr, Piece, UnpackErr, Write,
};

/// The header/key of a msgpack-encoded enum value.
pub struct EnumHeader<'a> {
    pub variant: Variant<'a>,

    pub unit: bool,
}

/// The discriminant or name of an enum variant.
pub enum Variant<'a> {
    Discriminant(isize),
    Name(&'a str),
}

impl<'a> From<&'a str> for Variant<'a> {
    #[inline(always)]
    fn from(name: &'a str) -> Self {
        Variant::Name(name)
    }
}

impl<'a> From<isize> for Variant<'a> {
    #[inline(always)]
    fn from(discriminant: isize) -> Self {
        Variant::Discriminant(discriminant)
    }
}

/// Pack an enum header.
///
/// **NOTE**: This function does not necessarily pack a complete msgpack value.
/// In the case of an enum with fields, the next value packed must be the fields of the enum.
///
/// # Panic
/// This function panics if the enum discriminant (which is represented as an isize) is too big to
/// fit in an i64. On most platforms, this is not possible.
pub fn pack_enum_header(header: EnumHeader<'_>) -> impl Iterator<Item = Piece<'_>> {
    (!header.unit)
        .then_some(Marker::FixMap(1).into())
        .into_iter()
        .chain(match header.variant {
            Variant::Discriminant(n) => {
                if isize::BITS > i64::BITS && n > i64::MAX as isize {
                    panic!("enum discriminant is bigger than i64::MAX");
                }

                Either::A(pack_i64(n as i64).pieces())
            }
            Variant::Name(s) => Either::B(s.pack()),
        })
}

/// Pack an enum header into a [Write].
///
/// **NOTE**: This function does not necessarily pack a complete msgpack value.
/// In the case of an enum with fields, the next value packed must be the fields of the enum.
///
/// # Panic
/// This function panics if the enum discriminant (which is represented as an isize) is too big to
/// fit in an i64. On most platforms, this is not possible.
pub fn pack_enum_header_to_writer(
    header: EnumHeader<'_>,
    w: &mut dyn Write,
) -> Result<usize, PackErr> {
    let mut n = 0usize;
    if !header.unit {
        let piece = Piece::from_marker(Marker::FixMap(1));
        w.write_all(piece.as_bytes())?;
        n += piece.as_bytes().len();
    }

    match header.variant {
        Variant::Discriminant(d) => {
            if isize::BITS > i64::BITS && d > i64::MAX as isize {
                panic!("enum discriminant is bigger than i64::MAX");
            }

            for piece in pack_i64(d as i64).pieces() {
                w.write_all(piece.as_bytes())?;
                n += piece.as_bytes().len();
            }
        }
        Variant::Name(s) => {
            for piece in s.pack() {
                w.write_all(piece.as_bytes())?;
                n += piece.as_bytes().len();
            }
        }
    }

    Ok(n)
}

/// Unpack an enum header.
///
/// **NOTE**: This function does not necessarily unpack a complete msgpack value.
/// In the case of an enum with fields, the next value unpacked must be the fields of the enum.
pub fn unpack_enum_header<'a>(bytes: &mut &'a [u8]) -> Result<EnumHeader<'a>, UnpackErr> {
    match bytes
        .first()
        .map(|&b| Marker::from_u8(b))
        .ok_or(UnpackErr::UnexpectedEof)?
    {
        // if the enum is just a string or an int, it doesn't have any fields.
        // decode the discriminant/name and return early.
        Marker::FixStr(..) | Marker::Str8 | Marker::Str16 | Marker::Str32 => {
            return Ok(EnumHeader {
                variant: Variant::Name(MsgUnpack::unpack(bytes)?),
                unit: true,
            });
        }

        Marker::FixNeg(..)
        | Marker::I8
        | Marker::I16
        | Marker::I32
        | Marker::I64
        | Marker::FixPos(..)
        | Marker::U8
        | Marker::U16
        | Marker::U32
        | Marker::U64 => {
            return Ok(EnumHeader {
                variant: Variant::Discriminant(MsgUnpack::unpack(bytes)?),
                unit: true,
            });
        }

        // if the enum is a map, it has at least 1 field.
        Marker::FixMap(_) | Marker::Map16 | Marker::Map32 => {
            let len = unpack_map_header(bytes)?;
            if len != 1 {
                return Err(UnpackErr::InvalidEnumHeader);
            }
        }
        m => return Err(UnpackErr::WrongMarker(m)),
    }

    // read the discriminant/name from the map key
    let variant = match bytes
        .first()
        .map(|&b| Marker::from_u8(b))
        .ok_or(UnpackErr::UnexpectedEof)?
    {
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
