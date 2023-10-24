use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Foo {
    pub bar: Bar,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Bar {
    pub a: u8,
    pub b: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Baz {
    Bill,
    Bob(u32),
    Bung { field1: Foo, field2: u32 },
}

fn main() {
    let foo = Baz::Bung {
        field1: Foo {
            bar: Bar { a: 0x5, b: 0xff },
        },
        field2: 0x1234,
    };

    println!("{foo:x?}");

    let bytes = rmp_serde::to_vec(&foo).unwrap();
    println!("{bytes:x?}");

    let decoded: Baz = rmp_serde::from_slice(&bytes).unwrap();
    println!("{decoded:x?}");
}
