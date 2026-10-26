//! Synthetic palette observations compared with frozen actual-old outputs.
//! No cartridge-derived fixtures.
//! Full RAM and return values cover effects; image replacement is between calls,
//! not evidence of a callback replacing ROM inside a single interpreter step.
#![forbid(unsafe_code)]
use crate::game::palette_effects as palette;
use crate::ram_map::{word_arrays as a, words as w};
use crate::{game_state::Game, rom::RomImage};
use std::panic::{AssertUnwindSafe, catch_unwind};
const SETUPS: [u32; 4] = [9291397, 9298436, 9299008, 9303856];
const PRE: [u32; 8] = [
    9291046, 9298443, 9298656, 9298809, 9301081, 9301380, 9301701, 9303585,
];
const COMMANDS: [u32; 19] = [
    9291157, 9291161, 9291170, 9291179, 9291188, 9291197, 9291206, 9291215, 9291220, 9291229,
    9291294, 9291321, 9291336, 9291349, 9291358, 9291370, 9291379, 9291388, 9302470,
];
pub(crate) const CASES: u32 = 512;
fn put(g: &mut Game, address: usize, value: u16) {
    g.g_ram[address..address + 2].copy_from_slice(&value.to_le_bytes());
}
fn rw(image: &RomImage, cursor: u16, value: u16) {
    let at = crate::rtl::lorom_offset(0x8d0000 | u32::from(cursor)) as usize;
    for (i, byte) in value.to_ne_bytes().into_iter().enumerate() {
        image.write(at + i, byte);
    }
}
fn slot(g: &mut Game, k: u16, column: usize, value: u16) {
    put(g, column + usize::from(k & !1), value);
}
fn seed(g: &mut Game, image: &RomImage) {
    for address in 0x1e79..0x1ef1 {
        g.g_ram[address] = (address * 37) as u8;
    }
    for k in (0..16).step_by(2) {
        for (column, value) in [
            (a::palettefx_ids, 0),
            (a::palettefx_pre_instr, 0xc526),
            (a::palettefx_instr_list_ptrs, 0x9000),
            (a::palettefx_instr_timers, 1),
        ] {
            slot(g, k, column, value);
        }
    }
    put(g, w::flag_for_palette_fx_objects, 0x9234);
    put(g, w::palettefx_index, 0);
    put(g, w::samus_in_heat_palfx_index, 0);
    put(g, w::samus_in_heat_palettefx_prev_index, 0);
    rw(image, 0x8000, SETUPS[0] as u16);
    rw(image, 0x8002, 0x9000);
    rw(image, 0x9000, 0x9234);
    for table in [0xe3e0u16, 0xe400, 0xe420] {
        for index in 0..4 {
            rw(image, table + index * 2, 0x9000 + index * 0x10);
        }
    }
}
fn branches(g: &mut Game, variant: u32) {
    put(
        g,
        w::equipped_items,
        [0, 1, 0x20, 0x21, 0, 0][variant as usize],
    );
    put(g, w::area_index, 0);
    put(g, 0xd828, if variant & 1 == 0 { 0 } else { 2 });
    put(
        g,
        w::cinematic_function,
        if variant & 1 == 0 { 0 } else { 0xb35f },
    );
    put(
        g,
        0xf78 + core::mem::offset_of!(crate::types::EnemyData, health),
        (variant & 1) as u16,
    );
    put(
        g,
        w::samus_y_pos,
        if variant & 1 == 0 { 0x37f } else { 0x380 },
    );
    put(g, w::samus_periodic_subdamage, 0xf000);
    put(g, w::samus_periodic_damage, 0xffff);
    put(g, w::samus_health, if variant == 4 { 0x46 } else { 0x47 });
    put(
        g,
        w::nmi_frame_counter_word,
        if variant == 5 { 1 } else { 0 },
    );
    put(g, w::samus_in_heat_palfx_index, (variant % 3 + 1) as u16);
    put(
        g,
        w::samus_in_heat_palettefx_prev_index,
        if variant == 0 { 1 } else { 0 },
    );
}
fn execute(id: u32, g: &mut Game, image: &RomImage) -> (u16, u16) {
    match id {
        0..=7 => {
            for k in (0..16).step_by(2) {
                slot(g, k, a::palettefx_ids, 0x8000 + k);
            }
            match id {
                0 => palette::clear(g),
                1 => {
                    palette::set_enabled(false, g);
                    let disabled = u16::from_le_bytes([g.g_ram[0x1e79], g.g_ram[0x1e7a]]);
                    palette::set_enabled(true, g);
                    return (
                        disabled,
                        u16::from_le_bytes([g.g_ram[0x1e79], g.g_ram[0x1e7a]]),
                    );
                }
                2 => {
                    g.loaded_rom = None;
                    palette::spawn(0xffff, g);
                }
                3..=6 => {
                    let k = [14, 0, 8, 6][(id - 3) as usize];
                    slot(g, k, a::palettefx_ids, 0);
                    if id == 6 {
                        slot(g, 2, a::palettefx_ids, 0);
                    }
                    rw(image, 0x8000, SETUPS[(id - 3) as usize] as u16);
                    branches(g, id - 3);
                    palette::spawn(0x8000, g);
                }
                _ => {
                    slot(g, 14, a::palettefx_ids, 0);
                    g.loaded_rom = None;
                    palette::spawn(0x8000, g);
                }
            }
        }
        8..=79 => {
            let n = id - 8;
            let k = [0, 1, 14][(n % 3) as usize];
            branches(g, n / 3 % 6);
            slot(g, k, a::palettefx_ids, 0x8123);
            palette::checks::setup(SETUPS[(n / 18) as usize], 0xffff, k, g);
        }
        80..=223 => {
            let n = id - 80;
            let k = [0, 1, 14][(n % 3) as usize];
            let variant = n / 3 % 6;
            branches(g, variant);
            slot(g, k, a::palettefx_ids, 0x8123);
            if n / 18 == 7 {
                put(
                    g,
                    w::flag_for_palette_fx_objects + usize::from(k),
                    (variant & 1) as u16,
                );
            }
            palette::checks::pre(PRE[(n / 18) as usize], k, g);
        }
        224..=451 => {
            let n = id - 224;
            let k = [0, 1, 14, 15, 16, 0x8000][(n / 2 % 6) as usize];
            slot(g, k, a::palettefx_timers, if n & 1 == 0 { 1 } else { 0 });
            put(g, w::palettefx_index, 3);
            if n & 1 != 0 {
                put(g, w::game_state, 40);
            }
            let result = palette::checks::command(COMMANDS[(n / 12) as usize], k, 0x9000, g);
            return (result.k, result.j);
        }
        452..=471 => {
            let n = id - 452;
            let k = if n & 1 == 0 { 0 } else { 14 };
            put(g, w::palettefx_index, k);
            slot(g, k, a::palettefx_ids, 0x8000);
            let color = [0, 1, 0x6c8, 0x1e1c, 0x4002, 0xa402][(n % 6) as usize];
            slot(g, k, a::palettefx_color_indexes, color);
            // Slot phase sets the color; color phase skips two colors, then
            // saves the continuation for Selected rather than the color index.
            for (at, value) in [
                (0x9000, COMMANDS[13] as u16),
                (0x9002, color),
                (0x9004, 3),
                (0x9006, 0x123),
                (0x9008, COMMANDS[1] as u16),
                (0x900a, 0x234),
                (0x900c, COMMANDS[0] as u16),
            ] {
                rw(image, at, value);
            }
            if n == 9 {
                rw(image, 0x900c, COMMANDS[7] as u16);
            }
            if n == 12 {
                palette::set_enabled(false, g);
            }
            if n == 13 {
                slot(g, k, a::palettefx_instr_timers, 0);
            }
            if n == 14 {
                slot(g, k, a::palettefx_instr_timers, 2);
            }
            if n == 15 {
                slot(g, k, a::palettefx_pre_instr, PRE[2] as u16);
            }
            if n >= 16 {
                slot(g, k, a::palettefx_pre_instr, PRE[3] as u16);
                branches(g, n - 16);
                // Selected heat table entries point to the same finite stream.
                for table in [0xe3e0u16, 0xe400, 0xe420] {
                    for index in 0..4 {
                        rw(image, table + index * 2, 0x9000);
                    }
                }
            }
            palette::advance(g);
            if n == 10 || n == 11 {
                let replacement = RomImage::zeroed(image.len());
                for (at, value) in [(0x9100, 5), (0x9102, 0x345), (0x9104, COMMANDS[0] as u16)] {
                    rw(&replacement, at, value);
                }
                g.loaded_rom = Some(replacement);
                put(g, w::palettefx_index, k);
                slot(g, k, a::palettefx_instr_timers, 1);
                slot(g, k, a::palettefx_instr_list_ptrs, 0x9100);
                palette::checks::process_one(k, g);
            }
        }
        472..=483 => {
            let n = id - 472;
            let length = [0, 1, 2][(n % 3) as usize];
            let short = RomImage::zeroed(crate::rtl::lorom_offset(0x8d9000) as usize + length);
            if length != 0 {
                short.write(short.len() - length, 0x34);
            }
            if length == 2 {
                short.write(short.len() - 1, 0x92);
            }
            g.loaded_rom = Some(short);
            slot(g, 1, a::palettefx_timers, if n / 3 == 0 { 1 } else { 0 });
            let ea = [COMMANDS[11], COMMANDS[11], COMMANDS[12], COMMANDS[15]][(n / 3) as usize];
            let result = palette::checks::command(ea, 1, 0x9000, g);
            return (result.k, result.j);
        }
        484..=487 => {
            // Low-window records diagnose but continue; lazy completed countdown
            // must skip even that diagnostic. Returned records mirror the high window.
            slot(g, 1, a::palettefx_timers, if id == 484 { 1 } else { 0 });
            let ea = [COMMANDS[11], COMMANDS[11], COMMANDS[12], COMMANDS[15]][(id - 484) as usize];
            let result = palette::checks::command(ea, 1, 0x1000, g);
            return (result.k, result.j);
        }
        488..=506 => {
            slot(g, 0xffff, a::palettefx_timers, 2);
            rw(image, 0xfffe, 0x9234);
            put(g, w::palettefx_index, 0xffff);
            let result = palette::checks::command(COMMANDS[(id - 488) as usize], 0xffff, 0xfffe, g);
            return (result.k, result.j);
        }
        507..=510 => {
            // The real music callback can alias this owner's fields through
            // an oversized queue index. Its paired delay write is also observable.
            let target = [0x1e79, 0x1e7b, 0x1e9d, 0x1eed][(id - 507) as usize];
            put(g, w::music_queue_write_pos, target - 0x619);
            put(g, w::music_queue_read_pos, 1);
            let result = palette::checks::command(COMMANDS[14], 1, 0x9000, g);
            return (result.k, result.j);
        }
        511 => {
            // Music changes Selected14 to0, while the current slot stays14.
            // Save-next uses the returned live Selected and the outer scan ends.
            put(g, w::palettefx_index, 14);
            put(g, w::music_queue_write_pos, 0x1862);
            slot(g, 14, a::palettefx_ids, 0x8000);
            slot(g, 14, a::palettefx_color_indexes, 0);
            for (at, value) in [
                (0x9000, COMMANDS[14] as u16),
                (0x9002, 0),
                (0x9003, 3),
                (0x9005, 0x123),
                (0x9007, COMMANDS[0] as u16),
            ] {
                rw(image, at, value);
            }
            palette::advance(g);
        }
        _ => unreachable!(),
    }
    (0, 0)
}
pub(crate) fn capture_case(id: u32) -> Option<Vec<u8>> {
    if id >= CASES {
        return None;
    }
    let mut game = Game::new();
    let image = RomImage::zeroed(0x80000);
    seed(&mut game, &image);
    game.loaded_rom = Some(image.clone());
    game.register_native_rom(image.clone());
    let result = catch_unwind(AssertUnwindSafe(|| execute(id, &mut game, &image)));
    let expected_panic = id == 7 || matches!(id, 475 | 476 | 478 | 481 | 482);
    assert_eq!(result.is_err(), expected_panic, "palette scenario {id}");
    assert_eq!(
        game.g_fail,
        (485..=487).contains(&id),
        "palette diagnostic {id}"
    );
    let mut bytes = game.g_ram.to_vec();
    let (k, j) = result.unwrap_or((0, 0));
    bytes.extend_from_slice(&k.to_le_bytes());
    bytes.extend_from_slice(&j.to_le_bytes());
    bytes.push(u8::from(expected_panic));
    bytes.push(u8::from(game.g_fail));
    Some(bytes)
}
