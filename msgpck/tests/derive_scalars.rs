use msgpck::{MsgPck, UnMsgPck};
use proptest::prelude::*;
use proptest_derive::Arbitrary;

#[derive(Arbitrary, Debug, PartialEq, MsgPck, UnMsgPck)]
struct Scalars {
    a: bool,
    b: i8,
    c: String,
    d: f64,
}

proptest! {
    #[test]
    fn roundtrip(s: Scalars) {
        let mut writer: Vec<_> = Vec::new();
        s.pack(&mut writer).unwrap();
        let d = Scalars::unpack(&mut &writer[..]).unwrap();
        assert_eq!(s, d);
    }
}
