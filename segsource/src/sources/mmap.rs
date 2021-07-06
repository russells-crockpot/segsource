use crate::{Endidness, Result, Segment, Source, U8Source};
#[cfg(feature = "with_bytes")]
use bytes::Bytes;
use fs3::FileExt as _;
use memmap2::{Mmap, MmapMut};
use std::{
    fs::File,
    path::{Path, PathBuf},
};
#[cfg(feature = "async")]
use tokio::task::spawn_blocking;

#[cfg(feature = "async")]
use async_trait::async_trait;

/// A [`U8Source`] whose data is owned by a memory mapped file. This source can only use `u8`s as
/// its item.
///
/// An important note: The mapped file is locked first via an advisory lock. This is to prevent
/// changes to the file while it is mapped. However, this is just an **advisory** lock, and other
/// processes may choose to ignore it. So, it's best not to alter it while it's mapped.
pub struct MappedFileSource {
    initial_offset: usize,
    data: Mmap,
    endidness: Endidness,
    maybe_mapped_file: Option<File>,
    maybe_path: Option<PathBuf>,
}

impl MappedFileSource {
    #[inline]
    fn new(
        initial_offset: usize,
        data: Mmap,
        endidness: Endidness,
        maybe_mapped_file: Option<File>,
        maybe_path: Option<PathBuf>,
    ) -> Self {
        Self {
            initial_offset,
            data,
            endidness,
            maybe_mapped_file,
            maybe_path,
        }
    }

    pub fn path(&self) -> Option<&Path> {
        self.maybe_path.as_ref().map(|p| p.as_ref())
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
        Self::from_u8_slice_with_offset(&items, initial_offset, Endidness::default())
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
impl U8Source for MappedFileSource {
    fn from_file_with_offset<P: AsRef<Path>>(
        path: P,
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self> {
        let file = File::open(&path)?;
        file.try_lock_shared()?;
        let mmap = unsafe { Mmap::map(&file)? };
        Ok(Self::new(
            initial_offset,
            mmap,
            endidness,
            Some(file),
            Some(path.as_ref().to_path_buf()),
        ))
    }

    #[cfg(feature = "with_bytes")]
    #[inline]
    fn from_bytes_with_offset(
        bytes: Bytes,
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self> {
        Self::from_u8_slice_with_offset(&bytes, initial_offset, endidness)
    }

    #[cfg(feature = "async")]
    #[inline]
    async fn from_file_with_offset_async<P>(
        path: P,
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self>
    where
        P: AsRef<Path> + Sync + Send,
    {
        let path = path.as_ref().to_path_buf();
        spawn_blocking(move || Self::from_file_with_offset(path, initial_offset, endidness))
            .await
            .unwrap()
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

    //#[inline]
    //fn u8_segment(&self, start: usize, end: usize) -> Result<Segment<u8>> {
    //Source::segment(self, start, end)
    //}
}

impl Drop for MappedFileSource {
    fn drop(&mut self) {
        if let Some(file) = &self.maybe_mapped_file {
            file.unlock().unwrap();
        }
    }
}
