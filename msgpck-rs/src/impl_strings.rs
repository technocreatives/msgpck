use crate::{marker::Marker, util::slice_take, MsgPack, MsgUnpack, Piece, UnpackErr};
use core::str::from_utf8;

impl MsgPack for str {
    fn pack(&self) -> impl Iterator<Item = Piece<'_>> {
        let marker_piece;
        let len_piece;

        match self.len() {
            ..=0x1f => {
                marker_piece = Marker::FixStr(self.len() as u8).into();
                len_piece = None;
            }
            ..=0xff => {
                marker_piece = Marker::Str8.into();
                len_piece = Some((self.len() as u8).into());
            }
            ..=0xffff => {
                marker_piece = Marker::Str16.into();
                len_piece = Some((self.len() as u16).into());
            }
            _ => {
                marker_piece = Marker::Str32.into();
                len_piece = Some((self.len() as u32).into());
            }
        }

        [
            Some(marker_piece),
            len_piece,
            Some(Piece::Bytes(self.as_bytes())),
        ]
        .into_iter()
        .flatten()
    }
}

impl MsgPack for &str {
    fn pack(&self) -> impl Iterator<Item = Piece<'_>> {
        str::pack(self)
    }
}

impl<'buf> MsgUnpack<'buf> for &'buf str {
    fn unpack(bytes: &mut &'buf [u8]) -> Result<Self, UnpackErr>
    where
        Self: Sized,
    {
        let &[b] = slice_take(bytes)?;
        let len: usize = match Marker::from_u8(b) {
            Marker::FixStr(len) => len.into(),
            Marker::Str8 => slice_take::<_, 1>(bytes)?[0].into(),
            Marker::Str16 => u16::from_be_bytes(*slice_take(bytes)?).into(),
            Marker::Str32 => u32::from_be_bytes(*slice_take(bytes)?).try_into()?,
            m => return Err(UnpackErr::WrongMarker(m)),
        };

        if len > bytes.len() {
            return Err(UnpackErr::UnexpectedEof);
        }

        //let str_bytes: &'buf [u8] = bytes.take(..len).ok_or(UnpackErr::UnexpectedEof)?;
        let (str_bytes, rest) = bytes.split_at(len);
        *bytes = rest;

        Ok(from_utf8(str_bytes)?)
    }
}
