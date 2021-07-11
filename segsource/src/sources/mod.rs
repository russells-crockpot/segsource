use crate::{
    error::{Error, Result},
    segment::Segment,
    Endidness,
};
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::path::Path;

mod vec_source;
pub use vec_source::VecSource;

//mod segment_like;
//pub use segment_like::*;

#[cfg(feature = "with_bytes")]
use bytes::Bytes;
#[cfg(feature = "with_bytes")]
mod bytes_source;

#[cfg(feature = "with_bytes")]
pub use bytes_source::BytesSource;

#[cfg(feature = "memmap")]
mod mmap;
#[cfg(feature = "memmap")]
pub use mmap::MappedFileSource;

#[cfg(feature = "async")]
use async_trait::async_trait;

/// Sources own their own data and are used to generate [`Segment`]s. The following sources are
/// included with segsource (although others can be implemented):
///
/// 1. [`VecSource`]: A source that stores its items as a simple `Vec`. This source is always
///    available.
/// 2. [`BytesSource`]: A source that uses a `Bytes` object from the wonderful `bytes` crate to
///    store its data. This source can only use `u8`s as its item. Requires the `bytes` feature.
/// 3. [`MappedFileSource`]: A source that stores its data using a memory mapped file. This source
///    can only use `u8`s as its item. Requires the `mmap` feature.
///
/// When a [`Source`] creates a new [`Segment`], that segment will have the same initial offset and
/// (if applicable) the same endidness as the source.
pub trait Source: Sized + Sync + Send {
    /// The type of item the [`Source`] and its generated [`Segment`]s will hold.
    type Item;

    /// Creates a new source using the data in the `Vec` for its data.
    fn from_vec(items: Vec<Self::Item>) -> Result<Self> {
        Self::from_vec_with_offset(items, 0)
    }

    /// Creates a new source with the provided initial offset, using the items in the`Vec` for its
    /// data.
    fn from_vec_with_offset(items: Vec<Self::Item>, initial_offset: usize) -> Result<Self>;

    /// Checks to make sure that the provided offset is valid. If it is, then an `Ok(())` will be
    /// returned. Otherwise, the appropriate error will be returned.
    fn validate_offset(&self, offset: usize) -> Result<()> {
        if offset < self.lower_offset_limit() {
            Err(Error::OffsetTooSmall { offset })
        } else if offset > self.upper_offset_limit() {
            Err(Error::OffsetTooLarge { offset })
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

    /// Returns a single segment containing all data in the source.
    fn all(&self) -> Result<Segment<Self::Item>> {
        self.segment(self.lower_offset_limit(), self.upper_offset_limit())
    }

    /// Creates a segment from the start offset (inclusive) to the end offset (exclusive).
    fn segment(&self, start: usize, end: usize) -> Result<Segment<Self::Item>>;

    /// Gets a segment of n items, starting at the given offset.
    fn get_n(&self, offset: usize, num_items: usize) -> Result<Segment<Self::Item>> {
        self.validate_offset(offset)?;
        self.validate_offset(offset + num_items)?;
        self.segment(offset, offset + num_items)
    }

    /// Gets all items in the source before the provided offset (exclusive).
    fn all_before(&self, offset: usize) -> Result<Segment<Self::Item>> {
        self.validate_offset(offset)?;
        self.get_n(self.lower_offset_limit(), offset)
    }

    /// Gets all items in the source after the provided offset (inclusive).
    fn all_after(&self, offset: usize) -> Result<Segment<Self::Item>> {
        self.validate_offset(offset)?;
        self.get_n(offset, self.upper_offset_limit())
    }

    /// The lowest valid offset that can be requested.
    #[inline]
    fn lower_offset_limit(&self) -> usize {
        self.initial_offset()
    }

    /// The highest valid offset that can be requested.
    #[inline]
    fn upper_offset_limit(&self) -> usize {
        self.size() + self.initial_offset()
    }
}

#[cfg_attr(feature = "async", async_trait)]
/// Segsource is mostly meant to work with binary data (although it by no means has to). Because of
/// this, sources can have some extra functionality when its item type is `u8`.
pub trait U8Source: Source<Item = u8> {
    /// The endidness of the source.
    fn endidness(&self) -> Endidness;

    /// Changes the default endidness. This does **not** change the endidness for any [`Segment`]s
    /// that have already been created, but only for [`Segment`]s that are created in the future.
    fn change_endidness(&mut self, endidness: Endidness);

    /// Creates a new source using the the provided slice and [`Endidness`].
    ///
    /// Note: because sources own their data, this will copy the data from the provided slice.
    #[inline]
    fn from_u8_slice(slice: &[u8], endidness: Endidness) -> Result<Self> {
        Self::from_u8_slice_with_offset(slice, 0, endidness)
    }

    /// Creates a new source using the the provided slice, [`Endidness`], and offset.
    ///
    /// Note: because sources own their data, this will copy the data from the provided slice.
    fn from_u8_slice_with_offset(
        slice: &[u8],
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self>;

    /// Creates a new source using the the provided vec and [`Endidness`].
    #[inline]
    fn from_u8_vec(items: Vec<u8>, endidness: Endidness) -> Result<Self> {
        Self::from_u8_vec_with_offset(items, 0, endidness)
    }

    /// Creates a new source using the the provided vec, [`Endidness`], and offset.
    #[inline]
    fn from_u8_vec_with_offset(
        items: Vec<u8>,
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self> {
        Self::from_u8_slice_with_offset(&items, initial_offset, endidness)
    }

    #[cfg(feature = "std")]
    /// Creates a new source using the the provided file and [`Endidness`].
    #[inline]
    fn from_file<P: AsRef<Path>>(path: P, endidness: Endidness) -> Result<Self> {
        Self::from_file_with_offset(path, 0, endidness)
    }

    #[cfg(feature = "std")]
    /// Creates a new source using the the provided file, [`Endidness`], and offset.
    fn from_file_with_offset<P: AsRef<Path>>(
        path: P,
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self>;

    #[cfg(feature = "async")]
    /// An async version of [`U8Source::from_file`].
    #[inline]
    async fn from_file_async<P>(path: P, endidness: Endidness) -> Result<Self>
    where
        P: AsRef<Path> + Sync + Send,
    {
        Self::from_file_with_offset_async(path, 0, endidness).await
    }

    #[cfg(feature = "async")]
    /// An async version of [`U8Source::from_file_with_offset`].
    async fn from_file_with_offset_async<P>(
        path: P,
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self>
    where
        P: AsRef<Path> + Sync + Send;

    #[cfg(feature = "with_bytes")]
    /// Creates a new source using the the provided Bytes and [`Endidness`].
    #[inline]
    fn from_bytes(bytes: Bytes, endidness: Endidness) -> Result<Self> {
        Self::from_bytes_with_offset(bytes, 0, endidness)
    }

    #[cfg(feature = "with_bytes")]
    /// Creates a new source using the the provided Bytes, [`Endidness`], and offset.
    fn from_bytes_with_offset(
        bytes: Bytes,
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self>;
}
