use msgpck::{MsgPck, UnMsgPck};
use proptest::prelude::*;
use proptest_derive::Arbitrary;

#[derive(Arbitrary, Debug, PartialEq, MsgPck, UnMsgPck)]
struct Foo {
    a: bool,
    b: String,
}

#[derive(Arbitrary, Debug, PartialEq, MsgPck, UnMsgPck)]
struct Bar {
    a: f32,
    foo: Foo,
}

proptest! {
    #[test]
    fn roundtrip_foo(s: Foo) {
        let mut writer: Vec<_> = Vec::new();
        s.pack(&mut writer).unwrap();
        let d = Foo::unpack(&mut &writer[..]).unwrap();
        assert_eq!(s, d);
    }

    #[test]
    fn roundtrip_bar(s: Bar) {
        let mut writer: Vec<_> = Vec::new();
        s.pack(&mut writer).unwrap();
        let d = Bar::unpack(&mut &writer[..]).unwrap();
        assert_eq!(s, d);
    }
}
