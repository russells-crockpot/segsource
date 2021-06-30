#[cfg(all(feature = "async", feature = "bytes"))]
use crate::util::async_bytes_from_file;
#[cfg(all(feature = "async", not(feature = "bytes")))]
use crate::util::async_u8_vec_from_file;
use crate::{Endidness, Result, Segment, Source, U8Source};
use bytes::{BufMut as _, Bytes, BytesMut};
use std::{fs, io, path::Path};

#[cfg(feature = "async")]
use async_trait::async_trait;

#[derive(Clone)]
pub struct BytesSource {
    initial_offset: usize,
    data: Bytes,
    endidness: Endidness,
}

impl BytesSource {
    #[inline]
    fn new(data: Bytes, initial_offset: usize, endidness: Endidness) -> Self {
        Self {
            initial_offset,
            data,
            endidness,
        }
    }
}

impl Source for BytesSource {
    type Item = u8;

    #[inline]
    fn initial_offset(&self) -> usize {
        self.initial_offset
    }

    #[inline]
    fn size(&self) -> usize {
        self.data.len() as usize
    }

    #[inline]
    fn from_vec_with_offset(items: Vec<Self::Item>, initial_offset: usize) -> Result<Self> {
        Self::from_u8_vec_with_offset(items, initial_offset, Endidness::Unknown)
    }

    fn segment(&self, start: usize, end: usize) -> Result<Segment<u8>> {
        self.validate_offset(start)?;
        self.validate_offset(end)?;
        Ok(Segment::with_offset_and_endidness(
            &self.data
                [(start - self.initial_offset) as usize..(end - self.initial_offset) as usize],
            start,
            self.endidness,
        ))
    }
}

#[cfg_attr(feature = "async", async_trait)]
impl U8Source for BytesSource {
    #[inline]
    fn from_file_with_offset<P: AsRef<Path>>(
        path: P,
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self> {
        Ok(Self::new(bytes_from_file(path)?, initial_offset, endidness))
    }

    #[cfg(all(feature = "async", feature = "bytes"))]
    async fn from_file_with_offset_async<P>(
        path: P,
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self>
    where
        P: AsRef<Path> + Sync + Send,
    {
        Ok(Self::new(
            async_bytes_from_file(path).await?,
            initial_offset,
            endidness,
        ))
    }

    #[cfg(all(feature = "async", not(feature = "bytes")))]
    async fn from_file_with_offset_async<P>(
        path: P,
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self>
    where
        P: AsRef<Path> + Sync + Send,
    {
        Ok(Self::new(
            async_u8_vec_from_file(path).await?,
            initial_offset,
            endidness,
        ))
    }

    #[inline]
    fn from_bytes_with_offset(
        bytes: Bytes,
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self> {
        Ok(Self::new(bytes, initial_offset, endidness))
    }

    #[inline]
    fn endidness(&self) -> Endidness {
        self.endidness
    }

    #[inline]
    fn change_endidness(&mut self, endidness: Endidness) {
        self.endidness = endidness
    }

    #[inline]
    fn from_u8_vec_with_offset(
        items: Vec<u8>,
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self> {
        Ok(Self::new(Bytes::from(items), initial_offset, endidness))
    }

    #[inline]
    fn from_u8_slice_with_offset(
        items: &[u8],
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self> {
        Ok(Self::new(
            Bytes::copy_from_slice(items),
            initial_offset,
            endidness,
        ))
    }

    #[inline]
    fn u8_segment(&self, start: usize, end: usize) -> Result<Segment<u8>> {
        Source::segment(self, start, end)
    }
}

fn bytes_from_file<P: AsRef<Path>>(path: P) -> io::Result<Bytes> {
    let capacity = fs::metadata(&path)?.len();
    let file = fs::File::open(path)?;
    bytes_from_bufread(io::BufReader::new(file), Some(capacity as usize))
}

fn bytes_from_bufread<R: io::BufRead>(mut reader: R, capacity: Option<usize>) -> io::Result<Bytes> {
    let mut bytes_mut = if let Some(size) = capacity {
        BytesMut::with_capacity(size)
    } else {
        BytesMut::new()
    };
    loop {
        let buf_len = {
            let buf = reader.fill_buf()?;
            if buf.is_empty() {
                break Ok(Bytes::from(bytes_mut));
            }
            bytes_mut.put_slice(buf);
            buf.len()
        };
        reader.consume(buf_len);
    }
}

impl From<BytesSource> for Bytes {
    fn from(src: BytesSource) -> Bytes {
        src.data
    }
}
