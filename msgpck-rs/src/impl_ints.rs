use crate::{
    impl_uints::pack_u64, marker::Marker, piece::Pair, util::slice_take, MsgPack, MsgUnpack, Piece,
    UnpackErr,
};

impl MsgPack for i8 {
    fn pack(&self) -> impl Iterator<Item = Piece<'_>> {
        pack_i64(i64::from(*self)).pieces()
    }
}

impl MsgPack for i16 {
    fn pack(&self) -> impl Iterator<Item = Piece<'_>> {
        pack_i64(i64::from(*self)).pieces()
    }
}

impl MsgPack for i32 {
    fn pack(&self) -> impl Iterator<Item = Piece<'_>> {
        pack_i64(i64::from(*self)).pieces()
    }
}

impl MsgPack for i64 {
    fn pack(&self) -> impl Iterator<Item = Piece<'_>> {
        pack_i64(*self).pieces()
    }
}

impl MsgPack for isize {
    fn pack(&self) -> impl Iterator<Item = Piece<'_>> {
        pack_i64(*self as i64).pieces()
    }
}

impl<'buf> MsgUnpack<'buf> for i8 {
    fn unpack(bytes: &mut &'buf [u8]) -> Result<Self, UnpackErr>
    where
        Self: Sized,
    {
        let n = unpack_i64(bytes)?;
        Ok(n.try_into()?)
    }
}

impl<'buf> MsgUnpack<'buf> for i16 {
    fn unpack(bytes: &mut &'buf [u8]) -> Result<Self, UnpackErr>
    where
        Self: Sized,
    {
        let n = unpack_i64(bytes)?;
        Ok(n.try_into()?)
    }
}

impl<'buf> MsgUnpack<'buf> for i32 {
    fn unpack(bytes: &mut &'buf [u8]) -> Result<Self, UnpackErr>
    where
        Self: Sized,
    {
        let n = unpack_i64(bytes)?;
        Ok(n.try_into()?)
    }
}

impl<'buf> MsgUnpack<'buf> for i64 {
    fn unpack(bytes: &mut &'buf [u8]) -> Result<Self, UnpackErr>
    where
        Self: Sized,
    {
        unpack_i64(bytes)
    }
}

impl<'buf> MsgUnpack<'buf> for isize {
    fn unpack(bytes: &mut &'buf [u8]) -> Result<Self, UnpackErr>
    where
        Self: Sized,
    {
        let n = unpack_i64(bytes)?;
        Ok(n.try_into()?)
    }
}

pub fn unpack_i64(bytes: &mut &[u8]) -> Result<i64, UnpackErr> {
    let &[b] = slice_take(bytes)?;
    Ok(match Marker::from_u8(b) {
        Marker::FixNeg(i) => i.into(),
        Marker::FixPos(n) => n.into(),
        Marker::I8 => {
            let &[i] = slice_take(bytes)?;
            (i as i8).into()
        }
        Marker::U8 => {
            let &[n] = slice_take(bytes)?;
            n.into()
        }
        Marker::I16 => i16::from_be_bytes(*slice_take(bytes)?).into(),
        Marker::U16 => u16::from_be_bytes(*slice_take(bytes)?).into(),
        Marker::I32 => i32::from_be_bytes(*slice_take(bytes)?).into(),
        Marker::U32 => u32::from_be_bytes(*slice_take(bytes)?).into(),
        Marker::I64 => i64::from_be_bytes(*slice_take(bytes)?),
        Marker::U64 => {
            let n = u64::from_be_bytes(*slice_take(bytes)?);
            n.try_into().map_err(UnpackErr::IntTooBig)?
        }
        m => return Err(UnpackErr::WrongMarker(m)),
    })
}

pub const fn pack_i64<'a>(i: i64) -> Pair<'a> {
    // Pack i into the smallest msgpack type that will fit it.
    match i {
        ..=-2147483649 => Pair(Piece::from_marker(Marker::I64), Some(Piece::from_i64(i))),
        ..=-32769 => Pair(
            Piece::from_marker(Marker::I32),
            Some(Piece::from_i32(i as i32)),
        ),
        ..=-129 => Pair(
            Piece::from_marker(Marker::I16),
            Some(Piece::from_i16(i as i16)),
        ),
        ..=-33 => Pair(
            Piece::from_marker(Marker::I8),
            Some(Piece::from_i8(i as i8)),
        ),
        ..=-1 => Pair(Piece::from_marker(Marker::FixNeg(i as i8)), None),
        // If the value is positive, pack as an unsigned integer.
        _ => return pack_u64(i as u64),
    }
}
