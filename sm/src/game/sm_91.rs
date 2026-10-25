use crate::game::player_transitions as pt;

pub use crate::game::prelude::*;
use crate::game::session_runtime::Runtime;

pub use crate::game::constants::bank_8d::*;
pub use crate::game::constants::bank_8f::*;
pub use crate::game::constants::bank_9b::*;
pub use crate::game::constants::bank_91::*;

use crate::game::animated_tiles;
use crate::game::enemy_projectiles::{DisableEprojs, EnableEprojs};
use crate::game::palette_effects;
use crate::game::room_mechanisms::set_enabled;

use crate::game::session_runtime::player_command as CallSomeSamusCode;
use crate::game::sm_80::InvalidInterrupt_Crash;
use crate::game::sm_88::SpawnHdmaObject;

pub const NULL: *mut c_void = null_mut::<c_void>();
crate::game::constants::typed_constants! { i32;
    true_0 = 1, false_0 = 0,
    fnSamus_MovementHandler_Normal = 0x90a337, fnProjPreInstr_Empty = 0x90b169,
    fnSamus_MoveHandlerVerticalShinespark = 0x90d0ab, fnSamus_MoveHandler_Shinespark_Diag = 0x90d0d7,
    fnSamus_MoveHandler_Shinespark_Horiz = 0x90d106, fnSamus_MoveHandler_Knockback = 0x90df38,
    fnSamus_MoveHandler_BombJumpStart = 0x90e025, fnSamus_Func3 = 0x90e0e6,
    fnSamus_Func9 = 0x90e41b, fnSamus_FrameHandlerAlfa_Func11 = 0x90e695,
    fnSamus_FrameHandlerAlfa_Func12 = 0x90e6c9, fnSamus_FrameHandlerAlfa_Func13 = 0x90e713,
    fnSamus_FrameHandlerBeta_Func17 = 0x90e725, fnSamus_FrameHandlerBeta_Func14 = 0x90e7f5,
    fnSamus_Func16 = 0x90e86a, fnEmptyFunction = 0x90e8cd,
    fnj_HandleDemoRecorder_2_0 = 0x90e8d9, fnSetContactDamageIndexAndUpdateMinimap = 0x90e8dc,
    fnnullsub_152 = 0x90e90e, fnSamus_InputHandler_E913 = 0x90e913,
    fnSamus_Func20_ = 0x90e918, fnSamus_InputHandler_E91D = 0x90e91d,
    fnHandleAutoJumpHack = 0x90e926, fnSamusMovementType_Xray = 0x90e94f,
    fnSamusDrawHandler_Default = 0x90eb52,
    fnDemoPreInstr_CheckLeaveDemo = 0x918a9b,
    fnGrappleBeamFunc_Inactive = 0x9bc4f0,
}
pub const __ASSERT_FUNCTION: [c_char; 28] =
    unsafe { transmute::<[u8; 28], [c_char; 28]>(*b"uint8 *RomPtr_RAM(uint16_t)\0") };
#[inline]
unsafe fn RomPtr_RAM(addr: u16, game: &mut Game) -> *mut u8 {
    unsafe {
        crate::native_assertion! {
            '_c2rust_label: addr < 0x2000;
            b"addr < 0x2000\0", b"src/sm_rtl.h\0", 51, pointer __ASSERT_FUNCTION.as_ptr()
        };
        ram_ptr!(game; addr as i32 as isize)
    }
}
pub unsafe fn XrayRunHandler(game: &mut Game) {
    unsafe {
        if word!(game.time_is_frozen_flag) == 0
            && (word!(game.button_config_run_b) & word!(game.joypad1_lastkeys)) != 0
            && Xray_Initialize(game) & 1 != 0
        {
            SpawnHdmaObject(
                0x91,
                rom_ptr!(game, SpawnHdmaObject_Args; 0x91caf2) as *const c_void,
                game,
            );
        }
    }
}
pub unsafe fn GameState_28_Unused_(game: &mut Game) {
    unsafe {
        Unreachable(game);
    }
}
// These legacy entries acquire the one aggregate; composed callers invoke the
// palette core directly on their existing Runtime.
pub unsafe fn Samus_CancelSpeedBoost(game: &mut Game) {
    crate::game::samus_palette::cancel_speed_boost(
        &mut crate::game::session_runtime::Runtime::new(game),
    );
}
pub unsafe fn Samus_LoadSuitPalette(game: &mut Game) {
    crate::game::samus_palette::load_suit(
        &mut crate::game::session_runtime::Runtime::new(game),
        crate::game::visual_state::PaletteBuffer::Current,
    );
}
pub unsafe fn Samus_LoadSuitTargetPalette(game: &mut Game) {
    crate::game::samus_palette::load_suit(
        &mut crate::game::session_runtime::Runtime::new(game),
        crate::game::visual_state::PaletteBuffer::Target,
    );
}
pub unsafe fn Samus_DealDamage(amount: u16, game: &mut Game) {
    unsafe {
        if amount & 0x8000 == 0 {
            if amount != 300 && game.time_is_frozen_flag() == 0 {
                let health = game.samus_health().wrapping_sub(amount);
                game.set_samus_health(health);
                if game.samus_health() & 0x8000 != 0 {
                    game.set_samus_health(0);
                }
            }
        } else {
            InvalidInterrupt_Crash(game);
        }
    }
}
pub unsafe fn Xray_Initialize(game: &mut Game) -> u8 {
    unsafe {
        static byte_91E291: [u8; 28] = [
            1, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
        ];
        if word!(game.cooldown_timer) == 7
            && word!(game.bomb_counter) == 5
            && word!(game.samus_x_speed_divisor) == 2
            || word!(game.samus_pose) as i32 - kPose_A4_FaceR_LandJump as i32 & 0x8000 == 0
                && (word!(game.samus_pose) as i32 - kPose_A8_FaceR_Grappling as i32 & 0x8000 != 0
                    || word!(game.samus_pose) as i32 - kPose_E0_FaceR_LandJump_AimU as i32 & 0x8000
                        == 0
                        && word!(game.samus_pose) as i32
                            - kPose_E8_FaceR_Drained_CrouchFalling as i32
                            & 0x8000
                            != 0)
            || word!(game.game_state) != 8
            || word!(game.power_bomb_explosion_status) != 0
            || word!(game.samus_y_speed) != 0
            || word!(game.samus_y_subspeed) != 0
            || byte_91E291[*ram_ptr!(game; 0xa11) as usize] == 0
        {
            return 0;
        }
        if byte_91E291[*ram_ptr!(game; 0xa1f) as usize] == 1 {
            if *ram_ptr!(game; 0xa1e) == 4 {
                word!(game.samus_new_pose_interrupted = kPose_D6_FaceL_Xray_Stand as u16);
            } else {
                word!(game.samus_new_pose_interrupted = kPose_D5_FaceR_Xray_Stand as u16);
            }
        } else {
            if byte_91E291[*ram_ptr!(game; 0xa1f) as usize] != 2 {
                return 0;
            }
            if *ram_ptr!(game; 0xa1e) == 4 {
                word!(game.samus_new_pose_interrupted = kPose_DA_FaceL_Xray_Crouch as u16);
            } else {
                word!(game.samus_new_pose_interrupted = kPose_D9_FaceR_Xray_Crouch as u16);
            }
        }
        word! {game; time_is_frozen_flag = 1; samus_special_transgfx_index = 5; }
        word_transfers!(game, descending, 256; le(hdma_table_1, 255) <- value(255));
        DisableEprojs(game);
        set_enabled(false, game);
        animated_tiles::set_enabled(false, game);
        palette_effects::set_enabled(false, game);
        mov24(ram_ptr!(game, LongPtr; 0xa88), 0x980001);
        *(ram_ptr!(game, u16; 0xa8a, 1)) = 0;
        word!(game.demo_input_prev = -26424i32 as u16);
        mov24(ram_ptr!(game, LongPtr; 0xa8e), 0x999098);
        word! { game, 0;
            hdma_var_1, demo_input_pre_instr, demo_input_instr_timer, demo_input_instr_ptr, demo_input,
            demo_input_new,
        }
        if *ram_ptr!(game; 0xa1e) == 4 {
            word!(game.xray_angle = 192);
        } else {
            word!(game.xray_angle = 64);
        }
        1
    }
}
pub unsafe fn ResponsibleForXrayStandupGlitch(game: &mut Game) {
    unsafe {
        if *ram_ptr!(game; 0xa1f) as i32 == kMovementType_05_Crouching as i32 {
            if *ram_ptr!(game; 0xa1e) == 4 {
                word!(game.samus_pose = kPose_28_FaceL_Crouch as u16);
            } else {
                word!(game.samus_pose = kPose_27_FaceR_Crouch as u16);
            }
        } else if *ram_ptr!(game; 0xa1e) as i32 == kPose_04_FaceL_AimU as i32 {
            word!(game.samus_pose = kPose_02_FaceL_Normal as u16);
        } else {
            word!(game.samus_pose = kPose_01_FaceR_Normal as u16);
        }
        pt::finish_pose_change(&mut Runtime::new(game));
        word! {game; samus_movement_handler = ((0x90a337i32) as u16);
        samus_input_handler = ((0x90e913i32) as u16); demo_timer_counter = -1i32 as u16; }
        let r18: u16 = (((*(rom_ptr!(game, SamusPoseParams; 0x91b629))
            .offset(word!(game.samus_pose) as isize))
        .y_radius) as u16)
            .wrapping_sub(word!(game.samus_y_radius));
        if (r18 & 0x8000) == 0 {
            word!(game.samus_y_pos -= r18);
            word!(game.samus_prev_y_pos = word!(game.samus_y_pos));
        }
        EnableEprojs(game);
        set_enabled(true, game);
        animated_tiles::set_enabled(true, game);
        palette_effects::set_enabled(true, game);
    }
}
pub unsafe fn Samus_Func20(game: &mut Game) {
    unsafe {
        if *ram_ptr!(game; 0xa1f) == 14 {
            if word!(game.samus_anim_frame) == 2 && word!(game.samus_anim_frame_timer) == 1 {
                if *ram_ptr!(game; 0xa1e) == 4 {
                    if word!(game.samus_pose) as i32 == kPose_25_FaceR_Turn_Stand as i32 {
                        word!(game.samus_pose = kPose_D6_FaceL_Xray_Stand as u16);
                    } else {
                        word!(game.samus_pose = kPose_DA_FaceL_Xray_Crouch as u16);
                    }
                } else if word!(game.samus_pose) as i32 == kPose_26_FaceL_Turn_Stand as i32 {
                    word!(game.samus_pose = kPose_D5_FaceR_Xray_Stand as u16);
                } else {
                    word!(game.samus_pose = kPose_D9_FaceR_Xray_Crouch as u16);
                }
                pt::finish_pose_change(&mut Runtime::new(game));
            }
        } else {
            if *ram_ptr!(game; 0xa1e) == 4 {
                if (word!(game.button_config_right) & word!(game.joypad1_lastkeys)) == 0 {
                    return;
                }
                word!(game.xray_angle = (256u16).wrapping_sub(word!(game.xray_angle)));
                if *ram_ptr!(game; 0xa1f) == 5 {
                    word!(game.samus_pose = kPose_44_FaceL_Turn_Crouch as u16);
                } else {
                    word!(game.samus_pose = kPose_26_FaceL_Turn_Stand as u16);
                }
            } else {
                if (word!(game.button_config_left) & word!(game.joypad1_lastkeys)) == 0 {
                    return;
                }
                word!(game.xray_angle = (256u16).wrapping_sub(word!(game.xray_angle)));
                if *ram_ptr!(game; 0xa1f) as i32 == kMovementType_05_Crouching as i32 {
                    word!(game.samus_pose = kPose_43_FaceR_Turn_Crouch as u16);
                } else {
                    word!(game.samus_pose = kPose_25_FaceR_Turn_Stand as u16);
                }
            }
            pt::finish_pose_change(&mut Runtime::new(game));
        };
    }
}
