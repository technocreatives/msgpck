use crate::{
    util::{pack_array_header, unpack_array_header},
    MsgPack, MsgUnpack, Piece, UnpackErr,
};
use alloc::vec::Vec;

impl<T: MsgPack> MsgPack for Vec<T> {
    type Iter<'a> = impl Iterator<Item = Piece<'a>>
    where
        Self: 'a;

    fn pack(&self) -> Self::Iter<'_> {
        pack_array_header(self.len()).chain(self.iter().flat_map(|elem| elem.pack()))
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
