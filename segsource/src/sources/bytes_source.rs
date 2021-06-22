#[cfg(feature = "async")]
use crate::segment::AsyncSegment;
use crate::{Endidness, Result, Segment, Source};
use bytes::{BufMut as _, Bytes, BytesMut};
use std::{fs, io, path::Path};

pub struct BytesSource {
    initial_offset: u64,
    data: Bytes,
    endidness: Endidness,
}

impl BytesSource {
    #[inline]
    fn new(data: Bytes, initial_offset: u64, endidness: Endidness) -> Self {
        Self {
            initial_offset,
            data,
            endidness,
        }
    }
}

impl Source for BytesSource {
    #[inline]
    fn from_file_with_offset<P: AsRef<Path>>(
        path: P,
        initial_offset: u64,
        endidness: Endidness,
    ) -> Result<Self> {
        Ok(Self::new(bytes_from_file(path)?, initial_offset, endidness))
    }

    #[inline]
    fn from_bytes_with_offset(
        bytes: Bytes,
        initial_offset: u64,
        endidness: Endidness,
    ) -> Result<Self> {
        Ok(Self::new(bytes, initial_offset, endidness))
    }

    #[inline]
    fn initial_offset(&self) -> u64 {
        self.initial_offset
    }

    #[inline]
    fn size(&self) -> u64 {
        self.data.len() as u64
    }

    #[inline]
    fn endidness(&self) -> Endidness {
        self.endidness
    }

    #[inline]
    fn change_endidness(&mut self, endidness: Endidness) {
        self.endidness = endidness
    }

    fn segment(&self, start: u64, end: u64) -> Result<Segment> {
        self.validate_offset(start)?;
        self.validate_offset(end)?;
        Ok(Segment::from_slice_with_offset(
            &self.data
                [(start - self.initial_offset) as usize..(end - self.initial_offset) as usize],
            start,
            self.endidness,
        ))
    }

    #[cfg(feature = "async")]
    fn async_segment(&self, start: u64, end: u64) -> Result<AsyncSegment> {
        self.validate_offset(start)?;
        self.validate_offset(end)?;
        Ok(AsyncSegment::from_slice_with_offset(
            &self.data
                [(start - self.initial_offset) as usize..(end - self.initial_offset) as usize],
            start,
            self.endidness,
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
