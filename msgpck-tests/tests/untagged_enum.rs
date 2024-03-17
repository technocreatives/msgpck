use msgpck::{pack_vec, unpack_slice};
use msgpck_tests::{Bar, Fgblrp, Fizz, Foo, Fuzz, NotPack, UntaggedBaz, UntaggedBazBung};

#[test]
fn pack_unit_enum() {
    let variant = UntaggedBaz::Bill;

    // msgpack representation of a 1 byte unsigned integer
    let msgpacked = pack_vec(&variant).expect("pack to vec");

    println!("packed: {:?}", msgpacked);
    let unpacked: Option<i32> = unpack_slice(&msgpacked).expect("unpack UntaggedBaz::Bill");

    assert!(unpacked.is_none());
}

#[test]
fn pack_untagged_newtype_enum() {
    let num = 1234;
    let variant = UntaggedBaz::Bob(num);

    // msgpack representation of a 1 byte unsigned integer
    let msgpacked = pack_vec(&variant).expect("pack to vec");

    let unpacked: u32 = unpack_slice(&msgpacked).expect("unpack UntaggedBaz::Bob");

    assert_eq!(unpacked, num);
}

#[test]
fn pack_untagged_struct_enum() {
    let foo = Foo {
        bar: Bar {
            a: 123,
            b: Fizz(345),
            c: vec![9, 9, 8],
            d: Fuzz,
            e: Fgblrp {
                t: vec![],
                y: -1234,
            },
            skipped: NotPack,
        },
    };

    let variant = UntaggedBaz::Bung {
        field1: foo.clone(),
        field2: 42,
    };

    // msgpack representation of a 1 byte unsigned integer
    let msgpacked = pack_vec(&variant).expect("pack to vec");

    let unpacked: UntaggedBazBung = unpack_slice(&msgpacked).expect("unpack UntaggedBazBung");

    assert_eq!(unpacked.field1, foo);
    assert_eq!(unpacked.field2, 42);
}
