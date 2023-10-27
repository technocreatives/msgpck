pub trait MsgWriter {
    fn write(&mut self, data: &[u8]) -> Result<(), WriteError>;
    fn flush(&mut self) -> Result<(), WriteError> {
        Ok(())
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub enum WriteError {}

#[cfg(feature = "alloc")]
impl MsgWriter for Vec<u8> {
    fn write(&mut self, data: &[u8]) -> Result<(), WriteError> {
        self.extend_from_slice(data);
        Ok(())
    }
}
