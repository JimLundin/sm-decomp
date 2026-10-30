//! Intro and Zebes cinematic objects: Rinkas, the baby, explosions, flights, scenes and transforms.
use crate::cinematic::*;
use crate::cinematic_objects::CinematicObjects;
use crate::ram::RawRam as _;
use crate::ram::{word, words as ram_words};
use crate::visual_state::{CeresExplosion, CinematicFade};

const RINKA_POSITIONS: [(u16, u16); 4] = [(112, 72), (192, 56), (128, 48), (232, 80)];

const RINKA_X_SPEED: [u32; 4] = [0x8000, 0xffff8000, 0x8000, 0xffff8000];

pub fn initialize_intro_rinka<H: crate::menus::host::Host + ?Sized>(object: u16, game: &mut H) {
    let variant = usize::from(word!(game.cinematic_spawn_param));
    let slot = usize::from(object >> 1);
    word!(game.cinematicspr_goto_timer[slot] = word!(game.cinematic_spawn_param));
    place_cinematic_sprite(object, RINKA_POSITIONS[variant], 3584, game);
}

pub fn start_intro_rinka<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    cursor: u16,
    game: &mut H,
) -> u16 {
    let misses_samus = word!(game.cinematicspr_goto_timer[(object >> 1)]) != 0;
    word!(
        game.cinematicspr_preinstr_func[(object >> 1)] = if misses_samus { 0xb93b } else { 0xb8d8 }
    );
    cursor
}

pub fn chase_intro_baby_y<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    y_offset: i32,
    game: &mut H,
) {
    let slot = usize::from(object >> 1);
    if ((game.player_y() as i32 - y_offset - word!(game.cinematicbg_arr8[slot]) as i32) as i16) < 0
    {
        if (word!(game.cinematic_var17) as i32 + 543) & 0x8000 == 0 {
            word!(game.cinematic_var17, wrapping_sub(32));
        }
    } else if (word!(game.cinematic_var17) as i32 - 544) & 0x8000 != 0 {
        word!(game.cinematic_var17, wrapping_add(32));
    }
    let velocity = ((word!(game.cinematic_var17) as i16 as i32) << 8) as u32;
    advance_cinematic_coordinate(object, true, velocity, game);
}

pub fn advance_intro_baby_pursuit<H: crate::menus::host::Host + ?Sized>(object: u16, game: &mut H) {
    let slot = usize::from(object >> 1);
    if word!(game.cinematic_var13) == 0 {
        resume_cinematic_sprite(slot, 0xce53, game);
        word!(game.intro_samus_display = 0);
        return;
    }
    if (word!(game.cinematicspr_goto_timer[slot]) as i32 - 128) & 0x8000 != 0 {
        let timer = word!(game.cinematicspr_goto_timer[slot]).wrapping_add(1);
        word!(game.cinematicspr_goto_timer[slot] = timer);
        if timer & 0x3f == 0 {
            game.queue_sound(2, 0x23_u8, 6);
        }
    }
    if (game.player_x() as i16).wrapping_sub(word!(game.cinematicbg_arr7[slot]) as i16) < 0 {
        if (word!(game.cinematic_var14) as i32 + 639) & 0x8000 == 0 {
            word!(game.cinematic_var14 -= 32);
        }
    } else if (word!(game.cinematic_var14) as i32 - 640) & 0x8000 != 0 {
        word!(game.cinematic_var14 += 32);
    }
    word!(
        game.intro_samus_display = if word!(game.cinematic_var14) & 0x8000 != 0 {
            1
        } else {
            u16::MAX
        }
    );
    let velocity = ((word!(game.cinematic_var14) as i16 as i32) << 8) as u32;
    advance_cinematic_coordinate(object, false, velocity, game);
    chase_intro_baby_y(object, 8, game);
}

pub fn advance_intro_rinka<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    misses_samus: bool,
    game: &mut H,
) {
    let slot = usize::from(object >> 1);
    let horizontal = if misses_samus {
        RINKA_X_SPEED[usize::from(word!(game.cinematicspr_goto_timer[slot]))]
    } else {
        0x8000
    };
    advance_cinematic_coordinate(object, false, horizontal, game);
    advance_cinematic_coordinate(object, true, 0x8000, game);
    if misses_samus {
        if !((word!(game.cinematicbg_arr8[slot]) as i32 - 16) & 0x8000 != 0
            || (word!(game.cinematicbg_arr8[slot]) as i32 - 208) & 0x8000 == 0
            || word!(game.cinematicspr_preinstr_func[0]) == 0xb80f)
        {
            return;
        }
    } else if (word!(game.cinematicbg_arr7[slot]) as i32 + 8 - (game.player_x() as i32 - 5))
        & 0x8000
        != 0
    {
        if word!(game.cinematicspr_preinstr_func[0]) != 0xb80f {
            return;
        }
    } else {
        word! { game, 11; samus_invincibility_timer, samus_knockback_timer }
        word!(game.knockback_x_dir = 1);
    }
    resume_cinematic_sprite(slot, 0xce53, game);
}

pub fn initialize_intro_brain_particle<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    big: bool,
    game: &mut H,
) {
    let parameter = usize::from(word!(game.cinematic_spawn_param));
    let slot = usize::from(object >> 1);
    word!(game.cinematicspr_goto_timer[slot] = word!(game.cinematic_spawn_param));
    let particles: &[(u16, u16, u16)] = if big {
        &[
            (56, 111, 1),
            (72, 95, 16),
            (40, 119, 32),
            (48, 95, 48),
            (64, 119, 64),
        ]
    } else {
        &[(72, 111, 1), (40, 115, 8), (40, 103, 16)]
    };
    word! { game;
        cinematicbg_arr7[slot] = particles[parameter].0;
        cinematicbg_arr8[slot] = particles[parameter].1;
        cinematicspr_instr_timer[slot] = particles[parameter].2;
        cinematicbg_arr9[slot] = 2560;
    }
}

pub fn initialize_ceres_explosion<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    pattern: CeresExplosion,
    game: &mut H,
) {
    let slot = usize::from(object >> 1);
    let parameter = if matches!(pattern, CeresExplosion::Final) {
        0
    } else {
        let parameter = usize::from(word!(game.cinematic_spawn_param));
        word!(game.cinematicspr_goto_timer[slot] = word!(game.cinematic_spawn_param));
        parameter
    };
    let (x, y, delays): (&[i16], &[i16], &[u16]) = match pattern {
        CeresExplosion::First => (
            &[16, -16, 16, -16, 0],
            &[-16, 16, 16, -16, 0],
            &[1, 16, 32, 48, 64],
        ),
        CeresExplosion::Second => (
            &[14, 8, -16, -8, 0, 16, -12, -8],
            &[-8, 12, 12, -14, 0, 14, 4, -16],
            &[],
        ),
        CeresExplosion::Third => (&[8, 12, -8, -12], &[-4, 8, -10, 12], &[1, 4, 8, 16]),
        CeresExplosion::Final => (&[0], &[0], &[]),
    };
    if !delays.is_empty() {
        word!(game.cinematicspr_instr_timer[slot] = delays[parameter]);
    }
    // Keep the Y lookup and camera reads after the X store for aliased slots.
    word! { game;
        cinematicbg_arr7[slot] = (x[parameter] as i32 + word!(game.reg_M7X) as i32
            - word!(game.cinematic_x) as i32) as u16;
        cinematicbg_arr8[slot] = (y[parameter] as i32 + word!(game.reg_M7Y) as i32
            - word!(game.cinematic_y) as i32) as u16;
        cinematicbg_arr9[slot] = 2560;
    }
}

pub fn advance_intro_landing_mosaic<H: crate::menus::host::Host + ?Sized>(
    fading: bool,
    game: &mut H,
) {
    if word!(game.cinematic_frame_counter) & 3 == 0 {
        game.ram()
            .write_byte(0x57, game.ram().read_byte(0x57).wrapping_sub(16));
        if !fading && game.ram().read_byte(0x57) & 0xf0 == 0 {
            game.ram().write_byte(0x55, 7);
            game.ram().write_byte(0x5f, 0x80);
            game.ram().write_byte(0x58, 0);
            game.ram().write_byte(0x5d, 0);
            game.ram().write_byte(0x69, 17);
            word! {game; cinematic_x = 128; cinematic_y = -104i32 as u16;
            mode7_angle = 32; mode7_zoom = 256; }
            for (definition, parameter) in [
                (0xcea3, 0x0100), // The preceding $198F store leaves $0100 in A.
                (0xcef7, 0x0e00), // Palette left in A by the preceding sprite initializer.
                (0xcefd, 0x0800), // Palette left in A by the preceding sprite initializer.
                (0xcf03, 0x0800), // Palette left in A by the preceding sprite initializer.
                (0xcf09, 0x0800), // Palette left in A by the preceding sprite initializer.
                (0xceaf, 0x0800), // Palette left in A by the preceding sprite initializer.
            ] {
                CinematicObjects::Sprites.spawn(definition, parameter, game);
            }
            crate::flow::set(game, crate::flow::Word::CinematicFunction, 0xa38f);
        }
    }
    if fading {
        game.fade(true);
        if game.ram().read_byte(0x51) == 15 {
            word! { game, 0; screen_fade_delay, screen_fade_counter }
            crate::flow::set(game, crate::flow::Word::CinematicFunction, 0xc7ca);
        }
    }
}

pub fn advance_ceres_station<H: crate::menus::host::Host + ?Sized>(object: u16, game: &mut H) {
    if crate::flow::current(game).cinematic_function == 0xbdf9 {
        let slot = usize::from(object >> 1);
        word!(game.cinematicspr_goto_timer[slot] += 128);
        let velocity = ((word!(game.cinematicspr_goto_timer[slot]) as i16 as i32) << 8) as u32;
        advance_cinematic_coordinate(object, true, velocity, game);
        advance_cinematic_coordinate(object, false, velocity, game);
        for low in [ram_words::cinematic_x_sub, ram_words::cinematic_y_sub] {
            add_native_pair(low + 2, low, velocity, game);
        }
    }
}

pub fn advance_zebes_flight<H: crate::menus::host::Host + ?Sized>(stage: u8, game: &mut H) {
    if stage == 0 {
        word!(game.cinematic_x = 62);
        word!(game.cinematic_y = -112i16 as u16);
        word!(game.mode7_angle = 32);
        word!(game.mode7_zoom = 16);
        crate::flow::set(game, crate::flow::Word::CinematicFunction, 0xc9f9);
        return;
    }
    let (horizontal, threshold, limit, increment, next) = [
        (0xffff8000, ram_words::mode7_zoom, 1152, 4, 0xca36),
        (0xffff8000, ram_words::cinematic_x, -128, 16, 0xca85),
        (0x2000, ram_words::mode7_zoom, 0x2000, 32, 0xcad0),
    ][usize::from(stage - 1)];
    add_native_pair(0x1997, 0x1995, 0x2000, game);
    add_native_pair(0x1993, 0x1991, horizontal, game);
    let negative = (word!(@read game, threshold) as i32 - limit) & 0x8000 != 0;
    if negative == (stage == 2) {
        if stage == 3 {
            game.ram().write_byte(0x69, 16);
        }
        crate::flow::set(game, crate::flow::Word::CinematicFunction, next);
        if stage == 3 {
            word!(game.cinematic_function_timer = 64);
        }
    } else {
        word!(game.mode7_zoom += increment);
        if stage == 2 && word!(game.mode7_angle) != 224 {
            word!(game.mode7_angle = (word!(game.mode7_angle) as u8).wrapping_sub(1) as u16);
        }
    }
}

pub fn advance_zebes_destruction_zoom<H: crate::menus::host::Host + ?Sized>(
    speed: u16,
    next_scene: u16,
    fade_in: Option<u16>,
    game: &mut H,
) {
    if word!(game.cinematic_frame_counter) & 1 == 0 {
        word!(game.mode7_angle = (word!(game.mode7_angle) as u8).wrapping_sub(1) as u16);
    }
    word!(game.mode7_zoom += speed);
    if (word!(game.mode7_zoom) as i32 - 384) & 0x8000 == 0
        && advance_cinematic_fade(CinematicFade::FastOut, game)
    {
        crate::flow::set(game, crate::flow::Word::CinematicFunction, next_scene);
    }
    if let Some(next) = fade_in {
        // Both fades may run this frame; the later transition wins.
        if advance_cinematic_fade(CinematicFade::FastIn, game) {
            crate::flow::set(game, crate::flow::Word::CinematicFunction, next);
        }
    }
}

pub fn advance_zebes_stars<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    moving: bool,
    game: &mut H,
) {
    let slot = usize::from(object >> 1);
    if moving {
        let offset = 2 * slot;
        add_native_pair(0x1b7d + offset, 0x1afd + offset, -32i32 as u32, game);
        let velocity = (u32::from(word!(game.cinematicspr_goto_timer[slot])) << 16)
            | u32::from(word!(game.cinematicspr_arr7[slot]));
        advance_cinematic_coordinate(object, false, velocity, game);
        wait_cinematic_scene(object, 0xde63, None, game);
    } else if crate::flow::current(game).cinematic_function == 0xdca5 {
        word!(game.cinematicspr_preinstr_func[slot] = 0xf375);
        word!(game.cinematicspr_arr7[slot] = 0x4000);
        word!(game.cinematicspr_goto_timer[slot] = 0);
    }
}

pub fn wait_cinematic_scene<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    scene: u16,
    next_preinstruction: Option<u16>,
    game: &mut H,
) {
    if crate::flow::current(game).cinematic_function == scene {
        let slot = usize::from(object >> 1);
        if let Some(next) = next_preinstruction {
            word!(game.cinematicspr_preinstr_func[slot] = next);
        } else {
            resume_cinematic_sprite(slot, 0xee9b, game);
        }
    }
}

pub fn launch_cinematic_sprite<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    acceleration: u16,
    advance_scene: bool,
    game: &mut H,
) {
    let slot = usize::from(object >> 1);
    word!(game.cinematicspr_goto_timer[slot] += acceleration);
    let velocity = -((word!(game.cinematicspr_goto_timer[slot]) as i16 as i32) << 8) as u32;
    advance_cinematic_coordinate(object, true, velocity, game);
    if (word!(game.cinematicbg_arr8[slot]) as i32 + 128) & 0x8000 != 0 {
        resume_cinematic_sprite(slot, 0xce53, game);
        if advance_scene {
            crate::flow::set(game, crate::flow::Word::CinematicFunction, 0xcadf);
        }
    }
}

pub fn update_cinematic_transform<H: crate::menus::host::Host + ?Sized>(
    rotated: bool,
    game: &mut H,
) {
    use crate::projectile_geometry::SINE_COSINE;
    if rotated {
        let angle = (word!(game.mode7_angle) as u8).wrapping_add(64);
        let cosine = scaled_cinematic_value(
            SINE_COSINE[usize::from(angle) + 64] as u16,
            word!(game.mode7_zoom),
        );
        word!(game.reg_M7A = cosine);
        word!(game.reg_M7D = word!(game.reg_M7A));
        let angle = word!(game.mode7_angle) as u8;
        let sine = scaled_cinematic_value(
            SINE_COSINE[usize::from(angle) + 64] as u16,
            word!(game.mode7_zoom),
        );
        word! {game; reg_M7B = sine; reg_M7C = sine.wrapping_neg(); }
    } else {
        word! {game; reg_M7A = word!(game.mode7_zoom); reg_M7D = word!(game.mode7_zoom); }
        word! { game, 0; reg_M7B, reg_M7C }
    }
    word!(game.reg_BG1HOFS = word!(game.cinematic_x));
    word!(game.reg_BG1VOFS = word!(game.cinematic_y));
}

pub fn draw_cinematic_rectangle<H: crate::menus::host::Host + ?Sized>(
    text: bool,
    mut definition: u16,
    position: u16,
    game: &mut H,
) {
    let mut row = 2 * u16::from(position as u8) + 64 * u16::from(position.to_ne_bytes()[1]);
    let header = cinematic_native_record::<0x8c>(definition, game);
    let width = u16::from(header.byte(2));
    let mut rows = i32::from(header.byte(3));
    loop {
        let mut destination = row;
        let mut remaining = width;
        loop {
            // The cursor wraps before the next lookup; the word at +4 does not.
            let tile = cinematic_native_record::<0x8c>(definition, game).word(4);
            let at = if text { 0x3000 } else { 0x3800 } + usize::from(destination);
            crate::word_transfers!(game, ascending, 1; native(at) <- value(tile));
            definition = definition.wrapping_add(2);
            destination = destination.wrapping_add(2);
            remaining = remaining.wrapping_sub(1);
            if remaining == 0 {
                break;
            }
        }
        // Text used a wrapping u16 row counter; background used signed i32.
        rows = if text {
            i32::from((rows as u16).wrapping_sub(1))
        } else {
            rows - 1
        };
        if rows == 0 {
            break;
        }
        row = row.wrapping_add(64);
    }
}
