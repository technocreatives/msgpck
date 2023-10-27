use core::iter;

use rmp::Marker;

use crate::{Piece, UnpackErr};

pub fn slice_take<'a, T, const N: usize>(s: &mut &'a [T]) -> Result<&'a [T; N], UnpackErr> {
    if s.len() < N {
        return Err(UnpackErr::UnexpectedEof);
    }

    let head = s[..N].try_into().expect("slice is big enough");
    *s = &s[N..];

    Ok(head)
}

/// Helper function that packs a msgpack array header.
///
/// **NOTE**: Values of the array are not included, and must therefore be packed next.
pub fn pack_array_header<'a>(len: usize) -> impl Iterator<Item = Piece<'a>> {
    iter::from_generator(move || {
        match len {
            ..=0xf => yield Marker::FixArray(len as u8).into(),
            ..=0xffff => {
                yield Marker::Array16.into();
                yield (len as u16).into();
            }
            _ => {
                yield Marker::Array32.into();
                yield (len as u32).into();
            }
        };
    })
}

/// Helper function that tries to decode a msgpack array header from a byte slice.
///
/// **NOTE**: This doesn't decode the elements of the array, they need to be decoded next.
///
/// ## Returns
/// The length of the array.
pub fn unpack_array_header(bytes: &mut &[u8]) -> Result<usize, UnpackErr> {
    let &[b] = slice_take(bytes)?;

    Ok(match Marker::from_u8(b) {
        Marker::FixArray(len) => len.into(),
        Marker::Array16 => u16::from_be_bytes(*slice_take(bytes)?).into(),
        Marker::Array32 => u32::from_be_bytes(*slice_take(bytes)?).try_into()?,
        m => return Err(UnpackErr::WrongMarker(m)),
    })
}

/// Helper function that packs a msgpack map header.
///
/// **NOTE**: Keys and values of the map are not included, and must therefore be packed next.
pub fn pack_map_header<'a>(len: usize) -> impl Iterator<Item = Piece<'a>> {
    iter::from_generator(move || match len {
        ..=0xf => yield Marker::FixMap(len as u8).into(),
        ..=0xffff => {
            yield Marker::Map16.into();
            yield (len as u16).into();
        }
        _ => {
            yield Marker::Map32.into();
            yield (len as u32).into();
        }
    })
}

/// Helper function that tries to decode a msgpack map header from a byte slice.
///
/// ## Returns
/// The length of the map.
pub fn unpack_map_header(bytes: &mut &[u8]) -> Result<usize, UnpackErr> {
    let &[b] = slice_take(bytes)?;

    Ok(match Marker::from_u8(b) {
        Marker::FixMap(len) => len.into(),
        Marker::Map16 => u16::from_be_bytes(*slice_take(bytes)?).into(),
        Marker::Map32 => u32::from_be_bytes(*slice_take(bytes)?).try_into()?,
        m => return Err(UnpackErr::WrongMarker(m)),
    })
}
