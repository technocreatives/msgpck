use heapless07::{LinearMap, Vec};

use crate::{
    util::{pack_array, pack_map, unpack_array, unpack_map},
    MsgPack, MsgUnpack, Piece,
};

impl<T, const N: usize> MsgPack for Vec<T, N>
where
    T: MsgPack,
{
    type Iter<'a> = impl Iterator<Item = Piece<'a>>
    where
        Self: 'a;

    fn pack(&self) -> Self::Iter<'_> {
        pack_array(self.len(), self.iter())
    }
}

impl<'buf, T, const N: usize> MsgUnpack<'buf> for Vec<T, N>
where
    T: MsgUnpack<'buf> + 'buf,
{
    fn unpack(bytes: &mut &'buf [u8]) -> Result<Self, crate::UnpackErr>
    where
        Self: Sized,
    {
        unpack_array(bytes)
    }
}

impl<K, V, const N: usize> MsgPack for LinearMap<K, V, N>
where
    K: MsgPack + Eq,
    V: MsgPack,
{
    type Iter<'a> = impl Iterator<Item = Piece<'a>>
    where
        Self: 'a;

    fn pack(&self) -> Self::Iter<'_> {
        pack_map(self.len(), self.iter())
    }
}

impl<'buf, K, V, const N: usize> MsgUnpack<'buf> for LinearMap<K, V, N>
where
    K: MsgUnpack<'buf> + Eq,
    V: MsgUnpack<'buf>,
{
    fn unpack(bytes: &mut &'buf [u8]) -> Result<Self, crate::UnpackErr>
    where
        Self: Sized,
    {
        unpack_map(bytes)
    }
}
