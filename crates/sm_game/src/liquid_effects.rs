//! Liquid setup, rising motion, tide and background HDMA tables.
use crate::constants::bank_87::{ADDR_K_ANIMTILES_ACID, ADDR_K_ANIMTILES_LAVA};
use crate::hdma::HdmaObject;
use crate::menus::prelude::*;

pub const WATER_WAVE: [i16; 16] = [0, 1, 1, 0, 0, -1, -1, 0, 0, 1, 1, 0, 0, -1, -1, 0];

pub fn initialize_lava<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    initialize_hot(ADDR_K_ANIMTILES_LAVA as u16, game);
}
pub fn initialize_acid<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    initialize_hot(ADDR_K_ANIMTILES_ACID as u16, game);
}
fn initialize_hot<H: crate::menus::host::Host + ?Sized>(tiles: u16, game: &mut H) {
    crate::liquid::LiquidWords::set(game, |words| &mut words.liquid.rising, 0xb343);
    crate::liquid::LiquidWords::set(
        game,
        |words| &mut words.liquid.lava_acid.whole,
        crate::liquid::LiquidWords::read(game).height,
    );
    static SURFACE: HdmaObjectDefinition = HdmaObjectDefinition::new(0x42, 0x12, 0xc3e1);
    static BACKGROUND: HdmaObjectDefinition = HdmaObjectDefinition::new(0x42, 0x10, 0xc3f0);
    game.spawn_hdma(0x88, &SURFACE);
    game.spawn_hdma(0x88, &BACKGROUND);
    game.spawn_bg3_scroll();
    game.spawn_animated_tiles(tiles);
}
pub fn initialize_water<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    crate::liquid::LiquidWords::set(game, |words| &mut words.liquid.rising, 0xc428);
    crate::liquid::LiquidWords::set(
        game,
        |words| &mut words.liquid.surface.whole,
        crate::liquid::LiquidWords::read(game).height,
    );
    static SURFACE: HdmaObjectDefinition = HdmaObjectDefinition::new(0x42, 0x11, 0xd856);
    static BACKGROUND: HdmaObjectDefinition = HdmaObjectDefinition::new(0x42, 0xf, 0xd847);
    game.spawn_hdma(0x88, &SURFACE);
    if crate::liquid::LiquidWords::read(game).options & 2 != 0 {
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

static LAVA_SOUND_EFFECTS: [u8; 8] = [0x12, 0x13, 0x14, 0x12, 0x13, 0x14, 0x12, 0x13];
pub fn lava_surface<H>(object: &mut HdmaObject, game: &mut H)
where
    H: crate::menus::host::Host + crate::runtime_access::RuntimeAccess + ?Sized,
{
    word!(game.FX_LAYER_BLENDING_CONFIG_C = word!(game.FX_LAYER_BLENDING_CONFIG_B));
    if word!(game.TIME_IS_FROZEN_FLAG) == 0 {
        step_liquid(game, crate::liquid::Surface::LavaAcid);
        word!(game.BG3_XPOS = word!(game.REG_BG1HOFS));
        crate::word_transfers!(game, ascending, 1; native(0x9c00) <- value(0));
        word!(game.BG3_YPOS = 0);
        let scroll = surface_scroll(
            crate::liquid::LiquidWords::read(game)
                .liquid
                .lava_acid
                .whole,
            crate::camera::current(game).layer1[1],
        );
        crate::word_transfers!(game, ascending, 1; native(0x9c02) <- value(scroll));
        if crate::liquid::LiquidWords::read(game).liquid.kind == 2
            && (crate::liquid::LiquidWords::read(game)
                .liquid
                .lava_acid
                .whole
                & 0x8000)
                == 0
        {
            let remaining = object.c;
            object.c = remaining.wrapping_sub(1);
            if remaining == 1 {
                object.c = 112;
                game.queue_sound(
                    1,
                    (LAVA_SOUND_EFFECTS[(word!(game.RANDOM_NUMBER) as i32 & 7) as usize] as u16)
                        as u8,
                    6,
                );
            }
        }
        word!(game.RANDOM_NUMBER = u16::swap_bytes(word!(game.RANDOM_NUMBER)));
        let v8 = table_height(
            crate::liquid::LiquidWords::read(game)
                .liquid
                .lava_acid
                .whole,
            crate::camera::current(game).layer1[1],
            false,
        );
        object.table = (3 * (((v8 as i32 ^ 0x1ff) + 1) & 0x3ff) + 0xb62a) as u16;
    }
}
pub fn water_surface<H>(object: &mut HdmaObject, game: &mut H)
where
    H: crate::menus::host::Host + crate::runtime_access::RuntimeAccess + ?Sized,
{
    word!(game.FX_LAYER_BLENDING_CONFIG_C = word!(game.FX_LAYER_BLENDING_CONFIG_B));
    if word!(game.TIME_IS_FROZEN_FLAG) != 0 {
        return;
    }
    step_liquid(game, crate::liquid::Surface::Water);
    let layer1 = crate::camera::current(game).layer1;
    word!(
        game.BG3_YPOS = surface_scroll(
            crate::liquid::LiquidWords::read(game).liquid.surface.whole,
            layer1[1]
        )
    );
    let r20: u16 = layer1[0].wrapping_add(((object.c >> 8) as u8 as i8) as u16);
    let remaining = object.b;
    object.b = remaining.wrapping_sub(1);
    if remaining == 1 {
        object.b = 10;
        object.a = ((object.a as i32 + 2) & 0x1e) as u16;
    }
    let mut v7: u8 = object.a as u8;
    for i in (0i32..=30).rev().step_by(2) {
        let value = (WATER_WAVE[(i >> 1) as usize] as u16).wrapping_add(r20);
        crate::word_transfers!(game, ascending, 1; native(0x9c04 + usize::from(v7)) <- value(value));
        v7 = ((v7 as i32 - 2i32) & 0x1e) as u8;
    }
    if (crate::liquid::LiquidWords::read(game).options & 1) != 0 {
        object.c = object.c.wrapping_add(64);
    }
    let v9 = table_height(
        crate::liquid::LiquidWords::read(game).liquid.surface.whole,
        crate::camera::current(game).layer1[1],
        true,
    );
    object.table = (3 * (((v9 as i32 ^ 0x1ff) + 1) & 0x3ff) + 0xc645) as u16;
}

#[derive(Clone, Copy)]
enum Wave {
    LavaHorizontal,
    LavaVertical,
    Water,
}
fn fill_wave<H: crate::menus::host::Host + ?Sized>(
    object: &mut HdmaObject,
    wave: Wave,
    game: &mut H,
) {
    object.b = object.b.wrapping_sub(1);
    if object.b == 0 {
        object.b = if matches!(wave, Wave::LavaVertical) {
            4
        } else {
            6
        };
        let step = if matches!(wave, Wave::Water) { 2 } else { -2 };
        object.a = ((object.a as i32 + step) & 0x1e) as u16;
    }
    let horizontal = matches!(wave, Wave::LavaHorizontal);
    let scroll_phase = if horizontal {
        0
    } else {
        2 * (word!(game.REG_BG2VOFS) & 0xf)
    };
    let mut sample = object.a as u8;
    if !horizontal {
        sample = ((sample as u16 + scroll_phase) & 0x1e) as u8;
    }
    let mut row = ((scroll_phase + 30) & 0x1e) as u8;
    for _ in 0..16 {
        let value = match wave {
            Wave::LavaHorizontal => {
                (crate::gameplay_tables::lava_wave(game, false, (sample >> 1) as isize)
                    .wrapping_add(word!(game.REG_BG2HOFS)))
                    & 0x1ff
            }
            Wave::LavaVertical => {
                (crate::gameplay_tables::lava_wave(game, true, (sample >> 1) as isize)
                    .wrapping_add(word!(game.REG_BG2VOFS)))
                    & 0x1ff
            }
            Wave::Water => {
                (WATER_WAVE[(sample >> 1) as usize] as u16).wrapping_add(word!(game.REG_BG2HOFS))
            }
        };
        // Water's table starts one word later than lava/acid's.
        word!(
            game.LIQUID_BG2_Y_SCROLL_TABLE
                [(row >> 1) as usize + matches!(wave, Wave::Water) as usize] = value
        );
        row = row.wrapping_sub(2) & 0x1e;
        sample = sample.wrapping_sub(2) & 0x1e;
    }
}
pub fn lava_background<H: crate::menus::host::Host + ?Sized>(
    object: &mut HdmaObject,
    game: &mut H,
) {
    {
        word!(game.BG2_Y_SCROLL_SPARE_ENTRY = word!(game.REG_BG2VOFS));
        let active = word!(game.TIME_IS_FROZEN_FLAG) == 0
            && crate::liquid::LiquidWords::read(game).options & 6 != 0;
        let horizontal = active && crate::liquid::LiquidWords::read(game).options & 2 == 0;
        game.write_register(
            (object.bank_slot & 0xff).wrapping_add(BBAD0 as u16),
            if horizontal { 0xf } else { 0x10 },
        );
        if active {
            fill_wave(
                object,
                if horizontal {
                    Wave::LavaHorizontal
                } else {
                    Wave::LavaVertical
                },
                game,
            );
        } else {
            let value = word!(game.REG_BG2VOFS) & 0x1ff;
            crate::word_transfers!(game, descending, 16; le(LIQUID_BG2_Y_SCROLL_TABLE, 15) <- value(value));
        }
        object.table = 3 * (word!(game.REG_BG2VOFS) & 0xf) + 0xc0b1;
    }
}
pub fn water_background<H: crate::menus::host::Host + ?Sized>(
    object: &mut HdmaObject,
    game: &mut H,
) {
    word!(game.BG2_Y_SCROLL_SPARE_ENTRY = word!(game.REG_BG2HOFS));
    if word!(game.TIME_IS_FROZEN_FLAG) != 0 {
        water_flat(game);
    } else if crate::liquid::LiquidWords::read(game).options & 2 == 0 {
        object.channels = 0;
        water_flat(game);
    } else {
        water_wave(object, game);
    }
    let height = table_height(
        crate::liquid::LiquidWords::read(game).liquid.surface.whole,
        crate::camera::current(game).layer1[1],
        true,
    );
    object.table = 3 * ((height ^ 0x1ff) & 0x3ff) + 0xcf46;
}
pub fn water_wave<H: crate::menus::host::Host + ?Sized>(object: &mut HdmaObject, game: &mut H) {
    fill_wave(object, Wave::Water, game);
}
pub fn water_flat<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    {
        let value = word!(game.REG_BG2HOFS);
        crate::word_transfers!(game, descending, 16; native(0x9c66) <- value(value));
    }
}
