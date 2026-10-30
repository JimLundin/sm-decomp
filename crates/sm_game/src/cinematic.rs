//! Cinematic palettes, fades, tilemap queueing and sprite resumption over RAM.
#![forbid(unsafe_code)]
use crate::ram::RawRam as _;
use crate::ram::word;

/// A bank-`BANK` cinematic record at `cursor`, after the low-half diagnostic.
pub fn cinematic_native_record<const BANK: u8>(
    cursor: u16,
    game: &mut (impl crate::menus::host::Host + ?Sized),
) -> sm_data::cartridge::RomRecord {
    crate::menus::menu_tilemap::native_record((u32::from(BANK) << 16) | u32::from(cursor), game)
}

pub fn advance_cinematic_fade<H: crate::menus::host::Host + ?Sized>(
    fade: crate::visual_state::CinematicFade,
    game: &mut H,
) -> bool {
    use crate::visual_state::CinematicFade::*;
    let slow = matches!(fade, SlowIn | SlowOut);
    if slow {
        // Only the low delay byte decrements, with the signed-byte wrap retained.
        game.ram()
            .write_byte(0x723, (word!(game.screen_fade_delay) as u8).wrapping_sub(1));
        if (game.ram().read_byte(0x723) as i8) > 0 {
            return false;
        }
    }
    let increasing = matches!(fade, FastIn | SlowIn);
    let brightness = game.ram().read_byte(0x51);
    if !increasing && brightness & 15 == 0 {
        return true;
    }
    let step = if slow {
        1
    } else {
        word!(game.screen_fade_counter) as u8
    };
    let (next, complete) = if increasing {
        let next = brightness.wrapping_add(step) & 31;
        (next.min(15), next >= 15)
    } else {
        let next = (brightness & 15).wrapping_sub(step);
        let complete = (next as i8) <= 0;
        (if complete { 0x80 } else { next }, complete)
    };
    game.ram().write_byte(0x51, next);
    if slow {
        game.ram().write_byte(
            0x723,
            if complete {
                0
            } else {
                word!(game.screen_fade_counter) as u8
            },
        );
        if complete {
            game.ram().write_byte(0x725, 0);
        }
    }
    complete
}

pub fn clear_cinematic_colors<H: crate::menus::host::Host + ?Sized>(
    offset: u16,
    count: u16,
    game: &mut H,
) {
    for index in cinematic_color_indices(offset, count) {
        for plane in [256, 512, 768, 1024] {
            word!(game.tilemap_stuff[index + plane] = 0);
        }
    }
}

pub fn compose_cinematic_palette<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    for index in 0..256 {
        // Red uses the physical high byte, independently of native word endianness.
        let red = u16::from(game.ram().read_byte(0x2401 + 2 * index) & 31);
        let green = (word!(game.tilemap_stuff[index + 768]) >> 3) & 0x3e0;
        let blue = (word!(game.tilemap_stuff[index + 1024]) << 2) & 0x7c00;
        let color = red | green | blue;
        word!(game.tilemap_stuff[index] = color);
        word!(game.palette_buffer[index] = color);
    }
}

pub fn fade_cinematic_colors<H: crate::menus::host::Host + ?Sized>(
    offset: u16,
    count: u16,
    increasing: bool,
    game: &mut H,
) {
    for index in cinematic_color_indices(offset, count) {
        for plane in [512, 768, 1024] {
            let current = index + plane;
            if increasing {
                let step = word!(game.tilemap_stuff[current + 768]);
                word!(
                    game.tilemap_stuff[current] =
                        step.wrapping_add(word!(game.tilemap_stuff[current])) & 0x1fff
                );
            } else {
                word!(game.tilemap_stuff[current] -= word!(game.tilemap_stuff[current + 768]));
            }
        }
    }
}

pub fn prepare_cinematic_palette_fade<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    for index in 0..256 {
        let color = word!(game.tilemap_stuff[index + 256]);
        for channel in 0..3 {
            let component = (color >> (5 * channel)) & 31;
            let current = index + 512 + 256 * channel;
            word!(game.tilemap_stuff[current] = component << 8);
            word!(game.tilemap_stuff[current + 768] = component << 3);
        }
    }
}

pub fn queue_cinematic_tilemap<H: crate::menus::host::Host + ?Sized>(
    tiles: crate::visual_state::CinematicTilemap,
    game: &mut H,
) {
    use crate::visual_state::CinematicTilemap::*;
    if !matches!(tiles, Japanese | JapaneseIntro) && game.ram().read_byte(0x55) == 7 {
        return;
    }
    let (size, source) = match tiles {
        Text => (1920, 0x3000),
        FullText => (2048, 0x3000),
        Eyes => (128, 0x3b40),
        Japanese => (1536, 0x4000),
        JapaneseIntro => (256, 0x3000),
    };
    game.queue_native_transfer(
        word!(game.vram_write_queue_tail),
        size,
        &|_| source,
        126,
        // Earlier queue writes can overlap this destination field.
        &|game| match tiles {
            Eyes => 18848,
            Japanese => 16768,
            JapaneseIntro => 20192,
            _ => word!(game.cinematic_bg_vram_address),
        },
    );
}

pub fn set_cinematic_blend<H: crate::menus::host::Host + ?Sized>(
    main_screen: u8,
    sub_screen: u8,
    color_window: u8,
    color_math: u8,
    game: &mut H,
) {
    game.ram().write_byte(0x69, main_screen);
    game.ram().write_byte(0x6b, sub_screen);
    game.ram().write_byte(0x6f, color_window);
    game.ram().write_byte(0x72, color_math);
}

pub fn set_cinematic_tilemap_updates<H: crate::menus::host::Host + ?Sized>(
    enabled: bool,
    game: &mut H,
) {
    word!(game.cinematic_enable_bg_tilemap, |old| (old & 0x7fff)
        | (u16::from(enabled) << 15));
}

#[forbid(unsafe_code)]
pub fn resume_cinematic_sprite<H: crate::menus::host::Host + ?Sized>(
    slot: usize,
    instruction: u16,
    game: &mut H,
) {
    word!(game.cinematicspr_instr_timer[slot] = 1);
    word!(game.cinematicspr_instr_ptr[slot] = instruction);
}

pub fn cinematic_color_indices(mut offset: u16, count: u16) -> impl Iterator<Item = usize> {
    // The do-while count admits zero (65,536 entries) and wrapping odd byte offsets.
    (0..=count.wrapping_sub(1)).map(move |_| {
        let index = usize::from(offset >> 1);
        offset = offset.wrapping_add(2);
        index
    })
}

// Native split words retain low-before-high reads and writes.
pub fn add_native_pair<H: crate::menus::host::Host + ?Sized>(
    high: usize,
    low: usize,
    velocity: u32,
    game: &mut H,
) {
    let fraction = u16::from_ne_bytes([game.ram().read_byte(low), game.ram().read_byte(low + 1)]);
    let whole = u16::from_ne_bytes([game.ram().read_byte(high), game.ram().read_byte(high + 1)]);
    let position = (u32::from(fraction) | (u32::from(whole) << 16)).wrapping_add(velocity);
    crate::word_transfers!(game, ascending, 1;
        native(low) <- value(position as u16),
        native(high) <- value((position >> 16) as u16),
    );
}

pub fn advance_cinematic_coordinate<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    vertical: bool,
    velocity: u32,
    game: &mut H,
) {
    let offset = usize::from(object & !1) + if vertical { 0x20 } else { 0 };
    add_native_pair(0x1a7d + offset, 0x1add + offset, velocity, game);
}

pub fn set_fixed_color<H: crate::menus::host::Host + ?Sized>(channels: [u8; 3], game: &mut H) {
    for (channel, value) in channels.into_iter().enumerate() {
        game.ram().write_byte(0x74 + channel, value);
    }
}

pub fn set_cinematic_backdrop<H: crate::menus::host::Host + ?Sized>(lit: bool, game: &mut H) {
    game.ram().write_byte(0x6f, 0);
    game.ram().write_byte(0x72, if lit { 0x81 } else { 0 });
    let color = if lit { [56, 88, 0x80] } else { [32, 64, 0x80] };
    set_fixed_color(color, game);
}

pub fn place_cinematic_sprite<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    position: (u16, u16),
    palette: u16,
    game: &mut H,
) {
    let slot = usize::from(object >> 1);
    word!(game.cinematicbg_arr7[slot] = position.0);
    word!(game.cinematicbg_arr8[slot] = position.1);
    word!(game.cinematicbg_arr9[slot] = palette);
}

pub fn drift_cinematic_sprite<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    horizontal: u32,
    vertical: Option<u32>,
    wrap_horizontal: bool,
    game: &mut H,
) {
    if let Some(velocity) = vertical {
        advance_cinematic_coordinate(object, true, velocity, game);
    }
    advance_cinematic_coordinate(object, false, horizontal, game);
    if wrap_horizontal {
        word!(game.cinematicbg_arr7[(object >> 1)] &= 0x1ff);
    }
}

pub fn capture_cinematic_palette<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    crate::word_transfers!(game, ascending, 256; native(0x2200) <- le(palette_buffer, 0));
}

pub fn fade_cinematic_fixed_color<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    for (lane, black) in [0x20u8, 0x40, 0x80].into_iter().enumerate() {
        let at = 0x74 + lane;
        // Keep decrement and clamp as separate stores in channel order.
        game.ram()
            .write_byte(at, game.ram().read_byte(at).wrapping_sub(1));
        if game.ram().read_byte(at).wrapping_sub(black) & 0x80 != 0 {
            game.ram().write_byte(at, black);
        }
    }
}

pub fn enable_cinematic_background<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    word!(game.cinematic_enable_objs |= 0x8000);
}

pub fn clear_cinematic_sprites<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    crate::word_transfers!(game, descending, 16;
        le(cinematicspr_instr_ptr, 15) <- value(0),
        le(cinematicspr_whattodraw, 15) <- value(0),
    );
}

pub fn clear_cinematic_background<H: crate::menus::host::Host + ?Sized>(fill: u16, game: &mut H) {
    crate::word_transfers!(game, descending, 1024; native(0x37fe) <- value(fill));
    crate::word_transfers!(game, descending, 4;
        le(cinematicbg_arr1, 3) <- value(0),
        le(cinematicbg_instr_ptr, 3) <- value(0),
    );
    word!(game.intro_text_click_flag = 0);
}

pub fn scaled_cinematic_value(value: u16, scale: u16) -> u16 {
    ((i32::from(value as i16) * i32::from(scale as i16)) >> 8) as u16
}

/// Project `point` through the cinematic Mode 7 matrix and centre in RAM.
pub fn project_cinematic_point<H: crate::ram::WorkRamAccess + ?Sized>(
    point: crate::geometry::Point,
    game: &H,
) -> crate::geometry::Point {
    use crate::ram::RawRam as _;
    project_cinematic_point_reads(point, |at| {
        u16::from_le_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)])
    })
}

/// Project `point` with the matrix and centre words `read` returns by RAM address.
pub fn project_cinematic_point_reads(
    point: crate::geometry::Point,
    mut read: impl FnMut(usize) -> u16,
) -> crate::geometry::Point {
    let x = point.x.wrapping_sub(read(0x80));
    let y = read(0x82).wrapping_sub(point.y);
    let projected_x =
        scaled_cinematic_value(x, read(0x78)).wrapping_add(scaled_cinematic_value(read(0x7a), y));
    let projected_y =
        scaled_cinematic_value(read(0x7c), x).wrapping_add(scaled_cinematic_value(read(0x78), y));
    crate::geometry::Point {
        x: projected_x.wrapping_add(read(0x80)),
        y: read(0x82).wrapping_sub(projected_y),
    }
}
