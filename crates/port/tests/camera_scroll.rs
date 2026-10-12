pub use port::{WRAM_SIZE, Wram, room, samus};
#[path = "../src/camera_scroll.rs"]
mod camera_scroll;

fn fixture() -> Vec<u8> {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    for (offset, value) in [
        (0x7a9, 16),
        (0x7ab, 16),
        (0x7ad, 112),
        (0x7af, 144),
        (0x911, 256),
        (0x915, 256),
        (0xaf6, 416),
        (0xafa, 368),
        (0xb10, 416),
        (0xb14, 368),
        (0xda2, 1),
        (0xda6, 1),
    ] {
        ram.set_word(offset, value);
    }
    ram.set_byte(0xa1e, 4);
    for index in 0..256 {
        ram.set_byte(0xcd20 + index, 2);
    }
    bytes
}

#[test]
fn completed_frame_calculates_fractional_distance_and_publishes_previous_positions() {
    let mut bytes = fixture();
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0xaf6, 420);
    ram.set_word(0xaf8, 0x1000);
    ram.set_word(0xb12, 0xf000);
    ram.set_word(0xafa, 370);
    ram.set_word(0xafc, 0x8000);
    ram.set_word(0xb16, 0);
    assert_eq!(camera_scroll::dispatch_rom(&mut ram, &[], "MainScrollingRoutine", 0, 0), Ok(0));
    assert_eq!((ram.word(0xda2), ram.word(0xda4)), (4, 0x2000));
    assert_eq!((ram.word(0xda6), ram.word(0xda8)), (3, 0x8000));
    assert_eq!((ram.word(0x911), ram.word(0x90f)), (260, 0x2000));
    assert_eq!((ram.word(0x915), ram.word(0x913)), (258, 0));
    assert_eq!((ram.word(0xb10), ram.word(0xb12)), (420, 0x1000));
    assert_eq!((ram.word(0xb14), ram.word(0xb16)), (370, 0x8000));
}

#[test]
fn slow_scroll_uses_unsigned_windows_and_keeps_fractions_and_distance_words() {
    let mut bytes = fixture();
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0xcf8, 1);
    ram.set_word(0xa78, 1);
    ram.set_word(0xaf6, 300);
    ram.set_word(0xafa, 400);
    ram.set_word(0x90f, 0x1234);
    ram.set_word(0x913, 0xabcd);
    ram.set_word(0xda2, 99);
    ram.set_word(0xda6, 77);
    assert_eq!(camera_scroll::dispatch_rom(&mut ram, &[], "MainScrollingRoutine", 0, 0), Ok(0));
    assert_eq!((ram.word(0x911), ram.word(0x915)), (253, 259));
    assert_eq!((ram.word(0x90f), ram.word(0x913)), (0x1234, 0xabcd));
    assert_eq!((ram.word(0xda2), ram.word(0xda6)), (99, 77));
}

#[test]
fn signed_high_x_skips_both_manual_axes_but_signed_high_y_only_skips_y() {
    for (x, y, expected) in [(0x8000, 400, (256, 256)), (500, 0x8000, (259, 256))] {
        let mut bytes = fixture();
        let mut ram = Wram::new(&mut bytes).unwrap();
        ram.set_word(0xcf8, 1);
        ram.set_word(0xa78, 1);
        ram.set_word(0xaf6, x);
        ram.set_word(0xafa, y);
        camera_scroll::dispatch_rom(&mut ram, &[], "MainScrollingRoutine", 0, 0).unwrap();
        assert_eq!((ram.word(0x911), ram.word(0x915)), expected);
    }
}

#[test]
fn native_x_table_uses_byte_fields_xor_and_accepts_odd_indices() {
    for (direction, movement, knockback, accel, index, distance) in [
        (4, 0, 0, 0, 0, 160),
        (8, 0, 0, 0, 0, 96),
        (4, 16, 0, 0, 3, 64),
        (8, 16, 0, 0, 3, 80),
        (4, 0, 1, 0, 0, 96),
        (4, 0, 0, 1, 7, 224),
    ] {
        let mut bytes = fixture();
        let mut ram = Wram::new(&mut bytes).unwrap();
        ram.set_word(0xaf6, 600);
        ram.set_word(0xda2, 1);
        ram.set_byte(0xa1e, direction);
        ram.set_byte(0xa1f, movement);
        ram.set_word(0xa52, knockback);
        ram.set_word(0xb4a, accel);
        ram.set_word(0x941, index);
        camera_scroll::dispatch_rom(&mut ram, &[], "Samus_HandleScroll_X", 0, 0).unwrap();
        assert_eq!(ram.word(0xb0a), 600 - distance);
        assert_eq!(ram.word(0x911), 257);
    }
}

#[test]
fn fraction_only_movement_uses_automatic_scroll_without_reading_the_table() {
    let mut bytes = fixture();
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0x941, 0xffff);
    ram.set_word(0xaf8, 0x8000);
    ram.set_word(0xafc, 0x4000);
    ram.set_word(0xb0a, 0xabcd);
    ram.set_word(0xb0e, 0x1234);
    assert!(camera_scroll::supports(&ram, "MainScrollingRoutine"));
    camera_scroll::dispatch_rom(&mut ram, &[], "MainScrollingRoutine", 0, 0).unwrap();
    assert_eq!((ram.word(0x911), ram.word(0x915)), (256, 256));
    assert_eq!((ram.word(0xb0a), ram.word(0xb0e)), (0xabcd, 0x1234));
    assert_eq!((ram.word(0xda2), ram.word(0xda4)), (1, 0x8000));
    assert_eq!((ram.word(0xda6), ram.word(0xda8)), (1, 0x4000));
}

#[test]
fn spore_spawn_hook_follows_scrolling_and_preserves_fraction() {
    let mut bytes = fixture();
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0x7e9, 0x9589);
    ram.set_word(0x913, 0xbeef);
    camera_scroll::dispatch_rom(&mut ram, &[], "MainScrollingRoutine", 0, 0).unwrap();
    assert_eq!((ram.word(0x915), ram.word(0x913)), (464, 0xbeef));
    assert_eq!(ram.word(0xb14), 368);
    ram.set_word(0x915, 465);
    camera_scroll::dispatch_rom(&mut ram, &[], "Samus_ScrollFinishedHook_SporeSpawnFight", 0, 0).unwrap();
    assert_eq!(ram.word(0x915), 465);
}

#[test]
fn y_constraints_use_the_column_published_by_x_in_the_same_frame() {
    let mut bytes = fixture();
    let mut ram = Wram::new(&mut bytes).unwrap();
    for (offset, value) in [
        (0x7a9, 4),
        (0x7ab, 4),
        (0x911, 383),
        (0x90f, 0),
        (0x915, 384),
        (0xaf6, 546),
        (0xb10, 545),
        (0xafa, 496),
        (0xb14, 496),
    ] {
        ram.set_word(offset, value);
    }
    // X crosses x+128's column boundary. Only the newly selected Y cell is red.
    ram.set_byte(0xcd20 + 6, 0);
    camera_scroll::dispatch_rom(&mut ram, &[], "MainScrollingRoutine", 0, 0).unwrap();
    assert_eq!(ram.word(0x911), 385);
    assert_eq!(ram.word(0x915), 387);
}

#[test]
fn refusing_any_dependency_leaves_full_wram_unchanged() {
    for writes in [
        vec![(0x7e9, 0x9999)],
        vec![(0xaf6, 420), (0x941, 8)],
        vec![(0x7a9, 0)],
        vec![(0x7ab, 17)],
        vec![(0xaf6, 420), (0x915, 0xff00), (0x90f, 0xffff)],
    ] {
        let mut bytes = fixture();
        {
            let mut ram = Wram::new(&mut bytes).unwrap();
            for (offset, value) in writes {
                ram.set_word(offset, value);
            }
        }
        let before = bytes.clone();
        {
            let mut ram = Wram::new(&mut bytes).unwrap();
            assert!(!camera_scroll::supports(&ram, "MainScrollingRoutine"));
            assert!(camera_scroll::dispatch_rom(&mut ram, &[], "MainScrollingRoutine", 0, 0).is_err());
        }
        assert_eq!(bytes, before);
    }
}
