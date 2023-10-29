#![no_std]

use heapless07::String;
use msgpck::{pack_slice, MsgPck, UnMsgPck};

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

pub fn test() -> u32 {
    let bar = Bar {
        a: 1.0,
        foo: Foo {
            a: true,
            b: String::from("hello"),
        },
    };
    let mut buffer = [0u8; 128];
    if let Err(e) = pack_slice(&mut buffer, &bar) {
        return 1;
    }
    0
}
