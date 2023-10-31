use core::str::from_utf8;

use crate::{
    marker::Marker, utils::slice_take, MsgPck, MsgWriter, PackError, SizeHint, UnMsgPck,
    UnpackError,
};

impl MsgPck for str {
    #[cfg_attr(feature = "reduce-size", inline(never))]
    fn pack(&self, writer: &mut dyn MsgWriter) -> Result<(), PackError> {
        match self.len() {
            ..=0x1f => {
                writer.write(&[Marker::FixStr(self.len() as u8).to_u8()])?;
            }
            0x20..=0xff => {
                writer.write(&[Marker::Str8.to_u8(), self.len() as u8])?;
            }
            0x100..=0xffff => {
                let [a, b] = (self.len() as u16).to_be_bytes();
                writer.write(&[Marker::Str16.to_u8(), a, b])?;
            }
            _ => {
                let [a, b, c, d] = (self.len() as u32).to_be_bytes();
                writer.write(&[Marker::Str32.to_u8(), a, b, c, d])?;
            }
        }

        writer.write(self.as_bytes())?;
        Ok(())
    }

    fn size_hint(&self) -> SizeHint {
        let header = match self.len() {
            ..=0x1f => 1,
            0x20..=0xff => 2,
            0x100..=0xffff => 3,
            _ => 5,
        };
        SizeHint {
            min: Some(self.len() + header),
            max: Some(self.len() + header),
        }
    }
}

#[cfg(feature = "async")]
impl crate::AsyncMsgPck for str {
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

impl<'buf> UnMsgPck<'buf> for &'buf str {
    #[cfg_attr(feature = "reduce-size", inline(never))]
    fn unpack(source: &mut &'buf [u8]) -> Result<Self, UnpackError>
    where
        Self: Sized,
    {
        let &[b] = slice_take(source)?;
        let len: usize = match Marker::from_u8(b) {
            Marker::FixStr(len) => len.into(),
            Marker::Str8 => slice_take::<_, 1>(source)?[0].into(),
            Marker::Str16 => u16::from_be_bytes(*slice_take(source)?).into(),
            Marker::Str32 => u32::from_be_bytes(*slice_take(source)?).try_into()?,
            m => return Err(UnpackError::WrongMarker(m)),
        };

        if source.len() < len {
            return Err(UnpackError::UnexpectedEof);
        }
        let (data, rest) = source.split_at(len);
        *source = rest;

        Ok(from_utf8(data)?)
    }
}

#[cfg(feature = "alloc")]
impl MsgPck for String {
    #[inline]
    fn pack(&self, writer: &mut dyn MsgWriter) -> Result<(), PackError> {
        self.as_str().pack(writer)
    }

    #[inline]
    fn size_hint(&self) -> SizeHint {
        self.as_str().size_hint()
    }
}

#[cfg(all(feature = "async", feature = "alloc"))]
impl crate::AsyncMsgPck for String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn teeny_str() {
        let data = "y".repeat(0x4);

        let mut writer: Vec<_> = Vec::new();
        data.pack(&mut writer).unwrap();
        assert_eq!(data, <&str>::unpack(&mut &writer[..]).unwrap());
    }

    #[test]
    fn smol_str() {
        let data = "y".repeat(0x44);

        let mut writer: Vec<_> = Vec::new();
        data.pack(&mut writer).unwrap();
        assert_eq!(data, <&str>::unpack(&mut &writer[..]).unwrap());
    }

    #[test]
    fn medium_str() {
        let data = "y".repeat(0x444);

        let mut writer: Vec<_> = Vec::new();
        data.pack(&mut writer).unwrap();
        assert_eq!(data, <&str>::unpack(&mut &writer[..]).unwrap());
    }

    #[test]
    fn large_str() {
        let data = "y".repeat(0x44444);

        let mut writer: Vec<_> = Vec::new();
        data.pack(&mut writer).unwrap();
        assert_eq!(data, <&str>::unpack(&mut &writer[..]).unwrap());
    }

    roundtrip_proptest!(random_strings: String);
}
