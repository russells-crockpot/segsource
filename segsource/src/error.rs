use std::io;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("An offset of 0x{0:x} is too small.")]
    OffsetTooSmall(u64),
    #[error("An offset of 0x{0:x} is too large.")]
    OffsetTooLarge(u64),
    #[error("Requested {0} bytes, but only {1} bytes left.")]
    /// NotEnoughData(bytes requested, bytes remaining)
    NotEnoughData(usize, u64),
    #[error("Attempted to call a method that requires knowing the endidness.")]
    UnknownEndidness,
    #[error("No more data left.")]
    NoMoreData,
    #[error("{0}")]
    IoError(io::Error),
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

pub type Result<V> = std::result::Result<V, Error>;
