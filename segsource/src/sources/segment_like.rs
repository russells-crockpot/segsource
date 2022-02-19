use crate::{Endidness, Result, Segment, Source, U8Source};
#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, vec::Vec};
#[cfg(feature = "with-bytes")]
use bytes::Bytes;
use core::{marker::PhantomPinned, ops::Deref};
#[cfg(feature = "std")]
use std::path::Path;

/// A [`SegmentLikeSource`] is a source that acts as a wrapper around another type of [`Source`].
/// Unlike a normal [`Source`], however, this one can be dereferenced into a [`Segment`].
///
/// **Note**: Unlike most [`Source`]s, this one is neither `Sync` nor `Send`. Because of this,
/// unlike other sources, it does not implement the [`crate::AsyncU8Source`] trait.
pub struct SegmentLikeSource<'s, S>(*const Segment<'s, S::Item>, *mut S, PhantomPinned)
where
    Self: 'static,
    S: Source + 'static;

impl<'s, S> SegmentLikeSource<'s, S>
where
    Self: 'static,
    S: Source + 's,
{
    pub fn new(source: S) -> Result<Self> {
        let src_ptr = Box::into_raw(Box::new(source));
        let segment = unsafe { Box::new((*src_ptr).all()?) };
        let seg_ptr = Box::into_raw(segment);
        Ok(Self(seg_ptr, src_ptr, PhantomPinned))
    }
}

impl<'s, S> Deref for SegmentLikeSource<'s, S>
where
    Self: 'static,
    S: Source + 's,
{
    type Target = Segment<'s, S::Item>;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref().unwrap() }
    }
}

macro_rules! impl_source_proxy_func {
    ($method:ident ($($arg_name:ident : $arg_type:ty),*) $(-> $rtype:ty)?) => {
        #[inline]
        fn $method(&self, $($arg_name: $arg_type),*) $(-> $rtype)? {
            unsafe {
                self.1.as_ref().unwrap().$method($($arg_name),*)
            }
        }
    };
    (mut, $method:ident ($($arg_name:ident : $arg_type:ty),*) $(-> $rtype:ty)?) => {
        #[inline]
        fn $method(&mut self, $($arg_name: $arg_type),*) $(-> $rtype)? {
            unsafe {
                self.1.as_mut().unwrap().$method($($arg_name),*)
            }
        }
    };
}

impl<'s, S> Source for SegmentLikeSource<'s, S>
where
    Self: 'static,
    S: Source + 's,
{
    type Item = S::Item;

    #[inline]
    fn from_segment(segment: Segment<'_, Self::Item>) -> Result<Self>
    where
        Self::Item: Clone,
    {
        Self::new(S::from_segment(segment)?)
    }

    #[inline]
    fn from_vec(items: Vec<Self::Item>) -> Result<Self> {
        Self::new(S::from_vec(items)?)
    }

    #[inline]
    fn from_vec_with_offset(items: Vec<Self::Item>, initial_offset: usize) -> Result<Self> {
        Self::new(S::from_vec_with_offset(items, initial_offset)?)
    }

    #[inline]
    fn from_slice(slice: &[Self::Item]) -> Result<Self>
    where
        Self::Item: Clone,
    {
        Self::new(S::from_slice(slice)?)
    }

    #[inline]
    fn from_slice_with_offset(slice: &[Self::Item], initial_offset: usize) -> Result<Self>
    where
        Self::Item: Clone,
    {
        Self::new(S::from_slice_with_offset(slice, initial_offset)?)
    }

    impl_source_proxy_func! { validate_offset(offset: usize) -> Result<()> }
    impl_source_proxy_func! { size() -> usize }
    impl_source_proxy_func! { initial_offset() -> usize }
    impl_source_proxy_func! { all() -> Result<Segment<Self::Item>> }
    impl_source_proxy_func! { segment(start: usize, end: usize) -> Result<Segment<Self::Item>> }
    impl_source_proxy_func! { get_n(offset: usize, num_items: usize) -> Result<Segment<Self::Item>> }
    impl_source_proxy_func! { all_before(offset: usize) -> Result<Segment<Self::Item>> }
    impl_source_proxy_func! { all_after(offset: usize) -> Result<Segment<Self::Item>> }
    impl_source_proxy_func! { lower_offset_limit() -> usize }
    impl_source_proxy_func! { upper_offset_limit() -> usize }
    impl_source_proxy_func! { mut, change_initial_offset(offset: usize) }
}

macro_rules! impl_u8_source_proxy_func {
    ($method:ident ($($arg_name:ident : $arg_type:ty),*)) => {
        #[inline]
        fn $method($($arg_name: $arg_type),*) -> Result<Self> {
            Self::new(S::$method($($arg_name),*)?)
        }
    };
}

impl<'s, S> U8Source for SegmentLikeSource<'s, S>
where
    Self: 'static,
    S: U8Source + 's,
{
    impl_source_proxy_func! { endidness() -> Endidness }
    impl_source_proxy_func! { mut, change_endidness(endidness: Endidness) }
    impl_u8_source_proxy_func! {from_u8_slice(slice: &[u8], endidness: Endidness)}
    impl_u8_source_proxy_func! { from_u8_slice_with_offset(
    slice: &[u8], initial_offset: usize, endidness: Endidness) }

    impl_u8_source_proxy_func! { from_u8_vec(items: Vec<u8>, endidness: Endidness) }

    impl_u8_source_proxy_func! { from_u8_vec_with_offset(
    items: Vec<u8>, initial_offset: usize, endidness: Endidness) }

    #[cfg(feature = "with-bytes")]
    impl_u8_source_proxy_func! { from_bytes(bytes: Bytes, endidness: Endidness) }

    #[cfg(feature = "with-bytes")]
    impl_u8_source_proxy_func! { from_bytes_with_offset(
    bytes: Bytes, initial_offset: usize, endidness: Endidness) }

    #[cfg(feature = "std")]
    #[inline]
    fn from_file<P: AsRef<Path>>(path: P, endidness: Endidness) -> Result<Self> {
        Self::new(S::from_file(path, endidness)?)
    }

    #[cfg(feature = "std")]
    #[inline]
    fn from_file_with_offset<P: AsRef<Path>>(
        path: P,
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self> {
        Self::new(S::from_file_with_offset(path, initial_offset, endidness)?)
    }
}
