use crate::{Endidness, Result, Source, U8Source, VecSource};

pub(crate) fn change_offset_tests<S: Source>(mut source: S) -> Result<()> {
    assert_eq!(source.initial_offset(), 0);
    assert_eq!(source.lower_offset_limit(), 0);
    assert_eq!(source.upper_offset_limit(), source.size());
    source.change_initial_offset(10);
    assert_eq!(source.initial_offset(), 10);
    assert_eq!(source.lower_offset_limit(), 10);
    assert_eq!(source.upper_offset_limit(), 10 + source.size());
    source.change_initial_offset(3);
    assert_eq!(source.initial_offset(), 3);
    assert_eq!(source.lower_offset_limit(), 3);
    assert_eq!(source.upper_offset_limit(), 3 + source.size());
    source.change_initial_offset(1001);
    assert_eq!(source.initial_offset(), 1001);
    assert_eq!(source.lower_offset_limit(), 1001);
    assert_eq!(source.upper_offset_limit(), 1001 + source.size());
    Ok(())
}

pub(crate) fn basic_u8_src_tests<S: U8Source>(mut source: S) -> Result<()> {
    assert_eq!(source.endidness(), Endidness::native());
    source.change_endidness(Endidness::Big);
    assert_eq!(source.endidness(), Endidness::Big);
    source.change_endidness(Endidness::Little);
    assert_eq!(source.endidness(), Endidness::Little);
    change_offset_tests(source)
}

#[allow(dead_code)]
pub(crate) fn segment_creation_tests<S: U8Source>(_source: S) -> Result<()> {
    todo!()
}

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
                    let source = $source;
                    let segment = source.all()?;
                    segment::$base_func_name(&segment$(, $($arg),+)?)?;
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

macro_rules! make_endidness_test {
    (
        $prefix:ident,
        $source:block,
        $width:literal,
        ($endidness_full:expr, $endidness_upper:ident, $endidness_lower:ident)
    ) => {
        paste! {
            make_source_segment_test_fn! {
                @cmp_func = basic_cmp,
                $source,
                $prefix,
                [<$endidness_lower _u $width _test>],
                [
                    @endidness_full, 0, (&segment::[<$endidness_upper _U $width _U8_DATA>]);
                ]
            }
        }
    };
    (
        $prefix:ident,
        $source:block,
        ($endidness_full:expr, $endidness_upper:ident, $endidness_lower:ident)
    ) => {
        make_endidness_test! {
            $prefix, $source, 16,
            ($endidness_full, $endidness_upper, $endidness_lower)
        }
        make_endidness_test! {
            $prefix, $source, 32,
            ($endidness_full, $endidness_upper, $endidness_lower)
        }
        make_endidness_test! {
            $prefix, $source, 64,
            ($endidness_full, $endidness_upper, $endidness_lower)
        }
        make_endidness_test! {
            $prefix, $source, 128,
            ($endidness_full, $endidness_upper, $endidness_lower)
        }
    };
}

macro_rules! make_segment_tests {
    (
        $prefix:ident,
        $source:block
    ) => {
        make_source_segment_test_fn! { $source, $prefix, basic_test_1, [0; 1; 5; 8] }
        make_source_segment_test_fn! { $source, $prefix, move_by_test, [0; 1; 5; 8] }
        make_source_segment_test_fn! { $source, $prefix, move_to_test, [0; 1; 5; 8] }
        make_source_segment_test_fn! { $source, $prefix, next_n_as_slice_test, [0; 1; 5; 8] }
        make_source_segment_test_fn! { $source, $prefix, next_items_are_test, [0; 1; 5; 8] }
        make_source_segment_test_fn! { $source, $prefix, indexing_test, [0; 1; 5; 8; 10; 50; 100] }
        make_source_segment_test_fn! { $source, $prefix, next_n_test,
            [
                0, (3, 3);
                10, (2, 4);
                50, (1, 4);
            ]
        }
        make_source_segment_test_fn! { $source, $prefix, all_before,
        [0, (4); 1, (3); 100, (2); 8, (5)] }
        make_source_segment_test_fn! { $source, $prefix, all_after,
        [0, (4); 1, (4); 100, (4); 8, (6)] }
    };
    (
        @with_le,
        $prefix:ident,
        $source:block
    ) => {
        make_segment_tests! {$prefix, $source}
        make_endidness_test! {
            $prefix,
            $source,
            (Endidness::Little, LE, le)
        }
    };
    (
        @with_be,
        $prefix:ident,
        $source:block
    ) => {
        make_segment_tests! {$prefix, $source}
        make_endidness_test! {
            $prefix,
            $source,
            (Endidness::Big, BE, be)
        }
    };
}

macro_rules! src_from {
    (
        $source:ty,
        @$make_func: path,
        $from:ident,
        $offset:literal
    ) => {
        paste! {
            $source::[<from_ $from _with_offset>](
                $make_func(&segment::TEST_U8_DATA as &[u8]), $offset)?
        }
    };
    (
        $source:ty,
        @$make_func: path,
        $from:ident
    ) => {
        paste! {
            $source::[<from_ $from>]($make_func(&segment::TEST_U8_DATA as &[u8]))?
        }
    };
    (
        $source:ty,
        $from:ident,
        $offset:literal
    ) => {
        paste! {
            $source::[<from_ $from _with_offset>](&segment::TEST_U8_DATA as &[u8], $offset)?
        }
    };
    (
        $source:ty,
        $from:ident
    ) => {
        paste! {
            $source::[<from_ $from>](&segment::TEST_U8_DATA as &[u8])?
        }
    };
}

macro_rules! u8_src_from {
    (
        $source:ty,
        @$make_func: path,
        $from:ident,
        $offset:literal,
        $endidness:expr
    ) => {
        paste! {
            $source::[<from_ $from _with_offset>](
                $make_func(&segment::TEST_U8_DATA as &[u8]),
                $offset,
                $endidness
            )?
        }
    };
    (
        $source:ty,
        @$make_func: path,
        $from:ident,
        $endidness:expr
    ) => {
        paste! {
            $source::[<from_ $from>](
                $make_func(&segment::TEST_U8_DATA as &[u8]),
                $endidness
            )?
        }
    };
    (
        $source:ty,
        $from:ident,
        $offset:literal,
        $endidness:expr
    ) => {
        paste! {
            $source::[<from_ $from _with_offset>](
                &segment::TEST_U8_DATA as &[u8],
                $offset,
                $endidness
            )?
        }
    };
    (
        $source:ty,
        $from:ident,
        $endidness:expr
    ) => {
        paste! {
            $source::[<from_ $from>](
                &segment::TEST_U8_DATA as &[u8],
                $endidness
            )?
        }
    };
}

macro_rules! add_imports {
    () => {
        #[cfg(feature = "with-bytes")]
        use crate::BytesSource;
        #[cfg(feature = "memmap")]
        use crate::MappedFileSource;
        use crate::{
            testing::{segment, sources::U8VecSource},
            Endidness, Result, SegmentLikeSource, Source as _, U8Source as _,
        };
        #[cfg(not(feature = "std"))]
        use alloc::vec::Vec;
        #[cfg(feature = "with-bytes")]
        use bytes::Bytes;
        use paste::paste;
        #[cfg(feature = "std")]
        use std::path::Path;
    };
}

macro_rules! make_from_tests {
    (is_u8, $source:ty, $mod_name:ident, $from:ident $(, $make_func:path)?) => {
        paste! {
            mod [<$mod_name _tests>] {
                #![allow(unused_imports)]
                add_imports!{}
                use crate::testing::sources::basic_u8_src_tests;

                #[test]
                fn [<from_ $from _test>]() -> Result<()> {
                    basic_u8_src_tests(u8_src_from!{$source, $(@$make_func,)? $from,
                        Endidness::native()})?;
                    basic_u8_src_tests(u8_src_from!{$source, $(@$make_func,)? $from, 0,
                        Endidness::native()})?;
                    Ok(())
                }

                make_segment_tests!{
                    @with_le,
                    le_no_offset,
                    {u8_src_from!{$source, $(@$make_func,)? $from, Endidness::Little}}}
                make_segment_tests!{
                    @with_le,
                    le_offset_of_7,
                    {u8_src_from!{$source, $(@$make_func,)? $from, 7, Endidness::Little}}}
                make_segment_tests!{
                    @with_le,
                    le_offset_of_100,
                    {u8_src_from!{$source, $(@$make_func,)? $from, 100, Endidness::Little}}}
                make_segment_tests!{
                    @with_be,
                    be_no_offset,
                    {u8_src_from!{$source, $(@$make_func,)? $from, Endidness::Big}}}
                make_segment_tests!{
                    @with_be,
                    be_offset_of_7,
                    {u8_src_from!{$source, $(@$make_func,)? $from, 7, Endidness::Big}}}
                make_segment_tests!{
                    @with_be,
                    be_offset_of_100,
                    {u8_src_from!{$source, $(@$make_func,)? $from, 100, Endidness::Big}}}
            }
        }
    };
    (is_u8, $source:ty, $from:ident $(, $make_func:path)?) => {
            make_from_tests! {
                is_u8,
                $source,
                $from,
                $from
                $(, $make_func)?
            }
    };
    (is_not_u8, $source:ty, $mod_name:ident, $from:ident $(, $make_func:path)?) => {
        paste! {
            mod [<$mod_name _tests>] {
                #![allow(unused_imports)]
                add_imports!{}
                use crate::testing::sources::change_offset_tests;

                #[test]
                fn [<from_ $from _test>]() -> Result<()> {

                    change_offset_tests(src_from!{$source, $(@$make_func,)? $from})?;
                    change_offset_tests(src_from!{$source, $(@$make_func,)? $from, 0})?;
                    Ok(())
                }
                make_segment_tests!{
                    no_offset, {
                        src_from!{$source, $(@$make_func,)? $from}
                    }
                }
                make_segment_tests!{
                    offset_of_7, {src_from!{$source, $(@$make_func,)? $from, 7}}
                }
                make_segment_tests!{
                    offset_of_100, {src_from!{$source, $(@$make_func,)? $from, 100}}
                }
            }
        }
    };
    (is_not_u8, $source:ty, $from:ident $(, $make_func:path)?) => {
        make_from_tests! {
            is_not_u8,
            $source,
            $from,
            $from
            $(, $make_func)?
        }
    };
}

macro_rules! make_source_tests {
    (
        $source:ty,
        $mod_name:ident
    ) => {
        mod $mod_name {
            #![allow(unused_imports)]
            add_imports! {}

            make_from_tests! { is_not_u8, $source, vec, Vec::from }
            make_from_tests! { is_u8, $source, u8_slice }
            make_from_tests! { is_u8, $source, u8_vec, Vec::from }
            #[cfg(feature = "with-bytes")]
            make_from_tests! { is_u8, $source, bytes, Bytes::copy_from_slice }
            //#[cfg(feature = "with-bytes")]
            //make_from_tests!{ is_u8, $source, bytes, Bytes::from }
        }
    };
}

pub(crate) type U8VecSource = VecSource<u8>;
make_source_tests! {U8VecSource, vec}
#[cfg(feature = "with-bytes")]
make_source_tests! {BytesSource, bytes}
#[cfg(feature = "memmap")]
make_source_tests! {MappedFileSource, memmap}

mod segment_like;
