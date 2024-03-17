use crate::{marker::Marker, util::slice_take, MsgPack, MsgUnpack, Piece, UnpackErr};

impl MsgPack for f32 {
    fn pack(&self) -> impl Iterator<Item = Piece<'_>> {
        [Marker::F32.into(), Piece::Bytes4(self.to_be_bytes())].into_iter()
    }
}

impl<'buf> MsgUnpack<'buf> for f32 {
    fn unpack(bytes: &mut &'buf [u8]) -> Result<Self, crate::UnpackErr>
    where
        Self: Sized,
    {
        let &[b] = slice_take(bytes)?;

        let marker = Marker::from_u8(b);
        let Marker::F32 = marker else {
            return Err(UnpackErr::WrongMarker(marker));
        };

        Ok(f32::from_be_bytes(*slice_take(bytes)?))
    }
}

impl MsgPack for f64 {
    fn pack(&self) -> impl Iterator<Item = Piece<'_>> {
        [Marker::F64.into(), Piece::Bytes8(self.to_be_bytes())].into_iter()
    }
}

impl<'buf> MsgUnpack<'buf> for f64 {
    fn unpack(bytes: &mut &'buf [u8]) -> Result<Self, crate::UnpackErr>
    where
        Self: Sized,
    {
        let &[b] = slice_take(bytes)?;

        let marker = Marker::from_u8(b);
        let Marker::F64 = marker else {
            return Err(UnpackErr::WrongMarker(marker));
        };

        Ok(f64::from_be_bytes(*slice_take(bytes)?))
    }
}
