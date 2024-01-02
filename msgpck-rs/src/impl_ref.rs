use crate::{MsgPack, Piece};

impl<'r, T> MsgPack for &'r T
where
    T: MsgPack,
{
    fn pack(&self) -> impl Iterator<Item = Piece<'_>> {
        (**self).pack()
    }
}

impl<'r, T> MsgPack for &'r mut T
where
    T: MsgPack,
{
    fn pack(&self) -> impl Iterator<Item = Piece<'_>> {
        (**self).pack()
    }
}
