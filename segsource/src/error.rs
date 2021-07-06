use std::io;

/// The standard errors used by segsource.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Returned if an offset is too small.
    #[error("An offset of 0x{0:x} is too small.")]
    OffsetTooSmall(usize),
    /// Returned if an offset is too large.
    #[error("An offset of 0x{0:x} is too large.")]
    OffsetTooLarge(usize),
    /// Returned if there's not enough data left in a [`crate::Segment`].
    #[error("Requested {0} bytes, but only {1} bytes left.")]
    NotEnoughData(usize, usize),
    /// Returned if there's no data left in a [`crate::Segment`] relative to its.
    /// [crate::Segment::current_offset].
    #[error("No more data left.")]
    NoMoreData,
    /// Wraps a `std::io::Error`.
    #[error("{0}")]
    IoError(io::Error),
    /// Any other sort of error.
    #[error("{0}")]
    Other(String),
}

impl From<Error> for io::Error {
    fn from(e: Error) -> Self {
        io::Error::new(io::ErrorKind::Other, e)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::IoError(error)
    }
}

/// Your usual `Result` object that has its error position filled (in this case with [`Error`].
pub type Result<V> = std::result::Result<V, Error>;
