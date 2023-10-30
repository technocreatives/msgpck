use core::hash::Hash;
use std::collections::{BTreeMap, HashMap};

use crate::{
    utils::{pack_map_header, unpack_map_header},
    MsgPck, MsgWriter, PackError, UnMsgPck, UnpackError,
};

impl<K, V> MsgPck for BTreeMap<K, V>
where
    K: MsgPck,
    V: MsgPck,
{
    #[inline(never)]
    fn pack(&self, writer: &mut dyn MsgWriter) -> Result<(), PackError> {
        pack_map_header(writer, self.len())?;
        for (k, v) in self.iter() {
            k.pack(writer)?;
            v.pack(writer)?;
        }
        Ok(())
    }
}

impl<'buf, K, V> UnMsgPck<'buf> for BTreeMap<K, V>
where
    K: UnMsgPck<'buf> + Ord,
    V: UnMsgPck<'buf>,
{
    #[inline(never)]
    fn unpack(source: &mut &'buf [u8]) -> Result<Self, UnpackError> {
        let len = unpack_map_header(source)?;

        // sanity check: make sure buffer has enough data for this map
        if source.len() < len {
            return Err(UnpackError::UnexpectedEof);
        }

        (0..len)
            .map(move |_| {
                let k = K::unpack(source)?;
                let v = V::unpack(source)?;
                Ok((k, v))
            })
            .collect()
    }
}

impl<K, V> MsgPck for HashMap<K, V>
where
    K: MsgPck,
    V: MsgPck,
{
    #[inline(never)]
    fn pack(&self, writer: &mut dyn MsgWriter) -> Result<(), PackError> {
        pack_map_header(writer, self.len())?;
        for (k, v) in self.iter() {
            k.pack(writer)?;
            v.pack(writer)?;
        }
        Ok(())
    }
}

impl<'buf, K, V> UnMsgPck<'buf> for HashMap<K, V>
where
    K: UnMsgPck<'buf> + Hash + Eq,
    V: UnMsgPck<'buf>,
{
    #[inline(never)]
    fn unpack(source: &mut &'buf [u8]) -> Result<Self, UnpackError> {
        let len = unpack_map_header(source)?;

        // sanity check: make sure buffer has enough data for this map
        let bytes_in_buffer = source.len();
        if bytes_in_buffer < len {
            return Err(UnpackError::UnexpectedEof);
        }

        (0..len)
            .map(move |_| {
                let k = K::unpack(source)?;
                let v = V::unpack(source)?;
                Ok((k, v))
            })
            .collect()
    }
}
