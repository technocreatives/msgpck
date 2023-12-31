use msgpck::{slice_take, Marker, MsgPck, MsgWriter, PackError, UnMsgPck, UnpackError};
use proptest::prelude::*;
use proptest_derive::Arbitrary;

#[derive(Arbitrary, Debug, PartialEq)]
struct Foo {
    a: bool,
    b: String,
}

#[derive(Arbitrary, Debug, PartialEq)]
struct Bar {
    a: f32,
    foo: Foo,
}

impl MsgPck for Foo {
    fn pack(&self, writer: &mut dyn MsgWriter) -> Result<(), PackError> {
        writer.write(&[Marker::FixArray(4).to_u8()])?;
        self.a.pack(writer)?;
        self.b.pack(writer)?;
        Ok(())
    }
}

impl<'buf> UnMsgPck<'buf> for Foo {
    fn unpack(source: &mut &'buf [u8]) -> Result<Self, UnpackError>
    where
        Self: Sized,
    {
        let &[b] = slice_take(source).map_err(|_| dbg!(UnpackError::UnexpectedEof))?;
        let m = Marker::from_u8(b);
        if !matches!(m, Marker::FixArray(4)) {
            return Err(UnpackError::WrongMarker(m));
        };
        Ok(Foo {
            a: bool::unpack(source)?,
            b: String::unpack(source)?,
        })
    }
}

impl MsgPck for Bar {
    fn pack(&self, writer: &mut dyn MsgWriter) -> Result<(), PackError> {
        writer.write(&[Marker::FixArray(4).to_u8()])?;
        self.a.pack(writer)?;
        self.foo.pack(writer)?;
        Ok(())
    }
}

impl<'buf> UnMsgPck<'buf> for Bar {
    fn unpack(source: &mut &'buf [u8]) -> Result<Self, UnpackError>
    where
        Self: Sized,
    {
        let &[b] = slice_take(source).map_err(|_| dbg!(UnpackError::UnexpectedEof))?;
        let m = Marker::from_u8(b);
        if !matches!(m, Marker::FixArray(4)) {
            return Err(UnpackError::WrongMarker(m));
        };
        Ok(Bar {
            a: f32::unpack(source)?,
            foo: Foo::unpack(source)?,
        })
    }
}

proptest! {
    #[test]
    fn roundtrip_foo(s: Foo) {
        let mut writer: Vec<_> = Vec::new();
        s.pack(&mut writer).unwrap();
        let d = Foo::unpack(&mut &writer[..]).unwrap();
        assert_eq!(s, d);
    }

    #[test]
    fn roundtrip_bar(s: Bar) {
        let mut writer: Vec<_> = Vec::new();
        s.pack(&mut writer).unwrap();
        let d = Bar::unpack(&mut &writer[..]).unwrap();
        assert_eq!(s, d);
    }
}
