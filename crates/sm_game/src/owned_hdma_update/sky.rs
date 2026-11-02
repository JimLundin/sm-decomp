//! Horizontal and parallax sky callbacks on the current owner.
#![forbid(unsafe_code)]
use super::context::Context;
use super::liquid::WATER_WAVE;
use crate::display::preparation::{Field as DisplayField, Store as DisplayStore};

use crate::fx_domain::native_order;
use crate::menus::prelude::do_until;
use crate::scene::hdma_sources::LiveOwner;
use crate::scene::state::WordOrder;

fn native_sky_record<H: Context>(address: u32, host: &mut H) -> sm_data::cartridge::RomRecord {
    crate::menus::menu_tilemap::native_record(address, host)
}
pub fn update_horizontal_scroll<H: Context>(hdma_index: usize, game: &mut H) {
    let layer1_y = game.session().room.camera.layer1[1];
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

fn append_upper_band<H: Context>(k: u16, r18: u16, r20: u16, r22: u16, game: &mut H) -> u16 {
    let layer1_y = game.session().room.camera.layer1[1];
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

fn write_sky_word<H: Context>(value: u16, offset: isize, game: &mut H) {
    game.store_source_signed_word(LiveOwner::SkyIndirect, offset, native_order(), value);
}

pub fn advance_parallax<H: Context>(game: &mut H) {
    game.store_display_control_byte(DisplayField::Screens1, 74, DisplayStore::Named);
    for v1 in 0u16..23 {
        let entry = native_sky_record(0x88aec1 + 8 * u32::from(v1), game);
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
    let layer1_y = game.session().room.camera.layer1[1];
    let mut r18: u16 = layer1_y.wrapping_add(32);
    let r20: u16 = layer1_y.wrapping_add(224);
    let mut v6: u16 = 3;
    for v5 in 0u16..23 {
        let entry = native_sky_record(0x88aec1 + 8 * u32::from(v5), game);
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

pub fn update_parallax<H: Context>(game: &mut H) {
    if game.session().timing.time_frozen == 0 {
        advance_parallax(game);
    }
}

fn write_scroll_pair<H: Context>(offset: usize, value: u16, game: &mut H) {
    game.store_source_word(LiveOwner::WindowLeft, offset, native_order(), value);
    write_sky_word(value, offset as isize, game);
}

fn room_sky_table_word<H: Context>(host: &H, address: u32, band: usize) -> u16 {
    // The original samples the native allocation anew for each table query.
    crate::cartridge::NativeImage::native_image(host).table_word(
        usize::try_from(sm_data::cartridge::lorom_offset(address)).expect("sky table"),
        isize::try_from(band * 3).expect("sky band"),
    )
}
pub(super) fn vertical_scroll<H: Context>(host: &mut H) {
    let y = host.session().player.y;
    let radius = host.session().player.contact.radius;
    if y.wrapping_sub(radius) & 0x8000 != 0 || y.wrapping_sub(radius).wrapping_sub(17) & 0x8000 != 0
    {
        host.session_mut().player.periodic_damage = 8;
    }
    let row = room_build_sky_bands(host);
    room_source_native_word(host, LiveOwner::LiquidSurface, usize::from(row), 0);
}
fn room_build_sky_bands<H: Context>(host: &mut H) -> u16 {
    room_fx_word(host, crate::fx_state::Field::SkyScreenRow, 32);
    room_source_native_word(host, LiveOwner::LiquidSurface, 0, 31);
    room_source_native_word(host, LiveOwner::LiquidSurface, 1, 0);
    let mut row = 3u16;
    let mut y = host.session().room.camera.layer1[1].wrapping_add(32);
    let mut band = 0usize;
    let mut inset = y & if y.wrapping_sub(1248) & 0x8000 != 0 {
        15
    } else {
        31
    };
    loop {
        while y.wrapping_sub(room_sky_table_word(host, 0x88a8e8, band)) & 0x8000 != 0
            || y.wrapping_sub(room_sky_table_word(host, 0x88a8ee, band)) & 0x8000 == 0
        {
            band += 1;
            if band >= 13 {
                return row;
            }
        }
        let height = room_sky_table_word(host, 0x88a8ea, band).wrapping_sub(inset);
        room_source_native_word(host, LiveOwner::LiquidSurface, usize::from(row), height);
        let scroll = room_sky_table_word(host, 0x88a8ec, band)
            .wrapping_add(inset)
            .wrapping_sub(host.fx_controls().sky_screen_row);
        room_source_native_word(host, LiveOwner::LiquidSurface, usize::from(row) + 1, scroll);
        let height = u16::from(host.source_byte(LiveOwner::LiquidSurface, usize::from(row)));
        y = y.wrapping_add(height);
        row = row.wrapping_add(3);
        let screen_row = host.fx_controls().sky_screen_row.wrapping_add(height);
        room_fx_word(host, crate::fx_state::Field::SkyScreenRow, screen_row);
        if host.fx_controls().sky_screen_row.wrapping_sub(224) & 0x8000 == 0 {
            return row;
        }
        inset = 0;
    }
}

fn room_source_native_word<H: Context>(host: &mut H, source: LiveOwner, offset: usize, value: u16) {
    let next = offset.checked_add(1).expect("room source extent");
    let [first, second] = value.to_ne_bytes();
    host.store_source_byte(source, offset, first);
    host.store_source_byte(source, next, second);
    host.refresh_inputs();
}
fn room_fx_word<H: Context>(host: &mut H, field: crate::fx_state::Field, value: u16) {
    crate::fx_encoding::write_control_word(
        host.session_mut(),
        crate::fx_encoding::address(field),
        value,
    );
    host.refresh_inputs();
}
