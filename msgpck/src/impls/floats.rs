use core::mem::size_of;

use crate::{
    marker::Marker, utils::slice_take, MsgPck, PackError, SizeHint, UnMsgPck, UnpackError,
};

impl MsgPck for f32 {
    fn pack(&self, writer: &mut dyn crate::MsgWriter) -> Result<(), PackError> {
        writer.write(&[Marker::F32.to_u8()])?;
        writer.write(&self.to_be_bytes())?;
        Ok(())
    }

    fn size_hint(&self) -> SizeHint {
        SizeHint {
            min: Some(size_of::<Self>() + 1),
            max: Some(size_of::<Self>() + 1),
        }
    }
}

impl<'buf> UnMsgPck<'buf> for f32 {
    fn unpack(source: &mut &'buf [u8]) -> Result<Self, UnpackError>
    where
        Self: Sized,
    {
        let &[b] = slice_take(source)?;

        let marker = Marker::from_u8(b);
        let Marker::F32 = marker else {
            return Err(UnpackError::WrongMarker(marker));
        };

        Ok(f32::from_be_bytes(*slice_take(source)?))
    }
}

impl MsgPck for f64 {
    fn pack(&self, writer: &mut dyn crate::MsgWriter) -> Result<(), PackError> {
        writer.write(&[Marker::F64.to_u8()])?;
        writer.write(&self.to_be_bytes())?;
        Ok(())
    }

    fn size_hint(&self) -> SizeHint {
        SizeHint {
            min: Some(size_of::<Self>() + 1),
            max: Some(size_of::<Self>() + 1),
        }
    }
}

impl<'buf> UnMsgPck<'buf> for f64 {
    fn unpack(source: &mut &'buf [u8]) -> Result<Self, UnpackError>
    where
        Self: Sized,
    {
        let &[b] = slice_take(source)?;

        let marker = Marker::from_u8(b);
        let Marker::F64 = marker else {
            return Err(UnpackError::WrongMarker(marker));
        };

        Ok(f64::from_be_bytes(*slice_take(source)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    roundtrip_proptest!(test_f32: f32);
    roundtrip_proptest!(test_f64: f64);
}
