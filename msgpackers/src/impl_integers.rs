use crate::{util::slice_take, MsgPack, MsgUnpack, Piece, UnpackErr};
use core::iter;
use rmp::Marker;

// TODO: impl for signed integers

impl MsgPack for u8 {
    type Iter<'a> = impl Iterator<Item = Piece<'a>>
    where
        Self: 'a;

    fn pack(&self) -> Self::Iter<'_> {
        pack_u64(u64::from(*self))
    }
}

impl MsgPack for u16 {
    type Iter<'a> = impl Iterator<Item = Piece<'a>>
    where
        Self: 'a;

    fn pack(&self) -> Self::Iter<'_> {
        pack_u64(u64::from(*self))
    }
}

impl MsgPack for u32 {
    type Iter<'a> = impl Iterator<Item = Piece<'a>>
    where
        Self: 'a;

    fn pack(&self) -> Self::Iter<'_> {
        pack_u64(u64::from(*self))
    }
}

impl MsgPack for u64 {
    type Iter<'a> = impl Iterator<Item = Piece<'a>>
    where
        Self: 'a;

    fn pack(&self) -> Self::Iter<'_> {
        pack_u64(*self)
    }
}

impl MsgUnpack for u8 {
    fn unpack<'buf>(bytes: &mut &'buf [u8]) -> Result<Self, UnpackErr>
    where
        Self: Sized + 'buf,
    {
        let n = unpack_u64(bytes)?;
        Ok(n.try_into()?)
    }
}

impl MsgUnpack for u16 {
    fn unpack<'buf>(bytes: &mut &'buf [u8]) -> Result<Self, UnpackErr>
    where
        Self: Sized + 'buf,
    {
        let n = unpack_u64(bytes)?;
        Ok(n.try_into()?)
    }
}

impl MsgUnpack for u32 {
    fn unpack<'buf>(bytes: &mut &'buf [u8]) -> Result<Self, UnpackErr>
    where
        Self: Sized + 'buf,
    {
        let n = unpack_u64(bytes)?;
        Ok(n.try_into()?)
    }
}

impl MsgUnpack for u64 {
    fn unpack<'buf>(bytes: &mut &'buf [u8]) -> Result<Self, UnpackErr>
    where
        Self: Sized + 'buf,
    {
        unpack_u64(bytes)
    }
}

fn unpack_u64(bytes: &mut &[u8]) -> Result<u64, UnpackErr> {
    let &[b] = slice_take(bytes)?;

    Ok(match Marker::from_u8(b) {
        Marker::FixPos(n) => n.into(),
        Marker::U8 => slice_take::<u8, 1>(bytes)?[0].into(),
        Marker::U16 => u16::from_le_bytes(*slice_take(bytes)?).into(),
        Marker::U32 => u32::from_le_bytes(*slice_take(bytes)?).into(),
        Marker::U64 => u64::from_le_bytes(*slice_take(bytes)?),
        m => return Err(UnpackErr::WrongMarker(m)),
    })
}

fn pack_u64<'a>(n: u64) -> impl Iterator<Item = Piece<'a>> {
    iter::from_generator(move || match n {
        ..=0x7f => yield (n as u8).into(),
        ..=0xff => {
            yield Marker::U8.into();
            yield (n as u8).into();
        }
        ..=0xffff => {
            yield Marker::U16.into();
            yield (n as u16).into();
        }
        ..=0xffffffff => {
            yield Marker::U32.into();
            yield (n as u32).into();
        }
        _ => {
            yield Marker::U64.into();
            yield n.into();
        }
    })
}
