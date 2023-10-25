use crate::{MsgPack, MsgUnpack, Piece, UnpackErr};
use rmp::decode::NumValueReadError;

impl MsgPack for i8 {
    type Iter<'a> = impl Iterator<Item = Piece<'a>>
    where
        Self: 'a;

    fn pack(&self) -> Self::Iter<'_> {
        pack_i64(i64::from(*self))
    }
}

impl MsgPack for i16 {
    type Iter<'a> = impl Iterator<Item = Piece<'a>>
    where
        Self: 'a;

    fn pack(&self) -> Self::Iter<'_> {
        pack_i64(i64::from(*self))
    }
}

impl MsgPack for i32 {
    type Iter<'a> = impl Iterator<Item = Piece<'a>>
    where
        Self: 'a;

    fn pack(&self) -> Self::Iter<'_> {
        pack_i64(i64::from(*self))
    }
}

impl MsgPack for i64 {
    type Iter<'a> = impl Iterator<Item = Piece<'a>>
    where
        Self: 'a;

    fn pack(&self) -> Self::Iter<'_> {
        pack_i64(*self)
    }
}

impl<'buf> MsgUnpack<'buf> for i8 {
    fn unpack(bytes: &mut &'buf [u8]) -> Result<Self, UnpackErr>
    where
        Self: Sized,
    {
        let n = unpack_i64(bytes)?;
        Ok(n.try_into()?)
    }
}

impl<'buf> MsgUnpack<'buf> for i16 {
    fn unpack(bytes: &mut &'buf [u8]) -> Result<Self, UnpackErr>
    where
        Self: Sized,
    {
        let n = unpack_i64(bytes)?;
        Ok(n.try_into()?)
    }
}

impl<'buf> MsgUnpack<'buf> for i32 {
    fn unpack(bytes: &mut &'buf [u8]) -> Result<Self, UnpackErr>
    where
        Self: Sized,
    {
        let n = unpack_i64(bytes)?;
        Ok(n.try_into()?)
    }
}

impl<'buf> MsgUnpack<'buf> for i64 {
    fn unpack(bytes: &mut &'buf [u8]) -> Result<Self, UnpackErr>
    where
        Self: Sized,
    {
        unpack_i64(bytes)
    }
}

pub(crate) fn unpack_i64(bytes: &mut &[u8]) -> Result<i64, UnpackErr> {
    rmp::decode::read_int(bytes).map_err(|e| match e {
        NumValueReadError::TypeMismatch(m) => UnpackErr::WrongMarker(m),
        _ => UnpackErr::UnexpectedEof,
    })
}

pub(crate) fn pack_i64<'a>(n: i64) -> impl Iterator<Item = Piece<'a>> {
    const INT_MAX_SIZE: usize = 9;
    let mut buf = [0u8; INT_MAX_SIZE];
    let mut w = &mut buf[..];

    let _ = rmp::encode::write_sint(&mut w, n);
    let bytes_written = INT_MAX_SIZE - w.len();

    let (&marker, data) = buf[..bytes_written]
        .split_first()
        .expect("rmp::write_sint must encode at least 1 byte");

    let data = match data.len() {
        0 => None,
        1 => Some(Piece::Byte(data[0])),
        2 => Some(Piece::Bytes2(data.try_into().unwrap())),
        4 => Some(Piece::Bytes4(data.try_into().unwrap())),
        8 => Some(Piece::Bytes8(data.try_into().unwrap())),
        _ => unreachable!(
            "msgpack integers are always encoded as 0, 1, 2, 4, or 8 bytes (excluding marker)"
        ),
    };

    [Some(Piece::Byte(marker)), data].into_iter().flatten()
}
