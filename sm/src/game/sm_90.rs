use crate::game::missile_firing_ram::fire as fire_missile;
use crate::game::special_attacks::Kind as SpecialKind;
use crate::game::special_attacks_ram as special_attacks;
use crate::game::projectile_initialization::Initialization;
use crate::game::projectile_initialization_ram::initialize;
use crate::game::bomb_lifecycle::{Kind as BombKind, SPREAD_FRACTIONS as kBombSpread_Tab3};
use crate::game::bomb_lifecycle_ram as bomb_lifecycle;
use crate::game::music::{PlayRoomMusicTrackAfterAFrames};
use crate::game::beam_firing_ram::{clear_animation, update_input};
use crate::game::launch_motion::MovementDirection;
use crate::game::launch_motion_ram;
pub use crate::game::prelude::*;
use crate::game::samus_animation::{
    HandleAtmosphericEffects, PlaySamusFanfare, Samus_Animate, Samus_Draw, Samus_DrawEchoes,
    Samus_DrawShinesparkCrashEchoes, Samus_DrawWhenNotAnimatingOrDying,
};

pub use crate::game::constants::bank_60::*;
pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_8d::*;
pub use crate::game::constants::bank_90::*;

use crate::game::explosion_window::spawn_crystal_flash;
use crate::game::progress_flags::CheckEventHappened;
use crate::game::projectile_animation::start_impact;
use crate::game::projectile_presentation_ram::draw_projectiles;
use crate::game::projectile_playback_ram::advance;
use crate::game::projectile_flight::{self, Flight};
use crate::game::sm_80::{
    DrawTimer, HandleAutoscrolling_X, HandleAutoscrolling_Y,
    HandleScrollingWhenTriggeringScrollDown, HandleScrollingWhenTriggeringScrollLeft,
    HandleScrollingWhenTriggeringScrollRight, HandleScrollingWhenTriggeringScrollUp,
    InvalidInterrupt_Crash, ProcessTimer, QueueSfx1_Max15, QueueSfx1_Max6, QueueSfx1_Max9,
    QueueSfx2_Max6, QueueSfx3_Max15, QueueSfx3_Max6, QueueSfx3_Max9,
};
use crate::game::sm_81::DrawBeamGrappleSpritemap;
use crate::game::sm_86::{EnableEprojs, SpawnEprojWithGfx};
use crate::game::sm_88::EnableHdmaObjects;
use crate::game::sm_8b::Samus_CalcPos_Mode7;
use crate::game::sm_8d::{DisablePaletteFx, SpawnPalfxObject};
use crate::game::sm_91::{
    DemoObjectInputHandler, MakeSamusFaceForward, SamusFunc_E633, SamusFunc_F433,
    Samus_CancelSpeedBoost, Samus_Func20, Samus_HandlePalette, Samus_HandleTransFromBlockColl,
    Samus_HandleTransitions, Samus_InputHandler, Samus_LoadSuitPalette,
    Samus_LoadSuitTargetPalette, Samus_RestoreHealth, XrayRunHandler,
};
use crate::game::sm_94::{
    scale_sine_word, BlockCollNoWaveBeamHoriz, BlockCollNoWaveBeamVert, BlockCollWaveBeamHoriz,
    BlockCollWaveBeamVert, BlockInsideDetection, HandleGrappleBeamGfx,
    PostGrappleCollisionDetect_X, PostGrappleCollisionDetect_Y, Samus_AlignYPosSlope,
};
use crate::game::sm_9b::{
    GrappleBeamFunc_BF1B, GrappleBeamHandler, HandleGrappleBeamFlare,
    UpdateGrappleBeamTilesAndIncrFlameCtr,
};
use crate::game::sm_a0::Samus_CheckSolidEnemyColl;

pub use crate::types::{Func_V_A, HandlerFunc};

pub use crate::types::LiquidPhysicsType;

crate::game::constants::typed_constants! { LiquidPhysicsType;
    kLiquidPhysicsType_LavaAcid = 2, kLiquidPhysicsType_Water = 1,
    kLiquidPhysicsType_Air = 0,
}

crate::typed_struct! {
    [
        #[derive(Copy, Clone)]
        #[repr(C, packed)]
    ];
    pub struct SamusSpeedTableEntry {
        pub accel, accel_sub, max_speed, max_speed_sub, decel, decel_sub: u16;
    }

}

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
const RomPtr_90: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x90>;
const RomPtr_91: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x91>;
crate::ram_map::ram_pointer_functions! {
    gOamEnt(v: i32) -> OamEnt = [(0x370 + v) as isize];
    gVramWriteEntry(a: i32) -> VramWriteEntry = [(0xd0 + a) as isize];
}
#[inline]
unsafe fn get_SamusSpeedTableEntry(a: u16, game: &mut Game) -> *mut SamusSpeedTableEntry {
    unsafe { RomPtr((0x900000u32) | ((a) as u32), game) as *mut SamusSpeedTableEntry }
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
/// Which liquid contains a vertical position. This query deliberately ignores
/// equipment: movement and graphics callers apply different suit rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LiquidAtHeight {
    None,
    Water,
    LavaAcid,
}

pub(crate) fn liquid_at_height(game: &Game, y: u16) -> LiquidAtHeight {
    let water_y = u16::from_le_bytes([game.g_ram[0x195e], game.g_ram[0x195f]]);
    // A present water surface takes precedence, even above the queried height.
    // Preserve the SNES sign-bit test on subtraction, including wraparound.
    if water_y & 0x8000 == 0 {
        if water_y.wrapping_sub(y) & 0x8000 != 0 {
            let options = u16::from_le_bytes([game.g_ram[0x197e], game.g_ram[0x197f]]);
            if options & 4 == 0 {
                return LiquidAtHeight::Water;
            }
        }
    } else {
        let lava_y = u16::from_le_bytes([game.g_ram[0x1962], game.g_ram[0x1963]]);
        if lava_y & 0x8000 == 0 && lava_y.wrapping_sub(y) & 0x8000 != 0 {
            return LiquidAtHeight::LavaAcid;
        }
    }
    LiquidAtHeight::None
}

pub unsafe fn SetLiquidPhysicsType(game: &mut Game) {
    unsafe {
        let r18: u16 = Samus_GetBottom_R18(game);
        match (word!(game.fx_type) as i32 & 0xf) >> 1 {
            1 | 2 => {
                if (word!(game.lava_acid_y_pos) & 0x8000) == 0
                    && word!(game.lava_acid_y_pos) as i32 - r18 as i32 & 0x8000 != 0
                {
                    word!(game.liquid_physics_type = kLiquidPhysicsType_LavaAcid as u16);
                } else {
                    word!(game.liquid_physics_type = 0);
                }
            }
            3 => {
                if (word!(game.fx_y_pos) & 0x8000) == 0
                    && word!(game.fx_y_pos) as i32 - r18 as i32 & 0x8000 != 0
                    && (word!(game.fx_liquid_options) & 4) == 0
                {
                    word!(game.liquid_physics_type = kLiquidPhysicsType_Water as u16);
                } else {
                    word!(game.liquid_physics_type = 0);
                }
            }
            _ => {
                word!(game.liquid_physics_type = 0);
            }
        };
    }
}
pub unsafe fn Samus_HandleMovement_X(game: &mut Game) {
    unsafe {
        Samus_HandleExtraRunspeedX(game);
        Samus_MoveX(
            Samus_CalcBaseSpeed_X(Samus_DetermineSpeedTableEntryPtr_X(game), game),
            game,
        );
    }
}
#[derive(Clone, Copy)]
enum SamusJumpKind {
    Normal,
    Wall,
    Knockback,
    Bomb,
}

unsafe fn samus_jump_environment(game: &mut Game) -> usize {
    unsafe {
        if word!(game.equipped_items) & 0x20 != 0 {
            return 0;
        }
        let bottom = Samus_GetBottom_R18(game);
        match liquid_at_height(game, bottom) {
            LiquidAtHeight::None => 0,
            LiquidAtHeight::Water => 1,
            LiquidAtHeight::LavaAcid => 2,
        }
    }
}

unsafe fn initialize_samus_jump(kind: SamusJumpKind, game: &mut Game) {
    unsafe {
        let environment = samus_jump_environment(game);
        let speeds = match kind {
            SamusJumpKind::Normal => {
                if word!(game.equipped_items) & 0x100 != 0 {
                    [(0, 6), (0x8000, 2), (0x8000, 3)]
                } else {
                    [(0xe000, 4), (0xc000, 1), (0xc000, 2)]
                }
            }
            SamusJumpKind::Wall => {
                if word!(game.equipped_items) & 0x100 != 0 {
                    [(0x8000, 5), (0x8000, 0), (0x8000, 3)]
                } else {
                    [(0xa000, 4), (0x4000, 0), (0xa000, 2)]
                }
            }
            SamusJumpKind::Knockback => [(0, 5), (0, 2), (0, 2)],
            SamusJumpKind::Bomb => [(0xc000, 2), (0x1000, 0), (0x1000, 0)],
        };
        word! {game; samus_y_subspeed = speeds[environment].0; samus_y_speed = speeds[environment].1; }
        if matches!(kind, SamusJumpKind::Normal | SamusJumpKind::Wall)
            && word!(game.equipped_items) & 0x2000 != 0
        {
            if matches!(kind, SamusJumpKind::Wall) {
                let _ = word!(game.samus_x_extra_run_speed) >> 1;
            }
            word!(game.samus_y_subspeed += word!(game.samus_x_extra_run_subspeed));
            word!(
                checked_i32,
                game.samus_y_speed += (word!(game.samus_x_extra_run_speed) as i32 >> 1)
            );
        }
        word! { game, 0; grapple_walljump_timer, reached_ceres_elevator_fade_timer }
        word!(game.samus_y_dir = 1);
    }
}

unsafe fn move_samus_x_in_direction(mut amt: i32, moving_left: bool, game: &mut Game) {
    unsafe {
        if moving_left {
            amt = Samus_CalcDisplacementMoveLeft(amt, game);
        } else {
            amt = Samus_CalcDisplacementMoveRight(amt, game);
        }
        if amt < 0 {
            Samus_MoveLeft(amt, game);
        } else {
            Samus_MoveRight(amt, game);
        };
    }
}

unsafe fn move_samus_air_x<const MOVE_AT_REST: bool>(amt: i32, game: &mut Game) {
    unsafe {
        if word!(game.samus_x_accel_mode) != 0
            || (word!(game.joypad1_lastkeys) as i32 & kButton_Right as i32) != 0
            || (word!(game.joypad1_lastkeys) as i32 & kButton_Left as i32) != 0
        {
            Samus_MoveX(amt, game);
        } else {
            word! { game, 0; samus_x_base_speed, samus_x_base_subspeed, samus_collision_flag }
            if MOVE_AT_REST {
                Samus_MoveX(0, game);
            }
        }
    }
}

unsafe fn check_samus_jump_release(game: &mut Game) {
    unsafe {
        if word!(game.samus_y_dir) == 1
            && ((word!(game.button_config_jump_a) & word!(game.joypad1_lastkeys)) == 0
                || (word!(game.samus_y_speed) & 0x8000) != 0)
        {
            word! { game, 0; samus_y_subspeed, samus_y_speed }
            word!(game.samus_y_dir = 2);
        }
    }
}

unsafe fn move_samus_horizontal_collision<const LEFT: bool>(mut amt: i32, game: &mut Game) {
    unsafe {
        if LEFT {
            amt = -amt;
        }
        let cres: CheckEnemyColl_Result = Samus_CheckSolidEnemyColl(amt, game);
        word!(game.samus_collision_flag = cres.collision);
        amt = cres.amt;
        if word!(game.samus_collision_flag) != 0 {
            Samus_ClearXSpeedIfColl(game);
            if LEFT {
                Samus_MoveLeft_NoColl(amt, game);
            } else {
                Samus_MoveRight_NoColl(amt, game);
            }
            Samus_AlignYPosSlope(game);
        } else {
            amt = crate::game::samus_room_contact_ram::move_right(
                if LEFT { -amt } else { amt },
                game,
            );
            if LEFT {
                launch_motion_ram::record_movement(game, MovementDirection::Left, amt);
            } else {
                launch_motion_ram::record_movement(game, MovementDirection::Right, amt);
            }
            if ((word!(game.samus_collision_direction) & 1) != 0) == LEFT {
                word!(game.samus_collision_flag = 0);
            }
            Samus_ClearXSpeedIfColl(game);
            Samus_AlignYPosSlope(game);
        };
    }
}

pub unsafe fn Samus_MoveX(mut amt: i32, game: &mut Game) {
    unsafe {
        crate::verify_probe::hit("samus.move.x");
        let moving_left: bool =
            if word!(game.samus_x_accel_mode) == 0 || word!(game.samus_x_accel_mode) == 2 {
                !(*ram_ptr!(game; 0xa1e) != 4)
            } else {
                !(*ram_ptr!(game; 0xa1e) != 8)
            };
        move_samus_x_in_direction(amt, moving_left, game);
    }
}
pub unsafe fn Samus_BombJumpRisingXMovement_(game: &mut Game) {
    unsafe {
        crate::verify_probe::hit("samus.bomb.rising-x");
        let mut amt: i32 = 0;
        let moving_left: bool = if word!(game.knockback_dir) != 0 {
            amt = Samus_CalcBaseSpeed_X(Samus_DetermineSpeedTableEntryPtr_X(game), game);
            (word!(game.knockback_x_dir) == 0)
        } else {
            amt = Samus_CalcBaseSpeed_X(addr_stru_909F25 as u16, game);
            !((word!(game.bomb_jump_dir) as u8) != 1)
        };
        move_samus_x_in_direction(amt, moving_left, game);
    }
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
                word!(game.samus_input_handler = ((0x90e913i32) as u16));
            }
        }
    }
}
pub unsafe fn Samus_BombJumpFallingXMovement_(game: &mut Game) {
    unsafe {
        crate::verify_probe::hit("samus.bomb.falling-x");
        let mut amt: i32 = Samus_CalcBaseSpeed_X(Samus_DetermineSpeedTableEntryPtr_X(game), game);
        move_samus_x_in_direction(amt, word!(game.samus_var62) == 1, game);
    }
}
pub unsafe fn Samus_BombJumpFallingYMovement_(game: &mut Game) {
    unsafe {
        if word!(game.samus_y_speed) as i32 - 5i32 & 0x8000 != 0 {
            words!(
                game.samus_y_speed,
                samus_y_subspeed += words!(game.samus_y_accel, samus_y_subaccel)
            );
        }
        Samus_MoveDown((words!(game.samus_y_speed, samus_y_subspeed)) as i32, game);
    }
}
pub unsafe fn Samus_JumpingMovement(game: &mut Game) {
    unsafe {
        crate::verify_probe::hit("samus.move.jumping");
        Samus_HandleExtraRunspeedX(game);
        if word!(game.samus_pose) as i32 == kPose_4B_FaceR_Jumptrans as i32
            || word!(game.samus_pose) as i32 == kPose_4C_FaceL_Jumptrans as i32
            || word!(game.samus_pose) as i32 - kPose_55_FaceR_Jumptrans_AimU as i32 & 0x8000 == 0
                && word!(game.samus_pose) as i32 - kPose_5B as i32 & 0x8000 != 0
        {
            word!(game.samus_x_accel_mode = 0);
            Samus_Move_NoBaseSpeed_X(game);
            Samus_MoveExtraY(game);
            return;
        }
        check_samus_jump_release(game);
        let pair: Pair_Bool_Amt =
            Samus_CalcBaseSpeed_NoDecel_X(Samus_DetermineSpeedTableEntryPtr_X(game), game);
        let move_horizontally: bool = if *ram_ptr!(game; 0xa1f) == 20 {
            if word!(game.samus_x_accel_mode) == 0 {
                word!(game.samus_x_accel_mode = 2);
            }
            true
        } else {
            (word!(game.samus_x_accel_mode) != 0
                || (word!(game.joypad1_lastkeys) & 0x100) != 0
                || (word!(game.joypad1_lastkeys) & 0x200) != 0)
                || ({
                    word! { game, 0;
                        samus_x_base_speed, samus_x_base_subspeed, samus_collision_flag,
                    }
                    false
                })
        };
        if move_horizontally {
            Samus_MoveX(pair.amt, game);
        }
        Samus_MoveY_WithSpeedCalc(game);
    }
}
pub unsafe fn Samus_SpinJumpMovement(game: &mut Game) {
    unsafe {
        crate::verify_probe::hit("samus.move.spinning");
        let mut current_block: u64;
        static g_word_909E9F: u16 = 8;
        Samus_HandleExtraRunspeedX(game);
        check_samus_jump_release(game);
        let mut pair: Pair_Bool_Amt =
            Samus_CalcBaseSpeed_NoDecel_X(Samus_DetermineSpeedTableEntryPtr_X(game), game);
        if pair.flag as i32 != 0 || word!(game.samus_x_accel_mode) == 1 {
            current_block = 15844657471994422988;
        } else {
            if *ram_ptr!(game; 0xa1e) == 4 {
                if (word!(game.joypad1_lastkeys) as i32 & kButton_Left as i32) != 0 {
                    current_block = 15844657471994422988;
                } else {
                    current_block = 11746438652493739369;
                }
            } else if (word!(game.joypad1_lastkeys) as i32 & kButton_Right as i32) == 0 {
                current_block = 11746438652493739369;
            } else {
                current_block = 15844657471994422988;
            }
            if current_block == 15844657471994422988 {
            } else {
                pair.amt = 0;
                word! { game, 0;
                    samus_x_base_speed, samus_x_base_subspeed, samus_collision_flag, samus_x_accel_mode,
                }
                current_block = 7037269245041226537;
            }
        }
        if current_block == 15844657471994422988 && word!(game.samus_x_accel_mode) == 0 {
            word!(game.samus_x_accel_mode = 2);
        }
        Samus_MoveX(pair.amt, game);
        if Samus_WallJumpCheck((g_word_909E9F as i16 as i32) << 16, game) == 0 {
            Samus_MoveY_WithSpeedCalc(game);
        }
    }
}
pub unsafe fn Samus_CheckStartFalling(game: &mut Game) {
    unsafe {
        if word!(game.samus_y_dir) == 1 && (word!(game.samus_y_speed) & 0x8000) != 0 {
            word! { game, 0; samus_y_subspeed, samus_y_speed }
            word!(game.samus_y_dir = 2);
        }
    }
}
pub unsafe fn Samus_MoveY_WithSpeedCalc(game: &mut Game) {
    unsafe {
        let mut amt: i32 = (words!(game.samus_y_speed, samus_y_subspeed)) as i32;
        if word!(game.samus_y_dir) == 2 {
            if word!(game.samus_y_speed) != 5 {
                words!(
                    game.samus_y_speed,
                    samus_y_subspeed += words!(game.samus_y_accel, samus_y_subaccel)
                );
            }
        } else {
            words!(
                game.samus_y_speed,
                samus_y_subspeed += -(words!(i32, game.samus_y_accel, samus_y_subaccel)) as u32
            );
        }
        if word!(game.samus_y_dir) != 2 {
            amt = -amt;
        }
        amt = (amt as u32).wrapping_add(words!(
            game.extra_samus_y_displacement,
            extra_samus_y_subdisplacement
        )) as i32;
        if amt < 0 {
            Samus_MoveUp(amt, game);
        } else {
            Samus_MoveDown(amt, game);
        };
    }
}
pub unsafe fn Samus_FallingMovement(game: &mut Game) {
    unsafe {
        crate::verify_probe::hit("samus.move.falling");
        Samus_HandleExtraRunspeedX(game);
        let pair: Pair_Bool_Amt =
            Samus_CalcBaseSpeed_NoDecel_X(Samus_DetermineSpeedTableEntryPtr_X(game), game);
        move_samus_air_x::<false>(pair.amt, game);
        Samus_CheckStartFalling(game);
        Samus_MoveY_WithSpeedCalc(game);
    }
}

pub unsafe fn Samus_MorphedFallingMovement(game: &mut Game) {
    unsafe {
        crate::verify_probe::hit("samus.ball.falling");
        let mut pair: Pair_Bool_Amt =
            Samus_CalcBaseSpeed_NoDecel_X(Samus_DetermineSpeedTableEntryPtr_X(game), game);
        move_samus_air_x::<true>(pair.amt, game);
        Samus_CheckStartFalling(game);
        Samus_MoveY_WithSpeedCalc(game);
    }
}
pub unsafe fn Samus_MorphedBouncingMovement(game: &mut Game) {
    unsafe {
        crate::verify_probe::hit("samus.ball.bouncing");
        let mut pair: Pair_Bool_Amt =
            Samus_CalcBaseSpeed_NoDecel_X(Samus_DetermineSpeedTableEntryPtr_X(game), game);
        move_samus_air_x::<true>(pair.amt, game);
        if word!(game.knockback_dir) == 0 {
            if word!(game.extra_samus_y_displacement) != 0
                || word!(game.extra_samus_y_subdisplacement) != 0
            {
                word!(game.samus_y_dir = 2);
                word! { game, 0; samus_y_subspeed, samus_y_speed }
                let amt: i32 = (words!(
                    game.extra_samus_y_displacement,
                    extra_samus_y_subdisplacement
                )) as i32;
                if amt < 0 {
                    Samus_MoveUp(amt, game);
                } else {
                    Samus_MoveDown(amt + ((1i16 as i32) << 16), game);
                }
            } else {
                Samus_CheckStartFalling(game);
                Samus_MoveY_WithSpeedCalc(game);
            }
        }
    }
}
pub unsafe fn Samus_Move_NoSpeedCalc_Y(game: &mut Game) {
    unsafe {
        let mut amt: i32 = (words!(
            game.extra_samus_y_displacement,
            extra_samus_y_subdisplacement
        )) as i32;
        if amt == 0 {
            if word!(game.samus_pos_adjusted_by_slope_flag) == 0 {
                amt = (words!(game.samus_total_x_speed, samus_total_x_subspeed)) as i32;
            }
            Samus_MoveDown(amt + ((1i16 as i32) << 16), game);
        } else if amt < 0 {
            Samus_MoveUp(amt, game);
        } else {
            Samus_MoveDown(amt + ((1i16 as i32) << 16), game);
        };
    }
}
pub unsafe fn Samus_MoveExtraY(game: &mut Game) {
    unsafe {
        let amt: i32 = (words!(
            game.extra_samus_y_displacement,
            extra_samus_y_subdisplacement
        )) as i32;
        if amt != 0 {
            if amt < 0 {
                Samus_MoveUp(amt, game);
            } else {
                Samus_MoveDown(amt + ((1i16 as i32) << 16), game);
            }
        }
    }
}
pub unsafe fn Samus_CheckAndMoveY(game: &mut Game) -> u8 {
    unsafe {
        if word!(game.samus_y_dir) == 0 {
            return 0;
        }
        Samus_CheckStartFalling(game);
        Samus_MoveY_WithSpeedCalc(game);
        1
    }
}
pub use self::Samus_CheckAndMoveY as Samus_MoveY_Simple_;
pub unsafe fn Samus_Move_NoBaseSpeed_X(game: &mut Game) {
    unsafe {
        Samus_MoveX(0, game);
    }
}
pub unsafe fn Samus_MoveLeft(amt: i32, game: &mut Game) {
    unsafe {
        crate::verify_probe::hit("samus.move.left");
        move_samus_horizontal_collision::<true>(amt, game);
    }
}
pub unsafe fn Samus_MoveRight(amt: i32, game: &mut Game) {
    unsafe {
        crate::verify_probe::hit("samus.move.right");
        move_samus_horizontal_collision::<false>(amt, game);
    }
}
pub unsafe fn Samus_MoveUp(mut amt: i32, game: &mut Game) {
    unsafe {
        word!(game.samus_collision_direction = 2);
        let cres: CheckEnemyColl_Result = Samus_CheckSolidEnemyColl(-amt, game);
        word!(game.samus_collision_flag = cres.collision);
        amt = cres.amt;
        if word!(game.samus_collision_flag) != 0 {
            Samus_MoveUp_SetPoseCalcInput(game);
            Samus_MoveUp_NoColl(amt, game);
        } else {
            amt = crate::game::samus_room_contact_ram::move_down(-amt, game);
            launch_motion_ram::record_movement(game, MovementDirection::Up, amt);
            Samus_MoveUp_SetPoseCalcInput(game);
        };
    }
}
pub unsafe fn Samus_MoveDown(mut amt: i32, game: &mut Game) {
    unsafe {
        word!(game.samus_collision_direction = 3);
        let cres: CheckEnemyColl_Result = Samus_CheckSolidEnemyColl(amt, game);
        word!(game.samus_collision_flag = cres.collision);
        amt = cres.amt;
        if word!(game.samus_collision_flag) != 0 {
            Samus_MoveDown_SetPoseCalcInput(game);
            Samus_MoveDown_NoColl(amt, game);
        } else {
            amt = crate::game::samus_room_contact_ram::move_down(amt, game);
            launch_motion_ram::record_movement(game, MovementDirection::Down, amt);
            Samus_MoveDown_SetPoseCalcInput(game);
        };
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
            if ((word!(game.samus_x_pos) & 0x8000) == 0) {
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
static kSamus_HandleExtraRunspeedX_Tab0: [u16; 3] = [0, 0, 0];
static kSamus_HandleExtraRunspeedX_Tab1: [u16; 3] = [0x1000, 0x400, 0x400];
static kSamus_HandleExtraRunspeedX_Tab2: [u16; 3] = [7, 4, 4];
static kSamus_HandleExtraRunspeedX_Tab3: [u16; 3] = [0, 0, 0];
static kSamus_HandleExtraRunspeedX_Tab4: [u16; 3] = [2, 1, 0];
static kSamus_HandleExtraRunspeedX_Tab5: [u16; 3] = [0, 0, 0];
pub unsafe fn Samus_HandleExtraRunspeedX(game: &mut Game) {
    unsafe {
        let mut current_block: u64;
        if (word!(game.equipped_items) & 0x20) == 0 {
            let mut r18: u16 = Samus_GetBottom_R18(game);
            match liquid_at_height(game, r18) {
                LiquidAtHeight::Water => current_block = 3594490907146900115,
                LiquidAtHeight::LavaAcid => current_block = 3594490907146900115,
                LiquidAtHeight::None => current_block = 13183875560443969876,
            }
        } else {
            current_block = 13183875560443969876;
        }
        if current_block == 13183875560443969876 {
            if *ram_ptr!(game; 0xa1f) != 1
                || (word!(game.button_config_run_b) & word!(game.joypad1_lastkeys)) == 0
            {
                current_block = 3594490907146900115;
            } else {
                if (word!(game.equipped_items) & 0x2000) != 0 {
                    if word!(game.samus_has_momentum_flag) == 0 {
                        word! { game, 1;
                            samus_has_momentum_flag, special_samus_palette_timer,
                        }
                        word! {game; special_samus_palette_frame = 0;
                        speed_boost_counter = crate::rtl::rom_table_word!(game, 0x91b61f, 0); }
                    }
                    if ((word!(game.samus_x_extra_run_speed)) as i16)
                        .wrapping_sub((kSamus_HandleExtraRunspeedX_Tab2[0i32 as usize]) as i16)
                        >= 0
                        && ((word!(game.samus_x_extra_run_subspeed)) as i16)
                            .wrapping_sub((kSamus_HandleExtraRunspeedX_Tab3[0i32 as usize]) as i16)
                            >= 0
                    {
                        word! { game;
                            samus_x_extra_run_speed = kSamus_HandleExtraRunspeedX_Tab2[0i32 as usize];
                            samus_x_extra_run_subspeed =
                                kSamus_HandleExtraRunspeedX_Tab3[0i32 as usize];
                        }
                        current_block = 7368539786447134940;
                    } else {
                        current_block = 15768484401365413375;
                    }
                } else {
                    if word!(game.samus_has_momentum_flag) == 0 {
                        word! {game; samus_has_momentum_flag = 1; speed_boost_counter = 0; }
                    }
                    if ((word!(game.samus_x_extra_run_speed)) as i16)
                        .wrapping_sub((kSamus_HandleExtraRunspeedX_Tab4[0i32 as usize]) as i16)
                        >= 0
                        && ((word!(game.samus_x_extra_run_subspeed)) as i16)
                            .wrapping_sub((kSamus_HandleExtraRunspeedX_Tab5[0i32 as usize]) as i16)
                            >= 0
                    {
                        word! { game;
                            samus_x_extra_run_speed = kSamus_HandleExtraRunspeedX_Tab4[0i32 as usize];
                            samus_x_extra_run_subspeed =
                                kSamus_HandleExtraRunspeedX_Tab5[0i32 as usize];
                        }
                        current_block = 7368539786447134940;
                    } else {
                        current_block = 15768484401365413375;
                    }
                }
                if current_block == 7368539786447134940 {
                } else {
                    words!(
                        game.samus_x_extra_run_speed,
                        samus_x_extra_run_subspeed += words!((
                            kSamus_HandleExtraRunspeedX_Tab0[0i32 as usize],
                            kSamus_HandleExtraRunspeedX_Tab1[0i32 as usize]
                        ))
                    );
                    current_block = 7368539786447134940;
                }
            }
        }
        if current_block == 3594490907146900115 && word!(game.samus_has_momentum_flag) == 0 {
            word! { game, 0; samus_x_extra_run_speed, samus_x_extra_run_subspeed }
        }
        if (word!(game.speed_boost_counter) & 0xff00) == 1024 {
            word!(game.samus_contact_damage_index = 1);
        }
    }
}
pub unsafe fn Samus_MoveRight_NoColl(amt: i32, game: &mut Game) {
    unsafe {
        words!(game.samus_x_pos, samus_x_subpos += amt as u32);
        launch_motion_ram::record_movement(game, MovementDirection::Right, amt);
    }
}
pub unsafe fn Samus_MoveLeft_NoColl(amt: i32, game: &mut Game) {
    unsafe {
        words!(game.samus_x_pos, samus_x_subpos += -amt as u32);
        launch_motion_ram::record_movement(game, MovementDirection::Left, -amt);
    }
}
pub unsafe fn Samus_MoveDown_NoColl(amt: i32, game: &mut Game) {
    unsafe {
        words!(game.samus_y_pos, samus_y_subpos += amt as u32);
        launch_motion_ram::record_movement(game, MovementDirection::Down, amt);
    }
}
pub unsafe fn Samus_MoveUp_NoColl(amt: i32, game: &mut Game) {
    unsafe {
        words!(game.samus_y_pos, samus_y_subpos += -amt as u32);
        launch_motion_ram::record_movement(game, MovementDirection::Up, -amt);
    }
}
pub unsafe fn Samus_InitJump(game: &mut Game) {
    unsafe {
        crate::verify_probe::hit("samus.jump.normal");
        initialize_samus_jump(SamusJumpKind::Normal, game);
    }
}
pub unsafe fn Samus_InitWallJump(game: &mut Game) {
    unsafe {
        crate::verify_probe::hit("samus.jump.wall");
        initialize_samus_jump(SamusJumpKind::Wall, game);
    }
}

pub unsafe fn Samus_SetSpeedForKnockback_Y(game: &mut Game) {
    unsafe {
        crate::verify_probe::hit("samus.jump.knockback");
        initialize_samus_jump(SamusJumpKind::Knockback, game);
    }
}
pub unsafe fn Samus_InitBombJump(game: &mut Game) {
    unsafe {
        crate::verify_probe::hit("samus.jump.bomb");
        initialize_samus_jump(SamusJumpKind::Bomb, game);
    }
}
unsafe fn IsGreaterThanQuirked(vhi: u16, vlo: u16, cmphi: u16, cmplo: u16) -> bool {
    if ((vhi) as i16).wrapping_sub((cmphi) as i16) >= 0
        && (vhi != cmphi || ((vlo) as i16).wrapping_sub((cmplo) as i16) >= 0 && vlo != cmplo)
    {
        return true_0 != 0;
    }
    false_0 != 0
}

macro_rules! decelerate_samus_base_speed {
    ($game:ident, $speed_table:ident) => {
        let delta: i32 = (if *ram_ptr!($game; 0xb4c) != 0 {
            words!((
                (Mult8x8(*ram_ptr!($game; 0xb4c), (*$speed_table).decel as u8) >> 8),
                Mult8x8(
                    *ram_ptr!($game; 0xb4c),
                    byte!((*$speed_table).decel_sub; 1)
                )
            ))
        } else {
            words!((*$speed_table), decel, decel_sub)
        }) as i32;
        words!(
            $game.samus_x_base_speed,
            samus_x_base_subspeed += -delta as u32
        );
        if (word!($game.samus_x_base_speed) as i16) < 0 {
            word! { $game, 0;
                samus_x_base_speed, samus_x_base_subspeed, samus_x_accel_mode,
            }
        }
    };
}

pub unsafe fn Samus_CalcBaseSpeed_X(k: u16, game: &mut Game) -> i32 {
    unsafe {
        let sste = get_SamusSpeedTableEntry(k, game);
        if word!(game.samus_x_accel_mode) != 0 {
            decelerate_samus_base_speed!(game, sste);
        } else {
            words!(
                game.samus_x_base_speed,
                samus_x_base_subspeed += words!((*sste), accel, accel_sub)
            );
            if IsGreaterThanQuirked(
                word!(game.samus_x_base_speed),
                word!(game.samus_x_base_subspeed),
                (*sste).max_speed,
                (*sste).max_speed_sub,
            ) {
                word!(game.samus_x_base_speed = (*sste).max_speed);
                word!(game.samus_x_base_subspeed = (*sste).max_speed_sub);
            }
        }
        (words!(game.samus_x_base_speed, samus_x_base_subspeed)) as i32
    }
}
unsafe fn Samus_CalcBaseSpeed_NoDecel_X(k: u16, game: &mut Game) -> Pair_Bool_Amt {
    unsafe {
        let sste = get_SamusSpeedTableEntry(k, game);
        let mut rv: bool = false_0 != 0;
        if (word!(game.samus_x_accel_mode) & 1) != 0 {
            decelerate_samus_base_speed!(game, sste);
        } else {
            words!(
                game.samus_x_base_speed,
                samus_x_base_subspeed += words!((*sste), accel, accel_sub)
            );
            if IsGreaterThanQuirked(
                word!(game.samus_x_base_speed),
                word!(game.samus_x_base_subspeed),
                (*sste).max_speed,
                (*sste).max_speed_sub,
            ) {
                word!(game.samus_x_base_speed = (*sste).max_speed);
                word!(game.samus_x_base_subspeed = (*sste).max_speed_sub);
                rv = true_0 != 0;
            }
        }
        Pair_Bool_Amt {
            flag: rv,
            amt: (words!(game.samus_x_base_speed, samus_x_base_subspeed)) as i32,
        }
    }
}
pub unsafe fn Samus_DetermineSpeedTableEntryPtr_X(game: &mut Game) -> u16 {
    unsafe {
        if (word!(game.equipped_items) & 0x20) == 0 {
            let r18: u16 = Samus_GetBottom_R18(game);
            match liquid_at_height(game, r18) {
                LiquidAtHeight::Water => {
                    word!(game.samus_x_speed_table_pointer = addr_kSamusSpeedTable_Water_X as u16);
                }
                LiquidAtHeight::LavaAcid => {
                    word!(
                        game.samus_x_speed_table_pointer = addr_kSamusSpeedTable_LavaAcid_X as u16
                    );
                }
                LiquidAtHeight::None => {}
            }
        }
        (word!(game.samus_x_speed_table_pointer) as i32 + 12 * *ram_ptr!(game; 0xa1f) as i32) as u16
    }
}
pub unsafe fn Samus_DetermineGrappleSwingSpeed_X(game: &mut Game) -> u16 {
    unsafe {
        if (word!(game.equipped_items) & 0x20) != 0 {
            return addr_stru_909F31 as u16;
        }
        let bottom = Samus_GetBottom_R18(game);
        match liquid_at_height(game, bottom) {
            LiquidAtHeight::None => addr_stru_909F31 as u16,
            LiquidAtHeight::Water => addr_stru_909F3D as u16,
            LiquidAtHeight::LavaAcid => addr_kSamusSpeedTable_Normal_X as u16,
        }
    }
}
pub unsafe fn Samus_DetermineAccel_Y(game: &mut Game) {
    unsafe {
        static g_word_909EA1: u16 = 0x1c00;
        static g_word_909EA3: u16 = 0x800;
        static g_word_909EA5: u16 = 0x900;
        static g_word_909EA7: u16 = 0;
        static g_word_909EA9: u16 = 0;
        static g_word_909EAB: u16 = 0;
        if ((word!(game.equipped_items) & 0x20) == 0) {
            let mut r18: u16 = Samus_GetBottom_R18(game);
            match liquid_at_height(game, r18) {
                LiquidAtHeight::Water => {
                    word! {game; samus_y_subaccel = g_word_909EA3; samus_y_accel = g_word_909EA9; }
                    return;
                }
                LiquidAtHeight::LavaAcid => {
                    word! {game; samus_y_subaccel = g_word_909EA5; samus_y_accel = g_word_909EAB; }
                    return;
                }
                LiquidAtHeight::None => {}
            }
        }
        word! {game; samus_y_subaccel = g_word_909EA1; samus_y_accel = g_word_909EA7; }
    }
}
pub unsafe fn Samus_GrappleWallJumpCheck(amt: i32, game: &mut Game) -> u8 {
    unsafe {
        game.set_enemy_index_to_shake(0xffff);
        let direction = match game.samus_pose_x_dir() {
            4 => 0,
            8 => 1,
            _ => return 0,
        };
        game.set_samus_collision_direction(direction);
        let result = Samus_CheckSolidEnemyColl(amt, game);
        if result.collision == 0 {
            crate::game::samus_room_contact_ram::wall_jump(result.amt, game);
            if game.samus_collision_flag() == 0 {
                return 0;
            }
            let jump = game.button_config_jump_a();
            let pressed = game.joypad1_newkeys();
            return u8::from(jump & pressed != 0);
        }
        let jump = game.button_config_jump_a();
        let pressed = game.joypad1_newkeys();
        if jump & pressed == 0 {
            return 0;
        }
        let enemy = game.collision_detection_index();
        game.set_enemy_index_to_shake(enemy);
        1
    }
}
pub unsafe fn Samus_WallJumpCheck(amt: i32, game: &mut Game) -> u8 {
    unsafe {
        if !matches!(game.samus_last_different_pose_movement_type(), 3 | 20) {
            return 0;
        }
        let pose = game.samus_pose();
        let screw_attack =
            pose == kPose_81_FaceR_Screwattack as u16 || pose == kPose_82_FaceL_Screwattack as u16;
        let animation = game.samus_anim_frame();
        let threshold = if screw_attack { 27 } else { 11 };
        let ready = animation.wrapping_sub(threshold) & 0x8000 == 0;
        if ready {
            game.set_enemy_index_to_shake(0xffff);
        }
        let held = game.joypad1_lastkeys();
        // Left takes precedence when both directions are held.
        let left = if held & kButton_Left as u16 != 0 {
            true
        } else if held & kButton_Right as u16 != 0 {
            false
        } else {
            return 0;
        };
        game.set_samus_collision_direction(u16::from(left));
        let result = Samus_CheckSolidEnemyColl(amt, game);
        let enemy_contact = result.collision != 0;
        let mut distance = result.amt;
        if !enemy_contact {
            if left {
                distance = crate::game::samus_room_contact_ram::wall_jump(distance, game);
            } else {
                distance = distance.wrapping_neg();
                crate::game::samus_room_contact_ram::wall_jump(distance, game);
            }
            if game.samus_collision_flag() == 0 {
                return 0;
            }
        }
        if !ready {
            // Probes may change the pose before choosing the animation to hold.
            let pose = game.samus_pose();
            let frame = if pose == kPose_81_FaceR_Screwattack as u16
                || pose == kPose_82_FaceL_Screwattack as u16
            {
                26
            } else {
                10
            };
            game.set_samus_anim_frame_timer(1);
            game.set_samus_anim_frame(frame);
            return 0;
        }
        if game.button_config_jump_a() & game.joypad1_newkeys() == 0 {
            return 0;
        }
        if ((distance >> 16) - 8) & 0x8000 == 0 {
            return 0;
        }
        game.set_input_to_pose_calc(5);
        if enemy_contact {
            game.set_enemy_index_to_shake(game.collision_detection_index());
        }
        1
    }
}
pub unsafe fn Samus_Movement_07_Unused(game: &mut Game) {
    unsafe {
        word!(game.input_to_pose_calc = 0);
    }
}
static kSamusMovementHandlers: [HandlerFunc; 28] = callback_table! { nonnull HandlerFunc;
    Samus_Movement_00_Standing, Samus_Movement_01_Running, Samus_Movement_02_NormalJumping,
    Samus_Movement_03_SpinJumping, Samus_Movement_04_MorphBallOnGround,
    Samus_Movement_05_Crouching, Samus_Movement_06_Falling, Samus_Movement_07_Unused,
    Samus_Movement_08_MorphBallFalling, Samus_Movement_09_Unused,
    Samus_Movement_0A_KnockbackOrCrystalFlashEnding, Samus_Movement_0B_Unused,
    Samus_Movement_0C_Unused, Samus_Movement_0D_Unused, Samus_Movement_0E_TurningAroundOnGround,
    Samus_Movement_0F_CrouchingEtcTransition, Samus_Movement_10_Moonwalking,
    Samus_Movement_11_SpringBallOnGround, Samus_Movement_12_SpringBallInAir,
    Samus_Movement_13_SpringBallFalling, Samus_Movement_14_WallJumping,
    Samus_Movement_15_RanIntoWall, Samus_Movement_16_Grappling,
    Samus_Movement_17_TurningAroundJumping, Samus_Movement_18_TurningAroundFalling,
    Samus_Movement_19_DamageBoost, Samus_Movement_1A_GrabbedByDraygon,
    Samus_Movement_1B_ShinesparkEtc,
};
pub unsafe fn Samus_Movement_00_Standing(game: &mut Game) {
    unsafe {
        if word!(game.samus_pose) != 0
            && word!(game.samus_pose) as i32 != kPose_9B_FaceF_VariaGravitySuit as i32
        {
            if (word!(game.samus_pose) as i32 == kPose_01_FaceR_Normal as i32
                || word!(game.samus_pose) as i32 == kPose_02_FaceL_Normal as i32)
                && (word!(game.button_config_shoot_x) & word!(game.joypad1_lastkeys)) != 0
            {
                word! {game; samus_anim_frame_timer = 16; samus_anim_frame = 0; }
            }
            Samus_Move_NoBaseSpeed_X(game);
            Samus_Move_NoSpeedCalc_Y(game);
            Samus_CancelSpeedBoost(game);
            word! { game, 0;
                samus_x_extra_run_speed, samus_x_extra_run_subspeed, samus_x_base_speed,
                samus_x_base_subspeed, samus_x_accel_mode,
            }
        } else {
            if word!(game.elevator_status) != 0 {
                word!(game.samus_collision_direction = 2);
                crate::game::samus_room_contact_ram::move_down((1i16 as i32) << 16, game);
            }
            word!(game.input_to_pose_calc = 0);
        };
    }
}
pub unsafe fn Samus_Movement_01_Running(game: &mut Game) {
    unsafe {
        static kSamusFootstepFrame: [u8; 10] = [0, 0, 1, 0, 0, 0, 0, 1, 0, 0];
        Samus_HandleMovement_X(game);
        Samus_Move_NoSpeedCalc_Y(game);
        if word!(game.samus_anim_frame_timer) == 1
            && kSamusFootstepFrame[word!(game.samus_anim_frame) as usize] != 0
        {
            Samus_FootstepGraphics(game);
            if word!(game.cinematic_function) == 0
                && word!(game.boss_id) == 0
                && word!(game.samus_shine_timer) == 0
                && (word!(game.speed_boost_counter) & 0x400) == 0
            {
                QueueSfx3_Max6(6, game);
            }
        }
    }
}
pub unsafe fn Samus_Movement_02_NormalJumping(game: &mut Game) {
    unsafe {
        Samus_JumpingMovement(game);
    }
}
pub unsafe fn Samus_Movement_03_SpinJumping(game: &mut Game) {
    unsafe {
        static kSamusPhys_JumpMinYVelAir: u16 = 0x280;
        static kSamusPhys_JumpMaxYVelAir: u16 = 0x500;
        static kSamusPhys_JumpMinYVelWater: u16 = 0x80;
        static kSamusPhys_JumpMaxYVelWater: u16 = 0x500;
        let mut r18: u16 = 0;
        if (word!(game.samus_suit_palette_index) & 4) == 0 {
            let r20: u16 = Samus_GetTop_R20(game);
            if liquid_at_height(game, r20) != LiquidAtHeight::None {
                r18 = 1;
            }
        }
        if r18 == 0 {
            if (word!(game.equipped_items) & 0x200) != 0 && (word!(game.samus_y_dir) == 2) {
                let skip_jump_restart: bool = if word!(game.liquid_physics_type) != 0 {
                    (((*(ram_ptr!(game, u16; 0xb2c, 1))) as i16)
                        .wrapping_sub((kSamusPhys_JumpMinYVelWater) as i16)
                        < 0
                        || ((*(ram_ptr!(game, u16; 0xb2c, 1))) as i16)
                            .wrapping_sub((kSamusPhys_JumpMaxYVelWater) as i16)
                            >= 0)
                } else {
                    (((*(ram_ptr!(game, u16; 0xb2c, 1))) as i16)
                        .wrapping_sub((kSamusPhys_JumpMinYVelAir) as i16)
                        < 0
                        || ((*(ram_ptr!(game, u16; 0xb2c, 1))) as i16)
                            .wrapping_sub((kSamusPhys_JumpMaxYVelAir) as i16)
                            >= 0)
                };
                if skip_jump_restart {
                } else {
                    word!(game.UNUSED_word_7E0DFA = (word!(game.UNUSED_word_7E0DFA) & 0xff00 | 1));
                    if (word!(game.button_config_jump_a) & word!(game.joypad1_newkeys)) != 0 {
                        Samus_InitJump(game);
                    }
                }
            }
            if word!(game.samus_pose) as i32 == kPose_81_FaceR_Screwattack as i32
                || word!(game.samus_pose) as i32 == kPose_82_FaceL_Screwattack as i32
            {
                word!(game.samus_contact_damage_index = 3);
            } else if word!(game.flare_counter) as i32 - 60i32 & 0x8000 == 0 {
                word!(game.samus_contact_damage_index = 4);
            }
        } else if word!(game.samus_anim_frame_timer) == 1
            && byte!(@pointer rom_ptr!(game; 0x90a514); word!(game.samus_anim_frame) as isize) != 0
        {
            QueueSfx1_Max6(0x2f, game);
        }
        Samus_SpinJumpMovement(game);
    }
}

macro_rules! move_grounded_morph_ball {
    ($game:ident, $left_ground_pose:ident, $right_ground_pose:ident) => {
        if word!($game.samus_x_accel_mode) == 0 {
            let use_normal_movement: bool = if *ram_ptr!($game; 0xa1e) == 4 {
                !(word!($game.samus_pose) as i32 == $left_ground_pose as i32)
            } else {
                !(word!($game.samus_pose) as i32 == $right_ground_pose as i32)
            };
            if use_normal_movement {
            } else {
                Samus_Move_NoBaseSpeed_X($game);
                if Samus_CheckAndMoveY($game) & 1 == 0 {
                    Samus_Move_NoSpeedCalc_Y($game);
                    Samus_CancelSpeedBoost($game);
                    word! { $game, 0;
                        samus_x_extra_run_speed, samus_x_extra_run_subspeed, samus_x_base_speed,
                        samus_x_base_subspeed, samus_x_accel_mode,
                    }
                }
                return;
            }
        }
        Samus_HandleMovement_X($game);
    };
}

pub unsafe fn Samus_Movement_04_MorphBallOnGround(game: &mut Game) {
    unsafe {
        move_grounded_morph_ball!(
            game,
            kPose_41_FaceL_Morphball_Ground,
            kPose_1D_FaceR_Morphball_Ground
        );
        if Samus_CheckAndMoveY(game) & 1 == 0 {
            Samus_Move_NoSpeedCalc_Y(game);
        }
    }
}
pub unsafe fn Samus_Movement_05_Crouching(game: &mut Game) {
    unsafe {
        Samus_Move_NoBaseSpeed_X(game);
        Samus_Move_NoSpeedCalc_Y(game);
        word! { game, 0;
            samus_x_extra_run_speed, samus_x_extra_run_subspeed, samus_x_base_speed,
            samus_x_base_subspeed, samus_x_accel_mode,
        }
    }
}
pub unsafe fn Samus_Movement_06_Falling(game: &mut Game) {
    unsafe {
        Samus_FallingMovement(game);
        if (word!(game.samus_pose) as i32 == kPose_29_FaceR_Fall as i32
            || word!(game.samus_pose) as i32 == kPose_2A_FaceL_Fall as i32
            || word!(game.samus_pose) as i32 == kPose_67_FaceR_Fall_Gun as i32
            || word!(game.samus_pose) as i32 == kPose_68_FaceL_Fall_Gun as i32)
            && word!(game.samus_y_speed) as i32 - 5i32 & 0x8000 == 0
            && word!(game.samus_anim_frame) as i32 - 5i32 & 0x8000 != 0
        {
            word! {game; samus_anim_frame_timer = 8; samus_anim_frame = 5; }
        }
    }
}
pub unsafe fn Samus_Movement_08_MorphBallFalling(game: &mut Game) {
    unsafe {
        if word!(game.joypad1_lastkeys) as i32 & (kButton_Left as i32 | kButton_Right as i32) == 0
            && word!(game.samus_x_accel_mode) == 0
        {
            Samus_CancelSpeedBoost(game);
            word! { game, 0;
                samus_x_extra_run_speed, samus_x_extra_run_subspeed, samus_x_base_speed,
                samus_x_base_subspeed, samus_x_accel_mode,
            }
        }
        if word!(game.used_for_ball_bounce_on_landing) != 0 {
            Samus_MorphedBouncingMovement(game);
        } else {
            Samus_MorphedFallingMovement(game);
        };
    }
}
pub fn Samus_Movement_09_Unused(game: &mut Game) {}
pub unsafe fn Samus_Movement_0A_KnockbackOrCrystalFlashEnding(game: &mut Game) {
    unsafe {
        word!(game.input_to_pose_calc = 0);
        Samus_Move_NoSpeedCalc_Y(game);
    }
}
pub use self::Samus_Movement_07_Unused as Samus_Movement_0B_Unused;
pub use self::Samus_Movement_07_Unused as Samus_Movement_0C_Unused;
pub fn Samus_Movement_0D_Unused(game: &mut Game) {}
pub fn nullsub_13(game: &mut Game) {}
static kSamusCrouchingEtcFuncs: [HandlerFunc; 12] = callback_table! { nonnull HandlerFunc;
    nullsub_13, nullsub_13, nullsub_13, nullsub_13, SamusCrouchingEtcFunc,
    SamusCrouchingEtcFunc, nullsub_13, nullsub_13, nullsub_13, nullsub_13,
    SamusCrouchingEtcFunc, SamusCrouchingEtcFunc,
};
pub unsafe fn Samus_Movement_0F_CrouchingEtcTransition(game: &mut Game) {
    unsafe {
        let apply_movement: bool = !(word!(game.samus_pose) as i32
            - kPose_F1_FaceR_CrouchTrans_AimU as i32
            & 0x8000
            != 0)
            || (!(word!(game.samus_pose) as i32 - kPose_DB as i32 & 0x8000 == 0)
                && ({
                    kSamusCrouchingEtcFuncs[(word!(game.samus_pose) as i32 - 53i32) as usize](game);
                    true
                }));
        if apply_movement {
            Samus_Move_NoBaseSpeed_X(game);
            if Samus_MoveY_Simple_(game) & 1 == 0 {
                Samus_Move_NoSpeedCalc_Y(game);
            }
        }
        if word!(game.input_to_pose_calc) == 1025 {
            word! { game, 0;
                samus_y_subspeed, samus_y_speed, samus_y_dir, used_for_ball_bounce_on_landing,
            }
        }
        word!(game.input_to_pose_calc = 0);
    }
}
pub unsafe fn SamusCrouchingEtcFunc(game: &mut Game) {
    unsafe {
        word! {game; enable_horiz_slope_coll = 3; UNUSEDword_7E0AA4 = 0; }
    }
}
pub unsafe fn Samus_Movement_0E_TurningAroundOnGround(game: &mut Game) {
    unsafe {
        Samus_HandleMovement_X(game);
        Samus_Move_NoSpeedCalc_Y(game);
        Samus_CancelSpeedBoost(game);
        word! { game, 0;
            samus_x_extra_run_speed, samus_x_extra_run_subspeed, input_to_pose_calc,
        }
    }
}
pub unsafe fn Samus_Movement_10_Moonwalking(game: &mut Game) {
    unsafe {
        Samus_HandleMovement_X(game);
        Samus_Move_NoSpeedCalc_Y(game);
    }
}
pub unsafe fn Samus_Movement_11_SpringBallOnGround(game: &mut Game) {
    unsafe {
        move_grounded_morph_ball!(
            game,
            kPose_7A_FaceL_Springball_Ground,
            kPose_79_FaceR_Springball_Ground
        );
        if Samus_CheckAndMoveY(game) & 1 == 0 {
            Samus_Move_NoSpeedCalc_Y(game);
        }
    }
}
pub unsafe fn Samus_Movement_12_SpringBallInAir(game: &mut Game) {
    unsafe {
        if word!(game.used_for_ball_bounce_on_landing) != 0 {
            Samus_MorphedBouncingMovement(game);
        } else {
            Samus_JumpingMovement(game);
        };
    }
}
pub use self::Samus_Movement_08_MorphBallFalling as Samus_Movement_13_SpringBallFalling;
pub unsafe fn Samus_Movement_14_WallJumping(game: &mut Game) {
    unsafe {
        if word!(game.samus_anim_frame) as i32 - 23i32 & 0x8000 != 0 {
            if word!(game.samus_anim_frame) as i32 - 3i32 & 0x8000 == 0
                && word!(game.flare_counter) as i32 - 60i32 & 0x8000 == 0
            {
                word!(game.samus_contact_damage_index = 4);
            }
        } else {
            word!(game.samus_contact_damage_index = 3);
        }
        Samus_JumpingMovement(game);
    }
}
pub unsafe fn Samus_Movement_15_RanIntoWall(game: &mut Game) {
    unsafe {
        Samus_Move_NoBaseSpeed_X(game);
        Samus_Move_NoSpeedCalc_Y(game);
        Samus_CancelSpeedBoost(game);
        word! { game, 0;
            samus_x_extra_run_speed, samus_x_extra_run_subspeed, samus_x_base_speed,
            samus_x_base_subspeed, samus_x_accel_mode,
        }
    }
}
pub unsafe fn Samus_Movement_16_Grappling(game: &mut Game) {
    unsafe {
        if word!(game.input_to_pose_calc) != 5 {
            word!(game.input_to_pose_calc = 0);
        }
    }
}
pub unsafe fn Samus_Movement_17_TurningAroundJumping(game: &mut Game) {
    unsafe {
        Samus_HandleMovement_X(game);
        if Samus_CheckAndMoveY(game) & 1 == 0 {
            Samus_Move_NoSpeedCalc_Y(game);
        }
        Samus_CancelSpeedBoost(game);
        word! { game, 0;
            samus_x_extra_run_speed, samus_x_extra_run_subspeed, input_to_pose_calc,
        }
    }
}
pub use self::Samus_Movement_02_NormalJumping as Samus_Movement_19_DamageBoost;
pub use self::Samus_Movement_07_Unused as Samus_Movement_1A_GrabbedByDraygon;
pub use self::Samus_Movement_07_Unused as Samus_Movement_1B_ShinesparkEtc;
pub use self::Samus_Movement_17_TurningAroundJumping as Samus_Movement_18_TurningAroundFalling;
pub unsafe fn Samus_HandleCooldown(game: &mut Game) {
    unsafe {
        if word!(game.time_is_frozen_flag) != 0 {
            word!(game.cooldown_timer = 32);
        } else if word!(game.cooldown_timer) != 0
            && ((word!(game.cooldown_timer) & 0x8000) != 0 || {
                word!(game.cooldown_timer -= 1);
                (word!(game.cooldown_timer) & 0x8000) != 0
            })
        {
            word!(game.cooldown_timer = 0);
        }
    }
}
pub unsafe fn UpdateBeamTilesAndPalette(game: &mut Game) {
    unsafe {
        let v0: u16 = (2 * (word!(game.equipped_beams) as i32 & 0xfff)) as u16;
        let mut v1: u16 = word!(game.vram_write_queue_tail);
        (*gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game)).size = 256;
        v1 = (v1).wrapping_add(2);
        (*gVramWriteEntry(v1 as i32, game)).size =
            crate::rtl::rom_table_word!(game, 0x90c3b1, (v0 >> 1) as isize);
        v1 = (v1).wrapping_add(2);
        crate::update_place! { previous fresh155 = v1 => v1.wrapping_add(1); }
        *(&raw mut (*(gVramWriteEntry as unsafe fn(i32, &mut Game) -> *mut VramWriteEntry)(
            fresh155 as i32,
            game,
        ))
        .size as *mut u8) = -102i32 as u8;
        (*gVramWriteEntry(v1 as i32, game)).size = addr_unk_606300 as u16;
        word!(game.vram_write_queue_tail = (v1).wrapping_add(2));
        WriteBeamPalette_Y(v0, game);
    }
}
pub unsafe fn WriteBeamPalette_A(a: u16, game: &mut Game) {
    unsafe {
        WriteBeamPalette_Y((2 * (a as i32 & 0xfff)) as u16, game);
    }
}
pub unsafe fn WriteBeamPalette_Y(j: u16, game: &mut Game) {
    unsafe {
        let r0: u16 = crate::rtl::rom_table_word!(game, 0x90c3c9, (j >> 1) as isize);
        let mut v2: u16 = 0;
        for v1 in (0u16..32).step_by(2) {
            *(ram_ptr!(game, u16; 0xc000)).offset(((v2 as i32 >> 1) + 224) as isize) =
                *(RomPtr_90((r0).wrapping_add(v1), game) as *mut u16);
            v2 = (v2).wrapping_add(2);
        }
    }
}
pub unsafe fn LoadProjectilePalette(a: u16, game: &mut Game) {
    unsafe {
        let r0: u16 = crate::rtl::rom_table_word!(game, 0x90c3c9, (a as i32 & 0xfff) as isize);
        let mut v2: u16 = 0;
        for v1 in (0u16..32).step_by(2) {
            *(ram_ptr!(game, u16; 0xc000)).offset(((v2 as i32 >> 1) + 224) as isize) =
                *(RomPtr_90((r0).wrapping_add(v1), game) as *mut u16);
            v2 = (v2).wrapping_add(2);
        }
    }
}
pub unsafe fn ResetProjectileData(game: &mut Game) {
    unsafe {
        word_transfers!(game, ascending, 10;
            le(projectile_timers, 0) <- value(0),
            le(projectile_x_pos, 0) <- value(0),
            le(projectile_y_pos, 0) <- value(0),
            le(projectile_dir, 0) <- value(0),
            le(projectile_bomb_x_speed, 0) <- value(0),
            le(projectile_bomb_y_speed, 0) <- value(0),
            le(projectile_x_radius, 0) <- value(0),
            le(projectile_y_radius, 0) <- value(0),
            le(projectile_type, 0) <- value(0),
            le(projectile_damage, 0) <- value(0),
            le(projectile_bomb_instruction_ptr, 0) <- value(0),
            le(projectile_bomb_instruction_timers, 0) <- value(0),
            le(projectile_variables, 0) <- value(0),
            le(projectile_spritemap_pointers, 0) <- value(0),
            le(projectile_bomb_pre_instructions, 0) <- value(((0x90b169i32) as u16)),
        );
        word! { game, 0; bomb_counter, cooldown_timer, projectile_counter, power_bomb_flag }
        if word!(game.hud_auto_cancel_flag) != 0 {
            word! { game, 0; hud_item_index, samus_auto_cancel_hud_item_index }
        }
        word! { game, 0;
            speed_echo_xpos[0], speed_echo_xpos[1], speed_echo_xpos[2], speed_echo_xpos[3],
            speed_echo_ypos[0], speed_echo_ypos[1], speed_echo_ypos[2], speed_echo_ypos[3],
            speed_echo_xspeed[0], speed_echo_xspeed[1], speed_echo_xspeed[2], speed_echo_xspeed[3],
            speed_echoes_index,
        }
        if word!(game.samus_special_super_palette_flags) != 0 {
            word!(game.samus_special_super_palette_flags = 0);
            Samus_LoadSuitTargetPalette(game);
        }
        if word!(game.hyper_beam_flag) != 0 {
            SpawnPalfxObject(addr_stru_8DE1F0 as u16, game);
        }
    }
}
#[forbid(unsafe_code)]
pub fn ClearProjectile(k: u16, game: &mut Game) {
    use crate::ram_map::checked_word as word;
    use crate::ram_map::{word_arrays::*, words};
    let slot = usize::from(k >> 1);
    for field in [
        projectile_x_pos,
        projectile_y_pos,
        projectile_bomb_x_subpos,
        projectile_bomb_y_subpos,
        projectile_dir,
        projectile_bomb_x_speed,
        projectile_bomb_y_speed,
        projectile_x_radius,
        projectile_y_radius,
        projectile_type,
        projectile_damage,
        projectile_bomb_instruction_ptr,
        projectile_bomb_instruction_timers,
        projectile_variables,
        projectile_spritemap_pointers,
    ] {
        word!(game, field + 2 * slot; = 0);
    }
    word!(game.projectile_bomb_pre_instructions[slot] = 0xb169);
    let counter = if (k as i16).wrapping_sub(10) >= 0 {
        words::bomb_counter
    } else {
        words::projectile_counter
    };
    word!(game, counter; = word!(game, counter;).wrapping_sub(1));
    if word!(game, counter;) & 0x8000 != 0 {
        word!(game, counter; = 0);
    }
}
#[forbid(unsafe_code)]
pub fn KillProjectile(k: u16, game: &mut Game) {
    crate::game::projectile_lifecycle_ram::retire(k, game);
}
crate::game::callback_dispatch! {
    pub unsafe fn CallProjPreInstr(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { Unreachable(game); };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    [9491666] ProjPreInstr_SpeedEcho [] {
        let mut v5: i16 = 0;
        let v1: i32 = k as i32 >> 1;
        word!(game.projectile_bomb_x_speed[v1] += 8);
        let pt: Point16U = Projectile_SinLookup(
            word!(game.projectile_variables[v1]),
            byte!(@pointer (ram_ptr!(game, u16; 0xbdc)).offset(v1 as isize); 0) as u16,
        );
        let v2: u16 = (pt.x).wrapping_add(word!(game.samus_x_pos));
        *(ram_ptr!(game, u16; 0xaae, k as i32 as isize)) =
            (pt.x).wrapping_add(word!(game.samus_x_pos));
        word!(game.projectile_x_pos[v1] = v2);
        let mut v3: i16 = ((v2) as i16).wrapping_sub((word!(game.layer1_x_pos)) as i16);
        if v3 < 0
            || v3 as i32 - 256i32 & 0x8000 == 0
            || {
                let mut v4: u16 = (pt.y).wrapping_add(word!(game.samus_y_pos));
                word!(
                    game.speed_echo_xpos[(v1 + 3)] = (pt.y).wrapping_add(word!(game.samus_y_pos))
                );
                word!(game.projectile_y_pos[v1] = v4);
                v5 = ((v4) as i16).wrapping_sub((word!(game.layer1_y_pos)) as i16);
                v5 < 0
            }
            || v5 as i32 - 256i32 & 0x8000 == 0
        {
            word!(game.speed_echo_ypos[(v1 + 3)] = 0);
            *(ram_ptr!(game, u16; 0xaae, k as i32 as isize)) = 0;
            word!(game.speed_echo_xpos[(v1 + 3)] = 0);
            ClearProjectile(k, game);
        }
    };
    [9498579] ProjPreInstr_UnknownProj8027 [] {
        static kProjPreInstr_UnknownProj8027_X: [i16; 4] = [-4, -4, 4, 4];
        static kProjPreInstr_UnknownProj8027_Y: [i16; 4] = [4, -4, -4, 4];
        static kProjPreInstr_UnknownProj8027_X2: [i16; 4] = [0x80, 0x80, -0x80, -0x80];
        static kProjPreInstr_UnknownProj8027_Y2: [i16; 4] = [-0x80, 0x80, 0x80, -0x80];
        let v1: i32 = k as i32 >> 1;
        word!(game.projectile_x_pos[v1] += (kProjPreInstr_UnknownProj8027_X[v1 as usize]) as u16);
        let v2: u16 = ((kProjPreInstr_UnknownProj8027_Y[v1 as usize]) as u16)
            .wrapping_add(word!(game.projectile_y_pos[v1]));
        word!(game.projectile_y_pos[v1] = v2);
        if v2 == word!(game.samus_y_pos) {
            if word!(game.projectile_variables[v1]) == 1 {
                if k == 6 {
                    word!(game.samus_movement_handler = ((0x90f04bi32) as u16));
                }
                ClearProjectile(k, game);
            } else {
                let v3: i32 = k as i32 >> 1;
                word!(game.projectile_variables[v3] += 1);
                word! { game;
                    samus_shine_timer = 180;
                    timer_for_shine_timer = 1;
                    special_samus_palette_frame = 0;
                    projectile_x_pos[v3] = ((kProjPreInstr_UnknownProj8027_X2[v3 as usize]) as u16)
                        .wrapping_add(word!(game.samus_x_pos));
                    projectile_y_pos[v3] = ((kProjPreInstr_UnknownProj8027_Y2[v3 as usize]) as u16)
                        .wrapping_add(word!(game.samus_y_pos));
                }
            }
        }
    };
    @external {
        9492371 => [special_attacks::update(k, SpecialKind::Plasma, game)];
        9490185 => [special_attacks::update(k, SpecialKind::Ice, game)];
        9490298 => [special_attacks::update(k, SpecialKind::IceReleased, game)];
        9493000 => [special_attacks::update(k, SpecialKind::Wave, game)];
        9493254 => [special_attacks::update(k, SpecialKind::Spazer, game)];
        9493660 => [special_attacks::update(k, SpecialKind::SpazerReleased, game)];
        9492449 => [special_attacks::plasma_radius(k, false, game)];
        9492474 => [special_attacks::plasma_radius(k, true, game)];
        9492499 => [special_attacks::depart_plasma(k, game)];
        9482393 => [bomb_lifecycle::update(k, BombKind::Bomb, game)];
        9482414 => [bomb_lifecycle::update(k, BombKind::PowerBomb, game)];
        9492727 => [bomb_lifecycle::update(k, BombKind::Spread, game)];
        9482601 => [()];
        9481971 => [projectile_flight::update(k, Flight::NonWave, 4, game)];
        9482058 => [projectile_flight::collide(2, Flight::NonWave, game)];
        9482066 => [projectile_flight::collide(3, Flight::NonWave, game)];
        9482080 => [projectile_flight::collide(1, Flight::NonWave, game)];
        9482088 => [projectile_flight::update(k, Flight::Missile, 4, game)];
        9482183 => [projectile_flight::collide(2, Flight::Missile, game)];
        9482191 => [projectile_flight::collide(3, Flight::Missile, game)];
        9482205 => [projectile_flight::collide(1, Flight::Missile, game)];
        9482213 => [projectile_flight::update(k, Flight::SuperMissile, 2, game)];
        9482311 => [projectile_flight::collide(2, Flight::SuperMissile, game)];
        9482322 => [projectile_flight::collide(3, Flight::SuperMissile, game)];
        9482346 => [projectile_flight::collide(1, Flight::SuperMissile, game)];
        9482357 => [projectile_flight::update_extension(k, game)];
        9482435 => [projectile_flight::update(k, Flight::Wave, 4, game)];
        9482468 => [projectile_flight::update(k, Flight::Wave, 3, game)];
        9482499 => [projectile_flight::advance(k, Flight::Wave, game)];
        9482555 => [projectile_flight::collide(2, Flight::Wave, game)];
        9482563 => [projectile_flight::collide(3, Flight::Wave, game)];
        9482577 => [projectile_flight::collide(1, Flight::Wave, game)];
        9482585 => [projectile_flight::update(k, Flight::Wave, 0, game)];
    }
}
pub unsafe fn HandleProjectile(game: &mut Game) {
    unsafe {
        word!(game.projectile_index = 18);
        let mut i: i32 = 18;
        while i >= 0 {
            let v1: i32 = i >> 1;
            if word!(game.projectile_bomb_instruction_ptr[v1]) != 0 {
                CallProjPreInstr(
                    ((word!(game.projectile_bomb_pre_instructions[v1])) as u32) | (0x900000u32),
                    i as u16,
                    game,
                );
                advance(game);
                i = word!(game.projectile_index) as i32;
            }
            i -= 2;
            word!(game.projectile_index = i as u16);
        }
    }
}
pub unsafe fn HandleChargingBeamGfxAudio(game: &mut Game) {
    unsafe {
        if word!(game.hyper_beam_flag) != 0 {
            if word!(game.flare_counter) != 0 {
                let mut i: i32 = 4;
                while i >= 0 {
                    let mut v7: bool = *(ram_ptr!(game, u16; 0xcdc, i as isize)) == 1;
                    crate::update_place! { unaligned_word fresh17 = &raw mut (*(ram_ptr!(game, u16; 0xcdc, i as isize))) => (u16::from_le(fresh17.read_unaligned())).wrapping_sub(1); }
                    let v8: bool = (u16::from_le(fresh17.read_unaligned()) & 0x8000) != 0;
                    if v7 || v8 {
                        crate::update_place! { unaligned_word fresh20 [fresh21] = &raw mut (*(ram_ptr!(game, u16; 0xcd6, i as isize))) => (u16::from_le(fresh20.read_unaligned())).wrapping_sub(1); }
                        v7 = fresh21 == 1;
                        if v7 {
                            if i == 4 {
                                word!(game.flare_counter = 0);
                            }
                        } else {
                            *(ram_ptr!(game, u16; 0xcdc, i as isize)) = 3;
                        }
                    }
                    DrawFlareAnimationComponent(i as u16, game);
                    i -= 2;
                }
            }
        } else if (word!(game.flare_counter) as i16) > 0 {
            if word!(game.flare_counter) == 1 {
                word! { game, 0;
                    flare_animation_frame, flare_slow_sparks_anim_frame, flare_fast_sparks_anim_frame,
                }
                word! {game; flare_animation_timer = 3; flare_slow_sparks_anim_timer = 5;
                flare_fast_sparks_anim_timer = 4; }
            }
            if word!(game.flare_counter) as i32 - 15i32 & 0x8000 == 0 {
                if word!(game.flare_counter) == 16 {
                    QueueSfx1_Max9(8, game);
                }
                let mut v0: u16 = 0;
                do_until! { (((v9) as i16).wrapping_sub(4) >= 0);
                    let mut v1: i16 = ((*(ram_ptr!(game, u16; 0xcdc, v0 as i32 as isize)))
                        as i16)
                        .wrapping_sub(1);
                    *(ram_ptr!(game, u16; 0xcdc, v0 as i32 as isize)) =
                        v1 as u16;
                    if v1 < 0 {
                        let v2: u16 = (*(ram_ptr!(game, u16; 0xcd6, v0 as i32 as isize)))
                            .wrapping_add(1);
                        *(ram_ptr!(game, u16; 0xcd6, v0 as i32 as isize)) = v2;
                        let mut v3: u16 = v2;
                        let r0 = RomPtr_90(
                            crate::rtl::rom_table_word!(game, 0x90c481, (v0 >> 1) as isize),
                            game,
                        );
                        let mut v4: i16 = *r0.offset(v2 as isize) as i16;
                        if v4 == 255 {
                            *(ram_ptr!(game, u16; 0xcd6, v0 as i32 as isize)) = 0;
                            v3 = 0;
                        } else if v4 == 254 {
                            let v5: u16 = (*(ram_ptr!(game, u16; 0xcd6, v0 as i32 as isize)))
                                .wrapping_sub((*r0.offset((v3 as i32 + 1) as isize)) as u16);
                            *(ram_ptr!(game, u16; 0xcd6, v0 as i32 as isize)) = v5;
                            v3 = v5;
                        }
                        *(ram_ptr!(game, u16; 0xcdc, v0 as i32 as isize)) = *r0.offset(v3 as isize) as u16;
                    }
                    let mut v9: u16 = v0;
                    DrawFlareAnimationComponent(v0, game);
                    if word!(game.flare_counter) as i32 - 30i32 & 0x8000 != 0 {
                        break;
                    }
                    v0 = (v0).wrapping_add(2);
                }
            }
        }
    }
}
static kProjectileOriginOffsets_X: [i16; 13] =
    [2, 18, 15, 17, 3, -4, -17, -15, -18, -2, -4, -4, -4];
static kProjectileOriginOffsets_Y: [i16; 13] = [-28, -19, 1, 6, 17, 17, 6, 1, -20, -28, -20, -2, 8];
static kProjectileOriginOffsets2_X: [i16; 10] = [2, 19, 20, 18, 3, -4, -18, -20, -19, -2];
static kProjectileOriginOffsets2_Y: [i16; 10] = [-32, -22, -3, 6, 25, 25, 6, -3, -20, -32];
pub unsafe fn DrawFlareAnimationComponent(k: u16, game: &mut Game) {
    unsafe {
        static word_93A225: [u16; 3] = [0, 0x1e, 0x24];
        static word_93A22B: [u16; 3] = [0, 0x2a, 0x30];
        let mut r20: u16 = 0;
        let mut r18: u16 = byte!(@pointer ram_ptr!(game, u16; 0xcd6); k as i32 as isize) as u16;
        let mut v1: u16 = if *ram_ptr!(game; 0xa1e) == 4 {
            word_93A22B[(k >> 1) as usize]
        } else {
            word_93A225[(k >> 1) as usize]
        };
        let r22: u16 = (r18).wrapping_add(v1);
        let r24: u16 = (*(rom_ptr!(game, SamusPoseParams; 0x91b629))
            .offset(word!(game.samus_pose) as isize))
        .y_offset_to_gfx as u16;
        let mut v2: i16 = (*(rom_ptr!(game, SamusPoseParams; 0x91b629))
            .offset(word!(game.samus_pose) as isize))
        .direction_shots_fired as i16;
        if v2 != 255 && v2 != 16 {
            let v3: u16 = (2 * (v2 as i32 & 0xf)) as u16;
            let old_x: u16 = word!(game.samus_x_pos);
            let old_y: u16 = word!(game.samus_y_pos);
            if (word!(game.ceres_status) & 0x8000) != 0 {
                Samus_CalcPos_Mode7(game);
            }
            let v4: i32 = v3 as i32 >> 1;
            let mut v5: u16 = if *ram_ptr!(game; 0xa1f) == 1 {
                r20 = (word!(game.samus_x_pos) as i32
                    + kProjectileOriginOffsets2_X[v4 as usize] as i32
                    - word!(game.layer1_x_pos) as i32) as u16;
                (word!(game.samus_y_pos) as i32 + kProjectileOriginOffsets2_Y[v4 as usize] as i32
                    - r24 as i32
                    - word!(game.layer1_y_pos) as i32) as u16
            } else {
                r20 = (word!(game.samus_x_pos) as i32
                    + kProjectileOriginOffsets_X[v4 as usize] as i32
                    - word!(game.layer1_x_pos) as i32) as u16;
                (word!(game.samus_y_pos) as i32 + kProjectileOriginOffsets_Y[v4 as usize] as i32
                    - r24 as i32
                    - word!(game.layer1_y_pos) as i32) as u16
            };
            r18 = v5;
            if (v5 & 0xff00) == 0 {
                DrawBeamGrappleSpritemap(r22, r20, r18, game);
            }
            if (word!(game.ceres_status) & 0x8000) != 0 {
                word! {game; samus_y_pos = old_y; samus_x_pos = old_x; }
            }
        }
    }
}
pub unsafe fn ProjectileReflection(r20: u16, game: &mut Game) {
    unsafe {
        let v0: u16 = r20;
        let v1: i32 = r20 as i32 >> 1;
        let v2: u16 = word!(game.projectile_type[v1]);
        if (v2 & 0x100) != 0 {
            initialize(r20, Initialization::Projectile, game);
            word!(game.projectile_bomb_pre_instructions[v1] = ((0x90af68i32) as u16));
            word!(game.projectile_variables[v1] = 240);
        } else if (v2 & 0x200) != 0 {
            let k: u16 = r20;
            ClearProjectile(
                byte!(@pointer (ram_ptr!(game, u16; 0xc7c)).offset(v1 as isize); 0) as u16,
                game,
            );
            initialize(k, Initialization::Projectile, game);
            let v3: i32 = k as i32 >> 1;
            word!(game.projectile_bomb_pre_instructions[v3] = ((0x90afe5i32) as u16));
            word!(game.projectile_variables[v3] = 240);
        } else {
            projectile_flight::initialize_beam_speed(r20, game);
            initialize(v0, Initialization::Projectile, game);
            word!(
                game.projectile_bomb_pre_instructions[v1] =
                    crate::game::beam_firing_ram::charged_instruction(
                        (word!(game.projectile_type[v1]) & 15) as usize
                    )
            );
        };
    }
}
pub unsafe fn HudSelectionHandler_MorphBall(game: &mut Game) {
    unsafe {
        if (word!(game.button_config_shoot_x) & word!(game.joypad1_lastkeys)) != 0 {
            if word!(game.hud_item_index) == 3 {
                if (word!(game.power_bomb_flag) & 0x8000) == 0
                    && HudSelectionHandler_MorphBall_Helper2(game) & 1 != 0
                    && word!(game.samus_power_bombs) != 0
                {
                    if (word!(game.samus_power_bombs -= 1, reread) & 0x8000) == 0 {
                        word!(game.power_bomb_flag = -1i32 as u16);
                        let mut v2: u16 = 10;
                        while word!(game.projectile_type[(v2 >> 1)]) != 0 {
                            v2 = (v2).wrapping_add(2);
                            if (((v2) as i16).wrapping_sub(20) < 0) {
                                continue;
                            }
                            v2 = (v2).wrapping_sub(2);
                            break;
                        }
                        let r18: u16 = u16::swap_bytes(word!(game.hud_item_index));
                        let v4: i32 = v2 as i32 >> 1;
                        let mut v5: u16 = (r18) | (word!(game.projectile_type[v4]));
                        word!(game.projectile_type[v4] = v5);
                        let v6: u8 = byte!(v5; 1);
                        word!(game.projectile_dir[v4] = 0);
                        word!(game.projectile_x_pos[v4] = word!(game.samus_x_pos));
                        word!(game.projectile_y_pos[v4] = word!(game.samus_y_pos));
                        word!(game.projectile_variables[v4] = 60);
                        initialize(v2, Initialization::Bomb, game);
                        word!(game.projectile_bomb_pre_instructions[v4] = ((0x90b0aei32) as u16));
                        word!(
                            game.cooldown_timer =
                                crate::game::projectile_weapon_definitions::COOLDOWNS
                                    [(v6 as i32 & 0xf) as usize]
                                    as u16
                        );
                        if word!(game.samus_auto_cancel_hud_item_index) != 0 {
                            word! { game, 0;
                                hud_item_index, samus_auto_cancel_hud_item_index,
                            }
                        } else if word!(game.hud_item_index) == 3
                            && word!(game.samus_power_bombs) == 0
                        {
                            word!(game.hud_item_index = 0);
                        }
                    }
                }
            } else if HudSelectionHandler_MorphBall_Helper(game) & 1 != 0 {
                let mut v0: u16 = 10;
                while word!(game.projectile_type[(v0 >> 1)]) != 0 {
                    v0 = (v0).wrapping_add(2);
                    if (((v0) as i16).wrapping_sub(20) < 0) {
                        continue;
                    }
                    v0 = (v0).wrapping_sub(2);
                    break;
                }
                let v1: i32 = v0 as i32 >> 1;
                word! {game; projectile_type[v1] = 1280; projectile_dir[v1] = 0;
                projectile_x_pos[v1] = word!(game.samus_x_pos);
                projectile_y_pos[v1] = word!(game.samus_y_pos); projectile_variables[v1] = 60; }
                initialize(v0, Initialization::Bomb, game);
                word!(game.projectile_bomb_pre_instructions[v1] = ((0x90b099i32) as u16));
                word!(
                    game.cooldown_timer =
                        crate::game::projectile_weapon_definitions::COOLDOWNS[5i32 as usize] as u16
                );
            }
        } else if word!(game.flare_counter) != 0 {
            QueueSfx1_Max9(2, game);
            word!(game.flare_counter = 0);
            clear_animation(game);
            Samus_LoadSuitPalette(game);
        }
    }
}
pub unsafe fn HudSelectionHandler_MorphBall_Helper(game: &mut Game) -> u8 {
    unsafe {
        if (word!(game.equipped_items) & 0x1000) != 0 {
            if word!(game.flare_counter) as i32 - 60i32 & 0x8000 != 0
                || word!(game.bomb_counter) != 0
            {
                return HudSelectionHandler_MorphBall_Helper2(game);
            }
            if (word!(game.joypad1_lastkeys) as i32 & kButton_Down as i32) != 0
                && (word!(game.bomb_spread_charge_timeout_counter) as i32 & 0xc0) - 192i32 & 0x8000
                    != 0
            {
                word!(game.bomb_spread_charge_timeout_counter += 1);
            } else {
                BombSpread(game);
                Samus_LoadSuitPalette(game);
                QueueSfx1_Max9(2, game);
            }
        }
        0
    }
}
pub unsafe fn HudSelectionHandler_MorphBall_Helper2(game: &mut Game) -> u8 {
    unsafe {
        if (word!(game.button_config_shoot_x) & word!(game.joypad1_newkeys)) == 0
            || word!(game.bomb_counter) != 0
                && (word!(game.bomb_counter) as i32 - 5i32 & 0x8000 == 0
                    || (word!(game.cooldown_timer) as u8) != 0)
        {
            if word!(game.flare_counter) != 0 {
                QueueSfx1_Max9(2, game);
                word!(game.flare_counter = 0);
                clear_animation(game);
                Samus_LoadSuitPalette(game);
            }
            0
        } else {
            word!(game.cooldown_timer += 1);
            word!(game.bomb_counter += 1);
            1
        }
    }
}
static kRunSwitchedToHudHandler: [Func_U8; 6] = callback_table! { nonnull Func_U8;
    SwitchToHudHandler_Nothing, SwitchToHudHandler_Missiles, SwitchToHudHandler_SuperMissiles,
    SwitchToHudHandler_PowerBombs, SwitchToHudHandler_Grapple, SwitchToHudHandler_Xray,
};
pub unsafe fn HandleSwitchingHudSelection(game: &mut Game) {
    unsafe {
        let mut r22: u16 = 0;
        let r18: u16 = word!(game.hud_item_index);
        let selection =
            if (word!(game.button_config_itemcancel_y) & word!(game.joypad1_newkeys)) != 0 {
                word!(game.samus_auto_cancel_hud_item_index = 0);
                Some(0)
            } else {
                r22 = ((word!(game.button_config_itemcancel_y) & word!(game.joypad1_lastkeys)) != 0)
                    as u16;
                if (word!(game.button_config_itemswitch) & word!(game.joypad1_newkeys)) == 0 {
                    None
                } else {
                    let next = (word!(game.hud_item_index)).wrapping_add(1);
                    Some(if word!(game.hud_item_index) as i32 - 5i32 & 0x8000 == 0 {
                        0
                    } else {
                        next
                    })
                }
            };
        if let Some(mut v0) = selection {
            word!(game.hud_item_index = v0);
            while kRunSwitchedToHudHandler[v0 as usize](game) & 1 != 0 {
                v0 = (word!(game.hud_item_index)).wrapping_add(1);
                word!(game.hud_item_index = v0);
                if v0 as i32 - 6i32 & 0x8000 == 0 {
                    v0 = 0;
                    word!(game.hud_item_index = 0);
                }
            }
            if r22 != 0 {
                word!(game.samus_auto_cancel_hud_item_index = word!(game.hud_item_index));
            } else {
                word!(game.samus_auto_cancel_hud_item_index = 0);
            }
        }
        if word!(game.hud_item_index) == r18 {
            let mut v1: u16 = (word!(game.hud_item_changed_this_frame)).wrapping_add(1);
            if word!(game.hud_item_changed_this_frame) as i32 - 2i32 & 0x8000 == 0 {
                v1 = 2;
            }
            word!(game.hud_item_changed_this_frame = v1);
        } else {
            word!(game.hud_item_changed_this_frame = 1);
        };
    }
}
pub unsafe fn SwitchToHudHandler_Nothing(game: &mut Game) -> u8 {
    unsafe {
        word!(game.flare_counter = 0);
        clear_animation(game);
        Samus_LoadSuitPalette(game);
        0
    }
}
pub unsafe fn SwitchToHudHandler_Missiles(game: &mut Game) -> u8 {
    unsafe {
        if word!(game.samus_missiles) == 0 {
            return 1;
        }
        word!(game.flare_counter = 0);
        clear_animation(game);
        Samus_LoadSuitPalette(game);
        0
    }
}
pub unsafe fn SwitchToHudHandler_SuperMissiles(game: &mut Game) -> u8 {
    unsafe {
        if word!(game.samus_super_missiles) == 0 {
            return 1;
        }
        word!(game.flare_counter = 0);
        clear_animation(game);
        Samus_LoadSuitPalette(game);
        0
    }
}
pub unsafe fn SwitchToHudHandler_PowerBombs(game: &mut Game) -> u8 {
    unsafe {
        if word!(game.samus_power_bombs) == 0 {
            return 1;
        }
        word!(game.flare_counter = 0);
        clear_animation(game);
        Samus_LoadSuitPalette(game);
        0
    }
}
pub unsafe fn SwitchToHudHandler_Grapple(game: &mut Game) -> u8 {
    unsafe {
        if (word!(game.equipped_items) & 0x4000) == 0 {
            return 1;
        }
        if word!(game.grapple_beam_function) as i32 == 0x9bc4f0 & 0xffff {
            Samus_LoadSuitPalette(game);
            word!(game.flare_counter = 0);
            clear_animation(game);
            word!(game.grapple_beam_function = ((0x9bc4f0i32) as u16));
        }
        0
    }
}
pub unsafe fn SwitchToHudHandler_Xray(game: &mut Game) -> u8 {
    unsafe {
        if (word!(game.equipped_items) & 0x8000) == 0 {
            return 1;
        }
        word!(game.flare_counter = 0);
        clear_animation(game);
        Samus_LoadSuitPalette(game);
        0
    }
}
pub unsafe fn HandleArmCannonOpenState(game: &mut Game) {
    unsafe {
        if *ram_ptr!(game; 0xaa7) != 0 || UpdateArmCannonIsOpenFlag(game) & 1 != 0 {
            AdvanceArmCannonFrame(game);
        }
        word!(
            game.arm_cannon_drawing_mode = *RomPtr_90(
                crate::rtl::rom_table_word!(game, 0x90c7df, word!(game.samus_pose) as isize),
                game,
            )
            .offset(1) as u16
        );
    }
}
pub unsafe fn UpdateArmCannonIsOpenFlag(game: &mut Game) -> u8 {
    unsafe {
        static kFlagShouldArmCannonBeOpen: [i8; 6] = [0, 1, 1, 0, 1, 0];
        if word!(game.hud_item_changed_this_frame) as i32 - 2i32 & 0x8000 != 0 {
            return 0;
        }
        let r18: u16 = kFlagShouldArmCannonBeOpen[word!(game.hud_item_index) as usize] as u16;
        if ((*ram_ptr!(game; 0xaa6)) as u16) == r18 {
            return 0;
        }
        if r18 != 0 {
            word!(game.arm_cannon_frame = 0);
        } else {
            word!(game.arm_cannon_frame = 4);
        }
        *(ram_ptr!(game, u16; 0xaa6)) = (r18) | (0x100u16);
        1
    }
}
pub unsafe fn AdvanceArmCannonFrame(game: &mut Game) {
    unsafe {
        if *ram_ptr!(game; 0xaa6) != 0 {
            if word!(game.arm_cannon_frame) as i32 - 2i32 & 0x8000 != 0 {
                word!(game.arm_cannon_frame += 1);
                return;
            }
            word!(game.arm_cannon_frame = 3);
        } else {
            if word!(game.arm_cannon_frame) != 1
                && ((word!(game.arm_cannon_frame)) as i16).wrapping_sub(1) >= 0
            {
                word!(game.arm_cannon_frame -= 1);
                return;
            }
            word!(game.arm_cannon_frame = 0);
        }
        *(ram_ptr!(game, u16; 0xaa6)) = *ram_ptr!(game; 0xaa6) as u16;
    }
}
pub unsafe fn Samus_ArmCannon_Draw(game: &mut Game) {
    unsafe {
        static kDrawArmCannon_Char: [u16; 10] = [
            0x281f, 0x281f, 0x281f, 0x681f, 0xa81f, 0xe81f, 0x281f, 0x681f, 0x681f, 0x681f,
        ];
        let mut v3: u16 = 0;
        let mut r22: u16 = 0;
        if word!(game.arm_cannon_frame) != 0
            && (word!(game.samus_invincibility_timer) == 0
                || (word!(game.nmi_frame_counter_word) & 1) == 0)
        {
            let v0: u16 =
                crate::rtl::rom_table_word!(game, 0x90c7df, word!(game.samus_pose) as isize);
            let v1 = RomPtr_90(v0, game);
            let v2: i16 = *v1 as i16;
            if (v2 as u16 & 0x80) != 0 {
                if word!(game.samus_anim_frame) != 0 {
                    v3 = (2 * (*v1.offset(2) as i32 & 0x7f)) as u16;
                } else {
                    v3 = (2 * (*v1 as i32 & 0x7f)) as u16;
                }
                r22 = (v0).wrapping_add(4);
            } else {
                v3 = (2u16).wrapping_mul((v2) as u16);
                r22 = (v0).wrapping_add(2);
            }
            let r24: u16 = kDrawArmCannon_Char[(v3 >> 1) as usize];
            let v4 = RomPtr_90(
                (r22 as i32 + 2 * word!(game.samus_anim_frame) as i32) as u16,
                game,
            );
            let r18: u16 = *v4 as i8 as u16;
            let r20: u16 = *v4.offset(1) as i8 as u16;
            r22 = (*(rom_ptr!(game, SamusPoseParams; 0x91b629))
                .offset(word!(game.samus_pose) as isize))
            .y_offset_to_gfx as u16;
            let v7: u16 = word!(game.oam_next_ptr);
            let v8: i16 = (r18 as i32 + word!(game.samus_x_pos) as i32
                - word!(game.layer1_x_pos) as i32) as i16;
            if v8 >= 0 && v8 as i32 - 256i32 & 0x8000 != 0 {
                let v9 = gOamEnt(word!(game.oam_next_ptr) as i32, game);
                (*v9).xcoord = v8 as u8;
                let v10: i16 = (r20 as i32 + word!(game.samus_y_pos) as i32
                    - r22 as i32
                    - word!(game.layer1_y_pos) as i32) as i16;
                if v10 >= 0 && v10 as i32 - 256i32 & 0x8000 != 0 {
                    (*v9).ycoord = v10 as u8;
                    *(&raw mut (*v9).charnum as *mut u16) = r24;
                    word!(game.oam_next_ptr = (v7).wrapping_add(4));
                }
            }
            let v11 = RomPtr_90(
                crate::rtl::rom_table_word!(game, 0x90c7df, word!(game.samus_pose) as isize),
                game,
            );
            let mut v12: u8 = *v11;
            if (v12 & 0x80) != 0 {
                v12 = (if word!(game.samus_anim_frame) != 0 {
                    *v11.offset(2) as i32
                } else {
                    *v11 as i32
                }) as u8;
            }
            let v13: u16 =
                (crate::rtl::rom_table_word!(game, 0x90c7a5, (v12 as i32 & 0x7f) as isize) as i32
                    + 2 * word!(game.arm_cannon_frame) as i32) as u16;
            let mut v14: u16 = word!(game.vram_write_queue_tail);
            (*gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game)).size = 32;
            v14 = (v14).wrapping_add(2);
            let v15: u16 = *(RomPtr_90(v13, game) as *mut u16);
            (*gVramWriteEntry(v14 as i32, game)).size = v15;
            v14 = (v14).wrapping_add(2);
            crate::update_place! { previous fresh15 = v14 => v14.wrapping_add(1); }
            *(&raw mut (*(gVramWriteEntry as unsafe fn(i32, &mut Game) -> *mut VramWriteEntry)(
                fresh15 as i32,
                game,
            ))
            .size as *mut u8) = -102i32 as u8;
            (*gVramWriteEntry(v14 as i32, game)).size = addr_unk_6061F0 as u16;
            word!(game.vram_write_queue_tail = (v14).wrapping_add(2));
        }
    }
}
pub unsafe fn Projectile_SinLookup(j: u16, a: u16) -> Point16U {
    unsafe {
        let r24: u16 = a;
        let r26: u16 = j;
        let mut v2: u16 = if j as i32 - 128i32 & 0x8000 != 0 {
            scale_sine_word((2u16).wrapping_mul(j), r24)
        } else {
            -(scale_sine_word(
                (2u16).wrapping_mul((((j) as u8).wrapping_add(0x80u8)) as u16),
                r24,
            ) as i32) as u16
        };
        let v3: i16 = (((r26) as u8).wrapping_sub(64)) as i16;
        let mut v4: u16 = if v3 as i32 - 128i32 & 0x8000 != 0 {
            scale_sine_word((2u16).wrapping_mul((v3) as u16), r24)
        } else {
            -(scale_sine_word(
                (2u16).wrapping_mul((((v3) as u8).wrapping_add(0x80u8)) as u16),
                r24,
            ) as i32) as u16
        };
        Point16U { x: v2, y: v4 }
    }
}
pub unsafe fn Projectile_Func7_Shinespark(game: &mut Game) {
    unsafe {
        word! {game; samus_movement_handler = ((0x90d068i32) as u16); samus_y_dir = 1;
        speed_boost_counter = 1024; }
        word! { game, 0; samus_y_subspeed, samus_y_speed, knockback_dir }
        word!(game.samus_x_extra_run_speed = 8);
        word! { game, 0;
            samus_x_extra_run_subspeed, samus_x_base_speed, samus_x_base_subspeed,
        }
        word!(game.substate = 7);
        word! { game, 0; suit_pickup_light_beam_pos, cooldown_timer }
        word! {game; timer_for_shinesparks_startstop = 30; samus_shine_timer = 60;
        timer_for_shine_timer = 6; }
        word! { game, 0; special_samus_palette_frame, bomb_jump_dir }
        if word!(game.flare_counter) != 0 {
            if word!(game.flare_counter) as i32 - 16i32 & 0x8000 == 0 {
                QueueSfx1_Max9(2, game);
            }
            word!(game.flare_counter = 0);
            clear_animation(game);
        }
    }
}
unsafe fn Samus_ClampSpeedHi(mut amt: i32, val: i32) -> u32 {
    if (amt >> 16) - val & 0x8000 == 0 {
        amt = (val) << 16 | amt & 0xffff;
    }
    amt as u32
}
pub unsafe fn Samus_ShinesparkMove_X(game: &mut Game) {
    unsafe {
        word!(game.samus_shine_timer = 15);
        words!(
            game.samus_x_extra_run_speed,
            samus_x_extra_run_subspeed += words!(game.samus_y_accel, samus_y_subaccel)
        );
        if word!(game.samus_x_extra_run_speed) as i32 - 15i32 & 0x8000 == 0 {
            words!(
                game.samus_x_extra_run_speed,
                samus_x_extra_run_subspeed = ((15i16 as i32) << 16) as u32
            );
        }
        let mut amt: i32 = 0;
        let move_right: bool = if *ram_ptr!(game; 0xa1e) == 4 {
            amt = Samus_ClampSpeedHi(-Samus_CalcDisplacementMoveLeft(amt, game), 15) as i32;
            let cres: CheckEnemyColl_Result = Samus_CheckSolidEnemyColl(amt, game);
            amt = cres.amt;
            if cres.collision != 0 {
                word!(game.samus_collision_flag = cres.collision);
                false
            } else {
                amt = -amt;
                true
            }
        } else {
            amt = Samus_ClampSpeedHi(Samus_CalcDisplacementMoveRight(amt, game), 25) as i32;
            let cres_0: CheckEnemyColl_Result = Samus_CheckSolidEnemyColl(amt, game);
            amt = cres_0.amt;
            !(cres_0.collision != 0)
                || ({
                    word!(game.samus_collision_flag = cres_0.collision);
                    false
                })
        };
        if move_right {
            crate::game::samus_room_contact_ram::move_right(amt, game);
            Samus_AlignYPosSlope(game);
        }
        let mut v4: i16 =
            ((word!(game.samus_x_pos)) as i16).wrapping_sub((word!(game.samus_prev_x_pos)) as i16);
        if ((word!(game.samus_x_pos)) as i16).wrapping_sub((word!(game.samus_prev_x_pos)) as i16)
            < 0
        {
            if v4 as i32 + 15 & 0x8000 != 0 {
                word!(game.samus_prev_x_pos = (word!(game.samus_x_pos)).wrapping_add(15));
            }
        } else if v4 as i32 - 16i32 & 0x8000 == 0 {
            word!(game.samus_prev_x_pos = (word!(game.samus_x_pos)).wrapping_sub(15));
        }
    }
}
pub unsafe fn Samus_ShinesparkMove_Y(game: &mut Game) {
    unsafe {
        word!(game.samus_shine_timer = 15);
        words!(
            game.substate,
            suit_pickup_light_beam_pos += words!(game.samus_y_accel, samus_y_subaccel)
        );
        words!(
            game.samus_y_speed,
            samus_y_subspeed += words!(game.substate, suit_pickup_light_beam_pos)
        );
        let mut amt: i32 =
            Samus_ClampSpeedHi((words!(game.samus_y_speed, samus_y_subspeed)) as i32, 14) as i32;
        amt = (amt as u32).wrapping_sub(words!(
            game.extra_samus_y_displacement,
            extra_samus_y_subdisplacement
        )) as i32;
        word!(game.samus_collision_direction = 2);
        let cres: CheckEnemyColl_Result =
            Samus_CheckSolidEnemyColl(Samus_ClampSpeedHi(amt, 15) as i32, game);
        amt = cres.amt;
        if cres.collision != 0 {
            word!(game.samus_collision_flag = cres.collision);
        } else {
            crate::game::samus_room_contact_ram::move_down(-amt, game);
        }
        if word!(game.samus_y_pos) as i32 - word!(game.samus_prev_y_pos) as i32 + 14 & 0x8000 != 0 {
            word!(game.samus_prev_y_pos = (word!(game.samus_y_pos)).wrapping_add(14));
        }
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
        crate::game::samus_pose::refresh_pose(game);
        if *ram_ptr!(game; 0xa1f) != 27 {
            return 1;
        }
        crate::game::samus_pose::record_pose_history(game);
        word!(game.samus_movement_handler = ((0x90d678i32) as u16));
        if word!(game.samus_input_handler) as i32 != 0x90e91d & 0xffff {
            word!(game.samus_input_handler = ((0x90e90ei32) as u16));
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
            Samus_RestoreHealth(0x32, game);
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
            Samus_RestoreHealth(0x32, game);
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
            Samus_RestoreHealth(0x32, game);
            let v0: bool = ((word!(game.substate)) as i16).wrapping_sub(1) < 0;
            if word!(game.substate -= 1, reread) == 0 || v0 {
                word! {game; samus_movement_handler = ((0x90d75bi32) as u16);
                samus_draw_handler = ((0x90eb52i32) as u16); samus_anim_frame_timer = 3;
                samus_anim_frame = 12; }
            }
        }
    }
}
static kBombSpread_Tab0: [u16; 5] = [0x78, 0x6e, 0x64, 0x6e, 0x78];
static kBombSpread_Tab1: [u16; 5] = [0x8100, 0x8080, 0, 0x80, 0x100];
static kBombSpread_Tab2: [u16; 5] = [0, 1, 2, 1, 0];
pub unsafe fn BombSpread(game: &mut Game) {
    unsafe {
        if word!(game.bomb_functions[0]) as i32 != 0x90d8f7 & 0xffff {
            for v0 in (10u16..20).step_by(2) {
                let v1: i32 = v0 as i32 >> 1;
                word! {game; projectile_type[v1] = -31488i32 as u16; projectile_dir[v1] = 0;
                projectile_bomb_pre_instructions[v1] = ((0x90d8f7i32) as u16); }
                initialize(v0, Initialization::Bomb, game);
                word! {game; projectile_x_pos[v1] = word!(game.samus_x_pos);
                projectile_bomb_x_subpos[v1] = 0; projectile_y_pos[v1] = word!(game.samus_y_pos);
                projectile_bomb_y_subpos[v1] = 0; }
                let v2: i32 = ((v0).wrapping_sub(10)) as i32 >> 1;
                word!(game.projectile_bomb_x_speed[v1] = kBombSpread_Tab1[v2 as usize]);
                word!(game.projectile_timers[v1] = kBombSpread_Tab3[v2 as usize]);
                let v3: u16 = -((kBombSpread_Tab2[v2 as usize] as i32
                    + (word!(game.bomb_spread_charge_timeout_counter) as i32 >> 6i32 & 3i32))
                    as i16 as i32) as u16;
                word! {game; projectile_bomb_y_speed[v1] = v3; projectile_unk_A[v1] = v3;
                projectile_variables[v1] = kBombSpread_Tab0[v2 as usize]; }
            }
            word!(game.bomb_counter = 5);
            word!(
                game.cooldown_timer = crate::game::projectile_weapon_definitions::COOLDOWNS
                    [(*((ram_ptr!(game, u16; 0xc18)).offset(5) as *mut u8).offset(1,) as i32 & 0xf)
                        as usize] as u16
            );
            word! { game, 0; bomb_spread_charge_timeout_counter, flare_counter }
        }
    }
}
static kHandleHudSpecificBehaviorAndProjs: [Func_V; 28] = callback_table! { nonnull Func_V;
    HudSelectionHandler_Normal, HudSelectionHandler_Normal, HudSelectionHandler_Normal,
    HudSelectionHandler_JumpEtc, HudSelectionHandler_MorphBall, HudSelectionHandler_Normal,
    HudSelectionHandler_Normal, HudSelectionHandler_MorphBall, HudSelectionHandler_MorphBall,
    HudSelectionHandler_MorphBall, HudSelectionHandler_JumpEtc, HudSelectionHandler_Grappling,
    HudSelectionHandler_Grappling, HudSelectionHandler_JumpEtc,
    HudSelectionHandler_TurningAround, HudSelectionHandler_CrouchEtcTrans,
    HudSelectionHandler_Normal, HudSelectionHandler_MorphBall, HudSelectionHandler_MorphBall,
    HudSelectionHandler_MorphBall, HudSelectionHandler_JumpEtc, HudSelectionHandler_Normal,
    HudSelectionHandler_Grappling, HudSelectionHandler_TurningAround,
    HudSelectionHandler_TurningAround, HudSelectionHandler_JumpEtc,
    HudSelectionHandler_GrabbedByDraygon, HudSelectionHandler_JumpEtc,
};
pub unsafe fn Samus_HandleHudSpecificBehaviorAndProjs(game: &mut Game) {
    unsafe {
        if word!(game.samus_pose) as i32 == kPose_00_FaceF_Powersuit as i32
            || word!(game.samus_pose) as i32 == kPose_9B_FaceF_VariaGravitySuit as i32
            || {
                Samus_HandleCooldown(game);
                HandleSwitchingHudSelection(game);
                kHandleHudSpecificBehaviorAndProjs[*ram_ptr!(game; 0xa1f) as usize](game);
                word!(game.time_is_frozen_flag) == 0
            }
        {
            HandleProjectile(game);
        }
    }
}
pub unsafe fn HudSelectionHandler_Normal(game: &mut Game) {
    unsafe {
        static kHudSelectionHandler_Normal: [Func_V; 7] = callback_table! { nonnull Func_V;
            update_input, fire_missile,
            fire_missile, update_input,
            HudSelectionHandler_Grappling, HudSelectionHandler_Xray, HudSelectionHandler_TurningAround,
        };
        let mut v0: u16 = if word!(game.grapple_beam_function) as i32 == 0x9bc4f0 & 0xffff {
            if word!(game.time_is_frozen_flag) != 0 {
                10
            } else {
                (2u16).wrapping_mul(word!(game.hud_item_index))
            }
        } else {
            8
        };
        kHudSelectionHandler_Normal[(v0 >> 1) as usize](game);
    }
}
pub unsafe fn HudSelectionHandler_Grappling(game: &mut Game) {
    unsafe {
        GrappleBeamHandler(game);
    }
}
pub unsafe fn HudSelectionHandler_TurningAround(game: &mut Game) {
    unsafe {
        if word!(game.new_projectile_direction_changed_pose) != 0 {
            HudSelectionHandler_Normal(game);
        } else if word!(game.grapple_beam_function) as i32 != 0x9bc4f0 & 0xffff {
            word!(game.grapple_beam_function = ((0x9bc856i32) as u16));
        }
    }
}
pub unsafe fn HudSelectionHandler_CrouchEtcTrans(game: &mut Game) {
    unsafe {
        static byte_90DDAA: [u8; 12] = [0, 0, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1];
        let handle_normal_selection: bool =
            (word!(game.samus_pose) as i32 - kPose_F1_FaceR_CrouchTrans_AimU as i32 & 0x8000 == 0)
                || ({
                    if word!(game.samus_pose) as i32 - kPose_DB as i32 & 0x8000 == 0 {
                        return;
                    }
                    !(byte_90DDAA[((word!(game.samus_pose)).wrapping_sub(53)) as usize] != 0)
                        || ({
                            if word!(game.grapple_beam_function) as i32 != 0x9bc4f0 & 0xffff {
                                word!(game.grapple_beam_function = ((0x9bc856i32) as u16));
                                HudSelectionHandler_Normal(game);
                            }
                            false
                        })
                });
        if handle_normal_selection {
            HudSelectionHandler_Normal(game);
        };
    }
}
pub unsafe fn HudSelectionHandler_JumpEtc(game: &mut Game) {
    unsafe {
        if word!(game.grapple_beam_function) as i32 != 0x9bc4f0 & 0xffff {
            word!(game.grapple_beam_function = ((0x9bc856i32) as u16));
            HudSelectionHandler_Normal(game);
        }
    }
}
pub unsafe fn HudSelectionHandler_Xray(game: &mut Game) {
    unsafe {
        if (word!(game.button_config_run_b) & word!(game.joypad1_lastkeys)) != 0 {
            XrayRunHandler(game);
        } else {
            update_input(game);
        };
    }
}
pub unsafe fn HudSelectionHandler_GrabbedByDraygon(game: &mut Game) {
    unsafe {
        if word!(game.samus_pose) as i32 == kPose_DB as i32 | kPose_04_FaceL_AimU as i32 {
            HudSelectionHandler_MorphBall(game);
        } else {
            HudSelectionHandler_Normal(game);
        };
    }
}
static kSamusHitInterrupts: [Func_U8; 28] = callback_table! { nonnull Func_U8;
    Samus_HitInterrupt_Stand, Samus_HitInterrupt_Stand, Samus_HitInterrupt_Stand,
    Samus_HitInterrupt_Stand, Samus_HitInterrupt_Ball, Samus_HitInterrupt_Stand,
    Samus_HitInterrupt_Falling, Samus_HitInterrupt_Unused, Samus_HitInterrupt_Ball,
    Samus_HitInterrupt_Ball, Samus_HitInterrupt_KnockbackGrapple, Samus_HitInterrupt_Turning,
    Samus_HitInterrupt_Turning, Samus_HitInterrupt_Stand, Samus_HitInterrupt_Turning,
    Samus_HitInterrupt_Turning, Samus_HitInterrupt_Stand, Samus_HitInterrupt_Ball,
    Samus_HitInterrupt_Ball, Samus_HitInterrupt_Ball, Samus_HitInterrupt_Stand,
    Samus_HitInterrupt_Stand, Samus_HitInterrupt_KnockbackGrapple, Samus_HitInterrupt_Turning,
    Samus_HitInterrupt_Turning, Samus_HitInterrupt_Turning, Samus_HitInterrupt_Turning,
    Samus_HitInterrupt_Shinespark,
};
pub unsafe fn Samus_HitInterruption(game: &mut Game) {
    unsafe {
        if word!(game.samus_knockback_timer) != 0 {
            if word!(game.debug_invincibility) as i32 - 7i32 & 0x8000 != 0 {
                if word!(game.time_is_frozen_flag) == 0
                    && word!(game.knockback_dir) == 0
                    && kSamusHitInterrupts[*ram_ptr!(game; 0xa1f) as usize](game) & 1 != 0
                {
                    word!(game.samus_special_transgfx_index = 1);
                }
            } else {
                word! { game, 0; samus_invincibility_timer, samus_knockback_timer }
            }
        } else if word!(game.knockback_dir) != 0 {
            if *ram_ptr!(game; 0xa1f) as i32 == kPose_0A_MoveL_NoAim as i32 {
                if word!(game.flare_counter) as i32 - 16i32 & 0x8000 == 0 {
                    QueueSfx1_Max6(0x41, game);
                }
                if *ram_ptr!(game; 0xa1e) == 4 {
                    word!(game.samus_new_pose_transitional = kPose_2A_FaceL_Fall as u16);
                } else {
                    word!(game.samus_new_pose_transitional = kPose_29_FaceR_Fall as u16);
                }
            } else {
                if word!(game.samus_hurt_switch_index) == 3 {
                    word!(game.samus_hurt_switch_index = 8);
                    return;
                }
                word!(game.samus_new_pose_transitional = word!(game.samus_pose));
            }
            word!(game.samus_hurt_switch_index = 1);
        } else if word!(game.bomb_jump_dir) != 0 {
            SetupBombJump(game);
        }
    }
}
pub unsafe fn Samus_HitInterrupt_Shinespark(game: &mut Game) -> u8 {
    unsafe {
        if word!(game.samus_pose) as i32 == kPose_E8_FaceR_Drained_CrouchFalling as i32
            || word!(game.samus_pose) as i32 == kPose_E9_FaceL_Drained_CrouchFalling as i32
        {
            word! {game; samus_anim_frame_timer = 17; samus_anim_frame = 26; }
        }
        word! { game, 0; samus_special_transgfx_index, knockback_dir }
        0
    }
}
pub unsafe fn Samus_HitInterrupt_KnockbackGrapple(game: &mut Game) -> u8 {
    unsafe {
        word!(game.samus_special_transgfx_index = 0);
        0
    }
}
pub unsafe fn Samus_HitInterrupt_Turning(game: &mut Game) -> u8 {
    unsafe {
        word! { game, 0; samus_special_transgfx_index, knockback_dir }
        0
    }
}
pub unsafe fn Samus_HitInterrupt_Falling(game: &mut Game) -> u8 {
    unsafe {
        if word!(game.frame_handler_gamma) as i32 != 0x90e41b & 0xffff {
            return Samus_HitInterrupt_Stand(game);
        }
        word! { game, 0; samus_special_transgfx_index, knockback_dir }
        0
    }
}
pub unsafe fn Samus_HitInterrupt_Stand(game: &mut Game) -> u8 {
    unsafe {
        if *ram_ptr!(game; 0xa1e) == 4 {
            word!(game.samus_new_pose_interrupted = 84);
        } else {
            word!(game.samus_new_pose_interrupted = 83);
        }
        1
    }
}
pub unsafe fn Samus_HitInterrupt_Ball(game: &mut Game) -> u8 {
    unsafe {
        word!(game.samus_new_pose_interrupted = word!(game.samus_pose));
        1
    }
}
pub unsafe fn Samus_HitInterrupt_Unused(game: &mut Game) -> u8 {
    unsafe {
        if *ram_ptr!(game; 0xa1e) == 4 {
            word!(game.samus_new_pose_interrupted = 52);
        } else {
            word!(game.samus_new_pose_interrupted = 51);
        }
        1
    }
}
static kSamus_MoveHandler_Knockback: [Func_V; 6] = callback_table! { nonnull Func_V;
    Samus_MoveHandler_Knockback_0, Samus_MoveHandler_Knockback_Up,
    Samus_MoveHandler_Knockback_Up, Samus_MoveHandler_Knockback_3,
    Samus_MoveHandler_Knockback_Down, Samus_MoveHandler_Knockback_Down,
};
pub unsafe fn Samus_ClearMoveVars(game: &mut Game) {
    unsafe {
        if word!(game.samus_collision_flag) != 0 {
            word! { game, 0;
                samus_x_accel_mode, samus_collides_with_solid_enemy, samus_is_falling_flag,
                UNUSED_word_7E0B1A, UNUSED_word_7E0B2A, samus_y_subspeed, samus_y_speed, samus_y_dir,
                UNUSED_word_7E0B38, samus_x_base_speed, samus_x_base_subspeed,
            }
            Samus_AlignBottomWithPrevPose(game);
        }
    }
}
static kSetupBombJump: [Func_U8; 28] = callback_table! { nonnull Func_U8;
    SetupBombJump_StandCrouch, SetupBombJump_1, SetupBombJump_2, SetupBombJump_2,
    SetupBombJump_4, SetupBombJump_StandCrouch, SetupBombJump_1, SetupBombJump_4,
    SetupBombJump_4, SetupBombJump_4, SetupBombJump_4, SetupBombJump_1, SetupBombJump_1,
    SetupBombJump_1, SetupBombJump_2, SetupBombJump_2, SetupBombJump_1, SetupBombJump_4,
    SetupBombJump_4, SetupBombJump_4, SetupBombJump_1, SetupBombJump_1, SetupBombJump_1,
    SetupBombJump_2, SetupBombJump_2, SetupBombJump_2, SetupBombJump_1A, SetupBombJump_1A,
};
pub unsafe fn SetupBombJump(game: &mut Game) {
    unsafe {
        if byte!(@pointer ram_ptr!(game, u16; 0xa56); 1) == 0
            && kSetupBombJump[*ram_ptr!(game; 0xa1f) as usize](game) & 1 != 0
        {
            word!(game.samus_special_transgfx_index = 3);
        }
    }
}
pub unsafe fn SetupBombJump_StandCrouch(game: &mut Game) -> u8 {
    unsafe {
        if word!(game.time_is_frozen_flag) == 0 {
            return SetupBombJump_1(game);
        }
        word!(game.bomb_jump_dir = 0);
        0
    }
}
pub unsafe fn SetupBombJump_1(game: &mut Game) -> u8 {
    unsafe {
        if *ram_ptr!(game; 0xa1e) == 4 {
            word!(game.samus_new_pose_interrupted = kPose_52_FaceL_Jump_NoAim_MoveF as u16);
        } else {
            word!(game.samus_new_pose_interrupted = kPose_51_FaceR_Jump_NoAim_MoveF as u16);
        }
        1
    }
}
pub use self::Samus_HitInterrupt_Ball as SetupBombJump_4;
pub unsafe fn SetupBombJump_2(game: &mut Game) -> u8 {
    unsafe {
        word!(game.samus_input_handler = ((0x90e913i32) as u16));
        SetupBombJump_1A(game)
    }
}
pub unsafe fn SetupBombJump_1A(game: &mut Game) -> u8 {
    unsafe {
        word!(game.bomb_jump_dir = 0);
        0
    }
}
static kSamus_MoveHandler_BombJumpMain: [Func_V; 4] = callback_table! { nonnull Func_V;
    Samus_MoveHandler_Knockback_0, Samus_HorizontalBombJump, Samus_VerticalBombJump,
    Samus_HorizontalBombJump,
};
pub unsafe fn Samus_HorizontalBombJump(game: &mut Game) {
    unsafe {
        crate::verify_probe::hit("samus.bomb.horizontal");
        Samus_BombJumpRisingXMovement_(game);
        Samus_VerticalBombJump(game);
    }
}
pub unsafe fn Samus_VerticalBombJump(game: &mut Game) {
    unsafe {
        Samus_BombJumpRisingYMovement_(game);
        if word!(game.samus_y_dir) == 2 || {
            Samus_MoveY_WithSpeedCalc(game);
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
            word!(game.frame_handler_gamma = ((0x90e114i32) as u16));
            DisablePaletteFx(game);
        }
        if word!(game.timer_status) != 0 {
            DrawTimer(game);
        }
    };
    [9494804] DrawTimer_ [] {
        DrawTimer(game);
    };
    [9494830] Samus_PushOutOfRidleysWay [] {
        if *ram_ptr!(game; 0xa1e) == 4 {
            word!(game.samus_pose = kPose_54_FaceL_Knockback as u16);
        } else {
            word!(game.samus_pose = kPose_53_FaceR_Knockback as u16);
        }
        crate::game::samus_pose::finish_pose_change(game);
        word!(checked_i32, game.samus_y_pos -= (21 - word!(game.samus_y_radius) as i32));
        if word!(game.samus_x_pos) as i32 - word!(game.layer1_x_pos) as i32 - 128i32 & 0x8000 != 0 {
            word!(game.samus_var62 = 1);
        } else {
            word!(game.samus_var62 = 2);
        }
        word!(game.samus_y_speed = 5);
        word! { game, 0; samus_y_subspeed, bomb_jump_dir }
        word! {game; frame_handler_gamma = ((0x90e1c8i32) as u16); samus_new_pose = -1i32 as u16;
            samus_new_pose_interrupted = -1i32 as u16;
            samus_new_pose_transitional = -1i32 as u16; }
        word! { game, 0;
            samus_momentum_routine_index, samus_special_transgfx_index, samus_hurt_switch_index,
        }
        ProcessTimer(game);
        if word!(game.timer_status) != 0 {
            DrawTimer(game);
        }
    };
    [9494984] Samus_Func4 [] {
        static off_90E1F7: [Option<Func_V>; 3] = callback_table! { unsafe Func_V;
            None, Samus_Func5, Samus_Func6,
        };
        if word!(game.samus_new_pose) as i32 == kPose_4F_FaceL_Dmgboost as i32
            || word!(game.samus_new_pose) as i32 == kPose_50_FaceR_Dmgboost as i32
        {
            word! {game; samus_new_pose = -1i32 as u16; samus_momentum_routine_index = 0; }
        }
        off_90E1F7[word!(game.samus_var62) as usize].expect("non-null function pointer")(game);
        word!(game.input_to_pose_calc = 0);
        ProcessTimer(game);
        if word!(game.timer_status) != 0 {
            DrawTimer(game);
        }
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
    [9495459] Samus_Func7 [] {
        Samus_BombJumpFallingXMovement_(game);
        if word!(game.samus_collision_flag) != 0 || {
            Samus_BombJumpFallingYMovement_(game);
            word!(game.samus_collision_flag) != 0
        } {
            word! {game; samus_movement_handler = ((0x90a337i32) as u16);
                frame_handler_gamma = ((0x90e90ei32) as u16); samus_var62 = 0; }
            Samus_ClearMoveVars(game);
            word! {game; samus_new_pose_interrupted = 65; samus_special_transgfx_index = 0; }
        }
    };
    [9495579] Samus_Func9 [] {
        if word!(game.samus_y_speed) as i32 - 5i32 & 0x8000 != 0 {
            words!(
                game.samus_y_speed,
                samus_y_subspeed += words!(game.samus_y_accel, samus_y_subaccel)
            );
        }
        if (word!(game.samus_pose) as i32 == kPose_29_FaceR_Fall as i32
            || word!(game.samus_pose) as i32 == kPose_2A_FaceL_Fall as i32
            || word!(game.samus_pose) as i32 == kPose_67_FaceR_Fall_Gun as i32
            || word!(game.samus_pose) as i32 == kPose_68_FaceL_Fall_Gun as i32)
            && word!(game.samus_y_speed) as i32 - 5i32 & 0x8000 == 0
        {
            word! {game; samus_anim_frame_timer = 16; samus_anim_frame = 4; }
        }
    };
    @external {
        9495422 => [()];
        9496846 => [()];
    }
}
pub unsafe fn RunFrameHandlerGamma(game: &mut Game) {
    unsafe {
        CallFrameHandlerGamma(
            ((word!(game.frame_handler_gamma)) as u32) | (0x900000u32),
            game,
        );
    }
}
pub unsafe fn Samus_SetPushedOutOfCeresRidley(game: &mut Game) {
    unsafe {
        word!(game.samus_movement_handler = ((0x90e90ei32) as u16));
        word!(game.frame_handler_gamma = ((0x90e12ei32) as u16));
    }
}
pub unsafe fn Samus_Func5(game: &mut Game) {
    unsafe {
        Samus_BombJumpFallingXMovement_(game);
        if word!(game.samus_collision_flag) != 0 {
            word! {game; samus_movement_handler = ((0x90a337i32) as u16);
            frame_handler_gamma = ((0x90e0e6i32) as u16); samus_var62 = 0; }
            Samus_ClearMoveVars(game);
        } else {
            Samus_BombJumpFallingYMovement_(game);
        };
    }
}
pub use self::Samus_Func5 as Samus_Func6;
pub unsafe fn Samus_SetGrabbedByDraygonPose(a: u16, game: &mut Game) {
    unsafe {
        if (a & 1) != 0 {
            word!(game.samus_pose = kPose_EC_FaceR_Draygon_NoMove_NoAim as u16);
        } else {
            word!(game.samus_pose = kPose_BA_FaceL_Draygon_NoMove_NoAim as u16);
        }
        crate::game::samus_pose::refresh_pose(game);
        word!(game.frame_handler_gamma = ((0x90e2a1i32) as u16));
        word!(game.samus_movement_handler = ((0x90e90ei32) as u16));
        word! { game, 0; substate, suit_pickup_light_beam_pos }
        *(ram_ptr!(game, u16; 0xdf0)) = 0;
        crate::game::samus_pose::record_pose_history(game);
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
        crate::game::samus_pose::refresh_pose(game);
        word!(game.samus_movement_handler = ((0x90a337i32) as u16));
        word!(game.frame_handler_gamma = ((0x90e90ei32) as u16));
        crate::game::samus_pose::record_pose_history(game);
        word! { game, 0; samus_x_base_speed, samus_x_base_subspeed }
        word! {game; samus_new_pose = -1i32 as u16; samus_new_pose_interrupted = -1i32 as u16;
        samus_new_pose_transitional = -1i32 as u16; }
        word! { game, 0;
            samus_momentum_routine_index, samus_special_transgfx_index, samus_hurt_switch_index,
            samus_y_speed, samus_y_subspeed, samus_y_dir, used_for_ball_bounce_on_landing,
            samus_x_accel_mode,
        }
        word!(game.samus_grapple_flags = (word!(game.samus_grapple_flags) & 0xfffd | 2));
    }
}
unsafe fn Samus_ClampSpeed(amt: i32) -> i32 {
    let mut r18: u16 = (amt >> 16) as u16;
    let r20: u16 = amt as u16;
    let v0: i16 = r18 as i16;
    if v0 < 0 {
        if v0 as i32 + 15 & 0x8000 != 0 {
            r18 = -15i32 as u16;
        }
    } else if v0 as i32 - 16i32 & 0x8000 == 0 {
        r18 = 15;
    }
    words!(i32, (r18, r20))
}
pub unsafe fn Samus_CalcDisplacementMoveLeft(mut amt: i32, game: &mut Game) -> i32 {
    unsafe {
        amt = Samus_CalcSpeed_X(amt as u32, game) as i32;
        word!(game.samus_collision_direction = 0);
        amt = (words!(
            game.extra_samus_x_displacement,
            extra_samus_x_subdisplacement
        ))
        .wrapping_sub(amt as u32) as i32;
        Samus_ClampSpeed(amt)
    }
}
pub unsafe fn Samus_CalcDisplacementMoveRight(mut amt: i32, game: &mut Game) -> i32 {
    unsafe {
        amt = Samus_CalcSpeed_X(amt as u32, game) as i32;
        word!(game.samus_collision_direction = 1);
        amt = (amt as u32).wrapping_add(words!(
            game.extra_samus_x_displacement,
            extra_samus_x_subdisplacement
        )) as i32;
        Samus_ClampSpeed(amt)
    }
}
pub unsafe fn Samus_CalcSpeed_X(mut amt: u32, game: &mut Game) -> u32 {
    unsafe {
        amt = amt.wrapping_add(words!(
            game.samus_x_extra_run_speed,
            samus_x_extra_run_subspeed
        ));
        amt >>= if word!(game.samus_x_speed_divisor) <= 4 {
            word!(game.samus_x_speed_divisor) as i32
        } else {
            4
        };
        words!(game.samus_total_x_speed, samus_total_x_subspeed = amt);
        amt
    }
}
pub unsafe fn Samus_ClearXSpeedIfColl(game: &mut Game) {
    unsafe {
        if word!(game.samus_collision_flag) != 0 {
            if word!(game.samus_collision_direction) != 0 {
                word!(game.samus_collides_with_solid_enemy = 8);
            } else {
                word!(game.samus_collides_with_solid_enemy = 4);
            }
            Samus_CancelSpeedBoost(game);
            word! { game, 0;
                samus_x_extra_run_speed, samus_x_extra_run_subspeed, samus_x_base_speed,
                samus_x_base_subspeed, samus_x_accel_mode,
            }
        } else {
            word! { game, 0; input_to_pose_calc, samus_collides_with_solid_enemy }
        };
    }
}
pub unsafe fn Samus_MoveUp_SetPoseCalcInput(game: &mut Game) {
    unsafe {
        if word!(game.samus_collision_flag) != 0 {
            word!(game.input_to_pose_calc = 4);
        } else {
            word!(game.input_to_pose_calc = 0);
        };
    }
}
static kSamus_MoveDown_SetPoseCalcInput_Tab0: [u8; 28] = [
    0, 0, 4, 4, 1, 0, 4, 2, 4, 4, 0, 0, 0, 0, 4, 4, 0, 3, 4, 4, 4, 0, 4, 4, 4, 4, 4, 4,
];
static kSamus_MoveDown_SetPoseCalcInput_Tab1: [u8; 28] = [
    4, 4, 0, 0, 4, 4, 0, 4, 1, 2, 0, 4, 4, 0, 4, 4, 4, 4, 3, 3, 0, 4, 4, 4, 4, 0, 4, 4,
];
pub unsafe fn Samus_MoveDown_SetPoseCalcInput(game: &mut Game) {
    unsafe {
        if word!(game.samus_collision_flag) != 0 {
            word!(game.input_to_pose_calc = 1);
            byte!(@pointer ram_ptr!(game, u16; 0xdc6); 1) =
                kSamus_MoveDown_SetPoseCalcInput_Tab1[*ram_ptr!(game; 0xa1f) as usize];
        } else if (word!(game.input_to_pose_calc) as u8) != 5 {
            word!(game.input_to_pose_calc = 2);
            byte!(@pointer ram_ptr!(game, u16; 0xdc6); 1) =
                kSamus_MoveDown_SetPoseCalcInput_Tab0[*ram_ptr!(game; 0xa1f) as usize];
        }
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallFrameHandlerAlfa(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [9496213] Samus_FrameHandlerAlfa_Func11 [] {
        word! {game; samus_new_pose = -1i32 as u16; samus_new_pose_interrupted = -1i32 as u16;
            samus_new_pose_transitional = -1i32 as u16; }
        word! { game, 0;
            samus_momentum_routine_index, samus_special_transgfx_index, samus_hurt_switch_index,
        }
        Samus_SetRadius(game);
        Samus_CallInputHandler(game);
        Samus_UpdateSuitPaletteIndex(game);
        Samus_DetermineAccel_Y(game);
        BlockInsideDetection(game);
        Samus_HandleHudSpecificBehaviorAndProjs(game);
        Samus_Func10(game);
    };
    [9496265] Samus_FrameHandlerAlfa_Func12 [] {
        word! {game; samus_new_pose = -1i32 as u16; samus_new_pose_interrupted = -1i32 as u16;
            samus_new_pose_transitional = -1i32 as u16; }
        word! { game, 0;
            samus_momentum_routine_index, samus_special_transgfx_index, samus_hurt_switch_index,
        }
        word! {game; controller1_input_for_demo = word!(game.joypad1_lastkeys);
            controller1_new_input_for_demo = word!(game.joypad1_newkeys);
            demo_backup_prev_controller_input = word!(game.joypad1_input_samusfilter);
            demo_backup_prev_controller_input_new = word!(game.joypad1_newinput_samusfilter); }
        Samus_SetRadius(game);
        Samus_UpdateSuitPaletteIndex(game);
        Samus_CallInputHandler(game);
        Samus_DetermineAccel_Y(game);
        BlockInsideDetection(game);
        Samus_HandleHudSpecificBehaviorAndProjs(game);
        Samus_Func10(game);
    };
    [9496339] Samus_FrameHandlerAlfa_Func13 [] {
        HandleProjectile(game);
        Samus_Func10(game);
    };
    @external {
        9496781 => [()];
    }
}
pub unsafe fn HandleControllerInputForGamePhysics(game: &mut Game) {
    unsafe {
        CallFrameHandlerAlfa(
            ((word!(game.frame_handler_alfa)) as u32) | (0x900000u32),
            game,
        );
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallFrameHandlerBeta(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [9496357] Samus_FrameHandlerBeta_Func17 [] {
        word!(game.samus_contact_damage_index = 0);
        RunSamusMovementHandler(game);
        crate::game::minimap::update(game);
        RunFrameHandlerGamma(game);
        Samus_Animate(game);
        Samus_HitInterruption(game);
        Samus_HandleTransFromBlockColl(game);
        Samus_HandleTransitions(game);
        Samus_HandlePalette(game);
        if crate::game::samus_periodic_damage_ram::apply(game).is_err() {
            InvalidInterrupt_Crash(game);
        }
        Samus_PauseCheck(game);
        Samus_LowHealthCheck_(game);
    };
    [9496530] HandleDemoRecorder_3 [] {
        if (word!(game.joypad2_new_keys) & 0x80) != 0 {
            word!(game.frame_handler_alfa = ((0x90e695i32) as u16));
            word!(game.frame_handler_beta = ((0x90e725i32) as u16));
            EnableEprojs(game);
            word!(game.time_is_frozen_flag = 0);
        }
    };
    [9496565] Samus_FrameHandlerBeta_Func14 [] {
        word!(game.samus_contact_damage_index = 0);
        RunSamusMovementHandler(game);
        crate::game::minimap::update(game);
        Samus_Animate(game);
        Samus_HitInterruption(game);
        Samus_HandleTransFromBlockColl(game);
        Samus_HandleTransitions(game);
        Samus_HandlePalette(game);
        word! {game; joypad1_lastkeys = word!(game.controller1_input_for_demo);
            joypad1_newkeys = word!(game.controller1_new_input_for_demo);
            joypad1_input_samusfilter = word!(game.demo_backup_prev_controller_input);
            joypad1_newinput_samusfilter = word!(game.demo_backup_prev_controller_input_new); }
    };
    [9496627] Samus_Func15 [] {
        word!(game.samus_contact_damage_index = 0);
        RunSamusMovementHandler(game);
        Samus_Animate(game);
        Samus_HitInterruption(game);
        Samus_HandleTransFromBlockColl(game);
        Samus_HandleTransitions(game);
        Samus_HandlePalette(game);
        word! {game; joypad1_lastkeys = word!(game.controller1_input_for_demo);
            joypad1_newkeys = word!(game.controller1_new_input_for_demo);
            joypad1_input_samusfilter = word!(game.demo_backup_prev_controller_input);
            joypad1_newinput_samusfilter = word!(game.demo_backup_prev_controller_input_new); }
    };
    [9496682] Samus_Func16 [] {
        Samus_SetRadius(game);
        crate::game::minimap::update(game);
        Samus_Animate(game);
        word! {game; elevator_status = 0; samus_prev_y_pos = word!(game.samus_y_pos); }
        if PlaySamusFanfare(game) & 1 != 0 {
            if word!(game.debug_invincibility) as i32 - 7i32 & 0x8000 != 0
                || (word!(game.joypad2_last) & 0x8000) == 0
            {
                word!(game.debug_invincibility = 0);
            }
            word!(game.frame_handler_alfa = ((0x90e695i32) as u16));
            word!(game.frame_handler_beta = ((0x90e725i32) as u16));
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
        crate::game::minimap::update(game);
    };
    [9496812] Samus_Func19 [] {
        word!(game.samus_contact_damage_index = 0);
        RunSamusMovementHandler(game);
        crate::game::minimap::update(game);
        Samus_Animate(game);
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
        DemoObjectInputHandler(game);
        Samus_InputHandler(game);
    };
    [9496870] HandleAutoJumpHack [] {
        let v0: u16 = word!(game.joypad1_newkeys);
        if word!(game.autojump_timer) != 0 && word!(game.autojump_timer) as i32 - 9i32 & 0x8000 != 0
        {
            word!(game.joypad1_newkeys |= (word!(game.button_config_jump_a)));
            word!(game.autojump_timer = 0);
        }
        Samus_InputHandler(game);
        word! {game; joypad1_newkeys = v0; samus_input_handler = ((0x90e913i32) as u16); }
    };
    @forward [pub unsafe fn (game: &mut Game)];
    Samus_CallInputHandler => CallSamusInputHandler(
        ((word!(game.samus_input_handler)) as u32) | (0x900000u32),
        game,
    );
    [9496851] Samus_InputHandler_E913 => Samus_InputHandler(game);
    [9496856] Samus_Func20_ => Samus_Func20(game);
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
        let amt: i32 = Samus_CalcBaseSpeed_X(Samus_DetermineGrappleSwingSpeed_X(game), game);
        if word!(game.samus_x_accel_mode) != 0
            || (word!(game.joypad1_lastkeys) as i32 & kButton_Right as i32) != 0
            || (word!(game.joypad1_lastkeys) as i32 & kButton_Left as i32) != 0
        {
            Samus_MoveX(amt, game);
        } else {
            word! { game, 0;
                samus_x_base_speed, samus_x_base_subspeed, samus_collision_flag,
            }
        }
        Samus_MoveY_WithSpeedCalc(game);
        if word!(game.input_to_pose_calc) != 0 {
            word!(game.samus_movement_handler = ((0x90a337i32) as u16));
        }
    };
    [9475275] Samus_HandleMovement_DrainedCrouching [] {
        Samus_MoveY_WithSpeedCalc(game);
        if word!(game.samus_collision_flag) != 0 {
            word! {game; samus_movement_handler = ((0x90a337i32) as u16);
                samus_anim_frame_timer = 8; samus_anim_frame = 7; }
            word! { game, 0; samus_y_subspeed, samus_y_speed }
        }
    };
    [9478967] Samus_MovementHandler_Normal [] {
        if word!(game.time_is_frozen_flag) == 0 {
            kSamusMovementHandlers[*ram_ptr!(game; 0xa1f) as usize]
                (game);
            Samus_UpdateSpeedEchoPos(game);
        }
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
            word!(game.samus_movement_handler = ((0x90d0abi32) as u16));
            word!(game.samus_input_handler = ((0x90e90ei32) as u16));
            word! { game, 0;
                speed_echoes_index, speed_echo_xspeed[0], speed_echo_xspeed[1], speed_echo_xpos[0],
                speed_echo_xpos[1],
            }
            QueueSfx3_Max9(0xf, game);
        }
    };
    [9490603] Samus_MoveHandlerVerticalShinespark [] {
        word! {game; samus_contact_damage_index = 2; samus_hurt_flash_counter = 8; }
        Samus_UpdateSpeedEchoPos(game);
        Samus_ShinesparkMove_Y(game);
        Samus_EndSuperJump(game);
        if word!(game.samus_health) as i32 - 30i32 & 0x8000 == 0 && {
            (word!(game.samus_health -= 1, reread) & 0x8000) != 0
        } {
            word!(game.samus_health = 0);
        }
    };
    [9490647] Samus_MoveHandler_Shinespark_Diag [] {
        word! {game; samus_contact_damage_index = 2; samus_hurt_flash_counter = 8; }
        Samus_UpdateSpeedEchoPos(game);
        Samus_ShinesparkMove_X(game);
        Samus_ShinesparkMove_Y(game);
        Samus_EndSuperJump(game);
        if word!(game.samus_health) as i32 - 30i32 & 0x8000 == 0 && {
            (word!(game.samus_health -= 1, reread) & 0x8000) != 0
        } {
            word!(game.samus_health = 0);
        }
    };
    [9490694] Samus_MoveHandler_Shinespark_Horiz [] {
        word! {game; samus_contact_damage_index = 2; samus_hurt_flash_counter = 8; }
        Samus_UpdateSpeedEchoPos(game);
        Samus_ShinesparkMove_X(game);
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
            let pt: Point16U = Projectile_SinLookup(
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
            word!(game.samus_movement_handler = ((0x90d3f3i32) as u16));
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
            word!(game.samus_movement_handler = ((0x90d40di32) as u16));
            word!(game.samus_draw_handler = ((0x90eb52i32) as u16));
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
                word!(game.projectile_bomb_pre_instructions[3] = ((0x90d4d2i32) as u16));
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
            word!(game.projectile_bomb_pre_instructions[4] = ((0x90d4d2i32) as u16));
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
            word!(game.samus_movement_handler = ((0x90d6cei32) as u16));
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
                word!(game.samus_input_handler = ((0x90e913i32) as u16));
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
        Samus_BombJumpRisingXMovement_(game);
        Samus_MoveY_WithSpeedCalc(game);
        Samus_ClearMoveVars(game);
    };
    [9494365] Samus_MoveHandler_Knockback_3 [] {
        Samus_MoveY_WithSpeedCalc(game);
        Samus_ClearMoveVars(game);
    };
    [9494372] Samus_MoveHandler_Knockback_Down [] {
        Samus_BombJumpRisingXMovement_(game);
        Samus_Move_NoSpeedCalc_Y(game);
        Samus_ClearMoveVars(game);
    };
    [9494565] Samus_MoveHandler_BombJumpStart [] {
        Samus_InitBombJump(game);
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
        word!(game.samus_movement_handler = ((0x90a337i32) as u16));
        if word!(game.samus_input_handler) as i32 != 0x90e91d & 0xffff {
            word!(game.samus_input_handler = ((0x90e913i32) as u16));
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
            word!(game.frame_handler_alfa = ((0x90e695i32) as u16));
            word!(game.samus_movement_handler = ((0x90a337i32) as u16));
        }
    };
    @external {
        9496846 => [()];
    }
}
pub unsafe fn RunSamusMovementHandler(game: &mut Game) {
    unsafe {
        CallSamusMovementHandler(
            ((word!(game.samus_movement_handler)) as u32) | (0x900000u32),
            game,
        );
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
pub unsafe fn Samus_Func10(game: &mut Game) {
    unsafe {
        launch_motion_ram::finish_projectile_phase(game);
        word! { game, 0; samus_anim_frame_skip, new_projectile_direction_changed_pose }
        word!(checked_i32, game.UNUSED_word_7E0DFA <<= 8);
        *(ram_ptr!(game, u16; 0xa10)) = *(ram_ptr!(game, u16; 0xa1e));
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
        HandleChargingBeamGfxAudio(game);
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
        HandleGrappleBeamFlare(game);
        if (word!(game.arm_cannon_drawing_mode) & 0xf) != 0 {
            if (word!(game.arm_cannon_drawing_mode) & 0xf) == 2 {
                HandleAtmosphericEffects(game);
                Samus_Draw(game);
                Samus_ArmCannon_Draw(game);
                UpdateGrappleBeamTilesAndIncrFlameCtr(game);
                if word!(game.grapple_beam_length) == 0 {
                    return;
                }
            } else {
                HandleAtmosphericEffects(game);
                Samus_ArmCannon_Draw(game);
                Samus_Draw(game);
                UpdateGrappleBeamTilesAndIncrFlameCtr(game);
                if word!(game.grapple_beam_length) == 0 {
                    return;
                }
            }
        } else {
            HandleAtmosphericEffects(game);
            Samus_Draw(game);
            UpdateGrappleBeamTilesAndIncrFlameCtr(game);
            if word!(game.grapple_beam_length) == 0 {
                return;
            }
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
        HandleArmCannonOpenState(game);
        CallSamusDrawHandler(
            ((word!(game.samus_draw_handler)) as u32) | (0x900000u32),
            game,
        );
    }
}
pub unsafe fn sub_90EB55(game: &mut Game) {
    unsafe {
        if (word!(game.arm_cannon_drawing_mode) & 0xf) != 0 {
            if (word!(game.arm_cannon_drawing_mode) & 0xf) == 2 {
                HandleAtmosphericEffects(game);
                Samus_Draw(game);
                Samus_ArmCannon_Draw(game);
            } else {
                HandleAtmosphericEffects(game);
                Samus_ArmCannon_Draw(game);
                Samus_Draw(game);
            }
            Samus_DrawEchoes(game);
        } else {
            HandleAtmosphericEffects(game);
            Samus_Draw(game);
            Samus_DrawEchoes(game);
        };
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
pub fn Samus_GetTop_R20(game: &Game) -> u16 {
    let y = game.samus_y_pos();
    let pose_offset = usize::from(game.samus_pose()) * 8;
    let radius = game.read_rom_byte(0x91b629, pose_offset + 6);
    y.wrapping_sub(u16::from(radius))
}
pub fn Samus_AlignBottomWithPrevPose(game: &mut Game) {
    let current_radius = game.read_rom_byte(0x91b629, usize::from(game.samus_pose()) * 8 + 6);
    let previous_pose = u16::from_le_bytes([game.g_ram[0xa20], game.g_ram[0xa21]]);
    let previous_radius = game.read_rom_byte(0x91b629, usize::from(previous_pose) * 8 + 6);
    let adjustment = u16::from(previous_radius).wrapping_sub(u16::from(current_radius));
    game.set_samus_y_pos(game.samus_y_pos().wrapping_add(adjustment));
    game.set_samus_prev_y_pos(game.samus_prev_y_pos().wrapping_add(adjustment));
}
pub unsafe fn Samus_UpdateSuitPaletteIndex(game: &mut Game) {
    unsafe {
        if (word!(game.equipped_items) & 0x20) != 0 {
            word!(game.samus_suit_palette_index = 4);
        } else if (word!(game.equipped_items) & 1) != 0 {
            word!(game.samus_suit_palette_index = 2);
        } else {
            word!(game.samus_suit_palette_index = 0);
        };
    }
}
static byte_90ED50: [u8; 28] = [
    0, 0, 0, 0, 1, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0,
];
static kSamus_FootstepGraphics: [Func_V; 8] = callback_table! { nonnull Func_V;
    Samus_FootstepGraphics_Crateria, Samus_FootstepGraphics_1, Samus_FootstepGraphics_1,
    Samus_FootstepGraphics_1, Samus_FootstepGraphics_Maridia, Samus_FootstepGraphics_1,
    Samus_FootstepGraphics_1, Samus_FootstepGraphics_1,
};
pub unsafe fn Samus_FootstepGraphics(game: &mut Game) {
    unsafe {
        kSamus_FootstepGraphics[word!(game.area_index) as usize](game);
    }
}
static byte_90EDC9: [u8; 16] = [1, 0, 0, 0, 0, 2, 0, 4, 0, 4, 4, 4, 4, 0, 4, 0];
pub unsafe fn Samus_FootstepGraphics_Crateria(game: &mut Game) {
    unsafe {
        if !(word!(game.cinematic_function) != 0
            || ((word!(game.room_index)) as i16).wrapping_sub(16) >= 0)
        {
            let use_default_footsteps: bool =
                if (byte_90EDC9[word!(game.room_index) as usize] & 1) == 0 {
                    if (byte_90EDC9[word!(game.room_index) as usize] & 2) != 0 {
                        !(word!(game.samus_y_pos) as i32 - 944i32 & 0x8000 == 0)
                    } else {
                        !((byte_90EDC9[word!(game.room_index) as usize] & 4) != 0)
                    }
                } else {
                    (word!(game.fx_type) != 10)
                };
            if use_default_footsteps {
            } else {
                Samus_FootstepGraphics_Maridia(game);
                return;
            }
        }
        Samus_FootstepGraphics_1(game);
    }
}
pub unsafe fn Samus_FootstepGraphics_Maridia(game: &mut Game) {
    unsafe {
        let r18: u16 = Samus_GetBottom_R18(game);
        if liquid_at_height(game, r18) != LiquidAtHeight::None {
            return;
        }
        if *ram_ptr!(game; 0xa1e) == 4 {
            word!(game.atmospheric_gfx_x_pos[0] = (word!(game.samus_x_pos)).wrapping_sub(12));
            word!(game.atmospheric_gfx_x_pos[1] = (word!(game.samus_x_pos)).wrapping_add(8));
        } else {
            word!(game.atmospheric_gfx_x_pos[0] = (word!(game.samus_x_pos)).wrapping_add(12));
            word!(game.atmospheric_gfx_x_pos[1] = (word!(game.samus_x_pos)).wrapping_sub(8));
        }
        word!(game.atmospheric_gfx_y_pos[0] = (word!(game.samus_y_pos)).wrapping_add(16));
        word!(game.atmospheric_gfx_y_pos[1] = (word!(game.samus_y_pos)).wrapping_add(16));
        word! { game, 256;
            atmospheric_gfx_frame_and_type[0], atmospheric_gfx_frame_and_type[1],
        }
        word!(game.atmospheric_gfx_anim_timer[0] = -32766i32 as u16);
        word!(game.atmospheric_gfx_anim_timer[1] = 3);
    }
}
pub unsafe fn Samus_FootstepGraphics_1(game: &mut Game) {
    unsafe {
        if (word!(game.speed_boost_counter) & 0xff00) == 1024 {
            let r18: u16 = Samus_GetBottom_R18(game);
            if liquid_at_height(game, r18) != LiquidAtHeight::None {
                return;
            }
            if *ram_ptr!(game; 0xa1e) == 4 {
                word!(game.atmospheric_gfx_x_pos[0] = (word!(game.samus_x_pos)).wrapping_sub(12));
                word!(game.atmospheric_gfx_x_pos[1] = (word!(game.samus_x_pos)).wrapping_add(8));
            } else {
                word!(game.atmospheric_gfx_x_pos[0] = (word!(game.samus_x_pos)).wrapping_add(12));
                word!(game.atmospheric_gfx_x_pos[1] = (word!(game.samus_x_pos)).wrapping_sub(8));
            }
            word!(game.atmospheric_gfx_y_pos[0] = (word!(game.samus_y_pos)).wrapping_add(16));
            word!(game.atmospheric_gfx_y_pos[1] = (word!(game.samus_y_pos)).wrapping_add(16));
            word! { game, 1792;
                atmospheric_gfx_frame_and_type[0], atmospheric_gfx_frame_and_type[1],
            }
            word!(game.atmospheric_gfx_anim_timer[0] = -32766i32 as u16);
            word!(game.atmospheric_gfx_anim_timer[1] = 3);
        }
    }
}
pub unsafe fn Samus_UpdateSpeedEchoPos(game: &mut Game) {
    unsafe {
        if (word!(game.speed_boost_counter) & 0xff00) == 1024
            && (word!(game.speed_echoes_index) & 0x8000) == 0
            && (word!(game.game_time_frames) & 3) == 0
        {
            let v0: u16 = word!(game.speed_echoes_index);
            let v1: i32 = word!(game.speed_echoes_index) as i32 >> 1;
            word!(game.speed_echo_xpos[v1] = word!(game.samus_x_pos));
            word!(game.speed_echo_ypos[v1] = word!(game.samus_y_pos));
            let mut v2: u16 = (v0).wrapping_add(2);
            if ((v2) as i16).wrapping_sub(4) >= 0 {
                v2 = 0;
            }
            word!(game.speed_echoes_index = v2);
        }
    }
}
pub fn Samus_PostGrappleCollisionDetect(game: &mut Game) {
    PostGrappleCollisionDetect_X(game);
    PostGrappleCollisionDetect_Y(game);
    if (game.distance_to_eject_samus_down() == 0 || game.distance_to_eject_samus_up() == 0)
        && game.distance_to_eject_samus_up() != 0
    {
        game.set_samus_y_pos(
            game.samus_y_pos()
                .wrapping_sub(game.distance_to_eject_samus_up()),
        );
        // Keep the sixteen-bit sign test, including extreme radius values.
        if game.samus_y_radius().wrapping_sub(16) & 0x8000 == 0 {
            PostGrappleCollisionDetect_Y(game);
            game.set_samus_y_pos(
                game.samus_y_pos()
                    .wrapping_sub(game.distance_to_eject_samus_up()),
            );
        }
    }
}
static kSamusCodeHandlers: [Func_U8; 32] = callback_table! { nonnull Func_U8;
    SamusCode_00_LockSamus, SamusCode_01_UnlockSamus, SamusCode_02_ReachCeresElevator,
    SamusCode_03, SamusCode_04, SamusCode_05_SetupDrained, SamusCode_06_LockToStation,
    SamusCode_07_SetupForElevator, SamusCode_08_SetupForCeresStart,
    SamusCode_08_SetupForZebesStart, SamusCode_0A_ClearDrawHandler,
    SamusCode_0B_DrawHandlerDefault, SamusCode_0C_UnlockFromMapStation,
    SamusCode_0D_IsGrappleActive, SamusCode_0E, SamusCode_0F_EnableTimerHandling, SamusCode_10,
    SamusCode_11_SetupForDeath, SamusCode_12_SetSuperPaletteFlag1,
    SamusCode_12_SetSuperPaletteFlag0, SamusCode_14, SamusCode_15_CalledBySuitAcquision,
    SamusCode_16, SamusCode_17_DisableRainbowSamusAndStandUp, SamusCode_18,
    SamusCode_17_FreezeDrainedSamus, SamusCode_1A, SamusCode_1B_CheckedLockSamus, SamusCode_1C,
    SamusCode_1D_ClearSoundInDoor, SamusCode_1E, SamusCode_1F,
};
pub unsafe fn CallSomeSamusCode(a: u16, game: &mut Game) -> u16 {
    unsafe {
        let code: u16 = kSamusCodeHandlers[(a as i32 & 0x1fi32) as usize](game) as u16;
        if code & 1 == 0 {
            return (code >> 1);
        }
        word! {game; samus_new_pose = -1i32 as u16; samus_new_pose_interrupted = -1i32 as u16;
        samus_new_pose_transitional = -1i32 as u16; }
        word! { game, 0;
            samus_momentum_routine_index, samus_special_transgfx_index, samus_hurt_switch_index,
        }
        -1i32 as u16
    }
}
pub unsafe fn SamusCode_00_LockSamus(game: &mut Game) -> u8 {
    unsafe {
        word!(game.frame_handler_alfa = ((0x90e713i32) as u16));
        word!(game.frame_handler_beta = ((0x90e8dci32) as u16));
        1
    }
}
pub unsafe fn SamusCode_01_UnlockSamus(game: &mut Game) -> u8 {
    unsafe {
        word!(game.frame_handler_alfa = ((0x90e695i32) as u16));
        word!(game.frame_handler_beta = ((0x90e725i32) as u16));
        1
    }
}
pub unsafe fn SamusCode_02_ReachCeresElevator(game: &mut Game) -> u8 {
    unsafe {
        if *ram_ptr!(game; 0xa1e) == 4 {
            word!(game.samus_pose = kPose_02_FaceL_Normal as u16);
        } else {
            word!(game.samus_pose = kPose_01_FaceR_Normal as u16);
        }
        word!(game.samus_anim_frame_skip = 0);
        crate::game::samus_pose::refresh_pose(game);
        word!(game.reached_ceres_elevator_fade_timer = 60);
        SamusCode_00_LockSamus(game)
    }
}
pub unsafe fn SamusCode_03(game: &mut Game) -> u8 {
    unsafe {
        if word!(game.grapple_beam_function) as i32 != 0x9bc4f0 & 0xffff {
            word!(game.grapple_beam_function = ((0x9bc8c5i32) as u16));
            return 0;
        }
        if *ram_ptr!(game; 0xa1f) as i32 != kMovementType_03_SpinJumping as i32
            && *ram_ptr!(game; 0xa1f) as i32 != kMovementType_14_WallJumping as i32
        {
            return 0;
        }
        if *ram_ptr!(game; 0xa1e) as i32 == kMovementType_04_MorphBallOnGround as i32 {
            word!(game.samus_pose = kPose_02_FaceL_Normal as u16);
        } else {
            word!(game.samus_pose = kPose_01_FaceR_Normal as u16);
        }
        word!(game.samus_anim_frame_skip = 0);
        crate::game::samus_pose::finish_pose_change(game);
        1
    }
}
pub use self::SwitchToHudHandler_Nothing as SamusCode_04_06_Common;
pub unsafe fn SamusCode_04(game: &mut Game) -> u8 {
    unsafe {
        word!(game.samus_charge_palette_index = 0);
        SamusCode_04_06_Common(game)
    }
}
pub unsafe fn SamusCode_06_LockToStation(game: &mut Game) -> u8 {
    unsafe {
        word!(game.frame_handler_alfa = ((0x90e713i32) as u16));
        word!(game.frame_handler_beta = ((0x90e8d6i32) as u16));
        if word!(game.flare_counter) as i32 - 15i32 & 0x8000 == 0 {
            QueueSfx1_Max15(2, game);
        }
        SamusCode_04_06_Common(game)
    }
}
pub unsafe fn SamusCode_07_SetupForElevator(game: &mut Game) -> u8 {
    unsafe {
        MakeSamusFaceForward(game);
        word! {game; frame_handler_beta = ((0x90e8eci32) as u16);
        samus_movement_handler = ((0x90a337i32) as u16);
        samus_draw_handler = ((0x90ec14i32) as u16);
        samus_input_handler = ((0x90e913i32) as u16); bomb_jump_dir = 0; }
        1
    }
}
pub unsafe fn SamusCode_08_SetupForCeresStart(game: &mut Game) -> u8 {
    unsafe {
        word! {game; frame_handler_alfa = ((0x90e8cdi32) as u16);
        frame_handler_beta = ((0x90e8dci32) as u16);
        samus_pose = kPose_00_FaceF_Powersuit as u16; samus_anim_frame_skip = 0; }
        crate::game::samus_pose::refresh_pose(game);
        Samus_LoadSuitPalette(game);
        word!(game.samus_draw_handler = ((0x90eb52i32) as u16));
        word!(game.samus_prev_pose = word!(game.samus_pose));
        word!(game.samus_last_different_pose = word!(game.samus_pose));
        *(ram_ptr!(game, u16; 0xa22)) = *(ram_ptr!(game, u16; 0xa1e));
        *(ram_ptr!(game, u16; 0xa26)) = *(ram_ptr!(game, u16; 0xa1e));
        word!(game.samus_y_pos = 0);
        let platform_parameter = SpawnEprojWithGfx(0, 0, addr_kEproj_CeresElevatorPad as u16, game);
        SpawnEprojWithGfx(
            platform_parameter,
            0,
            addr_kEproj_CeresElevatorPlatform as u16,
            game,
        );
        word!(game.debug_disable_minimap = 0);
        PlayRoomMusicTrackAfterAFrames(0x20, game);
        1
    }
}
pub unsafe fn SamusCode_08_SetupForZebesStart(game: &mut Game) -> u8 {
    unsafe {
        if (word!(game.equipped_items) & 0x20) != 0 {
            SpawnPalfxObject(addr_stru_8DE1FC as u16, game);
            word!(game.samus_pose = kPose_9B_FaceF_VariaGravitySuit as u16);
        } else if (word!(game.equipped_items) & 1) != 0 {
            SpawnPalfxObject(addr_stru_8DE1F8 as u16, game);
            word!(game.samus_pose = kPose_9B_FaceF_VariaGravitySuit as u16);
        } else {
            SpawnPalfxObject(addr_stru_8DE1F4 as u16, game);
            word!(game.samus_pose = kPose_00_FaceF_Powersuit as u16);
        }
        Samus_LoadSuitPalette(game);
        SamusFunc_F433(game);
        word! {game; samus_anim_frame_timer = 3; samus_anim_frame = 2; substate = 0; }
        1
    }
}
pub unsafe fn SamusCode_0A_ClearDrawHandler(game: &mut Game) -> u8 {
    unsafe {
        word!(game.samus_draw_handler = ((0x90e90ei32) as u16));
        0
    }
}
pub unsafe fn SamusCode_0B_DrawHandlerDefault(game: &mut Game) -> u8 {
    unsafe {
        word!(game.samus_draw_handler = ((0x90eb52i32) as u16));
        SamusCode_01_UnlockSamus(game)
    }
}
pub unsafe fn SamusCode_0C_UnlockFromMapStation(game: &mut Game) -> u8 {
    unsafe {
        SamusFunc_E633(game);
        if word!(game.frame_handler_beta) as i32 == 0x90e8d6 & 0xffff {
            word!(game.frame_handler_alfa = ((0x90e695i32) as u16));
            word!(game.frame_handler_beta = ((0x90e725i32) as u16));
        }
        1
    }
}
pub unsafe fn SamusCode_0D_IsGrappleActive(game: &mut Game) -> u8 {
    unsafe {
        ((if word!(game.grapple_beam_function) as i32 != 0x9bc4f0 & 0xffff {
            2
        } else {
            0
        }) as u8)
    }
}
pub unsafe fn SamusCode_0D_IsGrappleActive_A(game: &mut Game) -> u8 {
    unsafe { (word!(game.grapple_beam_function) as i32 != 0x9bc4f0 & 0xffff) as u8 }
}
pub unsafe fn SamusCode_0E(game: &mut Game) -> u8 {
    unsafe {
        word!(game.frame_handler_alfa = ((0x90e695i32) as u16));
        word!(game.frame_handler_beta = ((0x90e8aai32) as u16));
        1
    }
}
pub unsafe fn SamusCode_0F_EnableTimerHandling(game: &mut Game) -> u8 {
    unsafe {
        word!(game.frame_handler_gamma = ((0x90e0e6i32) as u16));
        0
    }
}
pub unsafe fn SamusCode_10(game: &mut Game) -> u8 {
    unsafe {
        if word!(game.frame_handler_beta) as i32 != 0x90e8d9 & 0xffff {
            word!(game.frame_handler_alfa = ((0x90e695i32) as u16));
            word!(game.frame_handler_beta = ((0x90e725i32) as u16));
        }
        1
    }
}
pub unsafe fn SamusCode_11_15_Common(game: &mut Game) -> u8 {
    unsafe {
        word! {game; frame_handler_alfa = ((0x90e713i32) as u16);
        frame_handler_beta = ((0x90e8cdi32) as u16); samus_draw_handler = ((0x90ec1di32) as u16); }
        1
    }
}
pub unsafe fn SamusCode_11_SetupForDeath(game: &mut Game) -> u8 {
    unsafe {
        DisablePaletteFx(game);
        SamusCode_11_15_Common(game)
    }
}
pub unsafe fn SamusCode_15_CalledBySuitAcquision(game: &mut Game) -> u8 {
    unsafe {
        crate::game::samus_pose::record_pose_history(game);
        word!(game.flare_counter = 0);
        clear_animation(game);
        Samus_LoadSuitPalette(game);
        SamusCode_11_15_Common(game)
    }
}
pub unsafe fn SamusCode_12_SetSuperPaletteFlag1(game: &mut Game) -> u8 {
    unsafe {
        word!(game.samus_special_super_palette_flags = 1);
        0
    }
}
pub unsafe fn SamusCode_12_SetSuperPaletteFlag0(game: &mut Game) -> u8 {
    unsafe {
        word!(game.samus_special_super_palette_flags = 0);
        Samus_LoadSuitPalette(game);
        0
    }
}
pub unsafe fn SamusCode_14(game: &mut Game) -> u8 {
    unsafe {
        if word!(game.samus_health) as i32 - 31i32 & 0x8000 != 0 {
            QueueSfx3_Max6(2, game);
        }
        if SamusCode_0D_IsGrappleActive_A(game) == 0 {
            if *ram_ptr!(game; 0xa1e) == 3 {
                if word!(game.samus_pose) as i32 == kPose_81_FaceR_Screwattack as i32
                    || word!(game.samus_pose) as i32 == kPose_82_FaceL_Screwattack as i32
                {
                    QueueSfx1_Max6(0x33, game);
                } else if word!(game.samus_pose) as i32 == kPose_1B_FaceR_SpaceJump as i32
                    || word!(game.samus_pose) as i32 == kPose_1C_FaceL_SpaceJump as i32
                {
                    QueueSfx1_Max6(0x3e, game);
                } else {
                    QueueSfx1_Max6(0x31, game);
                }
            }
        } else {
            QueueSfx1_Max6(6, game);
        }
        0
    }
}
pub unsafe fn SamusCode_05_SetupDrained(game: &mut Game) -> u8 {
    unsafe {
        word!(game.frame_handler_gamma = ((0x90e09bi32) as u16));
        Samus_SetupForBeingDrained(game)
    }
}
pub unsafe fn Samus_SetupForBeingDrained(game: &mut Game) -> u8 {
    unsafe {
        word! {game; samus_pose = kPose_54_FaceL_Knockback as u16; samus_anim_frame_skip = 0; }
        crate::game::samus_pose::finish_pose_change(game);
        word!(game.flare_counter = 0);
        clear_animation(game);
        Samus_LoadSuitPalette(game);
        word!(game.frame_handler_alfa = ((0x90e713i32) as u16));
        word!(game.frame_handler_beta = ((0x90e8d9i32) as u16));
        1
    }
}
pub unsafe fn SamusCode_18(game: &mut Game) -> u8 {
    unsafe {
        word!(game.frame_handler_gamma = ((0x90e0c5i32) as u16));
        Samus_SetupForBeingDrained(game)
    }
}
pub unsafe fn SamusCode_16(game: &mut Game) -> u8 {
    unsafe {
        word!(game.samus_special_super_palette_flags = 0x8000);
        word! { game, 1; special_samus_palette_frame, special_samus_palette_timer }
        word!(game.samus_charge_palette_index = 0);
        0
    }
}
pub unsafe fn SamusCode_17_DisableRainbowSamusAndStandUp(game: &mut Game) -> u8 {
    unsafe {
        word! { game, 0;
            samus_special_super_palette_flags, special_samus_palette_frame, special_samus_palette_timer,
            samus_charge_palette_index,
        }
        Samus_LoadSuitPalette(game);
        word! {game; samus_anim_frame_timer = 1; samus_anim_frame = 13; }
        0
    }
}
pub unsafe fn SamusCode_17_FreezeDrainedSamus(game: &mut Game) -> u8 {
    unsafe {
        word! {game; samus_anim_frame_timer = 1; samus_anim_frame = 28; }
        1
    }
}
pub unsafe fn SamusCode_1A(game: &mut Game) -> u8 {
    unsafe {
        word!(game.frame_handler_beta = ((0x90e902i32) as u16));
        0
    }
}
pub unsafe fn SamusCode_1B_CheckedLockSamus(game: &mut Game) -> u8 {
    unsafe {
        if word!(game.frame_handler_beta) as i32 == 0x90e8d9 & 0xffff {
            1
        } else {
            SamusCode_00_LockSamus(game)
        }
    }
}
pub unsafe fn SamusCode_1C(game: &mut Game) -> u8 {
    unsafe {
        let sound = if *ram_ptr!(game; 0xa1f) as i32 == kMovementType_14_WallJumping as i32 {
            if word!(game.samus_anim_frame) as i32 - 23i32 & 0x8000 != 0 {
                if word!(game.samus_anim_frame) as i32 - 13i32 & 0x8000 != 0 {
                    0x31
                } else {
                    0x3e
                }
            } else {
                0x33
            }
        } else {
            if *ram_ptr!(game; 0xa1f) as i32 != kMovementType_03_SpinJumping as i32 {
                return 0;
            }
            match word!(game.samus_pose) as u32 {
                kPose_81_FaceR_Screwattack | kPose_82_FaceL_Screwattack => 0x33,
                kPose_1B_FaceR_SpaceJump | kPose_1C_FaceL_SpaceJump => 0x3e,
                _ => 0x31,
            }
        };
        QueueSfx1_Max9(sound, game);
        0
    }
}
pub unsafe fn SamusCode_1D_ClearSoundInDoor(game: &mut Game) -> u8 {
    unsafe {
        if *ram_ptr!(game; 0xa1f) == 3 || *ram_ptr!(game; 0xa1f) == 20 {
            QueueSfx1_Max15(0x32, game);
            0
        } else {
            if (word!(game.button_config_shoot_x) & word!(game.joypad1_lastkeys)) == 0
                && word!(game.flare_counter) as i32 - 16i32 & 0x8000 != 0
            {
                QueueSfx1_Max15(2, game);
            }
            0
        }
    }
}
pub unsafe fn SamusCode_1E(game: &mut Game) -> u8 {
    unsafe {
        if word!(game.game_state) == 8 {
            if *ram_ptr!(game; 0xa1f) == 3 || *ram_ptr!(game; 0xa1f) == 20 {
                SamusCode_1C(game);
                return 0;
            }
            if word!(game.flare_counter) as i32 - 16i32 & 0x8000 == 0 {
                QueueSfx1_Max9(0x41, game);
            }
        }
        0
    }
}
pub unsafe fn SamusCode_1F(game: &mut Game) -> u8 {
    unsafe {
        if word!(game.grapple_beam_function) != (addr_loc_90C4F0 as u16) {
            word! { game, 0;
                grapple_beam_unkD1E, grapple_beam_unkD20, grapple_beam_direction, grapple_beam_unkD36,
                grapple_walljump_timer, slow_grabble_scrolling_flag, grapple_varCF6, grapple_beam_flags,
            }
            LoadProjectilePalette(word!(game.equipped_beams), game);
            word!(game.grapple_beam_function = -15120i32 as u16);
            word!(game.samus_draw_handler = ((0x90eb52i32) as u16));
        }
        0
    }
}
pub unsafe fn Samus_ShootCheck(game: &mut Game) {
    unsafe {
        let resume_charge_sound: bool = ((word!(game.play_resume_charging_beam_sfx) & 0x8000) != 0)
            || ({
                if word!(game.play_resume_charging_beam_sfx) != 0 {
                    if (word!(game.button_config_shoot_x) & word!(game.joypad1_lastkeys)) != 0 {
                        QueueSfx1_Max9(0x41, game);
                    }
                    word!(game.play_resume_charging_beam_sfx = 0);
                }
                if word!(game.samus_echoes_sound_flag) != 0
                    && (word!(game.speed_boost_counter) & 0x400) == 0
                {
                    word!(game.samus_echoes_sound_flag = 0);
                    QueueSfx3_Max15(0x25, game);
                }
                ((*ram_ptr!(game; 0xa11) == 3 || *ram_ptr!(game; 0xa11) == 20)
                    && *ram_ptr!(game; 0xa1f) as i32 != kMovementType_03_SpinJumping as i32
                    && *ram_ptr!(game; 0xa1f) as i32 != kMovementType_14_WallJumping as i32)
                    && ({
                        QueueSfx1_Max15(0x32, game);
                        (word!(game.flare_counter) as i32 - 16i32 & 0x8000 == 0
                            && (word!(game.button_config_shoot_x) & word!(game.joypad1_lastkeys))
                                != 0)
                    })
            });
        if resume_charge_sound {
            word!(game.play_resume_charging_beam_sfx = 1);
        }
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
