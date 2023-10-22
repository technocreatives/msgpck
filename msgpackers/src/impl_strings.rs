use crate::{util::slice_take, MsgPack, MsgUnpack, Piece, UnpackErr};
use core::iter;
use rmp::Marker;
use std::str::from_utf8;

impl MsgPack for str {
    type Iter<'a> = impl Iterator<Item = Piece<'a>>
    where
        Self: 'a;

    fn pack(&self) -> Self::Iter<'_> {
        iter::from_generator(move || {
            match self.len() {
                ..=0xf => yield Marker::FixStr(self.len() as u8).into(),
                ..=0xff => {
                    yield Marker::Str8.into();
                    yield (self.len() as u8).into();
                }
                ..=0xffff => {
                    yield Marker::Str16.into();
                    yield (self.len() as u16).into();
                }
                _ => {
                    yield Marker::Str32.into();
                    yield (self.len() as u32).into();
                }
            }

            yield Piece::Bytes(self.as_bytes());
        })
    }
}

impl<'buf> MsgUnpack<'buf> for &'buf str {
    fn unpack(bytes: &mut &'buf [u8]) -> Result<Self, UnpackErr>
    where
        Self: Sized,
    {
        let &[b] = slice_take(bytes)?;
        let len: usize = match Marker::from_u8(b) {
            Marker::FixStr(len) => len.into(),
            Marker::Str8 => slice_take::<_, 1>(bytes)?[0].into(),
            Marker::Str16 => u16::from_le_bytes(*slice_take(bytes)?).into(),
            Marker::Str32 => u32::from_le_bytes(*slice_take(bytes)?).try_into()?,
            m => return Err(UnpackErr::WrongMarker(m)),
        };

        let str_bytes: &'buf [u8] = bytes.take(..len).ok_or(UnpackErr::UnexpectedEof)?;
        Ok(from_utf8(str_bytes)?)
    }
}
