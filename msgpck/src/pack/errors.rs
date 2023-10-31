use crate::WriteError;

#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum PackError {
    #[cfg_attr(feature = "std", error("Write error"))]
    WriteError { source: WriteError },
    #[cfg(feature = "async")]
    #[cfg_attr(feature = "std", error("Async write error"))]
    AsyncWriteError {
        #[cfg(feature = "std")]
        error: String,
        kind: embedded_io_async::ErrorKind,
    },
}

impl From<WriteError> for PackError {
    fn from(source: WriteError) -> Self {
        Self::WriteError { source }
    }
}

#[cfg(feature = "async")]
impl<T: embedded_io_async::Error> From<T> for PackError {
    fn from(e: T) -> Self {
        Self::AsyncWriteError {
            #[cfg(feature = "std")]
            error: format!("{e:?}"),
            kind: e.kind(),
        }
    }
}
