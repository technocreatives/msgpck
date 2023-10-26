//! Top-level functions for packing/unpacking types which impl [MsgPack]/[MsgUnpack].

use crate::{MsgPack, MsgUnpack, UnpackErr};

/// Pack a [MsgPack] type into a `Vec<u8>`.
#[cfg(feature = "alloc")]
pub fn pack_vec(m: &impl MsgPack) -> alloc::vec::Vec<u8> {
    let mut out = alloc::vec![];
    for m in m.pack() {
        out.extend_from_slice(m.as_bytes());
    }
    out
}

/// Unpack a [MsgUnpack] type from a byte slice.
pub fn unpack_bytes<'a, T: MsgUnpack<'a>>(mut bytes: &'a [u8]) -> Result<T, UnpackErr> {
    let value = T::unpack(&mut bytes)?;
    if !bytes.is_empty() {
        return Err(UnpackErr::TrailingBytes(bytes.len()));
    }
    Ok(value)
}
