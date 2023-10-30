use pretty_hex::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct MaybeFoo {
    pub bar: Option<String>,
}

#[test]
fn options() {
    let some = MaybeFoo {
        bar: Some("hello".into()),
    };
    let packed = rmp_serde::to_vec(&some).expect("pack value using rmp_serde");
    dbg!(packed.hex_dump());

    let none = MaybeFoo { bar: None };
    let packed = rmp_serde::to_vec(&none).expect("pack value using rmp_serde");
    dbg!(packed.hex_dump());
}
