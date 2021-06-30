//!
use std::fmt;
// Needed for some macros to work in this package.
#[allow(unused_imports)]
use crate as segsource;

pub(crate) mod sources;
pub use sources::*;

pub(crate) mod error;
pub use error::*;

pub(crate) mod segment;
pub use segment::*;

pub mod util;

#[cfg(test)]
mod testing;

#[derive(Debug, Clone, Copy)]
pub enum Endidness {
    Big,
    Little,
    Unknown,
}

impl fmt::Display for Endidness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Big => write!(f, "Big"),
            Self::Little => write!(f, "Little"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

impl Endidness {
    #[cfg(target_endian = "big")]
    pub fn native() -> Self {
        Self::Big
    }
    #[cfg(target_endian = "little")]
    pub fn native() -> Self {
        Self::Little
    }
}
