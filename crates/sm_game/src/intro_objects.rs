//! Intro and Zebes cinematic objects: Rinkas, the baby, explosions, flights, scenes and transforms.
use crate::cinematic::*;
use crate::cinematic_objects::CinematicObjects;
use crate::cinematic_state::{Cinematic, Word};
use crate::display::sequence::scroll::BgScroll;
use crate::ram::RawRam as _;
use crate::ram::word;
use crate::visual_state::{CeresExplosion, CinematicFade};

const RINKA_POSITIONS: [(u16, u16); 4] = [(112, 72), (192, 56), (128, 48), (232, 80)];

const RINKA_X_SPEED: [u32; 4] = [0x8000, 0xffff8000, 0x8000, 0xffff8000];

pub fn initialize_intro_rinka(object: u16, cine: &mut Cinematic) {
    let variant = usize::from(*cine.spawn_parameter());
    let slot = usize::from(object >> 1);
    *cine.spr_goto_timer(slot) = *cine.spawn_parameter();
    place_cinematic_sprite(object, RINKA_POSITIONS[variant], 3584, cine);
}

pub fn start_intro_rinka(object: u16, cursor: u16, cine: &mut Cinematic) -> u16 {
    let misses_samus = *cine.spr_goto_timer(object >> 1) != 0;
    *cine.spr_pre_instruction(object >> 1) = if misses_samus { 0xb93b } else { 0xb8d8 };
    cursor
}

pub fn chase_intro_baby_y<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    y_offset: i32,
    cine: &mut Cinematic,
    game: &mut H,
) {
    let slot = usize::from(object >> 1);
    if ((game.player_y() as i32 - y_offset - *cine.bg_arr8(slot) as i32) as i16) < 0 {
        if (*cine.var17() as i32 + 543) & 0x8000 == 0 {
            *cine.var17() = cine.var17().wrapping_sub(32);
        }
    } else if (*cine.var17() as i32 - 544) & 0x8000 != 0 {
        *cine.var17() = cine.var17().wrapping_add(32);
    }
    let velocity = ((*cine.var17() as i16 as i32) << 8) as u32;
    advance_cinematic_coordinate(object, true, velocity, cine);
}

pub fn advance_intro_baby_pursuit<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    cine: &mut Cinematic,
    game: &mut H,
) {
    let slot = usize::from(object >> 1);
    if *cine.var13() == 0 {
        resume_cinematic_sprite(slot, 0xce53, cine);
        *cine.intro_samus_display() = 0;
        return;
    }
    if (*cine.spr_goto_timer(slot) as i32 - 128) & 0x8000 != 0 {
        let timer = (*cine.spr_goto_timer(slot)).wrapping_add(1);
        *cine.spr_goto_timer(slot) = timer;
        if timer & 0x3f == 0 {
            game.queue_sound(2, 0x23_u8, 6);
        }
    }
    if (game.player_x() as i16).wrapping_sub(*cine.bg_arr7(slot) as i16) < 0 {
        if (*cine.var14() as i32 + 639) & 0x8000 == 0 {
            *cine.var14() = cine.var14().wrapping_sub(32);
        }
    } else if (*cine.var14() as i32 - 640) & 0x8000 != 0 {
        *cine.var14() = cine.var14().wrapping_add(32);
    }
    *cine.intro_samus_display() = if *cine.var14() & 0x8000 != 0 {
        1
    } else {
        u16::MAX
    };
    let velocity = ((*cine.var14() as i16 as i32) << 8) as u32;
    advance_cinematic_coordinate(object, false, velocity, cine);
    chase_intro_baby_y(object, 8, cine, game);
}

pub fn advance_intro_rinka<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    misses_samus: bool,
    cine: &mut Cinematic,
    game: &mut H,
) {
    let slot = usize::from(object >> 1);
    let horizontal = if misses_samus {
        RINKA_X_SPEED[usize::from(*cine.spr_goto_timer(slot))]
    } else {
        0x8000
    };
    advance_cinematic_coordinate(object, false, horizontal, cine);
    advance_cinematic_coordinate(object, true, 0x8000, cine);
    if misses_samus {
        if !((*cine.bg_arr8(slot) as i32 - 16) & 0x8000 != 0
            || (*cine.bg_arr8(slot) as i32 - 208) & 0x8000 == 0
            || *cine.spr_pre_instruction(0) == 0xb80f)
        {
            return;
        }
    } else if (*cine.bg_arr7(slot) as i32 + 8 - (game.player_x() as i32 - 5)) & 0x8000 != 0 {
        if *cine.spr_pre_instruction(0) != 0xb80f {
            return;
        }
    } else {
        game.set_invincibility_timer(11);
        game.set_knockback_timer(11);
        crate::session_cohort::with(game, |_, words: &mut crate::session_words::SessionWords| {
            words.knockback_x_dir = 1
        });
    }
    resume_cinematic_sprite(slot, 0xce53, cine);
}

pub fn initialize_intro_brain_particle(object: u16, big: bool, cine: &mut Cinematic) {
    let parameter = usize::from(*cine.spawn_parameter());
    let slot = usize::from(object >> 1);
    *cine.spr_goto_timer(slot) = *cine.spawn_parameter();
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
    *cine.bg_arr7(slot) = particles[parameter].0;
    *cine.bg_arr8(slot) = particles[parameter].1;
    *cine.spr_instruction_timer(slot) = particles[parameter].2;
    *cine.bg_arr9(slot) = 2560;
}

pub fn initialize_ceres_explosion<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    pattern: CeresExplosion,
    cine: &mut Cinematic,
    game: &mut H,
) {
    let slot = usize::from(object >> 1);
    let parameter = if matches!(pattern, CeresExplosion::Final) {
        0
    } else {
        let parameter = usize::from(*cine.spawn_parameter());
        *cine.spr_goto_timer(slot) = *cine.spawn_parameter();
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
        *cine.spr_instruction_timer(slot) = delays[parameter];
    }
    // Keep the Y lookup and camera reads after the X store for aliased slots.
    *cine.bg_arr7(slot) =
        (x[parameter] as i32 + word!(game.REG_M7X) as i32 - *cine.x() as i32) as u16;
    *cine.bg_arr8(slot) =
        (y[parameter] as i32 + word!(game.REG_M7Y) as i32 - *cine.y() as i32) as u16;
    *cine.bg_arr9(slot) = 2560;
}

pub fn advance_intro_landing_mosaic<H: crate::menus::host::Host + ?Sized>(
    fading: bool,
    cine: &mut Cinematic,
    game: &mut H,
) {
    if *cine.frame_counter() & 3 == 0 {
        game.ram()
            .write_byte(0x57, game.ram().read_byte(0x57).wrapping_sub(16));
        if !fading && game.ram().read_byte(0x57) & 0xf0 == 0 {
            game.ram().write_byte(0x55, 7);
            game.ram().write_byte(0x5f, 0x80);
            game.ram().write_byte(0x58, 0);
            game.ram().write_byte(0x5d, 0);
            game.ram().write_byte(0x69, 17);
            *cine.x() = 128;
            *cine.y() = -104i32 as u16;
            *cine.mode7_angle() = 32;
            *cine.mode7_zoom() = 256;
            for (definition, parameter) in [
                (0xcea3, 0x0100), // The preceding $198F store leaves $0100 in A.
                (0xcef7, 0x0e00), // Palette left in A by the preceding sprite initializer.
                (0xcefd, 0x0800), // Palette left in A by the preceding sprite initializer.
                (0xcf03, 0x0800), // Palette left in A by the preceding sprite initializer.
                (0xcf09, 0x0800), // Palette left in A by the preceding sprite initializer.
                (0xceaf, 0x0800), // Palette left in A by the preceding sprite initializer.
            ] {
                CinematicObjects::Sprites.spawn(definition, parameter, cine, game);
            }
            crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
                flow.cinematic_function = 0xa38f
            });
        }
    }
    if fading {
        game.fade(true);
        if game.ram().read_byte(0x51) == 15 {
            crate::screen_fade::set_both(game, 0, 0);
            crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
                flow.cinematic_function = 0xc7ca
            });
        }
    }
}

pub fn advance_ceres_station<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    cine: &mut Cinematic,
    game: &mut H,
) {
    if crate::session_cohort::load::<crate::flow::Flow, _>(game).cinematic_function == 0xbdf9 {
        let slot = usize::from(object >> 1);
        *cine.spr_goto_timer(slot) = cine.spr_goto_timer(slot).wrapping_add(128);
        let velocity = ((*cine.spr_goto_timer(slot) as i16 as i32) << 8) as u32;
        advance_cinematic_coordinate(object, true, velocity, cine);
        advance_cinematic_coordinate(object, false, velocity, cine);
        for (high, low) in [(Word::X, Word::XSub), (Word::Y, Word::YSub)] {
            cine.add_pair(high, low, velocity);
        }
    }
}

pub fn advance_zebes_flight<H: crate::menus::host::Host + ?Sized>(
    stage: u8,
    cine: &mut Cinematic,
    game: &mut H,
) {
    if stage == 0 {
        *cine.x() = 62;
        *cine.y() = -112i16 as u16;
        *cine.mode7_angle() = 32;
        *cine.mode7_zoom() = 16;
        crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
            flow.cinematic_function = 0xc9f9
        });
        return;
    }
    let (horizontal, threshold, limit, increment, next) = [
        (0xffff8000, Word::Mode7Zoom, 1152, 4, 0xca36),
        (0xffff8000, Word::X, -128, 16, 0xca85),
        (0x2000, Word::Mode7Zoom, 0x2000, 32, 0xcad0),
    ][usize::from(stage - 1)];
    cine.add_pair(Word::Y, Word::YSub, 0x2000);
    cine.add_pair(Word::X, Word::XSub, horizontal);
    let negative = (*cine.word(threshold) as i32 - limit) & 0x8000 != 0;
    if negative == (stage == 2) {
        if stage == 3 {
            game.ram().write_byte(0x69, 16);
        }
        crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
            flow.cinematic_function = next
        });
        if stage == 3 {
            *cine.function_timer() = 64;
        }
    } else {
        *cine.mode7_zoom() = cine.mode7_zoom().wrapping_add(increment);
        if stage == 2 && *cine.mode7_angle() != 224 {
            *cine.mode7_angle() = (*cine.mode7_angle() as u8).wrapping_sub(1) as u16;
        }
    }
}

pub fn advance_zebes_destruction_zoom<H: crate::menus::host::Host + ?Sized>(
    speed: u16,
    next_scene: u16,
    fade_in: Option<u16>,
    cine: &mut Cinematic,
    game: &mut H,
) {
    if *cine.frame_counter() & 1 == 0 {
        *cine.mode7_angle() = (*cine.mode7_angle() as u8).wrapping_sub(1) as u16;
    }
    *cine.mode7_zoom() = cine.mode7_zoom().wrapping_add(speed);
    if (*cine.mode7_zoom() as i32 - 384) & 0x8000 == 0
        && advance_cinematic_fade(CinematicFade::FastOut, game)
    {
        crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
            flow.cinematic_function = next_scene
        });
    }
    if let Some(next) = fade_in {
        // Both fades may run this frame; the later transition wins.
        if advance_cinematic_fade(CinematicFade::FastIn, game) {
            crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
                flow.cinematic_function = next
            });
        }
    }
}

pub fn advance_zebes_stars<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    moving: bool,
    cine: &mut Cinematic,
    game: &mut H,
) {
    let slot = usize::from(object >> 1);
    if moving {
        crate::cinematic::add_cinematic_pair(
            crate::cinematic_state::Array::SprGotoTimer,
            crate::cinematic_state::Array::SprArr7,
            object >> 1,
            -32i32 as u32,
            cine,
        );
        let velocity =
            (u32::from(*cine.spr_goto_timer(slot)) << 16) | u32::from(*cine.spr_arr7(slot));
        advance_cinematic_coordinate(object, false, velocity, cine);
        wait_cinematic_scene(object, 0xde63, None, cine, game);
    } else if crate::session_cohort::load::<crate::flow::Flow, _>(game).cinematic_function == 0xdca5
    {
        *cine.spr_pre_instruction(slot) = 0xf375;
        *cine.spr_arr7(slot) = 0x4000;
        *cine.spr_goto_timer(slot) = 0;
    }
}

pub fn wait_cinematic_scene<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    scene: u16,
    next_preinstruction: Option<u16>,
    cine: &mut Cinematic,
    game: &mut H,
) {
    if crate::session_cohort::load::<crate::flow::Flow, _>(game).cinematic_function == scene {
        let slot = usize::from(object >> 1);
        if let Some(next) = next_preinstruction {
            *cine.spr_pre_instruction(slot) = next;
        } else {
            resume_cinematic_sprite(slot, 0xee9b, cine);
        }
    }
}

pub fn launch_cinematic_sprite<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    acceleration: u16,
    advance_scene: bool,
    cine: &mut Cinematic,
    game: &mut H,
) {
    let slot = usize::from(object >> 1);
    *cine.spr_goto_timer(slot) = cine.spr_goto_timer(slot).wrapping_add(acceleration);
    let velocity = -((*cine.spr_goto_timer(slot) as i16 as i32) << 8) as u32;
    advance_cinematic_coordinate(object, true, velocity, cine);
    if (*cine.bg_arr8(slot) as i32 + 128) & 0x8000 != 0 {
        resume_cinematic_sprite(slot, 0xce53, cine);
        if advance_scene {
            crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
                flow.cinematic_function = 0xcadf
            });
        }
    }
}

pub fn update_cinematic_transform<H: crate::menus::host::Host + ?Sized>(
    rotated: bool,
    cine: &mut Cinematic,
    game: &mut H,
) {
    use crate::projectile_geometry::SINE_COSINE;
    if rotated {
        let angle = (*cine.mode7_angle() as u8).wrapping_add(64);
        let cosine = scaled_cinematic_value(
            SINE_COSINE[usize::from(angle) + 64] as u16,
            *cine.mode7_zoom(),
        );
        word!(game.REG_M7A = cosine);
        word!(game.REG_M7D = word!(game.REG_M7A));
        let angle = *cine.mode7_angle() as u8;
        let sine = scaled_cinematic_value(
            SINE_COSINE[usize::from(angle) + 64] as u16,
            *cine.mode7_zoom(),
        );
        word! {game; REG_M7B = sine; REG_M7C = sine.wrapping_neg(); }
    } else {
        word! {game; REG_M7A = *cine.mode7_zoom(); REG_M7D = *cine.mode7_zoom(); }
        word! { game, 0; REG_M7B, REG_M7C }
    }
    let position = [*cine.x(), *cine.y()];
    crate::session_cohort::with(game, |_, scroll: &mut BgScroll| scroll.bg1 = position);
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
