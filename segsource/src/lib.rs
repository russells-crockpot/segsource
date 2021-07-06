//#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
//! Segsource is a crate designed to assist in reading data. Although is specifically designed with
//! binary (`u8`) data, it is not limited to this.
//!
//! ## Overview
//!
//! Segsource is designed to easily and efficiently allow the reading of data without using much
//! memory. It does this by having two basic concepts: [`Source`]s, which own the data and
//! [`Segment`]s which allow access the data.
//!
//! Each [`Segment`] is a thread-safe struct that retains minimal basic information such as the
//! position of an internal cursor (expressed via the [`Segment::current_offset`] method), a
//! reference to this data, etcetera.
//!
//! ## Feature Flags
//!
//! The following features are available for segsource:
//!
//! 1. `async` which adds support for various `async` operations using `tokio`.
//! 2. `bytes` which adds support for using the `bytes` crate.
//! 3. `derive` which includes several macros for creating structs from [`Segment`]s.
//! 4. `mmap` which adds support for memory mapped files.
//!
//! Of these, only `derive` is enabled by default.
//!
//! ## Why segsource?
//!
//! There are various other crates out there in a similar vein to segsource, and in your use case,
//! some of them might be a better idea. I'll go through a few of the other options and let you
//! decide for yourself:
//!
//! - `bytes`: `segsource` actually offers native support for `bytes` crate via the appropriately
//!   named `bytes` feature. While bytes is great, it does have its limitations, the two biggest
//!   ones being the most read operations require it to be mutable and that there's no way to go
//!   "back". Segsource solves both of these cases.
//!
//! - `binread`: Not a replacement for `segsource` as a whole, but for the derivations provided via
//!   the `derive` feature. As of this writing, `binread` is more feature rich than `segsource`'s
//!   derives (and since [`Segment`]s extend `std::io::Seek` and `std::io::Read`, they will work
//!   with `binread`]. Unfortunately, this again requires the passed in
//!
//! - `bitvec`: You may have noticed that you can essentially do simple memory emulation with
//!   `segsource (e.g. you can have an initial offset, you work in offsets, etcetera). Simple, being
//!   the keyword here. `bitvec` is not simple nor can it be given its scope.
//!
//! - `std`: You could use various items from the standard library, such as a `Vec` or an
//!   `io::Cursor`, but all of these have limitations (e.g. a `Vec` can't have an initial offset and
//!   a can only move relative to its current position).
//!
//! ## Derive
//!
//! Documentation is on my TODO list...
//!
//! ## Offsets
//!
//! Instead of indexes, segsource use offsets. Depending on your use case, these will probably end
//! up being the same. However, you can specify an initial offset that will essentially change the
//! index from zero to whatever the initial_offset is.
//!
//! For example:
//!
//! ```
//! # use segsource::{VecSource, Source as _, U8Source as _, Endidness};
//! # type SourceOfYourChoice = VecSource<u8>;
//! let test_data = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05];
//! let source = SourceOfYourChoice::from_u8_slice_with_offset(&test_data, 100, Endidness::Big).
//!     unwrap();
//! let segment = source.all().unwrap();
//! assert_eq!(segment.u8_at(100).unwrap(), 0);
//! ```
//!
//! ### Validation
//!
//! One thing you may have noticed is that we had to unwrap the value each time. This is because
//! methods first check to make an offset is valid. For example:
//!
//! ```
//! # use segsource::{VecSource, Source as _, U8Source as _, Endidness, Error};
//! # type SourceOfYourChoice = VecSource<u8>;
//! # let test_data = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05];
//! # let source = SourceOfYourChoice::from_u8_slice_with_offset(&test_data, 100, Endidness::Big).
//! #   unwrap();
//! # let segment = source.all().unwrap();
//! assert!(matches!(segment.u8_at(99), Err(Error::OffsetTooSmall(99))));
//! ```

use core::fmt;

pub(crate) mod sources;
pub use sources::*;

pub(crate) mod error;
pub use error::*;

pub(crate) mod segment;
pub use segment::*;

mod marker;

#[cfg(feature = "derive")]
#[doc(inline)]
pub use segsource_derive::{FromSegment, TryFromSegment};

#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
pub mod sync;

#[cfg(test)]
mod testing;

/// The "endidness" (i.e. big endian or little endian) of binary data. Defaults to the native
/// endidness.
#[derive(Debug, Clone, Copy)]
pub enum Endidness {
    Big,
    Little,
}

impl Endidness {
    #[cfg(target_endian = "big")]
    /// Returns the native endidness.
    #[inline]
    pub fn native() -> Self {
        Self::Big
    }
    #[cfg(target_endian = "little")]
    /// Returns the native endidness.
    #[inline]
    pub fn native() -> Self {
        Self::Little
    }
}

impl Default for Endidness {
    #[inline]
    fn default() -> Self {
        Self::native()
    }
}

impl fmt::Display for Endidness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Big => write!(f, "Big"),
            Self::Little => write!(f, "Little"),
        }
    }
}
