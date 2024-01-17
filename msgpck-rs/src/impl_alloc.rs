use core::ops::Deref;

use crate::{
    util::{pack_array, unpack_array},
    MsgPack, MsgUnpack, Piece,
};
use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

impl<T: MsgPack> MsgPack for Vec<T> {
    fn pack(&self) -> impl Iterator<Item = Piece<'_>> {
        pack_array(self.len(), self.iter())
    }
}

impl<'buf, T: MsgUnpack<'buf> + 'buf> MsgUnpack<'buf> for Vec<T> {
    fn unpack(bytes: &mut &'buf [u8]) -> Result<Self, crate::UnpackErr>
    where
        Self: Sized,
    {
        unpack_array(bytes)
    }
}

impl<T: MsgPack> MsgPack for Box<T> {
    #[inline(always)]
    fn pack(&self) -> impl Iterator<Item = Piece<'_>> {
        self.deref().pack()
    }
}

impl<'buf, T: MsgUnpack<'buf> + 'buf> MsgUnpack<'buf> for Box<T> {
    #[inline(always)]
    fn unpack(bytes: &mut &'buf [u8]) -> Result<Self, crate::UnpackErr>
    where
        Self: Sized,
    {
        T::unpack(bytes).map(Box::new)
    }
}

impl MsgPack for String {
    #[inline(always)]
    fn pack(&self) -> impl Iterator<Item = Piece<'_>> {
        self.deref().pack()
    }
}

impl<'buf> MsgUnpack<'buf> for String {
    #[inline(always)]
    fn unpack(bytes: &mut &'buf [u8]) -> Result<Self, crate::UnpackErr>
    where
        Self: Sized,
    {
        let s: &str = MsgUnpack::unpack(bytes)?;
        Ok(s.to_owned())
    }
}
