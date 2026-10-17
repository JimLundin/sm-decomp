use port::{WRAM_SIZE, Wram};

#[path = "../src/projectile_trails.rs"]
mod projectile_trails;

fn write_script(rom: &mut [u8], pointer: u16, words: &[u16]) {
    let offset = 0x80000 + usize::from(pointer & 0x7fff);
    for (index, word) in words.iter().enumerate() {
        rom[offset + index * 2..offset + index * 2 + 2].copy_from_slice(&word.to_le_bytes());
    }
}

fn trail(r: &mut Wram<'_>, k: usize, right: bool, timer: u16, x: u16, y: u16, tile: u16) {
    let side = usize::from(right) * 36;
    r.set_word(0xd658 + side + k, timer);
    r.set_word(0xd6a0 + side + k, 0x9000);
    r.set_word(0xd6e8 + side + k, tile);
    r.set_word(0xd730 + side + k, x);
    r.set_word(0xd778 + side + k, y);
}

#[test]
fn scripts_wrap_movement_and_terminate_without_consuming_a_tile_record() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut rom = vec![0; 0x88000];
    write_script(&mut rom, 0x9000, &[0xb525, 0xb587, 0xb5b3, 1, 0xbeef, 0]);
    let mut r = Wram::new(&mut bytes).unwrap();
    trail(&mut r, 34, false, 1, 20, 0xffff, 0x1234);
    trail(&mut r, 34, true, 2, 30, 0xffff, 0x5678);
    projectile_trails::handle(&mut r, &rom).unwrap();
    assert_eq!(r.word(0xd778 + 34), 0xffff);
    assert_eq!(r.word(0xd79c + 34), 0);
    assert_eq!(r.word(0xd6e8 + 34), 0xbeef);
    assert_eq!(r.word(0xd6a0 + 34), 0x900a);
    assert_eq!([r.byte(0x370), r.byte(0x371), r.byte(0x372), r.byte(0x373)], [30, 0, 0x78, 0x56]);
    assert_eq!(r.word(0x590), 4);
    r.set_word(0xd67c + 34, 0);
    projectile_trails::handle(&mut r, &rom).unwrap();
    assert_eq!(r.word(0xd658 + 34), 0);
    assert_eq!(r.word(0xd6a0 + 34), 0x900a);
    assert_eq!(r.word(0xd6e8 + 34), 0xbeef);
}

#[test]
fn vertically_clipped_trail_preserves_partial_x_word_without_advancing_cursor() {
    let mut bytes = vec![0xa5; WRAM_SIZE];
    bytes[0xd658..0xd6a0].fill(0);
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0xa78, 0);
    r.set_word(0x911, 0);
    r.set_word(0x915, 0);
    r.set_word(0x590, 0);
    trail(&mut r, 34, false, 2, 42, 256, 0xcdef);
    projectile_trails::handle(&mut r, &[]).unwrap();
    assert_eq!([r.byte(0x370), r.byte(0x371), r.byte(0x372), r.byte(0x373)], [42, 0, 0xa5, 0xa5]);
    assert_eq!(r.word(0x590), 0);
    assert_eq!(r.word(0xd658 + 34), 1);
}

#[test]
fn reverse_slot_order_draws_left_before_right_and_preserves_oam_extension_bits() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0xa78, 1);
    r.set_word(0x570, 0xabcd);
    for (k, x) in [(34, 10), (0, 30)] {
        trail(&mut r, k, false, 1, x, 20, 0x1234);
        trail(&mut r, k, true, 1, x + 1, 21, 0x5678);
    }
    projectile_trails::handle(&mut r, &[]).unwrap();
    assert_eq!([r.byte(0x370), r.byte(0x374), r.byte(0x378), r.byte(0x37c)], [10, 11, 30, 31]);
    assert_eq!(r.word(0x590), 16);
    assert_eq!(r.word(0x570), 0xabcd);
    assert_eq!(r.word(0xd658 + 34), 1);
    assert_eq!(r.word(0xd6a0 + 34), 0x9000);
}

#[test]
fn unaligned_cursor_writes_metadata_in_native_order_and_captures_initial_cursor() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0xa78, 1);
    r.set_word(0x590, 511);
    r.set_word(0x570, 0xa5a5);
    trail(&mut r, 34, false, 1, 7, 8, 0xabcd);
    trail(&mut r, 34, true, 1, 9, 10, 0xbcde);
    projectile_trails::handle(&mut r, &[]).unwrap();
    assert_eq!(r.byte(0x56f), 7);
    assert_eq!(r.byte(0x570), 8);
    assert_eq!(r.word(0x571), 0xabcd);
    assert_eq!(r.word(0x590), 515);
    assert_eq!(r.word(0xd67c + 34), 1);
}

#[test]
fn wrapped_oam_cursor_can_enable_a_later_slot_and_aliases_are_read_after_stores() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0xa78, 1);
    // 0xD658 - 0x370 = 0xD2E8. Native signed capacity subtraction admits
    // this cursor. The left store creates a timer for slot0 visited later.
    r.set_word(0x590, 0xd2e8);
    trail(&mut r, 34, false, 1, 1, 0, 0);
    r.set_word(0xd730, 2);
    r.set_word(0xd778, 3);
    r.set_word(0xd6e8, 0x4567);
    projectile_trails::handle(&mut r, &[]).unwrap();
    assert_eq!(r.word(0xd658), 1);
    assert_eq!(r.word(0xd65c), 0x0302);
    assert_eq!(r.word(0xd65e), 0x4567);
    assert_eq!(r.word(0x590), 0xd2f0);
}

#[test]
fn unsupported_later_script_rolls_back_previous_timer_movement_and_oam_writes() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut rom = vec![0; 0x88000];
    write_script(&mut rom, 0x9000, &[0xdead]);
    {
        let mut r = Wram::new(&mut bytes).unwrap();
        trail(&mut r, 34, false, 2, 10, 20, 0xbeef);
        trail(&mut r, 0, true, 1, 11, 21, 0xcafe);
        assert!(!projectile_trails::supports_call(&r, &rom, "HandleProjectileTrails", 0, 0).unwrap());
    }
    let before = bytes.clone();
    assert!(projectile_trails::handle(&mut Wram::new(&mut bytes).unwrap(), &rom).is_err());
    assert_eq!(bytes, before);
}

#[test]
fn frozen_frames_ignore_invalid_scripts_and_camera_wrap_reaches_pixel255() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0xa78, 0xffff);
    r.set_word(0x911, 0xff00);
    r.set_word(0x915, 0xffff);
    trail(&mut r, 0, true, 1, 0xffff, 0, 0x1234);
    r.set_word(0xd6c4, 0);
    assert!(projectile_trails::supports_call(&r, &[], "HandleProjectileTrails", 0, 0).unwrap());
    projectile_trails::handle(&mut r, &[]).unwrap();
    assert_eq!([r.byte(0x370), r.byte(0x371)], [255, 1]);
    assert_eq!(r.word(0xd67c), 1);
    assert_eq!(r.word(0xd6c4), 0);
}

#[test]
fn data_record_uses_physical_contiguous_word_then_wraps_its_saved_pointer() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut rom = vec![0; 0x88004];
    write_script(&mut rom, 0xfffe, &[2, 0xbeef]);
    let mut r = Wram::new(&mut bytes).unwrap();
    trail(&mut r, 0, false, 1, 0, 0, 0);
    r.set_word(0xd6a0, 0xfffe);
    projectile_trails::handle(&mut r, &rom).unwrap();
    assert_eq!(r.word(0xd6a0), 2);
    assert_eq!(r.word(0xd6e8), 0xbeef);
    assert_eq!(r.word(0x372), 0xbeef);
}

#[test]
fn direct_opcode_odd_slots_floor_and_wide_arguments_refuse_before_mutation() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0xd778 + 34, 0xffff);
    projectile_trails::dispatch_rom(&mut r, &[], "ProjInstr_MoveLeftProjectileTrailDown", 35, 0).unwrap();
    assert_eq!(r.word(0xd778 + 34), 0);
    for raw in [-65536, -1, 36, 65536] {
        assert!(projectile_trails::dispatch_rom(&mut r, &[], "ProjInstr_MoveLeftProjectileTrailDown", raw, 0).is_err());
        assert_eq!(r.word(0xd778 + 34), 0);
    }
}

#[test]
fn capacity_at512_still_updates_scripts_and_zero_timers_remain_zero() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut rom = vec![0; 0x88000];
    write_script(&mut rom, 0x9000, &[0xb525, 3, 0xbeef]);
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x590, 512);
    trail(&mut r, 34, false, 1, 5, 6, 0);
    projectile_trails::handle(&mut r, &rom).unwrap();
    assert_eq!(r.word(0xd778 + 34), 7);
    assert_eq!(r.word(0xd658 + 34), 3);
    assert_eq!(r.word(0xd658), 0);
    assert_eq!(r.word(0x370), 0);
    assert_eq!(r.word(0x590), 512);
}
