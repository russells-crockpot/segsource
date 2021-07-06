#[cfg(feature = "std")]
use std::io;

#[cfg(not(feature = "std"))]
use alloc::string::String;

use snafu::Snafu;

/// The standard errors used by segsource.
#[derive(Snafu, Debug)]
pub enum Error {
    /// Returned if an offset is too small.
    #[snafu(display("An offset of 0x{:x} is too small.", offset))]
    OffsetTooSmall { offset: usize },
    /// Returned if an offset is too large.
    #[snafu(display("An offset of 0x{:x} is too large.", offset))]
    OffsetTooLarge { offset: usize },
    /// Returned if there's not enough data left in a [`crate::Segment`].
    #[snafu(display("Requested {} bytes, but only {} bytes left.", requested, left))]
    NotEnoughData { requested: usize, left: usize },
    /// Returned if there's no data left in a [`crate::Segment`] relative to its.
    /// [crate::Segment::current_offset].
    #[snafu(display("No more data left.",))]
    NoMoreData,
    #[cfg(feature = "std")]
    /// Wraps a `std::io::Error`.
    #[snafu(display("{}", error))]
    IoError { error: io::Error },
    /// Any other sort of error.
    #[snafu(display("{}", message))]
    Other { message: String },
}

#[cfg(feature = "std")]
impl From<Error> for io::Error {
    fn from(e: Error) -> Self {
        io::Error::new(io::ErrorKind::Other, e)
    }
}

#[cfg(feature = "std")]
impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::IoError { error }
    }
}

/// Your usual `Result` object that has its error position filled (in this case with [`Error`].
pub type Result<V> = core::result::Result<V, Error>;
