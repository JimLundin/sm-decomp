//! Scrolling sky setup, parallax, HDMA bands and tile uploads.
use crate::display::preparation::{Field as DisplayField, Store as DisplayStore};
use crate::fx_domain::native_order;
use crate::liquid_effects::WATER_WAVE;
use crate::menus::menu_tilemap::native_record;
use crate::menus::prelude::*;
use crate::scene::hdma_sources::LiveOwner;
use crate::scene::state::WordOrder;

pub fn initialize_banded_sky<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    crate::liquid::LiquidWords::set(game, |words| &mut words.liquid.surface.whole, 1248);
    crate::liquid::LiquidWords::set(game, |words| &mut words.liquid.kind, 6);
    static UNK_88A62E: HdmaObjectDefinition = HdmaObjectDefinition::new(0x2, 0x12, 0xad63 as VoidP);
    static UNK_88A636: HdmaObjectDefinition =
        HdmaObjectDefinition::new(0x42, 0x11, 0xad4e as VoidP);
    static UNK_88A63E: HdmaObjectDefinition = HdmaObjectDefinition::new(0x42, 0xd, 0xad39 as VoidP);
    game.spawn_hdma(0x88, &UNK_88A62E);
    game.spawn_hdma(0x88, &UNK_88A636);
    game.spawn_hdma(0x88, &UNK_88A63E);
    update_vertical_scroll(game);
}
fn sky_table_word<H: crate::menus::host::Host + ?Sized>(
    address: u32,
    index: isize,
    game: &H,
) -> u16 {
    game.native_image()
        .table_word(sm_data::cartridge::lorom_offset(address) as usize, index)
}
fn build_vertical_bands<H: crate::menus::host::Host + ?Sized>(game: &mut H) -> u16 {
    {
        let fx_value = 32;
        let mut fx = game.fx_controls();
        fx.sky_screen_row = fx_value;
        game.publish_fx_controls(fx, &[crate::fx_state::Field::SkyScreenRow]);
    };
    game.store_source_word(LiveOwner::LiquidSurface, 0, native_order(), 31);
    game.store_source_word(LiveOwner::LiquidSurface, 1, native_order(), 0);
    let mut row = 3u16;
    let mut y = crate::camera::current(game).layer1[1].wrapping_add(32);
    let mut band = 0usize;
    let mut inset = y & if y.wrapping_sub(1248) & 0x8000 != 0 {
        15
    } else {
        31
    };
    loop {
        while (y as i16).wrapping_sub(sky_table_word(0x88a8e8, (band * 3) as isize, game) as i16)
            < 0
            || (y as i16).wrapping_sub(sky_table_word(0x88a8ee, (band * 3) as isize, game) as i16)
                >= 0
        {
            band += 1;
            if band >= 13 {
                return row;
            }
        }
        let height_word = sky_table_word(0x88a8ea, (band * 3) as isize, game).wrapping_sub(inset);
        game.store_source_word(
            LiveOwner::LiquidSurface,
            row as usize,
            native_order(),
            height_word,
        );
        let scroll_word = sky_table_word(0x88a8ec, (band * 3) as isize, game)
            .wrapping_add(inset)
            .wrapping_sub(game.fx_controls().sky_screen_row);
        game.store_source_word(
            LiveOwner::LiquidSurface,
            1 + row as usize,
            native_order(),
            scroll_word,
        );
        let height = u16::from(game.source_byte(LiveOwner::LiquidSurface, usize::from(row)));
        y = y.wrapping_add(height);
        row = row.wrapping_add(3);
        {
            let fx_value = game.fx_controls().sky_screen_row.wrapping_add(height);
            let mut fx = game.fx_controls();
            fx.sky_screen_row = fx_value;
            game.publish_fx_controls(fx, &[crate::fx_state::Field::SkyScreenRow]);
        };
        if game.fx_controls().sky_screen_row.wrapping_sub(224) & 0x8000 == 0 {
            return row;
        }
        inset = 0;
    }
}
pub fn update_vertical_scroll<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    damage_above_screen(game);
    let row = build_vertical_bands(game);
    game.store_source_word(LiveOwner::LiquidSurface, row as usize, native_order(), 0);
}
pub fn update_horizontal_scroll<H: crate::menus::host::Host + ?Sized>(
    hdma_index: usize,
    game: &mut H,
) {
    let layer1_y = crate::camera::current(game).layer1[1];
    if (layer1_y as i32 - 1024i32) & 0x8000 != 0 {
        if (game.display_control_byte(DisplayField::NmiFrameByte) & 1) == 0 {
            {
                let fx_value = ((game.fx_controls().sky_wave_phase as i32 + 2) & 0x1e) as u16;
                let mut fx = game.fx_controls();
                fx.sky_wave_phase = fx_value;
                game.publish_fx_controls(fx, &[crate::fx_state::Field::SkyWavePhase]);
            };
        }
        let mut v1: u16 = game.fx_controls().sky_wave_phase;
        let mut v2: u16 = 0;
        do_until! { ((v1) as i16).wrapping_sub(32) >= 0 ;
            let value = crate::gameplay_tables::sky_scroll_wave(game, isize::try_from(v1 >> 1).expect("sky wave index"));
            game.store_source_word(LiveOwner::SkyHorizontal, usize::from(v2), WordOrder::Little, value);
            v2 = (v2).wrapping_add(2);
            v1 = (v1).wrapping_add(2);
        }
    } else {
        let timer = crate::hdma_word!(game, hdma_index, B);
        crate::hdma_word!(game, hdma_index, B => timer.wrapping_sub(1));
        if timer == 1 {
            crate::hdma_word!(game, hdma_index, B => 6);
            crate::hdma_word!(game, hdma_index, A => ((crate::hdma_word!(game, hdma_index, A) as i32 + 2) & 0x1f) as u16);
        }
        let mut v5: u16 = crate::hdma_word!(game, hdma_index, A);
        for i in (0i32..=30).rev().step_by(2) {
            let value = (WATER_WAVE[(i >> 1) as usize] as u16)
                .wrapping_add(crate::menus::prelude::display_scroll!(game.Bg1H));
            game.store_source_word(
                LiveOwner::SkyHorizontal,
                2 * usize::from(v5 >> 1),
                WordOrder::Little,
                value,
            );
            v5 = ((v5 as i32 - 2i32) & 0x1f) as u16;
        }
    }
    write_scroll_pair(0, 0, game);
    write_scroll_pair(2, 31, game);
    let mut r18: u16 = 31;
    write_scroll_pair(3, -25088i32 as u16, game);
    if (layer1_y as i32 - 1024i32) & 0x8000 != 0 {
        write_sky_word(0, 2, game);
    } else {
        r18 = append_upper_band(0x105, r18, 177, 0, game);
    }
    let mut v7: u16 = 5;
    r18 = append_upper_band(5, r18, -24960i32 as u16, 128, game);
    let mut v8: i16 = (224i16).wrapping_sub((r18) as i16);
    while (v8).wrapping_sub(16) >= 0 {
        v8 = (v8).wrapping_sub(16);
        write_scroll_pair(v7 as usize, 144, game);
        write_scroll_pair(v7 as usize + 1, -24960i32 as u16, game);
        v7 = (v7).wrapping_add(3);
    }
    let mut v9: i16 = v8;
    if v8 != 0 {
        v9 = (v8).wrapping_add(128);
    }
    write_scroll_pair(v7 as usize, v9 as u16, game);
    write_scroll_pair(v7 as usize + 1, -24960i32 as u16, game);
    write_scroll_pair(v7 as usize + 3, 0, game);
}
fn append_upper_band<H: crate::menus::host::Host + ?Sized>(
    k: u16,
    r18: u16,
    r20: u16,
    r22: u16,
    game: &mut H,
) -> u16 {
    let layer1_y = crate::camera::current(game).layer1[1];
    let visible = 1216u16.wrapping_sub(layer1_y);
    if (1216i16).wrapping_sub(layer1_y as i16) < 0 || layer1_y == 1216 {
        return r18;
    }
    let mut v0: u16 = k;
    let mut r24 = visible;
    let height = if (visible as i32 - 128i32) & 0x8000 != 0 {
        visible
    } else {
        let height = if (visible as i32 - 193i32) & 0x8000 == 0 {
            193
        } else {
            visible
        };
        r24 = height;
        loop {
            let v2: i16 = (r24 as i16).wrapping_sub(16);
            if v2 < 0 {
                break;
            }
            r24 = v2 as u16;
            game.store_source_word(
                LiveOwner::WindowLeft,
                v0 as usize,
                native_order(),
                r22 | 0x10,
            );
            game.store_source_word(LiveOwner::WindowLeft, 1 + v0 as usize, native_order(), r20);
            v0 = v0.wrapping_add(3);
        }
        height
    };
    game.store_source_word(
        LiveOwner::WindowLeft,
        v0 as usize,
        native_order(),
        r22 | r24,
    );
    game.store_source_word(LiveOwner::WindowLeft, 1 + v0 as usize, native_order(), r20);
    height
}
static LAND_SCROLL: HdmaObjectDefinition = HdmaObjectDefinition::new(0x42, 0xf, 0xad76 as VoidP);
static OCEAN_SCROLL: HdmaObjectDefinition = HdmaObjectDefinition::new(0x42, 0xf, 0xad89 as VoidP);
pub fn initialize_land<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    game.ram()
        .write_byte(0x91b, game.ram().read_byte(0x91b) | 1);
    game.ram()
        .write_byte(0x91c, game.ram().read_byte(0x91c) | 1);
    game.spawn_hdma(0x88, &LAND_SCROLL);
    let mut fx = game.fx_controls();
    fx.sky_visible_height = 224;
    fx.sky_compatibility_word = 0;
    game.publish_fx_controls(
        fx,
        &[
            crate::fx_state::Field::SkyVisibleHeight,
            crate::fx_state::Field::SkyCompatibilityWord,
        ],
    );
}
pub fn initialize_ocean<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    game.store_display_control_byte(DisplayField::Screens1, 74, DisplayStore::Named);
    game.spawn_hdma(0x88, &OCEAN_SCROLL);
    let mut fx = game.fx_controls();
    fx.sky_visible_height = 224;
    fx.sky_compatibility_word = 0;
    game.publish_fx_controls(
        fx,
        &[
            crate::fx_state::Field::SkyVisibleHeight,
            crate::fx_state::Field::SkyCompatibilityWord,
        ],
    );
}
fn damage_above_screen<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    if ((game.player_y()) as i16).wrapping_sub((game.player_y_radius()) as i16) < 0
        || (game.player_y() as i32 - game.player_y_radius() as i32 - 17i32) & 0x8000 != 0
    {
        game.set_periodic_damage(8);
    }
}
fn write_sky_word<H: crate::fx_domain::Sources + ?Sized>(value: u16, offset: isize, game: &mut H) {
    game.store_source_signed_word(LiveOwner::SkyIndirect, offset, native_order(), value);
}

/// Advance the scrolling sky's parallax, on the lease the HDMA object opens.
pub fn advance_parallax<
    L: crate::actors::boss_lease::BossState
        + crate::fx_domain::Sources
        + crate::display::preparation::Access
        + ?Sized,
>(
    game: &mut L,
) {
    game.store_display_control_byte(DisplayField::Screens1, 74, DisplayStore::Named);
    for v1 in 0u16..23 {
        let entry = native_record(0x88aec1 + 8 * u32::from(v1), game);
        let selector = entry.word(6);
        let high = entry.word(4);
        let low = entry.word(2);
        let delta = (u32::from(high) << 16) | u32::from(low);
        game.add_parallax_source(selector, delta);
    }
    game.store_source_word(LiveOwner::SkyBg2, 88, WordOrder::Little, 0);
    game.store_source_word(LiveOwner::SkyBg2, 90, WordOrder::Little, 0);
    write_sky_word(31, 0, game);
    write_sky_word(1438, 1, game);
    let layer1_y = game.camera().layer1[1];
    let mut r18: u16 = layer1_y.wrapping_add(32);
    let r20: u16 = layer1_y.wrapping_add(224);
    let mut v6: u16 = 3;
    for v5 in 0u16..23 {
        let entry = native_record(0x88aec1 + 8 * u32::from(v5), game);
        while ((r18) as i16).wrapping_sub(entry.word(0) as i16) >= 0
            && ((r18) as i16).wrapping_sub(entry.word(8) as i16) < 0
        {
            let height = entry.word(8).wrapping_sub(r18);
            let mut v8 = height;
            if (v8 as i32 - 128i32) & 0x8000 == 0 {
                write_sky_word(127, v6 as isize, game);
                write_sky_word(entry.word(6).wrapping_add(2), v6 as isize + 1, game);
                v6 = (v6).wrapping_add(3);
                v8 = (height).wrapping_sub(127);
            }
            write_sky_word(v8, v6 as isize, game);
            write_sky_word(entry.word(6).wrapping_add(2), v6 as isize + 1, game);
            r18 = (r18).wrapping_add(height);
            v6 = (v6).wrapping_add(3);
            if (r18 as i32 - r20 as i32) & 0x8000 == 0 {
                write_sky_word(0, v6 as isize + 3, game);
                return;
            }
        }
    }
    let mut i: u16 = (1535u16).wrapping_sub(r18);
    loop {
        if (i as i32 - 128i32) & 0x8000 != 0 {
            break;
        }
        write_sky_word(127, v6 as isize, game);
        write_sky_word(181, v6 as isize + 1, game);
        v6 = (v6).wrapping_add(3);
        i = (i).wrapping_sub(127);
    }
    write_sky_word(i, v6 as isize, game);
    write_sky_word(181, v6 as isize + 1, game);
    write_sky_word(0, v6 as isize + 3, game);
}
/// Advance the parallax unless time is frozen.
pub fn update_parallax<
    L: crate::actors::boss_lease::BossState
        + crate::fx_domain::Sources
        + crate::display::preparation::Access
        + ?Sized,
>(
    game: &mut L,
) {
    if crate::sequence_word!(game.TIME_IS_FROZEN_FLAG) == 0 {
        advance_parallax(game);
    }
}
fn queue_tile_rows<H: crate::menus::host::Host + ?Sized>(address: u32, game: &mut H) {
    // Capture the original native source even when frozen; reads remain lazy.
    let src = game.native_image();
    let base = sm_data::cartridge::lorom_offset(address) as usize;
    if crate::sequence_word!(game.TIME_IS_FROZEN_FLAG) != 0 {
        write_sky_word(0, 0, game);
    } else {
        let layer1_y = crate::camera::current(game).layer1[1];
        crate::menus::prelude::display_scroll!(game.Bg2V = layer1_y);
        // Four queue entries: two rows above the screen and two below it.
        let tail = game.queue_tail();
        let stride = u16::try_from(VRAM_WRITE_ENTRY_BYTES).expect("queue entry size");
        let entry = |row: u16, field: u16| tail.wrapping_add(row * stride).wrapping_add(field);
        for row in 0..4 {
            game.queue_native_word(entry(row, 0), 64);
        }
        // Each row indexes the table by its own high byte ($88:AFC5/$AFE8), so near the top of
        // the room the upper row reads past the table and the lower row reads its entry 0.
        let source = |row: u16| {
            src.table_word(base, isize::from(row.to_be_bytes()[0]))
                .wrapping_add((row & 0xff) * 8)
        };
        let upper: VoidP = source((layer1_y & 0x7f8).wrapping_sub(16));
        game.queue_native_word(entry(0, 2), upper);
        game.queue_native_word(entry(1, 2), upper.wrapping_add(64));
        let lower: VoidP = source((layer1_y & 0x7f8) + 0xf0);
        game.queue_native_word(entry(2, 2), lower);
        game.queue_native_word(entry(3, 2), lower.wrapping_add(64));
        let queue = 0xd0 + usize::from(tail);
        for row in 0..4 {
            game.ram()
                .write_byte(queue + row * VRAM_WRITE_ENTRY_BYTES + 4, 0x8a);
        }
        let t: i32 = (game.display_control_byte(DisplayField::Screens1) as i32 & 0xfc) << 8;
        let upper_destination: u16 = (t + 4 * ((layer1_y as i32 - 16i32) & 0x1f8)) as u16;
        game.queue_native_word(entry(0, 5), upper_destination);
        game.queue_native_word(entry(1, 5), upper_destination.wrapping_add(32));
        let lower_destination: u16 = (t + 4 * ((layer1_y as i32 + 240) & 0x1f8)) as u16;
        game.queue_native_word(entry(2, 5), lower_destination);
        game.queue_native_word(entry(3, 5), lower_destination.wrapping_add(32));
        game.store_queue_tail(tail.wrapping_add(28));
    };
}
pub fn queue_land_rows<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    queue_tile_rows(0x88ad9c, game);
}
pub fn queue_ocean_rows<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    queue_tile_rows(0x88ada6, game);
}
// HDMA entries overlap word stores at byte offsets; retain channel write order.
fn write_scroll_pair<H: crate::menus::host::Host + ?Sized>(
    offset: usize,
    value: u16,
    game: &mut H,
) {
    game.store_source_word(LiveOwner::WindowLeft, offset, native_order(), value);
    write_sky_word(value, offset as isize, game);
}
