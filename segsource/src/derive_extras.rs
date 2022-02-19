#![allow(unused_imports, unused_variables, unused_mut)]
use crate::segment::Segment;
#[cfg(not(feature = "std"))]
use alloc::vec::{IntoIter as VecIter, Vec};
use core::{
    convert::{From, TryFrom},
    fmt::Debug,
    iter::{FromIterator, IntoIterator, Iterator},
    marker::PhantomData,
    result::Result,
};
#[cfg(feature = "std")]
use std::vec::IntoIter as VecIter;

//TODO
#[allow(clippy::needless_collect)]
pub fn iter_to_result<V, E, I>(mut iter: I) -> Result<VecIter<V>, E>
where
    I: Iterator<Item = Result<V, E>>,
    E: Debug,
{
    let mut error = None;
    let tmp_vec: Vec<V> = iter
        .map(|r| match r {
            Ok(v) => Some(v),
            Err(e) => {
                error = Some(e);
                None
            }
        })
        .take_while(Option::is_some)
        .map(Option::unwrap)
        .collect();
    if let Some(error) = error {
        Err(error)
    } else {
        Ok(tmp_vec.into_iter())
    }
}
