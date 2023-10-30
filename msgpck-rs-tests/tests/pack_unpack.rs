use msgpck_rs_tests::*;
use quickcheck_macros::quickcheck;
use std::collections::HashMap;

#[quickcheck]
fn signed_integers(i: i64) {
    test_int::<i64>(i);
    test_int::<i32>(i);
    test_int::<i16>(i);
    test_int::<i8>(i);
}

#[quickcheck]
fn unsigned_integers(i: u64) {
    test_uint::<u64>(i);
    test_uint::<u32>(i);
    test_uint::<u16>(i);
    test_uint::<u8>(i);
}

#[quickcheck]
fn f32s(f: f32) {
    if f.is_nan() {
        return; // i'm not dealing with this...
    }
    test_pack_unpack(&f);
}

#[quickcheck]
fn f64s(f: f32) {
    if f.is_nan() {
        return; // i'm not dealing with this...
    }
    test_pack_unpack(&f);
}

#[test]
fn bools() {
    test_pack_unpack(&true);
    test_pack_unpack(&false);
}

#[test]
fn unit_enum() {
    test_pack_unpack(&Baz::Bill);
}

#[test]
fn simple_enum() {
    test_pack_unpack(&Baz::Bob(1));
    test_pack_unpack(&Baz::Bob(0xff));
    test_pack_unpack(&Baz::Bob(0xabcd));
    test_pack_unpack(&Baz::Bob(u32::MAX));
}

#[test]
fn complex_enum() {
    test_pack_unpack(&Baz::Bung {
        field1: Foo {
            bar: Bar {
                a: 0xee,
                b: Fizz(3),
                c: vec![0xa, 0xb, 0xc],
                d: Fuzz,
                e: Fgblrp {
                    t: vec![],
                    y: -1234567890,
                },
            },
        },
        field2: u32::MAX,
    });
}

#[quickcheck]
fn hashmap(v: HashMap<u32, Box<i64>>) {
    test_pack_unpack(&v);
}

#[quickcheck]
fn string(v: String) {
    test_pack_unpack(&v);
}

#[quickcheck]
fn option(v: Option<i64>) {
    test_pack_unpack(&v);
}

// #[quickcheck]
// fn stacked_options(v: Option<Option<i64>>) {
//     test_pack_unpack(&v);
// }

// TODO
/*
fn struct_with_lifetime() {
    test_pack_unpack(&WithLifetime { s: "Hello there." });
}
*/
