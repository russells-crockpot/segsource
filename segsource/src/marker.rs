//! Markers for segsource.
use crate::Endidness;
use core::convert::TryInto;

/// An extension trait for integers.
pub trait Integer: Sized {
    const WIDTH: usize;
    fn from_be(bytes: &[u8]) -> Self;
    fn from_le(bytes: &[u8]) -> Self;
    fn from_ne(bytes: &[u8]) -> Self;
    fn with_endidness(bytes: &[u8], endidness: Endidness) -> Self {
        match endidness {
            Endidness::Big => Self::from_be(bytes),
            Endidness::Little => Self::from_le(bytes),
        }
    }
}

macro_rules! impl_integer {
    ($type:ty, $width:literal, $be_method:ident, $le_method:ident, $ne_method:ident) => {
        impl Integer for $type {
            const WIDTH: usize = $width;
            fn from_be(bytes: &[u8]) -> Self {
                <$type>::$be_method(bytes.try_into().unwrap())
            }
            fn from_le(bytes: &[u8]) -> Self {
                <$type>::$le_method(bytes.try_into().unwrap())
            }
            fn from_ne(bytes: &[u8]) -> Self {
                <$type>::$ne_method(bytes.try_into().unwrap())
            }
        }
    };
    ($type:ty, $width:literal) => {
        impl_integer! {$type, $width, from_be_bytes, from_le_bytes, from_ne_bytes}
    };
}

impl_integer! {u8, 1}
impl_integer! {u16, 2}
impl_integer! {u32, 4}
impl_integer! {u64, 8}
impl_integer! {u128, 16}
impl_integer! {i8, 1}
impl_integer! {i16, 2}
impl_integer! {i32, 4}
impl_integer! {i64, 8}
impl_integer! {i128, 16}
