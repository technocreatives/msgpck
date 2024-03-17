use msgpck::unpack_slice;
use msgpck_tests::CStyleEnum;
use quickcheck_macros::quickcheck;

/// Test to unpack an enum using the discriminant integer value, not the name.
#[quickcheck]
fn unpack_enum_discriminant(e: CStyleEnum) {
    let discriminant = e as i8;

    // msgpack representation of a 1 byte unsigned integer
    let msgpacked = [0xd0, discriminant as u8];

    let deserialized: CStyleEnum =
        unpack_slice(&msgpacked[..]).expect("unpack enum from discriminant");
    assert_eq!(deserialized, e);
}
