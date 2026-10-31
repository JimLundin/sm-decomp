//! Additional actual-old inputs for newly consolidated flare decisions/reset.
//! Compile as a child of verification/player_projectiles.rs, separate from44.
#![forbid(unsafe_code)]
use super::*;
pub(crate) const CASES: u32 = 12;
pub(crate) fn capture(id: u32) -> Option<Vec<u8>> {
    if id >= CASES {
        return None;
    }
    let mut g = Game::new();
    let mut images = Images::new();
    fixture(if id < 8 { 7 } else { 9 }, &mut g, &mut images);
    if id < 8 {
        put(&mut g, 0xcd0, 30);
        put(&mut g, 0xaf6, 100);
        put(&mut g, 0xafa, 100);
        for part in 0..3 {
            put(&mut g, 0xcd6 + part * 2, (part * 2) as u16);
            put(&mut g, 0xcdc + part * 2, 2);
        }
        images.native[lorom(0x91b629) + 3] = 2;
        images.native[lorom(0x91b629) + 4] = 4;
        // Different loaded pose data proves decisions still use native content.
        images.loaded[lorom(0x91b629) + 3] = 255;
        images.loaded[lorom(0x91b629) + 4] = 90;
        for index in 0..64 {
            images.word(0x93a1a1 + index * 2, 0xa800);
        }
        images.word(0x93a800, 1);
        images.word(0x93a802, 0);
        images.word(0x93a804, 0x7700);
        images.word(0x93a806, 0x22);
        match id {
            0 => {}
            1 => put(&mut g, 0xa1e, 0x0104),
            2 | 3 => {
                put(&mut g, crate::ram_map::words::CERES_STATUS, 0x8000);
                put(&mut g, crate::ram_map::words::REG_M7X, 80);
                put(&mut g, crate::ram_map::words::REG_M7Y, 120);
                put(
                    &mut g,
                    crate::ram_map::words::REG_M7A,
                    if id == 2 { 0x100 } else { 0xc0 },
                );
                put(
                    &mut g,
                    crate::ram_map::words::REG_M7B,
                    if id == 2 { 0 } else { 0x80 },
                );
                put(
                    &mut g,
                    crate::ram_map::words::REG_M7C,
                    if id == 2 { 0 } else { 0xff80 },
                );
            }
            4 => put(&mut g, 0xafa, 0xff00),
            5 => images.native[lorom(0x91b629) + 3] = 16,
            6 => images.native[lorom(0x91b629) + 3] = 12,
            7 => {
                images.native[lorom(0x91b629) + 3] = 12;
                put(&mut g, 0xa1e, 0x0100);
            }
            _ => unreachable!(),
        }
    } else {
        for at in (0xb64..0xccc).step_by(2) {
            put(&mut g, at, 0x3456);
        }
        for at in (0xd658..0xd7c0).step_by(2) {
            put(&mut g, at, 0x2468);
        }
        for at in (0xdaa..0xdba).step_by(2) {
            put(&mut g, at, 0x789a);
        }
        for at in (0xaae..0xac8).step_by(2) {
            put(&mut g, at, 0x1234);
        }
        put(
            &mut g,
            crate::ram_map::words::SAMUS_SPECIAL_SUPER_PALETTE_FLAGS,
            if id == 8 { 0 } else { 1 },
        );
        put(
            &mut g,
            crate::ram_map::words::HYPER_BEAM_FLAG,
            if id >= 10 { 1 } else { 0 },
        );
        put(
            &mut g,
            crate::ram_map::words::HUD_AUTO_CANCEL_FLAG,
            if id == 11 { 1 } else { 0 },
        );
        put(&mut g, 0x9d2, 3);
        put(&mut g, 0xa04, 3);
        images.word(0x8de1f0, 9291397u32 as u16);
        images.word(0x8de1f2, 0xe300);
        for index in 0..16 {
            images.word(0x9b9400 + index * 2, 0x100 + index as u16);
        }
    }
    g.loaded_rom = Some(RomImage::from(images.loaded));
    // Baseline11653c1b binding. Candidate changes only to register_native_rom.
    g.register_native_rom(RomImage::from(images.native));
    let result = catch_unwind(AssertUnwindSafe(|| {
        if id < 8 { flare(&mut g) } else { reset(&mut g) }
    }));
    let mut bytes = vec![u8::from(result.is_err()), u8::from(g.g_fail)];
    bytes.extend((0..g.ram().len()).map(|at| g.ram().read_byte(at)));
    Some(bytes)
}
