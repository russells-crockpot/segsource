#[cfg(feature = "async")]
use crate::sync::async_u8_vec_from_file;
use crate::{Endidness, Result, Segment, Source, U8Source};
#[cfg(feature = "bytes")]
use bytes::Bytes;
use std::{fs, io::Read as _, path::Path};

#[cfg(feature = "async")]
use async_trait::async_trait;

/// A [`Source`] that uses a `Vec` to store its data.
pub struct VecSource<I: Sync + Send> {
    initial_offset: usize,
    data: Vec<I>,
    endidness: Endidness,
}

impl<I: Sync + Send> VecSource<I> {
    #[inline]
    fn new(data: Vec<I>, initial_offset: usize, endidness: Endidness) -> Self {
        Self {
            initial_offset,
            data,
            endidness,
        }
    }
}

impl<I: Sync + Send> Source for VecSource<I> {
    type Item = I;

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
        Ok(Self {
            initial_offset,
            data: items,
            endidness: Endidness::default(),
        })
    }

    fn segment(&self, start: usize, end: usize) -> Result<Segment<I>> {
        self.validate_offset(start)?;
        self.validate_offset(end)?;
        Ok(Segment::with_offset(
            &self.data
                [(start - self.initial_offset) as usize..(end - self.initial_offset) as usize],
            start,
        ))
    }
}

#[cfg_attr(feature = "async", async_trait)]
impl U8Source for VecSource<u8> {
    #[inline]
    fn endidness(&self) -> Endidness {
        self.endidness
    }

    #[inline]
    fn change_endidness(&mut self, endidness: Endidness) {
        self.endidness = endidness;
    }

    #[cfg(feature = "async")]
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
    fn from_file_with_offset<P: AsRef<Path>>(
        path: P,
        initial_offset: usize,
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
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self> {
        Ok(Self::new(
            bytes.into_iter().collect(),
            initial_offset,
            endidness,
        ))
    }

    #[inline]
    fn from_u8_vec_with_offset(
        items: Vec<u8>,
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self> {
        Ok(Self::new(items, initial_offset, endidness))
    }

    fn from_u8_slice_with_offset(
        items: &[u8],
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self> {
        Ok(Self::new(Vec::from(items), initial_offset, endidness))
    }

    //fn u8_segment(&self, start: usize, end: usize) -> Result<Segment<u8>> {
    //self.validate_offset(start)?;
    //self.validate_offset(end)?;
    //Ok(Segment::with_offset_and_endidness(
    //&self.data
    //[(start - self.initial_offset) as usize..(end - self.initial_offset) as usize],
    //start,
    //self.endidness,
    //))
    //}
}
