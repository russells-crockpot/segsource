use super::Segment;
use crate::{
    error::{Error, Result},
    marker::Integer,
    Endidness,
};
use core::convert::TryFrom;
/// An alias for a segment that deals with binary data.
pub type DataSegment<'s> = Segment<'s, u8>;

macro_rules! make_num_method {
    ($type:ty, $name:ident, $method:ident, $($doc:literal),+) => {
        $(#[doc = $doc])+
        pub fn $name(&self) -> Result<$type> {
            self.$method::<$type>()
        }
    };
}
macro_rules! make_num_method_with_offset {
    ($type:ty, $name:ident, $method:ident, $($doc:literal),+) => {
        $(#[doc = $doc])+
        pub fn $name(&self, offset: usize) -> Result<$type> {
            self.$method::<$type>(offset)
        }
    };
}

impl<'s> DataSegment<'s> {
    /// Creates a new [`Segment`] using the provided endidness.
    ///
    /// Note: Only available if the [`Segment`]'s I is `u8`.
    #[inline]
    pub fn with_endidness(data: &'s [u8], endidness: Endidness) -> Self {
        Self::new_full(data, 0, 0, endidness)
    }

    /// Creates a new [`Segment`] using the provided endidness and initial offset.
    ///
    /// Note: Only available if the [`Segment`]'s I is `u8`.
    #[inline]
    pub fn with_offset_and_endidness(
        data: &'s [u8],
        initial_offset: usize,
        endidness: Endidness,
    ) -> Self {
        Self::new_full(data, initial_offset, 0, endidness)
    }

    #[inline]
    /// The endidness of the [`Segment`].
    ///
    /// Note: Only available if the [`Segment`]'s I is `u8`.
    pub fn endidness(&self) -> Endidness {
        self.endidness
    }

    /// Fills the provided buffer with the next n bytes, where n is the length of the buffer. This
    /// then advances the [`Segment::current_offset`] by n.
    ///
    /// Note: Only available if the [`Segment`]'s I is `u8`.
    pub fn next_bytes(&self, buf: &mut [u8]) -> Result<()> {
        //FIXME not async/threadsafe
        for i in 0..buf.len() {
            buf[i] = self.next_u8()?;
        }
        Ok(())
    }

    fn int_at_pos<N: Integer>(&self, pos: usize) -> Result<N> {
        self.validate_pos(pos, N::WIDTH - 1)?;
        Ok(N::with_endidness(
            &self.data[pos..pos + N::WIDTH],
            self.endidness,
        ))
    }

    /// Gets an integer of the provided type (e.g. `u8`, `i8`, `u16`, `i16`, etcetera) at the given
    /// offset without altering the [`Segment::current_offset`]. In most cases, you should use
    /// methods like [`Segment::u8_at`] instead.
    ///
    /// Note: Only available if the [`Segment`]'s I is `u8`.
    pub fn int_at<N: Integer>(&self, offset: usize) -> Result<N> {
        self.validate_offset(offset, N::WIDTH - 1)?;
        Ok(N::with_endidness(
            &self[offset..offset + N::WIDTH],
            self.endidness,
        ))
    }

    #[inline]
    /// See the documentation for [`Segment::int_at`].
    ///
    /// Note: Only available if the [`Segment`]'s I is `u8`.
    pub fn u8_at(&self, offset: usize) -> Result<u8> {
        self.item_at(offset)
    }
    make_num_method_with_offset! {u16, u16_at, int_at,
    "See the documentation for [`Segment::int_at`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}

    make_num_method_with_offset! {u32, u32_at, int_at,
    "See the documentation for [`Segment::int_at`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}

    make_num_method_with_offset! {u64, u64_at, int_at,
    "See the documentation for [`Segment::int_at`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}

    make_num_method_with_offset! {u128, u128_at, int_at,
    "See the documentation for [`Segment::int_at`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}

    make_num_method_with_offset! {i8, i8_at, int_at,
    "See the documentation for [`Segment::int_at`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}

    make_num_method_with_offset! {i16, i16_at, int_at,
    "See the documentation for [`Segment::int_at`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}

    make_num_method_with_offset! {i32, i32_at, int_at,
    "See the documentation for [`Segment::int_at`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}

    make_num_method_with_offset! {i64, i64_at, int_at,
    "See the documentation for [`Segment::int_at`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}

    make_num_method_with_offset! {i128, i128_at, int_at,
    "See the documentation for [`Segment::int_at`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}

    /// Gets an integer of the provided type (e.g. `u8`, `i8`, `u16`, `i16`, etcetera) starting at
    /// the at the [`Segment::current_offset`] without altering it. In most cases, you should use
    /// methods like [`Segment::current_u8`] instead.
    ///
    /// Note: Only available if the [`Segment`]'s I is `u8`.
    #[inline]
    pub fn current_int<N: Integer>(&self) -> Result<N> {
        self.int_at(self.current_offset())
    }

    make_num_method! {u8, current_u8, current_int,
    "See the documentation for [`Segment::current_int`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}

    make_num_method! {u16, current_u16, current_int,
    "See the documentation for [`Segment::current_int`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}

    make_num_method! {u32, current_u32, current_int,
    "See the documentation for [`Segment::current_int`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}

    make_num_method! {u64, current_u64, current_int,
    "See the documentation for [`Segment::current_int`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}

    make_num_method! {u128, current_u128, current_int,
    "See the documentation for [`Segment::current_int`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}

    make_num_method! {i8, current_i8, current_int,
    "See the documentation for [`Segment::current_int`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}

    make_num_method! {i16, current_i16, current_int,
    "See the documentation for [`Segment::current_int`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}

    make_num_method! {i32, current_i32, current_int,
    "See the documentation for [`Segment::current_int`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}

    make_num_method! {i64, current_i64, current_int,
    "See the documentation for [`Segment::current_int`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}

    make_num_method! {i128, current_i128, current_int,
    "See the documentation for [`Segment::current_int`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}

    /// Gets an integer of the provided type (e.g. `u8`, `i8`, `u16`, `i16`, etcetera) starting at
    /// the at the [`Segment::current_offset`] but without advancing the
    /// [`Segment::current_offset`]. In most
    /// cases, you should use methods like [`Segment::peek_u8`] instead.
    ///
    /// Note: Only available if the [`Segment`]'s I is `u8`.
    pub fn peek_int<N: Integer>(&self) -> Result<N> {
        let pos = self.adj_pos(N::WIDTH as i128)?;
        self.int_at(self.pos_to_offset(pos))
    }

    make_num_method! {u8, peek_u8, peek_int,
    "See the documentation for [`Segment::peek_int`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}

    make_num_method! {u16, peek_u16, peek_int,
    "See the documentation for [`Segment::peek_int`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}

    make_num_method! {u32, peek_u32, peek_int,
    "See the documentation for [`Segment::peek_int`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}

    make_num_method! {u64, peek_u64, peek_int,
    "See the documentation for [`Segment::peek_int`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}

    make_num_method! {u128, peek_u128, peek_int,
    "See the documentation for [`Segment::peek_int`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}

    make_num_method! {i8, peek_i8, peek_int,
    "See the documentation for [`Segment::peek_int`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}

    make_num_method! {i16, peek_i16, peek_int,
    "See the documentation for [`Segment::peek_int`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}

    make_num_method! {i32, peek_i32, peek_int,
    "See the documentation for [`Segment::peek_int`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}

    make_num_method! {i64, peek_i64, peek_int,
    "See the documentation for [`Segment::peek_int`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}

    make_num_method! {i128, peek_i128, peek_int,
    "See the documentation for [`Segment::peek_int`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}

    /// Gets an integer of the provided type (e.g. `u8`, `i8`, `u16`, `i16`, etcetera) starting at
    /// the at the [`Segment::current_offset`] and then advances the [`Segment::current_offset`] by
    /// n, where n is the number of bytes required to create the requested integer type. In most
    /// cases, you should use methods like [`Segment::next_u8`] instead.
    ///
    /// Note: Only available if the [`Segment`]'s I is `u8`.
    pub fn next_int<N: Integer>(&self) -> Result<N> {
        let pos = self.adj_pos(N::WIDTH as i128)?;
        self.int_at(self.pos_to_offset(pos))
    }

    #[inline]
    /// See the documentation for [`Segment::next_int`].
    ///
    /// Note: Only available if the [`Segment`]'s I is `u8`.
    pub fn next_u8(&self) -> Result<u8> {
        self.next_item()
    }

    make_num_method! {u16, next_u16, next_int,
    "See the documentation for [`Segment::next_int`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}

    make_num_method! {u32, next_u32, next_int,
    "See the documentation for [`Segment::next_int`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}

    make_num_method! {u64, next_u64, next_int,
    "See the documentation for [`Segment::next_int`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}

    make_num_method! {u128, next_u128, next_int,
    "See the documentation for [`Segment::next_int`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}

    make_num_method! {i8, next_i8, next_int,
    "See the documentation for [`Segment::next_int`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}

    make_num_method! {i16, next_i16, next_int,
    "See the documentation for [`Segment::next_int`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}

    make_num_method! {i32, next_i32, next_int,
    "See the documentation for [`Segment::next_int`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}

    make_num_method! {i64, next_i64, next_int,
    "See the documentation for [`Segment::next_int`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}

    make_num_method! {i128, next_i128, next_int,
    "See the documentation for [`Segment::next_int`].\n\n",
    "Note: Only available if the [`Segment`]'s I is `u8`."}
}
impl<'s> TryFrom<&DataSegment<'s>> for () {
    type Error = Error;
    fn try_from(_: &Segment<'s, u8>) -> Result<Self> {
        Ok(())
    }
}
impl<'s> TryFrom<&Segment<'s, u8>> for u8 {
    type Error = Error;
    fn try_from(segment: &Segment<'s, u8>) -> Result<Self> {
        segment.next_item()
    }
}

macro_rules! impl_try_from {
    ($type:ty) => {
        impl<'s> TryFrom<&Segment<'s, u8>> for $type {
            type Error = Error;
            fn try_from(segment: &Segment<'s, u8>) -> Result<Self> {
                segment.next_int()
            }
        }
        impl<'s, const N: usize> TryFrom<&Segment<'s, u8>> for [$type; N] {
            type Error = Error;
            fn try_from(segment: &Segment<'s, u8>) -> Result<Self> {
                let pos = segment.adj_pos((<$type>::WIDTH * N) as i128)?;
                let mut array = [0; N];
                for i in 0..N {
                    array[i] = segment.int_at_pos(pos + (i * <$type>::WIDTH))?
                }
                Ok(array)
            }
        }
    };
}

impl_try_from! { u16 }
impl_try_from! { u32 }
impl_try_from! { u64 }
impl_try_from! { u128 }

impl_try_from! { i8 }
impl_try_from! { i16 }
impl_try_from! { i32 }
impl_try_from! { i64 }
impl_try_from! { i128 }
