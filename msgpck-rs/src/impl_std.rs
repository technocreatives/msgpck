use core::hash::Hash;
use std::collections::HashMap;

use crate::{
    util::{pack_map, unpack_map},
    MsgPack, MsgUnpack, Piece,
};

impl<K, V> MsgPack for HashMap<K, V>
where
    K: MsgPack,
    V: MsgPack,
{
    type Iter<'a> = impl Iterator<Item = Piece<'a>>
    where
        Self: 'a;

    fn pack(&self) -> Self::Iter<'_> {
        pack_map(self.len(), self.iter())
    }
}

impl<'buf, K, V> MsgUnpack<'buf> for HashMap<K, V>
where
    K: MsgUnpack<'buf> + Hash + Eq,
    V: MsgUnpack<'buf>,
{
    fn unpack(bytes: &mut &'buf [u8]) -> Result<Self, crate::UnpackErr>
    where
        Self: Sized,
    {
        unpack_map(bytes)
    }
}
