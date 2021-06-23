use crate::{
    error::{Error, Result},
    Endidness,
};
use segsource_macros::make_number_methods;
use std::{borrow::Borrow, io};
#[cfg(feature = "async")]
pub(crate) mod sync;
#[cfg(not(feature = "thread-safe"))]
use std::cell::Cell;
#[cfg(feature = "thread-safe")]
use std::sync::RwLock;
#[cfg(feature = "async")]
pub use sync::*;

pub struct Segment<'r> {
    initial_offset: u64,
    #[cfg(not(feature = "thread-safe"))]
    position: Cell<usize>,
    #[cfg(feature = "thread-safe")]
    position: RwLock<usize>,
    data: &'r [u8],
    endidness: Endidness,
}

impl<'r> Segment<'r> {
    #[cfg(feature = "thread-safe")]
    #[inline]
    fn new(data: &'r [u8], initial_offset: u64, endidness: Endidness) -> Self {
        Self {
            initial_offset,
            position: RwLock::new(0),
            data,
            endidness,
        }
    }

    #[cfg(feature = "thread-safe")]
    #[inline]
    fn get_pos(&self) -> usize {
        #[allow(clippy::clone_on_copy)]
        self.position.read().unwrap().clone()
    }

    #[cfg(feature = "thread-safe")]
    fn set_pos(&self, pos: usize) {
        let mut lock = self.position.write().unwrap();
        *lock = pos
    }

    #[cfg(feature = "thread-safe")]
    fn adj_pos(&self, amt: i128) {
        let mut lock = self.position.write().unwrap();
        if amt > 0 {
            *lock += amt as usize;
        } else {
            *lock -= (-amt) as usize;
        }
    }

    #[cfg(not(feature = "thread-safe"))]
    #[inline]
    fn new(data: &'r [u8], initial_offset: u64, endidness: Endidness) -> Self {
        Self {
            initial_offset,
            position: Cell::new(0),
            data,
            endidness,
        }
    }

    #[cfg(not(feature = "thread-safe"))]
    #[inline]
    fn get_pos(&self) -> usize {
        self.position.get()
    }

    #[cfg(not(feature = "thread-safe"))]
    fn adj_pos(&self, amt: i128) {
        let tmp = self.get_pos() as i128;
        self.position.replace((tmp + amt) as usize);
    }

    #[cfg(not(feature = "thread-safe"))]
    #[inline]
    fn set_pos(&self, pos: usize) {
        self.position.replace(pos);
    }

    /// Functions the same as the [`Segment::from_slice_with_offset`], except the initial offset
    /// is always `0`.
    pub fn from_slice(slice: &'r [u8], endidness: Endidness) -> Self {
        Self::from_slice_with_offset(slice, 0, endidness)
    }

    /// Generates a new [`Segment`] using the provided slice, initial offset, and endidness.
    pub fn from_slice_with_offset(
        slice: &'r [u8],
        initial_offset: u64,
        endidness: Endidness,
    ) -> Self {
        Self::new(slice, initial_offset, endidness)
    }

    #[inline]
    /// The initial offset of the [`Segment`]. For more information, see the **Offsets** section
    /// of the [`Segment`] documentation.
    pub fn initial_offset(&self) -> u64 {
        self.initial_offset
    }

    #[inline]
    /// The amount of data in the reader. If the reader's size changes (which none of the
    /// implementations currently do), then this should return how much data was *initially* in the
    /// reader.
    pub fn size(&self) -> u64 {
        self.data.len() as u64
    }

    #[inline]
    /// The current offset of the reader's cursor.
    pub fn current_offset(&self) -> u64 {
        self.get_pos() as u64 + self.initial_offset
    }

    #[inline]
    /// The endidness of the reader.
    pub fn endidness(&self) -> Endidness {
        self.endidness
    }

    #[inline]
    /// Changes the default endidness.
    pub fn change_endidness(&mut self, endidness: Endidness) {
        self.endidness = endidness
    }

    /// Sets the reader's [`Segment::current_offset`].
    pub fn move_to(&self, offset: u64) -> Result<()> {
        self.validate_offset(offset, 0)?;
        self.set_pos((offset - self.initial_offset) as usize);
        Ok(())
    }

    /// Alters the [`Segment::current_offset`] by the given amount.
    pub fn move_offset(&self, num_bytes: i128) -> Result<()> {
        self.validate_offset((self.current_offset() as i128 + num_bytes) as u64, 0)?;
        self.adj_pos(num_bytes);
        Ok(())
    }

    /// Gets the current byte and then advances the cursor.
    pub fn next_u8(&self) -> Result<u8> {
        self.validate_offset(self.current_offset(), 1)?;
        self.adj_pos(1);
        Ok(self.data[self.get_pos() as usize - 1])
    }

    /// Returns slice of the requested size containing the next n bytes (where n is
    /// the `num_bytes` parameter) and then advances the cursor by that much.
    pub fn next_n_bytes(&self, num_bytes: usize) -> Result<&[u8]> {
        self.validate_offset(self.current_offset(), num_bytes)?;
        let start = (self.current_offset() - self.initial_offset()) as usize;
        let data = &self.as_ref()[start..start + num_bytes as usize];
        self.move_offset(num_bytes as i128)?;
        Ok(data)
    }

    #[inline]
    /// Gets a pointer to a slice of the byte at the [`Segment::current_offset`], as well as all
    /// all bytes afterwards. This does not alter the [`Segment::current_offset`].
    pub fn get_remaining(&self) -> Result<&[u8]> {
        self.range(self.current_offset(), self.upper_offset_limit())
    }

    #[inline]
    /// The lowest valid offset that can be requested. By default, this is the same as
    /// [`Segment::initial_offset`].
    pub fn lower_offset_limit(&self) -> u64 {
        self.initial_offset()
    }

    #[inline]
    /// The highest valid offset that can be requested. By default, this is the reader's
    /// [`Segment::size`] plus its [`Segment::initial_offset`].
    pub fn upper_offset_limit(&self) -> u64 {
        self.size() + self.initial_offset()
    }

    #[inline]
    /// Checks whether or not there is any data left, based off of the
    /// [`Segment::current_offset`].
    pub fn is_empty(&self) -> bool {
        self.remaining() == 0
    }

    #[inline]
    /// The amount of data left, based off of the [`Segment::current_offset`].
    pub fn remaining(&self) -> u64 {
        self.upper_offset_limit() - self.current_offset()
    }

    /// A helper method that validates an offset (mostly used by reader implementations).
    ///
    /// If the offset is valid, then `Ok(())` will be returned. Otherwise, the appropriate
    /// [`Error`] is returned (wrapped in `Err`, of course).
    pub fn validate_offset(&self, offset: u64, size: usize) -> Result<()> {
        if size > 0 && self.is_empty() {
            Err(Error::NoMoreData)
        } else if offset < self.lower_offset_limit() {
            Err(Error::OffsetTooSmall(offset))
        } else if offset > self.upper_offset_limit() {
            Err(Error::OffsetTooLarge(offset))
        } else if offset > self.upper_offset_limit() - size as u64 {
            Err(Error::NotEnoughData(size, self.remaining()))
        } else {
            Ok(())
        }
    }

    /// Takes an absolute offset and converts it to a relative offset, based off of the
    /// [`Segment::current_offset`].
    pub fn relative_offset(&self, abs_offset: u64) -> Result<u64> {
        self.validate_offset(abs_offset, 0)?;
        Ok(abs_offset - self.current_offset())
    }

    /// Returns `true` if the next bytes are the same as the ones provided.
    pub fn next_bytes_are(&self, prefix: &[u8]) -> Result<bool> {
        self.validate_offset(self.current_offset(), prefix.len())?;
        let mut buf = Vec::with_capacity(prefix.len());
        (0..buf.len()).for_each(|_| buf.push(0));
        self.bytes_at(self.current_offset(), &mut buf)?;
        Ok(prefix.iter().zip(buf.into_iter()).all(|(v1, v2)| *v1 == v2))
    }

    /// Fills the provided buffer with bytes, starting at the provided offset. This does not alter
    /// the [`Segment::current_offset`].
    pub fn bytes_at(&self, offset: u64, buf: &mut [u8]) -> Result<()> {
        self.validate_offset(offset, buf.len())?;
        for i in 0..buf.len() {
            buf[i] = self.u8_at(offset + i as u64)?;
        }
        Ok(())
    }

    /// Returns a subsequence (i.e. a `&[u8]`) of data of the requested size beginning at the
    /// provided offset.
    pub fn subseq(&self, offset: u64, num_bytes: usize) -> Result<&[u8]> {
        self.validate_offset(offset, num_bytes)?;
        self.range(offset, offset + num_bytes as u64)
    }

    /// Returns a slice of the data between the provided starting and ending offsets.
    pub fn range(&self, start: u64, end: u64) -> Result<&[u8]> {
        self.validate_offset(start, (end - start) as usize)?;
        Ok(&self.as_ref()[start as usize..end as usize])
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

    //TODO current, non-endian implementations.
    make_number_methods! {
        /// Gets the numendlong endian `numname` at the [`Segment::current_offset`] without
        /// altering the [`Segment::current_offset`].
        pub fn current_numname_numend(&self) -> Result<_numname_> {
            let mut buf = [0; _numwidth_];
            self.bytes_at(self.current_offset(), &mut buf)?;
            Ok(_numname_::from_numend_bytes(buf))
        }
    }

    /// Gets the `u8` at the provided offset without altering the [`Segment::current_offset`].
    pub fn u8_at(&self, offset: u64) -> Result<u8> {
        self.validate_offset(offset, 0)?;
        Ok(self.as_ref()[(offset - self.initial_offset()) as usize])
    }

    make_number_methods! {
        /// Gets the numendlong endian `numname` at the provided offset without altering the
        /// [`Segment::current_offset`].
        pub fn numname_numend_at(&self, offset: u64) -> Result<_numname_> {
            let mut buf = [0; _numwidth_];
            self.bytes_at(offset, &mut buf)?;
            Ok(_numname_::from_numend_bytes(buf))
        }
    }

    /// Gets the `u16` using the default endidness at the provided offset without altering the
    /// [`Segment::current_offset`]. If the current endidness is [`Endidness::Unknown`], then an
    /// error is returned.
    pub fn u16_at(&self, offset: u64) -> Result<u16> {
        match self.endidness() {
            Endidness::Big => self.u16_be_at(offset),
            Endidness::Little => self.u16_le_at(offset),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `u32` using the default endidness at the provided offset without altering the
    /// [`Segment::current_offset`]. If the current endidness is [`Endidness::Unknown`], then an
    /// error is returned.
    pub fn u32_at(&self, offset: u64) -> Result<u32> {
        match self.endidness() {
            Endidness::Big => self.u32_be_at(offset),
            Endidness::Little => self.u32_le_at(offset),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `u64` using the default endidness at the provided offset without altering the
    /// [`Segment::current_offset`]. If the current endidness is [`Endidness::Unknown`], then an
    /// error is returned.
    pub fn u64_at(&self, offset: u64) -> Result<u64> {
        match self.endidness() {
            Endidness::Big => self.u64_be_at(offset),
            Endidness::Little => self.u64_le_at(offset),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `u128` using the default endidness at the provided offset without altering the
    /// [`Segment::current_offset`]. If the current endidness is [`Endidness::Unknown`], then an
    /// error is returned.
    pub fn u128_at(&self, offset: u64) -> Result<u128> {
        match self.endidness() {
            Endidness::Big => self.u128_be_at(offset),
            Endidness::Little => self.u128_le_at(offset),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `i16` using the default endidness at the provided offset without altering the
    /// [`Segment::current_offset`]. If the current endidness is [`Endidness::Unknown`], then an
    /// error is returned.
    pub fn i16_at(&self, offset: u64) -> Result<i16> {
        match self.endidness() {
            Endidness::Big => self.i16_be_at(offset),
            Endidness::Little => self.i16_le_at(offset),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `i32` using the default endidness at the provided offset without altering the
    /// [`Segment::current_offset`]. If the current endidness is [`Endidness::Unknown`], then an
    /// error is returned.
    pub fn i32_at(&self, offset: u64) -> Result<i32> {
        match self.endidness() {
            Endidness::Big => self.i32_be_at(offset),
            Endidness::Little => self.i32_le_at(offset),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `i64` using the default endidness at the provided offset without altering the
    /// [`Segment::current_offset`]. If the current endidness is [`Endidness::Unknown`], then an
    /// error is returned.
    pub fn i64_at(&self, offset: u64) -> Result<i64> {
        match self.endidness() {
            Endidness::Big => self.i64_be_at(offset),
            Endidness::Little => self.i64_le_at(offset),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `i128` using the default endidness at the provided offset without altering the
    /// [`Segment::current_offset`]. If the current endidness is [`Endidness::Unknown`], then an
    /// error is returned.
    pub fn i128_at(&self, offset: u64) -> Result<i128> {
        match self.endidness() {
            Endidness::Big => self.i128_be_at(offset),
            Endidness::Little => self.i128_le_at(offset),
            Endidness::Unknown => Err(Error::UnknownEndidness),
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

    /// Gets the `u16` using the default endidness at the [`Segment::current_offset`] and then
    /// advances it by `1`. If the current endidness is [`Endidness::Unknown`], then an error is
    /// returned.
    pub fn next_u16(&self) -> Result<u16> {
        match self.endidness() {
            Endidness::Big => self.next_u16_be(),
            Endidness::Little => self.next_u16_le(),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `u16` using the default endidness at the [`Segment::current_offset`] and then
    /// advances it by `1`. If the current endidness is [`Endidness::Unknown`], then an error is
    /// returned.
    pub fn next_u32(&self) -> Result<u32> {
        match self.endidness() {
            Endidness::Big => self.next_u32_be(),
            Endidness::Little => self.next_u32_le(),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `u16` using the default endidness at the [`Segment::current_offset`] and then
    /// advances it by `1`. If the current endidness is [`Endidness::Unknown`], then an error is
    /// returned.
    pub fn next_u64(&self) -> Result<u64> {
        match self.endidness() {
            Endidness::Big => self.next_u64_be(),
            Endidness::Little => self.next_u64_le(),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `u16` using the default endidness at the [`Segment::current_offset`] and then
    /// advances it by `1`. If the current endidness is [`Endidness::Unknown`], then an error is
    /// returned.
    pub fn next_u128(&self) -> Result<u128> {
        match self.endidness() {
            Endidness::Big => self.next_u128_be(),
            Endidness::Little => self.next_u128_le(),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `i8` at the [`Segment::current_offset`] and then advances it by `1`. If the
    /// current endidness is [`Endidness::Unknown`], then an error is returned.
    pub fn next_i8(&self) -> Result<i8> {
        let mut buf = [0; 1];
        self.next_bytes(&mut buf)?;
        Ok(i8::from_be_bytes(buf))
    }

    /// Gets the `i16` using the default endidness at the [`Segment::current_offset`] and then
    /// advances it by `1`. If the current endidness is [`Endidness::Unknown`], then an error is
    /// returned.
    pub fn next_i16(&self) -> Result<i16> {
        match self.endidness() {
            Endidness::Big => self.next_i16_be(),
            Endidness::Little => self.next_i16_le(),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `i32` using the default endidness at the [`Segment::current_offset`] and then
    /// advances it by `1`. If the current endidness is [`Endidness::Unknown`], then an error is
    /// returned.
    pub fn next_i32(&self) -> Result<i32> {
        match self.endidness() {
            Endidness::Big => self.next_i32_be(),
            Endidness::Little => self.next_i32_le(),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `i64` using the default endidness at the [`Segment::current_offset`] and then
    /// advances it by `1`. If the current endidness is [`Endidness::Unknown`], then an error is
    /// returned.
    pub fn next_i64(&self) -> Result<i64> {
        match self.endidness() {
            Endidness::Big => self.next_i64_be(),
            Endidness::Little => self.next_i64_le(),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `i128` using the default endidness at the [`Segment::current_offset`] and then
    /// advances it by `1`. If the current endidness is [`Endidness::Unknown`], then an error is
    /// returned.
    pub fn next_i128(&self) -> Result<i128> {
        match self.endidness() {
            Endidness::Big => self.next_i128_be(),
            Endidness::Little => self.next_i128_le(),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    #[cfg(feature = "async")]
    pub fn async_segment_next_n(&self, num_bytes: u64) -> Result<AsyncSegment> {
        let start = self.current_offset();
        self.async_segment(start, start + num_bytes)
    }

    pub fn segment_next_n(&self, num_bytes: u64) -> Result<Segment> {
        let start = self.current_offset();
        self.segment(start, start + num_bytes)
    }

    #[cfg(feature = "async")]
    pub fn async_segment(&self, start: u64, end: u64) -> Result<AsyncSegment> {
        self.validate_offset(start, (end - start) as usize)?;
        Ok(AsyncSegment::from_slice_with_offset(
            &self.data
                [(start - self.initial_offset) as usize..(end - self.initial_offset) as usize],
            start,
            self.endidness,
        ))
    }

    pub fn segment(&self, start: u64, end: u64) -> Result<Segment> {
        self.validate_offset(start, (end - start) as usize)?;
        Ok(Segment::from_slice_with_offset(
            &self.data
                [(start - self.initial_offset) as usize..(end - self.initial_offset) as usize],
            start,
            self.endidness,
        ))
    }
}

impl<'r> AsRef<[u8]> for Segment<'r> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.data
    }
}

impl<'r> io::Read for Segment<'r> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.remaining() > buf.len() as u64 {
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
impl<'r> Borrow<[u8]> for Segment<'r> {
    fn borrow(&self) -> &[u8] {
        self.as_ref()
    }
}
impl<'r> io::Seek for Segment<'r> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        match pos {
            io::SeekFrom::Start(to) => self.move_to(to)?,
            io::SeekFrom::Current(by) => {
                self.move_to((self.current_offset() as i128 + by as i128) as u64)?
            }
            io::SeekFrom::End(point) => {
                self.move_to((self.upper_offset_limit() as i128 - point as i128) as u64)?
            }
        };
        Ok(self.current_offset() as u64)
    }
}

impl<'r> io::BufRead for Segment<'r> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        if self.remaining() >= 4096 {
            Ok(self.subseq(self.current_offset(), 4096)?)
        } else {
            Ok(self.subseq(self.current_offset(), self.remaining() as usize)?)
        }
    }
    fn consume(&mut self, amt: usize) {
        if !self.is_empty() {
            if self.remaining() < amt as u64 {
                self.move_offset(self.remaining() as i128).unwrap();
            } else {
                self.move_offset(amt as i128).unwrap();
            }
        }
    }
}

#[cfg(feature = "async")]
impl<'r> From<AsyncSegment<'r>> for Segment<'r> {
    fn from(other: AsyncSegment<'r>) -> Self {
        let seg = Segment::new(
            other.data,
            other.initial_offset(),
            other.endidness.into_inner(),
        );
        seg.set_pos(other.position.into_inner());
        seg
    }
}

impl<'r> Clone for Segment<'r> {
    #[cfg(feature = "thread-safe")]
    fn clone(&self) -> Self {
        Self {
            initial_offset: self.initial_offset,
            position: RwLock::new(self.get_pos()),
            data: self.data,
            endidness: self.endidness,
        }
    }
    #[cfg(not(feature = "thread-safe"))]
    fn clone(&self) -> Self {
        Self {
            initial_offset: self.initial_offset,
            position: Cell::new(self.get_pos()),
            data: self.data,
            endidness: self.endidness,
        }
    }
}
