//!

#![allow(clippy::needless_range_loop)]

// Needed for some macros to work in this package.
#[allow(unused_imports)]
use crate as segsource;

pub(crate) mod sources;
pub use sources::*;

pub(crate) mod error;
pub use error::*;

pub(crate) mod segment;
pub use segment::*;

#[cfg(test)]
mod testing;

#[derive(Debug, Clone, Copy)]
pub enum Endidness {
    Big,
    Little,
    Unknown,
}
