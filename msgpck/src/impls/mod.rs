#[cfg(test)]
macro_rules! roundtrip_proptest {
    ($testname:ident: $t:ty) => {
        proptest::proptest! {
            #[test]
            fn $testname(s: $t) {
                let mut writer: Vec<u8> = Vec::new();
                s.pack(&mut writer).unwrap();
                let d = <$t>::unpack(&mut &writer[..]).unwrap();
                assert_eq!(s, d);
            }
        }
    };
}

#[cfg(feature = "alloc")]
mod alloc;
mod array;
mod bools;
mod bytes;
mod enums;
mod floats;
#[cfg(feature = "heapless07")]
mod heapless07;
mod ints;
mod option;
mod refs;
#[cfg(feature = "std")]
mod std;
mod strings;

pub use bytes::ByteSlice;
pub use enums::{EnumHeader, Variant};
