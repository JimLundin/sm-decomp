use port::{WRAM_SIZE, Wram, enemy_projectile_instructions, enemy_projectiles};

fn rom() -> Vec<u8> {
    let mut rom = vec![0; 0x3140e];
    for (p, words) in [
        (0x9642, [0x93ca_u16, 0x940e, 0x9552, 0x0606, 0x5003]),
        (0xa387, [0xa2ee, 0xa328, 0xa28d, 0x0101, 0x3000]),
        (0xa395, [0xa31b, 0xa364, 0xa299, 0x0101, 0x3000]),
    ] {
        let o = 0x30000 + (p & 0x7fff);
        if rom.len() < o + 10 {
            rom.resize(o + 10, 0);
        }
        for (i, v) in words.iter().enumerate() {
            rom[o + i * 2..o + i * 2 + 2].copy_from_slice(&v.to_le_bytes());
        }
    }
    for (i, v) in [0x5003_u16, 0x9003, 0xd003].into_iter().enumerate() {
        rom[0x31408 + i * 2..0x3140a + i * 2].copy_from_slice(&v.to_le_bytes());
    }
    rom
}

fn change(bytes: &mut [u8], p: usize, v: u16) {
    bytes[p..p + 2].copy_from_slice(&v.to_le_bytes());
}

#[test]
fn ridley_direct_initializer_uses_enemy_zero_and_preserves_every_other_byte() {
    for (area, property) in [(0, 0x5003), (2, 0x9003), (5, 0xd003), (0xffff, 0x5003)] {
        for parameter in [0, 1, 2, 0x8000, 0xffff] {
            let mut bytes = vec![0xa5; WRAM_SIZE];
            let k = 16;
            {
                let mut r = Wram::new(&mut bytes).unwrap();
                r.set_word(0x79f, area);
                r.set_word(0x1993, parameter);
                r.set_word(0x1995, 0xcafe);
                r.set_word(0xe54, 64);
                r.set_word(0xf7a, 10);
                r.set_word(0xf7e, 20);
                r.set_word(0xf7a + 64, 300);
                r.set_word(0xf7e + 64, 400);
                r.set_word(0x7832, 0x8000);
                r.set_word(0x7834, 0xffff);
                r.set_word(0x7832 + 64, 0x1234);
                r.set_word(0x7834 + 64, 0x5678);
            }
            let mut expected = bytes.clone();
            for (p, v) in [
                (0x1b23, 0xcafe),
                (0x1a4b, if parameter == 0 { 0xfff1 } else { 35 }),
                (0x1a93, 0xffe9),
                (0x19bb, 2560),
                (0x1ab7, 0x8000),
                (0x1adb, 0xffff),
                (0x1bd7, property),
            ] {
                change(&mut expected, p + k, v);
            }
            enemy_projectiles::dispatch_rom(
                &mut Wram::new(&mut bytes).unwrap(),
                &rom(),
                "EprojInitializer",
                k as i32,
                0x93ca,
            )
            .unwrap();
            assert_eq!(bytes, expected, "area={area}, parameter={parameter}");
        }
    }
}

#[test]
fn ceres_direct_initializers_and_common_setup_have_native_partial_resets() {
    for k in (0..36).step_by(2) {
        for name in ["pad", "platform", "common"] {
            let mut bytes = vec![0x5a; WRAM_SIZE];
            {
                let mut r = Wram::new(&mut bytes).unwrap();
                r.set_word(0xaf6, 0xfffe);
                r.set_word(0xafa, 0xfff0);
            }
            let mut expected = bytes.clone();
            for p in [0x1a27, 0x1a6f, 0x1ab7, 0x1adb, 0x19bb] {
                change(&mut expected, p + k, 0);
            }
            change(&mut expected, 0x1a4b + k, 0xfffe);
            if name == "pad" {
                change(&mut expected, 0x1a93 + k, 12);
                change(&mut expected, 0x1aff + k, 60);
            } else if name == "platform" {
                change(&mut expected, 0x1a93 + k, 97);
            }
            let (symbol, address) = match name {
                "pad" => ("EprojInitializer", 0xa2ee),
                "platform" => ("EprojInitializer", 0xa31b),
                _ => ("sub_86A301", 0),
            };
            enemy_projectiles::dispatch_rom(&mut Wram::new(&mut bytes).unwrap(), &[], symbol, k as i32, address)
                .unwrap();
            assert_eq!(bytes, expected, "{name} slot={k}");
        }
    }
}

#[test]
fn observed_allocation_selects_highest_free_slot_and_resets_native_fields_only() {
    let rom = rom();
    for definition in [0x9642, 0xa387, 0xa395] {
        for k in (0..36).step_by(2) {
            let mut bytes = vec![0xa5; WRAM_SIZE];
            {
                let mut r = Wram::new(&mut bytes).unwrap();
                r.set_word(0x1997 + k, 0);
                if k >= 2 {
                    r.set_word(0x1997 + k - 2, 0);
                }
                r.set_word(0x1995, 0x1234);
                r.set_word(0xf7a, 10);
                r.set_word(0xf7e, 20);
                r.set_word(0x7832, 0xffff);
                r.set_word(0x7834, 0x8000);
                r.set_word(0xaf6, 99);
                r.set_word(0xafa, 100);
            }
            let mut expected = bytes.clone();
            change(&mut expected, 0x1993, 0xffff);
            change(&mut expected, 0x1997 + k, definition);
            for p in [0x1aff, 0x1b23, 0x19df, 0x1a27, 0x1a6f, 0x1bfb] {
                change(&mut expected, p + k, 0);
            }
            change(&mut expected, 0x1b8f + k, 1);
            change(&mut expected, 0x1b6b + k, 0x8000);
            let o = 0x30000 + usize::from(definition & 0x7fff);
            for (i, p) in [0x1a03, 0x1b47, 0x1bb3, 0x1bd7].into_iter().enumerate() {
                change(&mut expected, p + k, u16::from_le_bytes([rom[o + 2 + i * 2], rom[o + 3 + i * 2]]));
            }
            let fields = if definition == 0x9642 {
                [
                    (0x19bb, 2560),
                    (0x1a4b, 35),
                    (0x1a93, 0xffe9),
                    (0x1ab7, 0xffff),
                    (0x1adb, 0x8000),
                    (0x1b23, 0x1234),
                    (0x1bd7, 0x5003),
                ]
            } else {
                [
                    (0x19bb, 0),
                    (0x1a4b, 99),
                    (0x1a93, if definition == 0xa387 { 128 } else { 97 }),
                    (0x1ab7, 0),
                    (0x1adb, 0),
                    (0x1aff, if definition == 0xa387 { 60 } else { 0 }),
                    (0x1bd7, 0x3000),
                ]
            };
            for (p, v) in fields {
                change(&mut expected, p + k, v);
            }
            enemy_projectiles::spawn_room(&mut Wram::new(&mut bytes).unwrap(), &rom, definition, 0xffff).unwrap();
            assert_eq!(bytes, expected, "definition={definition:x} slot={k}");
        }
    }
}

#[test]
fn direct_initializers_refuse_invalid_slots_and_missing_ridley_table_before_mutation() {
    for (slot, address, rom) in [(1, 0x93ca, rom()), (36, 0xa2ee, rom()), (0xffff, 0xa31b, rom()), (0, 0x93ca, vec![])]
    {
        let mut bytes = vec![0xa5; WRAM_SIZE];
        let before = bytes.clone();
        assert!(
            enemy_projectiles::dispatch_rom(
                &mut Wram::new(&mut bytes).unwrap(),
                &rom,
                "EprojInitializer",
                slot,
                address
            )
            .is_err()
        );
        assert_eq!(bytes, before);
    }
}

#[test]
fn full_pool_with_invalid_definition_and_no_host_changes_only_parameter() {
    let mut bytes = vec![0xa5; WRAM_SIZE];
    let mut expected = bytes.clone();
    change(&mut expected, 0x1993, 0x1234);
    enemy_projectiles::spawn_room(&mut Wram::new(&mut bytes).unwrap(), &[], 0xffff, 0x1234).unwrap();
    assert_eq!(bytes, expected);
}

#[test]
fn stationary_fireball_noop_preserves_wram_and_scheduler_still_advances() {
    let mut bytes = vec![0xa5; WRAM_SIZE];
    let before = bytes.clone();
    enemy_projectile_instructions::preinstruction(&mut Wram::new(&mut bytes).unwrap(), &[], 0x950c, 34).unwrap();
    assert_eq!(bytes, before);
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x198d, 0x8000);
    r.set_word(0x1997 + 34, 0x9650);
    r.set_word(0x1a03 + 34, 0x950c);
    r.set_word(0x1b8f + 34, 2);
    r.set_word(0x1b47 + 34, 0x95a0);
    enemy_projectile_instructions::run_all(&mut r, &[], 16).unwrap();
    assert_eq!(r.word(0x1b8f + 34), 1);
    assert_eq!(r.word(0x1b47 + 34), 0x95a0);
    assert_eq!(r.word(0x1997 + 34), 0x9650);
}
