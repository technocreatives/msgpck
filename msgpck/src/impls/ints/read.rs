use crate::{marker::Marker, utils::slice_take};
use num_traits::FromPrimitive;

#[cfg_attr(feature = "reduce-size", inline(never))]
pub fn read_int<T: FromPrimitive>(source: &mut &[u8]) -> Result<T, NumValueReadError> {
    use NumValueReadError::*;

    let &[b] = slice_take(source).map_err(|_| InvalidMarker)?;
    let marker = Marker::from_u8(b);
    let val = match marker {
        Marker::FixPos(val) => T::from_u8(val),
        Marker::FixNeg(val) => T::from_i8(val),
        Marker::U8 => {
            let data = slice_take(source).map_err(|_| InvalidData)?;
            let data = u8::from_be_bytes(*data);
            T::from_u8(data)
        }
        Marker::U16 => {
            let data = slice_take(source).map_err(|_| InvalidData)?;
            let data = u16::from_be_bytes(*data);
            T::from_u16(data)
        }
        Marker::U32 => {
            let data = slice_take(source).map_err(|_| InvalidData)?;
            let data = u32::from_be_bytes(*data);
            T::from_u32(data)
        }
        Marker::U64 => {
            let data = slice_take(source).map_err(|_| InvalidData)?;
            let data = u64::from_be_bytes(*data);
            T::from_u64(data)
        }
        Marker::I8 => {
            let data = slice_take(source).map_err(|_| InvalidData)?;
            let data = i8::from_be_bytes(*data);
            T::from_i8(data)
        }
        Marker::I16 => {
            let data = slice_take(source).map_err(|_| InvalidData)?;
            let data = i16::from_be_bytes(*data);
            T::from_i16(data)
        }
        Marker::I32 => {
            let data = slice_take(source).map_err(|_| InvalidData)?;
            let data = i32::from_be_bytes(*data);
            T::from_i32(data)
        }
        Marker::I64 => {
            let data = slice_take(source).map_err(|_| InvalidData)?;
            let data = i64::from_be_bytes(*data);
            T::from_i64(data)
        }
        marker => return Err(NumValueReadError::TypeMismatch(marker)),
    };

    val.ok_or(NumValueReadError::OutOfRange)
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum NumValueReadError {
    InvalidMarker,
    InvalidData,
    TypeMismatch(Marker),
    OutOfRange,
}
