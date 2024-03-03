use crate::{marker::Marker, util::slice_take, MsgPack, MsgUnpack, Piece, UnpackErr};
use core::iter;

impl MsgPack for bool {
    fn pack(&self) -> impl Iterator<Item = Piece<'_>> {
        let marker = if *self { Marker::True } else { Marker::False };
        iter::once(marker.into())
    }
}

impl<'buf> MsgUnpack<'buf> for bool {
    fn unpack(bytes: &mut &'buf [u8]) -> Result<Self, crate::UnpackErr>
    where
        Self: Sized,
    {
        let &[b] = slice_take(bytes)?;
        match Marker::from_u8(b) {
            Marker::True => Ok(true),
            Marker::False => Ok(false),
            marker => Err(UnpackErr::WrongMarker(marker)),
        }
    }
}
