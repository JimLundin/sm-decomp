//! Samus animation, sprite selection, echoes, atmospheric graphics and tile uploads.
use crate::game::player_transitions as pt;
use crate::game::session_runtime::Runtime;

use crate::game::music::{PlayRoomMusicTrackAfterAFrames, QueueMusic_DelayedY};
pub use crate::game::prelude::*;
use crate::game::sm_8b::Samus_CalcPos_Mode7;

use crate::game::sm_81::DrawSamusSpritemap;

use crate::types::FuncXY_V;
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

pub unsafe fn Samus_Draw(game: &mut Game) {
    unsafe {
        let knockback_timer = u16::from_le_bytes([game.g_ram[0x18aa], game.g_ram[0x18ab]]);
        let invincibility_timer = u16::from_le_bytes([game.g_ram[0x18a8], game.g_ram[0x18a9]]);
        let shine_timer = u16::from_le_bytes([game.g_ram[0xa68], game.g_ram[0xa69]]);
        let nmi_frame = u16::from_le_bytes([game.g_ram[0x5b6], game.g_ram[0x5b7]]);
        if knockback_timer != 0
            || invincibility_timer == 0
            || shine_timer != 0
            || nmi_frame & 1 == 0
        {
            let doubled_pose = game.samus_pose().wrapping_mul(2);
            let frame = game.samus_anim_frame();
            let top_base = game.read_rom_word(0x929263, usize::from(game.samus_pose()) * 2);
            let top_spritemap = frame.wrapping_add(top_base);
            game.g_ram[0xac8..0xaca].copy_from_slice(&top_spritemap.to_le_bytes());
            let position = Samus_CalcSpritemapPos(game.samus_pose().wrapping_mul(2), game);
            DrawSamusSpritemap(top_spritemap, position.k, position.j, game);
            if bottom_is_visible(game) & 1 != 0 {
                // The lower table keeps the captured, word-wrapped pose; frame and position stay live.
                let frame = game.samus_anim_frame();
                let bottom_base = game.read_rom_word(0x92945d, usize::from(doubled_pose >> 1) * 2);
                let bottom_spritemap = frame.wrapping_add(bottom_base);
                game.g_ram[0xaca..0xacc].copy_from_slice(&bottom_spritemap.to_le_bytes());
                let x_pos = u16::from_le_bytes([game.g_ram[0xb04], game.g_ram[0xb05]]);
                let y_pos = u16::from_le_bytes([game.g_ram[0xb06], game.g_ram[0xb07]]);
                DrawSamusSpritemap(bottom_spritemap, x_pos, y_pos, game);
            }
        }
        SetSamusTilesDefsForCurAnim(game);
    }
}

pub unsafe fn Samus_DrawEchoes(game: &mut Game) {
    unsafe {
        if (word!(game.speed_echoes_index) & 0x8000) == 0 {
            if (word!(game.speed_boost_counter) & 0xff00) == 1024 {
                if word!(game.speed_echo_xpos[1]) != 0 {
                    crate::game::projectile_presentation_ram::echo(
                        2,
                        game,
                        &mut crate::game::projectile_presentation_ram::Native,
                    );
                }
                if word!(game.speed_echo_xpos[0]) != 0 {
                    crate::game::projectile_presentation_ram::echo(
                        0,
                        game,
                        &mut crate::game::projectile_presentation_ram::Native,
                    );
                }
            }
            return;
        }
        for i in (0i32..=2).rev().step_by(2) {
            let v1: i32 = i >> 1;
            if word!(game.speed_echo_xpos[v1]) != 0 {
                let v2: u16 = word!(game.speed_echo_ypos[v1]);
                let mut v3: u16 = 0;
                if v2 != word!(game.samus_y_pos) {
                    if ((v2) as i16).wrapping_sub((word!(game.samus_y_pos)) as i16) < 0 {
                        v3 = (v2).wrapping_add(2);
                    } else {
                        v3 = (v2).wrapping_sub(2);
                    }
                    word!(game.speed_echo_ypos[v1] = v3);
                }
                let mut skip_echo: bool = if (word!(game.speed_echo_xspeed[v1]) & 0x8000) != 0 {
                    let v5: u16 = (word!(game.speed_echo_xspeed[v1]))
                        .wrapping_add(word!(game.speed_echo_xpos[v1]));
                    word!(game.speed_echo_xpos[v1] = v5);
                    (((v5) as i16).wrapping_sub((word!(game.samus_x_pos)) as i16) < 0)
                        && ({
                            word!(game.speed_echo_xpos[v1] = 0);
                            true
                        })
                } else {
                    let v4: u16 = (word!(game.speed_echo_xspeed[v1]))
                        .wrapping_add(word!(game.speed_echo_xpos[v1]));
                    word!(game.speed_echo_xpos[v1] = v4);
                    (((v4) as i16).wrapping_sub((word!(game.samus_x_pos)) as i16) >= 0)
                        && ({
                            word!(game.speed_echo_xpos[v1] = 0);
                            true
                        })
                };
                if !(skip_echo) {
                    crate::game::projectile_presentation_ram::echo(
                        i as u16,
                        game,
                        &mut crate::game::projectile_presentation_ram::Native,
                    );
                }
            }
        }
        if word!(game.speed_echo_xpos[1]) == 0 && word!(game.speed_echo_xpos[0]) == 0 {
            word!(game.speed_echoes_index = 0);
        }
    }
}

pub unsafe fn Samus_DrawShinesparkCrashEchoes(k: u16, game: &mut Game) {
    unsafe {
        if (word!(game.nmi_frame_counter_word) & 1) != 0 {
            let a: u16 = (word!(game.samus_anim_frame)).wrapping_add(crate::rtl::rom_table_word!(
                game,
                0x929263,
                word!(game.samus_pose) as isize
            ));
            let v2: i16 = (word!(game.speed_echo_ypos[(k >> 1)]) as i32
                - (*(rom_ptr!(game, SamusPoseParams; 0x91b629))
                    .offset(word!(game.samus_pose) as isize))
                .y_offset_to_gfx as i32
                - word!(game.layer1_y_pos) as i32) as i16;
            if v2 >= 0 && v2 as i32 - 248i32 & 0x8000 != 0 {
                DrawSamusSpritemap(
                    a,
                    (word!(game.speed_echo_xpos[(k >> 1)])).wrapping_sub(word!(game.layer1_x_pos)),
                    v2 as u16,
                    game,
                );
                if bottom_is_visible(game) & 1 != 0 {
                    let v5: u16 =
                        (word!(game.samus_anim_frame)).wrapping_add(crate::rtl::rom_table_word!(
                            game,
                            0x92945d,
                            word!(game.samus_pose) as isize
                        ));
                    DrawSamusSpritemap(
                        v5,
                        (word!(game.speed_echo_xpos[(k >> 1)]))
                            .wrapping_sub(word!(game.layer1_x_pos)),
                        v2 as u16,
                        game,
                    );
                }
            }
        }
    }
}

pub unsafe fn Samus_DrawStartingDeathAnim(game: &mut Game) {
    unsafe {
        pt::advance_animation(&mut Runtime::new(game), pt::state::AnimationStep::Death);
        Samus_DrawDuringDeathAnim(game);
    }
}
pub unsafe fn Samus_DrawDuringDeathAnim(game: &mut Game) {
    unsafe {
        let v1: u16 = (2u16).wrapping_mul(word!(game.samus_pose));
        let a: u16 = (word!(game.samus_anim_frame)).wrapping_add(crate::rtl::rom_table_word!(
            game,
            0x929263,
            word!(game.samus_pose) as isize
        ));
        Samus_CalcSpritemapPos((2u16).wrapping_mul(word!(game.samus_pose)), game);
        DrawSamusSpritemap(
            a,
            (word!(game.layer1_x_pos)).wrapping_add(word!(game.samus_spritemap_x_pos)),
            (word!(game.layer1_y_pos)).wrapping_add(word!(game.samus_spritemap_y_pos)),
            game,
        );
        let R36: u16 = v1;
        if bottom_is_visible(game) & 1 != 0 {
            DrawSamusSpritemap(
                (word!(game.samus_anim_frame)).wrapping_add(crate::rtl::rom_table_word!(
                    game,
                    0x92945d,
                    (R36 >> 1) as isize
                )),
                (word!(game.layer1_x_pos)).wrapping_add(word!(game.samus_spritemap_x_pos)),
                (word!(game.layer1_y_pos)).wrapping_add(word!(game.samus_spritemap_y_pos)),
                game,
            );
        }
        SetSamusTilesDefsForCurAnim(game);
    }
}
pub unsafe fn Samus_DrawWhenNotAnimatingOrDying(game: &mut Game) {
    unsafe {
        let mut v0: PairU16 = PairU16 { k: 0, j: 0 };
        let v2: u16 = (2u16).wrapping_mul(word!(game.samus_pose));
        let a: u16 = (word!(game.samus_anim_frame)).wrapping_add(crate::rtl::rom_table_word!(
            game,
            0x929263,
            word!(game.samus_pose) as isize
        ));
        v0 = Samus_CalcSpritemapPos((2u16).wrapping_mul(word!(game.samus_pose)), game);
        DrawSamusSpritemap(a, v0.k, v0.j, game);
        let R36: u16 = v2;
        if bottom_is_visible(game) & 1 != 0 {
            DrawSamusSpritemap(
                (word!(game.samus_anim_frame)).wrapping_add(crate::rtl::rom_table_word!(
                    game,
                    0x92945d,
                    (R36 >> 1) as isize
                )),
                word!(game.samus_spritemap_x_pos),
                word!(game.samus_spritemap_y_pos),
                game,
            );
        }
        SetSamusTilesDefsForCurAnim(game);
    }
}
static kAtmosphericTypeFuncs: [Option<FuncXY_V>; 8] = callback_table! { unsafe FuncXY_V;
    None, AtmosphericTypeFunc_1_FootstepSplash, AtmosphericTypeFunc_1_FootstepSplash,
    AtmosphericTypeFunc_3_DivingSplash, AtmosphericTypeFunc_4_LavaSurfaceDmg,
    AtmosphericTypeFunc_5_Bubbles, AtmosphericTypeFunc_67_Dust, AtmosphericTypeFunc_67_Dust,
};
pub unsafe fn HandleAtmosphericEffects(game: &mut Game) {
    unsafe {
        for i in (0i32..=6).rev().step_by(2) {
            let v1: i32 = i >> 1;
            let mut v2: u16 = word!(game.atmospheric_gfx_frame_and_type[v1]);
            if v2 != 0 {
                let r18: u16 = (2u16).wrapping_mul((v2 as u8) as u16);
                let v3: u16 = (2u16).wrapping_mul((byte!(v2; 1)) as u16);
                let mut v4: i16 =
                    ((word!(game.atmospheric_gfx_anim_timer[v1])) as i16).wrapping_sub(1);
                word!(game.atmospheric_gfx_anim_timer[v1] = v4 as u16);
                let mut skip_effect: bool = if v4 != 0 {
                    (v4 < 0)
                        && ((v4 != (0x8000i32 as i16))
                            || ({
                                word!(
                                    game.atmospheric_gfx_anim_timer[v1] = *(RomPtr_90(
                                        (r18).wrapping_add(crate::rtl::rom_table_word!(
                                            game,
                                            0x908b93,
                                            (v3 >> 1) as isize,
                                        ),),
                                        game,
                                    )
                                        as *mut u16)
                                );
                                false
                            }))
                } else {
                    word!(
                        game.atmospheric_gfx_anim_timer[v1] = *(RomPtr_90(
                            (r18).wrapping_add(crate::rtl::rom_table_word!(
                                game,
                                0x908b93,
                                (v3 >> 1) as isize
                            ),),
                            game,
                        )
                            as *mut u16)
                    );
                    let v5: u16 = (word!(game.atmospheric_gfx_frame_and_type[v1])).wrapping_add(1);
                    word!(game.atmospheric_gfx_frame_and_type[v1] = v5);
                    (((v5 as u8) as i16).wrapping_sub(
                        (crate::rtl::rom_table_word!(game, 0x908bef, (v3 >> 1) as isize)) as i16,
                    ) >= 0)
                        && ({
                            word!(game.atmospheric_gfx_frame_and_type[v1] = 0);
                            true
                        })
                };
                if !(skip_effect) {
                    kAtmosphericTypeFuncs[(v3 >> 1) as usize].expect("non-null function pointer")(
                        v3, i as u16, game,
                    );
                }
            }
        }
    }
}
pub unsafe fn AtmosphericTypeFunc_1_FootstepSplash(k: u16, j: u16, game: &mut Game) {
    unsafe {
        let v2: i32 = j as i32 >> 1;
        let r18: u16 = (2u16).wrapping_mul(
            (byte!(@pointer (ram_ptr!(game, u16; 0xaec)).offset(v2 as isize); 0)) as u16,
        );
        let v3: u16 = word!(game.oam_next_ptr);
        let mut v4: i16 = (word!(game.atmospheric_gfx_x_pos[v2]) as i32
            - word!(game.layer1_x_pos) as i32
            - 4i32) as i16;
        if v4 >= 0
            && word!(game.atmospheric_gfx_x_pos[v2]) as i32
                - word!(game.layer1_x_pos) as i32
                - 260i32
                & 0x8000
                != 0
        {
            let mut v5: *mut OamEnt = gOamEnt(word!(game.oam_next_ptr) as i32, game);
            (*v5).xcoord = v4 as u8;
            let mut v6: i16 = (word!(game.atmospheric_gfx_y_pos[v2]) as i32
                - word!(game.layer1_y_pos) as i32
                - 4i32) as i16;
            if v6 >= 0
                && word!(game.atmospheric_gfx_y_pos[v2]) as i32
                    - word!(game.layer1_y_pos) as i32
                    - 260i32
                    & 0x8000
                    != 0
            {
                (*v5).ycoord = v6 as u8;
                *(&raw mut (*v5).charnum as *mut u16) = *(RomPtr_90(
                    (r18).wrapping_add(crate::rtl::rom_table_word!(
                        game,
                        0x908bff,
                        (k >> 1) as isize
                    )),
                    game,
                ) as *mut u16);
                word!(game.oam_next_ptr = (v3).wrapping_add(4));
            }
        }
    }
}
pub unsafe fn AtmosphericTypeFunc_Common(j: u16, a: u16, game: &mut Game) {
    unsafe {
        let v1: i32 = j as i32 >> 1;
        if (word!(game.atmospheric_gfx_y_pos[v1]) as i32 - word!(game.layer1_y_pos) as i32 & 0xff00)
            == 0
        {
            DrawSamusSpritemap(
                a,
                (word!(game.atmospheric_gfx_x_pos[v1])).wrapping_sub(word!(game.layer1_x_pos)),
                (word!(game.atmospheric_gfx_y_pos[v1])).wrapping_sub(word!(game.layer1_y_pos)),
                game,
            );
        }
    }
}
pub unsafe fn AtmosphericTypeFunc_3_DivingSplash(k: u16, j: u16, game: &mut Game) {
    unsafe {
        let v2: i32 = j as i32 >> 1;
        let r18: u16 = byte!(@pointer (ram_ptr!(game, u16; 0xaec)).offset(v2 as isize); 0) as u16;
        word!(game.atmospheric_gfx_y_pos[v2] = word!(game.fx_y_pos));
        AtmosphericTypeFunc_Common(j, (r18).wrapping_add(399), game);
    }
}
pub unsafe fn AtmosphericTypeFunc_4_LavaSurfaceDmg(k: u16, j: u16, game: &mut Game) {
    unsafe {
        let v2: i32 = j as i32 >> 1;
        let mut v3: u16 = if (j & 4) != 0 {
            (word!(game.atmospheric_gfx_x_pos[v2])).wrapping_sub(1)
        } else {
            (word!(game.atmospheric_gfx_x_pos[v2])).wrapping_add(1)
        };
        word!(game.atmospheric_gfx_x_pos[v2] = v3);
        word!(game.atmospheric_gfx_y_pos[(j >> 1)] -= 1);
        AtmosphericTypeFunc_1_FootstepSplash(k, j, game);
    }
}
pub unsafe fn AtmosphericTypeFunc_67_Dust(k: u16, j: u16, game: &mut Game) {
    unsafe {
        word!(game.atmospheric_gfx_y_pos[(j >> 1)] -= 1);
        AtmosphericTypeFunc_1_FootstepSplash(k, j, game);
    }
}
pub unsafe fn AtmosphericTypeFunc_5_Bubbles(k: u16, j: u16, game: &mut Game) {
    unsafe {
        let r18: u16 =
            byte!(@pointer (ram_ptr!(game, u16; 0xaec)).offset((j >> 1) as isize); 0) as u16;
        AtmosphericTypeFunc_Common(j, (r18).wrapping_add(390), game);
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

crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C, packed)]
    pub struct SamusTileAnimationDefs {
        pub top_half_idx, top_half_pos, bottom_half_idx, bottom_half_pos: u8;
    }
}
#[inline]
unsafe fn get_SamusTileAnimationDefs(a: u16, game: &mut Game) -> *mut SamusTileAnimationDefs {
    unsafe { RomPtr((0x920000u32) | ((a) as u32), game) as *mut SamusTileAnimationDefs }
}
pub unsafe fn SetSamusTilesDefsForCurAnim(game: &mut Game) {
    unsafe {
        let v0: u16 = (4 * word!(game.samus_anim_frame) as i32
            + crate::rtl::rom_table_word!(game, 0x92d94e, word!(game.samus_pose) as isize) as i32)
            as u16;
        let AD = get_SamusTileAnimationDefs(v0, game);
        word!(
            game.nmi_copy_samus_top_half_src = (7 * (*AD).top_half_pos as i32
                + crate::rtl::rom_table_word!(game, 0x92d91e, (*AD).top_half_idx as isize) as i32)
                as u16
        );
        byte!(@pointer ram_ptr!(game, u16; 0x71d); 0) = 1;
        if (*AD).bottom_half_idx != 255 {
            word!(
                game.nmi_copy_samus_bottom_half_src = (7 * (*AD).bottom_half_pos as i32
                    + crate::rtl::rom_table_word!(game, 0x92d938, (*AD).bottom_half_idx as isize)
                        as i32) as u16
            );
            byte!(@pointer ram_ptr!(game, u16; 0x71d); 1) = 1;
        }
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
