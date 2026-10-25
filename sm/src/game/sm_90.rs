use crate::game::player_movement::{self, forced};
use crate::game::player_transitions as pt;
use crate::game::player_update::{self, BetaMode, MovementStep};

use crate::game::player_projectiles::initialize;
pub use crate::game::prelude::*;
use crate::game::projectile_initialization::Initialization;
use crate::game::samus_animation::{
    PlaySamusFanfare, Samus_Draw, Samus_DrawEchoes, Samus_DrawShinesparkCrashEchoes,
    Samus_DrawWhenNotAnimatingOrDying,
};
use crate::game::session_runtime::Runtime;
use crate::game::{
    player_collision::operations as player_collision, session_runtime::Runtime as CollisionRuntime,
};

pub use crate::game::constants::bank_8d::*;
pub use crate::game::constants::bank_60::*;
pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_90::*;

use crate::game::enemy_projectiles::EnableEprojs;
use crate::game::explosion_window::spawn_crystal_flash;
use crate::game::gameplay_flag_runtime::CheckEventHappened;
use crate::game::palette_effects;
use crate::game::player_projectiles::draw_projectiles;
use crate::game::sm_9b::{GrappleBeamFunc_BF1B, UpdateGrappleBeamTilesAndIncrFlameCtr};
use crate::game::sm_80::{
    DrawTimer, HandleAutoscrolling_X, HandleAutoscrolling_Y,
    HandleScrollingWhenTriggeringScrollDown, HandleScrollingWhenTriggeringScrollLeft,
    HandleScrollingWhenTriggeringScrollRight, HandleScrollingWhenTriggeringScrollUp,
    InvalidInterrupt_Crash, ProcessTimer, QueueSfx1_Max6, QueueSfx3_Max6, QueueSfx3_Max9,
    QueueSfx3_Max15,
};

use crate::game::grapple_swing::draw_rope as HandleGrappleBeamGfx;
use crate::game::projectile_geometry::sine_offset;
use crate::game::session_runtime::xray_input;
use crate::game::sm_88::EnableHdmaObjects;

pub use crate::types::HandlerFunc;

crate::game::constants::typed_constants! { i32;
    true_0 = 1, false_0 = 0,
    fnSamus_MovementHandler_Normal = 0x90a337, fnProjPreInstr_Beam_NoWaveBeam = 0x90aef3,
    fnProjPreInstr_WavePlasmaEtc = 0x90b0c3,
    fnProjPreInstr_BeamOrIceWave = 0x90b0e4,
    fnProjPreInstr_Empty = 0x90b169,
    fnSamus_MoveHandlerVerticalShinespark = 0x90d0ab,
    fnSamus_Func3 = 0x90e0e6,
    fnSamus_Func9 = 0x90e41b, fnSamus_FrameHandlerAlfa_Func11 = 0x90e695,
    fnSamus_FrameHandlerAlfa_Func13 = 0x90e713, fnSamus_FrameHandlerBeta_Func17 = 0x90e725,
    fnEmptyFunction = 0x90e8cd,
    fnj_HandleDemoRecorder_2_0 = 0x90e8d9,
    fnSetContactDamageIndexAndUpdateMinimap = 0x90e8dc,
    fnnullsub_152 = 0x90e90e,
    fnSamus_InputHandler_E913 = 0x90e913,
    fnSamus_InputHandler_E91D = 0x90e91d, fnHandleAutoJumpHack = 0x90e926,
    fnSamusDrawHandler_Default = 0x90eb52,
    fnGrappleBeamFunc_Inactive = 0x9bc4f0, fnGrappleBeamFunc_Firing = 0x9bc703,
    fnGrappleBeamFunc_ConnectedLockedInPlace = 0x9bc77e,
    fnGrappleBeamFunc_Cancel = 0x9bc856, fnGrappleBeam_Func2 = 0x9bc8c5,
}
crate::game::constants::typed_constants! { u32;
    fnSamus_MoveHandler_ReleaseFromGrapple = 9475182,
    fnSamus_ScrollFinishedHook_SporeSpawnFight = 9475465,
    fnSamus_MoveHandler_Shinespark_Diag = 9490647, fnSamus_MoveHandler_Shinespark_Horiz = 9490694,
    fnSamus_MoveHandler_Knockback = 9494328,
    fnSamus_MoveHandler_BombJumpStart = 9494565,
    fnSamus_FrameHandlerAlfa_Func12 = 9496265,
    fnSamus_FrameHandlerBeta_Func14 = 9496565, fnSamus_Func15 = 9496627,
    fnSamus_Func16 = 9496682,
    fnSamus_Func20_ = 9496856,
    fnSamusMovementType_Xray = 9496911,
    fnsub_90EB86 = 9497478,
}

pub unsafe fn Samus_BombJumpRisingYMovement_(game: &mut Game) {
    unsafe {
        if word!(game.samus_y_dir) == 1 {
            if (word!(game.samus_y_speed) & 0x8000) != 0 {
                word! { game, 0; samus_y_subspeed, samus_y_speed }
                word!(game.samus_y_dir = 2);
                if (word!(game.bomb_jump_dir) as u8) != 2 {
                    word!(game.samus_x_accel_mode = 2);
                }
            } else if word!(game.samus_y_speed) as i32 - 1i32 & 0x8000 != 0
                && word!(game.samus_input_handler) as i32 != 0x90e91d & 0xffff
            {
                word!(game.samus_input_handler = (0x90e913i32) as u16);
            }
        }
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallScrollingFinishedHook(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [9475465] Samus_ScrollFinishedHook_SporeSpawnFight [] {
        if word!(game.layer1_y_pos) <= 0x1d0 {
            word!(game.layer1_y_pos = 464);
        }
    };
}
pub unsafe fn MainScrollingRoutine(game: &mut Game) {
    unsafe {
        if word!(game.slow_grabble_scrolling_flag) != 0 {
            if (word!(game.samus_x_pos) & 0x8000) == 0 {
                let mut v0: u16 = (word!(game.samus_x_pos)).wrapping_sub(word!(game.layer1_x_pos));
                if word!(game.samus_x_pos) < word!(game.layer1_x_pos) || v0 < 0x60 {
                    word!(game.layer1_x_pos -= 3);
                } else if v0 >= 0xa0 {
                    word!(game.layer1_x_pos += 3);
                }
                if (word!(game.samus_y_pos) & 0x8000) == 0 {
                    let v1: u16 = (word!(game.samus_y_pos)).wrapping_sub(word!(game.layer1_y_pos));
                    if word!(game.samus_y_pos) < word!(game.layer1_y_pos) || v1 < 0x70 {
                        word!(game.layer1_y_pos -= 3);
                    } else if v1 >= 0x90 {
                        word!(game.layer1_y_pos += 3);
                    }
                }
            }
            HandleAutoscrolling_X(game);
            HandleAutoscrolling_Y(game);
        } else {
            Samus_CalcDistanceMoved_X(game);
            Samus_HandleScroll_X(game);
            Samus_CalcDistanceMoved_Y(game);
            Samus_HandleScroll_Y(game);
        }
        if word!(game.scrolling_finished_hook) != 0 {
            CallScrollingFinishedHook(
                ((word!(game.scrolling_finished_hook)) as u32) | (0x900000u32),
                game,
            );
        }
        word!(game.samus_prev_x_pos = word!(game.samus_x_pos));
        word!(game.samus_prev_x_subpos = word!(game.samus_x_subpos));
        word!(game.samus_prev_y_pos = word!(game.samus_y_pos));
        word!(game.samus_prev_y_subpos = word!(game.samus_y_subpos));
    }
}
macro_rules! chase_ideal_layer_position {
    (
        $game:ident, $ideal_position:ident, $position:ident, $subposition:ident,
        $movement:ident, $movement_fraction:ident, $scroll_negative:path,
        $scroll_positive:path $(,)?
    ) => {
        if word!($game.$ideal_position) != word!($game.$position) {
            if ((word!($game.$ideal_position)) as i16).wrapping_sub((word!($game.$position)) as i16)
                < 0
            {
                words!(
                    $game.$position,
                    $subposition += -(words!(i32, $game.$movement, $movement_fraction)) as u32
                );
                $scroll_negative($game);
            } else {
                words!(
                    $game.$position,
                    $subposition += words!($game.$movement, $movement_fraction)
                );
                $scroll_positive($game);
            }
        }
    };
}

pub unsafe fn Samus_HandleScroll_X(game: &mut Game) {
    unsafe {
        static kSamus_HandleScroll_X_FaceLeft: [u16; 4] = [0xa0, 0x50, 0x20, 0xe0];
        static kSamus_HandleScroll_X_FaceRight: [u16; 4] = [0x60, 0x40, 0x20, 0xe0];
        if word!(game.samus_prev_x_pos) == word!(game.samus_x_pos) {
            HandleAutoscrolling_X(game);
            return;
        }
        if (word!(game.knockback_dir) != 0
            || *ram_ptr!(game; 0xa1f) == 16
            || word!(game.samus_x_accel_mode) == 1) as i32
            ^ (*ram_ptr!(game; 0xa1e) != 4) as i32
            != 0
        {
            word!(
                game.ideal_layer1_xpos = (word!(game.samus_x_pos)).wrapping_sub(
                    kSamus_HandleScroll_X_FaceRight
                        [(word!(game.camera_distance_index) >> 1) as usize],
                )
            );
        } else {
            word!(
                game.ideal_layer1_xpos = (word!(game.samus_x_pos)).wrapping_sub(
                    kSamus_HandleScroll_X_FaceLeft
                        [(word!(game.camera_distance_index) >> 1) as usize],
                )
            );
        }
        chase_ideal_layer_position!(
            game,
            ideal_layer1_xpos,
            layer1_x_pos,
            layer1_x_subpos,
            absolute_moved_last_frame_x,
            absolute_moved_last_frame_x_fract,
            HandleScrollingWhenTriggeringScrollLeft,
            HandleScrollingWhenTriggeringScrollRight
        );
    }
}
pub unsafe fn Samus_HandleScroll_Y(game: &mut Game) {
    unsafe {
        if word!(game.samus_prev_y_pos) == word!(game.samus_y_pos) {
            HandleAutoscrolling_Y(game);
        } else {
            if word!(game.samus_y_dir) == 1 {
                word!(
                    game.ideal_layer1_ypos =
                        (word!(game.samus_y_pos)).wrapping_sub(word!(game.down_scroller))
                );
            } else {
                word!(
                    game.ideal_layer1_ypos =
                        (word!(game.samus_y_pos)).wrapping_sub(word!(game.up_scroller))
                );
            }
            chase_ideal_layer_position!(
                game,
                ideal_layer1_ypos,
                layer1_y_pos,
                layer1_y_subpos,
                absolute_moved_last_frame_y,
                absolute_moved_last_frame_y_fract,
                HandleScrollingWhenTriggeringScrollUp,
                HandleScrollingWhenTriggeringScrollDown
            );
        };
    }
}
pub unsafe fn Samus_CalcDistanceMoved_X(game: &mut Game) {
    unsafe {
        if ((word!(game.samus_x_pos)) as i16).wrapping_sub((word!(game.samus_prev_x_pos)) as i16)
            >= 0
        {
            words!(
                game.absolute_moved_last_frame_x,
                absolute_moved_last_frame_x_fract = (words!(game.samus_x_pos, samus_x_subpos))
                    .wrapping_sub(words!(game.samus_prev_x_pos, samus_prev_x_subpos),)
                    .wrapping_add(((1i32) << 16) as u32)
            );
        } else {
            words!(
                game.absolute_moved_last_frame_x,
                absolute_moved_last_frame_x_fract =
                    (words!(game.samus_prev_x_pos, samus_prev_x_subpos))
                        .wrapping_sub(words!(game.samus_x_pos, samus_x_subpos),)
                        .wrapping_add(((1i32) << 16) as u32)
            );
        };
    }
}
pub unsafe fn Samus_CalcDistanceMoved_Y(game: &mut Game) {
    unsafe {
        if ((word!(game.samus_y_pos)) as i16).wrapping_sub((word!(game.samus_prev_y_pos)) as i16)
            >= 0
        {
            words!(
                game.absolute_moved_last_frame_y,
                absolute_moved_last_frame_y_fract = (words!(game.samus_y_pos, samus_y_subpos))
                    .wrapping_sub(words!(game.samus_prev_y_pos, samus_prev_y_subpos),)
                    .wrapping_add(((1i32) << 16) as u32)
            );
        } else {
            words!(
                game.absolute_moved_last_frame_y,
                absolute_moved_last_frame_y_fract =
                    (words!(game.samus_prev_y_pos, samus_prev_y_subpos))
                        .wrapping_sub(words!(game.samus_y_pos, samus_y_subpos),)
                        .wrapping_add(((1i32) << 16) as u32)
            );
        };
    }
}
pub unsafe fn Samus_EndSuperJump(game: &mut Game) -> u8 {
    unsafe {
        if word!(game.samus_health) as i32 - 30i32 & 0x8000 == 0
            && word!(game.samus_collision_flag) == 0
        {
            return 0;
        }
        if *ram_ptr!(game; 0xa1e) == 4 {
            word! {game; speed_echo_xspeed[0] = 32; speed_echo_xspeed[1] = 160;
            speed_echo_xpos[2] = 4; }
        } else {
            word! {game; speed_echo_xspeed[0] = 224; speed_echo_xspeed[1] = 96;
            speed_echo_xpos[2] = -4i32 as u16; }
        }
        word! { game, 0;
            samus_y_subspeed, samus_y_speed, samus_x_extra_run_speed, samus_x_extra_run_subspeed,
            speed_boost_counter, UNUSED_word_7E0B1A, samus_y_dir,
        }
        word! {game; samus_movement_handler = ((0x90d346i32) as u16);
        samus_draw_handler = ((0x90ebf3i32) as u16); speed_echoes_index = 0;
        speed_echo_xpos[0] = word!(game.samus_x_pos);
        speed_echo_xpos[1] = word!(game.samus_x_pos);
        speed_echo_ypos[0] = word!(game.samus_y_pos);
        speed_echo_ypos[1] = word!(game.samus_y_pos); }
        word! { game, 0; speed_echo_xspeed[2], samus_hurt_flash_counter }
        QueueSfx1_Max6(0x35, game);
        QueueSfx3_Max6(0x10, game);
        1
    }
}
static kSamus_MoveHandler_ShinesparkCrash: [Func_V; 3] = callback_table! { nonnull Func_V;
    Samus_MoveHandler_ShinesparkCrash_0, Samus_MoveHandler_ShinesparkCrash_1,
    Samus_MoveHandler_ShinesparkCrash_2,
};
pub unsafe fn Hdmaobj_CrystalFlash(game: &mut Game) -> u8 {
    unsafe {
        let mut v1: SamusPose = kPose_00_FaceF_Powersuit;
        if word!(game.game_state) as i32 - 40i32 & 0x8000 != 0
            && word!(game.joypad1_lastkeys) != word!(game.button_config_shoot_x) | 0x430
        {
            return 1;
        }
        if word!(game.samus_y_speed) != 0
            || word!(game.samus_y_subspeed) != 0
            || word!(game.samus_health) as i32 - 51i32 & 0x8000 == 0
            || word!(game.samus_reserve_health) != 0
            || word!(game.samus_missiles) as i32 - 10i32 & 0x8000 != 0
            || word!(game.samus_super_missiles) as i32 - 10i32 & 0x8000 != 0
            || word!(game.samus_power_bombs) as i32 - 10i32 & 0x8000 != 0
        {
            return 1;
        }
        v1 = (if *ram_ptr!(game; 0xa1e) == 4 {
            kPose_D4_FaceL_CrystalFlash as i32
        } else {
            kPose_D3_FaceR_CrystalFlash as i32
        }) as SamusPose;
        word!(game.samus_pose = v1 as u16);
        pt::refresh_pose(&mut Runtime::new(game));
        if *ram_ptr!(game; 0xa1f) != 27 {
            return 1;
        }
        pt::record_pose_history(Runtime::new(game).session_mut());
        word!(game.samus_movement_handler = (0x90d678i32) as u16);
        if word!(game.samus_input_handler) as i32 != 0x90e91d & 0xffff {
            word!(game.samus_input_handler = (0x90e90ei32) as u16);
        }
        word! {game; timer_for_shinesparks_startstop = 9; which_item_to_pickup = 0; substate = 10;
        suit_pickup_light_beam_pos = 0; }
        *(ram_ptr!(game, u16; 0xdf0)) = 0;
        word! {game; timer_for_shine_timer = 7; special_samus_palette_frame = 0;
        samus_shine_timer = 1; }
        *(ram_ptr!(game, u16; 0xdf2)) = 1;
        word! { game, 0; samus_invincibility_timer, samus_knockback_timer, knockback_dir }
        0
    }
}
static kSamusMoveHandler_CrystalFlashMainFuncs: [Func_V; 3] = callback_table! { nonnull Func_V;
    SamusMoveHandler_CrystalFlashMain_0, SamusMoveHandler_CrystalFlashMain_1,
    SamusMoveHandler_CrystalFlashMain_2,
};
pub unsafe fn SamusMoveHandler_CrystalFlashMain_0(game: &mut Game) {
    unsafe {
        if (word!(game.nmi_frame_counter_word) & 7) == 0 {
            word!(game.samus_missiles -= 1);
            crate::game::item_pickups_runtime::restore(
                crate::game::item_pickups::Resource::Health,
                0x32,
                game,
            );
            let v0: bool = ((word!(game.substate)) as i16).wrapping_sub(1) < 0;
            if word!(game.substate -= 1, reread) == 0 || v0 {
                word!(game.substate = 10);
                word!(game.which_item_to_pickup += 1);
            }
        }
    }
}
pub unsafe fn SamusMoveHandler_CrystalFlashMain_1(game: &mut Game) {
    unsafe {
        if (word!(game.nmi_frame_counter_word) & 7) == 0 {
            word!(game.samus_super_missiles -= 1);
            crate::game::item_pickups_runtime::restore(
                crate::game::item_pickups::Resource::Health,
                0x32,
                game,
            );
            let v0: bool = ((word!(game.substate)) as i16).wrapping_sub(1) < 0;
            if word!(game.substate -= 1, reread) == 0 || v0 {
                word!(game.substate = 10);
                word!(game.which_item_to_pickup += 1);
            }
        }
    }
}
pub unsafe fn SamusMoveHandler_CrystalFlashMain_2(game: &mut Game) {
    unsafe {
        if (word!(game.nmi_frame_counter_word) & 7) == 0 {
            word!(game.samus_power_bombs -= 1);
            crate::game::item_pickups_runtime::restore(
                crate::game::item_pickups::Resource::Health,
                0x32,
                game,
            );
            let v0: bool = ((word!(game.substate)) as i16).wrapping_sub(1) < 0;
            if word!(game.substate -= 1, reread) == 0 || v0 {
                word! {game; samus_movement_handler = ((0x90d75bi32) as u16);
                samus_draw_handler = ((0x90eb52i32) as u16); samus_anim_frame_timer = 3;
                samus_anim_frame = 12; }
            }
        }
    }
}
static kSamus_MoveHandler_Knockback: [Func_V; 6] = callback_table! { nonnull Func_V;
    Samus_MoveHandler_Knockback_0, Samus_MoveHandler_Knockback_Up,
    Samus_MoveHandler_Knockback_Up, Samus_MoveHandler_Knockback_3,
    Samus_MoveHandler_Knockback_Down, Samus_MoveHandler_Knockback_Down,
};
static kSamus_MoveHandler_BombJumpMain: [Func_V; 4] = callback_table! { nonnull Func_V;
    Samus_MoveHandler_Knockback_0, Samus_HorizontalBombJump, Samus_VerticalBombJump,
    Samus_HorizontalBombJump,
};
pub unsafe fn Samus_HorizontalBombJump(game: &mut Game) {
    unsafe {
        crate::verify_probe::hit("samus.bomb.horizontal");
        player_movement::speed::bomb_rising_x(&mut Runtime::new(game));
        Samus_VerticalBombJump(game);
    }
}
pub unsafe fn Samus_VerticalBombJump(game: &mut Game) {
    unsafe {
        Samus_BombJumpRisingYMovement_(game);
        if word!(game.samus_y_dir) == 2 || {
            player_movement::speed::move_y_with_speed(&mut Runtime::new(game));
            word!(game.samus_collision_flag) != 0
        } {
            Samus_MoveHandler_BombJumpFunc1(game);
        }
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallFrameHandlerGamma(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [9494683] Samus_Func1 [] {
        if word!(game.samus_pose) as i32 == kPose_E9_FaceL_Drained_CrouchFalling as i32
            && word!(game.samus_anim_frame) as i32 - 8i32 & 0x8000 == 0
            && (word!(game.joypad1_newkeys) as i32 & kButton_Up as i32) != 0
        {
            word! {game; samus_anim_frame_timer = 1; samus_anim_frame = 13;
                frame_handler_gamma = ((0x90e90ei32) as u16); }
        }
    };
    [9494725] Samus_Func2 [] {
        if word!(game.samus_anim_frame) as i32 - 8i32 & 0x8000 == 0
            && word!(game.samus_anim_frame) as i32 - 12i32 & 0x8000 != 0
            && (word!(game.joypad1_newkeys) as i32 & kButton_Up as i32) != 0
        {
            word! {game; samus_anim_frame_timer = 1; samus_anim_frame = 18; }
        }
    };
    [9494758] Samus_Func3 [] {
        if ProcessTimer(game) & 1 != 0 {
            word!(game.game_state = kGameState_35_TimeUp as u16);
            word_transfers!(game, descending, 256; le(target_palettes, 255) <- value(0x7fff));
            word!(game.frame_handler_gamma = (0x90e114i32) as u16);
            palette_effects::set_enabled(false, game);
        }
        if word!(game.timer_status) != 0 {
            DrawTimer(game);
        }
    };
    [9494804] DrawTimer_ [] {
        DrawTimer(game);
    };
    [9495201] Samus_GrabbedByDraygonFrameHandler [] {
        if word!(game.grapple_beam_function) as i32 == 0x9bc77e & 0xffff {
            word! {game; samus_new_pose = -1i32 as u16; samus_momentum_routine_index = 0; }
        }
        if word!(game.joypad1_newkeys) as i32
            & (kButton_Up as i32 | kButton_Down as i32 | kButton_Left as i32 | kButton_Right as i32)
            != 0
            && word!(game.joypad1_newkeys) as i32
                & (kButton_Up as i32
                    | kButton_Down as i32
                    | kButton_Left as i32
                    | kButton_Right as i32)
                != word!(game.suit_pickup_light_beam_pos) as i32
        {
            word!(
                game.suit_pickup_light_beam_pos = (word!(game.joypad1_newkeys) as i32
                    & (kButton_Up as i32
                        | kButton_Down as i32
                        | kButton_Left as i32
                        | kButton_Right as i32))
                    as u16
            );
            if ((word!(game.substate += 1, reread)) as i16).wrapping_sub(60) >= 0 {
                Samus_ReleaseFromDraygon(game);
            }
        }
    };
    @external {
        9494830 => [crate::game::session_runtime::ceres_movement(game, |h| forced::push_out_of_ceres(h))];
        9494984 => [crate::game::session_runtime::ceres_movement(game, |h| forced::advance_ceres_push(h))];
        9495459 => [forced::finish_forced_fall(&mut Runtime::new(game))];
        9495579 => [forced::animate_fall(Runtime::new(game).session_mut())];
        9495422 => [()];
        9496846 => [()];
    }
}

pub unsafe fn Samus_SetGrabbedByDraygonPose(a: u16, game: &mut Game) {
    unsafe {
        if (a & 1) != 0 {
            word!(game.samus_pose = kPose_EC_FaceR_Draygon_NoMove_NoAim as u16);
        } else {
            word!(game.samus_pose = kPose_BA_FaceL_Draygon_NoMove_NoAim as u16);
        }
        pt::refresh_pose(&mut Runtime::new(game));
        word!(game.frame_handler_gamma = (0x90e2a1i32) as u16);
        word!(game.samus_movement_handler = (0x90e90ei32) as u16);
        word! { game, 0; substate, suit_pickup_light_beam_pos }
        *(ram_ptr!(game, u16; 0xdf0)) = 0;
        pt::record_pose_history(Runtime::new(game).session_mut());
        word! {game; samus_new_pose = -1i32 as u16; samus_new_pose_interrupted = -1i32 as u16;
        samus_new_pose_transitional = -1i32 as u16; }
        word! { game, 0;
            samus_momentum_routine_index, samus_special_transgfx_index, samus_hurt_switch_index,
        }
    }
}
pub unsafe fn Samus_ReleaseFromDraygon_(game: &mut Game) {
    unsafe {
        Samus_ReleaseFromDraygon(game);
    }
}
pub unsafe fn Samus_ReleaseFromDraygon(game: &mut Game) {
    unsafe {
        if *ram_ptr!(game; 0xa1e) == 4 {
            word!(game.samus_pose = kPose_02_FaceL_Normal as u16);
        } else {
            word!(game.samus_pose = kPose_01_FaceR_Normal as u16);
        }
        pt::refresh_pose(&mut Runtime::new(game));
        word!(game.samus_movement_handler = (0x90a337i32) as u16);
        word!(game.frame_handler_gamma = (0x90e90ei32) as u16);
        pt::record_pose_history(Runtime::new(game).session_mut());
        word! { game, 0; samus_x_base_speed, samus_x_base_subspeed }
        word! {game; samus_new_pose = -1i32 as u16; samus_new_pose_interrupted = -1i32 as u16;
        samus_new_pose_transitional = -1i32 as u16; }
        word! { game, 0;
            samus_momentum_routine_index, samus_special_transgfx_index, samus_hurt_switch_index,
            samus_y_speed, samus_y_subspeed, samus_y_dir, used_for_ball_bounce_on_landing,
            samus_x_accel_mode,
        }
        word!(game.samus_grapple_flags = word!(game.samus_grapple_flags) & 0xfffd | 2);
    }
}
pub(crate) use crate::game::player_projectiles::prepare as HandleControllerInputForGamePhysics;
// Special movement stays RAM-authoritative until its retained callback returns.
unsafe fn advance_player_beta(
    mode: BetaMode,
    game: &mut Game,
) -> Result<(), crate::game::samus_periodic_damage::InvalidDamage> {
    unsafe {
        word!(game.samus_contact_damage_index = 0);
        let handler = word!(game.samus_movement_handler);
        let movement = if handler == 0xa337 {
            MovementStep::Normal
        } else {
            CallSamusMovementHandler(u32::from(handler) | 0x900000, game);
            MovementStep::AlreadyAdvanced
        };
        player_update::advance(&mut Runtime::new(game), mode, movement)
    }
}

crate::game::callback_dispatch! {
    pub unsafe fn CallFrameHandlerBeta(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [9496357] Samus_FrameHandlerBeta_Func17 [] {
        let damage_result = advance_player_beta(BetaMode::Gameplay, game);
        #[cfg(sm_verify_dll)]
        {
            assert!(
                !game.actor_identity_is_with_session(),
                "player settlement must return RAM authority"
            );
            crate::verify_probe::hit("player.settlement.caller17.return");
        }
        if damage_result.is_err() {
            InvalidInterrupt_Crash(game);
        }
        Samus_PauseCheck(game);
        Samus_LowHealthCheck_(game);
    };
    [9496530] HandleDemoRecorder_3 [] {
        if (word!(game.joypad2_new_keys) & 0x80) != 0 {
            word!(game.frame_handler_alfa = (0x90e695i32) as u16);
            word!(game.frame_handler_beta = (0x90e725i32) as u16);
            EnableEprojs(game);
            word!(game.time_is_frozen_flag = 0);
        }
    };
    [9496565] Samus_FrameHandlerBeta_Func14 [] {
        let _ = advance_player_beta(BetaMode::DemoMapped, game);
        word! {game; joypad1_lastkeys = word!(game.controller1_input_for_demo);
            joypad1_newkeys = word!(game.controller1_new_input_for_demo);
            joypad1_input_samusfilter = word!(game.demo_backup_prev_controller_input);
            joypad1_newinput_samusfilter = word!(game.demo_backup_prev_controller_input_new); }
    };
    [9496627] Samus_Func15 [] {
        let _ = advance_player_beta(BetaMode::DemoUnmapped, game);
        word! {game; joypad1_lastkeys = word!(game.controller1_input_for_demo);
            joypad1_newkeys = word!(game.controller1_new_input_for_demo);
            joypad1_input_samusfilter = word!(game.demo_backup_prev_controller_input);
            joypad1_newinput_samusfilter = word!(game.demo_backup_prev_controller_input_new); }
    };
    [9496682] Samus_Func16 [] {
        Samus_SetRadius(game);
        {
            let mut runtime = Runtime::new(game);
            player_update::minimap::update(&mut runtime);
            pt::advance_animation(&mut runtime, pt::state::AnimationStep::Gameplay);
        }
        word! {game; elevator_status = 0; samus_prev_y_pos = word!(game.samus_y_pos); }
        if PlaySamusFanfare(game) & 1 != 0 {
            if word!(game.debug_invincibility) as i32 - 7i32 & 0x8000 != 0
                || (word!(game.joypad2_last) & 0x8000) == 0
            {
                word!(game.debug_invincibility = 0);
            }
            word!(game.frame_handler_alfa = (0x90e695i32) as u16);
            word!(game.frame_handler_beta = (0x90e725i32) as u16);
        }
    };
    [9496746] Samus_Func18 [] {
        Samus_FrameHandlerBeta_Func17(game);
        if word!(game.frame_handler_gamma) as i32 == 0x90e114 & 0xffff
            && word!(game.game_state) as i32 != kGameState_35_TimeUp as i32
        {
            word!(game.game_state = kGameState_35_TimeUp as u16);
        }
    };
    [9496796] SetContactDamageIndexAndUpdateMinimap [] {
        word!(game.samus_contact_damage_index = 0);
        player_update::minimap::update(&mut Runtime::new(game));
    };
    [9496812] Samus_Func19 [] {
        let _ = advance_player_beta(BetaMode::AnimationOnly, game);
    };
    [9496834] Samus_LowHealthCheck [] {
        Samus_LowHealthCheck_(game);
    };
    @external {
        9496781 => [()]; 9496790 => [()]; 9496793 => [()];
    }
}
pub unsafe fn HandleSamusMovementAndPause(game: &mut Game) {
    unsafe {
        CallFrameHandlerBeta(
            ((word!(game.frame_handler_beta)) as u32) | (0x900000u32),
            game,
        );
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallSamusInputHandler(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [9496861] Samus_InputHandler_E91D [] {
        Runtime::new(game).handle_demo_input();
    };
    [9496870] HandleAutoJumpHack [] {
        let v0: u16 = word!(game.joypad1_newkeys);
        if word!(game.autojump_timer) != 0 && word!(game.autojump_timer) as i32 - 9i32 & 0x8000 != 0
        {
            word!(game.joypad1_newkeys |= (word!(game.button_config_jump_a)));
            word!(game.autojump_timer = 0);
        }
        Runtime::new(game).handle_input();
        word! {game; joypad1_newkeys = v0; samus_input_handler = ((0x90e913i32) as u16); }
    };
    @forward [pub unsafe fn (game: &mut Game)];
    Samus_CallInputHandler => CallSamusInputHandler(
        ((word!(game.samus_input_handler)) as u32) | (0x900000u32),
        game,
    );
    [9496851] Samus_InputHandler_E913 => Runtime::new(game).handle_input();
    [9496856] Samus_Func20_ => xray_input(game);
    @external {
        9496846 => [()];
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallSamusMovementHandler(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [9475182] Samus_MoveHandler_ReleaseFromGrapple [] {
        if word!(game.samus_y_dir) == 1 && (word!(game.samus_y_speed) & 0x8000) != 0 {
            word! { game, 0; samus_y_subspeed, samus_y_speed }
            word! {game; samus_y_dir = 2; samus_movement_handler = ((0x90a337i32) as u16); }
        }
        word!(game.samus_x_accel_mode = 2);
        {
            let mut runtime = Runtime::new(game);
            let profile = player_movement::speed::grapple_speed_profile(&mut runtime);
            let amount = player_movement::speed::base_speed(&mut runtime, profile);
            let request = player_movement::speed::horizontal_air::<false>(&mut runtime, amount);
            player_collision::horizontal_control(&mut runtime, request);
            player_movement::speed::move_y_with_speed(&mut runtime);
        }
        if word!(game.input_to_pose_calc) != 0 {
            word!(game.samus_movement_handler = (0x90a337i32) as u16);
        }
    };
    [9475275] Samus_HandleMovement_DrainedCrouching [] {
        player_movement::speed::move_y_with_speed(&mut Runtime::new(game));
        if word!(game.samus_collision_flag) != 0 {
            word! {game; samus_movement_handler = ((0x90a337i32) as u16);
                samus_anim_frame_timer = 8; samus_anim_frame = 7; }
            word! { game, 0; samus_y_subspeed, samus_y_speed }
        }
    };
    [9478967] Samus_MovementHandler_Normal [] {
        player_movement::advance_normal(&mut Runtime::new(game));
    };
    [9490536] Samus_MoveHandlerShinesparkWindup [] {
        let v0: bool = (word!(game.timer_for_shinesparks_startstop -= 1, reread) & 0x8000)
            != 0;
        if word!(game.timer_for_shinesparks_startstop) == 0 || v0 {
            if *ram_ptr!(game; 0xa1e) == 4 {
                word!(game.samus_new_pose_interrupted = 204);
            } else {
                word!(game.samus_new_pose_interrupted = 203);
            }
            word!(game.samus_movement_handler = (0x90d0abi32) as u16);
            word!(game.samus_input_handler = (0x90e90ei32) as u16);
            word! { game, 0;
                speed_echoes_index, speed_echo_xspeed[0], speed_echo_xspeed[1], speed_echo_xpos[0],
                speed_echo_xpos[1],
            }
            QueueSfx3_Max9(0xf, game);
        }
    };
    [9490603] Samus_MoveHandlerVerticalShinespark [] {
        word! {game; samus_contact_damage_index = 2; samus_hurt_flash_counter = 8; }
        player_movement::record_echo(&mut Runtime::new(game));
        player_collision::shinespark(&mut CollisionRuntime::new(game), player_collision::Axis::Y);
        Samus_EndSuperJump(game);
        if word!(game.samus_health) as i32 - 30i32 & 0x8000 == 0 && {
            (word!(game.samus_health -= 1, reread) & 0x8000) != 0
        } {
            word!(game.samus_health = 0);
        }
    };
    [9490647] Samus_MoveHandler_Shinespark_Diag [] {
        word! {game; samus_contact_damage_index = 2; samus_hurt_flash_counter = 8; }
        player_movement::record_echo(&mut Runtime::new(game));
        player_collision::shinespark(&mut CollisionRuntime::new(game), player_collision::Axis::X);
        player_collision::shinespark(&mut CollisionRuntime::new(game), player_collision::Axis::Y);
        Samus_EndSuperJump(game);
        if word!(game.samus_health) as i32 - 30i32 & 0x8000 == 0 && {
            (word!(game.samus_health -= 1, reread) & 0x8000) != 0
        } {
            word!(game.samus_health = 0);
        }
    };
    [9490694] Samus_MoveHandler_Shinespark_Horiz [] {
        word! {game; samus_contact_damage_index = 2; samus_hurt_flash_counter = 8; }
        player_movement::record_echo(&mut Runtime::new(game));
        player_collision::shinespark(&mut CollisionRuntime::new(game), player_collision::Axis::X);
        Samus_EndSuperJump(game);
        if word!(game.samus_health) as i32 - 30i32 & 0x8000 == 0 && {
            (word!(game.samus_health -= 1, reread) & 0x8000) != 0
        } {
            word!(game.samus_health = 0);
        }
    };
    [9491270] Samus_MoveHandler_ShinesparkCrash [] {
        word!(game.samus_shine_timer = 15);
        kSamus_MoveHandler_ShinesparkCrash
            [byte!(@pointer ram_ptr!(game, u16; 0xaae); 1isize) as usize]
            (game);
        for i in (0i32..=2).rev().step_by(2) {
            let v1: i32 = i >> 1;
            let pt = sine_offset::<true>(
                word!(game.speed_echo_xspeed[v1]),
                word!(game.speed_echoes_index) as u8 as u16,
            );
            word!(game.speed_echo_xpos[v1] = (pt.x).wrapping_add(word!(game.samus_x_pos)));
            word!(game.speed_echo_ypos[v1] = (pt.y).wrapping_add(word!(game.samus_y_pos)));
        }
    };
    [9491331] Samus_MoveHandler_ShinesparkCrash_0 [] {
        let mut v0: u16 = (word!(game.speed_echoes_index)).wrapping_add(4);
        if word!(game.speed_echoes_index) as i32 - 12i32 & 0x8000 == 0 {
            v0 = (v0) | (0x100u16);
        }
        word!(game.speed_echoes_index = v0);
    };
    [9491350] Samus_MoveHandler_ShinesparkCrash_1 [] {
        word! { game;
            speed_echo_xspeed[0] = ((*((ram_ptr!(game, u16; 0xab0)).offset(2) as *mut u8))
                .wrapping_add(*((ram_ptr!(game, u16; 0xac0)) as *mut u8)))
                as u16;
            speed_echo_xspeed[1] = ((*((ram_ptr!(game, u16; 0xab0)).offset(2) as *mut u8))
                .wrapping_add(*((ram_ptr!(game, u16; 0xac0)).offset(1) as *mut u8)))
                as u16;
        }
        word!(game.speed_echo_ypos[2] += 4);
        if word!(game.speed_echo_ypos[2]) as i32 - 128i32 & 0x8000 == 0 {
            word!(
                game.speed_echoes_index =
                    ((word!(game.speed_echoes_index) as u8) as u16) | (0x200u16)
            );
        }
    };
    [9491404] Samus_MoveHandler_ShinesparkCrash_2 [] {
        word!(game.speed_echoes_index -= 4);
        if word!(game.speed_echoes_index) as u8 == 0 {
            word!(game.samus_movement_handler = (0x90d3f3i32) as u16);
            word!(game.timer_for_shinesparks_startstop = 30);
            word! { game, 0;
                speed_echoes_index, speed_echo_xspeed[0], speed_echo_xspeed[1],
            }
        }
    };
    [9491443] Samus_MoveHandler_ShinesparkCrashEchoCircle [] {
        word!(game.samus_shine_timer = 15);
        let v0: bool = (word!(game.timer_for_shinesparks_startstop -= 1, reread) & 0x8000)
            != 0;
        if word!(game.timer_for_shinesparks_startstop) == 0 || v0 {
            word!(game.samus_movement_handler = (0x90d40di32) as u16);
            word!(game.samus_draw_handler = (0x90eb52i32) as u16);
        }
    };
    [9491469] Samus_MoveHandler_ShinesparkCrashFinish [] {
        static kShinesparkCrashFinish_Tab0: [u8; 12] = [
            0, 0x80, 0, 0x80, 0x40, 0xc0, 0x40, 0xc0, 0xe0, 0x60, 0x20, 0xa0,
        ];
        word!(game.speed_echoes_index = 0);
        if word!(game.projectile_counter) as i32 - 5i32 & 0x8000 != 0 {
            if word!(game.projectile_counter) as i32 - 4i32 & 0x8000 != 0 {
                word!(game.projectile_counter += 1);
                word!(game.speed_echo_xspeed[2] = 64);
                word!(game.speed_echo_xpos[2] = word!(game.samus_x_pos));
                word!(game.speed_echo_ypos[2] = word!(game.samus_y_pos));
                word!(game.projectile_type[3] = addr_loc_908029 as u16);
                initialize(6, Initialization::Echo, game);
                word!(game.projectile_bomb_pre_instructions[3] = (0x90d4d2i32) as u16);
                word!(
                    game.projectile_variables[3] = kShinesparkCrashFinish_Tab0
                        [(2 * (word!(game.samus_pose) as i32 - 201i32)) as u16 as usize]
                        as u16
                );
                word!(game.projectile_bomb_x_speed[3] = 0);
            }
            word!(game.projectile_counter += 1);
            word! {game; speed_echo_xspeed[3] = 64; speed_echo_xpos[3] = word!(game.samus_x_pos);
                speed_echo_ypos[3] = word!(game.samus_y_pos);
                projectile_type[4] = addr_loc_908029 as u16; }
            initialize(8, Initialization::Echo, game);
            word!(game.projectile_bomb_pre_instructions[4] = (0x90d4d2i32) as u16);
            word!(
                game.projectile_variables[4] = kShinesparkCrashFinish_Tab0
                    [((2 * (word!(game.samus_pose) as i32 - 201i32)) as u16 as i32 + 1) as usize]
                    as u16
            );
            word!(game.projectile_bomb_x_speed[4] = 0);
        }
        word! {game; cooldown_timer = 0; samus_shine_timer = 1; }
        if *ram_ptr!(game; 0xa1e) == 4 {
            word!(game.samus_new_pose_transitional = 2);
        } else {
            word!(game.samus_new_pose_transitional = 1);
        }
        word!(game.samus_hurt_switch_index = 2);
        word! { game, 0; substate, suit_pickup_light_beam_pos }
    };
    [9492088] SamusMoveHandler_CrystalFlashStart [] {
        word!(game.samus_y_pos -= 2);
        if (word!(game.timer_for_shinesparks_startstop -= 1, reread) & 0x8000)
            != 0
        {
            word! {game; samus_anim_frame_timer = 3; samus_anim_frame = 6; }
            *(ram_ptr!(game, u16; 0xdf0)) = word!(game.samus_y_pos);
            word!(game.samus_movement_handler = (0x90d6cei32) as u16);
            word! { game, 0; samus_invincibility_timer, samus_knockback_timer }
            QueueSfx3_Max15(1, game);
            word!(game.power_bomb_flag = 0);
            word!(game.power_bomb_explosion_x_pos = word!(game.samus_x_pos));
            word!(game.power_bomb_explosion_y_pos = word!(game.samus_y_pos));
            EnableHdmaObjects(game);
            spawn_crystal_flash(game);
        }
    };
    [9492174] SamusMoveHandler_CrystalFlashMain [] {
        kSamusMoveHandler_CrystalFlashMainFuncs[word!(game.which_item_to_pickup) as usize]
            (game);
        word! { game, 0; samus_invincibility_timer, samus_knockback_timer }
    };
    [9492315] kSamusMoveHandler_CrystalFlashFinish [] {
        if word!(game.samus_y_pos) != *(ram_ptr!(game, u16; 0xdf0)) {
            word!(game.samus_y_pos += 1);
        }
        if *ram_ptr!(game; 0xa1f) == 0 {
            word! {game; power_bomb_flag = 0; samus_shine_timer = -1i32 as u16;
                samus_movement_handler = ((0x90a337i32) as u16); }
            if word!(game.samus_input_handler) as i32 != 0x90e91d & 0xffff {
                word!(game.samus_input_handler = (0x90e913i32) as u16);
                word! { game, 0; samus_invincibility_timer, samus_knockback_timer }
            }
        }
    };
    [9494328] Samus_MoveHandler_Knockback [] {
        kSamus_MoveHandler_Knockback[word!(game.knockback_dir) as usize]
            (game);
        word!(game.input_to_pose_calc = 0);
    };
    [9494352] Samus_MoveHandler_Knockback_0 [] {
        InvalidInterrupt_Crash(game);
    };
    [9494355] Samus_MoveHandler_Knockback_Up [] {
        player_movement::speed::bomb_rising_x(&mut Runtime::new(game));
        player_movement::speed::move_y_with_speed(&mut Runtime::new(game));
        forced::clear_collided_motion(&mut Runtime::new(game));
    };
    [9494365] Samus_MoveHandler_Knockback_3 [] {
        player_movement::speed::move_y_with_speed(&mut Runtime::new(game));
        forced::clear_collided_motion(&mut Runtime::new(game));
    };
    [9494372] Samus_MoveHandler_Knockback_Down [] {
        player_movement::speed::bomb_rising_x(&mut Runtime::new(game));
        player_movement::speed::no_speed_y(&mut Runtime::new(game));
        forced::clear_collided_motion(&mut Runtime::new(game));
    };
    [9494565] Samus_MoveHandler_BombJumpStart [] {
        pt::initialize_jump(&mut Runtime::new(game), pt::state::JumpKind::Bomb);
        word! {game; samus_movement_handler = ((0x90e032i32) as u16); input_to_pose_calc = 0; }
    };
    [9494578] Samus_MoveHandler_BombJumpMain [] {
        if word!(game.bomb_jump_dir) != 0 {
            kSamus_MoveHandler_BombJumpMain[word!(game.bomb_jump_dir) as u8 as usize]
                (game);
        } else {
            Samus_MoveHandler_BombJumpFunc1(game);
        };
    };
    [9494653] Samus_MoveHandler_BombJumpFunc1 [] {
        #[cfg(sm_verify_dll)]
        crate::game::session_runtime::player_legacy_edges::special_entry(game);
        word!(game.samus_movement_handler = (0x90a337i32) as u16);
        if word!(game.samus_input_handler) as i32 != 0x90e91d & 0xffff {
            word!(game.samus_input_handler = (0x90e913i32) as u16);
        }
        word!(game.bomb_jump_dir = 0);
    };
    [9496911] SamusMovementType_Xray [] {
        let mut v0: u16 = 0;
        if *ram_ptr!(game; 0xa1f) as i32 != kMovementType_0E_TurningAroundOnGround as i32 {
            word!(game.samus_anim_frame_timer = 15);
            if *ram_ptr!(game; 0xa1e) == 4 {
                if word!(game.xray_angle) as i32 - 153i32 & 0x8000 != 0 {
                    v0 = 4;
                } else if word!(game.xray_angle) as i32 - 178i32 & 0x8000 != 0 {
                    v0 = 3;
                } else if word!(game.xray_angle) as i32 - 203i32 & 0x8000 != 0 {
                    v0 = 2;
                } else {
                    v0 = ((word!(game.xray_angle) as i32 - 228i32 & 0x8000) != 0) as u16;
                }
            } else if word!(game.xray_angle) as i32 - 25i32 & 0x8000 != 0 {
                v0 = 0;
            } else if word!(game.xray_angle) as i32 - 50i32 & 0x8000 != 0 {
                v0 = 1;
            } else if word!(game.xray_angle) as i32 - 75i32 & 0x8000 != 0 {
                v0 = 2;
            } else if word!(game.xray_angle) as i32 - 100i32 & 0x8000 != 0 {
                v0 = 3;
            } else {
                v0 = 4;
            }
            word!(game.samus_anim_frame = v0);
        }
    };
    [9498699] Samus_Func25_ShineSpark [] {
        if word!(game.samus_shine_timer) == 0 {
            word!(game.special_samus_palette_frame = 6);
            word! { game, 1; special_samus_palette_timer, demo_timer_counter }
            word! {game; timer_for_shine_timer = 10; samus_shine_timer = 120;
                samus_movement_handler = ((0x90f072i32) as u16); }
        }
    };
    [9498738] Samus_MoveHandler_F072 [] {
        if word!(game.samus_shine_timer) == 0 {
            word!(game.frame_handler_alfa = (0x90e695i32) as u16);
            word!(game.samus_movement_handler = (0x90a337i32) as u16);
        }
    };
    @external {
        9496846 => [()];
    }
}

pub unsafe fn Samus_PauseCheck(game: &mut Game) {
    unsafe {
        if word!(game.power_bomb_flag) == 0
            && word!(game.time_is_frozen_flag) == 0
            && word!(game.door_transition_flag_enemies) == 0
            && word!(game.area_index) != 6
            && word!(game.game_state) as i32 == kGameState_8_MainGameplay as i32
            && (word!(game.joypad1_newkeys) as i32 & kButton_Start as i32) != 0
        {
            word! { game, 1; screen_fade_delay, screen_fade_counter }
            word!(game.game_state = kGameState_12_Pausing as u16);
        }
    }
}
pub unsafe fn Samus_LowHealthCheck_(game: &mut Game) {
    if game.samus_health().wrapping_sub(31) & 0x8000 != 0 {
        if game.samus_health_warning() == 0 {
            QueueSfx3_Max6(2, game);
            game.set_samus_health_warning(1);
        }
    } else if game.samus_health_warning() != 0 {
        game.set_samus_health_warning(0);
        QueueSfx3_Max6(1, game);
    }
}
pub use self::Samus_LowHealthCheck as Samus_LowHealthCheck_0;
pub unsafe fn Samus_JumpCheck(game: &mut Game) {
    unsafe {
        if (word!(game.button_config_jump_a) & word!(game.joypad1_lastkeys)) != 0
            && (word!(game.button_config_jump_a) & word!(game.joypad1_input_samusfilter)) != 0
        {
            word!(game.autojump_timer += 1);
        } else {
            word!(game.autojump_timer = 0);
        }
        word!(game.joypad1_input_samusfilter = word!(game.joypad1_lastkeys));
        word!(game.joypad1_newinput_samusfilter = word!(game.joypad1_newkeys));
        let update_health_reference: bool = (((word!(game.samus_health)) as i16)
            .wrapping_sub((word!(game.samus_prev_health_for_flash)) as i16)
            >= 0)
            || ({
                if word!(game.samus_hurt_flash_counter) == 0 {
                    word!(game.samus_hurt_flash_counter = 1);
                }
                (word!(game.debug_invincibility) as i32 - 7i32 & 0x8000 != 0)
                    || ({
                        word!(game.samus_health = word!(game.samus_prev_health_for_flash));
                        false
                    })
            });
        if update_health_reference {
            word!(game.samus_prev_health_for_flash = word!(game.samus_health));
        };
    }
}
pub unsafe fn DrawSamusAndProjectiles(game: &mut Game) {
    unsafe {
        SamusDrawSprites(game);
        draw_projectiles(game);
        Samus_JumpCheck(game);
        Samus_ShootCheck(game);
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallSamusDrawHandler(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [9497426] SamusDrawHandler_Default [] {
        crate::game::player_projectiles::flare(game);
        sub_90EB55(game);
    };
    [9497478] sub_90EB86 [] {
        if word!(game.grapple_beam_function) as i32 + 0x37aa & 0x8000 == 0 {
            sub_90EB55(game);
            return;
        }
        if word!(game.grapple_beam_function) as i32 == 0x9bc703 & 0xffff
            || word!(game.grapple_beam_function) as i32 == 0x9bc759 & 0xffff
        {
            GrappleBeamFunc_BF1B(game);
        }
        crate::game::player_projectiles::grapple_flare(game);
        crate::game::visual_runtime::draw_player_with_cannon(game);
        UpdateGrappleBeamTilesAndIncrFlameCtr(game);
        if word!(game.grapple_beam_length) == 0 {
            return;
        }
        HandleGrappleBeamGfx(game);
    };
    [9497587] Samus_DrawHandler_EndOfShinespark [] {
        Samus_Draw(game);
        for i in (0i32..=2).rev().step_by(2) {
            Samus_DrawShinesparkCrashEchoes(i as u16, game);
        }
    };
    [9497620] SamusDisplayHandler_UsingElevator [] {
        if (word!(game.nmi_frame_counter_word) & 1) == 0 {
            SamusDisplayHandler_SamusReceivedFatal(game);
        }
    };
    [9497629] SamusDisplayHandler_SamusReceivedFatal [] {
        Samus_DrawWhenNotAnimatingOrDying(game);
    };
    @external {
        9496846 | 9497586 => [()];
    }
}
pub unsafe fn SamusDrawSprites(game: &mut Game) {
    unsafe {
        crate::game::visual_runtime::update_arm_cannon(game);
        CallSamusDrawHandler(
            ((word!(game.samus_draw_handler)) as u32) | (0x900000u32),
            game,
        );
    }
}
pub unsafe fn sub_90EB55(game: &mut Game) {
    crate::game::visual_runtime::draw_player_with_cannon(game);
    unsafe {
        Samus_DrawEchoes(game);
    }
}
pub fn Samus_SetRadius(game: &mut Game) {
    let pose_offset = usize::from(game.samus_pose()) * 8;
    let radius = u16::from(game.read_rom_byte(0x91b629, pose_offset + 6));
    game.g_ram[0xb00..0xb02].copy_from_slice(&radius.to_le_bytes());
    game.g_ram[0xafe..0xb00].copy_from_slice(&5u16.to_le_bytes());
}
pub fn Samus_GetBottom_R18(game: &Game) -> u16 {
    let y = game.samus_y_pos();
    // SamusPoseParams has eight bytes, with y_radius at byte 6.
    let pose_offset = usize::from(game.samus_pose()) * 8;
    let radius = game.read_rom_byte(0x91b629, pose_offset + 6);
    y.wrapping_add(u16::from(radius)).wrapping_sub(1)
}
pub fn Samus_PostGrappleCollisionDetect(game: &mut Game) {
    crate::game::player_collision::operations::settle_after_grapple(
        &mut crate::game::session_runtime::Runtime::new(game),
    );
}
pub unsafe fn Samus_ShootCheck(game: &mut Game) {
    unsafe {
        crate::game::player_projectiles::movement_sound(game);
        if word!(game.enable_debug) != 0 {
            if word!(game.samus_pose) as i32 == kPose_00_FaceF_Powersuit as i32
                || word!(game.samus_pose) as i32 == kPose_9B_FaceF_VariaGravitySuit as i32
            {
                if (word!(game.joypad2_last) & 0x30) == 48
                    && (word!(game.joypad2_new_keys) & 0x80) != 0
                {
                    word!(game.debug_invincibility = 7);
                }
            } else {
                if word!(game.debug_invincibility) as i32 - 7i32 & 0x8000 == 0 {
                    return;
                }
                word!(game.debug_invincibility = 0);
            }
        }
        if CheckEventHappened(0xe, game) & 1 != 0
            && word!(game.frame_handler_gamma) as i32 == 0x90e114 & 0xffff
            && word!(game.game_state) as i32 != kGameState_35_TimeUp as i32
        {
            word!(game.game_state = kGameState_35_TimeUp as u16);
        }
    }
}
