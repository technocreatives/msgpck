#![feature(impl_trait_in_assoc_type)]

use embedded_io_async::Write;
use msgpackers::{pack_enum_header, EnumHeader, MsgPack, MsgUnpack, Piece, UnpackErr, Variant};

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

#[derive(Debug, MsgUnpack)]
pub enum Baz {
    Bill,
    Bob(u32),
    Bung { field1: Bar, field2: u32 },
}

impl MsgPack for Baz {
    type Iter<'a> = impl Iterator<Item = Piece<'a>>
    where
        Self: 'a;

    fn pack(&self) -> Self::Iter<'_> {
        let header = match self {
            Self::Bill => EnumHeader {
                //variant: Variant::Discriminator(0),
                variant: Variant::Name("Bill"),
                unit: true,
            },
            Self::Bob(..) => EnumHeader {
                //variant: Variant::Discriminator(1),
                variant: Variant::Name("Bob"),
                unit: false,
            },
            Self::Bung { .. } => EnumHeader {
                //variant: Variant::Discriminator(2),
                variant: Variant::Name("Bung"),
                unit: false,
            },
        };

        match self {
            //Self::Bill => pack_enum_header(header),
            Self::Bob(n) => pack_enum_header(header).chain(n.pack()),
            _ => unimplemented!(),
        }
    }
}

fn main() {
    let foo = Baz::Bob(0xffff);

    println!("{foo:x?}");

    let bytes = msgpack_to_vec(&foo);
    println!("{bytes:x?}");

    let decoded: Baz = msgpack_from_bytes(&bytes).unwrap();
    println!("{decoded:x?}");
}
