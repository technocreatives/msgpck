//! A simple test that checks whether msgpackers is compatible with rmp_serde.

#![feature(impl_trait_in_assoc_type)]

use msgpackers::{MsgPack, MsgUnpack};
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
    let packed_msgpackers = msgpackers::pack_vec(original);

    println!("packed (rmp_serde):    {packed_rmp:x?}");
    println!("packed (msgpackers):   {packed_msgpackers:x?}");
    assert_eq!(
        packed_rmp, packed_msgpackers,
        "msgpackers must be compatible with rmp_serde"
    );

    let unpacked_rmp: T = rmp_serde::from_slice(&packed_rmp).expect("unpack value using rmp_serde");
    let unpacked_msgpackers: T =
        msgpackers::unpack_bytes(&packed_msgpackers).expect("unpack value using msgpackers");

    println!("unpacked (rmp_serde):  {unpacked_rmp:x?}");
    println!("unpacked (msgpackers): {unpacked_msgpackers:x?}");

    assert_eq!(
        original, &unpacked_rmp,
        "must be the same after unpacking with rmp"
    );
    assert_eq!(
        original, &unpacked_msgpackers,
        "must be the same after unpacking with msgpackers"
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
