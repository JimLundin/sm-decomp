//! Cinematic palettes, fades, tilemap queueing and sprite resumption over RAM.
#![forbid(unsafe_code)]
// The original's names and assignment shapes are kept.
#![allow(non_snake_case, non_upper_case_globals, unused_mut, unused_assignments)]
#[allow(unused_imports)]
use crate::ram::RawRam as _;
use crate::ram::word;

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
            _ => word!(game.cinematicbg_var3),
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
