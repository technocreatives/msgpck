use crate::{
    util::{pack_array, pack_map, unpack_array, unpack_map},
    MsgPack, MsgUnpack, Piece, UnpackErr,
};
use core::ops::Deref;
use heapless07::{LinearMap, String, Vec};

impl<T, const N: usize> MsgPack for Vec<T, N>
where
    T: MsgPack,
{
    fn pack(&self) -> impl Iterator<Item = Piece<'_>> {
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

impl<const N: usize> MsgPack for String<N> {
    fn pack(&self) -> impl Iterator<Item = Piece<'_>> {
        self.deref().pack()
    }
}

impl<'buf, const N: usize> MsgUnpack<'buf> for String<N> {
    fn unpack(bytes: &mut &'buf [u8]) -> Result<Self, crate::UnpackErr>
    where
        Self: Sized,
    {
        let s: &str = MsgUnpack::unpack(bytes)?;
        if s.len() > N {
            Err(UnpackErr::BufferOverflow)
        } else {
            Ok(s.into())
        }
    }
}

impl<K, V, const N: usize> MsgPack for LinearMap<K, V, N>
where
    K: MsgPack + Eq,
    V: MsgPack,
{
    fn pack(&self) -> impl Iterator<Item = Piece<'_>> {
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
