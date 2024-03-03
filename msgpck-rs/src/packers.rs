//! Top-level functions for packing/unpacking types which impl [MsgPack]/[MsgUnpack].

use crate::{BufferOverflow, MsgPack, MsgUnpack, UnpackErr};

/// Pack a [MsgPack] type into a `Vec<u8>`.
#[cfg(feature = "alloc")]
pub fn pack_vec(m: &impl MsgPack) -> alloc::vec::Vec<u8> {
    let mut out = alloc::vec![];
    let _ = m.pack_with_writer(&mut out); // writing to a Vec can't fail
    out
}

#[cfg(feature = "std")]
pub fn pack_write(w: &mut dyn std::io::Write, m: &impl MsgPack) -> std::io::Result<usize> {
    struct W<'a, 'b>(&'a mut dyn std::io::Write, &'b mut std::io::Result<()>);

    impl crate::Write for W<'_, '_> {
        fn write_all(&mut self, bytes: &[u8]) -> Result<(), BufferOverflow> {
            *self.1 = std::io::Write::write_all(self.0, bytes);
            Ok(())
        }
    }

    let mut result = Ok(());
    let mut w = W(w, &mut result);
    let Ok(n) = m.pack_with_writer(&mut w) else {
        unreachable!()
    };
    result.map(|()| n)
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
