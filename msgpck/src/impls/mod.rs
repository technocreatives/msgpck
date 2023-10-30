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

pub use enums::{EnumHeader, Variant};
