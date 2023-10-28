use msgpck::{slice_take, Marker, MsgPck, MsgWriter, PackError, UnMsgPck, UnpackError};
use proptest::prelude::*;
use proptest_derive::Arbitrary;

#[derive(Arbitrary, Debug, PartialEq)]
struct Scalars {
    a: bool,
    b: i8,
    c: String,
    d: f64,
}

impl MsgPck for Scalars {
    fn pack(&self, writer: &mut dyn MsgWriter) -> Result<(), PackError> {
        writer.write(&[Marker::FixArray(4).to_u8()])?;
        self.a.pack(writer)?;
        self.b.pack(writer)?;
        self.c.pack(writer)?;
        self.d.pack(writer)?;
        Ok(())
    }
}

impl<'buf> UnMsgPck<'buf> for Scalars {
    fn unpack(source: &mut &'buf [u8]) -> Result<Self, UnpackError>
    where
        Self: Sized,
    {
        let &[b] = slice_take(source).map_err(|_| dbg!(UnpackError::UnexpectedEof))?;
        let m = Marker::from_u8(b);
        if !matches!(m, Marker::FixArray(4)) {
            return Err(UnpackError::WrongMarker(m));
        };
        Ok(Scalars {
            a: bool::unpack(source)?,
            b: i8::unpack(source)?,
            c: String::unpack(source)?,
            d: f64::unpack(source)?,
        })
    }
}

proptest! {
    #[test]
    fn roundtrip(s: Scalars) {
        let mut writer: Vec<_> = Vec::new();
        s.pack(&mut writer).unwrap();
        let d = Scalars::unpack(&mut &writer[..]).unwrap();
        assert_eq!(s, d);
    }
}
