// TODO: impl for Box<T>

use crate::{slice_take, ByteSlice, Marker, MsgPck, SizeHint, UnMsgPck, UnpackError};

impl<'buf> UnMsgPck<'buf> for String {
    fn unpack(source: &mut &'buf [u8]) -> Result<Self, UnpackError> {
        <&str>::unpack(source).map(|s| s.to_owned())
    }
}

impl<T: MsgPck> MsgPck for Vec<T> {
    fn pack(&self, writer: &mut dyn crate::MsgWriter) -> Result<(), crate::PackError> {
        self.as_slice().pack(writer)
    }

    fn size_hint(&self) -> SizeHint {
        self.as_slice().size_hint()
    }
}

/// Helper type to pack a byte slice as a MessagePack binary instead of a MessagePack array.
#[derive(PartialEq, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ByteVec(Vec<u8>);

impl ByteVec {
    pub fn new(buf: Vec<u8>) -> Self {
        Self(buf)
    }

    pub fn as_byte_slice(&self) -> ByteSlice {
        ByteSlice::new(&self.0)
    }
}

impl MsgPck for ByteVec {
    fn pack(&self, writer: &mut dyn crate::MsgWriter) -> Result<(), crate::PackError> {
        self.as_byte_slice().pack(writer)
    }

    fn size_hint(&self) -> SizeHint {
        self.as_byte_slice().size_hint()
    }
}

impl<'buf> UnMsgPck<'buf> for ByteVec {
    #[cfg_attr(feature = "reduce-size", inline(never))]
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
        let (data, rest) = source.split_at(len);
        *source = rest;

        Ok(ByteVec::new(data.to_owned()))
    }
}

#[cfg(feature = "async")]
impl<T: crate::AsyncMsgPck> crate::AsyncMsgPck for Vec<T> {
    async fn pack_async(
        &self,
        writer: impl embedded_io_async::Write,
    ) -> Result<(), crate::PackError> {
        self.as_slice().pack_async(writer).await?;
        Ok(())
    }
}

impl<'buf, T: UnMsgPck<'buf>> UnMsgPck<'buf> for Vec<T> {
    #[cfg_attr(feature = "reduce-size", inline(never))]
    fn unpack(source: &mut &'buf [u8]) -> Result<Self, UnpackError> {
        let &[b] = slice_take(source)?;
        let len: usize = match Marker::from_u8(b) {
            Marker::FixArray(len) => len.into(),
            Marker::Array16 => u16::from_be_bytes(*slice_take(source)?).into(),
            Marker::Array32 => u32::from_be_bytes(*slice_take(source)?).try_into()?,
            m => return Err(UnpackError::WrongMarker(m)),
        };

        (0..len).map(move |_| T::unpack(source)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    roundtrip_proptest!(vec_str: Vec<String>);
}
