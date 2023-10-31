use crate::{MsgPck, MsgWriter, PackError, SizeHint, UnMsgPck, UnpackError};
use core::mem::size_of;

use self::helpers::read_int;

macro_rules! impl_msgpck_for_int {
    ($($t:ty),*) => {
        $(
            impl MsgPck for $t {
                fn pack(&self, writer: &mut dyn MsgWriter) -> Result<(), PackError> {
                    Ok(helpers::dummy_write_i64(writer, *self as i64)?)
                }

                fn size_hint(&self) -> SizeHint {
                    SizeHint {
                        min: Some(size_of::<Self>()),
                        max: Some(size_of::<Self>() + 1),
                    }
                }
            }

            #[cfg(feature = "async")]
            impl crate::AsyncMsgPck for $t {
                async fn pack_async(
                    &self,
                    mut writer: impl embedded_io_async::Write,
                ) -> Result<(), crate::PackError> {
                    // TODO: Use real small numbers
                    writer.write_all(&[crate::Marker::I64.to_u8()]).await?;
                    writer.write_all(&(*self as i64).to_be_bytes()).await?;
                    Ok(())
                }
            }

            impl<'buf> UnMsgPck<'buf> for $t {
                fn unpack(source: &mut &'buf [u8]) -> Result<Self, UnpackError>
                where
                    Self: Sized,
                {
                    read_int(source).map_err(|e| match e {
                        helpers::NumValueReadError::TypeMismatch(m) => UnpackError::WrongMarker(m),
                        _ => UnpackError::UnexpectedEof,
                    })
                }
            }

            #[cfg(test)]
            paste::paste! {
                mod [<test_ $t>] {
                    use super::*;
                    use proptest::prelude::*;

                    proptest! {
                        #[test]
                        fn test(s: $t) {
                            let mut writer: Vec<_> = Vec::new();
                            s.pack(&mut writer).unwrap();
                            let d = <$t>::unpack(&mut &writer[..]).unwrap();
                            assert_eq!(s, d);
                        }
                    }
                }
            }
        )*
    };
    () => {

    };
}

macro_rules! impl_msgpck_for_uint {
    ($($t:ty),*) => {
        $(
            impl MsgPck for $t {
                fn pack(&self, writer: &mut dyn MsgWriter) -> Result<(), PackError> {
                    Ok(helpers::dummy_write_u64(writer, *self as u64)?)
                }

                fn size_hint(&self) -> SizeHint {
                    SizeHint {
                        min: Some(size_of::<Self>()),
                        max: Some(size_of::<Self>() + 1),
                    }
                }
            }

            #[cfg(feature = "async")]
            impl crate::AsyncMsgPck for $t {
                async fn pack_async(
                    &self,
                    mut writer: impl embedded_io_async::Write,
                ) -> Result<(), crate::PackError> {
                    // TODO: Use real small numbers
                    writer.write_all(&[crate::Marker::U64.to_u8()]).await?;
                    writer.write_all(&(*self as u64).to_be_bytes()).await?;
                    Ok(())
                }
            }

            impl<'buf> UnMsgPck<'buf> for $t {
                fn unpack(source: &mut &'buf [u8]) -> Result<Self, UnpackError>
                where
                    Self: Sized,
                {
                    read_int(source).map_err(|e| match e {
                        helpers::NumValueReadError::TypeMismatch(m) => UnpackError::WrongMarker(m),
                        _ => UnpackError::UnexpectedEof,
                    })
                }
            }

            #[cfg(test)]
            paste::paste! {
                mod [<test_ $t>] {
                    use super::*;
                    use proptest::prelude::*;

                    proptest! {
                        #[test]
                        fn test(s: $t) {
                            let mut writer: Vec<_> = Vec::new();
                            s.pack(&mut writer).unwrap();
                            let d = <$t>::unpack(&mut &writer[..]).unwrap();
                            assert_eq!(s, d);
                        }
                    }
                }
            }
        )*
    };
    () => {

    };
}

impl_msgpck_for_int!(i8, i16, i32, i64, isize);
impl_msgpck_for_uint!(u8, u16, u32, u64, usize);

mod helpers {
    use num_traits::FromPrimitive;

    use crate::{marker::Marker, utils::slice_take, MsgWriter, WriteError};

    // TODO: Use real small numbers
    #[cfg_attr(feature = "reduce-size", inline(never))]
    pub fn dummy_write_i64(writer: &mut dyn MsgWriter, val: i64) -> Result<(), WriteError> {
        writer.write(&[Marker::I64.to_u8()])?;
        writer.write(&val.to_be_bytes())?;
        Ok(())
    }

    // TODO: Use real small numbers
    #[cfg_attr(feature = "reduce-size", inline(never))]
    pub fn dummy_write_u64(writer: &mut dyn MsgWriter, val: u64) -> Result<(), WriteError> {
        writer.write(&[Marker::U64.to_u8()])?;
        writer.write(&val.to_be_bytes())?;
        Ok(())
    }

    #[cfg_attr(feature = "reduce-size", inline(never))]
    pub fn read_int<T: FromPrimitive>(source: &mut &[u8]) -> Result<T, NumValueReadError> {
        use NumValueReadError::*;

        let &[b] = slice_take(source).map_err(|_| InvalidMarker)?;
        let marker = Marker::from_u8(b);
        let val = match marker {
            Marker::FixPos(val) => T::from_u8(val),
            Marker::FixNeg(val) => T::from_i8(val),
            Marker::U8 => {
                let data = slice_take(source).map_err(|_| InvalidData)?;
                let data = u8::from_be_bytes(*data);
                T::from_u8(data)
            }
            Marker::U16 => {
                let data = slice_take(source).map_err(|_| InvalidData)?;
                let data = u16::from_be_bytes(*data);
                T::from_u16(data)
            }
            Marker::U32 => {
                let data = slice_take(source).map_err(|_| InvalidData)?;
                let data = u32::from_be_bytes(*data);
                T::from_u32(data)
            }
            Marker::U64 => {
                let data = slice_take(source).map_err(|_| InvalidData)?;
                let data = u64::from_be_bytes(*data);
                T::from_u64(data)
            }
            Marker::I8 => {
                let data = slice_take(source).map_err(|_| InvalidData)?;
                let data = i8::from_be_bytes(*data);
                T::from_i8(data)
            }
            Marker::I16 => {
                let data = slice_take(source).map_err(|_| InvalidData)?;
                let data = i16::from_be_bytes(*data);
                T::from_i16(data)
            }
            Marker::I32 => {
                let data = slice_take(source).map_err(|_| InvalidData)?;
                let data = i32::from_be_bytes(*data);
                T::from_i32(data)
            }
            Marker::I64 => {
                let data = slice_take(source).map_err(|_| InvalidData)?;
                let data = i64::from_be_bytes(*data);
                T::from_i64(data)
            }
            marker => return Err(NumValueReadError::TypeMismatch(marker)),
        };

        val.ok_or(NumValueReadError::OutOfRange)
    }

    #[cfg_attr(feature = "debug", derive(Debug))]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum NumValueReadError {
        InvalidMarker,
        InvalidData,
        TypeMismatch(Marker),
        OutOfRange,
    }
}
