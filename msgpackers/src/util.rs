use rmp::Marker;

use crate::UnpackErr;

pub fn slice_take<'a, T, const N: usize>(s: &mut &'a [T]) -> Result<&'a [T; N], UnpackErr> {
    if s.len() < N {
        return Err(UnpackErr::UnexpectedEof);
    }

    let head = s[..N].try_into().expect("slice is big enough");
    *s = &s[N..];

    Ok(head)
}

/// Helper function that tries to decode a msgpack array header from a byte slice.
///
/// Returns the length of the array.
pub fn unpack_array_header(bytes: &mut &[u8]) -> Result<usize, UnpackErr> {
    let &[b] = slice_take(bytes)?;

    Ok(match Marker::from_u8(b) {
        Marker::FixArray(len) => len.into(),
        Marker::Array16 => u16::from_le_bytes(*slice_take(bytes)?).into(),
        Marker::Array32 => u32::from_le_bytes(*slice_take(bytes)?).try_into()?,
        m => return Err(UnpackErr::WrongMarker(m)),
    })
}
/// Helper function that tries to decode a msgpack map header from a byte slice.
///
/// Returns the length of the map.
pub fn unpack_map_header(bytes: &mut &[u8]) -> Result<usize, UnpackErr> {
    let &[b] = slice_take(bytes)?;

    Ok(match Marker::from_u8(b) {
        Marker::FixMap(len) => len.into(),
        Marker::Map16 => u16::from_le_bytes(*slice_take(bytes)?).into(),
        Marker::Map32 => u32::from_le_bytes(*slice_take(bytes)?).try_into()?,
        m => return Err(UnpackErr::WrongMarker(m)),
    })
}
