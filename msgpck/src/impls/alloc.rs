// TODO: impl for Box<T>

use crate::{slice_take, Marker, MsgPck, SizeHint, UnMsgPck, UnpackError};

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
