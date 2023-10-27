use crate::{marker::Marker, PackError, UnpackError};

pub fn slice_take<'a, T, const N: usize>(
    s: &mut &'a [T],
) -> Result<&'a [T; N], UnexpectedEofError> {
    if s.len() < N {
        return Err(UnexpectedEofError);
    }

    let head = s[..N].try_into().expect("slice is big enough");
    *s = &s[N..];

    Ok(head)
}

pub struct UnexpectedEofError;

/// Helper function that tries to decode a msgpack array header from a byte slice.
///
/// **NOTE**: This doesn't decode the elements of the array, they need to be decoded next.
///
/// ## Returns
/// The length of the array.
pub fn unpack_array_header(source: &mut &[u8]) -> Result<usize, UnpackError> {
    let &[b] = slice_take(source)?;

    Ok(match Marker::from_u8(b) {
        Marker::FixArray(len) => len.into(),
        Marker::Array16 => u16::from_be_bytes(*slice_take(source)?).into(),
        Marker::Array32 => u32::from_be_bytes(*slice_take(source)?).try_into()?,
        m => return Err(UnpackError::WrongMarker(m)),
    })
}

/// Helper function that packs a msgpack map header.
///
/// **NOTE**: Keys and values of the map are not included, and must therefore be packed next.
pub fn pack_map_header<'a>(writer: &mut dyn crate::MsgWriter, len: usize) -> Result<(), PackError> {
    match len {
        ..=0xf => writer.write(&[Marker::FixMap(len as u8).to_u8()])?,
        0x10..=0xffff => {
            writer.write(&[Marker::Map16.to_u8()])?;
            writer.write(&((len as u16).to_be_bytes()))?;
        }
        _ => {
            writer.write(&[Marker::Map32.to_u8()])?;
            writer.write(&((len as u32).to_be_bytes()))?;
        }
    }
    Ok(())
}

/// Helper function that tries to decode a msgpack map header from a byte slice.
///
/// ## Returns
/// The length of the map.
pub fn unpack_map_header(bytes: &mut &[u8]) -> Result<usize, UnpackError> {
    let &[b] = slice_take(bytes)?;

    Ok(match Marker::from_u8(b) {
        Marker::FixMap(len) => len.into(),
        Marker::Map16 => u16::from_be_bytes(*slice_take(bytes)?).into(),
        Marker::Map32 => u32::from_be_bytes(*slice_take(bytes)?).try_into()?,
        m => return Err(UnpackError::WrongMarker(m)),
    })
}
