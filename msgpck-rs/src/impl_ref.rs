use crate::{MsgPack, Piece};

impl<'r, T> MsgPack for &'r T
where
    T: MsgPack,
{
    type Iter<'a> = impl Iterator<Item = Piece<'a>>
    where
        Self: 'a;

    fn pack(&self) -> Self::Iter<'_> {
        (**self).pack()
    }
}

impl<'r, T> MsgPack for &'r mut T
where
    T: MsgPack,
{
    type Iter<'a> = impl Iterator<Item = Piece<'a>>
    where
        Self: 'a;

    fn pack(&self) -> Self::Iter<'_> {
        (**self).pack()
    }
}
