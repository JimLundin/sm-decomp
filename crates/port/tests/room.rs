use port::{WRAM_SIZE, Wram, room};
#[test]
fn pre_phantoon_room_spawn_clears_bg2_scroll_and_preserves_position() {
    let rom = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../../rom/sm.sfc")).unwrap();
    let mut b = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut b).unwrap();
    r.set_word(0x923, 0xffff);
    r.set_word(0x1a4b + 34, 0xabcd);
    room::dispatch_rom(&mut r, &rom, "RoomCode_8FC8C8", 0, 0).unwrap();
    assert_eq!(r.word(0x923), 0);
    assert_eq!(r.word(0x1997 + 34), 0xa3b0);
    assert_eq!(r.word(0x1a4b + 34), 0xabcd);
}

#[test]
fn dachora_camera_can_overshoot_gate_then_stops() {
    let mut b = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut b).unwrap();
    r.set_byte(0xcd2b, 2);
    r.set_word(0x915, 0x4ff);
    r.set_word(0x911, 0x37f);
    room::dispatch_rom(&mut r, &[], "RoomCode_ScrollRightDachora", 0, 0).unwrap();
    assert_eq!(r.word(0x911), 0x382);
    room::dispatch_rom(&mut r, &[], "RoomCode_ScrollRightDachora", 0, 0).unwrap();
    assert_eq!(r.word(0x911), 0x382);
}

#[test]
fn draygon_hooks_write_exact_long_pointers_and_test_full_hdma_word() {
    let mut b = vec![0xa5; WRAM_SIZE];
    let mut r = Wram::new(&mut b).unwrap();
    room::dispatch_rom(&mut r, &[], "RoomCode_SetPauseCodeForDraygon", 0, 0).unwrap();
    assert_eq!(r.word(0x601), 0xc8f6);
    assert_eq!(r.word(0x604), 0xc8fc);
    assert_eq!(r.byte(0x603), 0x8f);
    assert_eq!(r.byte(0x606), 0x8f);
    assert_eq!(r.byte(0x607), 0xa5);
    r.set_word(0x18b6, 0x108);
    assert_eq!(room::dispatch_rom(&mut r, &[], "UnpauseHook_DraygonRoom", 0, 0).unwrap(), 0);
    assert_eq!(r.word(0xa7), 0xa5a5);
    r.set_word(0x18b6, 8);
    room::dispatch_rom(&mut r, &[], "UnpauseHook_DraygonRoom", 0, 0).unwrap();
    assert_eq!(r.word(0xa7), 12);
}

#[test]
fn boss_room_shakes_reflect_negative_offset_and_complete_ridley_cycle() {
    let mut b = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut b).unwrap();
    r.set_word(0xfaa, 0x400);
    r.set_word(0xfee, 0xfff9);
    room::dispatch_rom(&mut r, &[], "RoomCode_CrocomireShaking", 0, 0).unwrap();
    assert_eq!(r.word(0xfee), 0xfff8);
    assert_eq!(r.word(0xb3), 0xfffa);
    assert_eq!(r.word(0xb7), 0xffca);
    r.set_word(0x10a8, 8);
    for address in [0xb1, 0xb3, 0xb5, 0xb7] {
        r.set_word(address, 0xffff);
    }
    for _ in 0..8 {
        room::dispatch_rom(&mut r, &[], "RoomCode_RidleyRoomShaking", 0, 0).unwrap();
    }
    assert_eq!(r.word(0x10a8), 0);
    for address in [0xb1, 0xb3, 0xb5, 0xb7] {
        assert_eq!(r.word(address), 0xffff);
    }
}

#[test]
fn diagonal_shake_expires_or_starts_without_explosion_when_frozen() {
    for (name, timer, kind, next_timer) in [
        ("RoomCode_ShakeScreenHorizDiag", 1, 18, 0),
        ("RoomCode_ShakeScreenHorizDiagStrong", 1, 21, 0),
        ("RoomCode_ShakeScreenHorizDiag", 0, 23, 42),
        ("RoomCode_ShakeScreenHorizDiagStrong", 0, 26, 42),
    ] {
        let mut b = vec![0; WRAM_SIZE];
        let mut r = Wram::new(&mut b).unwrap();
        r.set_word(0x7e1, timer);
        r.set_word(0x5e5, 1);
        r.set_word(0xa78, 1);
        room::dispatch_rom(&mut r, &[], name, 0, 0).unwrap();
        assert_eq!(r.word(0x183e), kind);
        assert_eq!(r.word(0x7e1), next_timer);
        assert_eq!(r.word(0x5e5), if timer == 0 { 0x116 } else { 1 });
        if name.ends_with("Strong") {
            assert_eq!(r.word(0x7e3), kind);
        }
    }
}

#[test]
fn collected_map_preserves_adjacent_byte_in_aliased_word() {
    let mut b = vec![0xa5; WRAM_SIZE];
    let mut r = Wram::new(&mut b).unwrap();
    r.set_word(0x79f, 7);
    r.set_word(0xd90f, 0xab20);
    room::dispatch_rom(&mut r, &[], "RoomCode_SetCollectedMap", 0, 0).unwrap();
    assert_eq!(r.word(0xd90f), 0xab21);
    assert_eq!(r.word(0x789), 0xab21);
    assert_eq!(r.byte(0xd90e), 0xa5);
}

#[test]
fn rescued_animals_gate_plm_but_always_start_earthquake() {
    let rom = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../../rom/sm.sfc")).unwrap();
    for rescued in [false, true] {
        let mut b = vec![0; WRAM_SIZE];
        let mut r = Wram::new(&mut b).unwrap();
        r.set_word(0x7a5, 64);
        r.set_byte(0xd821, if rescued { 0x80 } else { 0 });
        room::dispatch_rom(&mut r, &rom, "RoomSetup_AfterSavingAnimals", 0, 0).unwrap();
        assert_eq!(r.word(0x1c37 + 78), if rescued { 0xbb30 } else { 0 });
        assert_eq!(r.word(0x183e), 24);
        assert_eq!(r.word(0x1840), 0xffff);
        assert_eq!(r.word(0x1c87 + 78), (11 * 64 + 61) * 2);
    }
}

#[test]
fn native_door_scroll_names_preserve_actual_byte_indices_and_neighbors() {
    for (name, writes) in [
        ("DoorCode_Scroll13_Blue", &[(19usize, 1u8)][..]),
        ("DoorCode_Scroll_4Blue_8Green", &[(4, 0), (8, 2)][..]),
        ("UNUSED_DoorCode_SetScroll_51", &[(17, 0), (16, 1)][..]),
        ("DoorCode_SetScroll_48", &[(40, 2), (46, 2)][..]),
    ] {
        let mut b = vec![0xa5; WRAM_SIZE];
        let mut expected = b.clone();
        for &(index, value) in writes {
            expected[0xcd20 + index] = value;
        }
        room::dispatch_rom(&mut Wram::new(&mut b).unwrap(), &[], name, 0, 0).unwrap();
        assert_eq!(b, expected);
    }
}

#[test]
fn treadmill_door_allocates_both_objects_and_clears_exact_native_tile_range() {
    let rom = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../../rom/sm.sfc")).unwrap();
    for (name, anim, plm) in [
        ("DoorCode_StartWreckedShipTreadmillWest", 0x8275, 0xb64b),
        ("DoorCode_StartWreckedSkipTreadmill_East", 0x827b, 0xb64f),
    ] {
        let mut b = vec![0; WRAM_SIZE];
        let mut r = Wram::new(&mut b).unwrap();
        r.set_word(0x7a5, 16);
        let start = 0x10002 + (9 * 16 + 4) * 2;
        r.set_word(start - 2, 0xabcd);
        r.set_word(start + 112, 0xabcd);
        room::dispatch_rom(&mut r, &rom, name, 0, 0).unwrap();
        assert_eq!(r.word(0x1ef5 + 10), anim);
        assert_eq!(r.word(0x1c37 + 78), plm);
        for offset in (0..112).step_by(2) {
            assert_eq!(r.word(start + offset), 255);
        }
        assert_eq!(r.word(start - 2), 0xabcd);
        assert_eq!(r.word(start + 112), 0xabcd);
    }
}

#[test]
fn statue_setup_allocates_headers_in_native_order_and_full_table_needs_no_rom() {
    let rom = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../../rom/sm.sfc")).unwrap();
    let mut b = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut b).unwrap();
    room::dispatch_rom(&mut r, &rom, "RoomSetup_RunStatueUnlockAnims", 0, 0).unwrap();
    for (slot, header) in [(10, 0x8558), (8, 0x854c), (6, 0x855e), (4, 0x8552)] {
        assert_eq!(r.word(0x1ef5 + slot), header);
    }
    r.set_word(0x1ef5, 1);
    r.set_word(0x1ef7, 1);
    let before = b.clone();
    room::dispatch_rom(&mut Wram::new(&mut b).unwrap(), &[], "RoomSetup_RunStatueUnlockAnims", 0, 0).unwrap();
    assert_eq!(b, before);
}

#[test]
fn frozen_explosion_still_shakes_and_blank_tile_consumes_only_coordinate_rng() {
    let mut b = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut b).unwrap();
    r.set_word(0xa78, 1);
    r.set_word(0x1840, 3);
    r.set_word(0x5e5, 1);
    room::dispatch_rom(&mut r, &[], "RoomCode_ExplodeShakes", 0, 0).unwrap();
    assert_eq!(r.word(0x1840), 0x8003);
    assert_eq!(r.word(0x5e5), 1);
    r.set_word(0xa78, 0);
    r.set_word(0x7a5, 16);
    r.set_word(0x10004, 0x40ff);
    room::dispatch_rom(&mut r, &[], "RoomCode_GenRandomExplodes_Nonblank", 0, 0).unwrap();
    assert_eq!(r.word(0x5e5), 0x116);
    assert_eq!(r.word(0xef78 + 62), 0);
    r.set_word(0x5b6, 2);
    room::dispatch_rom(&mut r, &[], "RoomCode_GenRandomExplodes_4th", 0, 0).unwrap();
    assert_eq!(r.word(0x5e5), 0x116);
}

#[test]
fn parallax_manual_modes_and_signed_blocks() {
    let mut b = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut b).unwrap();
    r.set_word(0x911, 0x1234);
    r.set_byte(0x91b, 128);
    assert_eq!(room::dispatch_rom(&mut r, &[], "CalculateLayer2Xpos", 0, 0), Ok(0));
    assert_eq!(r.word(0x917), 0x091a);
    r.set_byte(0x91b, 1);
    r.set_word(0x911, 0xfff0);
    assert_eq!(room::dispatch_rom(&mut r, &[], "CalculateLayer2Xpos", 0, 0), Ok(1));
    assert_eq!(r.word(0x917), 0x091a);
    room::dispatch_rom(&mut r, &[], "CalculateBgScrollAndLayerPositionBlocks", 0, 0).unwrap();
    assert_eq!(r.word(0x8f7), 0xffff);
}
#[test]
fn closed_scroll_cell_pushes_camera_toward_next_screen() {
    let mut b = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut b).unwrap();
    r.set_word(0x7a9, 4);
    r.set_word(0x911, 127);
    r.set_word(0xb0a, 0);
    r.set_word(0xda2, 3);
    room::dispatch_rom(&mut r, &[], "HandleScrollingWhenTriggeringScrollLeft", 0, 0).unwrap();
    assert_eq!(r.word(0x911), 132);
    assert!(room::dispatch_rom(&mut r, &[], "HardwareTransition", 0, 0).is_err());
}
#[test]
fn door_negative_distance_uses_direction_specific_default() {
    let mut rom = vec![0; 0x20000];
    let o = (0x83 & 127) * 32768;
    rom[o..o + 12].copy_from_slice(&[0, 0x90, 0x80, 2, 0, 0, 3, 4, 255, 255, 0, 0]);
    let mut b = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut b).unwrap();
    r.set_word(0x78d, 0x8000);
    room::dispatch_rom(&mut r, &rom, "LoadDoorHeader", 0, 0).unwrap();
    assert_eq!(r.word(0x79b), 0x9000);
    assert_eq!(r.word(0x92d), 1);
    assert_eq!(r.word(0x92b), 0x8000);
    assert_eq!(r.word(0xe16), 128);
    assert_eq!(r.word(0x927), 0x300);
}
#[test]
fn room_state_morph_gate_selects_actual_rom_destination() {
    let mut rom = vec![0; 0x80000];
    let o = (0x8f & 127) * 32768;
    rom[o..o + 4].copy_from_slice(&[0x52, 0xe6, 0x20, 0x80]);
    let mut b = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut b).unwrap();
    assert_eq!(room::dispatch_rom(&mut r, &rom, "RoomDefStateSelect_MorphBallMissiles", 0x8002, 0), Ok(0x8004));
    r.set_word(0x9a4, 4);
    r.set_word(0x9c8, 5);
    assert_eq!(room::dispatch_rom(&mut r, &rom, "RoomDefStateSelect_MorphBallMissiles", 0x8002, 0), Ok(0));
    assert_eq!(r.word(0x7bb), 0x8020);
    assert!(room::dispatch_rom(&mut r, &rom, "RoomDefStateSelect_Finish", 0xfff0, 0).is_err());
}

#[test]
fn horizontal_ideal_reset_clears_its_own_fractional_word() {
    let mut b = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut b).unwrap();
    r.set_word(0x7a9, 4);
    r.set_word(0x911, 255);
    r.set_word(0xb0a, 0);
    r.set_word(0x90f, 0xa5a5);
    r.set_word(0x913, 0x1234);
    for i in 0..256 {
        r.set_byte(0xcd20 + i, 1);
    }
    room::dispatch_rom(&mut r, &[], "HandleScrollingWhenTriggeringScrollRight", 0, 0).unwrap();
    assert_eq!(r.word(0x911), 0);
    assert_eq!(r.word(0x90f), 0);
    assert_eq!(r.word(0x913), 0x1234);
}
