use crate::{marker::Marker, piece::Pair, util::slice_take, MsgPack, MsgUnpack, Piece, UnpackErr};

impl MsgPack for u8 {
    fn pack(&self) -> impl Iterator<Item = Piece<'_>> {
        pack_u64(u64::from(*self)).pieces()
    }
}

impl MsgPack for u16 {
    fn pack(&self) -> impl Iterator<Item = Piece<'_>> {
        pack_u64(u64::from(*self)).pieces()
    }
}

impl MsgPack for u32 {
    fn pack(&self) -> impl Iterator<Item = Piece<'_>> {
        pack_u64(u64::from(*self)).pieces()
    }
}

impl MsgPack for u64 {
    fn pack(&self) -> impl Iterator<Item = Piece<'_>> {
        pack_u64(*self).pieces()
    }
}

impl<'buf> MsgUnpack<'buf> for u8 {
    fn unpack(bytes: &mut &'buf [u8]) -> Result<Self, UnpackErr>
    where
        Self: Sized,
    {
        let n = unpack_u64(bytes)?;
        Ok(n.try_into()?)
    }
}

impl<'buf> MsgUnpack<'buf> for u16 {
    fn unpack(bytes: &mut &'buf [u8]) -> Result<Self, UnpackErr>
    where
        Self: Sized,
    {
        let n = unpack_u64(bytes)?;
        Ok(n.try_into()?)
    }
}

impl<'buf> MsgUnpack<'buf> for u32 {
    fn unpack(bytes: &mut &'buf [u8]) -> Result<Self, UnpackErr>
    where
        Self: Sized,
    {
        let n = unpack_u64(bytes)?;
        Ok(n.try_into()?)
    }
}

impl<'buf> MsgUnpack<'buf> for u64 {
    fn unpack(bytes: &mut &'buf [u8]) -> Result<Self, UnpackErr>
    where
        Self: Sized,
    {
        unpack_u64(bytes)
    }
}

pub fn unpack_u64(bytes: &mut &[u8]) -> Result<u64, UnpackErr> {
    let &[b] = slice_take(bytes)?;

    Ok(match Marker::from_u8(b) {
        Marker::FixPos(n) => n.into(),
        Marker::U8 => slice_take::<u8, 1>(bytes)?[0].into(),
        Marker::U16 => u16::from_be_bytes(*slice_take(bytes)?).into(),
        Marker::U32 => u32::from_be_bytes(*slice_take(bytes)?).into(),
        Marker::U64 => u64::from_be_bytes(*slice_take(bytes)?),
        m => return Err(UnpackErr::WrongMarker(m)),
    })
}

pub const fn pack_u64<'a>(n: u64) -> Pair<'a> {
    match n {
        ..=0x7f => Pair(Piece::from_marker(Marker::FixPos(n as u8)), None),
        ..=0xff => Pair(
            Piece::from_marker(Marker::U8),
            Some(Piece::from_u8(n as u8)),
        ),
        ..=0xffff => Pair(
            Piece::from_marker(Marker::U16),
            Some(Piece::from_u16(n as u16)),
        ),
        ..=0xffff_ffff => Pair(
            Piece::from_marker(Marker::U32),
            Some(Piece::from_u32(n as u32)),
        ),
        _ => Pair(Piece::from_marker(Marker::U64), Some(Piece::from_u64(n))),
    }
}
