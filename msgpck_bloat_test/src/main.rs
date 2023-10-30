use heapless07::String;
use msgpck::{pack_slice, unpack_bytes, MsgPck, UnMsgPck};

fn main() {
    let bar = Bar {
        a: 1.0,
        foo: Foo {
            a: true,
            b: String::from("hello"),
        },
    };
    let mut buffer = [0u8; 128];
    if let Err(_e) = pack_slice(&mut buffer, &bar) {
        eprintln!("pack error");
    }
    let x: Result<Bar, _> = unpack_bytes(&buffer);
    eprintln!("{}", x.is_ok());
}

#[derive(MsgPck, UnMsgPck)]
struct Foo {
    a: bool,
    b: String<12>,
}

#[derive(MsgPck, UnMsgPck)]
struct Bar {
    a: f32,
    foo: Foo,
}
