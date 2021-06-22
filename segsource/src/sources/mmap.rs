#[cfg(feature = "async")]
use crate::segment::AsyncSegment;
use crate::{Endidness, Result, Segment, Source};
#[cfg(feature = "bytes")]
use bytes::Bytes;
use fs3::FileExt as _;
use memmap2::Mmap;
#[cfg(feature = "bytes")]
use memmap2::MmapMut;
use std::{fs::File, path::Path};

pub struct MappedFileSource {
    initial_offset: u64,
    data: Mmap,
    endidness: Endidness,
    maybe_mapped_file: Option<File>,
}

impl MappedFileSource {
    fn new(
        initial_offset: u64,
        data: Mmap,
        endidness: Endidness,
        maybe_mapped_file: Option<File>,
    ) -> Self {
        Self {
            initial_offset,
            data,
            endidness,
            maybe_mapped_file,
        }
    }
}

impl Source for MappedFileSource {
    fn from_file_with_offset<P: AsRef<Path>>(
        path: P,
        initial_offset: u64,
        endidness: Endidness,
    ) -> Result<Self> {
        let file = File::open(path)?;
        file.try_lock_shared()?;
        let mmap = unsafe { Mmap::map(&file)? };
        Ok(Self::new(initial_offset, mmap, endidness, Some(file)))
    }

    #[cfg(feature = "bytes")]
    fn from_bytes_with_offset(
        bytes: Bytes,
        initial_offset: u64,
        endidness: Endidness,
    ) -> Result<Self> {
        let mut mmap_mut = MmapMut::map_anon(bytes.len())?;
        mmap_mut.copy_from_slice(&bytes);
        Ok(Self::new(
            initial_offset,
            mmap_mut.make_read_only()?,
            endidness,
            None,
        ))
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

    #[inline]
    fn initial_offset(&self) -> u64 {
        self.initial_offset
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

impl Drop for MappedFileSource {
    fn drop(&mut self) {
        if let Some(file) = &self.maybe_mapped_file {
            file.unlock().unwrap();
        }
    }
}
