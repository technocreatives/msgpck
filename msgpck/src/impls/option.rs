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

    // FIXME: Returns None for Some(None)
    // roundtrip_proptest!(option_option_str: Option<Option<f32>>);
}
