use crate::{MsgPck, SizeHint};

impl<'r, T> MsgPck for &'r T
where
    T: MsgPck,
{
    fn pack(&self, writer: &mut dyn crate::MsgWriter) -> Result<(), crate::PackError> {
        (**self).pack(writer)
    }

    fn size_hint(&self) -> SizeHint {
        (**self).size_hint()
    }
}

#[cfg(feature = "async")]
impl<'r, T: crate::AsyncMsgPck> crate::AsyncMsgPck for &'r T {
    async fn pack_async(
        &self,
        writer: impl embedded_io_async::Write,
    ) -> Result<(), crate::PackError> {
        (**self).pack_async(writer).await?;
        Ok(())
    }
}

impl<'a, T> MsgPck for &'a mut T
where
    T: MsgPck,
{
    fn pack(&self, writer: &mut dyn crate::MsgWriter) -> Result<(), crate::PackError> {
        (**self).pack(writer)
    }

    fn size_hint(&self) -> SizeHint {
        (**self).size_hint()
    }
}
