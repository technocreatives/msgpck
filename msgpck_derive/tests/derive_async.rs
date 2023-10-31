#![feature(async_fn_in_trait)]

use std::collections::{BTreeMap, HashMap};

use msgpck::{AsyncMsgPck, MsgPck, UnMsgPck};
use proptest::prelude::*;
use proptest_derive::Arbitrary;

#[derive(Arbitrary, Debug, Clone, PartialEq, AsyncMsgPck, MsgPck, UnMsgPck)]
struct Foo {
    a: bool,
    b: String,
}

#[derive(Arbitrary, Debug, PartialEq, AsyncMsgPck, MsgPck, UnMsgPck)]
struct Bar {
    a: f32,
    foo: Foo,
    collection: HashMap<u32, Foo>,
}

#[test]
fn roundtrip_foo_async() {
    let s = Foo {
        a: true,
        b: "hello".to_owned(),
    };
    let mut writer: Vec<_> = Vec::new();

    smol::block_on(async { s.pack_async(&mut writer).await }).unwrap();

    let d = Foo::unpack(&mut &writer[..]).unwrap();
    assert_eq!(s, d);
}

proptest! {
    #[test]
    fn roundtrip_bar_async(s: Bar) {
        let mut writer: Vec<_> = Vec::new();

        smol::block_on(async {
            s.pack_async(&mut writer).await
        }).unwrap();
        let d = Bar::unpack(&mut &writer[..]).unwrap();
        assert_eq!(s, d);
    }
}
#[test]
fn roundtrip_bar_async_fail() {
    let s = Bar {
        a: 0.0,
        foo: Foo {
            a: false,
            b: "".to_string(),
        },
        collection: HashMap::from([(
            0,
            Foo {
                a: false,
                b: "".to_string(),
            },
        )]),
    };
    let mut writer: Vec<_> = Vec::new();

    smol::block_on(async { s.pack_async(&mut writer).await }).unwrap();
    let d = Bar::unpack(&mut &writer[..]).unwrap();
    assert_eq!(s, d);
}
