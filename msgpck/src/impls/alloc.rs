// Vec<T>
// Box<T>

use crate::{pack::SizeHint, MsgPck};

impl<T: MsgPck> MsgPck for Vec<T> {
    fn pack(&self, writer: &mut dyn crate::MsgWriter) -> Result<(), crate::PackError> {
        self.as_slice().pack(writer)
    }

    fn size_hint(&self) -> SizeHint {
        self.as_slice().size_hint()
    }
}
