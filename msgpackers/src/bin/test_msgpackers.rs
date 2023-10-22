#![feature(impl_trait_in_assoc_type)]

use embedded_io_async::Write;
use msgpackers::{
    pack_enum_header, unpack_enum_header, EnumHeader, MsgPack, MsgUnpack, Piece, UnpackErr, Variant,
};

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
    pub c: Vec<u16>,
}

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
            Baz::Bill => EnumHeader {
                variant: Variant::Discriminator(0),
                unit: true,
            },
            Baz::Bob(..) => EnumHeader {
                variant: Variant::Discriminator(1),
                unit: false,
            },
            Baz::Bung { .. } => EnumHeader {
                variant: Variant::Discriminator(2),
                unit: false,
            },
        };

        pack_enum_header(header)

        // TODO: pack fields
    }
}

impl<'buf> MsgUnpack<'buf> for Baz {
    fn unpack(bytes: &mut &'buf [u8]) -> Result<Self, UnpackErr>
    where
        Self: Sized + 'buf,
    {
        let header = unpack_enum_header(bytes)?;

        use Variant::*;
        Ok(match &header.variant {
            Discriminator(0) | Name("Bill") => {
                if !header.unit {
                    return Err(todo!());
                }
                Self::Bill
            }
            Discriminator(1) | Name("Bob") => {
                if header.unit {
                    return Err(todo!());
                }
                todo!()
            }
            Discriminator(2) | Name("Bung") => {
                if header.unit {
                    return Err(todo!());
                }
                todo!("decode Bung");
            }
            // TODO include variant in error
            _variant => return Err(UnpackErr::UnknownVariant),
        })
    }
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

    let decoded: Foo = msgpack_from_bytes(&bytes).unwrap();
    println!("{decoded:x?}");
}
