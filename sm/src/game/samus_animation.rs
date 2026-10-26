//! Samus animation, sprite selection, echoes, atmospheric graphics and tile uploads.
#![forbid(unsafe_code)]
use crate::game::player_transitions as pt;
use crate::game::projectile_presentation_ram as sprite_output;
use crate::game::session_runtime::Runtime;
use crate::game::visual_host::{Content, Effects, Record, Source};

pub use crate::game::prelude::*;
use crate::game::visual_runtime::project_cinematic_point;

fn bottom_is_visible(game: &mut Game) -> u8 {
    let pose = game.samus_pose();
    let frame = game.samus_anim_frame();
    let visible = match game.samus_movement_type() {
        0 => {
            if pose == kPose_00_FaceF_Powersuit as u16 {
                let next = word!(game.oam_next_ptr);
                let entry = 0x370 + usize::from(word!(game.oam_next_ptr));
                word_transfers!(game, ascending, 1; native(entry) <- value(
                    (word!(game.samus_x_pos) as i32 - 7 - word!(game.layer1_x_pos) as i32) as u16
                ));
                word_transfers!(game, ascending, 1; native(entry + 1) <- value(
                    (word!(game.samus_y_pos) as i32 - 17 - word!(game.layer1_y_pos) as i32) as u16
                ));
                word_transfers!(game, ascending, 1; native(entry + 2) <- value(14369));
                word!(game.oam_next_ptr = next.wrapping_add(4));
            }
            true
        }
        1 | 2 | 5 | 6 | 11 | 12 | 14 | 16 | 21 | 22 | 23 | 24 | 26 => true,
        3 => {
            matches!(
                SamusPose::from(pose),
                kPose_81_FaceR_Screwattack
                    | kPose_82_FaceL_Screwattack
                    | kPose_1B_FaceR_SpaceJump
                    | kPose_1C_FaceL_SpaceJump
            ) || frame == 0
                || frame.wrapping_sub(11) & 0x8000 == 0
        }
        4 | 7 | 8 | 9 | 17 | 18 | 19 => false,
        10 => {
            !matches!(
                SamusPose::from(pose),
                kPose_D7_FaceR_CrystalFlashEnd | kPose_D8_FaceL_CrystalFlashEnd
            ) || frame.wrapping_sub(3) & 0x8000 == 0
        }
        // The unused handler's OR-composed pose constants are 0x65 and 0x66.
        13 => !matches!(pose, 0x65 | 0x66) || frame.wrapping_sub(1) & 0x8000 != 0,
        15 => {
            if pose.wrapping_sub(kPose_F1_FaceR_CrouchTrans_AimU as u16) & 0x8000 == 0 {
                true
            } else if pose.wrapping_sub(219) & 0x8000 == 0 {
                frame
                    == if pose.wrapping_sub(221) & 0x8000 != 0 {
                        0
                    } else {
                        2
                    }
            } else {
                matches!(
                    SamusPose::from(pose),
                    kPose_35_FaceR_CrouchTrans
                        | kPose_36_FaceL_CrouchTrans
                        | kPose_3B_FaceR_StandTrans
                        | kPose_3C_FaceL_StandTrans
                )
            }
        }
        20 => !(3..13).contains(&frame),
        25 => !(2..9).contains(&frame),
        27 => {
            if pose.wrapping_sub(kPose_CF_FaceR_Ranintowall_AimUR as u16) & 0x8000 != 0 {
                !matches!(
                    SamusPose::from(pose),
                    kPose_CB_FaceR_Shinespark_Vert | kPose_CC_FaceL_Shinespark_Vert
                )
            } else {
                !matches!(
                    SamusPose::from(pose),
                    kPose_E8_FaceR_Drained_CrouchFalling | kPose_E9_FaceL_Drained_CrouchFalling
                ) || frame.wrapping_sub(2) & 0x8000 == 0
            }
        }
        _ => panic!("invalid Samus movement type"),
    };
    if !visible {
        word!(game.samus_bottom_half_spritemap_index = 0);
    }
    u8::from(visible)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Body {
    Normal,
    Frozen,
    Death,
}

fn draw_body(game: &mut Game, mode: Body) {
    let h = &mut Runtime::new(game);
    let p = &h.session().player;
    if mode != Body::Normal
        || p.knockback_timer != 0
        || p.invincibility == 0
        || p.shine_timer != 0
        || h.inputs().frame & 1 == 0
    {
        let doubled_pose = p.pose.wrapping_mul(2);
        let top = sprite_index(h, 0x929263, p.pose, mode == Body::Normal);
        if mode == Body::Normal {
            h.session_mut().player.top_sprite = top;
        }
        let position = h.foreign(|game| sprite_position(doubled_pose, game));
        let (x, y) = if mode == Body::Death {
            body_position(h, mode)
        } else {
            (position.k, position.j)
        };
        h.draw_samus_sprite(top, x, y);
        if h.foreign(bottom_is_visible) & 1 != 0 {
            // The wrapped pose is captured before top drawing; frame and position stay live.
            let bottom = sprite_index(h, 0x92945d, doubled_pose >> 1, mode == Body::Normal);
            if mode == Body::Normal {
                h.session_mut().player.bottom_sprite = bottom;
            }
            let (x, y) = body_position(h, mode);
            h.draw_samus_sprite(bottom, x, y);
        }
    }
    queue_body_tiles(h);
}

fn sprite_index(h: &Runtime<'_>, table: u32, pose: u16, little_endian: bool) -> u16 {
    let frame = h.session().player.animation_clock.frame;
    let record = h.table(Source::Loaded, table);
    let offset = usize::from(pose) * 2;
    let base = if little_endian {
        u16::from_le_bytes(record.bytes(offset))
    } else {
        record.native_word(offset)
    };
    frame.wrapping_add(base)
}

fn body_position(h: &Runtime<'_>, mode: Body) -> (u16, u16) {
    // Published coordinates can be changed by drawing before the bottom-half read.
    let (x, y) = (h.word(0xb04), h.word(0xb06));
    if mode == Body::Death {
        (
            x.wrapping_add(h.inputs().camera_x),
            y.wrapping_add(h.inputs().camera_y),
        )
    } else {
        (x, y)
    }
}

pub fn Samus_Draw(game: &mut Game) {
    draw_body(game, Body::Normal);
}
pub fn Samus_DrawDuringDeathAnim(game: &mut Game) {
    draw_body(game, Body::Death);
}
pub fn Samus_DrawWhenNotAnimatingOrDying(game: &mut Game) {
    draw_body(game, Body::Frozen);
}
pub fn Samus_DrawStartingDeathAnim(game: &mut Game) {
    pt::advance_animation(&mut Runtime::new(game), pt::state::AnimationStep::Death);
    Samus_DrawDuringDeathAnim(game);
}

// Death phases share palette selection and the native upload queue with live drawing.
pub(crate) fn begin_death(game: &mut Game) -> CoroutineRet {
    Samus_DrawWhenNotAnimatingOrDying(game);
    if crate::game::music::HasQueuedMusic(game) == 0 {
        const FRAMES: [u8; 28] = [
            5, 5, 5, 5, 1, 5, 5, 0, 1, 0, 5, 5, 5, 5, 5, 5, 5, 1, 1, 1, 5, 5, 5, 5, 5, 5, 5, 5,
        ];
        let movement = game.g_ram[0xa1f];
        if game.g_ram[0xa1f] == 3 {
            crate::game::music::QueueSfx1_Max6(0x32, game);
        }
        let frame = u16::from(FRAMES[usize::from(movement)]);
        word!(game.samus_pose = if game.g_ram[0xa1e] == 4 { 0xd8 } else { 0xd7 });
        pt::finish_pose_change(&mut Runtime::new(game));
        word! { game; samus_anim_frame_skip = 0; samus_anim_frame = frame; }
        word!(game.samus_x_pos -= word!(game.layer1_x_pos));
        word!(game.samus_y_pos -= word!(game.layer1_y_pos));
        word!(game.game_state += 1);
    }
    0
}

pub(crate) fn flash_death(game: &mut Game) -> CoroutineRet {
    if word!(game.g_word_7E0DE6) as i32 - 4i32 & 0x8000 != 0 {
        queue_death_tiles((2u16).wrapping_mul(word!(game.g_word_7E0DE6)), game);
    }
    if word!(game.g_word_7E0DE6 += 1, reread) as i32 - 60i32 & 0x8000 != 0 {
        let expired = word!(game.game_options_screen_index -= 1, reread) & 0x8000 != 0;
        if word!(game.game_options_screen_index) == 0 || expired {
            if word!(game.g_word_7E0DE4) != 0 {
                word! { game; g_word_7E0DE4 = 0; game_options_screen_index = 3; }
            } else {
                word! { game, 1; g_word_7E0DE4, game_options_screen_index }
            }
            death_palette(Some(word!(game.g_word_7E0DE4).wrapping_mul(2)), game);
        }
        Samus_DrawDuringDeathAnim(game);
    } else {
        death_palette(None, game);
        queue_death_tiles(8, game);
        let delay = Runtime::new(game).table(Source::Native, 0x9bb823).byte(0);
        word!(game.game_options_screen_index = u16::from(delay));
        word! { game, 0; g_word_7E0DE4, g_word_7E0DE6 }
        advance_suit_explosion(game);
        word!(
            game.substate = (word!(game.joypad2_last) as i32 & 0xb0
                == kButton_A as i32 | kButton_L as i32 | kButton_R as i32)
                as u16
        );
        word!(game.game_state += 1);
    }
    0
}

fn death_palette(frame: Option<u16>, game: &mut Game) {
    let table = if frame.is_some() { 0x9bb6d2 } else { 0x9bb5c8 };
    let pointer = crate::rtl::rom_table_word!(
        game,
        table,
        (word!(game.samus_suit_palette_index) >> 1) as isize
    )
    .wrapping_add(frame.unwrap_or(0));
    let address = Runtime::new(game)
        .legacy_native_record(0x9b, pointer)
        .native_word(0);
    let colors = Runtime::new(game)
        .legacy_native_record(0x9b, address)
        .bytes::<32>(0);
    game.g_ram[0xc180..0xc1a0].copy_from_slice(&colors);
    // The second source is resolved only after the first complete palette copy.
    let address = match frame {
        Some(frame) => crate::rtl::rom_table_word!(game, 0x9bb80f, (frame >> 1) as isize),
        None => 0xa120,
    };
    let colors = Runtime::new(game)
        .legacy_native_record(0x9b, address)
        .bytes::<32>(0);
    game.g_ram[0xc1e0..0xc200].copy_from_slice(&colors);
}

fn queue_death_tiles(frame: u16, game: &mut Game) {
    const SOURCES: [u16; 5] = [0x8400, 0x8800, 0x8c00, 0x9000, 0x8000];
    const DESTINATIONS: [u16; 5] = [0x6200, 0x6400, 0x6600, 0x6800, 0x6000];
    crate::game::display_refresh::queue_native_transfer(
        word!(game.vram_write_queue_tail),
        1024,
        |_| SOURCES[usize::from(frame >> 1)],
        0x9b,
        |_| DESTINATIONS[usize::from(frame >> 1)],
        game,
    );
}

fn whiten_death_palette(game: &mut Game) {
    if word!(game.substate) == 0 && word!(game.g_word_7E0DE4) != 0 {
        const SHADES: [u16; 22] = [
            0x421, 0xc63, 0x14a5, 0x1ce7, 0x2529, 0x2d6b, 0x35ad, 0x4210, 0x4a52, 0x4e73, 0x5294,
            0x56b5, 0x5ad6, 0x5ef7, 0x6318, 0x6739, 0x6b5a, 0x6f7b, 0x739c, 0x77bd, 0x7bde, 0x7fff,
        ];
        let shade = SHADES[usize::from(word!(game.g_word_7E0DE6))];
        word_transfers!(game, ascending, 192; native(0xc000) <- value(shade));
        word_transfers!(game, ascending, 32; native(0xc1a0) <- value(shade));
        if word!(game.g_word_7E0DE6) as i32 - 20i32 & 0x8000 != 0 {
            word!(game.g_word_7E0DE6 += 1);
        }
    }
}

pub(crate) fn explode_suit(game: &mut Game) -> CoroutineRet {
    whiten_death_palette(game);
    if advance_suit_explosion(game) {
        word! { game, 1; screen_fade_delay, screen_fade_counter }
        word!(game.game_state += 1);
    }
    0
}

fn advance_suit_explosion(game: &mut Game) -> bool {
    let expired = word!(game.game_options_screen_index -= 1, reread) & 0x8000 != 0;
    if word!(game.game_options_screen_index) == 0 || expired {
        if word!(game.g_word_7E0DE4 += 1, reread) as i32 - 9i32 & 0x8000 == 0 {
            word!(game.g_word_7E0DE6 = 21);
            whiten_death_palette(game);
            word!(game.substate = 0);
            return true;
        }
        let recolor =
            word!(game.substate) == 0 || word!(game.g_word_7E0DE4) as i32 - 2i32 & 0x8000 != 0;
        // Keep native source capture ahead of each live, linear frame displacement.
        let delay = Runtime::new(game).table(Source::Native, 0x9bb823);
        word!(
            game.game_options_screen_index =
                u16::from(delay.byte(2 * usize::from(word!(game.g_word_7E0DE4))))
        );
        if recolor {
            let frames = Runtime::new(game).table(Source::Native, 0x9bb823);
            let frame = frames.byte(2 * usize::from(word!(game.g_word_7E0DE4)) + 1);
            death_palette(Some(2u16.wrapping_mul(u16::from(frame))), game);
        }
    }
    let sprite = (if game.g_ram[0xa1e] == 4 {
        word!(game.g_word_7E0DE4) as i32 + 2085
    } else {
        word!(game.g_word_7E0DE4) as i32 + 2076
    }) as u16;
    sprite_output::samus(
        sprite,
        (word!(game.samus_x_pos), word!(game.samus_y_pos)),
        game,
        &mut sprite_output::Native,
    );
    false
}

pub fn Samus_DrawEchoes(game: &mut Game) {
    let h = &mut Runtime::new(game);
    let converging = h.session().projectiles.echoes.index & 0x8000 != 0;
    if !converging && h.session().player.contact.speed_boost & 0xff00 != 0x400 {
        return;
    }
    for index in [1, 0] {
        if h.session().projectiles.echoes.x[index] == 0 {
            continue;
        }
        if converging {
            let s = h.session_mut();
            let e = &mut s.projectiles.echoes;
            if e.y[index] != s.player.y {
                e.y[index] = if (e.y[index].wrapping_sub(s.player.y) as i16) < 0 {
                    e.y[index].wrapping_add(2)
                } else {
                    e.y[index].wrapping_sub(2)
                };
            }
            e.x[index] = e.x[index].wrapping_add(e.x_speed[index]);
            let left = (e.x[index].wrapping_sub(s.player.x) as i16) < 0;
            if left == (e.x_speed[index] & 0x8000 != 0) {
                e.x[index] = 0;
                continue;
            }
        }
        h.foreign(|game| {
            crate::game::projectile_presentation_ram::echo(
                (index * 2) as u16,
                game,
                &mut crate::game::projectile_presentation_ram::Native,
            )
        });
    }
    if converging && h.session().projectiles.echoes.x[..2] == [0, 0] {
        h.session_mut().projectiles.echoes.index = 0;
    }
}

pub fn Samus_DrawShinesparkCrashEchoes(k: u16, game: &mut Game) {
    let h = &mut Runtime::new(game);
    if h.inputs().frame & 1 == 0 {
        return;
    }
    let top = sprite_index(h, 0x929263, h.session().player.pose, false);
    let height = h
        .table(Source::Native, 0x91b629)
        .byte(usize::from(h.session().player.pose) * 8 + 4);
    let y = h
        .word(0xab8 + usize::from(k & !1))
        .wrapping_sub(u16::from(height))
        .wrapping_sub(h.inputs().camera_y);
    if !(0..248).contains(&y) {
        return;
    }
    let x = h
        .word(0xab0 + usize::from(k & !1))
        .wrapping_sub(h.inputs().camera_x);
    h.draw_samus_sprite(top, x, y);
    if h.foreign(bottom_is_visible) & 1 != 0 {
        let bottom = sprite_index(h, 0x92945d, h.session().player.pose, false);
        let x = h
            .word(0xab0 + usize::from(k & !1))
            .wrapping_sub(h.inputs().camera_x);
        h.draw_samus_sprite(bottom, x, y);
    }
}

fn sprite_position(k: u16, game: &mut Game) -> PairU16 {
    let mut v1: PairU16 = PairU16 { k: 0, j: 0 };
    if (word!(game.ceres_status) & 0x8000) == 0 {
        v1 = calculate_sprite_position(k, game);
    } else {
        let old_x: u16 = word!(game.samus_x_pos);
        let old_y: u16 = word!(game.samus_y_pos);
        let point = project_cinematic_point(Point16U { x: old_x, y: old_y }, game);
        word! {game; samus_x_pos = point.x; samus_y_pos = point.y; }
        v1 = calculate_sprite_position(k, game);
        word! {game; samus_y_pos = old_y; samus_x_pos = old_x; }
    }
    MakePairU16(v1.k, v1.j)
}

fn queue_body_tiles(h: &mut Runtime<'_>) {
    let p = &h.session().player;
    let cursor = h
        .table(Source::Loaded, 0x92d94e)
        .native_word(usize::from(p.pose) * 2)
        .wrapping_add(p.animation_clock.frame.wrapping_mul(4));
    let definition = h.legacy_native_record(0x92, cursor);
    let top = (u16::from(definition.byte(1)) * 7).wrapping_add(
        h.table(Source::Loaded, 0x92d91e)
            .native_word(usize::from(definition.byte(0)) * 2),
    );
    h.write_word_le(top, 0x71f);
    h.set_byte(0x71d, 1);
    if definition.byte(2) != 255 {
        let bottom = (u16::from(definition.byte(3)) * 7).wrapping_add(
            h.table(Source::Loaded, 0x92d938)
                .native_word(usize::from(definition.byte(2)) * 2),
        );
        h.write_word_le(bottom, 0x721);
        h.set_byte(0x71e, 1);
    }
}

fn sprite_y_offset(doubled_pose: u16, game: &mut Game) -> i32 {
    static LANDING: [u8; 16] = [3, 6, 0, 0, 3, 6, 0, 0, 3, 3, 6, 0, 3, 3, 6, 0];
    static CROUCHING: [i8; 24] = [
        -8, 0, -8, 0, -4, -2, -4, -2, 0, 0, 0, 0, -4, 0, -4, 0, 5, 4, 5, 4, 0, 0, 0, 0,
    ];
    static DRAINED: [i8; 32] = [
        7, 5, -8, -8, -8, -8, -8, -5, 4, 4, 4, 4, 0, 0, 4, -3, -5, 0, 0, 4, -3, -5, -3, 4, 0, 0, 4,
        0, 0, 4, 0, 0,
    ];
    let pose = doubled_pose >> 1;
    let frame = game.samus_anim_frame();
    match game.samus_movement_type() {
        0 => {
            if matches!(
                SamusPose::from(pose),
                kPose_00_FaceF_Powersuit | kPose_9B_FaceF_VariaGravitySuit
            ) {
                if frame.wrapping_sub(2) & 0x8000 == 0 {
                    return -1;
                }
            } else if (0xa4..0xa8).contains(&pose) {
                // The legacy landing table deliberately reads a word from byte data.
                let offset = usize::from(frame.wrapping_add(4 * (pose - 0xa4)));
                return -i32::from(u16::from_ne_bytes([LANDING[offset], LANDING[offset + 1]]));
            }
        }
        15 if (0x35..0x41).contains(&pose) => {
            return CROUCHING[frame.wrapping_add(2 * (pose - 0x35)) as usize] as i32;
        }
        27 => {
            if matches!(
                SamusPose::from(pose),
                kPose_E8_FaceR_Drained_CrouchFalling | kPose_E9_FaceL_Drained_CrouchFalling
            ) {
                return DRAINED[frame as usize] as i32;
            }
            if matches!(
                SamusPose::from(pose),
                kPose_EA_FaceR_Drained_Stand | kPose_EB_FaceL_Drained_Stand
            ) && (frame as i16).wrapping_sub(5) >= 0
            {
                return -3;
            }
        }
        1..=26 => {}
        _ => panic!("invalid Samus movement type"),
    }
    let image = game.capture_native_rom();
    let offset = crate::rtl::lorom_offset(0x91b629) as usize
        + core::mem::offset_of!(SamusPoseParams, y_offset_to_gfx)
        + 4 * usize::from(doubled_pose);
    -i32::from(image.read(offset) as i8)
}

fn calculate_sprite_position(pose: u16, game: &mut Game) -> PairU16 {
    let offset = sprite_y_offset(pose, game);
    word!(
        game.samus_spritemap_y_pos =
            (word!(game.samus_y_pos) as i32 + offset - word!(game.layer1_y_pos) as i32) as u16
    );
    word!(
        game.samus_spritemap_x_pos = word!(game.samus_x_pos).wrapping_sub(word!(game.layer1_x_pos))
    );
    MakePairU16(
        word!(game.samus_spritemap_x_pos),
        word!(game.samus_spritemap_y_pos),
    )
}
