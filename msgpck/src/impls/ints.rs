use crate::{MsgPck, MsgWriter, PackError, SizeHint, UnMsgPck, UnpackError};
use core::mem::size_of;

mod read;
mod write;

macro_rules! impl_msgpck_for_int {
    ($($t:ty),*) => {
        $(
            impl MsgPck for $t {
                fn pack(&self, writer: &mut dyn MsgWriter) -> Result<(), PackError> {
                    write::pack_i64(writer, *self as i64)
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
                    writer: impl embedded_io_async::Write,
                ) -> Result<(), crate::PackError> {
                    write::pack_i64_async(writer, *self as i64).await?;
                    Ok(())
                }
            }

            impl<'buf> UnMsgPck<'buf> for $t {
                fn unpack(source: &mut &'buf [u8]) -> Result<Self, UnpackError>
                where
                    Self: Sized,
                {
                    read::read_int(source).map_err(|e| match e {
                        read::NumValueReadError::TypeMismatch(m) => UnpackError::WrongMarker(m),
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
                    write::pack_u64(writer, *self as u64)
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
                    writer: impl embedded_io_async::Write,
                ) -> Result<(), crate::PackError> {
                    write::pack_u64_async(writer, *self as u64).await?;
                    Ok(())
                }
            }

            impl<'buf> UnMsgPck<'buf> for $t {
                fn unpack(source: &mut &'buf [u8]) -> Result<Self, UnpackError>
                where
                    Self: Sized,
                {
                    read::read_int(source).map_err(|e| match e {
                        read::NumValueReadError::TypeMismatch(m) => UnpackError::WrongMarker(m),
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
