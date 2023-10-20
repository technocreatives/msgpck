use crate::{MsgPack, Piece};
use rmp::Marker;

impl MsgPack for f32 {
    type Iter<'a> = impl Iterator<Item = Piece<'a>>
    where
        Self: 'a;

    fn pack(&self) -> Self::Iter<'_> {
        [Marker::F32.into(), Piece::Bytes4(self.to_le_bytes())].into_iter()
    }
}

impl MsgPack for f64 {
    type Iter<'a> = impl Iterator<Item = Piece<'a>>
    where
        Self: 'a;

    fn pack(&self) -> Self::Iter<'_> {
        [Marker::F64.into(), Piece::Bytes8(self.to_le_bytes())].into_iter()
    }
}
