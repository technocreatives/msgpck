use crate::{MsgPck, PackError};

use super::writers::BufferWriter;

/// Pack yourself into a `Vec<u8>`.
///
/// This is a convenience method that calls `pack` on a `Vec<u8>` writer.
#[cfg(feature = "alloc")]
#[inline(never)]
pub fn pack_vec(data: impl MsgPck) -> Result<Vec<u8>, PackError> {
    let min_size = data.size_hint().min.unwrap_or(0);
    let mut writer = Vec::with_capacity(min_size);
    data.pack(&mut writer)?;
    Ok(writer)
}

/// Pack a [MsgPack] type into a `[u8]`.
///
/// # Returns
///
/// If the slice was too small, this returns [BufferOverflow].
/// Otherwise returns the number of bytes written.
pub fn pack_slice(mut buf: &mut [u8], data: &impl MsgPck) -> Result<usize, PackError> {
    let mut writer = BufferWriter::new(buf);
    data.pack(&mut writer)?;
    Ok(writer.pos)
}
