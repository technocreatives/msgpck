use crate::{
    utils::unpack_map_header, Marker, MsgPck, MsgWriter, PackError, UnMsgPck, UnpackError,
};

/// The header/key of a msgpack-encoded enum value.
pub struct EnumHeader<'v> {
    pub variant: Variant<'v>,

    pub unit: bool,
}

/// The discriminant or name of an enum variant.
pub enum Variant<'a> {
    Discriminant(u64),
    Name(&'a str),
}

impl<'a> From<&'a str> for Variant<'a> {
    fn from(name: &'a str) -> Self {
        Variant::Name(name)
    }
}

impl<'a> From<u64> for Variant<'a> {
    fn from(discriminant: u64) -> Self {
        Variant::Discriminant(discriminant)
    }
}

impl<'v> MsgPck for EnumHeader<'v> {
    fn pack(&self, writer: &mut dyn MsgWriter) -> Result<(), PackError> {
        if !self.unit {
            writer.write(&[Marker::FixMap(1).to_u8()])?;
        }
        match self.variant {
            Variant::Discriminant(n) => n.pack(writer)?,
            Variant::Name(s) => s.pack(writer)?,
        }

        Ok(())
    }
}

impl<'buf: 'v, 'v> UnMsgPck<'buf> for EnumHeader<'v> {
    fn unpack(source: &mut &'buf [u8]) -> Result<Self, crate::UnpackError>
    where
        Self: Sized,
    {
        if source.is_empty() {
            return Err(UnpackError::UnexpectedEof);
        }

        match Marker::from_u8(source[0]) {
            Marker::FixStr(..) | Marker::Str8 | Marker::Str16 | Marker::Str32 => {
                return Ok(EnumHeader {
                    variant: Variant::Name(UnMsgPck::unpack(source)?),
                    unit: true,
                });
            }
            Marker::FixPos(..) | Marker::U8 | Marker::U16 | Marker::U32 | Marker::U64 => {
                return Ok(EnumHeader {
                    variant: Variant::Discriminant(UnMsgPck::unpack(source)?),
                    unit: true,
                });
            }

            // if the enum is a map, it has at least 1 field.
            Marker::FixMap(_) | Marker::Map16 | Marker::Map32 => {
                let len = unpack_map_header(source)?;
                if len != 1 {
                    todo!("error on invalid enum map")
                }
            }
            m => return Err(UnpackError::WrongMarker(m)),
        }

        let variant = match Marker::from_u8(source[0]) {
            Marker::FixPos(..) | Marker::U8 | Marker::U16 | Marker::U32 | Marker::U64 => {
                Variant::Discriminant(UnMsgPck::unpack(source)?)
            }
            Marker::FixStr(..) | Marker::Str8 | Marker::Str16 | Marker::Str32 => {
                Variant::Name(UnMsgPck::unpack(source)?)
            }
            m => return Err(UnpackError::WrongMarker(m)),
        };

        Ok(EnumHeader {
            variant,
            unit: false,
        })
    }
}

#[cfg(test)]
mod tests_simple {
    use super::*;
    use proptest::prelude::*;
    use proptest_derive::Arbitrary;

    #[derive(Debug, PartialEq, Arbitrary)]
    enum Foo {
        Bar,
        Baz,
    }

    impl MsgPck for Foo {
        fn pack(&self, writer: &mut dyn MsgWriter) -> Result<(), PackError> {
            match self {
                Foo::Bar => EnumHeader {
                    variant: Variant::Name("Bar"),
                    unit: true,
                }
                .pack(writer)?,
                Foo::Baz => EnumHeader {
                    variant: Variant::Name("Baz"),
                    unit: true,
                }
                .pack(writer)?,
            }
            Ok(())
        }
    }

    impl<'buf> UnMsgPck<'buf> for Foo {
        fn unpack(source: &mut &'buf [u8]) -> Result<Self, UnpackError>
        where
            Self: Sized,
        {
            let header = EnumHeader::unpack(source)?;
            match header.variant {
                Variant::Name("Bar") => Ok(Foo::Bar),
                Variant::Name("Baz") => Ok(Foo::Baz),
                _ => todo!(),
            }
        }
    }

    proptest! {
        #[test]
        fn roundtrip(data: Foo) {
            let mut writer: Vec<_> = Vec::new();
            data.pack(&mut writer).unwrap();
            assert_eq!(data, Foo::unpack(&mut &writer[..]).unwrap());
        }
    }
}

#[cfg(test)]
mod tests_nested {
    use super::*;
    use proptest::prelude::*;
    use proptest_derive::Arbitrary;

    #[derive(Debug, PartialEq, Arbitrary)]
    enum Foo {
        Bar { x: i8 },
        Baz(String),
        Nope,
    }

    impl MsgPck for Foo {
        fn pack(&self, writer: &mut dyn MsgWriter) -> Result<(), PackError> {
            match self {
                Foo::Bar { x } => {
                    EnumHeader {
                        variant: Variant::Name("Bar"),
                        unit: false,
                    }
                    .pack(writer)?;
                    x.pack(writer)?;
                }
                Foo::Baz(s) => {
                    EnumHeader {
                        variant: Variant::Name("Baz"),
                        unit: false,
                    }
                    .pack(writer)?;
                    s.pack(writer)?;
                }
                Foo::Nope => EnumHeader {
                    variant: Variant::Name("Nope"),
                    unit: true,
                }
                .pack(writer)?,
            }
            Ok(())
        }
    }

    impl<'buf> UnMsgPck<'buf> for Foo {
        fn unpack(source: &mut &'buf [u8]) -> Result<Self, UnpackError>
        where
            Self: Sized,
        {
            let header = EnumHeader::unpack(source)?;
            match header.variant {
                Variant::Name("Bar") => {
                    if header.unit {
                        return Err(UnpackError::UnexpectedUnitVariant);
                    }
                    Ok(Self::Bar {
                        x: UnMsgPck::unpack(source)?,
                    })
                }
                Variant::Name("Baz") => {
                    if header.unit {
                        return Err(UnpackError::UnexpectedUnitVariant);
                    }
                    Ok(Self::Baz(UnMsgPck::unpack(source)?))
                }
                Variant::Name("Nope") => {
                    if !header.unit {
                        return Err(UnpackError::ExpectedUnitVariant);
                    }
                    Ok(Self::Nope)
                }
                _ => Err(UnpackError::UnknownVariant),
            }
        }
    }

    proptest! {
        #[test]
        fn roundtrip(data: Foo) {
            let mut writer: Vec<_> = Vec::new();
            data.pack(&mut writer).unwrap();
            assert_eq!(data, Foo::unpack(&mut &writer[..]).unwrap());
        }
    }
}
