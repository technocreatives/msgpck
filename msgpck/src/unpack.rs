pub(crate) mod errors;
pub(crate) mod helpers;

use errors::UnpackError;

pub trait UnMsgPck<'buf> {
    fn unpack(source: &mut &'buf [u8]) -> Result<Self, UnpackError>
    where
        Self: Sized;
}
