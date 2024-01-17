//! Top-level functions for packing/unpacking types which impl [MsgPack]/[MsgUnpack].

use crate::{BufferOverflow, MsgPack, MsgUnpack, UnpackErr};

/// Pack a [MsgPack] type into a `Vec<u8>`.
#[cfg(feature = "alloc")]
pub fn pack_vec(m: &impl MsgPack) -> alloc::vec::Vec<u8> {
    let mut out = alloc::vec![];
    for p in m.pack() {
        out.extend_from_slice(p.as_bytes());
    }
    out
}

/// Pack a [MsgPack] type into a `[u8]`.
///
/// # Returns
/// If the slice was too small, this returns [BufferOverflow].
/// Otherwise returns the number of bytes written.
pub fn pack_slice(mut buf: &mut [u8], m: &impl MsgPack) -> Result<usize, BufferOverflow> {
    m.pack_with_writer(&mut buf)
}

/// Unpack a [MsgUnpack] type from a byte slice.
pub fn unpack_slice<'a, T: MsgUnpack<'a>>(mut bytes: &'a [u8]) -> Result<T, UnpackErr> {
    let value = T::unpack(&mut bytes)?;
    if !bytes.is_empty() {
        return Err(UnpackErr::TrailingBytes(bytes.len()));
    }
    Ok(value)
}
