pub(crate) mod errors;
pub(crate) mod helpers;
pub(crate) mod size_hint;
pub(crate) mod writers;

use errors::PackError;
use size_hint::SizeHint;
use writers::MsgWriter;

/// Trait for serializing a type using [msgpack][https://msgpack.org/].
///
/// # Usage
///
/// The recommended usage is to use the derive macro `#[derive(MsgPck)]` on your
/// type which will generate an implementation for you.
///
/// See the crate-level documentation for a custom implementation.
pub trait MsgPck {
    /// Pack yourself into a writer.
    fn pack(&self, writer: &mut dyn MsgWriter) -> Result<(), PackError>;

    /// How big will the message be when packed?
    ///
    /// # Returns
    /// Tuple of `(min, max)`
    fn size_hint(&self) -> SizeHint {
        SizeHint::default()
    }
}
