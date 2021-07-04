#![allow(clippy::needless_range_loop)]
use crate::{
    error::{Error, Result},
    Endidness,
};
use segsource_macros::{for_each_number, make_number_methods};
use std::{
    borrow::Borrow,
    convert::TryFrom,
    io,
    ops::{self, Bound, Index, RangeBounds as _},
    sync::atomic::{AtomicUsize, Ordering},
};

/// A segment of a [`crate::Source`].
///
/// This is where data is actually read from. Each segment keeps track of a few things:
///
///     1. An initial offset (retrievable via [`Segment::initial_offset`]).
///     2. A cursor (retrievable via [`Segment::current_offset`]).
///     3. A reference to the source's data.
///
/// Additionally, it the data type is [`u8`] then the [`Endidness`] is also kep track of.
pub struct Segment<'s, I> {
    initial_offset: usize,
    position: AtomicUsize,
    data: &'s [I],
    // We use the slice's len a lot, and it never changes, so we might as well cache it.
    size: usize,
    // Used for u8 segments
    endidness: Endidness,
}

impl<'s, I> Segment<'s, I> {
    fn new_full(
        data: &'s [I],
        initial_offset: usize,
        position: usize,
        endidness: Endidness,
    ) -> Self {
        Self {
            initial_offset,
            position: AtomicUsize::new(position),
            data,
            endidness,
            size: data.len(),
        }
    }

    #[inline]
    fn get_pos(&self) -> usize {
        self.position.load(Ordering::Relaxed)
    }

    fn set_pos(&self, pos: usize) -> Result<()> {
        self.validate_pos(pos, 0)?;
        self.position.store(pos, Ordering::Relaxed);
        Ok(())
    }

    #[inline]
    fn to_pos(&self, offset: usize) -> usize {
        offset - self.initial_offset
    }

    fn adj_pos(&self, amt: i128) -> Result<usize> {
        let mut result = Ok(());
        let prev_pos = {
            let rval = self
                .position
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |p| {
                    let new_pos = (p as i128 + amt) as usize;
                    result = self.validate_pos(new_pos, 0);
                    if result.is_ok() {
                        Some(new_pos)
                    } else {
                        None
                    }
                });
            match rval {
                Ok(v) => v,
                Err(v) => v,
            }
        };
        match result {
            Err(e) => Err(e),
            Ok(_) => Ok(prev_pos),
        }
    }

    #[inline]
    fn inner_with_offset(data: &'s [I], initial_offset: usize, endidness: Endidness) -> Self {
        Self::new_full(data, initial_offset, 0, endidness)
    }

    #[inline]
    pub fn new(data: &'s [I]) -> Self {
        Self::new_full(data, 0, 0, Endidness::Unknown)
    }

    #[inline]
    pub fn set_initial_offset(&mut self, offset: usize) {
        self.initial_offset = offset;
    }

    /// Returns slice of the requested size containing the next n bytes (where n is
    /// the `num_bytes` parameter) and then advances the cursor by that much.
    pub fn next_n_as_slice(&self, num_items: usize) -> Result<&[I]> {
        let pos = self.adj_pos(num_items as i128)?;
        Ok(&self.data[pos..pos + num_items])
    }

    /// Gets the current byte and then advances the cursor.
    pub fn next_item_ref(&self) -> Result<&I> {
        let pos = self.adj_pos(1)?;
        Ok(&self.data[pos - 1])
    }

    pub fn next_n(&self, num_items: usize) -> Result<Segment<I>> {
        let pos = self.adj_pos(num_items as i128)?;
        Ok(Self::new_full(
            &self.data[pos..pos + num_items],
            self.initial_offset + pos,
            0,
            self.endidness,
        ))
    }

    /// Fills the provided buffer with the next n items, where n is the length of the buffer. This
    /// then advances the [`Segment::current_offset`] by n.
    pub fn next_item_refs(&self, buf: &mut [&'s I]) -> Result<()> {
        let offset = self.current_offset();
        self.validate_offset(offset, buf.len())?;
        let idx = self.to_pos(offset);
        let slice = &self.data[idx..idx + buf.len()];
        for i in 0..buf.len() {
            buf[i] = &slice[i];
        }
        Ok(())
    }

    #[inline]
    /// Generates a new [`Segment`] using the provided slice and initial offset.
    pub fn with_offset(data: &'s [I], initial_offset: usize) -> Self {
        Self::inner_with_offset(data, initial_offset, Endidness::Unknown)
    }

    #[inline]
    /// The initial offset of the [`Segment`]. For more information, see the **Offsets** section
    /// of the [`Segment`] documentation.
    pub fn initial_offset(&self) -> usize {
        self.initial_offset
    }

    #[inline]
    /// The amount of data in the reader. If the reader's size changes (which none of the
    /// implementations currently do), then this should return how much data was *initially* in the
    /// reader.
    pub fn size(&self) -> usize {
        self.size
    }

    #[inline]
    /// The current offset of the reader's cursor.
    pub fn current_offset(&self) -> usize {
        self.get_pos() + self.initial_offset
    }

    /// Sets the reader's [`Segment::current_offset`].
    pub fn move_to(&self, offset: usize) -> Result<()> {
        self.set_pos((offset - self.initial_offset) as usize)?;
        Ok(())
    }

    /// Alters the [`Segment::current_offset`] by the given amount.
    pub fn move_by(&self, num_items: i128) -> Result<()> {
        self.adj_pos(num_items)?;
        Ok(())
    }

    /// Gets the item at the provided offset without altering the [`Segment::current_offset`].
    pub fn item_ref_at(&self, offset: usize) -> Result<&I> {
        self.validate_offset(offset, 0)?;
        Ok(&self[offset])
    }

    pub fn current_item_ref(&self) -> Result<&I> {
        self.item_ref_at(self.current_offset())
    }

    #[inline]
    /// Gets a pointer to a slice of the byte at the [`Segment::current_offset`], as well as all
    /// all bytes afterwards. This does not alter the [`Segment::current_offset`].
    pub fn get_remaining_as_slice(&self) -> Result<&[I]> {
        let pos = self.adj_pos(self.remaining() as i128)?;
        Ok(&self.data[pos..])
    }

    #[inline]
    /// Gets a pointer to a slice of the byte at the [`Segment::current_offset`], as well as all
    /// all bytes afterwards. This does not alter the [`Segment::current_offset`].
    pub fn get_remaining(&self) -> Result<Self> {
        let remaining = self.remaining();
        //TODO remaining may have change between here
        let pos = self.adj_pos(remaining as i128)?;
        Ok(Self::new_full(
            &self.data[pos..pos + remaining],
            self.initial_offset + pos,
            0,
            self.endidness,
        ))
    }

    #[inline]
    /// The lowest valid offset that can be requested. By default, this is the same as
    /// [`Segment::initial_offset`].
    pub fn lower_offset_limit(&self) -> usize {
        self.initial_offset
    }

    #[inline]
    /// The highest valid offset that can be requested. By default, this is the reader's
    /// [`Segment::size`] plus its [`Segment::initial_offset`].
    pub fn upper_offset_limit(&self) -> usize {
        self.initial_offset + self.size
    }

    #[inline]
    /// Checks whether or not there is any data left, based off of the
    /// [`Segment::current_offset`].
    pub fn is_empty(&self) -> bool {
        self.remaining() == 0
    }

    #[inline]
    /// The amount of data left, based off of the [`Segment::current_offset`].
    fn calc_remaining(&self, pos: usize) -> usize {
        if pos > self.size {
            0
        } else {
            self.size - pos
        }
    }

    #[inline]
    /// The amount of data left, based off of the [`Segment::current_offset`].
    pub fn remaining(&self) -> usize {
        self.calc_remaining(self.get_pos())
    }

    #[inline]
    pub fn has_more(&self) -> bool {
        self.remaining() > 0
    }

    /// Fills the provided buffer with bytes, starting at the provided offset. This does not alter
    /// the [`Segment::current_offset`].
    pub fn item_refs_at<'a>(&'s self, offset: usize, buf: &mut [&'a I]) -> Result<()>
    where
        's: 'a,
    {
        self.validate_offset(offset, buf.len())?;
        for i in 0..buf.len() {
            buf[i] = self.item_ref_at(offset + i as usize)?;
        }
        Ok(())
    }
    fn validate_pos(&self, pos: usize, size: usize) -> Result<()> {
        if size > 0 && self.is_empty() {
            Err(Error::NoMoreData)
        } else if pos > self.size {
            Err(Error::OffsetTooLarge(pos))
        } else if pos > self.size - size as usize {
            Err(Error::NotEnoughData(size, self.remaining()))
        } else {
            Ok(())
        }
    }

    /// A helper method that validates an offset (mostly used by reader implementations).
    ///
    /// If the offset is valid, then `Ok(())` will be returned. Otherwise, the appropriate
    /// [`Error`] is returned (wrapped in `Err`, of course).
    pub fn validate_offset(&self, offset: usize, size: usize) -> Result<()> {
        if size > 0 && self.is_empty() {
            Err(Error::NoMoreData)
        } else if offset < self.lower_offset_limit() {
            Err(Error::OffsetTooSmall(offset))
        } else {
            self.validate_pos(self.to_pos(offset), size)
        }
    }

    /// Takes an absolute offset and converts it to a relative offset, based off of the
    /// [`Segment::current_offset`].
    pub fn relative_offset(&self, abs_offset: usize) -> Result<usize> {
        self.validate_offset(abs_offset, 0)?;
        Ok(abs_offset - self.current_offset())
    }

    /// Returns a subsequence (i.e. a `&[u8]`) of data of the requested size beginning at the
    /// provided offset.
    pub fn get_n(&self, offset: usize, num_items: usize) -> Result<Segment<I>> {
        self.validate_offset(offset, num_items)?;
        Ok(Segment::inner_with_offset(
            self.get_as_slice(offset, offset + num_items as usize)?,
            offset,
            self.endidness,
        ))
    }

    /// Returns a subsequence (i.e. a `&[u8]`) of data of the requested size beginning at the
    /// provided offset.
    pub fn get_n_as_slice(&self, offset: usize, num_items: usize) -> Result<&[I]> {
        self.validate_offset(offset, num_items)?;
        self.get_as_slice(offset, offset + num_items as usize)
    }

    /// Returns a slice of the data between the provided starting and ending offsets.
    pub fn get_as_slice(&self, start: usize, end: usize) -> Result<&[I]> {
        self.validate_offset(start, (end - start) as usize)?;
        Ok(&self.data[start as usize..end as usize])
    }

    pub fn segment(&self, start: usize, end: usize) -> Result<Segment<I>> {
        self.validate_offset(start, (end - start) as usize)?;
        Ok(Segment::inner_with_offset(
            &self[start..end],
            start,
            self.endidness,
        ))
    }

    pub fn all_after(&self, offset: usize) -> Result<Segment<I>> {
        self.validate_offset(offset, 0)?;
        Ok(Segment::inner_with_offset(
            &self[offset..],
            offset,
            self.endidness,
        ))
    }

    pub fn all_before(&self, offset: usize) -> Result<Segment<I>> {
        self.validate_offset(offset, 0)?;
        Ok(Segment::inner_with_offset(
            &self[..offset],
            self.initial_offset,
            self.endidness,
        ))
    }
}

impl<'s, I> Segment<'s, I>
where
    I: Default + Copy,
{
    pub fn next_n_as_array<const N: usize>(&self) -> Result<[I; N]> {
        let pos = self.adj_pos(N as i128)?;
        let mut array = [I::default(); N];
        array[..N].clone_from_slice(&self.data[pos..(N + pos)]);
        Ok(array)
    }
}

impl<'s, I> Segment<'s, I>
where
    I: PartialEq,
{
    /// Returns `true` if the next bytes are the same as the ones provided.
    pub fn next_items_are(&self, prefix: &[I]) -> Result<bool> {
        self.validate_offset(self.current_offset(), prefix.len())?;
        for i in 0..prefix.len() {
            if prefix[i] != self[self.current_offset() + i] {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

impl<'s> Segment<'s, u8> {
    #[inline]
    pub fn with_endidness(data: &'s [u8], endidness: Endidness) -> Self {
        Self::new_full(data, 0, 0, endidness)
    }

    #[inline]
    pub fn with_offset_and_endidness(
        data: &'s [u8],
        initial_offset: usize,
        endidness: Endidness,
    ) -> Self {
        Self::new_full(data, initial_offset, 0, endidness)
    }

    /// Gets the current byte and then advances the cursor.
    #[inline]
    pub fn next_u8(&self) -> Result<u8> {
        self.next_item()
    }

    #[inline]
    /// The endidness of the reader.
    pub fn endidness(&self) -> Endidness {
        self.endidness
    }

    /// Fills the provided buffer with the next n bytes, where n is the length of the buffer. This
    /// then advances the [`Segment::current_offset`] by n.
    pub fn next_bytes(&self, buf: &mut [u8]) -> Result<()> {
        for i in 0..buf.len() {
            buf[i] = self.next_u8()?;
        }
        Ok(())
    }

    /// Gets the [`u8`] at the [`Segment::current_offset`] without altering the
    /// [`Segment::current_offset`].
    pub fn current_u8(&self) -> Result<u8> {
        self.u8_at(self.current_offset())
    }

    make_number_methods! {
        /// Gets the numendlong endian `numname` at the [`Segment::current_offset`] without
        /// altering the [`Segment::current_offset`].
        pub fn current_numname_numend(&self) -> Result<_numname_> {
            let mut buf = [0; _numwidth_];
            self.items_at(self.current_offset(), &mut buf)?;
            Ok(_numname_::from_numend_bytes(buf))
        }
    }

    for_each_number! {
        pub fn current_numname(&self) -> Result<_numname_> {
            match self.endidness() {
                Endidness::Big => self.current_numname_be(),
                Endidness::Little => self.current_numname_le(),
                Endidness::Unknown => Err(Error::UnknownEndidness),
            }
        }
    }

    /// Gets the `u8` at the provided offset without altering the [`Segment::current_offset`].
    #[inline]
    pub fn u8_at(&self, offset: usize) -> Result<u8> {
        self.item_at(offset)
    }

    make_number_methods! {
        /// Gets the numendlong endian `numname` at the provided offset without altering the
        /// [`Segment::current_offset`].
        pub fn numname_numend_at(&self, offset: usize) -> Result<_numname_> {
            let mut buf = [0; _numwidth_];
            self.items_at(offset, &mut buf)?;
            Ok(_numname_::from_numend_bytes(buf))
        }
    }

    for_each_number! {
        /// Gets the `numname` using the default endidness at the provided offset without altering
        /// the [`Segment::current_offset`]. If the current endidness is [`Endidness::Unknown`],
        /// then an error is returned.
        pub fn numname_at(&self, offset: usize) -> Result<_numname_> {
            match self.endidness() {
                Endidness::Big => self.numname_be_at(offset),
                Endidness::Little => self.numname_le_at(offset),
                Endidness::Unknown => Err(Error::UnknownEndidness),
            }
        }
    }

    make_number_methods! {
        /// Gets the numendlong endian `numname` at the [`Segment::current_offset`] and then
        /// advances it by `1`.
        pub fn next_numname_numend(&self) -> Result<_numname_> {
            let mut buf = [0; _numwidth_];
            self.next_bytes(&mut buf)?;
            Ok(_numname_::from_numend_bytes(buf))
        }
    }

    /// Gets the `i8` at the [`Segment::current_offset`] and then advances it by `1`. If the
    /// current endidness is [`Endidness::Unknown`], then an error is returned.
    pub fn next_i8(&self) -> Result<i8> {
        let mut buf = [0; 1];
        self.next_bytes(&mut buf)?;
        Ok(i8::from_be_bytes(buf))
    }

    for_each_number! {
        /// Gets the `numname` using the default endidness at the [`Segment::current_offset`] and
        /// then advances it by `1`. If the current endidness is [`Endidness::Unknown`], then an
        /// error is returned.
        pub fn next_numname(&self) -> Result<_numname_> {
             match self.endidness() {
                Endidness::Big => self.next_numname_be(),
                Endidness::Little => self.next_numname_le(),
                Endidness::Unknown => Err(Error::UnknownEndidness),
            }
        }
    }
}

impl<'s> TryFrom<&Segment<'s, u8>> for u8 {
    type Error = Error;
    fn try_from(segment: &Segment<'s, u8>) -> Result<Self> {
        segment.next_u8()
    }
}

impl<'s> TryFrom<&Segment<'s, u8>> for i8 {
    type Error = Error;
    fn try_from(segment: &Segment<'s, u8>) -> Result<Self> {
        segment.next_i8()
    }
}

for_each_number! {
    impl<'s> TryFrom<&Segment<'s, u8>> for _numname_ {
        type Error = Error;
        fn try_from(segment: &Segment<'s, u8>) -> Result<Self> {
            segment.next_numname()
        }
    }
}

impl<'s, I: Clone> Segment<'s, I> {
    /// Fills the provided buffer with bytes, starting at the provided offset. This does not alter
    /// the [`Segment::current_offset`].
    pub fn items_at(&self, offset: usize, buf: &mut [I]) -> Result<()> {
        self.validate_offset(offset, buf.len())?;
        for i in 0..buf.len() {
            buf[i] = self.item_at(offset + i as usize)?.clone();
        }
        Ok(())
    }

    /// Gets the current byte and then advances the cursor.
    pub fn next_item(&self) -> Result<I> {
        let pos = self.adj_pos(1)?;
        Ok(self.data[pos].clone())
    }

    pub fn next_items(&self, buf: &mut [I]) -> Result<()> {
        let pos = self.adj_pos(buf.len() as i128)?;
        buf.clone_from_slice(&self.data[pos..pos + buf.len()]);
        Ok(())
    }

    /// Gets the item at the provided offset without altering the [`Segment::current_offset`].
    pub fn item_at(&self, offset: usize) -> Result<I> {
        self.validate_offset(offset, 0)?;
        Ok(self[offset].clone())
    }

    pub fn current_item(&self) -> Result<I> {
        self.item_at(self.current_offset())
    }
}

impl<'s, I> AsRef<[I]> for Segment<'s, I> {
    #[inline]
    fn as_ref(&self) -> &[I] {
        self.data
    }
}

impl<'s, I> Index<usize> for Segment<'s, I> {
    type Output = I;
    fn index(&self, idx: usize) -> &Self::Output {
        &self.data[self.to_pos(idx)]
    }
}

macro_rules! add_idx_range {
    ($type:ty) => {
        impl<'s, I> Index<$type> for Segment<'s, I> {
            type Output = [I];

            fn index(&self, idx: $type) -> &Self::Output {
                let start = match idx.start_bound() {
                    Bound::Unbounded => 0,
                    Bound::Included(i) => i - self.initial_offset,
                    Bound::Excluded(i) => (i + 1) - self.initial_offset,
                };
                let end = match idx.end_bound() {
                    Bound::Unbounded => self.size,
                    Bound::Included(i) => (i + 1) - self.initial_offset,
                    Bound::Excluded(i) => i - self.initial_offset,
                };
                &self.data[start..end]
            }
        }
    };
}

add_idx_range! { ops::Range<usize> }
add_idx_range! { ops::RangeFrom<usize> }
add_idx_range! { ops::RangeInclusive<usize> }
add_idx_range! { ops::RangeTo<usize> }
add_idx_range! { ops::RangeToInclusive<usize> }
add_idx_range! { ops::RangeFull }

impl<'s, I> Borrow<[I]> for Segment<'s, I> {
    #[inline]
    fn borrow(&self) -> &[I] {
        self.as_ref()
    }
}

impl<'s> io::Read for Segment<'s, u8> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.remaining() > buf.len() {
            self.next_bytes(buf)?;
            Ok(buf.len())
        } else {
            let read = self.remaining() as usize;
            for i in 0..read {
                buf[i] = self.next_u8()?;
            }
            Ok(read)
        }
    }
}

impl<'s> io::Seek for Segment<'s, u8> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        match pos {
            io::SeekFrom::Start(to) => self.move_to(to as usize)?,
            io::SeekFrom::Current(by) => {
                self.move_to((self.current_offset() as i128 + by as i128) as usize)?
            }
            io::SeekFrom::End(point) => {
                self.move_to((self.upper_offset_limit() as i128 - point as i128) as usize)?
            }
        };
        Ok(self.current_offset() as u64)
    }
}

impl<'s> io::BufRead for Segment<'s, u8> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        let pos = self.get_pos();
        if self.size - pos >= 4096 {
            Ok(&self.data[pos..pos + 4096])
        } else {
            Ok(&self.data[pos..])
        }
    }
    fn consume(&mut self, amt: usize) {
        if !self.is_empty() {
            if self.remaining() < amt {
                self.move_to(self.upper_offset_limit()).unwrap();
            } else {
                self.adj_pos(amt as i128).unwrap();
            }
        }
    }
}

impl<'s, I> Clone for Segment<'s, I> {
    fn clone(&self) -> Self {
        Self {
            initial_offset: self.initial_offset,
            position: AtomicUsize::new(self.get_pos()),
            data: self.data,
            endidness: self.endidness,
            size: self.size,
        }
    }
}

#[cfg(feature = "async")]
mod sync {
    use super::Segment;
    use crate::error::Error;
    use std::{
        cmp::min,
        io,
        pin::Pin,
        sync::atomic::Ordering,
        task::{Context, Poll},
    };
    use tokio::io::{AsyncBufRead, AsyncRead, AsyncSeek, ReadBuf};

    impl<'r> AsyncRead for Segment<'r, u8> {
        fn poll_read(
            self: Pin<&mut Self>,
            _: &mut Context,
            buf: &mut ReadBuf,
        ) -> Poll<io::Result<()>> {
            let to_fill = buf.capacity() - buf.filled().len();
            let mut end: usize = 0;
            let maybe_pos = self
                .position
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| {
                    let remaining = self.calc_remaining(n);
                    if remaining == 0 {
                        None
                    } else {
                        let new = min(n + to_fill, n + remaining);
                        end = new;
                        Some(new)
                    }
                });
            if let Ok(pos) = maybe_pos {
                buf.put_slice(&self.data[pos..end]);
            }
            Poll::Ready(Ok(()))
        }
    }

    impl<'r> AsyncSeek for Segment<'r, u8> {
        fn start_seek(self: Pin<&mut Self>, pos: io::SeekFrom) -> io::Result<()> {
            let result = match pos {
                io::SeekFrom::Start(to) => self.move_to(to as usize),
                io::SeekFrom::Current(by) => self.move_by(by as i128),
                io::SeekFrom::End(adj) => {
                    self.move_to((self.upper_offset_limit() as i64 + adj) as usize)
                }
            };
            match result {
                Ok(()) => Ok(()),
                Err(Error::IoError(e)) => Err(e),
                Err(e) => panic!("{}", e),
            }
        }
        fn poll_complete(self: Pin<&mut Self>, _: &mut Context) -> Poll<io::Result<u64>> {
            Poll::Ready(Ok(self.current_offset() as u64))
        }
    }

    impl<'r> AsyncBufRead for Segment<'r, u8> {
        fn poll_fill_buf(self: Pin<&mut Self>, _: &mut Context) -> Poll<io::Result<&[u8]>> {
            if self.remaining() == 0 {
                Poll::Ready(Ok(&[]))
            } else {
                let pos = self.get_pos();
                let to_get = min(8192, self.calc_remaining(pos));
                Poll::Ready(Ok(&self.data[pos..pos + to_get]))
            }
        }

        fn consume(self: Pin<&mut Self>, amount: usize) {
            self.adj_pos(amount as i128).unwrap();
        }
    }
}
#[cfg(feature = "async")]
pub use sync::*;
