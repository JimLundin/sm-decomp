use super::*;
use crate::{Entry, Outcome, run};

fn fixture() -> Vec<u8> {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(WIDTH_BLOCKS, 32);
    for (i, value) in [0x0011, 0x0022, 0x0033, 0x0044].into_iter().enumerate() {
        r.set_word(TILE_TABLE + 8 + i * 2, value);
    }
    for offset in (0x10000..WRAM_SIZE).step_by(2) {
        r.set_word(offset, 1);
    }
    bytes
}

struct NoFlush;
impl Host for NoFlush {
    fn flush_vram(&mut self, _: &mut [u8]) -> Result<()> {
        panic!("unexpected foreign flush")
    }
}

#[test]
fn block_flips_preserve_quadrants_and_tile_attributes() {
    for (flip, expected) in [
        (0, [0x11, 0x22, 0x33, 0x44]),
        (0x400, [0x4022, 0x4011, 0x4044, 0x4033]),
        (0x800, [0x8033, 0x8044, 0x8011, 0x8022]),
        (0xc00, [0xc044, 0xc033, 0xc022, 0xc011]),
    ] {
        let mut bytes = fixture();
        let mut r = Wram::new(&mut bytes).unwrap();
        r.set_word(0x10002, 1 | flip);
        column(&mut r, false).unwrap();
        assert_eq!(
            [r.word(COLUMN_LEFT), r.word(COLUMN_RIGHT), r.word(COLUMN_LEFT + 2), r.word(COLUMN_RIGHT + 2)],
            expected
        );
        row(&mut r, false).unwrap();
        assert_eq!([r.word(ROW_TOP), r.word(ROW_TOP + 2), r.word(ROW_BOTTOM), r.word(ROW_BOTTOM + 2)], expected);
    }
}

#[test]
fn column_stride_wraps_at_word_width_and_multiplies_low_bytes() {
    let mut bytes = fixture();
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(WIDTH_BLOCKS, 0x8001);
    r.set_word(UPDATE_Y, 0x0101);
    r.set_word(0x10004, 0x0401);
    r.set_word(0x10006, 0x0801);
    column(&mut r, false).unwrap();
    assert_eq!((r.word(COLUMN_LEFT), r.word(COLUMN_LEFT + 4)), (0x4022, 0x8033));
}

#[test]
fn row_wraps_between_vram_screens_and_keeps_seventeen_blocks() {
    let mut bytes = fixture();
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(VRAM_X, 15);
    r.set_word(VRAM_Y, 2);
    row(&mut r, false).unwrap();
    assert_eq!((r.word(ROW), r.word(ROW + 2)), (4, 64));
    assert_eq!((r.word(ROW + 4), r.word(ROW + 6)), (0x509e, 0x5480));
    assert_eq!((r.word(ROW_TOP + 64), r.word(ROW_BOTTOM + 64)), (0x11, 0x33));
}

#[test]
fn layer_ratio_one_preserves_background_position_and_register() {
    let mut bytes = fixture();
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(MODE7, 1);
    r.set_word(X, 0x1234);
    r.set_word(Y, 0x3456);
    r.set_word(L2_X, 0x9876);
    r.set_word(0xb5, 0xabcd);
    r.0[0x91b] = 1;
    r.0[0x91c] = 128;
    r.set_word(0x923, 0xfff0);
    layer2_and_scrolls(&mut r).unwrap();
    assert_eq!((r.word(L2_X), r.word(0xb5)), (0x9876, 0xabcd));
    assert_eq!((r.word(L2_Y), r.word(0xb7)), (0x1a2b, 0x1a1b));
}

#[test]
fn layer_blocks_sign_extend_while_register_blocks_are_unsigned() {
    let mut bytes = fixture();
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(MODE7, 1);
    r.set_word(X, 0xfff0);
    r.set_word(Y, 0x8000);
    update_scroll_vars(&mut r).unwrap();
    assert_eq!((r.word(BLOCK_X), r.word(BLOCK_Y)), (0xffff, 0xf800));
    assert_eq!((r.word(BG1_BLOCK_X), r.word(BG1_BLOCK_Y)), (0x0fff, 0x0800));
}

struct Flush {
    seen: Vec<(u16, u16)>,
    fail_at: Option<usize>,
}
impl Host for Flush {
    fn flush_vram(&mut self, bytes: &mut [u8]) -> Result<()> {
        let mut r = Wram::new(bytes)?;
        self.seen.push((r.word(BLOCK_X), r.word(COLUMN_LEFT)));
        assert_eq!(r.word(COLUMN + 12), 0xab01);
        // Mimic the native service's low-byte clears, preserving upper bytes.
        for address in [COLUMN + 12, ROW + 12, COLUMN + BG2_OFFSET + 12, ROW + BG2_OFFSET + 12] {
            r.0[address] = 0;
        }
        r.set_word(0x330, 0);
        if self.fail_at == Some(self.seen.len()) {
            return Err("retained flush failure".into());
        }
        if self.seen.len() == 1 {
            // A later column must consume state returned from the service.
            r.set_word(0x10004, 0x0401);
        }
        Ok(())
    }
}

#[test]
fn display_flushes_each_column_before_preparing_the_next() {
    let mut bytes = fixture();
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(COLUMN + 12, 0xab00);
    r.0[0x58] = 0x51;
    r.0[0x59] = 0x49;
    let mut host = Flush { seen: Vec::new(), fail_at: None };
    assert_eq!(run(Entry::DisplayViewablePartOfRoom, &mut bytes, &mut host), Ok(Outcome::Handled));
    assert_eq!(host.seen.len(), 17);
    assert_eq!(host.seen[0], (0, 0x11));
    assert_eq!(host.seen[1], (1, 0x4022));
    assert_eq!(host.seen.iter().map(|(x, _)| *x).collect::<Vec<_>>(), (0..17).collect::<Vec<_>>());
    let r = Wram::new(&mut bytes).unwrap();
    assert_eq!((r.word(BLOCK_X), r.word(BG2_SIZE), r.word(COLUMN + 12)), (17, 0x800, 0xab00));
}

#[test]
fn service_failure_stops_owned_display_at_the_failing_column() {
    let mut bytes = fixture();
    Wram::new(&mut bytes).unwrap().set_word(COLUMN + 12, 0xab00);
    let mut host = Flush { seen: Vec::new(), fail_at: Some(2) };
    assert_eq!(run(Entry::DisplayViewablePartOfRoom, &mut bytes, &mut host), Err("retained flush failure".into()));
    assert_eq!(host.seen.len(), 2);
    assert_eq!(Wram::new(&mut bytes).unwrap().word(BLOCK_X), 1);
}

#[test]
fn invalid_tile_extent_is_error_not_speculative_refusal() {
    let mut bytes = fixture();
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(UPDATE_X, 0x7fef);
    assert!(row(&mut r, false).unwrap_err().contains("outside full WRAM"));
    assert_eq!(r.word(ROW), 64);
}

#[test]
fn frozen_root_preserves_all_state_without_a_service() {
    let mut bytes = fixture();
    Wram::new(&mut bytes).unwrap().set_word(FROZEN, 1);
    let before = bytes.clone();
    assert_eq!(run(Entry::CalculateLayer2PosAndScrollsWhenScrolling, &mut bytes, &mut NoFlush), Ok(Outcome::Handled));
    assert_eq!(bytes, before);
}
