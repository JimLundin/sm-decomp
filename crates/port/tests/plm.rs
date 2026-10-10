use port::{WRAM_SIZE, Wram, plm};
#[test]
fn quicksand_clamps_fractional_displacement_and_tracks_contact() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0xb02, 3);
    let mut amount = 0x100aa;
    assert_eq!(plm::setup(&mut ram, 0xb4c4, 78, &mut amount), Ok(0));
    assert_eq!(amount, 0x30aa);
    assert_eq!(ram.word(0x1e71), 1);
    ram.set_word(0xa6e, 1);
    assert_eq!(plm::setup(&mut ram, 0xb4c4, 78, &mut amount), Ok(1));
    assert_eq!(amount, 0);
}
#[test]
fn native_spawn_uses_highest_free_slot_and_rejects_unknown_before_mutation() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    let mut rom = vec![0; 0x300000];
    let off = (4 << 15) + 0x372b;
    rom[off..off + 4].copy_from_slice(&[0xc4, 0xb4, 0xe3, 0xaa]);
    ram.set_word(0xdc4, 16);
    ram.set_word(0xb02, 3);
    let mut amount = 0x10000;
    assert_eq!(plm::spawn(&mut ram, &rom, 0xb72b, &mut amount), Ok(0));
    assert_eq!(amount, 0x3000);
    assert_eq!(ram.word(0x1c37 + 78), 0xb72b);
    assert_eq!(ram.word(0x1c87 + 78), 32);
    assert_eq!(ram.word(0x1d27 + 78), 0xaae3);
    let before: Vec<_> = (0..WRAM_SIZE).map(|i| ram.byte(i)).collect();
    rom[off..off + 2].copy_from_slice(&[0xff, 0xff]);
    assert!(plm::spawn(&mut ram, &rom, 0xb72b, &mut amount).is_err());
    assert_eq!(before, (0..WRAM_SIZE).map(|i| ram.byte(i)).collect::<Vec<_>>());
}
#[test]
fn occupied_pool_does_not_read_unknown_header_or_change_displacement() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    for j in (0..80).step_by(2) {
        ram.set_word(0x1c37 + j, 0xffff);
    }
    let mut amount = -1;
    assert_eq!(plm::spawn(&mut ram, &[], 0, &mut amount), Ok(0));
    assert_eq!(amount, -1);
}

#[test]
fn supported_setup_failure_commits_neither_slot_nor_displacement() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    let mut rom = vec![0; 0x300000];
    let off = (4 << 15) + 0x1000;
    rom[off..off + 4].copy_from_slice(&[0x4a, 0xb0, 0x00, 0x80]);
    ram.set_word(0xdc4, 0x63ff);
    ram.set_word(0x1c27, 18);
    let before: Vec<_> = (0..WRAM_SIZE).map(|i| ram.byte(i)).collect();
    let mut amount = -123;
    assert_eq!(plm::supports_header(&rom, 0x9000), Ok(true));
    assert_eq!(plm::spawn(&mut ram, &rom, 0x9000, &mut amount), Err("room row extent"));
    assert_eq!(before, (0..WRAM_SIZE).map(|i| ram.byte(i)).collect::<Vec<_>>());
    assert_eq!(amount, -123);
}

#[test]
fn room_spawn_preserves_arguments_and_room_preinstruction() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    let mut rom = vec![0; 0x300000];
    let header = (4 << 15) + 0x1000;
    rom[header..header + 4].copy_from_slice(&[0x27, 0xab, 0x00, 0x80]);
    ram.set_word(0x7a5, 0x110);
    ram.set_word(0x12, 0x9000);
    ram.set_byte(0x14, 3);
    ram.set_byte(0x15, 2);
    ram.set_word(0x16, 0x1234);
    plm::spawn_room(&mut ram, &rom, 0x12).unwrap();
    assert_eq!(ram.word(0x1c87 + 78), 70);
    assert_eq!(ram.word(0x1dc7 + 78), 0x1234);
    assert_eq!(ram.word(0x1cd7 + 78), 0x84e6);
}

#[test]
fn hardcoded_spawn_and_full_pool_preserve_original_order() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    let mut rom = vec![0; 0x300000];
    let off = (4 << 15) + 0x1000;
    rom[off..off + 4].copy_from_slice(&[0x27, 0xab, 0x34, 0x92]);
    ram.set_word(0x7a5, 16);
    plm::spawn_hardcoded(&mut ram, &rom, 3, 2, 0x9000).unwrap();
    assert_eq!(ram.word(0x1c87 + 78), 70);
    assert_eq!(ram.word(0x1cd7 + 78), 0x8469);
    assert_eq!(ram.word(0x1d27 + 78), 0x9234);
    for j in (0..80).step_by(2) {
        ram.set_word(0x1c37 + j, 0xffff);
    }
    let before: Vec<_> = (0..WRAM_SIZE).map(|i| ram.byte(i)).collect();
    plm::spawn_room(&mut ram, &[], 0).unwrap();
    plm::spawn_hardcoded(&mut ram, &[], 255, 255, 0).unwrap();
    assert_eq!(before, (0..WRAM_SIZE).map(|i| ram.byte(i)).collect::<Vec<_>>());
}

#[test]
fn room_rom_entry_and_collision_only_setup_admission() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    let mut rom = vec![0; 0x300000];
    let off = (4 << 15) + 0x1000;
    rom[off..off + 4].copy_from_slice(&[0x27, 0xab, 0x00, 0x80]);
    let entry = (15 << 15) + 0x1000;
    rom[entry..entry + 6].copy_from_slice(&[0x00, 0x90, 4, 3, 0xcd, 0xab]);
    ram.set_word(0x7a5, 16);
    plm::spawn_room(&mut ram, &rom, 0x9000).unwrap();
    assert_eq!(ram.word(0x1c87 + 78), 104);
    assert_eq!(ram.word(0x1dc7 + 78), 0xabcd);
    rom[off..off + 2].copy_from_slice(&[0xc4, 0xb4]);
    assert_eq!(plm::supports_positioned_header(&rom, 0x9000), Ok(false));
    let before: Vec<_> = (0..WRAM_SIZE).map(|i| ram.byte(i)).collect();
    assert!(plm::spawn_room(&mut ram, &rom, 0x9000).is_err());
    assert_eq!(before, (0..WRAM_SIZE).map(|i| ram.byte(i)).collect::<Vec<_>>());
}

#[test]
fn hardcoded_sound_enable_timer_uses_area_and_original_preinstruction() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    let mut rom = vec![0; 0x300000];
    rom[0x237eb..0x237ef].copy_from_slice(&[0xc3, 0xb7, 0xe9, 0xb7]);
    ram.set_word(0x7a5, 16);
    ram.set_word(0x79f, 6);
    plm::spawn_hardcoded(&mut ram, &rom, 8, 8, 0xb7eb).unwrap();
    assert_eq!(ram.word(0x1d77 + 78), 32);
    assert_eq!(ram.word(0x1cd7 + 78), 0xb7dd);
}

#[test]
fn grey_door_extracts_condition_and_preserves_room_argument_bits() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0x1c87, 0x100);
    ram.set_word(0x1dc7, 0xfe35);
    ram.set_word(0x10102, 0x1234);
    plm::setup(&mut ram, 0xc794, 0, &mut 0).unwrap();
    assert_eq!(ram.word(0x1e17), 62);
    assert_eq!(ram.word(0x1dc7), 0x8235);
    assert_eq!(ram.word(0x10102), 0xc234);
    assert_eq!(ram.byte(0x16482), 0x44);
}

#[test]
fn blue_door_power_bomb_deletes_and_negative_projectile_sentinel_makes_solid() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0x1c87, 0x100);
    ram.set_word(0x1c37, 0xbeef);
    ram.set_word(0xc18, 0x300);
    plm::setup(&mut ram, 0xc7bb, 0, &mut 0).unwrap();
    assert_eq!(ram.word(0x1c37), 0);
    ram.set_word(0xc04, 0);
    ram.set_word(0xdde, 0xffff);
    ram.set_word(0x10102, 0xf234);
    plm::setup(&mut ram, 0xc7bb, 0, &mut 0).unwrap();
    assert_eq!(ram.word(0x10102), 0x8234);
}

#[test]
fn open_upward_gate_writes_five_bts_cells_and_keeps_adjacent_bytes() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0x1c87, 0x100);
    ram.set_word(0x7a5, 16);
    ram.set_byte(0x16483, 0xaa);
    plm::setup(&mut ram, 0xc6dc, 0, &mut 0).unwrap();
    for index in [128, 112, 96, 80, 64] {
        assert_eq!(ram.byte(0x16402 + index), 0x10);
    }
    assert_eq!(ram.byte(0x16483), 0xaa);
}

#[test]
fn escape_door_initializes_vertical_extensions_and_scroll_setup_resets_words() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0x1c87, 0x100);
    ram.set_word(0x7a5, 16);
    plm::setup(&mut ram, 0xb5f8, 0, &mut 0).unwrap();
    assert_eq!(ram.byte(0x10103), 0x90);
    assert_eq!(ram.byte(0x16482), 1);
    for index in [144, 160, 176] {
        assert_eq!(ram.byte(0x16402 + index), 0xff);
    }
    plm::setup(&mut ram, 0xb8dc, 0, &mut 0).unwrap();
    assert_eq!(ram.word(0xcd20), 1);
    assert_eq!(ram.word(0xcd22), 0);
}

#[test]
fn left_green_trigger_wakes_highest_adjacent_gate_without_resetting_timer() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0x1c87, 0x100);
    ram.set_word(0x1c37, 0xbeef);
    ram.set_word(0xc18, 0x200);
    ram.set_word(0x1c87 + 78, 0x102);
    assert_eq!(plm::setup(&mut ram, 0xc54d, 0, &mut 0), Ok(1));
    assert_eq!(ram.word(0x1d77 + 78), 1);
    assert_eq!(ram.word(0x1c37), 0);
    ram.set_word(0x1d77 + 78, 9);
    assert_eq!(plm::setup(&mut ram, 0xc54d, 0, &mut 0), Ok(1));
    assert_eq!(ram.word(0x1d77 + 78), 9);
}

#[test]
fn left_yellow_non_bomb_falls_through_and_wakes_opposite_gate() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0x1c87, 0x100);
    ram.set_word(0x1c87 + 78, 0xfe);
    ram.set_word(0xc18, 0x100);
    assert_eq!(plm::setup(&mut ram, 0xc5d3, 0, &mut 0), Ok(1));
    assert_eq!(ram.word(0x1d77 + 78), 1);
    assert_eq!(ram.word(0x1c37), 0);
}

#[test]
fn super_missile_door_wakeup_sets_hit_variable_and_link_timer() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0x1d77, 0x8200);
    ram.set_word(0xdebc, 0x9234);
    plm::preinstruction(&mut ram, 0xbd50, 0).unwrap();
    assert_eq!(ram.word(0x1d77), 0);
    assert_eq!(ram.word(0xdf0c), 119);
    assert_eq!(ram.word(0x1d27), 0x9234);
    assert_eq!(ram.word(0xde1c), 1);
}
