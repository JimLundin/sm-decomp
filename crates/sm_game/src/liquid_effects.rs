//! Liquid setup, rising motion, tide and background HDMA tables.
// The original's names and assignment shapes are kept.
#![allow(non_snake_case, non_upper_case_globals, unused_mut, unused_assignments)]
use crate::constants::bank_87::{addr_kAnimtiles_Acid, addr_kAnimtiles_Lava};
use crate::menus::prelude::*;
#[allow(unused_imports)]
use crate::ram::RawRam as _;

pub const WATER_WAVE: [i16; 16] = [0, 1, 1, 0, 0, -1, -1, 0, 0, 1, 1, 0, 0, -1, -1, 0];

pub fn initialize_lava<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    initialize_hot(addr_kAnimtiles_Lava as u16, game);
}
pub fn initialize_acid<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    initialize_hot(addr_kAnimtiles_Acid as u16, game);
}
fn initialize_hot<H: crate::menus::host::Host + ?Sized>(tiles: u16, game: &mut H) {
    word!(game.fx_rising_function_bank_88 = 0xb343);
    word!(game.lava_acid_y_pos = word!(game.fx_base_y_pos));
    static SURFACE: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0x42, 0x12, 0xc3e1);
    static BACKGROUND: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0x42, 0x10, 0xc3f0);
    game.spawn_hdma(0x88, &SURFACE);
    game.spawn_hdma(0x88, &BACKGROUND);
    game.spawn_bg3_scroll();
    game.spawn_animated_tiles(tiles);
}
pub fn initialize_water<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    word!(game.fx_rising_function_bank_88 = 0xc428);
    word!(game.fx_y_pos = word!(game.fx_base_y_pos));
    static SURFACE: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0x42, 0x11, 0xd856);
    static BACKGROUND: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0x42, 0xf, 0xd847);
    game.spawn_hdma(0x88, &SURFACE);
    if word!(game.fx_liquid_options) & 2 != 0 {
        game.spawn_hdma(0x88, &BACKGROUND);
    }
    game.spawn_bg3_scroll();
}

/// One frame of the liquid's motion, tide and surface under one Session lease;
/// the room shake and an unknown handler's report follow once it is published.
/// Neither touches liquid state, so their order relative to it is unchanged.
fn step_liquid<H>(game: &mut H, surface: crate::liquid::Surface)
where
    H: crate::menus::host::Host + crate::runtime_access::RuntimeAccess + ?Sized,
{
    let effects = with_liquid(game, |words| crate::liquid::step(words, surface));
    if effects.shake {
        game.shake_room_with_sound(21);
    }
    if effects.unknown_handler {
        game.rtl_unreachable();
        with_liquid(game, |words| {
            crate::liquid::advance_tide(words);
            crate::liquid::place_surface(words, surface);
        });
    }
}

/// Run `update` on the liquid words: on the owned Room under one lease, or on
/// the RAM bridge when a lease is not admissible. Both store the same bytes.
fn with_liquid<H, R>(game: &mut H, update: impl FnOnce(&mut crate::liquid::LiquidWords) -> R) -> R
where
    H: crate::menus::host::Host + crate::runtime_access::RuntimeAccess + ?Sized,
{
    use crate::liquid::LiquidWords;
    use crate::session_access::SessionAccess;
    if game.runtime_admissible() {
        let mut runtime = game.runtime();
        let room = &mut runtime.session_mut().room;
        let mut words = LiquidWords::of(room);
        let result = update(&mut words);
        words.store(room);
        result
    } else {
        let mut words = LiquidWords::read(game);
        let result = update(&mut words);
        words.write(game);
        result
    }
}
fn surface_scroll(position: u16, camera: u16) -> u16 {
    let relative = (position as i16).wrapping_sub(camera as i16);
    if position & 0x8000 != 0 {
        0
    } else if relative <= 0 {
        (relative as u16 ^ 0x1f) & 0x1f | 0x100
    } else if relative < 0x100 {
        !relative as u8 as u16
    } else {
        0
    }
}
fn table_height(position: u16, camera: u16, water: bool) -> u16 {
    let relative = position.wrapping_sub(camera).wrapping_add(256) as i16;
    if position & 0x8000 != 0 {
        511
    } else if relative < 0 {
        if water {
            relative as u16 & 0xf | 0x100
        } else {
            255
        }
    } else {
        (relative as u16).min(511)
    }
}

static kLavaSoundEffects: [u8; 8] = [0x12, 0x13, 0x14, 0x12, 0x13, 0x14, 0x12, 0x13];
#[forbid(unsafe_code)]
pub fn lava_surface<H>(_k: u16, game: &mut H)
where
    H: crate::menus::host::Host + crate::runtime_access::RuntimeAccess + ?Sized,
{
    word!(game.fx_layer_blending_config_c = word!(game.fx_layer_blending_config_b));
    if word!(game.time_is_frozen_flag) == 0 {
        step_liquid(game, crate::liquid::Surface::LavaAcid);
        word!(game.bg3_xpos = word!(game.reg_BG1HOFS));
        crate::word_transfers!(game, ascending, 1; native(0x9c00) <- value(0));
        word!(game.bg3_ypos = 0);
        let scroll = surface_scroll(
            word!(game.lava_acid_y_pos),
            crate::camera::current(game).layer1[1],
        );
        crate::word_transfers!(game, ascending, 1; native(0x9c02) <- value(scroll));
        if word!(game.fx_type) == 2 && (word!(game.lava_acid_y_pos) & 0x8000) == 0 {
            let v5: i32 = word!(game.hdma_object_index) as u8 as i32 >> 1;
            let remaining = word!(game.hdma_object_C[v5]);
            word!(game.hdma_object_C[v5] = remaining.wrapping_sub(1));
            if remaining == 1 {
                word!(game.hdma_object_C[v5] = 112);
                game.queue_sound(
                    1,
                    (kLavaSoundEffects[(word!(game.random_number) as i32 & 7) as usize] as u16)
                        as u8,
                    6,
                );
            }
        }
        word!(game.random_number = u16::swap_bytes(word!(game.random_number)));
        let v8 = table_height(
            word!(game.lava_acid_y_pos),
            crate::camera::current(game).layer1[1],
            false,
        );
        let pointer = (3 * ((v8 as i32 ^ 0x1ff) + 1 & 0x3ff) + 0xb62a) as u16;
        crate::word_transfers!(game, ascending, 1;
            native(0x18d8 + 2 * usize::from(word!(game.hdma_object_index) as u8 >> 1)) <- value(pointer));
    } else {
    }
}
#[forbid(unsafe_code)]
pub fn water_surface<H>(k: u16, game: &mut H)
where
    H: crate::menus::host::Host + crate::runtime_access::RuntimeAccess + ?Sized,
{
    word!(game.fx_layer_blending_config_c = word!(game.fx_layer_blending_config_b));
    if word!(game.time_is_frozen_flag) != 0 {
        return;
    }
    step_liquid(game, crate::liquid::Surface::Water);
    let layer1 = crate::camera::current(game).layer1;
    word!(game.bg3_ypos = surface_scroll(word!(game.fx_y_pos), layer1[1]));
    let v5: i32 = k as i32 >> 1;
    let r20: u16 =
        layer1[0].wrapping_add((game.ram().read_byte(0x192d + 2 * v5 as usize) as i8) as u16);
    let remaining = word!(game.hdma_object_B[v5]);
    word!(game.hdma_object_B[v5] = remaining.wrapping_sub(1));
    if remaining == 1 {
        word!(game.hdma_object_B[v5] = 10);
        word!(game.hdma_object_A[v5] = (word!(game.hdma_object_A[v5]) as i32 + 2 & 0x1e) as u16);
    }
    let mut v7: u8 = word!(game.hdma_object_A[v5]) as u8;
    for i in (0i32..=30).rev().step_by(2) {
        let value = (WATER_WAVE[(i >> 1) as usize] as u16).wrapping_add(r20);
        crate::word_transfers!(game, ascending, 1; native(0x9c04 + usize::from(v7)) <- value(value));
        v7 = (v7 as i32 - 2i32 & 0x1e) as u8;
    }
    if (word!(game.fx_liquid_options) & 1) != 0 {
        word!(game.hdma_object_C[(k >> 1)] += 64);
    }
    let v9 = table_height(
        word!(game.fx_y_pos),
        crate::camera::current(game).layer1[1],
        true,
    );
    word!(
        game.hdma_object_table_pointers[(k >> 1)] =
            (3 * ((v9 as i32 ^ 0x1ff) + 1 & 0x3ff) + 0xc645) as u16
    );
}

#[derive(Clone, Copy)]
enum Wave {
    LavaHorizontal,
    LavaVertical,
    Water,
}
#[forbid(unsafe_code)]
fn fill_wave<H: crate::menus::host::Host + ?Sized>(k: u16, wave: Wave, game: &mut H) {
    let slot = (k >> 1) as usize;
    if word!(game.hdma_object_B[slot] -= 1, reread) == 0 {
        word!(
            game.hdma_object_B[slot] = if matches!(wave, Wave::LavaVertical) {
                4
            } else {
                6
            }
        );
        let step = if matches!(wave, Wave::Water) { 2 } else { -2 };
        word!(
            game.hdma_object_A[slot] =
                (word!(game.hdma_object_A[slot]) as i32 + step & 0x1e) as u16
        );
    }
    let horizontal = matches!(wave, Wave::LavaHorizontal);
    let scroll_phase = if horizontal {
        0
    } else {
        2 * (word!(game.reg_BG2VOFS) & 0xf)
    };
    let mut sample = if horizontal {
        game.ram().read_byte(0x1914 + usize::from(k))
    } else {
        word!(game.hdma_object_A[slot]) as u8
    };
    if !horizontal {
        sample = (sample as u16 + scroll_phase & 0x1e) as u8;
    }
    let mut row = (scroll_phase + 30 & 0x1e) as u8;
    for _ in 0..16 {
        let value = match wave {
            Wave::LavaHorizontal => {
                (crate::gameplay_tables::lava_wave(game, false, (sample >> 1) as isize)
                    .wrapping_add(word!(game.reg_BG2HOFS)))
                    & 0x1ff
            }
            Wave::LavaVertical => {
                (crate::gameplay_tables::lava_wave(game, true, (sample >> 1) as isize)
                    .wrapping_add(word!(game.reg_BG2VOFS)))
                    & 0x1ff
            }
            Wave::Water => {
                (WATER_WAVE[(sample >> 1) as usize] as u16).wrapping_add(word!(game.reg_BG2HOFS))
            }
        };
        // Water's table starts one word later than lava/acid's.
        word!(
            game.g_word_7E9C46[(row >> 1) as usize + matches!(wave, Wave::Water) as usize] = value
        );
        row = row.wrapping_sub(2) & 0x1e;
        sample = sample.wrapping_sub(2) & 0x1e;
    }
}
pub fn lava_background<H: crate::menus::host::Host + ?Sized>(k: u16, game: &mut H) {
    {
        word!(game.g_word_7E9C44 = word!(game.reg_BG2VOFS));
        let active = word!(game.time_is_frozen_flag) == 0 && word!(game.fx_liquid_options) & 6 != 0;
        let horizontal = active && word!(game.fx_liquid_options) & 2 == 0;
        game.write_register(
            (game.ram().read_byte(0x18c0 + usize::from(k)) as u16).wrapping_add(BBAD0 as u16),
            if horizontal { 0xf } else { 0x10 },
        );
        if active {
            fill_wave(
                k,
                if horizontal {
                    Wave::LavaHorizontal
                } else {
                    Wave::LavaVertical
                },
                game,
            );
        } else {
            let value = word!(game.reg_BG2VOFS) & 0x1ff;
            crate::word_transfers!(game, descending, 16; le(g_word_7E9C46, 15) <- value(value));
        }
        word!(
            game.hdma_object_table_pointers[(word!(game.hdma_object_index) as u8 >> 1)] =
                3 * (word!(game.reg_BG2VOFS) & 0xf) + 0xc0b1
        );
    }
}
#[forbid(unsafe_code)]
pub fn water_background<H: crate::menus::host::Host + ?Sized>(k: u16, game: &mut H) {
    word!(game.g_word_7E9C44 = word!(game.reg_BG2HOFS));
    if word!(game.time_is_frozen_flag) != 0 {
        water_flat(k, game);
    } else if word!(game.fx_liquid_options) & 2 == 0 {
        word!(game.hdma_object_channels_bitmask[(k as u8 >> 1)] = 0);
        water_flat(k, game);
    } else {
        water_wave(k, game);
    }
    let height = table_height(
        word!(game.fx_y_pos),
        crate::camera::current(game).layer1[1],
        true,
    );
    word!(
        game.hdma_object_table_pointers[(k as u8 >> 1)] = 3 * ((height ^ 0x1ff) & 0x3ff) + 0xcf46
    );
}
#[forbid(unsafe_code)]
pub fn water_wave<H: crate::menus::host::Host + ?Sized>(k: u16, game: &mut H) {
    fill_wave(k as u8 as u16, Wave::Water, game);
}
#[forbid(unsafe_code)]
pub fn water_flat<H: crate::menus::host::Host + ?Sized>(_k: u16, game: &mut H) {
    {
        let value = word!(game.reg_BG2HOFS);
        crate::word_transfers!(game, descending, 16; native(0x9c66) <- value(value));
    }
}
