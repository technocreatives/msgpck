#![allow(dead_code)]

use crate::{marker::Marker, piece::Pair, MsgPack, MsgUnpack, Piece, UnpackErr};

pub fn slice_take<'a, T, const N: usize>(s: &mut &'a [T]) -> Result<&'a [T; N], UnpackErr> {
    let Ok(head) = s[..N].try_into() else {
        return Err(UnpackErr::UnexpectedEof);
    };
    *s = &s[N..];

    Ok(head)
}

/// Helper function that packs a msgpack array header.
///
/// **NOTE**: Values of the array are not included, and must therefore be packed next.
pub fn pack_array_header<'a>(len: usize) -> impl Iterator<Item = Piece<'a>> {
    match len {
        ..=0xf => Pair(Marker::FixArray(len as u8).into(), None),
        ..=0xffff => Pair(Marker::Array16.into(), Some((len as u16).into())),
        _ => Pair(Marker::Array32.into(), Some((len as u32).into())),
    }
    .pieces()
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
    match len {
        ..=0xf => Pair(Marker::FixMap(len as u8).into(), None),
        ..=0xffff => Pair(Marker::Map16.into(), Some((len as u16).into())),
        _ => Pair(Marker::Map32.into(), Some((len as u32).into())),
    }
    .pieces()
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

pub fn pack_map<'a, K, V>(
    len: usize,
    kvs: impl Iterator<Item = (&'a K, &'a V)> + 'a,
) -> impl Iterator<Item = Piece<'a>> + 'a
where
    K: MsgPack + 'a,
    V: MsgPack + 'a,
{
    pack_map_header(len).chain(kvs.flat_map(|(k, v)| k.pack().chain(v.pack())))
}

pub fn unpack_map<'a, K, V, C>(bytes: &mut &'a [u8]) -> Result<C, UnpackErr>
where
    K: MsgUnpack<'a>,
    V: MsgUnpack<'a>,
    C: FromIterator<(K, V)>,
{
    let len = unpack_map_header(bytes)?;

    // sanity check
    // make sure that it's plausible the array could contain this many elements
    if bytes.len() < len {
        return Err(UnpackErr::UnexpectedEof);
    }

    (0..len)
        .map(move |_| {
            let k = K::unpack(bytes)?;
            let v = V::unpack(bytes)?;
            Ok((k, v))
        })
        .collect()
}

pub fn pack_array<'a, T>(
    len: usize,
    elements: impl Iterator<Item = &'a T> + 'a,
) -> impl Iterator<Item = Piece<'a>> + 'a
where
    T: MsgPack + 'a,
{
    pack_array_header(len).chain(elements.flat_map(|elem| elem.pack()))
}
pub fn unpack_array<'a, T, C>(bytes: &mut &'a [u8]) -> Result<C, UnpackErr>
where
    T: MsgUnpack<'a>,
    C: FromIterator<T>,
{
    let len = unpack_array_header(bytes)?;
    (0..len).map(move |_| T::unpack(bytes)).collect()
}

pub enum Either<A, B> {
    A(A),
    B(B),
}

impl<A, B, T> Iterator for Either<A, B>
where
    A: Iterator<Item = T>,
    B: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Either::A(a) => a.next(),
            Either::B(b) => b.next(),
        }
    }
}
