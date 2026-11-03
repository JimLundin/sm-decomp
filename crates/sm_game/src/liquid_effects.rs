//! Liquid setup, rising motion, tide and background HDMA tables.
use crate::constants::bank_87::{ADDR_K_ANIMTILES_ACID, ADDR_K_ANIMTILES_LAVA};
use crate::fx_domain::native_order;
use crate::menus::prelude::*;
use crate::scene::hdma_sources::LiveOwner;
use crate::scene::state::WordOrder;

pub const WATER_WAVE: [i16; 16] = [0, 1, 1, 0, 0, -1, -1, 0, 0, 1, 1, 0, 0, -1, -1, 0];

/// Start a lava or acid liquid: it idles, its surface at
/// the base height. The room's FX initialisation runs this on its lease, then
/// [`initialize_lava`] or [`initialize_acid`] on the game.
pub fn start_hot(room: &mut crate::session::Room) {
    room.liquid.rising = crate::liquid::rising::HOT_IDLE;
    room.liquid.lava_acid.whole = room.liquid_height;
}
/// Start water: it idles, its surface at the base height. Returns
/// whether the liquid options ask for the background wave, which
/// [`initialize_water`] then spawns.
pub fn start_water(room: &mut crate::session::Room) -> bool {
    room.liquid.rising = crate::liquid::rising::WATER_IDLE;
    room.liquid.surface.whole = room.liquid_height;
    room.liquid_options & 2 != 0
}

/// Spawn the lava's HDMA objects and animated tiles, after [`start_hot`].
pub fn initialize_lava<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    initialize_hot(ADDR_K_ANIMTILES_LAVA as u16, game);
}
/// Spawn the acid's HDMA objects and animated tiles, after [`start_hot`].
pub fn initialize_acid<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    initialize_hot(ADDR_K_ANIMTILES_ACID as u16, game);
}
fn initialize_hot<H: crate::menus::host::Host + ?Sized>(tiles: u16, game: &mut H) {
    static SURFACE: HdmaObjectDefinition = HdmaObjectDefinition::new(0x42, 0x12, 0xc3e1);
    static BACKGROUND: HdmaObjectDefinition = HdmaObjectDefinition::new(0x42, 0x10, 0xc3f0);
    game.spawn_hdma(0x88, &SURFACE);
    game.spawn_hdma(0x88, &BACKGROUND);
    crate::hdma_objects::spawn_bg3_scroll(game);
    crate::animated_tiles::execution::leased::spawn(tiles, game);
}
/// Spawn the water's HDMA objects after [`start_water`]: its surface, and its
/// background wave when `background` is set.
pub fn initialize_water<H: crate::menus::host::Host + ?Sized>(background: bool, game: &mut H) {
    static SURFACE: HdmaObjectDefinition = HdmaObjectDefinition::new(0x42, 0x11, 0xd856);
    static BACKGROUND: HdmaObjectDefinition = HdmaObjectDefinition::new(0x42, 0xf, 0xd847);
    game.spawn_hdma(0x88, &SURFACE);
    if background {
        game.spawn_hdma(0x88, &BACKGROUND);
    }
    crate::hdma_objects::spawn_bg3_scroll(game);
}

/// One frame of the liquid's motion, tide and surface under one Session lease;
/// the room shake and an unknown handler's report follow once it is published.
/// Neither touches liquid state, so their order relative to it is unchanged.
/// Returns the liquid words and the camera as the step left them: the rest of
/// the surface's frame stores neither.
fn step_liquid<H: crate::menus::host::Host + ?Sized>(
    game: &mut H,
    surface: crate::liquid::Surface,
    handler_carry: bool,
) -> Stepped {
    let (effects, mut stepped) = with_liquid(game, |words| crate::liquid::step(words, surface));
    if effects.shake {
        crate::room_visuals::shake_room_with_sound(21, handler_carry, game);
    }
    if effects.unknown_handler {
        game.rtl_unreachable();
        ((), stepped) = with_liquid(game, |words| {
            crate::liquid::advance_tide(words);
            crate::liquid::place_surface(words, surface);
        });
    }
    stepped
}

/// The liquid words and the camera after a liquid step.
struct Stepped {
    words: crate::liquid::LiquidWords,
    camera: crate::camera::Camera,
}

/// Run `update` on the liquid words: on the owned Room under one lease, or on
/// the RAM bridge when a lease is not admissible. Both store the same bytes.
fn with_liquid<H, R>(
    game: &mut H,
    update: impl FnOnce(&mut crate::liquid::LiquidWords) -> R,
) -> (R, Stepped)
where
    H: crate::menus::host::Host + ?Sized,
{
    use crate::liquid::LiquidWords;
    use crate::session_access::SessionAccess;
    if game.runtime_admissible() {
        let mut runtime = game.runtime();
        let room = &mut runtime.session_mut().room;
        let mut words = LiquidWords::of(room);
        let result = update(&mut words);
        words.store(room);
        let camera = room.camera;
        (result, Stepped { words, camera })
    } else {
        let mut words = LiquidWords::read(game);
        let result = update(&mut words);
        words.write(game);
        let camera = crate::camera::current(game);
        (result, Stepped { words, camera })
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
pub fn lava_surface<H: crate::menus::host::Host + ?Sized>(
    hdma_index: usize,
    handler_carry: bool,
    game: &mut H,
) {
    {
        let fx_value = game.room_blending().liquid;
        let mut fx = game.fx_controls();
        fx.blending = fx_value;
        game.publish_fx_controls(fx, &[crate::fx_state::Field::Blending]);
    };
    if crate::sequence_word!(game.TIME_IS_FROZEN_FLAG) == 0 {
        let Stepped { words, camera } =
            step_liquid(game, crate::liquid::Surface::LavaAcid, handler_carry);
        {
            let fx_value = crate::menus::prelude::display_scroll!(game.Bg1H);
            let mut fx = game.fx_controls();
            fx.effect_scroll[0] = fx_value;
            game.publish_fx_controls(fx, &[crate::fx_state::Field::EffectHorizontal]);
        };
        game.store_source_word(LiveOwner::LiquidSurface, 0, native_order(), 0);
        {
            let fx_value = 0;
            let mut fx = game.fx_controls();
            fx.effect_scroll[1] = fx_value;
            game.publish_fx_controls(fx, &[crate::fx_state::Field::EffectVertical]);
        };
        let scroll = surface_scroll(words.liquid.lava_acid.whole, camera.layer1[1]);
        game.store_source_word(LiveOwner::LiquidSurface, 2, native_order(), scroll);
        if words.liquid.kind == 2 && (words.liquid.lava_acid.whole & 0x8000) == 0 {
            let remaining = crate::hdma_word!(game, hdma_index, C);
            crate::hdma_word!(game, hdma_index, C => remaining.wrapping_sub(1));
            if remaining == 1 {
                crate::hdma_word!(game, hdma_index, C => 112);
                game.queue_sound(
                    1,
                    (LAVA_SOUND_EFFECTS[(game.view_word(crate::projection::Field::RandomNumber)
                        as i32
                        & 7) as usize] as u16) as u8,
                    6,
                );
            }
        }
        let random = game.view_word(crate::projection::Field::RandomNumber);
        game.set_view_word(crate::projection::Field::RandomNumber, random.swap_bytes());
        let v8 = table_height(words.liquid.lava_acid.whole, camera.layer1[1], false);
        crate::hdma_word!(game, hdma_index, Table => (3 * (((v8 as i32 ^ 0x1ff) + 1) & 0x3ff) + 0xb62a) as u16);
    }
}
pub fn water_surface<H: crate::menus::host::Host + ?Sized>(
    hdma_index: usize,
    handler_carry: bool,
    game: &mut H,
) {
    {
        let fx_value = game.room_blending().liquid;
        let mut fx = game.fx_controls();
        fx.blending = fx_value;
        game.publish_fx_controls(fx, &[crate::fx_state::Field::Blending]);
    };
    if crate::sequence_word!(game.TIME_IS_FROZEN_FLAG) != 0 {
        return;
    }
    let Stepped { words, camera } = step_liquid(game, crate::liquid::Surface::Water, handler_carry);
    let layer1 = camera.layer1;
    {
        let fx_value = surface_scroll(words.liquid.surface.whole, layer1[1]);
        let mut fx = game.fx_controls();
        fx.effect_scroll[1] = fx_value;
        game.publish_fx_controls(fx, &[crate::fx_state::Field::EffectVertical]);
    };
    let r20: u16 =
        layer1[0].wrapping_add(((crate::hdma_word!(game, hdma_index, C) >> 8) as u8 as i8) as u16);
    let remaining = crate::hdma_word!(game, hdma_index, B);
    crate::hdma_word!(game, hdma_index, B => remaining.wrapping_sub(1));
    if remaining == 1 {
        crate::hdma_word!(game, hdma_index, B => 10);
        crate::hdma_word!(game, hdma_index, A => ((crate::hdma_word!(game, hdma_index, A) as i32 + 2) & 0x1e) as u16);
    }
    let mut v7: u8 = crate::hdma_word!(game, hdma_index, A) as u8;
    for i in (0i32..=30).rev().step_by(2) {
        let value = (WATER_WAVE[(i >> 1) as usize] as u16).wrapping_add(r20);
        game.store_source_word(
            LiveOwner::LiquidSurface,
            4 + usize::from(v7),
            native_order(),
            value,
        );
        v7 = ((v7 as i32 - 2i32) & 0x1e) as u8;
    }
    if (words.options & 1) != 0 {
        crate::hdma_word!(game, hdma_index, C => crate::hdma_word!(game, hdma_index, C).wrapping_add(64));
    }
    let v9 = table_height(words.liquid.surface.whole, layer1[1], true);
    crate::hdma_word!(game, hdma_index, Table => (3 * (((v9 as i32 ^ 0x1ff) + 1) & 0x3ff) + 0xc645) as u16);
}

#[derive(Clone, Copy)]
enum Wave {
    LavaHorizontal,
    LavaVertical,
    Water,
}
fn fill_wave<H: crate::menus::host::Host + ?Sized>(hdma_index: usize, wave: Wave, game: &mut H) {
    crate::hdma_word!(game, hdma_index, B => crate::hdma_word!(game, hdma_index, B).wrapping_sub(1));
    if crate::hdma_word!(game, hdma_index, B) == 0 {
        crate::hdma_word!(game, hdma_index, B => if matches!(wave, Wave::LavaVertical) {
            4
        } else {
            6
        });
        let step = if matches!(wave, Wave::Water) { 2 } else { -2 };
        crate::hdma_word!(game, hdma_index, A => ((crate::hdma_word!(game, hdma_index, A) as i32 + step) & 0x1e) as u16);
    }
    let horizontal = matches!(wave, Wave::LavaHorizontal);
    let scroll_phase = if horizontal {
        0
    } else {
        2 * (crate::menus::prelude::display_scroll!(game.Bg2V) & 0xf)
    };
    let mut sample = crate::hdma_word!(game, hdma_index, A) as u8;
    if !horizontal {
        sample = ((sample as u16 + scroll_phase) & 0x1e) as u8;
    }
    let mut row = ((scroll_phase + 30) & 0x1e) as u8;
    for _ in 0..16 {
        let value = match wave {
            Wave::LavaHorizontal => {
                (crate::gameplay_tables::lava_wave(game, false, (sample >> 1) as isize)
                    .wrapping_add(crate::menus::prelude::display_scroll!(game.Bg2H)))
                    & 0x1ff
            }
            Wave::LavaVertical => {
                (crate::gameplay_tables::lava_wave(game, true, (sample >> 1) as isize)
                    .wrapping_add(crate::menus::prelude::display_scroll!(game.Bg2V)))
                    & 0x1ff
            }
            Wave::Water => (WATER_WAVE[(sample >> 1) as usize] as u16)
                .wrapping_add(crate::menus::prelude::display_scroll!(game.Bg2H)),
        };
        // Water's table starts one word later than lava/acid's.
        let index = usize::from(row >> 1) + usize::from(matches!(wave, Wave::Water));
        game.store_source_word(
            LiveOwner::LiquidBackground,
            2 * index,
            WordOrder::Little,
            value,
        );
        row = row.wrapping_sub(2) & 0x1e;
        sample = sample.wrapping_sub(2) & 0x1e;
    }
}
pub fn lava_background<H: crate::menus::host::Host + ?Sized>(hdma_index: usize, game: &mut H) {
    game.store_source_word(
        LiveOwner::LiquidSurface,
        0x44,
        WordOrder::Little,
        crate::menus::prelude::display_scroll!(game.Bg2V),
    );
    let active = crate::sequence_word!(game.TIME_IS_FROZEN_FLAG) == 0
        && crate::liquid::LiquidWords::read(game).options & 6 != 0;
    let horizontal = active && crate::liquid::LiquidWords::read(game).options & 2 == 0;
    game.write_register(
        (crate::hdma_word!(game, hdma_index, BankSlot) & 0xff).wrapping_add(BBAD0 as u16),
        if horizontal { 0xf } else { 0x10 },
    );
    if active {
        fill_wave(
            hdma_index,
            if horizontal {
                Wave::LavaHorizontal
            } else {
                Wave::LavaVertical
            },
            game,
        );
    } else {
        let value = crate::menus::prelude::display_scroll!(game.Bg2V) & 0x1ff;
        for index in (0..16).rev() {
            game.store_source_word(
                LiveOwner::LiquidBackground,
                2 * index,
                WordOrder::Little,
                value,
            );
        }
    }
    crate::hdma_word!(game, hdma_index, Table => 3 * (crate::menus::prelude::display_scroll!(game.Bg2V) & 0xf) + 0xc0b1);
}
pub fn water_background<H: crate::menus::host::Host + ?Sized>(hdma_index: usize, game: &mut H) {
    game.store_source_word(
        LiveOwner::LiquidSurface,
        0x44,
        WordOrder::Little,
        crate::menus::prelude::display_scroll!(game.Bg2H),
    );
    if crate::sequence_word!(game.TIME_IS_FROZEN_FLAG) != 0 {
        water_flat(game);
    } else if crate::liquid::LiquidWords::read(game).options & 2 == 0 {
        crate::hdma_word!(game, hdma_index, Channels => 0);
        water_flat(game);
    } else {
        water_wave(hdma_index, game);
    }
    let height = table_height(
        crate::liquid::LiquidWords::read(game).liquid.surface.whole,
        crate::camera::current(game).layer1[1],
        true,
    );
    crate::hdma_word!(game, hdma_index, Table => 3 * ((height ^ 0x1ff) & 0x3ff) + 0xcf46);
}
pub fn water_wave<H: crate::menus::host::Host + ?Sized>(hdma_index: usize, game: &mut H) {
    fill_wave(hdma_index, Wave::Water, game);
}
pub fn water_flat<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    let value = crate::menus::prelude::display_scroll!(game.Bg2H);
    // The native fill starts at the last water-wave entry and descends.
    for index in 0..16 {
        game.store_source_word(
            LiveOwner::LiquidSurface,
            0x66 - 2 * index,
            native_order(),
            value,
        );
    }
}
