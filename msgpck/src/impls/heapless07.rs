use crate::{
    slice_take,
    utils::{pack_map_header, unpack_map_header},
    Marker, MsgPck, MsgWriter, PackError, UnMsgPck, UnpackError,
};
use heapless07::{LinearMap, String, Vec};

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

impl<K, V, const N: usize> MsgPck for LinearMap<K, V, N>
where
    K: MsgPck + Eq,
    V: MsgPck,
{
    fn pack(&self, writer: &mut dyn MsgWriter) -> Result<(), PackError> {
        pack_map_header(writer, self.len())?;
        for (k, v) in self.iter() {
            k.pack(writer)?;
            v.pack(writer)?;
        }
        Ok(())
    }
}

impl<'buf, K, V, const N: usize> UnMsgPck<'buf> for LinearMap<K, V, N>
where
    K: UnMsgPck<'buf> + Eq,
    V: UnMsgPck<'buf>,
{
    fn unpack(source: &mut &'buf [u8]) -> Result<Self, UnpackError> {
        let len = unpack_map_header(source)?;

        // sanity check: make sure buffer has enough data for this map
        if source.len() < len {
            return Err(UnpackError::UnexpectedEof);
        }

        (0..len)
            .map(move |_| {
                let k = K::unpack(source)?;
                let v = V::unpack(source)?;
                Ok((k, v))
            })
            .collect()
    }
}
