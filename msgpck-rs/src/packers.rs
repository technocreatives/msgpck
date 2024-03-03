//! Top-level functions for packing/unpacking types which impl [MsgPack]/[MsgUnpack].

use crate::{MsgPack, MsgUnpack, PackErr, UnpackErr};

#[cfg(feature = "alloc")]
use alloc::{vec, vec::Vec};

/// Pack a [MsgPack] type into a `Vec<u8>`.
#[cfg(feature = "alloc")]
pub fn pack_vec(m: &impl MsgPack) -> Result<Vec<u8>, PackErr> {
    let mut out = vec![];
    m.pack_with_writer(&mut out)?;
    Ok(out)
}

/// Pack a [MsgPack] type into a `std::io::Write`.
#[cfg(feature = "std")]
pub fn pack_write(w: &mut dyn std::io::Write, m: &impl MsgPack) -> Result<usize, PackErr> {
    // impl msgpck_rs::Write for this struct that wraps a std::io::Write
    struct W<'a>(&'a mut dyn std::io::Write);

    impl crate::Write for W<'_> {
        fn write_all(&mut self, bytes: &[u8]) -> Result<(), PackErr> {
            std::io::Write::write_all(self.0, bytes)?;
            Ok(())
        }
    }

    let mut w = W(w);
    m.pack_with_writer(&mut w)
}

/// Pack a [MsgPack] type into a `[u8]`.
///
/// # Returns
/// If the slice was too small, this returns [BufferOverflow].
/// Otherwise returns the number of bytes written.
pub fn pack_slice(mut buf: &mut [u8], m: &impl MsgPack) -> Result<usize, PackErr> {
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
