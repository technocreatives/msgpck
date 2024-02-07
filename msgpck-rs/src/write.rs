use core::mem;

use crate::BufferOverflow;

pub trait Write {
    fn write_all(&mut self, bytes: &[u8]) -> Result<(), BufferOverflow>;
}

impl Write for &mut [u8] {
    fn write_all(&mut self, bytes: &[u8]) -> Result<(), BufferOverflow> {
        let n = bytes.len();
        if n > self.len() {
            return Err(BufferOverflow);
        }

        let (a, b) = mem::take(self).split_at_mut(n);
        a.copy_from_slice(bytes);
        *self = b;

        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl Write for alloc::vec::Vec<u8> {
    fn write_all(&mut self, bytes: &[u8]) -> Result<(), BufferOverflow> {
        self.extend_from_slice(bytes);
        Ok(())
    }
}
