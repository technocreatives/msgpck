use crate::{marker::Marker, util::slice_take, MsgPack, MsgUnpack, Piece, UnpackErr};

impl MsgPack for [u8] {
    fn pack(&self) -> impl Iterator<Item = Piece<'_>> {
        let marker_piece;
        let len_piece;

        match self.len() {
            ..=0xff => {
                marker_piece = Marker::Bin8.into();
                len_piece = (self.len() as u8).into();
            }
            ..=0xffff => {
                marker_piece = Marker::Bin16.into();
                len_piece = (self.len() as u16).into();
            }
            _ => {
                marker_piece = Marker::Bin32.into();
                len_piece = (self.len() as u32).into();
            }
        }

        [marker_piece, len_piece, Piece::Bytes(self)].into_iter()
    }
}

impl<'buf> MsgUnpack<'buf> for &'buf [u8] {
    fn unpack(bytes: &mut &'buf [u8]) -> Result<Self, UnpackErr>
    where
        Self: Sized,
    {
        let &[b] = slice_take(bytes)?;
        let len: usize = match Marker::from_u8(b) {
            Marker::Bin8 => slice_take::<_, 1>(bytes)?[0].into(),
            Marker::Bin16 => u16::from_be_bytes(*slice_take(bytes)?).into(),
            Marker::Bin32 => u32::from_be_bytes(*slice_take(bytes)?).try_into()?,
            m => return Err(UnpackErr::WrongMarker(m)),
        };

        if len > bytes.len() {
            return Err(UnpackErr::UnexpectedEof);
        }

        let (bin, rest) = bytes.split_at(len);
        *bytes = rest;

        Ok(bin)
    }
}
