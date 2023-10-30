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
mod refs;
mod strings;

pub use enums::{EnumHeader, Variant};
