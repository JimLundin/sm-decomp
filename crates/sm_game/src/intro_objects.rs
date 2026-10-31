//! Intro and Zebes cinematic objects: Rinkas, the baby, explosions, flights, scenes and transforms.
use crate::cinematic::*;
use crate::cinematic_objects::CinematicObjects;
use crate::cinematic_state::{self, Word, cinematic};
use crate::ram::RawRam as _;
use crate::ram::{word, words as ram_words};
use crate::visual_state::{CeresExplosion, CinematicFade};

const RINKA_POSITIONS: [(u16, u16); 4] = [(112, 72), (192, 56), (128, 48), (232, 80)];

const RINKA_X_SPEED: [u32; 4] = [0x8000, 0xffff8000, 0x8000, 0xffff8000];

pub fn initialize_intro_rinka<H: crate::menus::host::Host + ?Sized>(object: u16, game: &mut H) {
    let variant = usize::from(cinematic!(game.SpawnParameter));
    let slot = usize::from(object >> 1);
    cinematic!(game.SprGotoTimer[slot] = cinematic!(game.SpawnParameter));
    place_cinematic_sprite(object, RINKA_POSITIONS[variant], 3584, game);
}

pub fn start_intro_rinka<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    cursor: u16,
    game: &mut H,
) -> u16 {
    let misses_samus = cinematic!(game.SprGotoTimer[object >> 1]) != 0;
    cinematic!(game.SprPreInstruction[object >> 1] = if misses_samus { 0xb93b } else { 0xb8d8 });
    cursor
}

pub fn chase_intro_baby_y<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    y_offset: i32,
    game: &mut H,
) {
    let slot = usize::from(object >> 1);
    if ((game.player_y() as i32 - y_offset - cinematic!(game.BgArr8[slot]) as i32) as i16) < 0 {
        if (cinematic!(game.Var17) as i32 + 543) & 0x8000 == 0 {
            cinematic!(game.Var17 -= 32);
        }
    } else if (cinematic!(game.Var17) as i32 - 544) & 0x8000 != 0 {
        cinematic!(game.Var17 += 32);
    }
    let velocity = ((cinematic!(game.Var17) as i16 as i32) << 8) as u32;
    advance_cinematic_coordinate(object, true, velocity, game);
}

pub fn advance_intro_baby_pursuit<H: crate::menus::host::Host + ?Sized>(object: u16, game: &mut H) {
    let slot = usize::from(object >> 1);
    if cinematic!(game.Var13) == 0 {
        resume_cinematic_sprite(slot, 0xce53, game);
        cinematic!(game.IntroSamusDisplay = 0);
        return;
    }
    if (cinematic!(game.SprGotoTimer[slot]) as i32 - 128) & 0x8000 != 0 {
        let timer = cinematic!(game.SprGotoTimer[slot]).wrapping_add(1);
        cinematic!(game.SprGotoTimer[slot] = timer);
        if timer & 0x3f == 0 {
            game.queue_sound(2, 0x23_u8, 6);
        }
    }
    if (game.player_x() as i16).wrapping_sub(cinematic!(game.BgArr7[slot]) as i16) < 0 {
        if (cinematic!(game.Var14) as i32 + 639) & 0x8000 == 0 {
            cinematic!(game.Var14 -= 32);
        }
    } else if (cinematic!(game.Var14) as i32 - 640) & 0x8000 != 0 {
        cinematic!(game.Var14 += 32);
    }
    cinematic!(
        game.IntroSamusDisplay = if cinematic!(game.Var14) & 0x8000 != 0 {
            1
        } else {
            u16::MAX
        }
    );
    let velocity = ((cinematic!(game.Var14) as i16 as i32) << 8) as u32;
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
        RINKA_X_SPEED[usize::from(cinematic!(game.SprGotoTimer[slot]))]
    } else {
        0x8000
    };
    advance_cinematic_coordinate(object, false, horizontal, game);
    advance_cinematic_coordinate(object, true, 0x8000, game);
    if misses_samus {
        if !((cinematic!(game.BgArr8[slot]) as i32 - 16) & 0x8000 != 0
            || (cinematic!(game.BgArr8[slot]) as i32 - 208) & 0x8000 == 0
            || cinematic!(game.SprPreInstruction[0]) == 0xb80f)
        {
            return;
        }
    } else if (cinematic!(game.BgArr7[slot]) as i32 + 8 - (game.player_x() as i32 - 5)) & 0x8000
        != 0
    {
        if cinematic!(game.SprPreInstruction[0]) != 0xb80f {
            return;
        }
    } else {
        game.set_invincibility_timer(11);
        game.set_knockback_timer(11);
        word!(game.KNOCKBACK_X_DIR = 1);
    }
    resume_cinematic_sprite(slot, 0xce53, game);
}

pub fn initialize_intro_brain_particle<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    big: bool,
    game: &mut H,
) {
    let parameter = usize::from(cinematic!(game.SpawnParameter));
    let slot = usize::from(object >> 1);
    cinematic!(game.SprGotoTimer[slot] = cinematic!(game.SpawnParameter));
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
    cinematic! { game;
        BgArr7[slot] = particles[parameter].0;
        BgArr8[slot] = particles[parameter].1;
        SprInstructionTimer[slot] = particles[parameter].2;
        BgArr9[slot] = 2560;
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
        let parameter = usize::from(cinematic!(game.SpawnParameter));
        cinematic!(game.SprGotoTimer[slot] = cinematic!(game.SpawnParameter));
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
        cinematic!(game.SprInstructionTimer[slot] = delays[parameter]);
    }
    // Keep the Y lookup and camera reads after the X store for aliased slots.
    cinematic! { game;
        BgArr7[slot] = (x[parameter] as i32 + word!(game.REG_M7X) as i32
            - cinematic!(game.X) as i32) as u16;
        BgArr8[slot] = (y[parameter] as i32 + word!(game.REG_M7Y) as i32
            - cinematic!(game.Y) as i32) as u16;
        BgArr9[slot] = 2560;
    }
}

pub fn advance_intro_landing_mosaic<H: crate::menus::host::Host + ?Sized>(
    fading: bool,
    game: &mut H,
) {
    if cinematic!(game.FrameCounter) & 3 == 0 {
        game.ram()
            .write_byte(0x57, game.ram().read_byte(0x57).wrapping_sub(16));
        if !fading && game.ram().read_byte(0x57) & 0xf0 == 0 {
            game.ram().write_byte(0x55, 7);
            game.ram().write_byte(0x5f, 0x80);
            game.ram().write_byte(0x58, 0);
            game.ram().write_byte(0x5d, 0);
            game.ram().write_byte(0x69, 17);
            cinematic! {game; X = 128; Y = -104i32 as u16;
            Mode7Angle = 32; Mode7Zoom = 256; }
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
            crate::screen_fade::set_both(game, 0, 0);
            crate::flow::set(game, crate::flow::Word::CinematicFunction, 0xc7ca);
        }
    }
}

pub fn advance_ceres_station<H: crate::menus::host::Host + ?Sized>(object: u16, game: &mut H) {
    if crate::flow::current(game).cinematic_function == 0xbdf9 {
        let slot = usize::from(object >> 1);
        cinematic!(game.SprGotoTimer[slot] += 128);
        let velocity = ((cinematic!(game.SprGotoTimer[slot]) as i16 as i32) << 8) as u32;
        advance_cinematic_coordinate(object, true, velocity, game);
        advance_cinematic_coordinate(object, false, velocity, game);
        for low in [ram_words::CINEMATIC_X_SUB, ram_words::CINEMATIC_Y_SUB] {
            add_native_pair(low + 2, low, velocity, game);
        }
    }
}

pub fn advance_zebes_flight<H: crate::menus::host::Host + ?Sized>(stage: u8, game: &mut H) {
    if stage == 0 {
        cinematic!(game.X = 62);
        cinematic!(game.Y = -112i16 as u16);
        cinematic!(game.Mode7Angle = 32);
        cinematic!(game.Mode7Zoom = 16);
        crate::flow::set(game, crate::flow::Word::CinematicFunction, 0xc9f9);
        return;
    }
    let (horizontal, threshold, limit, increment, next) = [
        (0xffff8000, Word::Mode7Zoom, 1152, 4, 0xca36),
        (0xffff8000, Word::X, -128, 16, 0xca85),
        (0x2000, Word::Mode7Zoom, 0x2000, 32, 0xcad0),
    ][usize::from(stage - 1)];
    add_native_pair(0x1997, 0x1995, 0x2000, game);
    add_native_pair(0x1993, 0x1991, horizontal, game);
    let negative = (cinematic_state::get(game, threshold) as i32 - limit) & 0x8000 != 0;
    if negative == (stage == 2) {
        if stage == 3 {
            game.ram().write_byte(0x69, 16);
        }
        crate::flow::set(game, crate::flow::Word::CinematicFunction, next);
        if stage == 3 {
            cinematic!(game.FunctionTimer = 64);
        }
    } else {
        cinematic!(game.Mode7Zoom += increment);
        if stage == 2 && cinematic!(game.Mode7Angle) != 224 {
            cinematic!(
                game.Mode7Angle = (cinematic!(game.Mode7Angle) as u8).wrapping_sub(1) as u16
            );
        }
    }
}

pub fn advance_zebes_destruction_zoom<H: crate::menus::host::Host + ?Sized>(
    speed: u16,
    next_scene: u16,
    fade_in: Option<u16>,
    game: &mut H,
) {
    if cinematic!(game.FrameCounter) & 1 == 0 {
        cinematic!(game.Mode7Angle = (cinematic!(game.Mode7Angle) as u8).wrapping_sub(1) as u16);
    }
    cinematic!(game.Mode7Zoom += speed);
    if (cinematic!(game.Mode7Zoom) as i32 - 384) & 0x8000 == 0
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
        let velocity = (u32::from(cinematic!(game.SprGotoTimer[slot])) << 16)
            | u32::from(cinematic!(game.SprArr7[slot]));
        advance_cinematic_coordinate(object, false, velocity, game);
        wait_cinematic_scene(object, 0xde63, None, game);
    } else if crate::flow::current(game).cinematic_function == 0xdca5 {
        cinematic!(game.SprPreInstruction[slot] = 0xf375);
        cinematic!(game.SprArr7[slot] = 0x4000);
        cinematic!(game.SprGotoTimer[slot] = 0);
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
            cinematic!(game.SprPreInstruction[slot] = next);
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
    cinematic!(game.SprGotoTimer[slot] += acceleration);
    let velocity = -((cinematic!(game.SprGotoTimer[slot]) as i16 as i32) << 8) as u32;
    advance_cinematic_coordinate(object, true, velocity, game);
    if (cinematic!(game.BgArr8[slot]) as i32 + 128) & 0x8000 != 0 {
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
        let angle = (cinematic!(game.Mode7Angle) as u8).wrapping_add(64);
        let cosine = scaled_cinematic_value(
            SINE_COSINE[usize::from(angle) + 64] as u16,
            cinematic!(game.Mode7Zoom),
        );
        word!(game.REG_M7A = cosine);
        word!(game.REG_M7D = word!(game.REG_M7A));
        let angle = cinematic!(game.Mode7Angle) as u8;
        let sine = scaled_cinematic_value(
            SINE_COSINE[usize::from(angle) + 64] as u16,
            cinematic!(game.Mode7Zoom),
        );
        word! {game; REG_M7B = sine; REG_M7C = sine.wrapping_neg(); }
    } else {
        word! {game; REG_M7A = cinematic!(game.Mode7Zoom); REG_M7D = cinematic!(game.Mode7Zoom); }
        word! { game, 0; REG_M7B, REG_M7C }
    }
    word!(game.REG_BG1HOFS = cinematic!(game.X));
    word!(game.REG_BG1VOFS = cinematic!(game.Y));
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
