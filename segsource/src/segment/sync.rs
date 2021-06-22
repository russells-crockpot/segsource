use super::Segment;
use crate::{
    error::{Error, Result},
    Endidness,
};
use std::borrow::Borrow;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use segsource_macros::make_number_methods;

pub struct AsyncSegment<'r> {
    initial_offset: u64,
    pub(crate) data: &'r [u8],
    pub(crate) endidness: RwLock<Endidness>,
    pub(crate) position: RwLock<usize>,
}

impl<'r> AsyncSegment<'r> {
    fn new(data: &'r [u8], initial_offset: u64, endidness: Endidness) -> Self {
        Self {
            initial_offset,
            position: RwLock::new(0),
            data,
            endidness: RwLock::new(endidness),
        }
    }

    #[inline]
    async fn get_pos_write<'a>(&'a self) -> RwLockWriteGuard<'a, usize> {
        self.position.write().await
    }

    #[inline]
    async fn get_pos_read<'a>(&'a self) -> RwLockReadGuard<'a, usize> {
        self.position.read().await
    }

    #[inline]
    async fn get_pos(&self) -> usize {
        *self.position.read().await
    }

    fn adj_pos_sync(&self, amt: i128, lock: &mut RwLockWriteGuard<usize>) {
        if amt > 0 {
            **lock += amt as usize;
        } else {
            **lock -= (-amt) as usize;
        }
    }

    /// Functions the same as the [`AsyncSegment::from_slice_with_offset`], except the initial offset
    /// is always `0`.
    pub fn from_slice(slice: &'r [u8], endidness: Endidness) -> Self {
        Self::from_slice_with_offset(slice, 0, endidness)
    }

    /// Generates a new [`AsyncSegment`] using the provided slice, initial offset, and endidness.
    pub fn from_slice_with_offset(
        slice: &'r [u8],
        initial_offset: u64,
        endidness: Endidness,
    ) -> Self {
        Self::new(slice, initial_offset, endidness)
    }

    #[inline]
    /// The initial offset of the [`AsyncSegment`]. For more information, see the **Offsets** section
    /// of the [`AsyncSegment`] documentation.
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
    pub async fn current_offset(&self) -> u64 {
        self.get_pos().await as u64 + self.initial_offset
    }

    #[inline]
    /// The endidness of the reader.
    pub async fn endidness(&self) -> Endidness {
        *self.endidness.read().await
    }

    #[inline]
    /// Changes the default endidness.
    pub async fn change_endidness(&mut self, endidness: Endidness) {
        let mut lock = self.endidness.write().await;
        *lock = endidness
    }

    /// Sets the reader's [`AsyncSegment::current_offset`].
    pub async fn move_to(&self, offset: u64) -> Result<()> {
        let mut lock = self.get_pos_write().await;
        self.move_to_sync(offset, &mut lock)
    }

    fn move_to_sync(&self, offset: u64, lock: &mut RwLockWriteGuard<usize>) -> Result<()> {
        self.validate_abs_offset(offset, 0)?;
        **lock = (offset - self.initial_offset) as usize;
        Ok(())
    }

    /// Alters the [`AsyncSegment::current_offset`] by the given amount.
    pub async fn move_offset(&self, num_bytes: i128) -> Result<()> {
        let mut lock = self.get_pos_write().await;
        self.move_offset_sync(num_bytes, &mut lock)
    }

    fn move_offset_sync(&self, num_bytes: i128, lock: &mut RwLockWriteGuard<usize>) -> Result<()> {
        let start = self.calc_current_offset(**lock) as i128 + num_bytes;
        self.validate_abs_offset(start as u64, 0)?;
        self.adj_pos_sync(num_bytes, lock);
        Ok(())
    }

    /// Gets the current byte and then advances the cursor.
    pub async fn next_u8(&self) -> Result<u8> {
        let mut lock = self.get_pos_write().await;
        self.next_u8_sync(&mut lock)
    }

    fn next_u8_sync(&self, lock: &mut RwLockWriteGuard<usize>) -> Result<u8> {
        self.validate_buf(1, **lock)?;
        self.adj_pos_sync(1, lock);
        Ok(self.data[**lock - 1])
    }

    /// Returns slice of the requested size containing the next n bytes (where n is
    /// the `num_bytes` parameter) and then advances the cursor by that much.
    pub async fn next_n_bytes(&self, num_bytes: usize) -> Result<&[u8]> {
        let mut lock = self.get_pos_write().await;
        let current_offset = self.calc_current_offset(*lock);
        self.validate_abs_offset(current_offset, num_bytes)?;
        let start = (current_offset - self.initial_offset()) as usize;
        let data = &self.data[start..start + num_bytes as usize];
        self.move_offset_sync(num_bytes as i128, &mut lock)?;
        Ok(data)
    }

    #[inline]
    /// Gets a pointer to a slice of the byte at the [`AsyncSegment::current_offset`], as well as all
    /// all bytes afterwards. This does not alter the [`AsyncSegment::current_offset`].
    pub async fn get_remaining(&self) -> Result<&[u8]> {
        self.range(self.current_offset().await, self.upper_offset_limit())
    }

    #[inline]
    /// The lowest valid offset that can be requested. By default, this is the same as
    /// [`AsyncSegment::initial_offset`].
    pub fn lower_offset_limit(&self) -> u64 {
        self.initial_offset
    }

    #[inline]
    /// The highest valid offset that can be requested. By default, this is the reader's
    /// [`AsyncSegment::size`] plus its [`AsyncSegment::initial_offset`].
    pub fn upper_offset_limit(&self) -> u64 {
        self.size() + self.initial_offset()
    }

    #[inline]
    /// Checks whether or not there is any data left, based off of the
    /// [`AsyncSegment::current_offset`].
    pub async fn is_empty(&self) -> bool {
        self.remaining().await == 0
    }

    #[inline]
    fn calc_current_offset(&self, pos: usize) -> u64 {
        pos as u64 + self.initial_offset
    }

    #[inline]
    fn calc_remaining(&self, pos: usize) -> u64 {
        self.upper_offset_limit() - self.calc_current_offset(pos)
    }

    #[inline]
    /// The amount of data left, based off of the [`AsyncSegment::current_offset`].
    pub async fn remaining(&self) -> u64 {
        self.calc_remaining(self.get_pos().await)
    }

    fn validate_buf(&self, size: usize, pos: usize) -> Result<()> {
        let offset = self.calc_current_offset(pos);
        if size > 0 && self.calc_remaining(pos) == 0 {
            Err(Error::NoMoreData)
        } else if offset < self.lower_offset_limit() {
            Err(Error::OffsetTooSmall(offset))
        } else if offset > self.upper_offset_limit() {
            Err(Error::OffsetTooLarge(offset))
        } else if offset > self.upper_offset_limit() - size as u64 {
            Err(Error::NotEnoughData(size, self.size() - offset))
        } else {
            Ok(())
        }
    }

    fn validate_abs_offset(&self, offset: u64, size: usize) -> Result<()> {
        if offset < self.lower_offset_limit() {
            Err(Error::OffsetTooSmall(offset))
        } else if offset > self.upper_offset_limit() {
            Err(Error::OffsetTooLarge(offset))
        } else if offset > self.upper_offset_limit() - size as u64 {
            Err(Error::NotEnoughData(size, self.size() - offset))
        } else {
            Ok(())
        }
    }

    /// Takes an absolute offset and converts it to a relative offset, based off of the
    /// [`AsyncSegment::current_offset`].
    pub async fn relative_offset(&self, abs_offset: u64) -> Result<u64> {
        let lock = self.get_pos_read().await;
        self.validate_abs_offset(abs_offset, 0)?;
        Ok(abs_offset - self.calc_current_offset(*lock))
    }

    /// Returns `true` if the next bytes are the same as the ones provided.
    pub async fn next_bytes_are(&self, prefix: &[u8]) -> Result<bool> {
        let current_offset = self.current_offset().await;
        self.validate_abs_offset(current_offset, prefix.len())?;
        let mut buf = Vec::with_capacity(prefix.len());
        (0..buf.len()).for_each(|_| buf.push(0));
        self.bytes_at(current_offset, &mut buf)?;
        Ok(prefix.iter().zip(buf.into_iter()).all(|(v1, v2)| *v1 == v2))
    }

    /// Fills the provided buffer with bytes, starting at the provided offset. This does not alter
    /// the [`AsyncSegment::current_offset`].
    pub fn bytes_at(&self, offset: u64, buf: &mut [u8]) -> Result<()> {
        self.validate_abs_offset(offset, buf.len())?;
        for i in 0..buf.len() {
            buf[i] = self.u8_at(offset + i as u64)?;
        }
        Ok(())
    }

    /// Returns a subsequence (i.e. a `&[u8]`) of data of the requested size beginning at the
    /// provided offset.
    pub fn subseq(&self, offset: u64, num_bytes: usize) -> Result<&[u8]> {
        self.validate_abs_offset(offset, num_bytes)?;
        self.range(offset, offset + num_bytes as u64)
    }

    /// Returns a slice of the data between the provided starting and ending offsets.
    pub fn range(&self, start: u64, end: u64) -> Result<&[u8]> {
        self.validate_abs_offset(start, (end - start) as usize)?;
        Ok(&self.data[start as usize..end as usize])
    }

    /// Fills the provided buffer with the next n bytes, where n is the length of the buffer. This
    /// then advances the [`AsyncSegment::current_offset`] by n.
    pub async fn next_bytes(&self, buf: &mut [u8]) -> Result<()> {
        let current_pos = self.get_pos().await;
        self.validate_buf(buf.len(), current_pos)?;
        buf.copy_from_slice(&self.data[current_pos..current_pos + buf.len()]);
        Ok(())
    }

    /// Gets the [`u8`] at the [`AsyncSegment::current_offset`] without altering the
    /// [`AsyncSegment::current_offset`].
    pub async fn current_u8(&self) -> Result<u8> {
        self.u8_at(self.current_offset().await)
    }

    //TODO current, non-endian implementations.
    make_number_methods! {
        /// Gets the numendlong endian `numname` at the [`AsyncSegment::current_offset`] without
        /// altering the [`AsyncSegment::current_offset`].
        pub async fn current_numname_numend(&self) -> Result<_numname_> {
            let mut buf = [0; _numwidth_];
            self.bytes_at(self.current_offset().await, &mut buf)?;
            Ok(_numname_::from_numend_bytes(buf))
        }
    }

    /// Gets the `u8` at the provided offset without altering the [`AsyncSegment::current_offset`].
    pub fn u8_at(&self, offset: u64) -> Result<u8> {
        self.validate_abs_offset(offset, 0)?;
        Ok(self.data[(offset - self.initial_offset()) as usize])
    }

    make_number_methods! {
        /// Gets the numendlong endian `numname` at the provided offset without altering the
        /// [`AsyncSegment::current_offset`].
        pub fn numname_numend_at(&self, offset: u64) -> Result<_numname_> {
            let mut buf = [0; _numwidth_];
            self.bytes_at(offset, &mut buf)?;
            Ok(_numname_::from_numend_bytes(buf))
        }
    }

    /// Gets the `u16` using the default endidness at the provided offset without altering the
    /// [`AsyncSegment::current_offset`]. If the current endidness is [`Endidness::Unknown`], then an
    /// error is returned.
    pub async fn u16_at(&self, offset: u64) -> Result<u16> {
        match self.endidness().await {
            Endidness::Big => self.u16_be_at(offset),
            Endidness::Little => self.u16_le_at(offset),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `u32` using the default endidness at the provided offset without altering the
    /// [`AsyncSegment::current_offset`]. If the current endidness is [`Endidness::Unknown`], then an
    /// error is returned.
    pub async fn u32_at(&self, offset: u64) -> Result<u32> {
        match self.endidness().await {
            Endidness::Big => self.u32_be_at(offset),
            Endidness::Little => self.u32_le_at(offset),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `u64` using the default endidness at the provided offset without altering the
    /// [`AsyncSegment::current_offset`]. If the current endidness is [`Endidness::Unknown`], then an
    /// error is returned.
    pub async fn u64_at(&self, offset: u64) -> Result<u64> {
        match self.endidness().await {
            Endidness::Big => self.u64_be_at(offset),
            Endidness::Little => self.u64_le_at(offset),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `u128` using the default endidness at the provided offset without altering the
    /// [`AsyncSegment::current_offset`]. If the current endidness is [`Endidness::Unknown`], then an
    /// error is returned.
    pub async fn u128_at(&self, offset: u64) -> Result<u128> {
        match self.endidness().await {
            Endidness::Big => self.u128_be_at(offset),
            Endidness::Little => self.u128_le_at(offset),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `i16` using the default endidness at the provided offset without altering the
    /// [`AsyncSegment::current_offset`]. If the current endidness is [`Endidness::Unknown`], then an
    /// error is returned.
    pub async fn i16_at(&self, offset: u64) -> Result<i16> {
        match self.endidness().await {
            Endidness::Big => self.i16_be_at(offset),
            Endidness::Little => self.i16_le_at(offset),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `i32` using the default endidness at the provided offset without altering the
    /// [`AsyncSegment::current_offset`]. If the current endidness is [`Endidness::Unknown`], then an
    /// error is returned.
    pub async fn i32_at(&self, offset: u64) -> Result<i32> {
        match self.endidness().await {
            Endidness::Big => self.i32_be_at(offset),
            Endidness::Little => self.i32_le_at(offset),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `i64` using the default endidness at the provided offset without altering the
    /// [`AsyncSegment::current_offset`]. If the current endidness is [`Endidness::Unknown`], then an
    /// error is returned.
    pub async fn i64_at(&self, offset: u64) -> Result<i64> {
        match self.endidness().await {
            Endidness::Big => self.i64_be_at(offset),
            Endidness::Little => self.i64_le_at(offset),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `i128` using the default endidness at the provided offset without altering the
    /// [`AsyncSegment::current_offset`]. If the current endidness is [`Endidness::Unknown`], then an
    /// error is returned.
    pub async fn i128_at(&self, offset: u64) -> Result<i128> {
        match self.endidness().await {
            Endidness::Big => self.i128_be_at(offset),
            Endidness::Little => self.i128_le_at(offset),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    make_number_methods! {
        /// Gets the numendlong endian `numname` at the [`AsyncSegment::current_offset`] and then
        /// advances it by `1`.
        pub async fn next_numname_numend(&self) -> Result<_numname_> {
            let mut buf = [0; _numwidth_];
            self.next_bytes(&mut buf).await?;
            Ok(_numname_::from_numend_bytes(buf))
        }
    }

    /// Gets the `u16` using the default endidness at the [`AsyncSegment::current_offset`] and then
    /// advances it by `1`. If the current endidness is [`Endidness::Unknown`], then an error is
    /// returned.
    pub async fn next_u16(&self) -> Result<u16> {
        match self.endidness().await {
            Endidness::Big => self.next_u16_be().await,
            Endidness::Little => self.next_u16_le().await,
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `u16` using the default endidness at the [`AsyncSegment::current_offset`] and then
    /// advances it by `1`. If the current endidness is [`Endidness::Unknown`], then an error is
    /// returned.
    pub async fn next_u32(&self) -> Result<u32> {
        match self.endidness().await {
            Endidness::Big => self.next_u32_be().await,
            Endidness::Little => self.next_u32_le().await,
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `u16` using the default endidness at the [`AsyncSegment::current_offset`] and then
    /// advances it by `1`. If the current endidness is [`Endidness::Unknown`], then an error is
    /// returned.
    pub async fn next_u64(&self) -> Result<u64> {
        match self.endidness().await {
            Endidness::Big => self.next_u64_be().await,
            Endidness::Little => self.next_u64_le().await,
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `u16` using the default endidness at the [`AsyncSegment::current_offset`] and then
    /// advances it by `1`. If the current endidness is [`Endidness::Unknown`], then an error is
    /// returned.
    pub async fn next_u128(&self) -> Result<u128> {
        match self.endidness().await {
            Endidness::Big => self.next_u128_be().await,
            Endidness::Little => self.next_u128_le().await,
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `i8` at the [`AsyncSegment::current_offset`] and then advances it by `1`. If the
    /// current endidness is [`Endidness::Unknown`], then an error is returned.
    pub async fn next_i8(&self) -> Result<i8> {
        let mut buf = [0; 1];
        self.next_bytes(&mut buf).await?;
        Ok(i8::from_be_bytes(buf))
    }

    /// Gets the `i16` using the default endidness at the [`AsyncSegment::current_offset`] and then
    /// advances it by `1`. If the current endidness is [`Endidness::Unknown`], then an error is
    /// returned.
    pub async fn next_i16(&self) -> Result<i16> {
        match self.endidness().await {
            Endidness::Big => self.next_i16_be().await,
            Endidness::Little => self.next_i16_le().await,
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `i32` using the default endidness at the [`AsyncSegment::current_offset`] and then
    /// advances it by `1`. If the current endidness is [`Endidness::Unknown`], then an error is
    /// returned.
    pub async fn next_i32(&self) -> Result<i32> {
        match self.endidness().await {
            Endidness::Big => self.next_i32_be().await,
            Endidness::Little => self.next_i32_le().await,
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `i64` using the default endidness at the [`AsyncSegment::current_offset`] and then
    /// advances it by `1`. If the current endidness is [`Endidness::Unknown`], then an error is
    /// returned.
    pub async fn next_i64(&self) -> Result<i64> {
        match self.endidness().await {
            Endidness::Big => self.next_i64_be().await,
            Endidness::Little => self.next_i64_le().await,
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `i128` using the default endidness at the [`AsyncSegment::current_offset`] and then
    /// advances it by `1`. If the current endidness is [`Endidness::Unknown`], then an error is
    /// returned.
    pub async fn next_i128(&self) -> Result<i128> {
        match self.endidness().await {
            Endidness::Big => self.next_i128_be().await,
            Endidness::Little => self.next_i128_le().await,
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    pub async fn segment_next_n<'a>(&'a self, num_bytes: u64) -> Result<Segment<'a>> {
        let mut lock = self.get_pos_write().await;
        self.validate_buf(num_bytes as usize, *lock)?;
        let seg = Segment::from_slice_with_offset(
            &self.data[*lock..*lock + num_bytes as usize],
            self.calc_current_offset(*lock),
            self.endidness().await,
        );
        self.adj_pos_sync(num_bytes as i128, &mut lock);
        Ok(seg)
    }

    pub async fn async_segment_next_n<'a>(&'a self, num_bytes: u64) -> Result<AsyncSegment<'a>> {
        let mut lock = self.get_pos_write().await;
        self.validate_buf(num_bytes as usize, *lock)?;
        let seg = AsyncSegment::from_slice_with_offset(
            &self.data[*lock..*lock + num_bytes as usize],
            self.calc_current_offset(*lock),
            self.endidness().await,
        );
        self.adj_pos_sync(num_bytes as i128, &mut lock);
        Ok(seg)
    }

    pub async fn segment<'a>(&'a self, start: u64, end: u64) -> Result<Segment<'a>> {
        self.validate_abs_offset(start, (end - start) as usize)?;
        Ok(Segment::from_slice_with_offset(
            &self.data
                [(start - self.initial_offset) as usize..(end - self.initial_offset) as usize],
            start,
            self.endidness().await,
        ))
    }

    pub async fn async_segment<'a>(&'a self, start: u64, end: u64) -> Result<AsyncSegment<'a>> {
        self.validate_abs_offset(start, (end - start) as usize)?;
        Ok(AsyncSegment::from_slice_with_offset(
            &self.data
                [(start - self.initial_offset) as usize..(end - self.initial_offset) as usize],
            start,
            self.endidness().await,
        ))
    }
}

impl<'r> AsRef<[u8]> for AsyncSegment<'r> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.data
    }
}

impl<'r> Borrow<[u8]> for AsyncSegment<'r> {
    fn borrow(&self) -> &[u8] {
        self.as_ref()
    }
}

impl<'r> From<Segment<'r>> for AsyncSegment<'r> {
    fn from(other: Segment<'r>) -> Self {
        Self {
            initial_offset: other.initial_offset,
            #[cfg(feature = "thread-safe")]
            position: RwLock::new(other.position.into_inner().unwrap()),
            #[cfg(not(feature = "thread-safe"))]
            position: RwLock::new(other.position.into_inner()),
            data: other.data,
            endidness: RwLock::new(other.endidness),
        }
    }
}
