#[cfg(all(feature = "async", feature = "with_bytes"))]
use crate::sync::async_bytes_from_file;
#[cfg(all(feature = "async", not(feature = "with_bytes")))]
use crate::sync::async_u8_vec_from_file;
#[cfg(feature = "async")]
use crate::AsyncU8Source;
use crate::{Endidness, Result, Segment, Source, U8Source};
use bytes::{BufMut as _, Bytes, BytesMut};
use std::{fs, io, path::Path};

#[cfg(feature = "async")]
use async_trait::async_trait;

#[derive(Clone)]
/// A [`U8Source`] that uses a `Bytes` object from the wonderful `bytes` crate to store its
/// underlying data. This source can only use `u8`s as its item.
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

    add_basic_source_items! {@add_u8_items}
}

impl U8Source for BytesSource {
    #[inline]
    fn from_file_with_offset<P: AsRef<Path>>(
        path: P,
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self> {
        Ok(Self::new(bytes_from_file(path)?, initial_offset, endidness))
    }

    #[inline]
    fn from_bytes_with_offset(
        bytes: Bytes,
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self> {
        Ok(Self::new(bytes, initial_offset, endidness))
    }
    impl_endidness_items! {}

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

    //#[inline]
    //fn u8_segment(&self, start: usize, end: usize) -> Result<Segment<u8>> {
    //Source::segment(self, start, end)
    //}
}

#[cfg(feature = "async")]
#[async_trait]
impl AsyncU8Source for BytesSource {
    #[cfg(all(feature = "async", feature = "with_bytes"))]
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

    #[cfg(all(feature = "async", not(feature = "with_bytes")))]
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
