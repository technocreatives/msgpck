pub mod bool;
pub mod borrow;
pub mod bytes;
pub mod floats;
pub mod ints;
pub mod option;
pub mod strings;
pub mod uints;

#[cfg(feature = "alloc")]
pub mod alloc;

#[cfg(feature = "std")]
pub mod std;

#[cfg(feature = "heapless07")]
pub mod heapless07;

#[cfg(feature = "heapless08")]
pub mod heapless08;
