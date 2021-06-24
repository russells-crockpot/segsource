use crate::{Endidness, Segment};

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
pub fn basic_test_1() {
    let segment = Segment::new(&TEST_U8_DATA);
    assert_eq!(segment.current_offset(), 0);
    assert_eq!(segment.size(), TEST_U8_DATA.len());
    assert_eq!(segment.upper_offset_limit(), TEST_U8_DATA.len());
    assert_eq!(segment.remaining(), TEST_U8_DATA.len());
    for i in 0..TEST_U8_DATA.len() {
        assert_eq!(i as u8, segment.item_at(i).unwrap());
    }
    for i in 0..TEST_U8_DATA.len() {
        assert_eq!(segment.current_offset(), i);
        assert_eq!(segment.size(), TEST_U8_DATA.len());
        assert_eq!(segment.remaining(), TEST_U8_DATA.len() - i);
        assert_eq!(i as u8, segment.next_u8().unwrap());
    }
    let segment = Segment::with_offset(&TEST_U8_DATA, 5);
    assert_eq!(segment.current_offset(), 5);
    assert_eq!(segment.size(), TEST_U8_DATA.len());
    assert_eq!(segment.upper_offset_limit(), TEST_U8_DATA.len() + 5);
    assert_eq!(segment.remaining(), TEST_U8_DATA.len());
    for i in 0..TEST_U8_DATA.len() {
        assert_eq!(i as u8, segment.item_at(i + 5).unwrap());
    }
    for i in 0..TEST_U8_DATA.len() {
        assert_eq!(segment.current_offset(), i + 5);
        assert_eq!(segment.size(), TEST_U8_DATA.len());
        assert_eq!(segment.remaining(), TEST_U8_DATA.len() - i);
        assert_eq!(i as u8, segment.next_u8().unwrap());
    }
}

#[test]
pub fn test_move_by() {
    let segment = Segment::new(&TEST_U8_DATA);
    for i in 0..TEST_U8_DATA.len() {
        assert_eq!(segment.current_offset(), i);
        assert_eq!(segment.size(), TEST_U8_DATA.len());
        assert_eq!(segment.remaining(), TEST_U8_DATA.len() - i);
        assert_eq!(segment.current_item().unwrap(), TEST_U8_DATA[i]);
        segment.move_by(1).unwrap();
    }
    let segment = Segment::with_offset(&TEST_U8_DATA, 8);
    for i in 0..TEST_U8_DATA.len() {
        assert_eq!(segment.current_offset(), i + 8);
        assert_eq!(segment.size(), TEST_U8_DATA.len());
        assert_eq!(segment.remaining(), TEST_U8_DATA.len() - i);
        assert_eq!(segment.current_item().unwrap(), TEST_U8_DATA[i]);
        segment.move_by(1).unwrap();
    }
}

#[test]
pub fn test_move_to() {
    let segment = Segment::new(&TEST_U8_DATA);
    for i in 0..TEST_U8_DATA.len() {
        assert_eq!(segment.current_offset(), i);
        assert_eq!(segment.size(), TEST_U8_DATA.len());
        assert_eq!(segment.remaining(), TEST_U8_DATA.len() - i);
        assert_eq!(segment.current_item().unwrap(), TEST_U8_DATA[i]);
        segment.move_to(i + 1).unwrap();
    }
    let segment = Segment::with_offset(&TEST_U8_DATA, 7);
    for i in 0..TEST_U8_DATA.len() {
        assert_eq!(segment.current_offset(), i + 7);
        assert_eq!(segment.size(), TEST_U8_DATA.len());
        assert_eq!(segment.remaining(), TEST_U8_DATA.len() - i);
        assert_eq!(segment.current_item().unwrap(), TEST_U8_DATA[i]);
        segment.move_to(i + 1 + segment.initial_offset()).unwrap();
    }
}

#[test]
pub fn next_n_as_slice_test() {
    let segment = Segment::new(&TEST_U8_DATA);
    let slice1 = segment.next_n_as_slice(5).unwrap();
    assert_eq!(slice1, &TEST_U8_DATA[..5]);
    let slice2 = segment.next_n_as_slice(5).unwrap();
    assert_eq!(slice2, &TEST_U8_DATA[5..10]);
    assert_eq!(segment.get_remaining().unwrap(), &TEST_U8_DATA[10..]);
}

#[test]
pub fn basic_le_test() {
    let mut segment = Segment::with_offset_and_endidness(&TEST_U8_DATA, 0, Endidness::Little);
    for num in LE_U16_U8_DATA.iter() {
        assert_eq!(*num, segment.next_u16().unwrap());
    }
    segment = Segment::with_offset_and_endidness(&TEST_U8_DATA, 0, Endidness::Little);
    for num in LE_U32_U8_DATA.iter() {
        assert_eq!(*num, segment.next_u32().unwrap());
    }
    segment = Segment::with_offset_and_endidness(&TEST_U8_DATA, 0, Endidness::Little);
    for num in LE_U64_U8_DATA.iter() {
        assert_eq!(*num, segment.next_u64().unwrap());
    }
    segment = Segment::with_offset_and_endidness(&TEST_U8_DATA, 0, Endidness::Little);
    assert_eq!(LE_U128_U8_DATA, segment.next_u128().unwrap());
}

#[test]
pub fn basic_be_test() {
    let mut segment = Segment::with_offset_and_endidness(&TEST_U8_DATA, 0, Endidness::Big);
    for num in BE_U16_U8_DATA.iter() {
        assert_eq!(*num, segment.next_u16().unwrap());
    }
    segment = Segment::with_offset_and_endidness(&TEST_U8_DATA, 0, Endidness::Big);
    for num in BE_U32_U8_DATA.iter() {
        assert_eq!(*num, segment.next_u32().unwrap());
    }
    segment = Segment::with_offset_and_endidness(&TEST_U8_DATA, 0, Endidness::Big);
    for num in BE_U64_U8_DATA.iter() {
        assert_eq!(*num, segment.next_u64().unwrap());
    }
    segment = Segment::with_offset_and_endidness(&TEST_U8_DATA, 0, Endidness::Big);
    assert_eq!(BE_U128_U8_DATA, segment.next_u128().unwrap());
}
/*
#[test]
pub fn test_sliced_retain_offset() {
    let base_segment = Segment::new(&TEST_U8_DATA);
    base_segment.move_to(0x03);
    let sliced_segment = base_segment.next_n_bytes_as_segment_retain_offset(5);
    base_segment.move_to(0x03);
    assert_eq!(sliced_segment.initial_offset(), base_segment.current_offset());
    assert_eq!(
        sliced_segment.lower_offset_limit(),
        base_segment.current_offset()
    );
    assert_eq!(sliced_segment.current_offset(), base_segment.current_offset());
    assert_eq!(
        sliced_segment.upper_offset_limit(),
        base_segment.current_offset() + sliced_segment.size()
    );
}
*/
