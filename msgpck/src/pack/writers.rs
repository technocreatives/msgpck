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

#[cfg(feature = "alloc")]
impl MsgWriter for Vec<u8> {
    fn write(&mut self, data: &[u8]) -> Result<(), WriteError> {
        self.extend_from_slice(data);
        Ok(())
    }
}
