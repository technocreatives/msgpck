use core::mem::size_of;

use crate::{marker::Marker, utils::slice_take, SizeHint, *};

impl MsgPck for bool {
    fn pack(&self, writer: &mut dyn MsgWriter) -> Result<(), PackError> {
        let data = if *self { Marker::True } else { Marker::False };
        writer.write(&[data.to_u8()])?;
        Ok(())
    }

    fn size_hint(&self) -> SizeHint {
        SizeHint {
            min: Some(size_of::<Self>()),
            max: Some(size_of::<Self>()),
        }
    }
}

impl<'buf> UnMsgPck<'buf> for bool {
    fn unpack(source: &mut &'buf [u8]) -> Result<Self, UnpackError>
    where
        Self: Sized,
    {
        let &[b] = slice_take(source)?;
        let marker = Marker::from_u8(b);
        match marker {
            Marker::True => Ok(true),
            Marker::False => Ok(false),
            _ => Err(UnpackError::WrongMarker(marker)),
        }
    }
}

#[cfg(test)]
mod tests {
    // that's the entire point
    #![allow(clippy::bool_assert_comparison)]

    use super::*;

    #[test]
    fn test_true() {
        let mut writer: Vec<_> = Vec::new();
        true.pack(&mut writer).unwrap();
        assert_eq!(true, bool::unpack(&mut &writer[..]).unwrap());
    }

    #[test]
    fn test_false() {
        let mut writer: Vec<_> = Vec::new();
        false.pack(&mut writer).unwrap();
        assert_eq!(false, bool::unpack(&mut writer.as_slice()).unwrap());
    }

    roundtrip_proptest!(booleans: bool);
}
