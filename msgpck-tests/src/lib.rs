//! A simple test that checks whether msgpck is compatible with rmp_serde.

use msgpck::{MsgPack, MsgUnpack};
use quickcheck::Arbitrary;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, MsgPack, MsgUnpack)]
pub struct Foo {
    pub bar: Bar,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, MsgPack, MsgUnpack)]
pub struct Bar {
    pub a: u8,
    pub b: Fizz,
    pub c: Vec<u16>,
    pub d: Fuzz,
    pub e: Fgblrp<Vec<i32>, i64>,

    #[serde(skip)]
    #[msgpck(skip)]
    pub skipped: NotPack,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct NotPack;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, MsgPack, MsgUnpack)]
pub struct Fizz(pub u16);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, MsgPack, MsgUnpack)]
pub struct Fuzz;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, MsgPack, MsgUnpack)]
pub struct Fgblrp<T, Y> {
    pub t: T,
    pub y: Y,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, MsgPack, MsgUnpack)]
pub struct WithLifetime<'a> {
    pub s: &'a str,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, MsgPack, MsgUnpack)]
pub enum Baz {
    Bill,
    Bob(u32),
    Bung { field1: Foo, field2: u32 },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, MsgPack)]
#[msgpck(untagged)]
#[serde(untagged)]
pub enum UntaggedBaz {
    Bill,
    Bob(u32),
    Bung { field1: Foo, field2: u32 },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, MsgPack, MsgUnpack)]
pub struct UntaggedBazBung {
    pub field1: Foo,
    pub field2: u32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, MsgPack, MsgUnpack)]
#[repr(i8)]
pub enum CStyleEnum {
    Zero,
    One,
    Five = 5,
    Six,
    Eight = 8,
    Neg = -45,
    Neg2,
}

impl Arbitrary for CStyleEnum {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        *g.choose(&[
            CStyleEnum::Zero,
            CStyleEnum::One,
            CStyleEnum::Five,
            CStyleEnum::Six,
            CStyleEnum::Eight,
            CStyleEnum::Neg,
            CStyleEnum::Neg2,
        ])
        .unwrap()
    }
}

pub fn test_pack_unpack<T>(original: &T)
where
    T: Debug + Serialize + PartialEq + MsgPack,
    T: for<'a> Deserialize<'a>,
    T: for<'a> MsgUnpack<'a>,
{
    println!("original: {original:x?}");

    let packed_rmp = rmp_serde::to_vec(original).expect("pack value using rmp_serde");
    let packed_msgpck = pack_with_iterator(original);
    let packed_msgpck2 = msgpck::pack_vec(original).unwrap();

    println!("packed (rmp_serde):        {packed_rmp:x?}");
    println!("packed (msgpck iter):   {packed_msgpck:x?}");
    println!("packed (msgpck writer): {packed_msgpck2:x?}");
    assert_eq!(
        packed_rmp, packed_msgpck,
        "msgpck must be compatible with rmp_serde"
    );
    assert_eq!(
        packed_msgpck, packed_msgpck2,
        "`MsgPack::pack` must be compatible with `MsgPack::pack_with_writer`",
    );

    let unpacked_rmp: T = rmp_serde::from_slice(&packed_rmp).expect("unpack value using rmp_serde");
    let unpacked_msgpck: T =
        msgpck::unpack_slice(&packed_msgpck).expect("unpack value using msgpck");

    println!("unpacked (rmp_serde): {unpacked_rmp:x?}");
    println!("unpacked (msgpck): {unpacked_msgpck:x?}");

    assert_eq!(
        original, &unpacked_rmp,
        "must be the same after unpacking with rmp"
    );
    assert_eq!(
        original, &unpacked_msgpck,
        "must be the same after unpacking with msgpck"
    );

    println!();
}

/// Pack using [MsgPack::pack] instead of [MsgPack::pack_with_writer],
/// which [msgpck::pack_vec] uses.
fn pack_with_iterator(v: &impl MsgPack) -> Vec<u8> {
    let mut out = vec![];
    for piece in v.pack() {
        out.extend_from_slice(piece.as_bytes());
    }
    out
}

pub fn test_uint<I: TryFrom<u64>>(int: u64)
where
    I: TryFrom<u64>,
    I: Debug + Serialize + PartialEq + MsgPack,
    I: for<'a> Deserialize<'a>,
    I: for<'a> MsgUnpack<'a>,
{
    let Ok(int) = I::try_from(int) else {
        return;
    };

    test_pack_unpack(&int);
}

pub fn test_int<I: TryFrom<i64>>(int: i64)
where
    I: TryFrom<i64>,
    I: Debug + Serialize + PartialEq + MsgPack,
    I: for<'a> Deserialize<'a>,
    I: for<'a> MsgUnpack<'a>,
{
    let Ok(int) = I::try_from(int) else {
        return;
    };

    test_pack_unpack(&int);
}
