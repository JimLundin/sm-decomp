//! Synthetic current-entry observations; shared with the later owned integration.
//! First batch: checked command/creation paths and aligned native drawing only.
//! Excludes known unaligned original save/respawn/grey-door/item-frame loads.
use crate::{game_state::Game, rom::RomImage};
use crate::game::{
    sm_84 as old, plm_spawning,
    enemy_script::{EnemyScript, bank_base},
};
use std::panic::{catch_unwind, AssertUnwindSafe};
const COMMANDS: [u32; 14] = [
    8685236, 8685244, 8685249, 8685258, 8685390, 8685375, 8686138, 8686116, 8686126, 8686194,
    8685412, 8685541, 8686608, 8686913,
];
pub(crate) const CASES: u32 = 90;
fn put(g: &mut Game, a: usize, v: u16) {
    g.g_ram[a..a + 2].copy_from_slice(&v.to_le_bytes());
}
fn rom_word(image: &RomImage, address: u32, value: u16) {
    let a = crate::rtl::lorom_offset(address) as usize;
    for (i, b) in value.to_le_bytes().into_iter().enumerate() {
        image.write(a + i, b);
    }
}
fn slot(g: &mut Game, k: usize, header: u16) {
    for (a, v) in [
        (0x1c37, header),
        (0x1c87, 34),
        (0x1cd7, 0x86d0),
        (0x1d27, 0x9000),
        (0x1d77, 2),
        (0x1dc7, 0x8000),
        (0x1e17, 0x1234),
        (0xde1c, 1),
        (0xde6c, 0xa000),
        (0xdebc, 0x9300),
        (0xdf0c, 0x5678),
    ] {
        put(g, a + 2 * k, v);
    }
}
fn seed(g: &mut Game, image: &RomImage) {
    for (a, v) in [
        (0x7a5, 16),
        (0x7a9, 8),
        (0x7ab, 8),
        (0x7b9, 1024),
        (0x1c23, 0x8005),
        (0x1c27, 0),
        (0xdc4, 17),
        (0xaf6, 24),
        (0xafa, 24),
    ] {
        put(g, a, v);
    }
    for i in 0..40 {
        slot(g, i, 0);
    }
    rom_word(image, 0x848000, 8698220u32 as u16);
    rom_word(image, 0x848002, 0x9000);
    for (a, v) in [
        (0x8f9000, 0x8000),
        (0x8f9002, 0x0203),
        (0x8f9004, 0x4567),
        (0x849000, 8685236u32 as u16),
        (0x849100, 0x9200),
        (0x849102, 0x1234),
        (0x849104, 0x5678),
        (0x849106, 0x9abc),
        (0x849108, 0xdef0),
        (0x84a000, 1),
        (0x84a002, 0x123),
        (0x84a004, 0),
    ] {
        rom_word(image, a, v);
    }
    for i in 0..4 {
        rom_word(image, 0x8487cd + 2 * i, 0x6000 + i as u16 * 0x80);
        rom_word(image, 0x8487d5 + 2 * i, 0x40 + i as u16 * 16);
        rom_word(image, 0x8487dd + 2 * i, 0x100 + i as u16 * 8);
    }
    for i in 0..2048 {
        put(g, 0xa000 + 2 * i, (i as u16) ^ 0x1357);
    }
}
fn result(script: EnemyScript) -> (u8, u16) {
    match script {
        EnemyScript::Stop => (0, 0),
        EnemyScript::Jump(a) => (1, a),
        s => (2, s.offset_from(bank_base(0x84)) as u16),
    }
}
// All unsafe calls below are actual current production entries, never modeled effects.
unsafe fn execute(id: u32, g: &mut Game, image: &RomImage) -> (u8, u16) {
    unsafe {
        match id {
            0..=11 => {
                let mode = id / 4;
                let source = id % 4;
                if mode != 0 {
                    for i in 0..40 {
                        put(g, 0x1c37 + 2 * i, 0x8000);
                    }
                    if mode == 2 {
                        put(g, 0x1c37 + 2 * 17, 0);
                    }
                }
                match source {
                    0 => {
                        plm_spawning::SpawnHardcodedPlm(
                            crate::types::SpawnHardcodedPlmArgs {
                                block_x: 3,
                                block_y: 2,
                                header: 0x8000,
                            },
                            g,
                        );
                        (0, 0)
                    }
                    1 => (0, u16::from(plm_spawning::SpawnPLM(0x8000, g))),
                    2 => {
                        plm_spawning::SpawnRoomPLM(0x9000, g);
                        (0, 0)
                    }
                    _ => {
                        for (a, v) in [(0x12, 0x8000), (0x14, 0x0203), (0x16, 0x4567)] {
                            put(g, a, v);
                        }
                        plm_spawning::SpawnRoomPLM(0x12, g);
                        (0, 0)
                    }
                }
            }
            12..=15 => {
                for i in 0..40 {
                    put(g, 0x1c37 + 2 * i, 0x8000);
                }
                put(g, 0x1c2d, 6);
                put(g, 0x1c2f, 0x9999);
                match id {
                    12 => old::ClearPLMs(g),
                    13 => {
                        put(g, 0x1c23, 0x4567);
                        old::EnablePLMs(g)
                    }
                    14 => {
                        put(g, 0x1c23, 0xc567);
                        old::DisablePLMs(g)
                    }
                    _ => {
                        old::DisablePLMs(g);
                        old::EnablePLMs(g)
                    }
                };
                (0, 0)
            }
            16..=57 => {
                let choice = ((id - 16) / 3) as usize;
                let k = [0, 1, 80][((id - 16) % 3) as usize];
                slot(g, 0, 0x8000);
                put(g, 0x1c2d, 6);
                result(old::CallPlmInstr(
                    COMMANDS[choice],
                    EnemyScript::at(image.clone(), crate::rtl::lorom_offset(0x849100) as usize),
                    k,
                    g,
                ))
            }
            58..=65 => {
                let mode = id - 58;
                slot(g, 39, 0x8000);
                match mode {
                    0 => put(g, 0x1c23, 5),
                    1 => put(g, 0x1c37 + 78, 0),
                    2 => put(g, 0xde1c + 78, 0),
                    3 => put(g, 0xde1c + 78, 2),
                    4 => {
                        rom_word(image, 0x849000, 8685390u32 as u16);
                        image.write(crate::rtl::lorom_offset(0x849002) as usize, 7);
                        rom_word(image, 0x849003, 8685236u32 as u16);
                    }
                    5 => {
                        rom_word(image, 0x849000, 1);
                        rom_word(image, 0x849002, 0xa000);
                    }
                    6 => g.g_ram[0x613] = 2,
                    _ => g.g_ram[0x613] = 255,
                }
                (3, u16::from(old::PlmHandler_Async(g)))
            }
            66..=81 => {
                let mode = id - 66;
                slot(g, 0, 0x8000);
                let vertical = mode & 1 != 0;
                let flags = ((mode >> 1) & 3) as u16 * 0x400;
                rom_word(image, 0x84a000, if vertical { 0x8003 } else { 3 });
                for i in 0..3 {
                    rom_word(image, 0x84a002 + 2 * i, 0x123 | flags);
                }
                rom_word(image, 0x84a008, 0);
                if mode >= 8 {
                    for (a, v) in [
                        (0x1800, if vertical { 0x8003 } else { 3 }),
                        (0x1802, 0x123 | flags),
                        (0x1804, 0x124 | flags),
                        (0x1806, 0x125 | flags),
                        (0x1808, 0),
                    ] {
                        put(g, a, v);
                    }
                    put(g, 0xde6c, 0x1800);
                }
                if mode == 12 {
                    put(g, 0x1c87, 30);
                }
                if mode == 13 {
                    put(g, 0x911, 32);
                }
                if mode == 14 {
                    put(g, 0x915, 32);
                }
                if mode == 15 {
                    put(g, 0x1c25, 504);
                }
                old::ProcessPlmDrawInstruction(0, g);
                old::CalculatePlmBlockCoords(0, g);
                old::DrawPlm(0, g);
                (0, 0)
            }
            82..=85 => {
                slot(g, 0, 0x8000);
                put(g, 0x1c2d, 6);
                let short = RomImage::zeroed(1);
                short.write(0, 7);
                result(old::CallPlmInstr(
                    [8685249, 8685390, 8685412, 8685541][(id - 82) as usize],
                    EnemyScript::at(short, 0),
                    0,
                    g,
                ))
            }
            86..=89 => {
                put(g, 0x7a5, [0, 16, 256, 0xffff][(id - 86) as usize]);
                old::CalculatePlmBlockCoords(0, g);
                (0, 0)
            }
            _ => unreachable!(),
        }
    }
}
pub(crate) fn capture_case(id: u32) -> Option<Vec<u8>> {
    if id >= CASES {
        return None;
    }
    let mut g = Game::new();
    let image = RomImage::zeroed(0x200000);
    seed(&mut g, &image);
    g.loaded_rom = Some(image.clone());
    g.g_rom = image.as_mut_ptr();
    if (66..=81).contains(&id) || id == 63 {
        assert_eq!(
            (image.as_mut_ptr() as usize + crate::rtl::lorom_offset(0x84a000) as usize) & 1,
            0,
            "native ROM stream alignment"
        );
        assert_eq!(
            (g.g_ram.as_ptr() as usize + 0x1800) & 1,
            0,
            "native RAM stream alignment"
        );
    }
    let outcome = catch_unwind(AssertUnwindSafe(|| unsafe { execute(id, &mut g, &image) }));
    assert_eq!(
        outcome.is_err(),
        matches!(id, 82 | 84 | 85),
        "unexpected result for baseline case {id}"
    );
    assert!(!g.g_fail, "native diagnostic for baseline case {id}");
    let mut bytes = g.g_ram.to_vec();
    match outcome {
        Ok((kind, value)) => {
            bytes.push(kind);
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        Err(_) => bytes.extend_from_slice(&[0xff, 0, 0]),
    };
    bytes.push(u8::from(g.g_fail));
    Some(bytes)
}
