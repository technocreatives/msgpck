#[cfg(feature = "alloc")]
mod alloc;
mod array;
mod bools;
mod bytes;
mod enums;
mod floats;
mod ints;
mod refs;
mod strings;

pub use enums::{EnumHeader, Variant};
