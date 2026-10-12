use port::{Wram, ridley_enemy_instructions as instructions};

fn native_rom() -> Vec<u8> {
    let path = std::env::var_os("SM_RIDLEY_ROM")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rom/sm.sfc"));
    std::fs::read(path).expect("pinned native ROM is required for Ridley script controls")
}
fn initial(slot: u16) -> Vec<u8> {
    let mut bytes = vec![0; port::WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0xe54, slot);
    let enemy = 0xf78 + usize::from(slot);
    ram.set_word(enemy, 0xe13f);
    ram.set_byte(enemy + 0x2e, 0xa6);
    ram.set_word(enemy + 0x1a, 0xe73a);
    ram.set_word(enemy + 0x1c, 1);
    ram.set_word(0x9c2, 99);
    ram.set_word(0xf7a, 180);
    ram.set_word(0xf7e, 111);
    ram.set_word(0xaf6, 80);
    ram.set_word(0xafa, 180);
    bytes
}
fn host() -> instructions::ProjectileHost {
    instructions::ProjectileHost::from_words([0x5aa5, 0xa55a, 11, 22, 33, 44, 55, 66])
}
fn invoke(bytes: &mut [u8], rom: &[u8], name: &str, slot: i32, pointer: i32) -> u32 {
    instructions::dispatch_host(&mut Wram::new(bytes).unwrap(), rom, &mut host(), name, slot, pointer).unwrap()
}

#[test]
fn native_entry_selects_both_orientations_and_health_exit_before_attack_mutation() {
    let rom = native_rom();
    for orientation in [0, 1, 2, 0xffff] {
        for health in [29, 30, 31, 0x8000, 0x801d, 0x801e, 0xffff] {
            for area in [0, 2] {
                let mut bytes = initial(64);
                let mut ram = Wram::new(&mut bytes).unwrap();
                ram.set_word(0x7820, orientation);
                ram.set_word(0x9c2, health);
                ram.set_word(0x79f, area);
                ram.set_word(0x781e, 0x1234);
                let report = instructions::process_instructions(&mut ram, &rom, &mut host(), 20).unwrap();
                let exiting = area != 2 && (health.wrapping_sub(30) as i16) < 0;
                let expected = match (orientation != 0, exiting) {
                    (false, false) => (0xe746, 0xe983, 8),
                    (true, false) => (0xe7bc, 0xe9a5, 8),
                    (false, true) => (0xe7b2, 0xe983, 1),
                    (true, true) => (0xe826, 0xe9a5, 1),
                };
                assert_eq!((ram.word(0xfd2), ram.word(0xfce), ram.word(0xfd4)), expected);
                assert_eq!(ram.word(0x7800), if exiting { 8 } else { 0 });
                assert_eq!(ram.word(0x781e), if exiting { 0 } else { 0x1234 });
                assert_eq!(ram.word(0xf92), 0);
                assert!(report.effects.is_empty());
                assert_eq!(report.opcodes[0].address, 0xa6e517);
            }
        }
    }
}

#[test]
fn callbacks_always_aim_from_enemy_zero_and_keep_pointer_and_host_identity() {
    let rom = native_rom();
    for orientation in [0, 1, 2, 0xffff] {
        let mut a = initial(0);
        let mut b = initial(64);
        for bytes in [&mut a, &mut b] {
            let mut ram = Wram::new(bytes).unwrap();
            ram.set_word(0x7820, orientation);
            ram.set_word(0xfba, 5000);
            ram.set_word(0xfbe, 6000);
        }
        assert_eq!(invoke(&mut a, &rom, "Ridley_Instr_11", 0, 0xa6e755), 0xa6e755);
        assert_eq!(invoke(&mut b, &rom, "Ridley_Instr_11", 64, 0xa6e755), 0xa6e755);
        assert_eq!(&a[0x7832..0x7836], &b[0x7832..0x7836]);
        assert_eq!(&b[0x7872..0x7876], &[0; 4]);
    }
}

#[test]
fn exact_angle_clamps_match_independent_native_sine_table_at_every_raw_angle() {
    let rom = native_rom();
    // Choose one pair for each raw angle using the native hardware division
    // domain; the expectations below never call the maintained angle or trig.
    let mut coordinates = [None; 256];
    for dx in -255_i16..=255 {
        for dy in -255_i16..=255 {
            let quadrant = usize::from(dx < 0) * 2 + usize::from(dy < 0);
            let x = dx.unsigned_abs();
            let y = dy.unsigned_abs();
            let horizontal = y < x;
            let (numerator, denominator) = if horizontal { (y << 8, x as u8) } else { (x << 8, y as u8) };
            let q = if denominator == 0 { 0xffff } else { numerator / u16::from(denominator) };
            let step = (q >> 3) as u8;
            let angle = if horizontal {
                match quadrant {
                    0 => 64u8.wrapping_add(step),
                    1 => 64u8.wrapping_sub(step),
                    2 => 192u8.wrapping_sub(step),
                    _ => step.wrapping_sub(64),
                }
            } else {
                match quadrant {
                    0 => 128u8.wrapping_sub(step),
                    1 => step,
                    2 => 128u8.wrapping_add(step),
                    _ => 0u8.wrapping_sub(step),
                }
            };
            let raw = angle.wrapping_add(128).wrapping_neg();
            coordinates[usize::from(raw)].get_or_insert((dx as u16, dy as u16));
        }
    }
    // Signed 16-bit sine/cosine table shipped in the original ROM, bank A0.
    let sine_offset = 0x100000 + (0xb3c3 - 0x8000);
    for orientation in [0_u16, 1, 2, 0xffff] {
        for raw in 0..=255_u8 {
            let (dx, dy) = coordinates[usize::from(raw)].expect("every raw angle must have a fixture");
            let clamped = if orientation != 0 {
                if !(0x15..0xc0).contains(&raw) { 0x15 } else { raw.min(0x50) }
            } else if !(0x40..0xeb).contains(&raw) {
                0xeb
            } else {
                raw.max(0xb0)
            };
            let expected = |angle: u8| {
                let offset = sine_offset + (usize::from(angle) + 64) * 2;
                let sine = i16::from_le_bytes([rom[offset], rom[offset + 1]]);
                let magnitude = ((u32::from(sine.unsigned_abs()) * 0x500) >> 8) as u16;
                if sine < 0 { magnitude.wrapping_neg() } else { magnitude }
            };
            let mut bytes = initial(64);
            let mut ram = Wram::new(&mut bytes).unwrap();
            ram.set_word(0x7820, orientation);
            let mouth_x = ram.word(0xf7a).wrapping_add(if orientation != 0 { 25 } else { (-25_i16) as u16 });
            let mouth_y = ram.word(0xf7e).wrapping_sub(43);
            ram.set_word(0xaf6, mouth_x.wrapping_add(dx));
            ram.set_word(0xafa, mouth_y.wrapping_add(dy));
            instructions::dispatch_host(&mut ram, &rom, &mut host(), "Ridley_Instr_11", 64, 0xa6e754).unwrap();
            assert_eq!(
                (ram.word(0x7832), ram.word(0x7834)),
                (expected(clamped), expected(clamped.wrapping_add(64))),
                "orientation={orientation}, raw={raw}"
            );
        }
    }
}

#[test]
fn spawn_writes_f_source_before_highest_free_allocation_and_saturation() {
    let rom = native_rom();
    for name in ["Ridley_Instr_12", "Ridley_Instr_13"] {
        for free in (0..18).map(|i| Some(i * 2)).chain(std::iter::once(None)) {
            for orientation in [0, 1, 2, 0xffff] {
                let mut bytes = initial(64);
                let mut ram = Wram::new(&mut bytes).unwrap();
                ram.set_word(0x7820, orientation);
                ram.set_word(0x7832, 0x2345);
                ram.set_word(0x7834, 0xcdef);
                ram.set_word(0x1995, 0xa5a5);
                for slot in (0..36).step_by(2) {
                    ram.set_word(0x1997 + slot, 0x7777);
                }
                if let Some(slot) = free {
                    ram.set_word(0x1997 + slot, 0);
                }
                let mut context = host();
                let before_host = context;
                assert_eq!(
                    instructions::dispatch_host(&mut ram, &rom, &mut context, name, 64, 0xa6e755).unwrap(),
                    0xa6e755
                );
                assert_eq!(ram.word(0x1995), if name == "Ridley_Instr_12" { 0 } else { 14 });
                assert_eq!(ram.word(0x1993), orientation);
                assert_eq!(context, before_host);
                if let Some(slot) = free {
                    assert_eq!(ram.word(0x1997 + slot), 0x9642);
                    assert_eq!(ram.word(0x1b23 + slot), ram.word(0x1995));
                    assert_eq!(ram.word(0x1ab7 + slot), 0x2345);
                    assert_eq!(ram.word(0x1adb + slot), 0xcdef);
                    assert_eq!(
                        ram.word(0x1a4b + slot),
                        ram.word(0xf7a).wrapping_add(if orientation == 0 { (-25_i16) as u16 } else { 25 })
                    );
                } else {
                    for slot in (0..36).step_by(2) {
                        assert_eq!(ram.word(0x1997 + slot), 0x7777);
                    }
                }
            }
        }
    }
}

#[test]
fn refusal_preserves_timer_sound_allocation_and_host_before_any_mutation() {
    let rom = native_rom();
    for fault in 0..6 {
        let mut bytes = initial(0);
        let mut altered = rom.clone();
        let mut ram = Wram::new(&mut bytes).unwrap();
        ram.set_word(0xf92, 0xe746); // This frame would queue SFX before rendering.
        match fault {
            0 => ram.set_word(0xf92, 0xe74a), // Frame operand, not a boundary.
            1 => ram.set_byte(0xfa6, 0xa5),
            2 => ram.set_word(0xf78, 0xdead),
            3 => ram.set_byte(0x647, 16),
            4 => altered[0x136752] ^= 1,
            _ => altered.truncate(0x1000),
        }
        let before = bytes.clone();
        let mut ram = Wram::new(&mut bytes).unwrap();
        let mut context = host();
        let before_host = context;
        assert!(!instructions::supports_call(&ram, &altered, "ProcessEnemyInstructions", 0, 0).unwrap());
        assert!(instructions::process_instructions(&mut ram, &altered, &mut context, 20).is_err());
        assert_eq!(bytes, before);
        assert_eq!(context, before_host);
    }
}

#[test]
fn scheduled_native_attack_chain_reaches_eight_allocations_then_sleep() {
    let rom = native_rom();
    for orientation in [0, 1] {
        let mut bytes = initial(0);
        let mut ram = Wram::new(&mut bytes).unwrap();
        ram.set_word(0x7820, orientation);
        let mut context = host();
        let before_host = context;
        let mut allocations = Vec::new();
        let mut sounds = 0;
        let mut sleep = false;
        for _ in 0..240 {
            let report = instructions::process_instructions(&mut ram, &rom, &mut context, 20).unwrap();
            for effect in report.effects {
                match effect {
                    instructions::InstructionEffect::RoomProjectile { definition, parameter, source_f, slot } => {
                        assert_eq!((definition, parameter), (0x9642, orientation));
                        allocations.push((source_f, slot));
                    }
                    instructions::InstructionEffect::Sound { channel, sound, maximum } => {
                        assert_eq!((channel, sound, maximum), (1, 0x59, 6));
                        sounds += 1;
                    }
                }
            }
            if report.opcodes.iter().any(|o| o.address == 0xa6812f) {
                sleep = true;
                break;
            }
        }
        assert!(sleep);
        assert_eq!(allocations.iter().map(|(f, _)| *f).collect::<Vec<_>>(), [0, 14, 14, 14, 0, 14, 14, 14]);
        assert_eq!(
            allocations.iter().map(|(_, slot)| *slot).collect::<Vec<_>>(),
            [Some(34), Some(32), Some(30), Some(28), Some(26), Some(24), Some(22), Some(20)]
        );
        assert_eq!(sounds, 2);
        assert_eq!(ram.word(0xf92), if orientation == 0 { 0xe7b2 } else { 0xe826 });
        assert_eq!(ram.word(0xf94), 0);
        assert_eq!(ram.word(0x781e), 0);
        assert_eq!(context, before_host);
        instructions::process_instructions(&mut ram, &rom, &mut context, 20).unwrap();
        assert_eq!(ram.word(0xf94), 0xffff);
    }
}

#[test]
fn wrapping_timers_frozen_frames_and_bad_contracts_are_bounded() {
    let rom = native_rom();
    for timer in [0, 1, 0xffff] {
        for frozen in [false, true] {
            let mut bytes = initial(0);
            let mut ram = Wram::new(&mut bytes).unwrap();
            ram.set_word(0xf94, timer);
            ram.set_word(0xf8a, if frozen { 4 } else { 0 });
            let report = instructions::process_instructions(&mut ram, &rom, &mut host(), 20).unwrap();
            if frozen {
                assert!(report.frozen);
                assert_eq!(ram.word(0xf94), timer);
            } else if timer != 1 {
                assert!(report.timer_only);
                assert_eq!(ram.word(0xf94), timer.wrapping_sub(1));
            } else {
                assert_eq!(ram.word(0xf94), 8);
            }
        }
    }
    let mut bytes = initial(0);
    let before = bytes.clone();
    let mut ram = Wram::new(&mut bytes).unwrap();
    assert!(instructions::process_instructions(&mut ram, &rom, &mut host(), 0).is_err());
    assert_eq!(bytes, before);
    for (slot, ptr) in [(1, 0xa6e754), (-1, 0xa6e754), (0x800, 0xa6e754), (0, 0xa5e754), (0, 0xa6ffff)] {
        let mut ram = Wram::new(&mut bytes).unwrap();
        assert!(instructions::dispatch_host(&mut ram, &rom, &mut host(), "Ridley_Instr_10", slot, ptr).is_err());
        assert_eq!(bytes, before);
    }
}
