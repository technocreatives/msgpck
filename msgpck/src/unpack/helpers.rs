use crate::{UnMsgPck, UnpackError};

#[cfg(feature = "alloc")]
#[inline(never)]
pub fn unpack_bytes<'buf, T: UnMsgPck<'buf> + Sized>(source: &'buf [u8]) -> Result<T, UnpackError> {
    let mut source = source;
    UnMsgPck::unpack(&mut source)
}
