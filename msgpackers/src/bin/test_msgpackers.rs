#![feature(impl_trait_in_assoc_type)]

use embedded_io_async::Write;
use msgpackers::MsgPack;

/// Serialize a [MsgPack] type to a `Vec<u8>`.
fn msgpack_to_vec(m: &impl MsgPack) -> Vec<u8> {
    let mut out = vec![];
    for m in m.encode() {
        out.extend_from_slice(m.as_bytes());
    }
    out
}

// PoC of an async serializer
async fn _msgpack_to_async_write<W: Write>(m: &impl MsgPack, w: &mut W) -> Result<(), W::Error> {
    for m in m.encode() {
        w.write_all(m.as_bytes()).await?;
    }
    Ok(())
}

#[derive(Debug, MsgPack)]
pub struct Foo {
    pub bar: Bar,
}

#[derive(Debug, MsgPack)]
pub struct Bar {
    pub a: u8,
    pub b: u16,
    pub c: Vec<u16>,
}

fn main() {
    let foo = Foo {
        bar: Bar {
            a: 0xee,
            b: 3,
            c: vec![0xa, 0xb, 0xc],
        },
    };

    println!("{foo:x?}");

    let bytes = msgpack_to_vec(&foo);
    println!("{bytes:x?}");
}
