use crate::{MsgPack, Piece};
use core::iter;
use rmp::Marker;

impl<T: MsgPack> MsgPack for Vec<T> {
    type Iter<'a> = impl Iterator<Item = Piece<'a>>
    where
        Self: 'a;

    fn encode(&self) -> Self::Iter<'_> {
        iter::from_generator(move || {
            match self.len() {
                0..=0xf => yield Marker::FixArray(self.len() as u8).into(),
                0..=0xffff => {
                    yield Marker::Array16.into();
                    yield (self.len() as u16).into();
                }
                _ => {
                    yield Marker::Array32.into();
                    yield (self.len() as u32).into();
                }
            };

            for t in self {
                for m in t.encode() {
                    yield m;
                }
            }
        })
    }
}
