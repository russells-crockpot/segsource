use crate::{
    error::{Error, Result},
    segment::Segment,
    Endidness,
};
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "async")]
use async_trait::async_trait;
#[cfg(feature = "with-bytes")]
use bytes::Bytes;
#[cfg(feature = "std")]
use std::path::Path;

macro_rules! add_basic_source_items {
    ($data_prop_name:ident) => {
        #[inline]
        fn initial_offset(&self) -> usize {
            self.initial_offset
        }

        #[inline]
        fn change_initial_offset(&mut self, offset: usize) {
            self.initial_offset = offset
        }

        #[inline]
        fn size(&self) -> usize {
            self.$data_prop_name.len() as usize
        }
    };
    (@add_u8_items, $data_prop_name:ident) => {
        add_basic_source_items! { $data_prop_name }
        #[inline]
        fn from_slice_with_offset(slice: &[Self::Item], initial_offset: usize) -> Result<Self>
        where
            Self::Item: Clone,
        {
            Self::from_u8_slice_with_offset(&slice, initial_offset, Endidness::default())
        }

        #[inline]
        fn from_vec_with_offset(items: Vec<Self::Item>, initial_offset: usize) -> Result<Self> {
            Self::from_u8_slice_with_offset(&items, initial_offset, Endidness::default())
        }

        fn from_segment(segment: Segment<'_, Self::Item>) -> Result<Self>
        where
            Self::Item: Clone,
        {
            let mut src = Self::from_slice_with_offset(segment.as_ref(), segment.current_offset())?;
            src.change_endidness(segment.endidness());
            Ok(src)
        }

        fn segment(&self, start: usize, end: usize) -> Result<Segment<u8>> {
            self.validate_offset(start)?;
            self.validate_offset(end)?;
            Ok(Segment::with_offset_and_endidness(
                &self.$data_prop_name
                    [(start - self.initial_offset) as usize..(end - self.initial_offset) as usize],
                start,
                self.endidness,
            ))
        }
    };
    () => {
        add_basic_source_items! { data }
    };
    (@add_u8_items) => {
        add_basic_source_items! { @add_u8_items, data }
    };
}

macro_rules! impl_endidness_items {
    () => {
        #[inline]
        fn endidness(&self) -> Endidness {
            self.endidness
        }

        #[inline]
        fn change_endidness(&mut self, endidness: Endidness) {
            self.endidness = endidness
        }
    };
}

mod vec_source;
pub use vec_source::VecSource;

mod segment_like;
pub use segment_like::*;
#[cfg(feature = "with-bytes")]
mod bytes_source;

#[cfg(feature = "with-bytes")]
pub use bytes_source::BytesSource;

#[cfg(feature = "memmap")]
mod mmap;
#[cfg(feature = "memmap")]
pub use mmap::MappedFileSource;

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
pub trait Source: Sized {
    /// The type of item the [`Source`] and its generated [`Segment`]s will hold.
    type Item;

    fn from_slice_with_offset(slice: &[Self::Item], initial_offset: usize) -> Result<Self>
    where
        Self::Item: Clone;

    #[inline]
    fn from_slice(slice: &[Self::Item]) -> Result<Self>
    where
        Self::Item: Clone,
    {
        Self::from_slice_with_offset(slice, 0)
    }

    #[inline]
    fn from_segment(segment: Segment<'_, Self::Item>) -> Result<Self>
    where
        Self::Item: Clone,
    {
        Self::from_slice_with_offset(segment.as_ref(), segment.current_offset())
    }

    /// Creates a new source using the data in the `Vec` for its data.
    #[inline]
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

    /// Changes the [`Source::initial_offset`] This does **not** change the initial offset for any
    /// [`Segment`]s that have already been created, but all new [`Segment`]s will use the new
    /// offset.
    fn change_initial_offset(&mut self, offset: usize);

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
        self.segment(self.lower_offset_limit(), offset)
    }

    /// Gets all items in the source after the provided offset (inclusive).
    fn all_after(&self, offset: usize) -> Result<Segment<Self::Item>> {
        self.segment(offset, self.upper_offset_limit())
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

/// Segsource is mostly meant to work with binary data (although it by no means has to). Because of
/// this, sources can have some extra functionality when its item type is `u8`.
pub trait U8Source: Source<Item = u8> {
    /// The endidness of the source.
    fn endidness(&self) -> Endidness;

    /// Changes the default endidness. This does **not** change the endidness for any [`Segment`]s
    /// that have already been created, but all new [`Segment`]s will use the new endidness.
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

    #[cfg(feature = "with-bytes")]
    /// Creates a new source using the the provided Bytes and [`Endidness`].
    #[inline]
    fn from_bytes(bytes: Bytes, endidness: Endidness) -> Result<Self> {
        Self::from_bytes_with_offset(bytes, 0, endidness)
    }

    #[cfg(feature = "with-bytes")]
    /// Creates a new source using the the provided Bytes, [`Endidness`], and offset.
    fn from_bytes_with_offset(
        bytes: Bytes,
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self>;
}

#[cfg(feature = "async")]
/// A trait that extends a [`U8Source`] with some additional, asynchronous methods.
#[async_trait]
pub trait AsyncU8Source: U8Source {
    /// An async version of [`U8Source::from_file`].
    #[inline]
    async fn from_file_async<P>(path: P, endidness: Endidness) -> Result<Self>
    where
        P: AsRef<Path> + Sync + Send,
    {
        Self::from_file_with_offset_async(path, 0, endidness).await
    }

    /// An async version of [`U8Source::from_file_with_offset`].
    async fn from_file_with_offset_async<P>(
        path: P,
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self>
    where
        P: AsRef<Path> + Sync + Send;
}
