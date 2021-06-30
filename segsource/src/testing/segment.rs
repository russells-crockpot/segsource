#![allow(clippy::needless_range_loop)]
use crate::{Endidness, Result, Segment};

pub const TEST_U8_DATA: [u8; 16] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
];

const LE_U16_U8_DATA: [u16; 8] = [
    0x0100, 0x0302, 0x0504, 0x0706, 0x0908, 0x0b0a, 0x0d0c, 0x0f0e,
];
const BE_U16_U8_DATA: [u16; 8] = [
    0x0001, 0x0203, 0x0405, 0x0607, 0x0809, 0x0a0b, 0x0c0d, 0x0e0f,
];

const LE_U32_U8_DATA: [u32; 4] = [0x03020100, 0x07060504, 0x0b0a0908, 0x0f0e0d0c];
const BE_U32_U8_DATA: [u32; 4] = [0x00010203, 0x04050607, 0x08090a0b, 0x0c0d0e0f];

const LE_U64_U8_DATA: [u64; 2] = [0x0706050403020100, 0x0f0e0d0c0b0a0908];
const BE_U64_U8_DATA: [u64; 2] = [0x0001020304050607, 0x08090a0b0c0d0e0f];

const LE_U128_U8_DATA: u128 = 0x0f0e0d0c0b0a09080706050403020100;
const BE_U128_U8_DATA: u128 = 0x000102030405060708090a0b0c0d0e0f;

#[test]
pub fn basic_test_1() -> Result<()> {
    let segment = Segment::new(&TEST_U8_DATA);
    assert_eq!(segment.current_offset(), 0);
    assert_eq!(segment.size(), TEST_U8_DATA.len());
    assert_eq!(segment.upper_offset_limit(), TEST_U8_DATA.len());
    assert_eq!(segment.remaining(), TEST_U8_DATA.len());
    for i in 0..TEST_U8_DATA.len() {
        assert_eq!(i as u8, segment.item_at(i)?);
    }
    for i in 0..TEST_U8_DATA.len() {
        assert_eq!(segment.current_offset(), i);
        assert_eq!(segment.size(), TEST_U8_DATA.len());
        assert_eq!(segment.remaining(), TEST_U8_DATA.len() - i);
        assert_eq!(i as u8, segment.next_u8()?);
    }
    let segment = Segment::with_offset(&TEST_U8_DATA, 5);
    assert_eq!(segment.current_offset(), 5);
    assert_eq!(segment.size(), TEST_U8_DATA.len());
    assert_eq!(segment.upper_offset_limit(), TEST_U8_DATA.len() + 5);
    assert_eq!(segment.remaining(), TEST_U8_DATA.len());
    for i in 0..TEST_U8_DATA.len() {
        assert_eq!(i as u8, segment.item_at(i + 5)?);
    }
    for i in 0..TEST_U8_DATA.len() {
        assert_eq!(segment.current_offset(), i + 5);
        assert_eq!(segment.size(), TEST_U8_DATA.len());
        assert_eq!(segment.remaining(), TEST_U8_DATA.len() - i);
        assert_eq!(i as u8, segment.next_u8()?);
    }
    Ok(())
}

#[test]
pub fn test_move_by() -> Result<()> {
    let segment = Segment::new(&TEST_U8_DATA);
    for i in 0..TEST_U8_DATA.len() {
        assert_eq!(segment.current_offset(), i);
        assert_eq!(segment.size(), TEST_U8_DATA.len());
        assert_eq!(segment.remaining(), TEST_U8_DATA.len() - i);
        assert_eq!(segment.current_item()?, TEST_U8_DATA[i]);
        segment.move_by(1)?;
    }
    let segment = Segment::with_offset(&TEST_U8_DATA, 8);
    for i in 0..TEST_U8_DATA.len() {
        assert_eq!(segment.current_offset(), i + 8);
        assert_eq!(segment.size(), TEST_U8_DATA.len());
        assert_eq!(segment.remaining(), TEST_U8_DATA.len() - i);
        assert_eq!(segment.current_item()?, TEST_U8_DATA[i]);
        segment.move_by(1)?;
    }
    Ok(())
}

#[test]
pub fn test_move_to() -> Result<()> {
    let segment = Segment::new(&TEST_U8_DATA);
    for i in 0..TEST_U8_DATA.len() {
        assert_eq!(segment.current_offset(), i);
        assert_eq!(segment.size(), TEST_U8_DATA.len());
        assert_eq!(segment.remaining(), TEST_U8_DATA.len() - i);
        assert_eq!(segment.current_item()?, TEST_U8_DATA[i]);
        segment.move_to(i + 1)?;
    }
    let segment = Segment::with_offset(&TEST_U8_DATA, 7);
    for i in 0..TEST_U8_DATA.len() {
        assert_eq!(segment.current_offset(), i + 7);
        assert_eq!(segment.size(), TEST_U8_DATA.len());
        assert_eq!(segment.remaining(), TEST_U8_DATA.len() - i);
        assert_eq!(segment.current_item()?, TEST_U8_DATA[i]);
        segment.move_to(i + 1 + segment.initial_offset())?;
    }
    Ok(())
}

#[test]
pub fn next_n_as_slice_test() -> Result<()> {
    let segment = Segment::new(&TEST_U8_DATA);
    let slice1 = segment.next_n_as_slice(5)?;
    assert_eq!(slice1, &TEST_U8_DATA[..5]);
    let slice2 = segment.next_n_as_slice(5)?;
    assert_eq!(slice2, &TEST_U8_DATA[5..10]);
    assert_eq!(segment.get_remaining_as_slice()?, &TEST_U8_DATA[10..]);
    Ok(())
}

#[test]
pub fn basic_le_test() -> Result<()> {
    let mut segment = Segment::with_offset_and_endidness(&TEST_U8_DATA, 0, Endidness::Little);
    for num in LE_U16_U8_DATA.iter() {
        assert_eq!(*num, segment.next_u16()?);
    }
    segment = Segment::with_offset_and_endidness(&TEST_U8_DATA, 0, Endidness::Little);
    for num in LE_U32_U8_DATA.iter() {
        assert_eq!(*num, segment.next_u32()?);
    }
    segment = Segment::with_offset_and_endidness(&TEST_U8_DATA, 0, Endidness::Little);
    for num in LE_U64_U8_DATA.iter() {
        assert_eq!(*num, segment.next_u64()?);
    }
    segment = Segment::with_offset_and_endidness(&TEST_U8_DATA, 0, Endidness::Little);
    assert_eq!(LE_U128_U8_DATA, segment.next_u128()?);
    Ok(())
}

#[test]
pub fn basic_be_test() -> Result<()> {
    let mut segment = Segment::with_offset_and_endidness(&TEST_U8_DATA, 0, Endidness::Big);
    for num in BE_U16_U8_DATA.iter() {
        assert_eq!(*num, segment.next_u16()?);
    }
    segment = Segment::with_offset_and_endidness(&TEST_U8_DATA, 0, Endidness::Big);
    for num in BE_U32_U8_DATA.iter() {
        assert_eq!(*num, segment.next_u32()?);
    }
    segment = Segment::with_offset_and_endidness(&TEST_U8_DATA, 0, Endidness::Big);
    for num in BE_U64_U8_DATA.iter() {
        assert_eq!(*num, segment.next_u64()?);
    }
    segment = Segment::with_offset_and_endidness(&TEST_U8_DATA, 0, Endidness::Big);
    assert_eq!(BE_U128_U8_DATA, segment.next_u128()?);
    Ok(())
}

#[test]
pub fn test_next_items_are() -> Result<()> {
    let segment = Segment::new(&TEST_U8_DATA);
    assert!(!segment.next_items_are(&[0x1, 0x2, 0x3])?);
    assert!(segment.next_items_are(&[0x0, 0x1, 0x2])?);
    segment.move_by(1)?;
    assert!(segment.next_items_are(&[0x1, 0x2, 0x3])?);
    assert!(!segment.next_items_are(&[0x0, 0x1, 0x2])?);
    Ok(())
}

macro_rules! idx_tests {
    ($offset:literal) => {
        let segment = Segment::with_offset(&TEST_U8_DATA, $offset);
        for i in 0..TEST_U8_DATA.len() {
            assert_eq!(segment[i + $offset], TEST_U8_DATA[i]);
        }
        assert_eq!(segment[0 + $offset..5 + $offset], TEST_U8_DATA[0..5]);
        assert_eq!(segment[5 + $offset..10 + $offset], TEST_U8_DATA[5..10]);
        assert_eq!(segment[..5 + $offset], TEST_U8_DATA[..5]);
        assert_eq!(segment[5 + $offset..], TEST_U8_DATA[5..]);
        assert_eq!(segment[0 + $offset..=5 + $offset], TEST_U8_DATA[0..=5]);
        assert_eq!(segment[5 + $offset..=10 + $offset], TEST_U8_DATA[5..=10]);
        assert_eq!(segment[..=5 + $offset], TEST_U8_DATA[..=5]);
    };
}

#[test]
pub fn test_indexing() -> Result<()> {
    idx_tests! { 0 };
    idx_tests! { 10 };
    idx_tests! { 50 };
    Ok(())
}

macro_rules! next_n_tests {
    ($offset:literal, $move_by:literal, $n:literal) => {
        let segment = Segment::with_offset(&TEST_U8_DATA, $offset);
        segment.move_by($move_by)?;
        let child1 = segment.next_n($n)?;
        assert_eq!(child1.initial_offset(), segment.current_offset() - $n);
        assert_eq!(child1.current_offset(), segment.current_offset() - $n);
        assert_eq!(child1.size(), $n);
        assert_eq!(child1.upper_offset_limit(), segment.current_offset());
        for i in $move_by..$n + $move_by {
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
        let child2 = remaining.next_n($n)?;
        assert_eq!(child2.initial_offset(), remaining.current_offset() - $n);
        assert_eq!(child2.current_offset(), remaining.current_offset() - $n);
        assert_eq!(child2.size(), $n);
        assert_eq!(child2.upper_offset_limit(), remaining.current_offset());
        for i in $move_by + $n..($n * 2) + $move_by {
            assert_eq!(child2[i + $offset], TEST_U8_DATA[i]);
        }
        remaining.move_by(1)?;
        let child3 = remaining.next_n($n)?;
        assert_eq!(child3.initial_offset(), remaining.current_offset() - $n);
        assert_eq!(child3.current_offset(), remaining.current_offset() - $n);
        assert_eq!(child3.size(), $n);
        assert_eq!(child3.upper_offset_limit(), remaining.current_offset());
        for i in $move_by + ($n * 2) + 1..($n * 3) + $move_by + 1 {
            assert_eq!(child3[i + $offset], TEST_U8_DATA[i]);
        }
    };
}

#[test]
pub fn next_n_test() -> Result<()> {
    next_n_tests! { 0, 3, 3 };
    next_n_tests! { 10, 2, 4 };
    next_n_tests! { 50, 1, 4 };
    Ok(())
}

#[test]
pub fn all_before() -> Result<()> {
    let segment = Segment::new(&TEST_U8_DATA);
    segment.move_by(3)?;
    let before = segment.all_before(4)?;
    assert_eq!(before.initial_offset(), 0);
    assert_eq!(before.as_ref(), &TEST_U8_DATA[..4]);
    let segment = Segment::with_offset(&TEST_U8_DATA, 5);
    segment.move_by(3)?;
    let before = segment.all_before(9)?;
    assert_eq!(before.initial_offset(), 5);
    assert_eq!(before.as_ref(), &TEST_U8_DATA[..4]);
    Ok(())
}

#[test]
pub fn all_after() -> Result<()> {
    let segment = Segment::new(&TEST_U8_DATA);
    segment.move_by(3)?;
    let after = segment.all_after(10)?;
    assert_eq!(after.initial_offset(), 10);
    assert_eq!(after.as_ref(), &TEST_U8_DATA[10..]);
    let segment = Segment::with_offset(&TEST_U8_DATA, 5);
    segment.move_by(3)?;
    let after = segment.all_after(15)?;
    assert_eq!(after.initial_offset(), 15);
    assert_eq!(after.as_ref(), &TEST_U8_DATA[10..]);
    Ok(())
}
