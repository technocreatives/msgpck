use crate::{marker::Marker, util::slice_take, MsgPack, MsgUnpack, Piece, UnpackErr};
use core::iter;

impl MsgPack for [u8] {
    type Iter<'a> = impl Iterator<Item = Piece<'a>>
    where
        Self: 'a;

    fn pack(&self) -> Self::Iter<'_> {
        iter::from_generator(move || {
            match self.len() {
                ..=0xff => {
                    yield Marker::Bin8.into();
                    yield (self.len() as u8).into();
                }
                ..=0xffff => {
                    yield Marker::Bin16.into();
                    yield (self.len() as u16).into();
                }
                _ => {
                    yield Marker::Bin32.into();
                    yield (self.len() as u32).into();
                }
            }

            yield Piece::Bytes(self);
        })
    }
}

impl<'buf> MsgUnpack<'buf> for &'buf [u8] {
    fn unpack(bytes: &mut &'buf [u8]) -> Result<Self, UnpackErr>
    where
        Self: Sized,
    {
        let &[b] = slice_take(bytes)?;
        let len: usize = match Marker::from_u8(b) {
            Marker::Bin8 => slice_take::<_, 1>(bytes)?[0].into(),
            Marker::Bin16 => u16::from_be_bytes(*slice_take(bytes)?).into(),
            Marker::Bin32 => u32::from_be_bytes(*slice_take(bytes)?).try_into()?,
            m => return Err(UnpackErr::WrongMarker(m)),
        };

        bytes.take(..len).ok_or(UnpackErr::UnexpectedEof)
    }
}
