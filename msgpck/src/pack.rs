use crate::writers::{MsgWriter, WriteError};

pub trait MsgPck {
    fn pack(&self, writer: &mut dyn MsgWriter) -> Result<(), PackError>;

    /// How big will the message be when packed?
    ///
    /// # Returns
    /// Tuple of `(min, max)`
    fn size_hint(&self) -> (Option<usize>, Option<usize>) {
        (None, None)
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub enum PackError {
    WriteError(WriteError),
}

impl From<WriteError> for PackError {
    fn from(e: WriteError) -> Self {
        Self::WriteError(e)
    }
}
