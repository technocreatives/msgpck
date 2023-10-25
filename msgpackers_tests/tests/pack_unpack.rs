use msgpackers_tests::*;

/// Test pack/unpack for integers of various size
#[test]
fn integers() {
    let integers: &[u64] = &[
        1,
        2,
        3,
        0x7f,
        0xef,
        0xff,
        0xff,
        0x100,
        0xabcd,
        0xabcdef,
        0xabcdeffe,
        0xffffffff,
        0xffffffffffffffff,
    ];

    for &int in integers {
        test_int_size::<u64>(int);
        test_int_size::<u32>(int);
        test_int_size::<u16>(int);
        test_int_size::<u8>(int);
    }
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
            },
        },
        field2: u32::MAX,
    });
}
