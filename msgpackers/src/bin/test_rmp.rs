use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Foo {
    pub bar: Bar,
}

#[derive(Debug, Serialize, Deserialize)]
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

    let bytes = rmp_serde::to_vec(&foo).unwrap();
    println!("{bytes:x?}");

    let decoded: Foo = rmp_serde::from_slice(&bytes).unwrap();
    println!("{decoded:x?}");
}
