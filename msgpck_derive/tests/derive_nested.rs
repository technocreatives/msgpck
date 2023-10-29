use msgpck::{MsgPck, UnMsgPck};
use pretty_hex::*;
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

    #[test]
    fn roundtrip_size(s: Foo) {
        let assumed_size = s.size_hint();
        let mut writer: Vec<_> = Vec::new();
        s.pack(&mut writer).unwrap();

        // dbg!(writer.len(), &s, writer.hex_dump(), &assumed_size);
        match assumed_size {
            msgpck::SizeHint { min: Some(min), max: Some(max) } => {
                assert!(min <= writer.len());
                assert!(writer.len() <= max);
            },
            _ => panic!("SizeHint should be precise"),
        };
    }
}
