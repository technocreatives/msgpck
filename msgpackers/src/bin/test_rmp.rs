use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Foo {
    pub bar: Bar,
}

#[derive(Debug, Serialize)]
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

    let bytes = rmp_serde::to_vec(&foo);
    println!("{bytes:x?}");
}
