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
