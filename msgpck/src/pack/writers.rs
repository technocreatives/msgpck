pub trait MsgWriter {
    fn write(&mut self, data: &[u8]) -> Result<(), WriteError>;
    fn flush(&mut self) -> Result<(), WriteError> {
        Ok(())
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum WriteError {
    #[cfg_attr(feature = "std", error("Not enough space in buffer"))]
    BufferOverflow,
}

pub struct BufferWriter<'buf> {
    buf: &'buf mut [u8],
    pub(crate) pos: usize,
}

impl<'buf> BufferWriter<'buf> {
    pub fn new(buf: &'buf mut [u8]) -> Self {
        Self { buf, pos: 0 }
    }
}

#[cfg(feature = "debug")]
impl<'buf> std::fmt::Debug for BufferWriter<'buf> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BufferWriter")
            .field("buf", &format_args!("{} bytes", self.buf.len()))
            .field("pos", &self.pos)
            .finish()
    }
}

impl<'buf> MsgWriter for BufferWriter<'buf> {
    fn write(&mut self, data: &[u8]) -> Result<(), WriteError> {
        if self.pos + data.len() > self.buf.len() {
            return Err(WriteError::BufferOverflow);
        }
        self.buf[self.pos..self.pos + data.len()].copy_from_slice(data);
        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl MsgWriter for Vec<u8> {
    fn write(&mut self, data: &[u8]) -> Result<(), WriteError> {
        self.extend_from_slice(data);
        Ok(())
    }
}
