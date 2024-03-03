//! Top-level functions for packing/unpacking types which impl [MsgPack]/[MsgUnpack].

use crate::{MsgPack, MsgUnpack, PackErr, UnpackErr};

#[cfg(feature = "alloc")]
use alloc::{vec, vec::Vec};

/// Pack a [MsgPack] type into a `Vec<u8>`.
#[cfg(feature = "alloc")]
pub fn pack_vec<T: MsgPack>(value: &T) -> Result<Vec<u8>, PackErr> {
    let mut out = vec![];
    value.pack_with_writer(&mut out)?;
    Ok(out)
}

/// Pack a [MsgPack] type into a `std::io::Write`.
#[cfg(feature = "std")]
pub fn pack_write<T: MsgPack>(w: &mut dyn std::io::Write, value: &T) -> Result<usize, PackErr> {
    let mut w = crate::write::IoWrite(w);
    value.pack_with_writer(&mut w)
}

/// Pack a [MsgPack] type into a `[u8]`.
///
/// # Returns
/// If the slice was too small, this returns [PackErr::BufferOverflow].
/// Otherwise returns the number of bytes written.
pub fn pack_slice<T: MsgPack>(mut buf: &mut [u8], value: &T) -> Result<usize, PackErr> {
    value.pack_with_writer(&mut buf)
}

/// Unpack a [MsgUnpack] type from a byte slice.
pub fn unpack_slice<'a, T: MsgUnpack<'a>>(mut bytes: &'a [u8]) -> Result<T, UnpackErr> {
    let value = T::unpack(&mut bytes)?;
    if !bytes.is_empty() {
        return Err(UnpackErr::TrailingBytes(bytes.len()));
    }
    Ok(value)
}
