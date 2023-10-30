use std::collections::{BTreeMap, HashMap};

use msgpck::{MsgPck, UnMsgPck};
use proptest::prelude::*;
use proptest_derive::Arbitrary;

#[derive(Arbitrary, Debug, Clone, PartialEq, MsgPck, UnMsgPck)]
struct Foo {
    a: bool,
    b: String,
}

#[derive(Arbitrary, Debug, PartialEq, MsgPck, UnMsgPck)]
struct Bar {
    a: f32,
    foo: Foo,
    collection: Foo,
}

#[derive(Arbitrary, Debug, PartialEq, MsgPck, UnMsgPck)]
struct Baz {
    a: f32,
    foo: Foo,
    hashmap: HashMap<String, Vec<Foo>>,
    btreemap: BTreeMap<u16, Foo>,
    list: Vec<Foo>,
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

        // use pretty_hex::*;
        // dbg!(writer.len(), &s, writer.hex_dump());

        let d = Bar::unpack(&mut &writer[..]).unwrap();
        assert_eq!(s, d);
    }

    #[test]
    fn roundtrip_baz(s: Baz) {
        let mut writer: Vec<_> = Vec::new();
        s.pack(&mut writer).unwrap();

        // use pretty_hex::*;
        // dbg!(writer.len(), &s, writer.hex_dump());

        let d = Baz::unpack(&mut &writer[..]).unwrap();
        assert_eq!(s, d);
    }

    #[test]
    fn roundtrip_size(s: Foo) {
        let assumed_size = s.size_hint();
        let mut writer: Vec<_> = Vec::new();
        s.pack(&mut writer).unwrap();

        // use pretty_hex::*;
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
