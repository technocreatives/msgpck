#![feature(async_fn_in_trait)]

use msgpck::{AsyncMsgPck, MsgPck, UnMsgPck};
use proptest::prelude::*;
use proptest_derive::Arbitrary;

#[derive(Arbitrary, Debug, PartialEq, MsgPck, UnMsgPck, AsyncMsgPck)]
enum Foo {
    A,
    B(String, String),
    C { x: f32, y: f32 },
    D { a: String, b: Bar },
}

#[derive(Arbitrary, Debug, PartialEq, MsgPck, UnMsgPck, AsyncMsgPck)]
struct Bar {
    a: f32,
    b: String,
}

proptest! {
    #[test]
    fn roundtrip(s: Foo) {
        let mut writer: Vec<_> = Vec::new();
        s.pack(&mut writer).unwrap();
        let d = Foo::unpack(&mut &writer[..]).unwrap();
        assert_eq!(s, d);
    }

    #[test]
    fn roundtrip_async(s: Foo) {
        let mut writer: Vec<_> = Vec::new();
        smol::block_on(async { s.pack_async(&mut writer).await }).unwrap();
        let d = Foo::unpack(&mut &writer[..]).unwrap();
        assert_eq!(s, d);
    }
}
