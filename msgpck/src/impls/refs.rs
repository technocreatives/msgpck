use crate::MsgPck;

impl<'r, T> MsgPck for &'r T
where
    T: MsgPck,
{
    fn pack(&self, writer: &mut dyn crate::MsgWriter) -> Result<(), crate::PackError> {
        (**self).pack(writer)
    }
}

impl<'a, T> MsgPck for &'a mut T
where
    T: MsgPck,
{
    fn pack(&self, writer: &mut dyn crate::MsgWriter) -> Result<(), crate::PackError> {
        (**self).pack(writer)
    }
}
