use crate::{marker::Marker, PackError};

#[cfg_attr(feature = "reduce-size", inline(never))]
pub(crate) fn pack_u64(writer: &mut dyn crate::MsgWriter, n: u64) -> Result<(), PackError> {
    match n {
        0x0..=0x7f => {
            writer.write(&[Marker::FixPos(n as u8).to_u8()])?;
        }
        0x80..=0xff => {
            writer.write(&[Marker::U8.to_u8(), n as u8])?;
        }
        0x100..=0xffff => {
            writer.write(&[Marker::U16.to_u8()])?;
            writer.write(&(n as u16).to_be_bytes())?;
        }
        0x10000..=0xffffffff => {
            writer.write(&[Marker::U32.to_u8()])?;
            writer.write(&(n as u32).to_be_bytes())?;
        }
        _ => {
            writer.write(&[Marker::U64.to_u8()])?;
            writer.write(&n.to_be_bytes())?;
        }
    }
    Ok(())
}

#[cfg_attr(feature = "reduce-size", inline(never))]
pub(crate) fn pack_i64(writer: &mut dyn crate::MsgWriter, n: i64) -> Result<(), PackError> {
    // Pack i into the smallest msgpack type that will fit it.
    match n {
        ..=-2147483649 => {
            writer.write(&[Marker::I64.to_u8()])?;
            writer.write(&n.to_be_bytes())?;
        }
        ..=-32769 => {
            writer.write(&[Marker::I32.to_u8()])?;
            writer.write(&(n as i32).to_be_bytes())?;
        }
        ..=-129 => {
            writer.write(&[Marker::I16.to_u8()])?;
            writer.write(&(n as i16).to_be_bytes())?;
        }
        ..=-33 => {
            writer.write(&[Marker::I8.to_u8()])?;
            writer.write(&(n as u8).to_be_bytes())?;
        }
        ..=-1 => {
            writer.write(&[Marker::FixNeg(n as i8).to_u8()])?;
        }
        // If the value is positive, pack as an unsigned integer.
        _ => return pack_u64(writer, n as u64),
    }
    Ok(())
}

#[cfg(feature = "async")]
#[cfg_attr(feature = "reduce-size", inline(never))]
pub(crate) async fn pack_u64_async<'a>(
    mut writer: impl embedded_io_async::Write,
    n: u64,
) -> Result<(), PackError> {
    match n {
        0x0..=0x7f => {
            writer.write_all(&[Marker::FixPos(n as u8).to_u8()]).await?;
        }
        0x80..=0xff => {
            writer.write_all(&[Marker::U8.to_u8(), n as u8]).await?;
        }
        0x100..=0xffff => {
            writer.write_all(&[Marker::U16.to_u8()]).await?;
            writer.write_all(&(n as u16).to_be_bytes()).await?;
        }
        0x10_00_00..=0xffffffff => {
            writer.write_all(&[Marker::U32.to_u8()]).await?;
            writer.write_all(&(n as u32).to_be_bytes()).await?;
        }
        _ => {
            writer.write_all(&[Marker::U64.to_u8()]).await?;
            writer.write_all(&n.to_be_bytes()).await?;
        }
    }
    Ok(())
}

#[cfg(feature = "async")]
#[cfg_attr(feature = "reduce-size", inline(never))]
pub(crate) async fn pack_i64_async<'a>(
    mut writer: impl embedded_io_async::Write,
    n: i64,
) -> Result<(), PackError> {
    // Pack n into the smallest msgpack type that will fit it.
    match n {
        ..=-2147483649 => {
            writer.write_all(&[Marker::I64.to_u8()]).await?;
            writer.write_all(&n.to_be_bytes()).await?;
        }
        ..=-32769 => {
            writer.write_all(&[Marker::I32.to_u8()]).await?;
            writer.write_all(&(n as i32).to_be_bytes()).await?;
        }
        ..=-129 => {
            writer.write_all(&[Marker::I16.to_u8()]).await?;
            writer.write_all(&(n as i16).to_be_bytes()).await?;
        }
        ..=-33 => {
            writer.write_all(&[Marker::I8.to_u8()]).await?;
            writer.write_all(&(n as u8).to_be_bytes()).await?;
        }
        ..=-1 => {
            writer.write_all(&[Marker::FixNeg(n as i8).to_u8()]).await?;
        }
        // If the value is positive, pack as an unsigned integer.
        _ => {
            pack_u64_async(writer, n as u64).await?;
        }
    }
    Ok(())
}
