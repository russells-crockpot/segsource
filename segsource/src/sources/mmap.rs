use crate::{Endidness, Result, Segment, Source, U8Source};
#[cfg(feature = "bytes")]
use bytes::Bytes;
use fs3::FileExt as _;
use memmap2::Mmap;
#[cfg(feature = "bytes")]
use memmap2::MmapMut;
use std::{fs::File, path::Path};

pub struct MappedFileSource {
    initial_offset: usize,
    data: Mmap,
    endidness: Endidness,
    maybe_mapped_file: Option<File>,
}

impl MappedFileSource {
    #[inline]
    fn new(
        initial_offset: usize,
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
    type Item = u8;

    #[inline]
    fn size(&self) -> usize {
        self.data.len() as usize
    }

    #[inline]
    fn initial_offset(&self) -> usize {
        self.initial_offset
    }

    #[inline]
    fn from_vec_with_offset(items: Vec<Self::Item>, initial_offset: usize) -> Result<Self> {
        Self::from_u8_slice_with_offset(&items, initial_offset, Endidness::Unknown)
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

impl U8Source for MappedFileSource {
    fn from_file_with_offset<P: AsRef<Path>>(
        path: P,
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self> {
        let file = File::open(path)?;
        file.try_lock_shared()?;
        let mmap = unsafe { Mmap::map(&file)? };
        Ok(Self::new(initial_offset, mmap, endidness, Some(file)))
    }

    #[cfg(feature = "bytes")]
    #[inline]
    fn from_bytes_with_offset(
        bytes: Bytes,
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self> {
        Self::from_u8_slice_with_offset(&bytes, initial_offset, endidness)
    }

    fn from_u8_slice_with_offset(
        bytes: &[u8],
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self> {
        let mut mmap_mut = MmapMut::map_anon(bytes.len())?;
        mmap_mut.copy_from_slice(bytes);
        Ok(Self::new(
            initial_offset,
            mmap_mut.make_read_only()?,
            endidness,
            None,
        ))
    }

    #[inline]
    fn endidness(&self) -> Endidness {
        self.endidness
    }

    #[inline]
    fn change_endidness(&mut self, endidness: Endidness) {
        self.endidness = endidness
    }
}

impl Drop for MappedFileSource {
    fn drop(&mut self) {
        if let Some(file) = &self.maybe_mapped_file {
            file.unlock().unwrap();
        }
    }
}
