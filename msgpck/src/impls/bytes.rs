use crate::{
    marker::Marker, utils::slice_take, MsgPck, MsgWriter, PackError, SizeHint, UnMsgPck,
    UnpackError,
};

/// Helper type to pack a byte slice as a MessagePack binary instead of a MessagePack array.
#[derive(PartialEq, Clone, Copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ByteSlice<'buf>(&'buf [u8]);

impl<'buf> ByteSlice<'buf> {
    pub fn new(buf: &'buf [u8]) -> Self {
        Self(buf)
    }
}

impl core::ops::Deref for ByteSlice<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'buf> MsgPck for ByteSlice<'buf> {
    #[cfg_attr(feature = "reduce-size", inline(never))]
    fn pack(&self, writer: &mut dyn MsgWriter) -> Result<(), PackError> {
        match self.len() {
            ..=0xff => {
                writer.write(&[Marker::Bin8.to_u8(), self.len() as u8])?;
            }
            0x100..=0xffff => {
                let [a, b] = (self.len() as u16).to_be_bytes();
                writer.write(&[Marker::Bin16.to_u8(), a, b])?;
            }
            _ => {
                let [a, b, c, d] = (self.len() as u32).to_be_bytes();
                writer.write(&[Marker::Bin32.to_u8(), a, b, c, d])?;
            }
        }

        writer.write(self)?;
        Ok(())
    }

    fn size_hint(&self) -> SizeHint {
        let header = match self.len() {
            ..=0xff => 1,
            0x100..=0xffff => 3,
            _ => 5,
        };
        SizeHint {
            min: Some(self.len() + header),
            max: Some(self.len() + header),
        }
    }
}

impl<'buf> UnMsgPck<'buf> for ByteSlice<'buf> {
    #[cfg_attr(feature = "reduce-size", inline(never))]
    fn unpack(source: &mut &'buf [u8]) -> Result<Self, UnpackError>
    where
        Self: Sized,
    {
        let &[b] = slice_take(source)?;
        let len: usize = match Marker::from_u8(b) {
            Marker::Bin8 => slice_take::<_, 1>(source)?[0].into(),
            Marker::Bin16 => u16::from_be_bytes(*slice_take(source)?).into(),
            Marker::Bin32 => u32::from_be_bytes(*slice_take(source)?).try_into()?,
            m => return Err(UnpackError::WrongMarker(m)),
        };

        if source.len() < len {
            return Err(UnpackError::UnexpectedEof);
        }
        let (data, rest) = source.split_at(len);
        *source = rest;

        Ok(ByteSlice(data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smol_array() {
        let data = ByteSlice(&[1, 2, 3]);

        let mut writer: Vec<_> = Vec::new();
        data.pack(&mut writer).unwrap();
        assert_eq!(data, <ByteSlice>::unpack(&mut &writer[..]).unwrap());
    }

    #[test]
    fn medium_array() {
        let data = ByteSlice(&[42; 0x101]);

        let mut writer: Vec<_> = Vec::new();
        data.pack(&mut writer).unwrap();
        assert_eq!(data, <ByteSlice>::unpack(&mut &writer[..]).unwrap());
    }

    #[test]
    fn large_array() {
        let data = ByteSlice(&[42; 0x10101]);

        let mut writer: Vec<_> = Vec::new();
        data.pack(&mut writer).unwrap();
        assert_eq!(data, <ByteSlice>::unpack(&mut &writer[..]).unwrap());
    }
}
