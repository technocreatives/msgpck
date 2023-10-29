use crate::{pack::SizeHint, Marker, MsgPck, MsgWriter, PackError};

impl<'a, T: MsgPck> MsgPck for &'a [T] {
    fn pack(&self, writer: &mut dyn MsgWriter) -> Result<(), PackError> {
        match self.len() {
            ..=0xff => {
                writer.write(&[Marker::FixArray(self.len() as u8).to_u8()])?;
            }
            0x100..=0xffff => {
                let [a, b] = (self.len() as u16).to_be_bytes();
                writer.write(&[Marker::Array16.to_u8(), a, b])?;
            }
            _ => {
                let [a, b, c, d] = (self.len() as u32).to_be_bytes();
                writer.write(&[Marker::Array32.to_u8(), a, b, c, d])?;
            }
        }

        for item in *self {
            item.pack(writer)?;
        }
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
