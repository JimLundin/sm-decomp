pub use port::{WRAM_SIZE, Wram, room};
#[path = "../src/room_scroll.rs"]
mod room_scroll;

fn fixture() -> Vec<u8> {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0x7a5, 32);
    for (index, value) in [0x0011, 0x0022, 0x0033, 0x0044].into_iter().enumerate() {
        ram.set_word(0xa000 + 8 + index * 2, value);
    }
    for index in 0..0x8000 {
        ram.set_word(0x10000 + index * 2, 1);
    }
    bytes
}

#[test]
fn column_and_row_preserve_all_four_native_flip_orders() {
    for (flip, expected) in [
        (0, [0x11, 0x22, 0x33, 0x44]),
        (0x400, [0x4022, 0x4011, 0x4044, 0x4033]),
        (0x800, [0x8033, 0x8044, 0x8011, 0x8022]),
        (0xc00, [0xc044, 0xc033, 0xc022, 0xc011]),
    ] {
        let mut bytes = fixture();
        let mut ram = Wram::new(&mut bytes).unwrap();
        ram.set_word(0x10002, 1 | flip);
        room_scroll::dispatch_rom(&mut ram, &[], "UploadLevelDataColumn", 0, 0).unwrap();
        assert_eq!([ram.word(0xc8c8), ram.word(0xc908), ram.word(0xc8ca), ram.word(0xc90a)], expected);
        room_scroll::dispatch_rom(&mut ram, &[], "UpdateLevelDataRow", 0, 0).unwrap();
        assert_eq!([ram.word(0xc948), ram.word(0xc94a), ram.word(0xc98c), ram.word(0xc98e)], expected);
    }
}

#[test]
fn background_buffers_keep_native_264_byte_offset_and_foreground_unchanged() {
    let mut bytes = fixture();
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0x98e, 0x800);
    ram.set_word(0x994, 17);
    ram.set_word(0x996, 3);
    ram.set_word(0xc8c8, 0xbeef);
    room_scroll::dispatch_rom(&mut ram, &[], "UploadBackgroundDataColumn", 0, 0).unwrap();
    assert_eq!(ram.word(0xc8c8), 0xbeef);
    assert_eq!(ram.word(0xc9d0), 0x11);
    assert_eq!(ram.word(0x972), 52);
    assert_eq!(ram.word(0x974), 12);
    assert_eq!(ram.word(0x976), 0x4cc2);
    assert_eq!(ram.word(0x978), 0x4c02);
    assert_eq!(ram.word(0x97a), 0xca04);
    assert_eq!(ram.word(0x97c), 0xca44);
}

#[test]
fn row_contains_seventeen_blocks_and_keeps_wrapped_and_unwrapped_lengths() {
    let mut bytes = fixture();
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0x994, 15);
    ram.set_word(0x996, 2);
    room_scroll::dispatch_rom(&mut ram, &[], "UpdateLevelDataRow", 0, 0).unwrap();
    assert_eq!((ram.word(0x964), ram.word(0x966)), (4, 64));
    assert_eq!((ram.word(0x968), ram.word(0x96a)), (0x509e, 0x5480));
    assert_eq!((ram.word(0x96c), ram.word(0x96e)), (0xc94c, 0xc990));
    assert_eq!(ram.word(0xc988), 0x11);
    assert_eq!(ram.word(0xc9cc), 0x33);
}

#[test]
fn column_uses_low_byte_product_and_wrapping_word_stride() {
    let mut bytes = fixture();
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0x7a5, 0x8001);
    ram.set_word(0x992, 0x0101);
    ram.set_word(0x10004, 0x0401);
    ram.set_word(0x10006, 0x0801);
    room_scroll::dispatch_rom(&mut ram, &[], "UploadLevelDataColumn", 0, 0).unwrap();
    assert_eq!(ram.word(0xc8c8), 0x4022);
    assert_eq!(ram.word(0xc8cc), 0x8033);
}

#[test]
fn mode7_and_frozen_frame_do_not_write_queue_or_tile_buffers() {
    for (symbol, flag) in [("UploadLevelDataColumn", 0x783), ("CalculateLayer2PosAndScrollsWhenScrolling", 0xa78)] {
        let mut bytes = fixture();
        {
            let mut ram = Wram::new(&mut bytes).unwrap();
            ram.set_word(flag, 1);
            ram.set_word(0x990, 0x7fff);
        }
        let before = bytes.clone();
        {
            let mut ram = Wram::new(&mut bytes).unwrap();
            room_scroll::dispatch_rom(&mut ram, &[], symbol, 0, 0).unwrap();
        }
        assert_eq!(bytes, before);
    }
}

#[test]
fn rejects_full_frame_tile_extent_before_publishing_registers_or_previous_blocks() {
    let mut bytes = fixture();
    {
        let mut ram = Wram::new(&mut bytes).unwrap();
        ram.set_word(0x911, 32);
        ram.set_word(0x915, 0x0800);
        ram.set_word(0x7a5, 255);
    }
    let before = bytes.clone();
    {
        let mut ram = Wram::new(&mut bytes).unwrap();
        assert!(!room_scroll::supports(&ram, "CalculateLayer2PosAndScrollsWhenScrolling", 0));
        assert!(room_scroll::dispatch_rom(&mut ram, &[], "CalculateLayer2PosAndScrollsWhenScrolling", 0, 0).is_err());
    }
    assert_eq!(bytes, before);
}

#[test]
fn full_frame_schedules_foreground_and_background_x_then_y() {
    let mut bytes = fixture();
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0x911, 32);
    ram.set_word(0x915, 48);
    room_scroll::dispatch_rom(&mut ram, &[], "CalculateLayer2PosAndScrollsWhenScrolling", 0, 0).unwrap();
    for offset in [0x962, 0x970, 0x97e, 0x98c] {
        assert_eq!(ram.word(offset), 1);
    }
    assert_eq!((ram.word(0x8ff), ram.word(0x901), ram.word(0x903), ram.word(0x905)), (2, 3, 2, 3));
    assert_eq!((ram.word(0x990), ram.word(0x992), ram.word(0x994), ram.word(0x996)), (2, 18, 2, 18));
    assert_eq!((ram.word(0xb1), ram.word(0xb3), ram.word(0xb5), ram.word(0xb7)), (32, 48, 32, 48));
}

struct Flush(Vec<(u16, u16)>);
impl room_scroll::Host for Flush {
    fn flush_vram(&mut self, ram: &mut Wram<'_>) -> Result<(), &'static str> {
        self.0.push((ram.word(0x8f7), ram.word(0x990)));
        assert_eq!(ram.word(0x962) & 0xff, 1);
        for offset in [0x962, 0x970, 0x97e, 0x98c] {
            ram.set_byte(offset, 0);
        }
        ram.set_word(0x330, 0);
        Ok(())
    }
}

#[test]
fn full_display_flushes_seventeen_columns_before_incrementing_each_block() {
    let mut bytes = fixture();
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_byte(0x58, 0x51);
    ram.set_byte(0x59, 0x49);
    let mut host = Flush(Vec::new());
    room_scroll::dispatch_host(&mut ram, &[], &mut host, "DisplayViewablePartOfRoom", 0, 0).unwrap();
    assert_eq!(host.0, (0..17).map(|i| (i, i)).collect::<Vec<_>>());
    assert_eq!(ram.word(0x98e), 0x800);
    assert_eq!(ram.word(0x8f7), 17);
    assert_eq!(ram.word(0x97e), 0);
}

#[test]
fn full_display_rejects_all_columns_before_first_flush_or_write() {
    let mut bytes = fixture();
    {
        let mut ram = Wram::new(&mut bytes).unwrap();
        ram.set_word(0x7a5, 255);
        ram.set_word(0x915, 0x7ff0);
    }
    let before = bytes.clone();
    let mut host = Flush(Vec::new());
    {
        let mut ram = Wram::new(&mut bytes).unwrap();
        assert!(room_scroll::dispatch_host(&mut ram, &[], &mut host, "DisplayViewablePartOfRoom", 0, 0).is_err());
    }
    assert!(host.0.is_empty());
    assert_eq!(bytes, before);
}
