use core::mem;

use crate::error::PackErr;

pub trait Write {
    fn write_all(&mut self, bytes: &[u8]) -> Result<(), PackErr>;
}

impl Write for &mut [u8] {
    fn write_all(&mut self, bytes: &[u8]) -> Result<(), PackErr> {
        let n = bytes.len();
        if n > self.len() {
            return Err(PackErr::BufferOverflow);
        }

        let (a, b) = mem::take(self).split_at_mut(n);
        a.copy_from_slice(bytes);
        *self = b;

        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl Write for alloc::vec::Vec<u8> {
    fn write_all(&mut self, bytes: &[u8]) -> Result<(), PackErr> {
        self.try_reserve(bytes.len())
            .map_err(|_| PackErr::OutOfMemory)?;
        self.extend_from_slice(bytes);
        Ok(())
    }
}

/// Wrapper type for a [std::io::Write] that impls [Write].
#[cfg(feature = "std")]
pub struct IoWrite<W: std::io::Write>(pub W);

#[cfg(feature = "std")]
impl<W: std::io::Write> Write for IoWrite<W> {
    #[inline]
    fn write_all(&mut self, bytes: &[u8]) -> Result<(), PackErr> {
        std::io::Write::write_all(&mut self.0, bytes)?;
        Ok(())
    }
}
