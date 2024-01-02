//! A simple test that checks whether msgpck_rs is compatible with rmp_serde.

use msgpck_rs::{MsgPack, MsgUnpack};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize, PartialEq, MsgPack, MsgUnpack)]
pub struct Foo {
    pub bar: Bar,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, MsgPack, MsgUnpack)]
pub struct Bar {
    pub a: u8,
    pub b: Fizz,
    pub c: Vec<u16>,
    pub d: Fuzz,
    pub e: Fgblrp<Vec<i32>, i64>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, MsgPack, MsgUnpack)]
pub struct Fizz(pub u16);

#[derive(Debug, Serialize, Deserialize, PartialEq, MsgPack, MsgUnpack)]
pub struct Fuzz;

#[derive(Debug, Serialize, Deserialize, PartialEq, MsgPack, MsgUnpack)]
pub struct Fgblrp<T, Y> {
    pub t: T,
    pub y: Y,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, MsgPack, MsgUnpack)]
pub struct WithLifetime<'a> {
    pub s: &'a str,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, MsgPack, MsgUnpack)]
pub enum Baz {
    Bill,
    Bob(u32),
    Bung { field1: Foo, field2: u32 },
}

pub fn test_pack_unpack<T>(original: &T)
where
    T: Debug + Serialize + PartialEq + MsgPack,
    T: for<'a> Deserialize<'a>,
    T: for<'a> MsgUnpack<'a>,
{
    println!("original: {original:x?}");

    let packed_rmp = rmp_serde::to_vec(original).expect("pack value using rmp_serde");
    let packed_msgpck_rs = msgpck_rs::pack_vec(original);

    println!("packed (rmp_serde):   {packed_rmp:x?}");
    println!("packed (msgpck_rs):   {packed_msgpck_rs:x?}");
    assert_eq!(
        packed_rmp, packed_msgpck_rs,
        "msgpck_rs must be compatible with rmp_serde"
    );

    let unpacked_rmp: T = rmp_serde::from_slice(&packed_rmp).expect("unpack value using rmp_serde");
    let unpacked_msgpck_rs: T =
        msgpck_rs::unpack_bytes(&packed_msgpck_rs).expect("unpack value using msgpck_rs");

    println!("unpacked (rmp_serde): {unpacked_rmp:x?}");
    println!("unpacked (msgpck_rs): {unpacked_msgpck_rs:x?}");

    assert_eq!(
        original, &unpacked_rmp,
        "must be the same after unpacking with rmp"
    );
    assert_eq!(
        original, &unpacked_msgpck_rs,
        "must be the same after unpacking with msgpck_rs"
    );

    println!();
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
