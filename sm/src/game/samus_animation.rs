//! Samus animation, sprite selection, echoes, atmospheric graphics and tile uploads.
use crate::game::player_transitions as pt;
use crate::game::session_runtime::Runtime;
use crate::game::visual_host::{Content, Effects, Record, Source};

use crate::game::music::{PlayRoomMusicTrackAfterAFrames, QueueMusic_DelayedY};
pub use crate::game::prelude::*;
use crate::game::sm_8b::Samus_CalcPos_Mode7;

use crate::game::sm_81::DrawSamusSpritemap;

const RomPtr_90: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x90>;
const RomPtr_91: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x91>;
crate::ram_map::ram_pointer_functions! {
    gOamEnt(v: i32) -> OamEnt = [(0x370 + v) as isize];
}

unsafe fn bottom_is_visible(game: &mut Game) -> u8 {
    unsafe {
        let pose = game.samus_pose();
        let frame = game.samus_anim_frame();
        let visible = match game.samus_movement_type() {
            0 => {
                if pose == kPose_00_FaceF_Powersuit as u16 {
                    let next = word!(game.oam_next_ptr);
                    let entry = gOamEnt(word!(game.oam_next_ptr) as i32, game);
                    *(&raw mut (*entry).xcoord as *mut u16) =
                        (word!(game.samus_x_pos) as i32 - 7 - word!(game.layer1_x_pos) as i32)
                            as u16;
                    *(&raw mut (*entry).ycoord as *mut u16) =
                        (word!(game.samus_y_pos) as i32 - 17 - word!(game.layer1_y_pos) as i32)
                            as u16;
                    *(&raw mut (*entry).charnum as *mut u16) = 14369;
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
        let position = h.foreign(|game| unsafe { Samus_CalcSpritemapPos(doubled_pose, game) });
        let (x, y) = if mode == Body::Death {
            body_position(h, mode)
        } else {
            (position.k, position.j)
        };
        h.draw_samus_sprite(top, x, y);
        if h.foreign(|game| unsafe { bottom_is_visible(game) }) & 1 != 0 {
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
    if h.foreign(|game| unsafe { bottom_is_visible(game) }) & 1 != 0 {
        let bottom = sprite_index(h, 0x92945d, h.session().player.pose, false);
        let x = h
            .word(0xab0 + usize::from(k & !1))
            .wrapping_sub(h.inputs().camera_x);
        h.draw_samus_sprite(bottom, x, y);
    }
}

pub unsafe fn Samus_CalcSpritemapPos(k: u16, game: &mut Game) -> PairU16 {
    unsafe {
        let mut v1: PairU16 = PairU16 { k: 0, j: 0 };
        if (word!(game.ceres_status) & 0x8000) == 0 {
            v1 = calculate_sprite_position(k, game);
        } else {
            let old_x: u16 = word!(game.samus_x_pos);
            let old_y: u16 = word!(game.samus_y_pos);
            Samus_CalcPos_Mode7(game);
            v1 = calculate_sprite_position(k, game);
            word! {game; samus_y_pos = old_y; samus_x_pos = old_x; }
        }
        MakePairU16(v1.k, v1.j)
    }
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
pub unsafe fn PlaySamusFanfare(game: &mut Game) -> u8 {
    unsafe {
        if word!(game.substate) != 0 {
            if word!(game.substate) == 5 {
                PlayRoomMusicTrackAfterAFrames(0x168, game);
            }
        } else {
            QueueMusic_DelayedY(1, 0xe, game);
        }
        if word!(game.substate) as i32 - 359i32 & 0x8000 != 0 {
            word!(game.substate += 1);
            0
        } else {
            pt::record_pose_history(Runtime::new(game).session_mut());
            word!(game.substate = 0);
            1
        }
    }
}
pub unsafe fn DrawSamusSuitExploding(game: &mut Game) {
    unsafe {
        let r18: u16 = (if *ram_ptr!(game; 0xa1e) == 4 {
            word!(game.g_word_7E0DE4) as i32 + 2085
        } else {
            word!(game.g_word_7E0DE4) as i32 + 2076
        }) as u16;
        DrawSamusSpritemap(r18, word!(game.samus_x_pos), word!(game.samus_y_pos), game);
    }
}

unsafe fn sprite_y_offset(doubled_pose: u16, game: &mut Game) -> i32 {
    unsafe {
        static LANDING: [u8; 16] = [3, 6, 0, 0, 3, 6, 0, 0, 3, 3, 6, 0, 3, 3, 6, 0];
        static CROUCHING: [i8; 24] = [
            -8, 0, -8, 0, -4, -2, -4, -2, 0, 0, 0, 0, -4, 0, -4, 0, 5, 4, 5, 4, 0, 0, 0, 0,
        ];
        static DRAINED: [i8; 32] = [
            7, 5, -8, -8, -8, -8, -8, -5, 4, 4, 4, 4, 0, 0, 4, -3, -5, 0, 0, 4, -3, -5, -3, 4, 0,
            0, 4, 0, 0, 4, 0, 0,
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
                    let offset = frame.wrapping_add(4 * (pose - 0xa4));
                    return -(*(LANDING.as_ptr().offset(offset as isize) as *const u16) as i32);
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
        -(*(&raw mut (*(rom_ptr!(game, SamusPoseParams; 0x91b629))).y_offset_to_gfx)
            .offset((4 * doubled_pose as i32) as isize) as i8 as i32)
    }
}

unsafe fn calculate_sprite_position(pose: u16, game: &mut Game) -> PairU16 {
    unsafe {
        let offset = sprite_y_offset(pose, game);
        word!(
            game.samus_spritemap_y_pos =
                (word!(game.samus_y_pos) as i32 + offset - word!(game.layer1_y_pos) as i32) as u16
        );
        word!(
            game.samus_spritemap_x_pos =
                word!(game.samus_x_pos).wrapping_sub(word!(game.layer1_x_pos))
        );
        MakePairU16(
            word!(game.samus_spritemap_x_pos),
            word!(game.samus_spritemap_y_pos),
        )
    }
}
