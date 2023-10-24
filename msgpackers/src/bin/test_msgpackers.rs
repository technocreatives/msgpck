#![feature(impl_trait_in_assoc_type)]

use embedded_io_async::Write;
use msgpackers::{MsgPack, MsgUnpack, Piece, UnpackErr};

/// Serialize a [MsgPack] type to a `Vec<u8>`.
fn msgpack_to_vec(m: &impl MsgPack) -> Vec<u8> {
    let mut out = vec![];
    for m in m.pack() {
        out.extend_from_slice(m.as_bytes());
    }
    out
}

/// Deserialize a [MsgPack] type from bytes
fn msgpack_from_bytes<'a, T: MsgUnpack<'a>>(mut bytes: &'a [u8]) -> Result<T, UnpackErr> {
    let value = T::unpack(&mut bytes)?;
    if !bytes.is_empty() {
        return Err(UnpackErr::TrailingBytes(bytes.len()));
    }
    Ok(value)
}

// PoC of an async serializer
async fn _msgpack_to_async_write<W: Write>(m: &impl MsgPack, w: &mut W) -> Result<(), W::Error> {
    for m in m.pack() {
        w.write_all(m.as_bytes()).await?;
    }
    Ok(())
}

#[derive(Debug, MsgPack, MsgUnpack)]
pub struct Foo {
    pub bar: Bar,
}

#[derive(Debug, MsgPack, MsgUnpack)]
pub struct Bar {
    pub a: u8,
    pub b: u16,
}

#[derive(Debug, MsgPack, MsgUnpack)]
pub enum Baz {
    Bill,
    Bob(u32),
    Bung { field1: Foo, field2: u32 },
}

fn main() {
    let foo = Baz::Bob(1234567);
    //let foo = Baz::Bung {
    //    field1: Foo {
    //        bar: Bar { a: 0x5, b: 0xff },
    //    },
    //    field2: 0x1234,
    //};

    println!("{foo:x?}");

    let bytes = msgpack_to_vec(&foo);
    println!("{bytes:x?}");

    let decoded: Baz = msgpack_from_bytes(&bytes).unwrap();
    println!("{decoded:x?}");
}
