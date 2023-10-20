use crate::{MsgPack, Piece};
use core::iter;
use rmp::Marker;

// TODO: impl for signed integers

impl MsgPack for u8 {
    type Iter<'a> = impl Iterator<Item = Piece<'a>>
    where
        Self: 'a;

    fn encode(&self) -> Self::Iter<'_> {
        msgpack_u32(u32::from(*self))
    }
}

impl MsgPack for u16 {
    type Iter<'a> = impl Iterator<Item = Piece<'a>>
    where
        Self: 'a;

    fn encode(&self) -> Self::Iter<'_> {
        msgpack_u32(u32::from(*self))
    }
}

impl MsgPack for u32 {
    type Iter<'a> = impl Iterator<Item = Piece<'a>>
    where
        Self: 'a;

    fn encode(&self) -> Self::Iter<'_> {
        msgpack_u32(*self)
    }
}

fn msgpack_u32<'a>(n: u32) -> impl Iterator<Item = Piece<'a>> {
    iter::from_generator(move || match n {
        ..=0x7f => yield (n as u8).into(),
        ..=0xff => {
            yield Marker::U8.into();
            yield (n as u8).into();
        }
        ..=0xffff => {
            yield Marker::U16.into();
            yield (n as u16).into();
        }
        _ => {
            yield Marker::U32.into();
            yield n.into();
        }
    })
}
