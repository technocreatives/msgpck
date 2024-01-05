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

pub fn pack_u64<'a>(n: u64) -> Pair<'a> {
    match n {
        ..=0x7f => Pair(Marker::FixPos(n as u8).into(), None),
        ..=0xff => Pair(Marker::U8.into(), Some((n as u8).into())),
        ..=0xffff => Pair(Marker::U16.into(), Some((n as u16).into())),
        ..=0xffffffff => Pair(Marker::U32.into(), Some((n as u32).into())),
        _ => Pair(Marker::U64.into(), Some(n.into())),
    }
}
