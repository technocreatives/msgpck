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
