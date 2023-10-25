use core::iter;

use rmp::Marker;

use crate::{util::slice_take, MsgPack, MsgUnpack, Piece, UnpackErr};

impl MsgPack for bool {
    type Iter<'a> = impl Iterator<Item = Piece<'a>>
    where
        Self: 'a;

    fn pack(&self) -> Self::Iter<'_> {
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
