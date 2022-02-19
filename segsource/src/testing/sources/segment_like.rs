#![allow(dead_code, unused_macros, unused_imports)]
use crate::{testing::segment, Endidness, Result, SegmentLikeSource, Source, U8Source, VecSource};

macro_rules! make_source_segment_test_fn {
    (
        @cmp_func = $base_func_name: path,
        $source:block,
        $prefix:ident,
        $name: ident,
        [$(@$endidness:expr, $offset:literal $(, ($($arg:expr),+))?);+ $(;)?]
    ) => {
        paste! {
            #[test]
            fn [<$prefix _ $name>]() -> Result<()> {
                $({
                    let source = SegmentLikeSource::new($source)?;
                    let segment = source.all()?;
                    segment::$base_func_name(&segment$(, $($arg),+)?)?;
                })+
                Ok(())
            }
            #[test]
            fn [<$prefix _ $name _src_as_seg>]() -> Result<()> {
                $({
                    let source = SegmentLikeSource::new($source)?;
                    segment::$base_func_name(&source$(, $($arg),+)?)?;
                })+
                Ok(())
            }
        }
    };
    (
        $source:block,
        $prefix:ident,
        $name: ident,
        [$(@$endidness:expr, $offset:literal $(, ($($arg:expr),+))?);+ $(;)?]
    ) => {
        paste!{
            make_source_segment_test_fn! {
                @cmp_func = [<$name _impl>],
                $source,
                $prefix,
                $name,
                [$(@$endidness, $offset $(, ($($arg),+))?);+]
            }
        }
    };
    (
        $(@cmp_func = $base_func_name: ident,)?
        $source:block,
        $prefix:ident,
        $name: ident,
        [$($offset:literal $(, ($($arg:expr),+))?);+ $(;)?]
    ) => {
        make_source_segment_test_fn! {
            $(@cmp_func = $base_func_name,)?
            $source,
                $prefix,
            $name,
            [$(@Endidness::native(), $offset $(, ($($arg),+))?);+]
        }
    };
    (
        $(@cmp_func = $base_func_name: ident,)?
        $source:block,
        $prefix:ident,
        $name: ident,
        [$($offset:literal);+ $(;)?]
    ) => {
        make_source_segment_test_fn! {
            $(@cmp_func = $base_func_name,)?
                $source,
                $prefix,
                $name,
                [$($offset, ());+]
        }
    };
}
//include!{"macros.rs"}

pub(crate) type U8VecSource = VecSource<u8>;
make_source_tests! {U8VecSource, vec}
#[cfg(feature = "with-bytes")]
make_source_tests! {BytesSource, bytes}
#[cfg(feature = "memmap")]
make_source_tests! {MappedFileSource, memmap}
