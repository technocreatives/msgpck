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

#[cfg(all(feature = "async", feature = "alloc"))]
impl<const N: usize> crate::AsyncMsgPck for String<N> {
    async fn pack_async(
        &self,
        mut writer: impl embedded_io_async::Write,
    ) -> Result<(), crate::PackError> {
        match self.len() {
            ..=0x1f => {
                writer
                    .write_all(&[Marker::FixStr(self.len() as u8).to_u8()])
                    .await?;
            }
            0x20..=0xff => {
                writer
                    .write_all(&[Marker::Str8.to_u8(), self.len() as u8])
                    .await?;
            }
            0x100..=0xffff => {
                let [a, b] = (self.len() as u16).to_be_bytes();
                writer.write_all(&[Marker::Str16.to_u8(), a, b]).await?;
            }
            _ => {
                let [a, b, c, d] = (self.len() as u32).to_be_bytes();
                writer
                    .write_all(&[Marker::Str32.to_u8(), a, b, c, d])
                    .await?;
            }
        }

        writer.write_all(self.as_bytes()).await?;
        Ok(())
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

#[cfg(feature = "async")]
impl<T: crate::AsyncMsgPck, const N: usize> crate::AsyncMsgPck for Vec<T, N> {
    async fn pack_async(
        &self,
        writer: impl embedded_io_async::Write,
    ) -> Result<(), crate::PackError> {
        self.as_slice().pack_async(writer).await?;
        Ok(())
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

#[cfg(feature = "async")]
impl<K, V, const N: usize> crate::AsyncMsgPck for LinearMap<K, V, N>
where
    K: crate::AsyncMsgPck + Eq,
    V: crate::AsyncMsgPck,
{
    async fn pack_async(
        &self,
        mut writer: impl embedded_io_async::Write,
    ) -> Result<(), crate::PackError> {
        crate::utils::pack_map_header_async(&mut writer, self.len()).await?;
        for (k, v) in self.iter() {
            k.pack_async(&mut writer).await?;
            v.pack_async(&mut writer).await?;
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
