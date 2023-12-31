use crate::{MsgPck, MsgWriter, PackError, UnMsgPck, UnpackError};

const NONE: u8 = 0xc0;

impl<T: MsgPck> MsgPck for Option<T> {
    fn pack(&self, writer: &mut dyn MsgWriter) -> Result<(), PackError> {
        match self {
            Some(data) => data.pack(writer),
            None => Ok(writer.write(&[NONE])?),
        }
    }
}

#[cfg(feature = "async")]
impl<T: crate::AsyncMsgPck> crate::AsyncMsgPck for Option<T> {
    async fn pack_async(
        &self,
        mut writer: impl embedded_io_async::Write,
    ) -> Result<(), crate::PackError> {
        match self {
            Some(data) => data.pack_async(writer).await?,
            None => writer.write_all(&[NONE]).await?,
        };
        Ok(())
    }
}

impl<'buf, T: UnMsgPck<'buf>> UnMsgPck<'buf> for Option<T> {
    fn unpack(source: &mut &'buf [u8]) -> Result<Self, UnpackError>
    where
        Self: Sized,
    {
        let Some(first_byte) = source.first() else {
            return Err(UnpackError::UnexpectedEof);
        };
        if *first_byte == NONE {
            Ok(None)
        } else {
            Ok(Some(T::unpack(source)?))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none() {
        let s: Option<String> = None;
        let mut writer: Vec<_> = Vec::new();
        s.pack(&mut writer).unwrap();
        let d = <Option<String>>::unpack(&mut &writer[..]).unwrap();
        assert_eq!(s, d);
    }

    #[test]
    fn some() {
        let s: Option<String> = Some("hello".into());
        let mut writer: Vec<_> = Vec::new();
        s.pack(&mut writer).unwrap();

        // use pretty_hex::*;
        // dbg!(&writer.hex_dump());

        let d = <Option<String>>::unpack(&mut &writer[..]).unwrap();
        assert_eq!(s, d);
    }

    roundtrip_proptest!(option_str: Option<String>);
    roundtrip_proptest!(option_vec_f32: Option<Vec<f32>>);
}
