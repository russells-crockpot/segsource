#[cfg(feature = "async")]
use crate::segment::AsyncSegment;
use crate::{Endidness, Result, Segment, Source};
#[cfg(feature = "bytes")]
use bytes::Bytes;
use std::{fs, io::Read as _, path::Path};

pub struct VecSource {
    initial_offset: u64,
    data: Vec<u8>,
    endidness: Endidness,
}

impl VecSource {
    #[inline]
    fn new(data: Vec<u8>, initial_offset: u64, endidness: Endidness) -> Self {
        Self {
            initial_offset,
            data,
            endidness,
        }
    }
}

impl Source for VecSource {
    #[inline]
    fn from_file_with_offset<P: AsRef<Path>>(
        path: P,
        initial_offset: u64,
        endidness: Endidness,
    ) -> Result<Self> {
        let md = path.as_ref().metadata()?;
        let mut data = Vec::with_capacity(md.len() as usize);
        {
            let mut file = fs::File::open(path)?;
            file.read_to_end(&mut data)?;
        }
        Ok(Self::new(data, initial_offset, endidness))
    }

    #[inline]
    #[cfg(feature = "bytes")]
    fn from_bytes_with_offset(
        bytes: Bytes,
        initial_offset: u64,
        endidness: Endidness,
    ) -> Result<Self> {
        Ok(Self::new(
            bytes.into_iter().collect(),
            initial_offset,
            endidness,
        ))
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
