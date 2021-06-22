#[cfg(feature = "async")]
use crate::segment::AsyncSegment;
use crate::{
    error::{Error, Result},
    segment::Segment,
    Endidness,
};
use std::path::Path;

mod vec_source;
pub use vec_source::VecSource;

#[cfg(feature = "bytes")]
use bytes::Bytes;
#[cfg(feature = "bytes")]
mod bytes_source;

#[cfg(feature = "bytes")]
pub use bytes_source::BytesSource;

#[cfg(feature = "memmap")]
mod mmap;
#[cfg(feature = "memmap")]
pub use mmap::MappedFileSource;

pub trait Source: Sized {
    fn from_file_with_offset<P: AsRef<Path>>(
        path: P,
        initial_offset: u64,
        endidness: Endidness,
    ) -> Result<Self>;

    fn from_file<P: AsRef<Path>>(path: P, endidness: Endidness) -> Result<Self> {
        Self::from_file_with_offset(path, 0, endidness)
    }

    #[cfg(feature = "bytes")]
    fn from_bytes_with_offset(
        bytes: Bytes,
        initial_offset: u64,
        endidness: Endidness,
    ) -> Result<Self>;

    #[cfg(feature = "bytes")]
    fn from_bytes(bytes: Bytes, endidness: Endidness) -> Result<Self> {
        Self::from_bytes_with_offset(bytes, 0, endidness)
    }

    fn validate_offset(&self, offset: u64) -> Result<()> {
        if offset < self.lower_offset_limit() {
            Err(Error::OffsetTooSmall(offset))
        } else if offset > self.upper_offset_limit() {
            Err(Error::OffsetTooLarge(offset))
        } else {
            Ok(())
        }
    }

    /// The amount of data in the reader. If the reader's size changes (which none of the
    /// implementations currently do), then this should return how much data was *initially* in the
    /// reader.
    fn size(&self) -> u64;

    /// The initial offset of the [`Source`]. For more information, see the **Offsets** section
    /// of the [`Source`] documentation.
    fn initial_offset(&self) -> u64;

    /// The endidness of the reader.
    fn endidness(&self) -> Endidness;

    /// Changes the default endidness.
    fn change_endidness(&mut self, endidness: Endidness);

    fn all(&self) -> Result<Segment> {
        self.segment(self.lower_offset_limit(), self.upper_offset_limit())
    }

    fn segment(&self, start: u64, end: u64) -> Result<Segment>;

    #[cfg(feature = "async")]
    fn async_all(&self) -> Result<AsyncSegment> {
        self.async_segment(self.lower_offset_limit(), self.upper_offset_limit())
    }

    #[cfg(feature = "async")]
    fn async_segment(&self, start: u64, end: u64) -> Result<AsyncSegment>;

    #[inline]
    /// The lowest valid offset that can be requested. By default, this is the same as
    /// [`Source::initial_offset`].
    fn lower_offset_limit(&self) -> u64 {
        self.initial_offset()
    }

    #[inline]
    /// The highest valid offset that can be requested. By default, this is the reader's
    /// [`Source::size`] plus its [`Source::initial_offset`].
    fn upper_offset_limit(&self) -> u64 {
        self.size() + self.initial_offset()
    }
}
