use crate::{util::unpack_array_header, MsgPack, MsgUnpack, Piece, UnpackErr};
use core::iter;
use rmp::Marker;

impl<T: MsgPack> MsgPack for Vec<T> {
    type Iter<'a> = impl Iterator<Item = Piece<'a>>
    where
        Self: 'a;

    fn pack(&self) -> Self::Iter<'_> {
        iter::from_generator(move || {
            match self.len() {
                ..=0xf => yield Marker::FixArray(self.len() as u8).into(),
                ..=0xffff => {
                    yield Marker::Array16.into();
                    yield (self.len() as u16).into();
                }
                _ => {
                    yield Marker::Array32.into();
                    yield (self.len() as u32).into();
                }
            };

            for t in self {
                for m in t.pack() {
                    yield m;
                }
            }
        })
    }
}

impl<'buf, T: MsgUnpack<'buf> + 'buf> MsgUnpack<'buf> for Vec<T> {
    fn unpack(bytes: &mut &'buf [u8]) -> Result<Self, crate::UnpackErr>
    where
        Self: Sized,
    {
        let len: usize = unpack_array_header(bytes)?;

        // sanity check
        // make sure that it's plausible the array could contain this many elements
        if bytes.len() < len {
            return Err(UnpackErr::UnexpectedEof);
        }

        let mut vec = Vec::with_capacity(len);

        for _ in 0..len {
            vec.push(T::unpack(bytes)?);
        }

        Ok(vec)
    }
}
