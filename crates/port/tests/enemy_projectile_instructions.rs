use port::{WRAM_SIZE, Wram, enemy_projectile_instructions as e};
#[test]
fn native_graphics_and_property_leaves_preserve_other_fields() {
    let mut bytes = vec![0xa5; WRAM_SIZE];
    let mut expected = bytes.clone();
    expected[0x19bb + 34..0x19bd + 34].copy_from_slice(&0u16.to_le_bytes());
    expected[0x1bd7 + 34..0x1bd9 + 34].copy_from_slice(&0x3000u16.to_le_bytes());
    let mut r = Wram::new(&mut bytes).unwrap();
    assert_eq!(e::instruction(&mut r, &[], 0xd1c7, 0xffff, 34).unwrap(), 0xffff);
    assert_eq!(e::instruction(&mut r, &[], 0xdc5a, 0xffff, 34).unwrap(), 0xffff);
    assert_eq!(bytes, expected);
}

#[test]
fn fixed_projectile_sound_keeps_pointer_and_queues_channel_two() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    assert_eq!(e::instruction(&mut r, &[], 0xee97, 0xffff, 34).unwrap(), 0xffff);
    assert_eq!(r.byte(0x666), 0x24);
    assert_eq!(r.byte(0x647), 1);
}

#[test]
fn randomized_sprite_uses_one_rng_draw_and_keeps_projectile_position() {
    let rom = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../../rom/sm.sfc")).unwrap();
    for (opcode, operand, radius) in [(0xece3, 0xeccb, 32u16), (0xed17, 0xecb1, 16)] {
        let mut bytes = vec![0; WRAM_SIZE];
        let mut r = Wram::new(&mut bytes).unwrap();
        r.set_word(0x5e5, 1);
        r.set_word(0x1a4b + 34, 0);
        r.set_word(0x1a93 + 34, 0xffff);
        assert_eq!(e::instruction(&mut r, &rom, opcode, operand, 34).unwrap(), operand + 2);
        let random = r.word(0x5e5);
        assert_eq!(random, 0x116);
        let mask = radius * 2 - 1;
        assert_eq!(r.word(0xf0f8 + 62), (random & mask).wrapping_sub(radius));
        assert_eq!(r.word(0xf1f8 + 62), 0xffffu16.wrapping_add((random >> 8) & mask).wrapping_sub(radius));
        assert_eq!(r.word(0x1a4b + 34), 0);
        assert_eq!(r.word(0x1a93 + 34), 0xffff);
        assert_ne!(r.word(0xef78 + 62), 0);
    }
}

#[test]
fn ceres_pad_waits_then_moves_and_releases_samus_with_wrapping_height() {
    for (timer, height, remaining, samus, pad, released) in [
        (2u16, 72u16, 1u16, 72u16, 0xfaceu16, false),
        (0, 71, 0, 72, 99, false),
        (1, 72, 0, 72, 100, true),
        (0xffff, 72, 0xfffe, 72, 100, true),
        (0x8000, 72, 0x7fff, 72, 0xface, false),
        (0, 0xffff, 0, 0, 27, false),
        (0, 0x8048, 0, 0x8049, 0x8064, false),
    ] {
        let mut bytes = vec![0xa5; WRAM_SIZE];
        {
            let mut r = Wram::new(&mut bytes).unwrap();
            r.set_word(0x1aff + 34, timer);
            r.set_word(0xafa, height);
            r.set_word(0x1a93 + 34, 0xface);
        }
        let mut expected = bytes.clone();
        {
            let mut r = Wram::new(&mut expected).unwrap();
            r.set_word(0x1aff + 34, remaining);
            r.set_word(0xafa, samus);
            r.set_word(0x1a93 + 34, pad);
            if released {
                r.set_word(0x1b8f + 34, 1);
                r.set_word(0x1b47 + 34, 0xa28b);
                r.set_word(0xa42, 0xe695);
                r.set_word(0xa44, 0xe8aa);
                for address in [0xa28, 0xa2a, 0xa2c] {
                    r.set_word(address, 0xffff);
                }
                for address in [0xa2e, 0xa30, 0xa32] {
                    r.set_word(address, 0);
                }
            }
        }
        e::dispatch_rom(&mut Wram::new(&mut bytes).unwrap(), &[], "EprojPreInstr_CeresElevatorPad", 34, 0).unwrap();
        assert_eq!(bytes, expected);
        assert!(
            e::dispatch_rom(&mut Wram::new(&mut bytes).unwrap(), &[], "EprojPreInstr_CeresElevatorPad", 35, 0).is_err()
        );
        assert_eq!(bytes, expected);
    }
}
#[test]
fn fractional_velocity_preserves_low_subpixel_byte() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x1a27, 0xff37);
    r.set_word(0x1a4b, 10);
    r.set_word(0x1ab7, 0xff80);
    assert_eq!(e::move_x(&mut r, 0).unwrap(), 10);
    assert_eq!(r.word(0x1a27), 0x7f37);
}
#[test]
fn relative_branch_underflows_timer_and_sleep_keeps_opcode_address() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    let mut rom = vec![0; 0x38000];
    rom[0x31000] = 0xfe;
    assert_eq!(e::instruction(&mut r, &rom, 0x81ce, 0x9000, 0).unwrap(), 0x8ffe);
    assert_eq!(r.word(0x19df), 0xffff);
    assert_eq!(e::instruction(&mut r, &rom, 0x8159, 0x9002, 0).unwrap(), 0);
    assert_eq!(r.word(0x1b47), 0x9000);
}
#[test]
fn palette_count_is_inclusive_and_property_clear_uses_mask() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    let mut rom = vec![0; 0x38000];
    rom[0x31000..0x31005].copy_from_slice(&[0, 0x91, 2, 0, 1]);
    rom[0x31100..0x31104].copy_from_slice(&[0x34, 0x12, 0x78, 0x56]);
    assert_eq!(e::instruction(&mut r, &rom, 0x82d5, 0x9000, 0).unwrap(), 0x9005);
    assert_eq!(r.word(0xc002), 0x1234);
    assert_eq!(r.word(0xc004), 0x5678);
    r.set_word(0x1bd7, 0xffff);
    rom[0x31000..0x31002].copy_from_slice(&[0, 0x20]);
    e::instruction(&mut r, &rom, 0x823c, 0x9000, 0).unwrap();
    assert_eq!(r.word(0x1bd7), 0x2000);
}
#[test]
fn frame_runs_deleting_preinstruction_before_script_and_descends_slots() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    let mut rom = vec![0; 0x38000];
    r.set_word(0x198d, 0x8000);
    r.set_word(0x1997 + 34, 0x9000);
    r.set_word(0x1a03 + 34, 0xe4fe);
    r.set_word(0x1a4b + 34, 256);
    r.set_word(0x1b8f + 34, 1);
    r.set_word(0x1b47 + 34, 0x9000);
    rom[0x31000..0x31004].copy_from_slice(&[7, 0, 0x34, 0x92]);
    e::run_all(&mut r, &rom, 64).unwrap();
    assert_eq!(r.word(0x1997 + 34), 0);
    assert_eq!(r.word(0x1b8f + 34), 7);
    assert_eq!(r.word(0x1b6b + 34), 0x9234);
    assert_eq!(r.word(0x1991), 0);
}
#[test]
fn shard_spawns_sparkle_into_lower_slot_that_runs_in_same_frame() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    let mut rom = vec![0; 0x38000];
    // Pinned initializer and preinstruction, with a short independent sparkle script.
    rom[0x34f0a..0x34f14].copy_from_slice(&[0x6d, 0xce, 0x70, 0x81, 0, 0x91, 0, 0, 0, 0]);
    rom[0x31100..0x31104].copy_from_slice(&[9, 0, 0x34, 0x92]);
    r.set_word(0x198d, 0x8000);
    r.set_word(0x1997 + 34, 0x9000);
    r.set_word(0x1a03 + 34, 0xce9b);
    r.set_word(0x1a4b + 34, 100);
    r.set_word(0x1a93 + 34, 100);
    r.set_word(0x1b8f + 34, 2);
    // NextRandom(0)=273, whose 0x420 bits are both clear.
    e::run_all(&mut r, &rom, 64).unwrap();
    assert_eq!(r.word(0x1997 + 32), 0xcf0a);
    assert_eq!(r.word(0x19bb + 32), 1600);
    assert_eq!(r.word(0x1b8f + 32), 9);
    assert_eq!(r.word(0x1b6b + 32), 0x9234);
    assert_eq!(r.word(0x1adb + 34), 32);
    assert_eq!(r.word(0x1b8f + 34), 1);
}
#[test]
fn script_cycle_is_rejected_and_disabled_scheduler_preserves_index() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    let mut rom = vec![0; 0x38000];
    r.set_word(0x1991, 12);
    e::run_all(&mut r, &rom, 0).unwrap();
    assert_eq!(r.word(0x1991), 12);
    r.set_word(0x198d, 0x8000);
    r.set_word(0x1997, 0x9000);
    r.set_word(0x1a03, 0x8170);
    r.set_word(0x1b8f, 1);
    r.set_word(0x1b47, 0x9000);
    rom[0x31000..0x31004].copy_from_slice(&[0xab, 0x81, 0, 0x90]);
    assert_eq!(e::run_all(&mut r, &rom, 16), Err("enemy projectile instruction budget"));
}

#[test]
fn ceres_platform_changes_only_its_timer_and_script_at_exact_samus_height() {
    use port::enemy_projectile_instructions::dispatch_rom;
    for y in [0u16, 71, 72, 73, 0x8000, 0xffff] {
        let mut bytes = vec![0xa5; port::WRAM_SIZE];
        bytes[0xafa..0xafc].copy_from_slice(&y.to_le_bytes());
        let mut expected = bytes.clone();
        if y == 72 {
            expected[0x1b8f + 34..0x1b91 + 34].copy_from_slice(&1u16.to_le_bytes());
            expected[0x1b47 + 34..0x1b49 + 34].copy_from_slice(&0xa28bu16.to_le_bytes());
        }
        assert_eq!(
            dispatch_rom(&mut port::Wram::new(&mut bytes).unwrap(), &[], "EprojPreInstr_CeresElevatorPlatform", 34, 0)
                .unwrap(),
            0
        );
        assert_eq!(bytes, expected);
        let before = bytes.clone();
        assert!(
            dispatch_rom(&mut port::Wram::new(&mut bytes).unwrap(), &[], "EprojPreInstr_CeresElevatorPlatform", 35, 0)
                .is_err()
        );
        assert_eq!(bytes, before);
    }
}

#[test]
fn native_script_leaves_preserve_wram_and_return_bank86_pointers() {
    use port::enemy_projectile_instructions::dispatch_rom;
    let mut rom = vec![0; 0x400000];
    rom[0x30001..0x30003].copy_from_slice(&0xa28bu16.to_le_bytes());
    for slot in [0, 16, 34] {
        for (name, address, value, returned) in [
            ("EprojInstr_Delete", 0x1997, 0u16, 0),
            ("EprojInstr_SetPreInstr_", 0x1a03, 0xa28b, 0x868003),
            ("EprojInstr_ClearPreInstr", 0x1a03, 0x8170, 0x868001),
            ("EprojInstr_Goto", 0, 0, 0x86a28b),
        ] {
            let mut bytes = vec![0xa5; port::WRAM_SIZE];
            let mut expected = bytes.clone();
            if address != 0 {
                expected[address + slot..address + slot + 2].copy_from_slice(&value.to_le_bytes());
            }
            assert_eq!(
                dispatch_rom(&mut port::Wram::new(&mut bytes).unwrap(), &rom, name, slot as i32, 0x868001).unwrap(),
                returned
            );
            assert_eq!(bytes, expected);
            let before = bytes.clone();
            assert!(dispatch_rom(&mut port::Wram::new(&mut bytes).unwrap(), &rom, name, 35, 0x868001).is_err());
            assert_eq!(bytes, before);
        }
    }
}
