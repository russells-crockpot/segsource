#![allow(clippy::needless_range_loop)]
use crate::{marker::Integer, Endidness, Result, Segment};
use core::fmt::Debug;
use paste::paste;

pub const TEST_U8_DATA: [u8; 16] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
];

pub const LE_U16_U8_DATA: [u16; 8] = [
    0x0100, 0x0302, 0x0504, 0x0706, 0x0908, 0x0b0a, 0x0d0c, 0x0f0e,
];
pub const BE_U16_U8_DATA: [u16; 8] = [
    0x0001, 0x0203, 0x0405, 0x0607, 0x0809, 0x0a0b, 0x0c0d, 0x0e0f,
];

pub const LE_U32_U8_DATA: [u32; 4] = [0x03020100, 0x07060504, 0x0b0a0908, 0x0f0e0d0c];
pub const BE_U32_U8_DATA: [u32; 4] = [0x00010203, 0x04050607, 0x08090a0b, 0x0c0d0e0f];

pub const LE_U64_U8_DATA: [u64; 2] = [0x0706050403020100, 0x0f0e0d0c0b0a0908];
pub const BE_U64_U8_DATA: [u64; 2] = [0x0001020304050607, 0x08090a0b0c0d0e0f];

pub const LE_U128_U8_DATA: [u128; 1] = [0x0f0e0d0c0b0a09080706050403020100];
pub const BE_U128_U8_DATA: [u128; 1] = [0x000102030405060708090a0b0c0d0e0f];

macro_rules! make_basic_test_fn {
    (
        @cmp_func = $base_func_name: ident,
        $test_func_name: ident,
        [$(@$endidness:expr, $offset:literal $(, ($($arg:expr),+))?);+ $(;)?]
    ) => {
        #[test]
        fn $test_func_name() -> Result<()> {
            $({
                let segment = Segment::with_offset_and_endidness(
                    &TEST_U8_DATA, $offset, $endidness);
                $base_func_name(&segment$(, $($arg),+)?)?;
            })+
            Ok(())
        }
    };
    (
        $name: ident,
        [$(@$endidness:expr, $offset:literal $(, ($($arg:expr),+))?);+ $(;)?]
    ) => {
        paste!{
            make_basic_test_fn! {
                @cmp_func = [<$name _impl>],
                $name,
                [$(@$endidness, $offset $(, ($($arg),+))?);+]
            }
        }
    };
    (
        $(@cmp_func = $base_func_name: ident,)?
        $name: ident,
        [$($offset:literal $(, ($($arg:expr),+))?);+ $(;)?]
    ) => {
        make_basic_test_fn! {
            $(@cmp_func = $base_func_name,)?
            $name,
            [$(@Endidness::native(), $offset $(, ($($arg),+))?);+]
        }
    };
    (
        $(@cmp_func = $base_func_name: ident,)?
        $name: ident,
        [$($offset:literal);+ $(;)?]
    ) => {
        make_basic_test_fn! { $(@cmp_func = $base_func_name,)? $name, [$($offset, ());+] }
    };
}

pub fn basic_cmp<I: Integer + Debug + PartialEq>(
    segment: &Segment<'_, u8>,
    compare_to: &[I],
) -> Result<()> {
    for num in compare_to.iter() {
        assert_eq!(*num, segment.next_int()?);
    }
    Ok(())
}

make_basic_test_fn! { basic_test_1, [0; 1; 5; 8] }
make_basic_test_fn! { move_by_test, [0; 1; 5; 8] }
make_basic_test_fn! { move_to_test, [0; 1; 5; 8] }
make_basic_test_fn! { next_n_as_slice_test, [0; 1; 5; 8] }
make_basic_test_fn! { next_items_are_test, [0; 1; 5; 8] }
make_basic_test_fn! { indexing_test, [0; 1; 5; 8; 10; 50; 100] }
make_basic_test_fn! { next_n_test,
    [
        0, (3, 3);
        10, (2, 4);
        50, (1, 4);
    ]
}
make_basic_test_fn! { all_before, [0, (4); 1, (3); 100, (2); 8, (5)] }
make_basic_test_fn! { all_after, [0, (4); 1, (4); 100, (4); 8, (6)] }

make_basic_test_fn! {
    @cmp_func = basic_cmp,
    basic_le_test,
    [
        @Endidness::Little, 0, (&LE_U16_U8_DATA);
        @Endidness::Little, 0, (&LE_U32_U8_DATA);
        @Endidness::Little, 0, (&LE_U64_U8_DATA);
        @Endidness::Little, 0, (&LE_U128_U8_DATA);
    ]
}

make_basic_test_fn! {
    @cmp_func = basic_cmp,
    basic_be_test,
    [
        @Endidness::Big, 0, (&BE_U16_U8_DATA);
        @Endidness::Big, 0, (&BE_U32_U8_DATA);
        @Endidness::Big, 0, (&BE_U64_U8_DATA);
        @Endidness::Big, 0, (&BE_U128_U8_DATA);
    ]
}

pub fn basic_test_1_impl(segment: &Segment<'_, u8>) -> Result<()> {
    let initial_offset = segment.initial_offset();
    assert_eq!(segment.current_offset(), initial_offset);
    assert_eq!(segment.size(), TEST_U8_DATA.len());
    assert_eq!(
        segment.upper_offset_limit(),
        TEST_U8_DATA.len() + initial_offset
    );
    assert_eq!(segment.remaining(), TEST_U8_DATA.len());
    for i in 0..TEST_U8_DATA.len() {
        assert_eq!(i as u8, segment.item_at(i + initial_offset)?);
    }
    for i in 0..TEST_U8_DATA.len() {
        assert_eq!(segment.current_offset(), i + initial_offset);
        assert_eq!(segment.size(), TEST_U8_DATA.len());
        assert_eq!(segment.remaining(), TEST_U8_DATA.len() - i);
        assert_eq!(i as u8, segment.next_u8()?);
    }
    Ok(())
}

pub fn move_by_test_impl(segment: &Segment<'_, u8>) -> Result<()> {
    let initial_offset = segment.initial_offset();
    for i in 0..TEST_U8_DATA.len() {
        assert_eq!(segment.current_offset(), i + initial_offset);
        assert_eq!(segment.size(), TEST_U8_DATA.len());
        assert_eq!(segment.remaining(), TEST_U8_DATA.len() - i);
        assert_eq!(segment.current_item()?, TEST_U8_DATA[i]);
        segment.move_by(1)?;
    }
    Ok(())
}

pub fn move_to_test_impl(segment: &Segment<'_, u8>) -> Result<()> {
    let initial_offset = segment.initial_offset();
    for i in 0..TEST_U8_DATA.len() {
        assert_eq!(segment.current_offset(), i + initial_offset);
        assert_eq!(segment.size(), TEST_U8_DATA.len());
        assert_eq!(segment.remaining(), TEST_U8_DATA.len() - i);
        assert_eq!(segment.current_item()?, TEST_U8_DATA[i]);
        segment.move_to(i + 1 + segment.initial_offset())?;
    }
    Ok(())
}

pub fn next_n_as_slice_test_impl(segment: &Segment<'_, u8>) -> Result<()> {
    let slice1 = segment.next_n_as_slice(5)?;
    assert_eq!(slice1, &TEST_U8_DATA[..5]);
    let slice2 = segment.next_n_as_slice(5)?;
    assert_eq!(slice2, &TEST_U8_DATA[5..10]);
    assert_eq!(segment.get_remaining_as_slice()?, &TEST_U8_DATA[10..]);
    Ok(())
}

pub fn next_items_are_test_impl(segment: &Segment<'_, u8>) -> Result<()> {
    assert!(!segment.next_items_are(&[0x1, 0x2, 0x3])?);
    assert!(segment.next_items_are(&[0x0, 0x1, 0x2])?);
    segment.move_by(1)?;
    assert!(segment.next_items_are(&[0x1, 0x2, 0x3])?);
    assert!(!segment.next_items_are(&[0x0, 0x1, 0x2])?);
    Ok(())
}

pub fn indexing_test_impl(segment: &Segment<'_, u8>) -> Result<()> {
    let initial_offset = segment.initial_offset();
    for i in 0..TEST_U8_DATA.len() {
        assert_eq!(segment[i + initial_offset], TEST_U8_DATA[i]);
    }
    assert_eq!(
        segment[initial_offset..5 + initial_offset],
        TEST_U8_DATA[0..5]
    );
    assert_eq!(
        segment[5 + initial_offset..10 + initial_offset],
        TEST_U8_DATA[5..10]
    );
    assert_eq!(segment[..5 + initial_offset], TEST_U8_DATA[..5]);
    assert_eq!(segment[5 + initial_offset..], TEST_U8_DATA[5..]);
    assert_eq!(
        segment[initial_offset..=5 + initial_offset],
        TEST_U8_DATA[0..=5]
    );
    assert_eq!(
        segment[5 + initial_offset..=10 + initial_offset],
        TEST_U8_DATA[5..=10]
    );
    assert_eq!(segment[..=5 + initial_offset], TEST_U8_DATA[..=5]);
    Ok(())
}

pub fn next_n_test_impl(segment: &Segment<'_, u8>, move_by: usize, n: usize) -> Result<()> {
    let initial_offset = segment.initial_offset();
    segment.move_by(move_by as i128)?;
    let child1 = segment.next_n(n)?;
    assert_eq!(child1.initial_offset(), segment.current_offset() - n);
    assert_eq!(child1.current_offset(), segment.current_offset() - n);
    assert_eq!(child1.size(), n);
    assert_eq!(child1.upper_offset_limit(), segment.current_offset());
    for i in move_by..n + move_by {
        assert_eq!(child1.next_item()?, TEST_U8_DATA[i]);
    }
    let remaining = segment.get_remaining()?;
    assert_eq!(
        remaining.initial_offset(),
        segment.current_offset() - remaining.size()
    );
    assert_eq!(
        remaining.current_offset(),
        segment.current_offset() - remaining.size()
    );
    assert_eq!(remaining.upper_offset_limit(), segment.current_offset());
    let child2 = remaining.next_n(n)?;
    assert_eq!(child2.initial_offset(), remaining.current_offset() - n);
    assert_eq!(child2.current_offset(), remaining.current_offset() - n);
    assert_eq!(child2.size(), n);
    assert_eq!(child2.upper_offset_limit(), remaining.current_offset());
    for i in move_by + n..(n * 2) + move_by {
        assert_eq!(child2[i + initial_offset], TEST_U8_DATA[i]);
    }
    remaining.move_by(1)?;
    let child3 = remaining.next_n(n)?;
    assert_eq!(child3.initial_offset(), remaining.current_offset() - n);
    assert_eq!(child3.current_offset(), remaining.current_offset() - n);
    assert_eq!(child3.size(), n);
    assert_eq!(child3.upper_offset_limit(), remaining.current_offset());
    for i in move_by + (n * 2) + 1..(n * 3) + move_by + 1 {
        assert_eq!(child3[i + initial_offset], TEST_U8_DATA[i]);
    }
    Ok(())
}

pub fn all_before_impl(segment: &Segment<'_, u8>, start_at: usize) -> Result<()> {
    let initial_offset = segment.initial_offset();
    segment.move_by(3)?;
    let before = segment.all_before(start_at + initial_offset)?;
    assert_eq!(before.as_ref(), &TEST_U8_DATA[..start_at]);
    Ok(())
}

pub fn all_after_impl(segment: &Segment<'_, u8>, start_at: usize) -> Result<()> {
    let initial_offset = segment.initial_offset();
    segment.move_by(3)?;
    let after = segment.all_after(start_at + initial_offset)?;
    assert_eq!(after.as_ref(), &TEST_U8_DATA[start_at..]);
    Ok(())
}
