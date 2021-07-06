/// WIP which I'm not even sure can be done in Rust. Or the very least, done *well*...
use crate::{Result, Segment, Source};
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
use core::{marker::Unpin, mem::ManuallyDrop, ops::Deref, pin::Pin, ptr, sync::Arc};

struct PinnableSource<S: Source + Unpin>(S);
impl<S: Source + Unpin> Deref for PinnableSource<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct SegmentRefableSource<'s, S, I>
where
    S: Source<Item = I> + Unpin,
    Self: 's,
{
    //source: Pin<PinnableSource<S>>,
    segment: Segment<'s, I>,
    //segment: *const Segment<'s, I>,
    source: ManuallyDrop<S>,
}

impl<'s, S, I> SegmentRefableSource<'s, S, I>
where
    S: Source<Item = I> + Unpin,
    Self: 's,
{
    pub unsafe fn new(inner: S) -> Result<Self> {
        let source = ManuallyDrop::new(inner);
        let segment = source.all()?;
        Ok(Self { source, segment })
    }
}
