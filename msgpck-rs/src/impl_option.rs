use core::iter;

use crate::{util::slice_take, Marker, MsgPack, MsgUnpack, Piece, UnpackErr};

impl<T: MsgPack> MsgPack for Option<T> {
    type Iter<'a> = impl Iterator<Item = Piece<'a>>
    where
        Self: 'a;

    /// Pack the Option.
    ///
    /// Note that `Option<Option<T>>` will pack into the same representation if either Option is
    /// `None`. i.e. there is no destinction between `None` and `Some(None)`.
    fn pack(&self) -> Self::Iter<'_> {
        iter::from_generator(move || match self {
            Some(i) => {
                for p in i.pack() {
                    yield p;
                }
            }
            None => yield Marker::Null.into(),
        })
    }
}

impl<'buf, T: MsgUnpack<'buf>> MsgUnpack<'buf> for Option<T> {
    /// Unpack the Option from msgpack bytes.
    ///
    /// Note that `Option<Option<T>>` will never unpack into `Some(None)` because of how Optionals
    /// are represented in msgpack.
    fn unpack(bytes: &mut &'buf [u8]) -> Result<Self, crate::UnpackErr>
    where
        Self: Sized,
    {
        // peek at the next marker, check if it's Null
        let &marker = bytes.first().ok_or(UnpackErr::UnexpectedEof)?;

        if let Marker::Null = marker.into() {
            // if it's Null, pop the marker and return None;
            let _ = slice_take::<_, 1>(bytes);
            return Ok(None);
        }

        let t = T::unpack(bytes)?;
        Ok(Some(t))
    }
}
