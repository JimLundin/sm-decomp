//! Synthetic independent captures protect the remaining observation decoders.
//! Addresses and expected quantities are stated independently of the source table.

mod content;

use crate::decode::directional_travel;
use crate::source::{self, SlotField, WramArray, WramBlock, WramByte, WramTable, WramWord};
use core::num::Wrapping;
use types::{Direction, TravelDistance};

fn put(wram: &mut [u8], offset: usize, value: u16) {
    wram[offset..offset.saturating_add(2)].copy_from_slice(&value.to_le_bytes());
}

#[test]
fn movement_position_sources_read_independently_placed_words() {
    let mut wram = vec![0; 0x10000];
    put(&mut wram, 0x0AF6, 0x1234);
    put(&mut wram, 0x0AF8, 0x5678);
    put(&mut wram, 0x0AFA, 0x9ABC);
    put(&mut wram, 0x0AFC, 0xDEF0);
    for (source, expected) in [
        (source::SAMUS_X_POS, 0x1234),
        (source::SAMUS_X_SUBPOS, 0x5678),
        (source::SAMUS_Y_POS, 0x9ABC),
        (source::SAMUS_Y_SUBPOS, 0xDEF0),
    ] {
        assert_eq!(source.read(&wram), Ok(expected));
    }
}

#[test]
fn directional_observations_preserve_sign_fraction_and_direction() {
    let mut wram = vec![0; 0x10000];
    for (offset, value) in [
        (0x0DAA, 0xFFFF),
        (0x0DAC, 0x8000),
        (0x0DAE, 2),
        (0x0DB0, 0x2345),
        (0x0DB2, 0xFFFE),
        (0x0DB4, 0xFFFF),
        (0x0DB6, 0),
        (0x0DB8, 1),
    ] {
        put(&mut wram, offset, value);
    }
    let actual = directional_travel(&wram).unwrap();
    for (direction, expected) in [
        (Direction::Left, 0xFFFF_8000),
        (Direction::Right, 0x0002_2345),
        (Direction::Up, 0xFFFE_FFFF),
        (Direction::Down, 1),
    ] {
        assert_eq!(actual[direction], TravelDistance(Wrapping(expected)));
    }
    assert!(directional_travel(&wram[..0x0DB9]).is_err());
}

#[test]
fn source_reads_reject_truncation_and_preserve_endianness() {
    let bytes = [0x31, 0x82, 0x49];
    let byte = WramByte {
        name: "synthetic byte",
        offset: 2,
    };
    let word = WramWord {
        name: "synthetic word",
        offset: 1,
    };
    let block = WramBlock {
        name: "synthetic block",
        offset: 1,
        len: 2,
    };
    assert_eq!(byte.read(&bytes), Ok(0x49));
    assert_eq!(word.read(&bytes), Ok(0x4982));
    assert_eq!(block.read(&bytes), Ok(&bytes[1..]));
    assert!(byte.read(&bytes[..2]).is_err());
    assert!(word.read(&bytes[..2]).is_err());
    assert!(block.read(&bytes[..2]).is_err());
    let outside = WramBlock {
        offset: u32::MAX,
        ..block
    };
    assert!(outside.read(&bytes).is_err());
}

#[test]
fn table_and_array_readers_reject_address_wraparound() {
    let table = WramTable {
        name: "synthetic table",
        offset: 6,
        stride: 4,
    };
    let field = SlotField {
        name: "synthetic field",
        offset: 2,
    };
    let array = WramArray {
        name: "synthetic array",
        offset: 6,
    };
    let mut bytes = vec![0; 20];
    put(&mut bytes, 16, 0x1234);
    put(&mut bytes, 10, 0xABCD);
    assert_eq!(table.word(field, 2).unwrap().read(&bytes), Ok(0x1234));
    assert_eq!(array.word(2).unwrap().read(&bytes), Ok(0xABCD));
    assert!(table.word(field, u16::MAX).is_err());
    assert!(array.word(u16::MAX).is_err());
    assert!(
        WramTable {
            offset: u16::MAX,
            ..table
        }
        .word(field, 0)
        .is_err()
    );
    assert!(
        WramArray {
            offset: u16::MAX,
            ..array
        }
        .word(1)
        .is_err()
    );
}
