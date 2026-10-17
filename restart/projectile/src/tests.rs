use super::*;

struct NoPlm;
impl Host for NoPlm {
    fn spawn_plm(&mut self, _: &mut [u8], _: u16) -> Result<u8> {
        Err("unexpected foreign request".into())
    }
}

fn setup(r: &mut Wram<'_>) {
    for (at, value) in [
        (0x7a5, 16),
        (0x7a7, 16),
        (0x7a9, 1),
        (0x7ab, 1),
        (0x7b9, 512),
        (core::X, 100),
        (core::Y, 100),
        (core::RADIUS_X, 4),
        (core::RADIUS_Y, 4),
        (0xcce, 1),
    ] {
        r.set_word(at, value);
    }
}

fn rom_offset(address: u32) -> usize {
    (((address >> 16) & 127) * 0x8000 + (address & 0x7fff)) as usize
}
fn rom_word(rom: &mut [u8], address: u32, value: u16) {
    let at = rom_offset(address);
    rom[at..at + 2].copy_from_slice(&value.to_le_bytes());
}

#[test]
fn unsupported_inventory_refuses_without_mutation() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[0x9a6] = 1;
    let before = ram.clone();
    assert_eq!(run(Entry::FireUnchargedBeam, &mut ram, &[], &mut NoPlm), Ok(Outcome::Refused));
    assert_eq!(ram, before);
}

const ENTRIES: [Entry; 4] = [
    Entry::FireUnchargedBeam,
    Entry::HandleProjectile,
    Entry::DrawPlayerExplosions2,
    Entry::DrawBombAndProjectileExplosions,
];

#[test]
fn escape_admission_is_update_only() {
    for entry in ENTRIES {
        let mut ram = vec![0; WRAM_SIZE];
        let mut r = Wram::new(&mut ram).unwrap();
        // Exercise an admitted impact slot at the end of the ordinary inventory.
        for (at, value) in [
            (0x93f, 0x8001),
            (core::INSTRUCTION + 8, 0x9000),
            (core::TYPE + 8, 0x8700),
            (core::PRE + 8, 0xb169),
            (core::TIMER + 8, 2),
        ] {
            r.set_word(at, value);
        }
        let mut expected = ram.clone();
        let outcome = if entry == Entry::HandleProjectile {
            let mut r = Wram::new(&mut expected).unwrap();
            r.set_word(core::TIMER + 8, 1);
            r.set_word(core::INDEX, 0xfffe);
            Outcome::Handled
        } else {
            Outcome::Refused
        };
        assert_eq!(run(entry, &mut ram, &[], &mut NoPlm), Ok(outcome), "{entry:?}");
        assert_eq!(ram, expected, "{entry:?}");
    }
}

#[test]
fn escape_keeps_all_inventory_exclusions_without_mutation() {
    let exclusions: &[&[(usize, u16)]] = &[
        &[(0x9a6, 1)],
        &[(0xa76, 1)],
        &[(0xac4, 1)],
        &[(0xac6, 1)],
        &[(core::INSTRUCTION + 10, 0x9000)],
        &[(core::INSTRUCTION, 0x9000), (core::TYPE, 0x8010), (core::PRE, 0xaef3)],
        &[(core::INSTRUCTION, 0x9000), (core::TYPE, 0x8000), (core::PRE, 0xb169)],
        &[(core::INSTRUCTION, 0x9000), (core::TYPE, 0x8700), (core::PRE, 0xaef3)],
    ];
    for status in [0, 0x8000] {
        for fields in exclusions {
            for entry in ENTRIES {
                let mut ram = vec![0; WRAM_SIZE];
                let mut r = Wram::new(&mut ram).unwrap();
                r.set_word(0x93f, status);
                for &(at, value) in *fields {
                    r.set_word(at, value);
                }
                let before = ram.clone();
                assert_eq!(run(entry, &mut ram, &[], &mut NoPlm), Ok(Outcome::Refused));
                assert_eq!(ram, before, "{entry:?}, status {status:04x}, {fields:?}");
            }
        }
    }
}

#[test]
fn mode7_signed_truncated_products_wrap_and_use_m7a_for_y() {
    // X, Y, A, B, C, D, origin X/Y, camera X/Y, expected screen X/Y.
    // The half-products case distinguishes separate arithmetic shifts from
    // shifting a combined sum; the final two cases exercise 16-bit wrapping.
    let cases = [
        ([100, 80, 0x100, 0, 0, 0x7777, 64, 64, 20, 30], (80, 50)),
        ([1, 1, 0x80, 0x80, 0xff80, 0x100, 0, 0, 0, 0], (0xffff, 2)),
        ([0x8000, 0x7fff, 0x8000, 0x7fff, 0x8000, 0x100, 0, 0, 0xffff, 0x8000], (0x100, 0x8080)),
        ([1, 0xfffe, 0x7fff, 0xff81, 0x101, 0x1234, 0xfffe, 1, 0xfff0, 0xfff8], (0x18b, 0xfe87)),
    ];
    for (values, expected) in cases {
        let mut ram = vec![0; WRAM_SIZE];
        let mut r = Wram::new(&mut ram).unwrap();
        for (at, value) in
            [core::X + 4, core::Y + 4, 0x78, 0x7a, 0x7c, 0x7e, 0x80, 0x82, 0x911, 0x915].into_iter().zip(values)
        {
            r.set_word(at, value);
        }
        assert_eq!(trails::mode7(&r, 4), expected, "{values:x?}");
    }
}

#[test]
fn escape_update_allocates_both_trails_before_flight_and_instruction_progression() {
    let mut rom = vec![0; 0x300000];
    for (address, value) in [
        (0x90b5bb, 0xb700),
        (0x90b609, 0xb800),
        (0x9ba4b3, 0xb000),
        (0x9bb004, 0xb100), // Direction 2.
        (0x939000, 3),
        (0x939002, 0x9100),
        (0x939004, 0x0404),
        (0x939006, 1), // The current instruction selects the second offset record.
    ] {
        rom_word(&mut rom, address, value);
    }
    let offsets = rom_offset(0x9bb104);
    rom[offsets..offsets + 4].copy_from_slice(&[0x80, 127, 3, 0xf9]);
    for status in [1, 0x8000] {
        let mut ram = vec![0; WRAM_SIZE];
        let mut r = Wram::new(&mut ram).unwrap();
        setup(&mut r);
        for (at, value) in [
            (0x93f, status),
            (0x78, 0x80),
            (0x7a, 0x80),
            (0x7c, 0xff80),
            (0x7e, 0x1234),
            (0x911, 20),
            (0x915, 30),
            (core::INSTRUCTION, 0x9000),
            (core::TIMER, 1),
            (core::TYPE, 0x8000),
            (core::PRE, 0xaef3),
            (core::DIRECTION, 2),
            (core::SPEED_X, 0x100),
            (0xc90, 1),
            (0xd658 + 34, 9), // Highest slot occupied: allocation must choose 32.
        ] {
            r.set_word(at, value);
        }
        assert_eq!(run(Entry::HandleProjectile, &mut ram, &rom, &mut NoPlm), Ok(Outcome::Handled));
        let r = Wram::new(&mut ram).unwrap();
        let expected_x: u16 = if status & 0x8000 != 0 { 0 } else { 100 };
        assert_eq!(
            (r.word(0xd730 + 32), r.word(0xd754 + 32)),
            (expected_x.wrapping_sub(132), expected_x.wrapping_sub(1))
        );
        assert_eq!((r.word(0xd778 + 32), r.word(0xd79c + 32)), (223, 89));
        assert_eq!((r.word(0xd658 + 32), r.word(0xd67c + 32)), (1, 1));
        assert_eq!((r.word(0xd6a0 + 32), r.word(0xd6c4 + 32)), (0xb700, 0xb800));
        assert_eq!(r.word(0xd658 + 34), 9);
        assert_eq!((r.word(core::X), r.word(core::SUB_X), r.word(core::SPEED_X)), (101, 0x1000, 0x110));
        assert_eq!((r.word(core::INSTRUCTION), r.word(core::TIMER), r.word(core::SPRITEMAP)), (0x9008, 3, 0x9100));
        assert_eq!(r.word(0xc90), 4);
        assert_eq!(run(Entry::HandleProjectile, &mut ram, &rom, &mut NoPlm), Ok(Outcome::Handled));
        let r = Wram::new(&mut ram).unwrap();
        assert_eq!((r.word(core::X), r.word(core::SUB_X), r.word(core::SPEED_X)), (102, 0x3000, 0x120));
        assert_eq!((r.word(core::TIMER), r.word(0xc90)), (2, 3));
        assert_eq!(r.word(0xd658 + 30), 0);
    }
}

#[test]
fn signed_flight_preserves_fraction() {
    let mut ram = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut ram).unwrap();
    setup(&mut r);
    r.set_word(core::SPEED_X, 0xff80);
    assert_eq!(collision::collide(&mut r, &[], &mut NoPlm, 0, false), Ok(0));
    assert_eq!(r.word(core::X), 99);
    assert_eq!(r.word(core::SUB_X), 0x8000);
}

#[test]
fn initial_left_probe_precedes_regular_velocity() {
    let mut rom = vec![0; 0x300000];
    rom[rom_offset(0x91b629 + 8) + 3] = 7;
    rom_word(&mut rom, 0x9383c1, 0x9000);
    rom_word(&mut rom, 0x939000, 20);
    rom_word(&mut rom, 0x939010, 0xa000);
    rom_word(&mut rom, 0x93a004, 0x0404);
    let mut ram = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut ram).unwrap();
    setup(&mut r);
    for (at, value) in [(0xa1c, 1), (0xaf6, 100), (0xafa, 100), (0xcce, 0), (0x9b2, 0x40), (0x8f, 0x40)] {
        r.set_word(at, value);
    }
    core::fire(&mut r, &rom, &mut NoPlm).unwrap();
    assert_eq!((r.word(core::X), r.word(core::SUB_X), r.word(core::SPEED_X), r.word(0xcce)), (88, 0, 0xfc00, 1));
}

#[test]
fn contact_deletion_still_runs_instruction_timer_decrement() {
    let mut ram = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut ram).unwrap();
    r.set_word(core::INSTRUCTION, 0x9000);
    r.set_word(core::PRE, 0xaef3);
    r.set_word(core::TYPE, 0x8000);
    r.set_word(core::DIRECTION, 0x12);
    r.set_word(0xcce, 1);
    core::update(&mut r, &[], &mut NoPlm).unwrap();
    assert_eq!(r.word(core::INSTRUCTION), 0);
    assert_eq!(r.word(core::TIMER), u16::MAX);
    assert_eq!(r.word(0xcce), 0);
}

#[test]
fn malformed_instruction_is_owned_error() {
    let mut rom = vec![0; 0x300000];
    rom_word(&mut rom, 0x939000, 0xdead);
    for status in [0, 0x8000] {
        let mut ram = vec![0; WRAM_SIZE];
        let mut r = Wram::new(&mut ram).unwrap();
        for (at, value) in
            [(0x93f, status), (core::INSTRUCTION, 0x9000), (core::TIMER, 1), (core::PRE, 0xb169), (core::TYPE, 0x8700)]
        {
            r.set_word(at, value);
        }
        assert!(
            run(Entry::HandleProjectile, &mut ram, &rom, &mut NoPlm)
                .unwrap_err()
                .contains("unknown projectile instruction")
        );
    }
}

#[test]
fn normal_slot_zero_flickers_on_odd_frames() {
    let mut rom = vec![0; 0x300000];
    rom_word(&mut rom, 0x939000, 1);
    rom_word(&mut rom, 0x939002, 0);
    rom[rom_offset(0x939004)] = 0;
    rom_word(&mut rom, 0x939005, 0x2345);
    for (frame, expected) in [(0, 0), (1, 4)] {
        let mut ram = vec![0; WRAM_SIZE];
        let mut r = Wram::new(&mut ram).unwrap();
        for (at, value) in [
            (core::INSTRUCTION, 0x9010),
            (core::SPRITEMAP, 0x9000),
            (core::TYPE, 0x8000),
            (core::X, 100),
            (core::Y, 80),
            (0x5b6, frame),
        ] {
            r.set_word(at, value);
        }
        draw::player(&mut r, &rom).unwrap();
        assert_eq!(r.word(0x590), expected);
        if expected != 0 {
            assert_eq!((r.byte(0x370), r.byte(0x371), r.word(0x372)), (100, 80, 0x2345));
        }
    }
}
