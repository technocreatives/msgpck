use msgpck::{MsgPck, UnMsgPck};
use proptest::prelude::*;
use proptest_derive::Arbitrary;

#[derive(Arbitrary, Debug, PartialEq, MsgPck, UnMsgPck)]
enum Foo {
    A,
    B(String, String),
    C { x: f32, y: f32 },
    D { a: String, b: Bar },
}

#[derive(Arbitrary, Debug, PartialEq, MsgPck, UnMsgPck)]
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
}
