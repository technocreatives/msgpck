// Vec<T>
// Box<T>

use crate::MsgPck;

impl<T: MsgPck> MsgPck for Vec<T> {
    fn pack(&self, writer: &mut dyn crate::MsgWriter) -> Result<(), crate::PackError> {
        self.as_slice().pack(writer)
    }

    fn size_hint(&self) -> (Option<usize>, Option<usize>) {
        self.as_slice().size_hint()
    }
}
