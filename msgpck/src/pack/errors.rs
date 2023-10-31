use crate::WriteError;

#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum PackError {
    #[cfg_attr(feature = "std", error("Write error"))]
    WriteError { source: WriteError },
}

impl From<WriteError> for PackError {
    fn from(source: WriteError) -> Self {
        Self::WriteError { source }
    }
}
