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

pub trait Source: Sized + Sync + Send {
    type Item;

    fn from_vec(items: Vec<Self::Item>) -> Result<Self> {
        Self::from_vec_with_offset(items, 0)
    }

    fn from_vec_with_offset(items: Vec<Self::Item>, initial_offset: usize) -> Result<Self>;

    fn validate_offset(&self, offset: usize) -> Result<()> {
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
    fn size(&self) -> usize;

    /// The initial offset of the [`Source`]. For more information, see the **Offsets** section
    /// of the [`Source`] documentation.
    fn initial_offset(&self) -> usize;

    fn all(&self) -> Result<Segment<Self::Item>> {
        self.segment(self.lower_offset_limit(), self.upper_offset_limit())
    }

    fn segment(&self, start: usize, end: usize) -> Result<Segment<Self::Item>>;

    #[inline]
    /// The lowest valid offset that can be requested. By default, this is the same as
    /// [`Source::initial_offset`].
    fn lower_offset_limit(&self) -> usize {
        self.initial_offset()
    }

    #[inline]
    /// The highest valid offset that can be requested. By default, this is the reader's
    /// [`Source::size`] plus its [`Source::initial_offset`].
    fn upper_offset_limit(&self) -> usize {
        self.size() + self.initial_offset()
    }
}

pub trait U8Source: Source<Item = u8> {
    /// The endidness of the reader.
    fn endidness(&self) -> Endidness;

    /// Changes the default endidness.
    fn change_endidness(&mut self, endidness: Endidness);

    #[inline]
    fn from_u8_slice(slice: &[u8], endidness: Endidness) -> Result<Self> {
        Self::from_u8_slice_with_offset(slice, 0, endidness)
    }

    fn from_u8_slice_with_offset(
        slice: &[u8],
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self>;

    #[inline]
    fn from_u8_vec(items: Vec<u8>, endidness: Endidness) -> Result<Self> {
        Self::from_u8_vec_with_offset(items, 0, endidness)
    }

    #[inline]
    fn from_u8_vec_with_offset(
        items: Vec<u8>,
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self> {
        Self::from_u8_slice_with_offset(&items, initial_offset, endidness)
    }

    fn from_file_with_offset<P: AsRef<Path>>(
        path: P,
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self>;

    #[inline]
    fn from_file<P: AsRef<Path>>(path: P, endidness: Endidness) -> Result<Self> {
        Self::from_file_with_offset(path, 0, endidness)
    }

    #[cfg(feature = "bytes")]
    fn from_bytes_with_offset(
        bytes: Bytes,
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self>;

    #[cfg(feature = "bytes")]
    #[inline]
    fn from_bytes(bytes: Bytes, endidness: Endidness) -> Result<Self> {
        Self::from_bytes_with_offset(bytes, 0, endidness)
    }

    #[inline]
    fn segment(&self, start: usize, end: usize) -> Result<Segment<u8>> {
        Source::segment(self, start, end)
    }
}
