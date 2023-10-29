use crate::{slice_take, Marker, MsgPck, MsgWriter, PackError, UnMsgPck, UnpackError};
use heapless07::{String, Vec};

impl<const N: usize> MsgPck for String<N> {
    fn pack(&self, writer: &mut dyn MsgWriter) -> Result<(), PackError> {
        self.as_str().pack(writer)
    }
}

impl<'buf, const N: usize> UnMsgPck<'buf> for String<N> {
    fn unpack(source: &mut &'buf [u8]) -> Result<Self, UnpackError> {
        <&str>::unpack(source).map(String::from)
    }
}

impl<T: MsgPck, const N: usize> MsgPck for Vec<T, N> {
    fn pack(&self, writer: &mut dyn MsgWriter) -> Result<(), PackError> {
        self.as_slice().pack(writer)
    }
}

impl<'buf, T: UnMsgPck<'buf>, const N: usize> UnMsgPck<'buf> for Vec<T, N> {
    fn unpack(source: &mut &'buf [u8]) -> Result<Self, UnpackError> {
        let &[b] = slice_take(source)?;
        let len: usize = match Marker::from_u8(b) {
            Marker::FixArray(len) => len.into(),
            Marker::Array16 => u16::from_be_bytes(*slice_take(source)?).into(),
            Marker::Array32 => u32::from_be_bytes(*slice_take(source)?).try_into()?,
            m => return Err(UnpackError::WrongMarker(m)),
        };

        if source.len() < len {
            return Err(UnpackError::UnexpectedEof);
        }
        let (mut data, rest) = source.split_at(len);
        *source = rest;

        (0..len).map(move |_| T::unpack(&mut data)).collect()
    }
}
