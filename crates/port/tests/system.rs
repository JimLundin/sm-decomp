use port::{Wram, system};

#[test]
fn held_buttons_wait_then_report_new_edges_once() {
    let mut bytes = vec![0; port::WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0x8b, 0x0200);
    ram.set_word(0x5d9, 0x0200);
    ram.set_word(0x5db, 1);
    ram.set_word(0x5df, 0x0100);
    system::dispatch(&mut ram, "ReleaseButtonsFilter", 12, 0).unwrap();
    assert_eq!(ram.word(0x5db), 0);
    assert_eq!(ram.word(0x5df), 0);
    system::dispatch(&mut ram, "ReleaseButtonsFilter", 12, 0).unwrap();
    assert_eq!(ram.word(0x5db), 0);
    assert_eq!(ram.word(0x5df), 0x0200);
    assert_eq!(ram.word(0x5e3), 0);
    assert_eq!(ram.word(0x5e1), 0x0200);
    system::dispatch(&mut ram, "ReleaseButtonsFilter", 12, 0).unwrap();
    assert_eq!(ram.word(0x5e3), 0x0200);
    assert_eq!(ram.word(0x5e1), 0);
}

#[test]
fn released_buttons_reset_timer_when_release_mask_changes() {
    let mut bytes = vec![0xa5; port::WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0x8b, 0x0300);
    ram.set_word(0x8f, 0x0100);
    ram.set_word(0x5d9, 0);
    ram.set_word(0x5e3, 0x8000);
    system::dispatch(&mut ram, "ReleaseButtonsFilter", 12, 0).unwrap();
    assert_eq!(ram.word(0x5d9), 0x0200);
    assert_eq!(ram.word(0x5db), 12);
    assert_eq!(ram.word(0x5dd), 12);
    assert_eq!(ram.word(0x5df), 0);
    assert_eq!(ram.word(0x5e1), 0);
    assert_eq!(ram.word(0x5e3), 0x8000);
}

#[test]
fn event_access_sets_scratch_mask_and_preserves_adjacent_boss_state() {
    let mut bytes = vec![0; port::WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_byte(0xd827, 0x01);
    ram.set_byte(0xd828, 0xa5);
    assert_eq!(system::dispatch(&mut ram, "PrepareBitAccess", 0xffff, 0), Ok(8191));
    assert_eq!(ram.word(0x5e7), 0x80);
    system::dispatch(&mut ram, "SetEventHappened", 63, 0).unwrap();
    assert_eq!(ram.byte(0xd827), 0x81);
    assert_eq!(system::dispatch(&mut ram, "CheckEventHappened", 63, 0), Ok(1));
    system::dispatch(&mut ram, "ClearEventHappened", 63, 0).unwrap();
    assert_eq!(ram.byte(0xd827), 1);
    assert_eq!(system::dispatch(&mut ram, "CheckEventHappened", 63, 0), Ok(0));
    assert_eq!(ram.word(0x5e7), 0x80);
    assert_eq!(ram.byte(0xd828), 0xa5);
    assert!(system::dispatch(&mut ram, "SetEventHappened", 64, 0).is_err());
}

#[test]
fn boss_bits_truncate_updates_to_bytes_but_accept_wide_masks() {
    let mut bytes = vec![0; port::WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0x79f, 7);
    ram.set_byte(0xd82e, 0xa5);
    ram.set_byte(0xd82f, 2);
    system::dispatch(&mut ram, "SetBossBitForCurArea", 0xff04, 0).unwrap();
    assert_eq!(ram.byte(0xd82f), 6);
    assert_eq!(system::dispatch(&mut ram, "CheckBossBitForCurArea", 0xff00, 0), Ok(0));
    assert_eq!(system::dispatch(&mut ram, "CheckBossBitForCurArea", 0xff04, 0), Ok(1));
    system::dispatch(&mut ram, "ClearBossBitForCurArea", 0xff02, 0).unwrap();
    assert_eq!(ram.byte(0xd82f), 4);
    assert_eq!(ram.byte(0xd82e), 0xa5);
    ram.set_word(0x79f, 8);
    assert!(system::dispatch(&mut ram, "SetBossBitForCurArea", 1, 0).is_err());
}

#[test]
fn multiply_returns_full_unsigned_product_without_removed_scratch_writes() {
    let mut bytes = vec![0xa5; port::WRAM_SIZE];
    let before = bytes.clone();
    let mut ram = Wram::new(&mut bytes).unwrap();
    assert_eq!(system::dispatch(&mut ram, "Multiply16x16", 0xffff, 0xffff), Ok(0xfffe0001));
    assert_eq!(bytes, before);
}

#[test]
fn fade_preserves_upstream_signed_counter_and_brightness_quirks() {
    let mut bytes = vec![0; port::WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0x723, 3);
    ram.set_word(0x725, 0x8000);
    ram.set_byte(0x51, 0x8f);
    system::dispatch(&mut ram, "HandleFadeOut", 0, 0).unwrap();
    assert_eq!(ram.word(0x725), 0x7fff);
    assert_eq!(ram.byte(0x51), 0x8f);
    ram.set_word(0x725, 0x8001);
    system::dispatch(&mut ram, "HandleFadeOut", 0, 0).unwrap();
    assert_eq!(ram.word(0x725), 3);
    assert_eq!(ram.byte(0x51), 14);
    ram.set_word(0x725, 0);
    ram.set_byte(0x51, 1);
    system::dispatch(&mut ram, "HandleFadeOut", 0, 0).unwrap();
    assert_eq!(ram.byte(0x51), 0x80);
    ram.set_word(0x725, 0);
    system::dispatch(&mut ram, "HandleFadeIn", 0, 0).unwrap();
    assert_eq!(ram.byte(0x51), 1);
    ram.set_word(0x725, 0);
    ram.set_byte(0x51, 0x8f);
    system::dispatch(&mut ram, "HandleFadeIn", 0, 0).unwrap();
    assert_eq!(ram.byte(0x51), 0x8f);
}

#[test]
fn unused_oam_clears_only_y_coordinates_after_flooring_cursor() {
    let mut bytes = vec![0xa5; port::WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0x590, 7);
    system::dispatch(&mut ram, "ClearUnusedOam", 0, 0).unwrap();
    assert_eq!(ram.word(0x590), 0);
    assert_eq!(ram.byte(0x371), 0xa5);
    assert_eq!(ram.byte(0x375), 0xf0);
    assert_eq!(ram.byte(0x56d), 0xf0);
    assert_eq!(ram.byte(0x56e), 0xa5);
    system::dispatch(&mut ram, "ClearOamExt", 0, 0).unwrap();
    for offset in 0x570..0x590 {
        assert_eq!(ram.byte(offset), 0);
    }
    assert_eq!(ram.byte(0x56f), 0xa5);
    assert_eq!(ram.byte(0x592), 0xa5);
}

#[test]
fn map_fill_writes_exactly_1024_little_endian_words_per_map() {
    let mut bytes = vec![0xa5; port::WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    system::dispatch(&mut ram, "sub_8088EB", 0x1234, 0).unwrap();
    assert_eq!(ram.word(0x3000), 0x1234);
    assert_eq!(ram.word(0x37fe), 0x1234);
    assert_eq!(ram.word(0x3800), 0xa5a5);
    system::dispatch(&mut ram, "sub_8088FE", 0x5678, 0).unwrap();
    assert_eq!(ram.word(0x47fe), 0x5678);
    system::dispatch(&mut ram, "sub_808911", 0x9abc, 0).unwrap();
    assert_eq!(ram.word(0x67fe), 0x9abc);
    system::dispatch(&mut ram, "WriteLotsOf0x1c2f", 0, 0).unwrap();
    for offset in [0x3000, 0x37fe, 0x4000, 0x47fe, 0x6000, 0x67fe] {
        assert_eq!(ram.word(offset), 0x1c2f);
    }
    for offset in [0x2ffe, 0x3800, 0x3ffe, 0x4800, 0x5ffe, 0x6800] {
        assert_eq!(ram.word(offset), 0xa5a5);
    }
}
