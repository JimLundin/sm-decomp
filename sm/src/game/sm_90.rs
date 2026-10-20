use crate::game::callback_tables::callback_table;
pub use crate::game::constants::bank_60::*;
pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_8d::*;
pub use crate::game::constants::bank_90::*;
pub use crate::game::constants::buttons::*;
pub use crate::game::constants::game_states::*;
pub use crate::game::constants::samus_movement::*;
pub use crate::game::constants::samus_poses::*;
use crate::game::sm_80::{
    CheckEventHappened, DrawTimer, HandleAutoscrolling_X, HandleAutoscrolling_Y,
    HandleScrollingWhenTriggeringScrollDown, HandleScrollingWhenTriggeringScrollLeft,
    HandleScrollingWhenTriggeringScrollRight, HandleScrollingWhenTriggeringScrollUp,
    InvalidInterrupt_Crash, NextRandom, ProcessTimer, QueueSfx1_Max15, QueueSfx1_Max6,
    QueueSfx1_Max9, QueueSfx2_Max6, QueueSfx3_Max15, QueueSfx3_Max3, QueueSfx3_Max6,
    QueueSfx3_Max9,
};
use crate::game::sm_81::{DrawBeamGrappleSpritemap, DrawSamusSpritemap};
use crate::game::sm_82::PlayRoomMusicTrackAfterAFrames;
use crate::game::sm_86::{DisableEprojs, EnableEprojs, SpawnEprojWithGfx};
use crate::game::sm_88::{EnableHdmaObjects, SpawnCrystalFlashHdmaObjs, SpawnPowerBombExplosion};
use crate::game::sm_8b::Samus_CalcPos_Mode7;
use crate::game::sm_8d::{DisablePaletteFx, SpawnPalfxObject};
use crate::game::sm_91::{
    DemoObjectInputHandler, MakeSamusFaceForward, SamusFunc_E633, SamusFunc_F433,
    Samus_CancelSpeedBoost, Samus_Func20, Samus_HandlePalette, Samus_HandleTransFromBlockColl,
    Samus_HandleTransitions, Samus_InputHandler, Samus_LoadSuitPalette,
    Samus_LoadSuitTargetPalette, Samus_RestoreHealth, Samus_SetAnimationFrameIfPoseChanged,
    XrayRunHandler,
};
use crate::game::sm_92::{PlaySamusFanfare, SetSamusTilesDefsForCurAnim};
use crate::game::sm_93::{
    DrawPlayerExplosions2, InitializeBombExplosion, InitializeInstrForMissile,
    InitializeInstrForSuperMissile, InitializeProjectile, InitializeSbaProjectile,
    InitializeShinesparkEchoOrSpazerSba, KillProjectileInner, RunProjectileInstructions,
};
use crate::game::sm_94::{
    kSinCosTable8bit_Sext, BlockCollMissileHoriz, BlockCollMissileVert, BlockCollNoWaveBeamHoriz,
    BlockCollNoWaveBeamVert, BlockCollSpreadBomb, BlockCollWaveBeamHoriz, BlockCollWaveBeamVert,
    BlockInsideDetection, BombOrPowerBomb_Func1, HandleGrappleBeamGfx,
    PostGrappleCollisionDetect_X, PostGrappleCollisionDetect_Y, Samus_AlignYPosSlope,
    Samus_MoveDown_NoSolidColl, Samus_MoveRight_NoSolidColl, WallJumpBlockCollDetect,
};
use crate::game::sm_9b::{
    GrappleBeamFunc_BF1B, GrappleBeamHandler, HandleGrappleBeamFlare, ProjectileTrail_Func5,
    UpdateGrappleBeamTilesAndIncrFlameCtr,
};
use crate::game::sm_a0::Samus_CheckSolidEnemyColl;
use crate::game_state::Game;
use crate::ram_map::ram_ptr;
use crate::ram_map::set_ram_words;
use crate::ram_map::{ram_word, set_ram_word, update_ram_word};
use crate::rtl::abs16;
use crate::rtl::{AddToHiLo, SetHiLo};
use crate::rtl::{Load24, MakePairU16, Mult8x8, RomPtr, Unreachable};
pub use crate::types::{
    FuncXY_V, Func_U8, Func_V, Func_V_A, Func_Y_V, HandlerFunc, LongPtr, PairU16, Point16U, VoidP,
};
use ::core::ptr::null_mut;
pub type Func_Y_To_PairU16 = unsafe fn(u16, &mut Game) -> PairU16;
pub use crate::types::LiquidPhysicsType;
pub use crate::types::{OamEnt, SamusPose, VramWriteEntry};
crate::game::constants::typed_constants! { LiquidPhysicsType;
    kLiquidPhysicsType_LavaAcid = 2, kLiquidPhysicsType_Water = 1,
    kLiquidPhysicsType_Air = 0,
}
pub use crate::types::SamusPoseParams;
crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C, packed)]
    pub struct SamusSpeedTableEntry {
        pub accel, accel_sub, max_speed, max_speed_sub, decel, decel_sub: u16;
    }
}
pub use crate::types::SamusMovementType;
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct DisableMinimapAndMarkBossRoomAsExploredEnt {
    pub boss_id_: u16,
    pub ptrs: VoidP,
}
pub use crate::types::Buttons;
pub use crate::types::Consts_60;
pub use crate::types::Consts_86;
pub use crate::types::Consts_8D;
pub use crate::types::Consts_90;
pub use crate::types::GameState;
pub use crate::types::{
    CheckEnemyColl_Result, CheckSolidEnemyColl_Result, Func_SpazerSba_V, Pair_Bool_Amt,
};
crate::game::constants::typed_constants! { i32;
    true_0 = 1, false_0 = 0,
}
const RomPtr_82: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x82>;
const RomPtr_90: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x90>;
const RomPtr_91: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x91>;
crate::ram_map::ram_pointer_functions! {
    gOamEnt(v: i32) -> OamEnt = [(0x370i32 + v) as isize];
    gVramWriteEntry(a: i32) -> VramWriteEntry = [(0xd0i32 + a) as isize];
}
#[inline]
unsafe fn get_SamusSpeedTableEntry(a: u16, game: &mut Game) -> *mut SamusSpeedTableEntry {
    RomPtr((0x900000u32) | ((a) as u32), game) as *mut SamusSpeedTableEntry
}
crate::game::constants::typed_constants! { u32;
    fnSamus_MoveHandler_ReleaseFromGrapple = 9475182,
}
crate::game::constants::typed_constants! { i32;
    fnSamus_HandleMovement_DrainedCrouching = 0x9094cb,
}
crate::game::constants::typed_constants! { u32;
    fnSamus_ScrollFinishedHook_SporeSpawnFight = 9475465,
}
crate::game::constants::typed_constants! { i32;
    fnSamus_MovementHandler_Normal = 0x90a337, fnProjPreInstr_Beam_NoWaveBeam = 0x90aef3,
}
crate::game::constants::typed_constants! { u32;
    fnProjPreInstr_Dir0459 = 9482058, fnProjPreInstr_Dir1368 = 9482066,
    fnProjPreInstr_Dir27 = 9482080,
}
crate::game::constants::typed_constants! { i32;
    fnProjPreInstr_Missile = 0x90af68,
}
crate::game::constants::typed_constants! { u32;
    fnProjPreInstr_Missile_Func0459 = 9482183, fnProjPreInstr_Missile_Func1368 = 9482191,
    fnProjPreInstr_Missile_Func27 = 9482205,
}
crate::game::constants::typed_constants! { i32;
    fnProjPreInstr_SuperMissile = 0x90afe5,
}
crate::game::constants::typed_constants! { u32;
    fnProjPreInstr_SuperMissile_Func0459 = 9482311, fnProjPreInstr_SuperMissile_Func1368 = 9482322,
    fnProjPreInstr_SuperMissile_Func27 = 9482346,
}
crate::game::constants::typed_constants! { i32;
    fnProjPreInstr_Func1 = 0x90b075, fnProjPreInstr_Bomb = 0x90b099,
    fnProjPreInstr_PowerBomb = 0x90b0ae, fnProjPreInstr_WavePlasmaEtc = 0x90b0c3,
    fnProjPreInstr_BeamOrIceWave = 0x90b0e4,
}
crate::game::constants::typed_constants! { u32;
    fnProjPreInstr_Wave_Shared = 9482499, fnProjPreInstr_WavePlasmaEtc_0459 = 9482555,
    fnProjPreInstr_WavePlasmaEtc_1368 = 9482563, fnProjPreInstr_WavePlasmaEtc_27 = 9482577,
}
crate::game::constants::typed_constants! { i32;
    fnProjPreInstr_HyperBeam = 0x90b159, fnProjPreInstr_Empty = 0x90b169,
}
crate::game::constants::typed_constants! { u32;
    fnProjInstr_MoveLeftProjectileTrailDown = 9483557, fnProjInstr_MoveRightProjectileTrailDown = 9483655,
    fnProjInstr_MoveLeftProjectileTrailUp = 9483699,
}
crate::game::constants::typed_constants! { i32;
    fnProjPreInstr_IceSba = 0x90cf09, fnProjPreInstr_IceSba2 = 0x90cf7a,
    fnSamus_MoveHandlerShinesparkWindup = 0x90d068, fnSamus_MoveHandlerVerticalShinespark = 0x90d0ab,
}
crate::game::constants::typed_constants! { u32;
    fnSamus_MoveHandler_Shinespark_Diag = 9490647, fnSamus_MoveHandler_Shinespark_Horiz = 9490694,
}
crate::game::constants::typed_constants! { i32;
    fnSamus_MoveHandler_ShinesparkCrash = 0x90d346,
}
crate::game::constants::typed_constants! { u32;
    fnSamus_MoveHandler_ShinesparkCrash_0 = 9491331, fnSamus_MoveHandler_ShinesparkCrash_1 = 9491350,
    fnSamus_MoveHandler_ShinesparkCrash_2 = 9491404,
}
crate::game::constants::typed_constants! { i32;
    fnSamus_MoveHandler_ShinesparkCrashEchoCircle = 0x90d3f3, fnSamus_MoveHandler_ShinesparkCrashFinish = 0x90d40d,
    fnProjPreInstr_SpeedEcho = 0x90d4d2, fnSamusMoveHandler_CrystalFlashStart = 0x90d678,
    fnSamusMoveHandler_CrystalFlashMain = 0x90d6ce, fnkSamusMoveHandler_CrystalFlashFinish = 0x90d75b,
    fnProjPreInstr_PlasmaSba = 0x90d793,
}
crate::game::constants::typed_constants! { u32;
    fnProjPreInstr_PlasmaSbaFunc_0 = 9492449, fnProjPreInstr_PlasmaSbaFunc_1 = 9492474,
    fnProjPreInstr_PlasmaSbaFunc_2 = 9492499,
}
crate::game::constants::typed_constants! { i32;
    fnProjPreInstr_SpreadBomb = 0x90d8f7, fnProjPreInstr_WaveSba = 0x90da08,
    fnProjPreInstr_SpazerSba = 0x90db06, fnProjPreInstr_EndOfSpazerSba = 0x90dc9c,
}
crate::game::constants::typed_constants! { u32;
    fnSamus_MoveHandler_Knockback = 9494328, fnSamus_MoveHandler_Knockback_0 = 9494352,
    fnSamus_MoveHandler_Knockback_Up = 9494355, fnSamus_MoveHandler_Knockback_3 = 9494365,
    fnSamus_MoveHandler_Knockback_Down = 9494372, fnSamus_MoveHandler_BombJumpStart = 9494565,
}
crate::game::constants::typed_constants! { i32;
    fnSamus_MoveHandler_BombJumpMain = 0x90e032,
}
crate::game::constants::typed_constants! { u32;
    fnSamus_MoveHandler_BombJumpFunc1 = 9494653,
}
crate::game::constants::typed_constants! { i32;
    fnSamus_Func1 = 0x90e09b, fnSamus_Func2 = 0x90e0c5,
    fnSamus_Func3 = 0x90e0e6, fnDrawTimer_ = 0x90e114,
    fnSamus_PushOutOfRidleysWay = 0x90e12e, fnSamus_Func4 = 0x90e1c8,
    fnSamus_GrabbedByDraygonFrameHandler = 0x90e2a1,
}
crate::game::constants::typed_constants! { u32;
    fnnullsub_151 = 9495422, fnSamus_Func7 = 9495459,
}
crate::game::constants::typed_constants! { i32;
    fnSamus_Func9 = 0x90e41b, fnSamus_FrameHandlerAlfa_Func11 = 0x90e695,
}
crate::game::constants::typed_constants! { u32;
    fnSamus_FrameHandlerAlfa_Func12 = 9496265,
}
crate::game::constants::typed_constants! { i32;
    fnSamus_FrameHandlerAlfa_Func13 = 0x90e713, fnSamus_FrameHandlerBeta_Func17 = 0x90e725,
    fnHandleDemoRecorder_3 = 0x90e7d2,
}
crate::game::constants::typed_constants! { u32;
    fnSamus_FrameHandlerBeta_Func14 = 9496565, fnSamus_Func15 = 9496627,
    fnSamus_Func16 = 9496682,
}
crate::game::constants::typed_constants! { i32;
    fnSamus_Func18 = 0x90e8aa, fnEmptyFunction = 0x90e8cd,
    fnj_HandleDemoRecorder_2 = 0x90e8d6, fnj_HandleDemoRecorder_2_0 = 0x90e8d9,
    fnSetContactDamageIndexAndUpdateMinimap = 0x90e8dc, fnSamus_Func19 = 0x90e8ec,
    fnSamus_LowHealthCheck = 0x90e902, fnnullsub_152 = 0x90e90e,
    fnSamus_InputHandler_E913 = 0x90e913,
}
crate::game::constants::typed_constants! { u32;
    fnSamus_Func20_ = 9496856,
}
crate::game::constants::typed_constants! { i32;
    fnSamus_InputHandler_E91D = 0x90e91d, fnHandleAutoJumpHack = 0x90e926,
}
crate::game::constants::typed_constants! { u32;
    fnSamusMovementType_Xray = 9496911,
}
crate::game::constants::typed_constants! { i32;
    fnSamusDrawHandler_Default = 0x90eb52,
}
crate::game::constants::typed_constants! { u32;
    fnsub_90EB86 = 9497478, fnnullsub_156 = 9497586,
}
crate::game::constants::typed_constants! { i32;
    fnSamus_DrawHandler_EndOfShinespark = 0x90ebf3, fnSamusDisplayHandler_UsingElevator = 0x90ec14,
    fnSamusDisplayHandler_SamusReceivedFatal = 0x90ec1d,
}
crate::game::constants::typed_constants! { u32;
    fnProjPreInstr_UnknownProj8027 = 9498579,
}
crate::game::constants::typed_constants! { i32;
    fnSamus_Func25_ShineSpark = 0x90f04b, fnSamus_MoveHandler_F072 = 0x90f072,
    fnGrappleBeamFunc_Inactive = 0x9bc4f0, fnGrappleBeamFunc_Firing = 0x9bc703,
    fnUNUSED_sub_9BC759 = 0x9bc759, fnGrappleBeamFunc_ConnectedLockedInPlace = 0x9bc77e,
    fnGrappleBeamFunc_Cancel = 0x9bc856, fnGrappleBeam_Func2 = 0x9bc8c5,
}
static kUnchargedProjectile_Sfx: [u16; 12] = [
    0xb, 0xd, 0xc, 0xe, 0xf, 0x12, 0x10, 0x11, 0x13, 0x16, 0x14, 0x15,
];
static kChargedProjectile_Sfx: [u16; 12] = [
    0x17, 0x19, 0x18, 0x1a, 0x1b, 0x1e, 0x1c, 0x1d, 0x1f, 0x22, 0x20, 0x21,
];
static kNonBeamProjectile_Sfx: [u16; 9] = [0, 3, 4, 0, 0, 0, 0, 0, 0];
static kProjectileCooldown_Uncharged: [u8; 38] = [
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 12, 15, 0, 0, 0, 0, 30, 30, 30, 30, 30, 30, 30, 30, 30,
    30, 30, 30, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
static kNonBeamProjectileCooldowns: [u8; 9] = [0, 0xa, 0x14, 0x28, 0, 0x10, 0, 0, 0];
static kBeamAutoFireCooldowns: [u8; 12] = [
    0x19, 0x19, 0x19, 0x19, 0x19, 0x19, 0x19, 0x19, 0x19, 0x19, 0x19, 0x19,
];
pub unsafe fn Samus_Animate(game: &mut Game) {
    let fx_type = u16::from_le_bytes([game.g_ram[0x196e], game.g_ram[0x196f]]);
    match (fx_type & 0xf) >> 1 {
        0 => Samus_Animate_NoFx(game),
        1 => Samus_Animate_LavaFx(game),
        2 => Samus_Animate_AcidFx(game),
        3 => Samus_Animate_WaterFx(game),
        // FX kinds 4..=7 leave the animation FX state unchanged.
        _ => {}
    }
    let pose = game.samus_pose();
    let mut timer = u16::from_le_bytes([game.g_ram[0xa94], game.g_ram[0xa95]]);
    if (pose == kPose_4D_FaceR_Jump_NoAim_NoMove_NoGun as u16
        || pose == kPose_4E_FaceL_Jump_NoAim_NoMove_NoGun as u16)
        && u16::from_le_bytes([game.g_ram[0xb36], game.g_ram[0xb37]]) != 2
        && game.samus_anim_frame() == 1
        && timer == 1
    {
        timer = 4;
    }
    timer = timer.wrapping_sub(1);
    game.set_samus_anim_frame_timer(timer);
    if timer == 0 || timer & 0x8000 != 0 {
        game.set_samus_anim_frame(game.samus_anim_frame().wrapping_add(1));
        Samus_HandleAnimDelay(game);
    }
}
pub unsafe fn Samus_Animate_NoFx(game: &mut Game) {
    let bottom = Samus_GetBottom_R18(game);
    let speed_divisor = u16::from_le_bytes([game.g_ram[0xa66], game.g_ram[0xa67]]);
    game.g_ram[0xa9c..0xa9e].copy_from_slice(&speed_divisor.to_le_bytes());

    let liquid_physics = u16::from_le_bytes([game.g_ram[0xad2], game.g_ram[0xad3]]);
    if liquid_physics == 0 {
        return;
    }
    game.g_ram[0xad2..0xad4].copy_from_slice(&0u16.to_le_bytes());
    if liquid_physics & 1 == 0 {
        return;
    }

    QueueSfx2_Max6(0xe, game);
    let suit_palette = u16::from_le_bytes([game.g_ram[0xa74], game.g_ram[0xa75]]);
    if suit_palette & 4 == 0 && matches!(game.samus_movement_type(), 3 | 20) {
        QueueSfx1_Max6(0x30, game);
    }
    Samus_SpawnWaterSplash(bottom, game);
}
pub unsafe fn Samus_Animate_WaterFx(game: &mut Game) {
    static kSamusPhys_AnimDelayInWater: u16 = 3u16;
    let r18: u16 = Samus_GetBottom_R18(game);
    if (ram_word!(game, fx_y_pos) as i32 & 0x8000i32) == 0i32
        && ram_word!(game, fx_y_pos) as i32 - r18 as i32 & 0x8000i32 != 0
        && (ram_word!(game, fx_liquid_options) as i32 & 4i32) == 0i32
    {
        set_ram_word!(game.samus_anim_frame_buffer = kSamusPhys_AnimDelayInWater);
        if ram_word!(game, liquid_physics_type) as i32 == kLiquidPhysicsType_Water as i32 {
            Samus_SpawnAirBubbles(game);
        } else {
            set_ram_word!(game.liquid_physics_type = kLiquidPhysicsType_Water as u16);
            QueueSfx2_Max6(0xdu16, game);
            Samus_SpawnWaterSplash(r18, game);
        }
    } else {
        Samus_Animate_NoFx(game);
    };
}
pub unsafe fn Samus_SpawnWaterSplash(r18: u16, game: &mut Game) {
    static kWaterSplashTypeTable: [u8; 28] = [
        1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
    ];
    if kWaterSplashTypeTable[*ram_ptr!(game; 0xa1f) as usize] != 0 {
        set_ram_words! { game, 256u16;
            atmospheric_gfx_frame_and_type[0], atmospheric_gfx_frame_and_type[1],
        }
        set_ram_words! { game, 3u16; atmospheric_gfx_anim_timer[0], atmospheric_gfx_anim_timer[1] }
        set_ram_word!(
            game.atmospheric_gfx_x_pos[0] = (ram_word!(game, samus_x_pos)).wrapping_add(4u16)
        );
        set_ram_word!(
            game.atmospheric_gfx_x_pos[1] = (ram_word!(game, samus_x_pos)).wrapping_sub(3u16)
        );
        set_ram_word!(game.atmospheric_gfx_y_pos[0] = (r18).wrapping_sub(4u16));
        set_ram_word!(game.atmospheric_gfx_y_pos[1] = (r18).wrapping_sub(4u16));
    } else {
        set_ram_word!(game.atmospheric_gfx_frame_and_type[0] = 768u16);
        set_ram_word!(game.atmospheric_gfx_anim_timer[0] = 2u16);
        set_ram_word!(game.atmospheric_gfx_x_pos[0] = ram_word!(game, samus_x_pos));
        set_ram_word!(game.atmospheric_gfx_y_pos[0] = ram_word!(game, fx_y_pos));
    }
    Samus_SpawnAirBubbles(game);
}
pub unsafe fn Samus_SpawnAirBubbles(game: &mut Game) {
    let r20: u16 = Samus_GetTop_R20(game);
    if ((r20 as i32 - 24i32 - ram_word!(game, fx_y_pos) as i32) as i16) >= 0i16
        && (ram_word!(game, nmi_frame_counter_word) as i32 & 0x7fi32) == 0i32
        && ram_word!(game, atmospheric_gfx_frame_and_type[2]) == 0
    {
        set_ram_word!(game.atmospheric_gfx_frame_and_type[2] = 1280u16);
        set_ram_word!(game.atmospheric_gfx_anim_timer[2] = 3u16);
        set_ram_word!(game.atmospheric_gfx_x_pos[2] = ram_word!(game, samus_x_pos));
        set_ram_word!(
            game.atmospheric_gfx_y_pos[2] = (ram_word!(game, samus_y_pos) as i32
                - ram_word!(game, samus_y_radius) as i32
                + 6i32) as u16
        );
        QueueSfx2_Max6(
            (if NextRandom(game) as i32 & 1i32 != 0 {
                15i32
            } else {
                17i32
            }) as u16,
            game,
        );
    }
    if ram_word!(game, samus_pose) as i32 == kPose_00_FaceF_Powersuit as i32
        || ram_word!(game, samus_pose) as i32 == kPose_9B_FaceF_VariaGravitySuit as i32
        || (ram_word!(game, equipped_items) as i32 & 0x20i32) != 0i32
    {
        set_ram_word!(game.samus_anim_frame_buffer = 0u16);
    }
}
pub unsafe fn Samus_Animate_LavaFx(game: &mut Game) {
    static kSamusPhys_LavaDamagePerFrame: u16 = 0u16;
    static kSamusPhys_LavaSubdamagePerFrame: u16 = 0x8000u16;
    let r18: u16 = Samus_GetBottom_R18(game);
    if (ram_word!(game, lava_acid_y_pos) as i32 & 0x8000i32) == 0i32
        && ram_word!(game, lava_acid_y_pos) as i32 - r18 as i32 & 0x8000i32 != 0
    {
        if ram_word!(game, speed_boost_counter) != 0 {
            Samus_CancelSpeedBoost(game);
            set_ram_words! { game, 0u16; samus_x_extra_run_speed, samus_x_extra_run_subspeed }
        }
        if (ram_word!(game, equipped_items) as i32 & 0x20i32) != 0i32 {
            set_ram_word!(game.samus_anim_frame_buffer = 0u16);
            set_ram_word!(game.liquid_physics_type = 2u16);
        } else {
            if (ram_word!(game, game_time_frames) as i32 & 7i32) == 0i32
                && ram_word!(game, samus_health) as i32 - 71i32 & 0x8000i32 == 0
            {
                QueueSfx3_Max3(0x2du16, game);
            }
            AddToHiLo(
                ram_ptr!(game, u16; 0xa50),
                ram_ptr!(game, u16; 0xa4e),
                (kSamusPhys_LavaDamagePerFrame as u32) << 16i32
                    | kSamusPhys_LavaSubdamagePerFrame as u32,
            );
            Samus_Animate_SubmergedLavaAcid(game);
        }
    } else {
        Samus_Animate_NoFx(game);
    };
}
pub unsafe fn Samus_Animate_AcidFx(game: &mut Game) {
    static kSamusPhys_AcidSubdamagePerFrame: u16 = 0x8000u16;
    static kSamusPhys_AcidDamagePerFrame: u16 = 1u16;
    let r18: u16 = Samus_GetBottom_R18(game);
    if (ram_word!(game, lava_acid_y_pos) as i32 & 0x8000i32) == 0i32
        && ram_word!(game, lava_acid_y_pos) as i32 - r18 as i32 & 0x8000i32 != 0
    {
        if (ram_word!(game, game_time_frames) as i32 & 7i32) == 0i32
            && ram_word!(game, samus_health) as i32 - 71i32 & 0x8000i32 == 0
        {
            QueueSfx3_Max3(0x2du16, game);
        }
        AddToHiLo(
            ram_ptr!(game, u16; 0xa50),
            ram_ptr!(game, u16; 0xa4e),
            (kSamusPhys_AcidDamagePerFrame as u32) << 16i32
                | kSamusPhys_AcidSubdamagePerFrame as u32,
        );
        Samus_Animate_SubmergedLavaAcid(game);
    } else {
        Samus_Animate_NoFx(game);
    };
}
pub unsafe fn Samus_Animate_SubmergedLavaAcid(game: &mut Game) {
    static kSamusPhys_AnimDelayInAcid: u16 = 2u16;
    let r20: u16 = Samus_GetTop_R20(game);
    set_ram_word!(game.samus_anim_frame_buffer = kSamusPhys_AnimDelayInAcid);
    set_ram_word!(game.liquid_physics_type = 2u16);
    if ((r20) as i16).wrapping_sub((ram_word!(game, lava_acid_y_pos)) as i16) < 0i16
        && (ram_word!(game, atmospheric_gfx_frame_and_type[0]) as i32 & 0x400i32) == 0i32
    {
        set_ram_words! { game, 1024u16;
            atmospheric_gfx_frame_and_type[0], atmospheric_gfx_frame_and_type[1],
            atmospheric_gfx_frame_and_type[2], atmospheric_gfx_frame_and_type[3],
        }
        set_ram_words! { game, 3u16; atmospheric_gfx_anim_timer[0], atmospheric_gfx_anim_timer[3] }
        set_ram_word!(game.atmospheric_gfx_anim_timer[1] = -32766i32 as u16);
        set_ram_word!(game.atmospheric_gfx_anim_timer[2] = -32766i32 as u16);
        set_ram_word!(game.atmospheric_gfx_y_pos[0] = ram_word!(game, lava_acid_y_pos));
        set_ram_word!(game.atmospheric_gfx_y_pos[1] = ram_word!(game, lava_acid_y_pos));
        set_ram_word!(game.atmospheric_gfx_y_pos[2] = ram_word!(game, lava_acid_y_pos));
        set_ram_word!(game.atmospheric_gfx_y_pos[3] = ram_word!(game, lava_acid_y_pos));
        set_ram_word!(
            game.atmospheric_gfx_x_pos[0] = (ram_word!(game, samus_x_pos)).wrapping_add(6u16)
        );
        set_ram_word!(game.atmospheric_gfx_x_pos[1] = ram_word!(game, samus_x_pos));
        set_ram_word!(game.atmospheric_gfx_x_pos[2] = ram_word!(game, samus_x_pos));
        set_ram_word!(
            game.atmospheric_gfx_x_pos[3] = (ram_word!(game, samus_x_pos)).wrapping_sub(6u16)
        );
        if (ram_word!(game, game_time_frames) as i32 & 1i32) == 0i32 {
            QueueSfx2_Max6(0x10u16, game);
        }
    }
    if ram_word!(game, samus_pose) as i32 == kPose_00_FaceF_Powersuit as i32
        || ram_word!(game, samus_pose) as i32 == kPose_9B_FaceF_VariaGravitySuit as i32
        || (ram_word!(game, equipped_items) as i32 & 0x20i32) != 0i32
    {
        set_ram_word!(game.samus_anim_frame_buffer = 0u16);
    }
}
fn Samus_AnimDelayFunc_6_GotoStartIfLittleHealth(game: &mut Game) -> bool {
    let frame = if game.samus_health().wrapping_sub(30) & 0x8000 != 0 {
        game.samus_anim_frame().wrapping_add(1)
    } else {
        0
    };
    game.set_samus_anim_frame(frame);
    true
}
unsafe fn Samus_AnimDelayFunc_7(jp: *const u8, game: &mut Game) -> bool {
    set_ram_word!(game.samus_movement_handler = ((0x9094cbi32) as u16));
    update_ram_word!(game.samus_anim_frame, wrapping_add(1));
    true_0 != 0
}
fn Samus_AnimDelayFunc_13_TransToPose(pose: u8, game: &mut Game) -> bool {
    game.set_samus_new_pose_transitional(u16::from(pose));
    game.set_samus_hurt_switch_index(3);
    false
}
unsafe fn Samus_AnimDelayFunc_8_AutoJumpHack(instruction: *const u8, game: &mut Game) -> bool {
    let input_handler = u16::from_le_bytes([game.g_ram[0xa60], game.g_ram[0xa61]]);
    if input_handler == fnSamus_InputHandler_E91D as u16 {
        return Samus_AnimDelayFunc_13_TransToPose(*instruction.add(1), game);
    }
    let new_pose = u16::from_le_bytes([game.g_ram[0xa28], game.g_ram[0xa29]]);
    if new_pose != kPose_4B_FaceR_Jumptrans as u16
        && new_pose != kPose_4C_FaceL_Jumptrans as u16
        && new_pose != kPose_19_FaceR_SpinJump as u16
        && new_pose != kPose_1A_FaceL_SpinJump as u16
    {
        game.set_samus_input_handler(fnHandleAutoJumpHack as u16);
        return Samus_AnimDelayFunc_13_TransToPose(*instruction.add(1), game);
    }
    false
}
unsafe fn Samus_AnimDelayFunc_9_TransToPose(instruction: *const u8, game: &mut Game) -> bool {
    let item_mask = u16::from_le_bytes([*instruction.add(1), *instruction.add(2)]);
    let equipped_items = u16::from_le_bytes([game.g_ram[0x9a2], game.g_ram[0x9a3]]);
    let has_item = item_mask & equipped_items != 0;
    let y_speed = u16::from_le_bytes([game.g_ram[0xb2e], game.g_ram[0xb2f]]);
    let moving_vertically =
        y_speed != 0 || u16::from_le_bytes([game.g_ram[0xb2c], game.g_ram[0xb2d]]) != 0;
    let pose_offset = match (has_item, moving_vertically) {
        (false, false) => 3,
        (false, true) => 4,
        (true, false) => 5,
        (true, true) => 6,
    };
    game.set_samus_new_pose_transitional(u16::from(*instruction.add(pose_offset)));
    game.set_samus_hurt_switch_index(3);
    false
}
unsafe fn UNUSED_Samus_AnimDelayFunc_10(jp: *const u8, game: &mut Game) -> bool {
    if ram_word!(game, samus_y_speed) != 0u16 || ram_word!(game, samus_y_subspeed) != 0u16 {
        set_ram_word!(game.samus_new_pose_transitional = *jp.offset(2) as u16);
    } else {
        set_ram_word!(game.samus_new_pose_transitional = *jp.offset(1) as u16);
    }
    set_ram_word!(game.samus_hurt_switch_index = 3u16);
    false_0 != 0
}
unsafe fn Samus_AnimDelayFunc_11_SelectDelaySequenceWalljump(game: &mut Game) -> bool {
    let gravity_suit = u16::from_le_bytes([game.g_ram[0x9a2], game.g_ram[0x9a3]]) & 0x20 != 0;
    let liquid_limits_animation = if gravity_suit {
        false
    } else {
        let top = Samus_GetTop_R20(game);
        liquid_at_height(game, top) != LiquidAtHeight::None
    };
    let (sound, frame_advance) = if liquid_limits_animation {
        (0x31, 1)
    } else {
        let equipped_items = u16::from_le_bytes([game.g_ram[0x9a2], game.g_ram[0x9a3]]);
        if equipped_items & 8 != 0 {
            (0x33, 21)
        } else if equipped_items & 0x200 != 0 {
            (0x3e, 11)
        } else {
            (0x31, 1)
        }
    };
    QueueSfx1_Max6(sound, game);
    game.set_samus_anim_frame(game.samus_anim_frame().wrapping_add(frame_advance));
    true
}
unsafe fn Samus_AnimDelayFunc_12_TransToPose(jp: *const u8, game: &mut Game) -> bool {
    if (*(jp.offset(1) as *mut u16) as i32 & ram_word!(game, equipped_items) as i32) != 0i32 {
        set_ram_word!(game.samus_new_pose_transitional = *jp.offset(4) as u16);
    } else {
        set_ram_word!(game.samus_new_pose_transitional = *jp.offset(3) as u16);
    }
    set_ram_word!(game.samus_hurt_switch_index = 3u16);
    false_0 != 0
}
fn Samus_AnimDelayFunc_14_Goto(frames_back: u8, game: &mut Game) -> bool {
    game.set_samus_anim_frame(game.samus_anim_frame().wrapping_sub(u16::from(frames_back)));
    true
}
fn Samus_AnimDelayFunc_15_GotoStart(game: &mut Game) -> bool {
    game.set_samus_anim_frame(0);
    true
}
unsafe fn Samus_HandleSpeedBoosterAnimDelay(jp: *const u8, game: &mut Game) -> u8 {
    let has_momentum = u16::from_le_bytes([game.g_ram[0xb3c], game.g_ram[0xb3d]]);
    let run_button = u16::from_le_bytes([game.g_ram[0x9b6], game.g_ram[0x9b7]]);
    if has_momentum == 0
        || run_button & game.joypad1_lastkeys() == 0
        || game.samus_movement_type() != 1
    {
        return *jp;
    }
    let equipped_items = u16::from_le_bytes([game.g_ram[0x9a2], game.g_ram[0x9a3]]);
    if equipped_items & 0x2000 == 0 {
        game.set_samus_anim_frame(0);
        let frame_buffer = u16::from_le_bytes([game.g_ram[0xa9c], game.g_ram[0xa9d]]);
        let animation_list = game.read_rom_word(0x91b5d1, 0);
        let delay = *RomPtr_91(animation_list, game);
        game.set_samus_anim_frame_timer(frame_buffer.wrapping_add(u16::from(delay)));
        return 0;
    }
    let counter = u16::from_le_bytes([game.g_ram[0xb3e], game.g_ram[0xb3f]]).wrapping_sub(1);
    game.g_ram[0xb3e..0xb40].copy_from_slice(&counter.to_le_bytes());
    if counter as u8 != 0 {
        return *jp;
    }
    let mut staged_counter = counter;
    if counter & 0x400 == 0 {
        staged_counter = counter.wrapping_add(0x100);
        game.g_ram[0xb3e..0xb40].copy_from_slice(&staged_counter.to_le_bytes());
        if staged_counter & 0x400 != 0 {
            game.g_ram[0xb40..0xb42].copy_from_slice(&1u16.to_le_bytes());
            QueueSfx3_Max6(3, game);
        }
    }
    // The table stage is captured before sound; the counter is read again afterward.
    let stage_offset = usize::from(staged_counter >> 8) * 2;
    let stage_duration = game.read_rom_word(0x91b61f, stage_offset);
    let live_counter = u16::from_le_bytes([game.g_ram[0xb3e], game.g_ram[0xb3f]]);
    let reloaded_counter = stage_duration | (live_counter & 0xff00);
    game.g_ram[0xb3e..0xb40].copy_from_slice(&reloaded_counter.to_le_bytes());
    game.set_samus_anim_frame(0);
    let frame_buffer = u16::from_le_bytes([game.g_ram[0xa9c], game.g_ram[0xa9d]]);
    let animation_list = game.read_rom_word(0x91b5de, stage_offset);
    let delay = *RomPtr_91(animation_list, game);
    game.set_samus_anim_frame_timer(frame_buffer.wrapping_add(u16::from(delay)));
    0
}
unsafe fn Samus_HandleAnimDelay(game: &mut Game) {
    let animation_list = game.read_rom_word(0x91b010, usize::from(game.samus_pose()) * 2);
    let mut delays = RomPtr_91(animation_list, game);
    if *delays.add(usize::from(game.samus_anim_frame())) & 0x80 != 0 {
        let command = Samus_HandleSpeedBoosterAnimDelay(
            delays.add(usize::from(game.samus_anim_frame())),
            game,
        );
        // Keep the captured list and reread the frame after each callback.
        let instruction = delays.add(usize::from(game.samus_anim_frame()));
        let update_timer = match command & 0xf {
            6 => Samus_AnimDelayFunc_6_GotoStartIfLittleHealth(game),
            7 => Samus_AnimDelayFunc_7(instruction, game),
            8 => Samus_AnimDelayFunc_8_AutoJumpHack(instruction, game),
            9 => Samus_AnimDelayFunc_9_TransToPose(instruction, game),
            10 => UNUSED_Samus_AnimDelayFunc_10(instruction, game),
            11 => Samus_AnimDelayFunc_11_SelectDelaySequenceWalljump(game),
            12 => Samus_AnimDelayFunc_12_TransToPose(instruction, game),
            13 => Samus_AnimDelayFunc_13_TransToPose(*instruction.add(1), game),
            14 => Samus_AnimDelayFunc_14_Goto(*instruction.add(1), game),
            15 => Samus_AnimDelayFunc_15_GotoStart(game),
            // Commands 0..=5 retain the timer selected by the speed helper.
            _ => false,
        };
        if !update_timer {
            return;
        }
    } else {
        let has_momentum = u16::from_le_bytes([game.g_ram[0xb3c], game.g_ram[0xb3d]]);
        if has_momentum != 0 && game.samus_movement_type() == 1 {
            let equipped_items = u16::from_le_bytes([game.g_ram[0x9a2], game.g_ram[0x9a3]]);
            let animation_list = if equipped_items & 0x2000 != 0 {
                let speed_boost_stage = usize::from(game.g_ram[0xb3f]);
                game.read_rom_word(0x91b5de, speed_boost_stage * 2)
            } else {
                game.read_rom_word(0x91b5d1, 0)
            };
            delays = RomPtr_91(animation_list, game);
        }
    }
    let frame_buffer = u16::from_le_bytes([game.g_ram[0xa9c], game.g_ram[0xa9d]]);
    let delay = *delays.add(usize::from(game.samus_anim_frame()));
    game.set_samus_anim_frame_timer(frame_buffer.wrapping_add(u16::from(delay)));
}

static kSamusIsBottomDrawnFuncs: [Option<Func_U8>; 28] = callback_table! { unsafe Func_U8;
    SamusBottomDrawn_0_Standing, SamusBottomDrawn_1, SamusBottomDrawn_1,
    SamusBottomDrawn_3_SpinJump, SamusBottomDrawn_4, SamusBottomDrawn_1, SamusBottomDrawn_1,
    SamusBottomDrawn_4, SamusBottomDrawn_4, SamusBottomDrawn_4, SamusBottomDrawn_A_Knockback,
    SamusBottomDrawn_1, SamusBottomDrawn_1, UNUSED_SamusBottomDrawn_D, SamusBottomDrawn_1,
    SamusBottomDrawn_F_Transitions, SamusBottomDrawn_1, SamusBottomDrawn_4, SamusBottomDrawn_4,
    SamusBottomDrawn_4, SamusBottomDrawn_14_WallJump, SamusBottomDrawn_1, SamusBottomDrawn_1,
    SamusBottomDrawn_1, SamusBottomDrawn_1, SamusBottomDrawn_19_DamageBoost, SamusBottomDrawn_1,
    SamusBottomDrawn_1B,
};
pub unsafe fn Samus_Draw(game: &mut Game) {
    let knockback_timer = u16::from_le_bytes([game.g_ram[0x18aa], game.g_ram[0x18ab]]);
    let invincibility_timer = u16::from_le_bytes([game.g_ram[0x18a8], game.g_ram[0x18a9]]);
    let shine_timer = u16::from_le_bytes([game.g_ram[0xa68], game.g_ram[0xa69]]);
    let nmi_frame = u16::from_le_bytes([game.g_ram[0x5b6], game.g_ram[0x5b7]]);
    if knockback_timer != 0 || invincibility_timer == 0 || shine_timer != 0 || nmi_frame & 1 == 0 {
        let doubled_pose = game.samus_pose().wrapping_mul(2);
        let frame = game.samus_anim_frame();
        let top_base = game.read_rom_word(0x929263, usize::from(game.samus_pose()) * 2);
        let top_spritemap = frame.wrapping_add(top_base);
        game.g_ram[0xac8..0xaca].copy_from_slice(&top_spritemap.to_le_bytes());
        let position = Samus_CalcSpritemapPos(game.samus_pose().wrapping_mul(2), game);
        DrawSamusSpritemap(top_spritemap, position.k, position.j, game);
        if kSamusIsBottomDrawnFuncs[usize::from(game.samus_movement_type())]
            .expect("non-null function pointer")(game)
            & 1
            != 0
        {
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
pub fn SamusBottomDrawn_1(game: &mut Game) -> u8 {
    1u8
}
pub unsafe fn SamusBottomDrawn_4(game: &mut Game) -> u8 {
    set_ram_word!(game.samus_bottom_half_spritemap_index = 0u16);
    0u8
}
pub unsafe fn SamusBottomDrawn_0_Standing(game: &mut Game) -> u8 {
    let mut v1: *mut OamEnt = null_mut::<OamEnt>();
    if ram_word!(game, samus_pose) as i32 == kPose_00_FaceF_Powersuit as i32 {
        let v0: u16 = ram_word!(game, oam_next_ptr);
        v1 = gOamEnt(ram_word!(game, oam_next_ptr) as i32, game);
        *(&raw mut (*v1).xcoord as *mut u16) =
            (ram_word!(game, samus_x_pos) as i32 - 7i32 - ram_word!(game, layer1_x_pos) as i32)
                as u16;
        *(&raw mut (*v1).ycoord as *mut u16) =
            (ram_word!(game, samus_y_pos) as i32 - 17i32 - ram_word!(game, layer1_y_pos) as i32)
                as u16;
        *(&raw mut (*v1).charnum as *mut u16) = 14369u16;
        set_ram_word!(game.oam_next_ptr = (v0).wrapping_add(4u16));
    }
    1u8
}
pub fn SamusBottomDrawn_3_SpinJump(game: &mut Game) -> u8 {
    let pose = SamusPose::from(game.samus_pose());
    let frame = game.samus_anim_frame();
    if matches!(
        pose,
        kPose_81_FaceR_Screwattack
            | kPose_82_FaceL_Screwattack
            | kPose_1B_FaceR_SpaceJump
            | kPose_1C_FaceL_SpaceJump
    ) || frame == 0
        || frame.wrapping_sub(11) & 0x8000 == 0
    {
        return 1;
    }
    game.g_ram[0xaca..0xacc].fill(0);
    0
}
pub fn SamusBottomDrawn_A_Knockback(game: &mut Game) -> u8 {
    let crystal_flash_end = matches!(
        SamusPose::from(game.samus_pose()),
        kPose_D7_FaceR_CrystalFlashEnd | kPose_D8_FaceL_CrystalFlashEnd
    );
    if !crystal_flash_end || game.samus_anim_frame().wrapping_sub(3) & 0x8000 == 0 {
        return 1;
    }
    game.g_ram[0xaca..0xacc].fill(0);
    0
}
pub unsafe fn SamusBottomDrawn_F_Transitions(game: &mut Game) -> u8 {
    let draw_bottom: bool;
    if ram_word!(game, samus_pose) as i32 - kPose_F1_FaceR_CrouchTrans_AimU as i32 & 0x8000i32 == 0
    {
        return 1u8;
    }
    if ram_word!(game, samus_pose) as i32 - 219i32 & 0x8000i32 == 0 {
        draw_bottom = if ram_word!(game, samus_pose) as i32 - 221i32 & 0x8000i32 != 0 {
            (ram_word!(game, samus_anim_frame) != 0)
        } else {
            (ram_word!(game, samus_anim_frame) != 2u16)
        };
        if !(draw_bottom) {
            return 1u8;
        }
    } else if ram_word!(game, samus_pose) as i32 == kPose_35_FaceR_CrouchTrans as i32
        || ram_word!(game, samus_pose) as i32 == kPose_36_FaceL_CrouchTrans as i32
        || ram_word!(game, samus_pose) as i32 == kPose_3B_FaceR_StandTrans as i32
        || ram_word!(game, samus_pose) as i32 == kPose_3C_FaceL_StandTrans as i32
    {
        return 1u8;
    }
    set_ram_word!(game.samus_bottom_half_spritemap_index = 0u16);
    0u8
}
pub unsafe fn UNUSED_SamusBottomDrawn_D(game: &mut Game) -> u8 {
    if ram_word!(game, samus_pose) as i32
        != kPose_44_FaceL_Turn_Crouch as i32 | kPose_01_FaceR_Normal as i32 | 0x20i32
        && ram_word!(game, samus_pose) as i32
            != kPose_44_FaceL_Turn_Crouch as i32 | kPose_02_FaceL_Normal as i32 | 0x20i32
        || ram_word!(game, samus_anim_frame) as i32 - 1i32 & 0x8000i32 != 0
    {
        return 1u8;
    }
    set_ram_word!(game.samus_bottom_half_spritemap_index = 0u16);
    0u8
}
pub fn SamusBottomDrawn_14_WallJump(game: &mut Game) -> u8 {
    if !(3..13).contains(&game.samus_anim_frame()) {
        return 1;
    }
    game.g_ram[0xaca..0xacc].fill(0);
    0
}
pub fn SamusBottomDrawn_19_DamageBoost(game: &mut Game) -> u8 {
    if !(2..9).contains(&game.samus_anim_frame()) {
        return 1;
    }
    game.g_ram[0xaca..0xacc].fill(0);
    0
}
pub unsafe fn SamusBottomDrawn_1B(game: &mut Game) -> u8 {
    if ram_word!(game, samus_pose) as i32 - kPose_CF_FaceR_Ranintowall_AimUR as i32 & 0x8000i32 != 0
    {
        if ram_word!(game, samus_pose) as i32 != kPose_CB_FaceR_Shinespark_Vert as i32
            && ram_word!(game, samus_pose) as i32 != kPose_CC_FaceL_Shinespark_Vert as i32
        {
            return 1u8;
        }
    } else if ram_word!(game, samus_pose) as i32 != kPose_E8_FaceR_Drained_CrouchFalling as i32
        && ram_word!(game, samus_pose) as i32 != kPose_E9_FaceL_Drained_CrouchFalling as i32
        || ram_word!(game, samus_anim_frame) as i32 - 2i32 & 0x8000i32 == 0
    {
        return 1u8;
    }
    set_ram_word!(game.samus_bottom_half_spritemap_index = 0u16);
    0u8
}
pub unsafe fn Samus_DrawEchoes(game: &mut Game) {
    if (ram_word!(game, speed_echoes_index) as i32 & 0x8000i32) == 0i32 {
        if (ram_word!(game, speed_boost_counter) as i32 & 0xff00i32) == 1024i32 {
            if ram_word!(game, speed_echo_xpos[1]) != 0 {
                Samus_DrawEcho(2u16, game);
            }
            if ram_word!(game, speed_echo_xpos[0]) != 0 {
                Samus_DrawEcho(0u16, game);
            }
        }
        return;
    }
    let mut skip_echo: bool;
    let mut i: i32 = 2i32;
    while i >= 0i32 {
        let v1: i32 = i >> 1i32;
        if (ram_word!(game, speed_echo_xpos[v1]) != 0) {
            let v2: u16 = ram_word!(game, speed_echo_ypos[v1]);
            let mut v3: u16 = 0;
            if v2 != ram_word!(game, samus_y_pos) {
                if ((v2) as i16).wrapping_sub((ram_word!(game, samus_y_pos)) as i16) < 0i16 {
                    v3 = (v2).wrapping_add(2u16);
                } else {
                    v3 = (v2).wrapping_sub(2u16);
                }
                set_ram_word!(game.speed_echo_ypos[v1] = v3);
            }
            skip_echo = if (ram_word!(game, speed_echo_xspeed[v1]) as i32 & 0x8000i32) != 0i32 {
                let v5: u16 = (ram_word!(game, speed_echo_xspeed[v1]))
                    .wrapping_add(ram_word!(game, speed_echo_xpos[v1]));
                set_ram_word!(game.speed_echo_xpos[v1] = v5);
                (((v5) as i16).wrapping_sub((ram_word!(game, samus_x_pos)) as i16) < 0i16)
                    && ({
                        set_ram_word!(game.speed_echo_xpos[v1] = 0u16);
                        true
                    })
            } else {
                let v4: u16 = (ram_word!(game, speed_echo_xspeed[v1]))
                    .wrapping_add(ram_word!(game, speed_echo_xpos[v1]));
                set_ram_word!(game.speed_echo_xpos[v1] = v4);
                (((v4) as i16).wrapping_sub((ram_word!(game, samus_x_pos)) as i16) >= 0i16)
                    && ({
                        set_ram_word!(game.speed_echo_xpos[v1] = 0u16);
                        true
                    })
            };
            if !(skip_echo) {
                Samus_DrawEcho(i as u16, game);
            }
        }
        i -= 2i32;
    }
    if ram_word!(game, speed_echo_xpos[1]) == 0 && ram_word!(game, speed_echo_xpos[0]) == 0 {
        set_ram_word!(game.speed_echoes_index = 0u16);
    }
}
pub unsafe fn Samus_DrawEcho(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    let v2: i16 = (ram_word!(game, speed_echo_ypos[v1]) as i32
        - (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629)) as *mut SamusPoseParams)
            .offset(ram_word!(game, samus_pose) as isize))
        .y_offset_to_gfx as i32
        - ram_word!(game, layer1_y_pos) as i32) as i16;
    if v2 >= 0i16 && v2 as i32 - 248i32 & 0x8000i32 != 0 {
        DrawSamusSpritemap(
            ram_word!(game, samus_top_half_spritemap_index),
            (ram_word!(game, speed_echo_xpos[v1])).wrapping_sub(ram_word!(game, layer1_x_pos)),
            v2 as u16,
            game,
        );
        if ram_word!(game, samus_bottom_half_spritemap_index) != 0 {
            DrawSamusSpritemap(
                ram_word!(game, samus_bottom_half_spritemap_index),
                (ram_word!(game, speed_echo_xpos[v1])).wrapping_sub(ram_word!(game, layer1_x_pos)),
                v2 as u16,
                game,
            );
        }
    }
}
pub unsafe fn Samus_DrawShinesparkCrashEchoes(k: u16, game: &mut Game) {
    if (ram_word!(game, nmi_frame_counter_word) as i32 & 1i32) != 0i32 {
        let a: u16 = (ram_word!(game, samus_anim_frame)).wrapping_add(crate::rtl::rom_table_word!(
            game,
            0x929263,
            ram_word!(game, samus_pose) as isize
        ));
        let v2: i16 = (ram_word!(game, speed_echo_ypos[(k as i32 >> 1i32)]) as i32
            - (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629)) as *mut SamusPoseParams)
                .offset(ram_word!(game, samus_pose) as isize))
            .y_offset_to_gfx as i32
            - ram_word!(game, layer1_y_pos) as i32) as i16;
        if v2 >= 0i16 && v2 as i32 - 248i32 & 0x8000i32 != 0 {
            DrawSamusSpritemap(
                a,
                (ram_word!(game, speed_echo_xpos[(k as i32 >> 1i32)]))
                    .wrapping_sub(ram_word!(game, layer1_x_pos)),
                v2 as u16,
                game,
            );
            if kSamusIsBottomDrawnFuncs[*ram_ptr!(game; 0xa1f) as usize]
                .expect("non-null function pointer")(game) as i32
                & 1i32
                != 0
            {
                let v5: u16 =
                    (ram_word!(game, samus_anim_frame)).wrapping_add(crate::rtl::rom_table_word!(
                        game,
                        0x92945d,
                        ram_word!(game, samus_pose) as isize
                    ));
                DrawSamusSpritemap(
                    v5,
                    (ram_word!(game, speed_echo_xpos[(k as i32 >> 1i32)]))
                        .wrapping_sub(ram_word!(game, layer1_x_pos)),
                    v2 as u16,
                    game,
                );
            }
        }
    }
}
pub unsafe fn Samus_DrawShinesparkCrashEchoProjectiles(game: &mut Game) {
    if (ram_word!(game, nmi_frame_counter_word) as i32 & 1i32) != 0i32 {
        if ram_word!(game, speed_echo_xspeed[3]) != 0 {
            Samus_DrawEcho(6u16, game);
        }
        if ram_word!(game, speed_echo_xspeed[2]) != 0 {
            Samus_DrawEcho(4u16, game);
        }
    }
}
pub unsafe fn Samus_DrawStartingDeathAnim(game: &mut Game) {
    let v0: bool = (update_ram_word!(game.samus_anim_frame_timer, wrapping_sub(1), reread) as i32
        & 0x8000i32)
        != 0i32;
    if ram_word!(game, samus_anim_frame_timer) == 0 || v0 {
        update_ram_word!(game.samus_anim_frame, wrapping_add(1));
        Samus_HandleAnimDelay(game);
    }
    Samus_DrawDuringDeathAnim(game);
}
pub unsafe fn Samus_DrawDuringDeathAnim(game: &mut Game) {
    let v1: u16 = (2u16).wrapping_mul(ram_word!(game, samus_pose));
    let a: u16 = (ram_word!(game, samus_anim_frame)).wrapping_add(crate::rtl::rom_table_word!(
        game,
        0x929263,
        ram_word!(game, samus_pose) as isize
    ));
    Samus_CalcSpritemapPos((2u16).wrapping_mul(ram_word!(game, samus_pose)), game);
    DrawSamusSpritemap(
        a,
        (ram_word!(game, layer1_x_pos)).wrapping_add(ram_word!(game, samus_spritemap_x_pos)),
        (ram_word!(game, layer1_y_pos)).wrapping_add(ram_word!(game, samus_spritemap_y_pos)),
        game,
    );
    let R36: u16 = v1;
    if kSamusIsBottomDrawnFuncs[*ram_ptr!(game; 0xa1f) as usize].expect("non-null function pointer")(
        game,
    ) as i32
        & 1i32
        != 0
    {
        DrawSamusSpritemap(
            (ram_word!(game, samus_anim_frame)).wrapping_add(crate::rtl::rom_table_word!(
                game,
                0x92945d,
                (R36 as i32 >> 1i32) as isize
            )),
            (ram_word!(game, layer1_x_pos)).wrapping_add(ram_word!(game, samus_spritemap_x_pos)),
            (ram_word!(game, layer1_y_pos)).wrapping_add(ram_word!(game, samus_spritemap_y_pos)),
            game,
        );
    }
    SetSamusTilesDefsForCurAnim(game);
}
pub unsafe fn Samus_DrawWhenNotAnimatingOrDying(game: &mut Game) {
    let mut v0: PairU16 = PairU16 { k: 0, j: 0 };
    let v2: u16 = (2u16).wrapping_mul(ram_word!(game, samus_pose));
    let a: u16 = (ram_word!(game, samus_anim_frame)).wrapping_add(crate::rtl::rom_table_word!(
        game,
        0x929263,
        ram_word!(game, samus_pose) as isize
    ));
    v0 = Samus_CalcSpritemapPos((2u16).wrapping_mul(ram_word!(game, samus_pose)), game);
    DrawSamusSpritemap(a, v0.k, v0.j, game);
    let R36: u16 = v2;
    if kSamusIsBottomDrawnFuncs[*ram_ptr!(game; 0xa1f) as usize].expect("non-null function pointer")(
        game,
    ) as i32
        & 1i32
        != 0
    {
        DrawSamusSpritemap(
            (ram_word!(game, samus_anim_frame)).wrapping_add(crate::rtl::rom_table_word!(
                game,
                0x92945d,
                (R36 as i32 >> 1i32) as isize
            )),
            ram_word!(game, samus_spritemap_x_pos),
            ram_word!(game, samus_spritemap_y_pos),
            game,
        );
    }
    SetSamusTilesDefsForCurAnim(game);
}
static kAtmosphericTypeFuncs: [Option<FuncXY_V>; 8] = callback_table! { unsafe FuncXY_V;
    None, AtmosphericTypeFunc_1_FootstepSplash, AtmosphericTypeFunc_1_FootstepSplash,
    AtmosphericTypeFunc_3_DivingSplash, AtmosphericTypeFunc_4_LavaSurfaceDmg,
    AtmosphericTypeFunc_5_Bubbles, AtmosphericTypeFunc_67_Dust, AtmosphericTypeFunc_67_Dust,
};
pub unsafe fn HandleAtmosphericEffects(game: &mut Game) {
    let mut skip_effect: bool;
    let mut i: i32 = 6i32;
    while i >= 0i32 {
        let v1: i32 = i >> 1i32;
        let mut v2: u16 = ram_word!(game, atmospheric_gfx_frame_and_type[v1]);
        if (v2 != 0) {
            let r18: u16 = (2u16).wrapping_mul((v2 as u8) as u16);
            let v3: u16 = (2u16).wrapping_mul((*(&raw mut v2 as *mut u8).offset(1isize)) as u16);
            let mut v4: i16 =
                ((ram_word!(game, atmospheric_gfx_anim_timer[v1])) as i16).wrapping_sub(1i16);
            set_ram_word!(game.atmospheric_gfx_anim_timer[v1] = v4 as u16);
            skip_effect = if v4 != 0 {
                (v4 < 0i16)
                    && ((v4 != (0x8000i32 as i16))
                        || ({
                            set_ram_word!(
                                game.atmospheric_gfx_anim_timer[v1] = *(RomPtr_90(
                                    (r18).wrapping_add(crate::rtl::rom_table_word!(
                                        game,
                                        0x908b93,
                                        (v3 as i32 >> 1i32) as isize,
                                    ),),
                                    game,
                                )
                                    as *mut u16)
                            );
                            false
                        }))
            } else {
                set_ram_word!(
                    game.atmospheric_gfx_anim_timer[v1] = *(RomPtr_90(
                        (r18).wrapping_add(crate::rtl::rom_table_word!(
                            game,
                            0x908b93,
                            (v3 as i32 >> 1i32) as isize
                        ),),
                        game,
                    ) as *mut u16)
                );
                let v5: u16 =
                    (ram_word!(game, atmospheric_gfx_frame_and_type[v1])).wrapping_add(1u16);
                set_ram_word!(game.atmospheric_gfx_frame_and_type[v1] = v5);
                (((v5 as u8) as i16).wrapping_sub(
                    (crate::rtl::rom_table_word!(game, 0x908bef, (v3 as i32 >> 1i32) as isize))
                        as i16,
                ) >= 0i16)
                    && ({
                        set_ram_word!(game.atmospheric_gfx_frame_and_type[v1] = 0u16);
                        true
                    })
            };
            if !(skip_effect) {
                kAtmosphericTypeFuncs[(v3 as i32 >> 1i32) as usize]
                    .expect("non-null function pointer")(v3, i as u16, game);
            }
        }
        i -= 2i32;
    }
}
pub unsafe fn AtmosphericTypeFunc_1_FootstepSplash(k: u16, j: u16, game: &mut Game) {
    let mut v5: *mut OamEnt = null_mut::<OamEnt>();
    let v2: i32 = j as i32 >> 1i32;
    let r18: u16 = (2u16).wrapping_mul(
        (*((ram_ptr!(game, u16; 0xaec)).offset(v2 as isize) as *mut u8).offset(0)) as u16,
    );
    let v3: u16 = ram_word!(game, oam_next_ptr);
    let mut v4: i16 = (ram_word!(game, atmospheric_gfx_x_pos[v2]) as i32
        - ram_word!(game, layer1_x_pos) as i32
        - 4i32) as i16;
    if v4 >= 0i16
        && ram_word!(game, atmospheric_gfx_x_pos[v2]) as i32
            - ram_word!(game, layer1_x_pos) as i32
            - 260i32
            & 0x8000i32
            != 0
    {
        v5 = gOamEnt(ram_word!(game, oam_next_ptr) as i32, game);
        (*v5).xcoord = v4 as u8;
        let mut v6: i16 = (ram_word!(game, atmospheric_gfx_y_pos[v2]) as i32
            - ram_word!(game, layer1_y_pos) as i32
            - 4i32) as i16;
        if v6 >= 0i16
            && ram_word!(game, atmospheric_gfx_y_pos[v2]) as i32
                - ram_word!(game, layer1_y_pos) as i32
                - 260i32
                & 0x8000i32
                != 0
        {
            (*v5).ycoord = v6 as u8;
            *(&raw mut (*v5).charnum as *mut u16) = *(RomPtr_90(
                (r18).wrapping_add(crate::rtl::rom_table_word!(
                    game,
                    0x908bff,
                    (k as i32 >> 1i32) as isize
                )),
                game,
            ) as *mut u16);
            set_ram_word!(game.oam_next_ptr = (v3).wrapping_add(4u16));
        }
    }
}
pub unsafe fn AtmosphericTypeFunc_Common(j: u16, a: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    if (ram_word!(game, atmospheric_gfx_y_pos[v1]) as i32 - ram_word!(game, layer1_y_pos) as i32
        & 0xff00i32)
        == 0i32
    {
        DrawSamusSpritemap(
            a,
            (ram_word!(game, atmospheric_gfx_x_pos[v1]))
                .wrapping_sub(ram_word!(game, layer1_x_pos)),
            (ram_word!(game, atmospheric_gfx_y_pos[v1]))
                .wrapping_sub(ram_word!(game, layer1_y_pos)),
            game,
        );
    }
}
pub unsafe fn AtmosphericTypeFunc_3_DivingSplash(k: u16, j: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1i32;
    let r18: u16 = *((ram_ptr!(game, u16; 0xaec)).offset(v2 as isize) as *mut u8).offset(0) as u16;
    set_ram_word!(game.atmospheric_gfx_y_pos[v2] = ram_word!(game, fx_y_pos));
    AtmosphericTypeFunc_Common(j, (r18).wrapping_add(399u16), game);
}
pub unsafe fn AtmosphericTypeFunc_4_LavaSurfaceDmg(k: u16, j: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1i32;
    let mut v3: u16 = 0;
    if (j as i32 & 4i32) != 0i32 {
        v3 = (ram_word!(game, atmospheric_gfx_x_pos[v2])).wrapping_sub(1u16);
    } else {
        v3 = (ram_word!(game, atmospheric_gfx_x_pos[v2])).wrapping_add(1u16);
    }
    set_ram_word!(game.atmospheric_gfx_x_pos[v2] = v3);
    update_ram_word!(
        game.atmospheric_gfx_y_pos[(j as i32 >> 1i32)],
        wrapping_sub(1)
    );
    AtmosphericTypeFunc_1_FootstepSplash(k, j, game);
}
pub unsafe fn AtmosphericTypeFunc_67_Dust(k: u16, j: u16, game: &mut Game) {
    update_ram_word!(
        game.atmospheric_gfx_y_pos[(j as i32 >> 1i32)],
        wrapping_sub(1)
    );
    AtmosphericTypeFunc_1_FootstepSplash(k, j, game);
}
pub unsafe fn AtmosphericTypeFunc_5_Bubbles(k: u16, j: u16, game: &mut Game) {
    let r18: u16 = *((ram_ptr!(game, u16; 0xaec)).offset((j as i32 >> 1i32) as isize) as *mut u8)
        .offset(0) as u16;
    AtmosphericTypeFunc_Common(j, (r18).wrapping_add(390u16), game);
}
static kSamus_CalcSpritemapPos: [Option<Func_Y_To_PairU16>; 28] = callback_table! { Func_Y_To_PairU16;
    Samus_CalcSpritemapPos_Standing, Samus_CalcSpritemapPos_Default,
    Samus_CalcSpritemapPos_Default, Samus_CalcSpritemapPos_Default,
    Samus_CalcSpritemapPos_Default, Samus_CalcSpritemapPos_Default,
    Samus_CalcSpritemapPos_Default, Samus_CalcSpritemapPos_Default,
    Samus_CalcSpritemapPos_Default, Samus_CalcSpritemapPos_Default,
    Samus_CalcSpritemapPos_Default, Samus_CalcSpritemapPos_Default,
    Samus_CalcSpritemapPos_Default, Samus_CalcSpritemapPos_Default,
    Samus_CalcSpritemapPos_Default, Samus_CalcSpritemapPos_Crouch,
    Samus_CalcSpritemapPos_Default, Samus_CalcSpritemapPos_Default,
    Samus_CalcSpritemapPos_Default, Samus_CalcSpritemapPos_Default,
    Samus_CalcSpritemapPos_Default, Samus_CalcSpritemapPos_Default,
    Samus_CalcSpritemapPos_Default, Samus_CalcSpritemapPos_Default,
    Samus_CalcSpritemapPos_Default, Samus_CalcSpritemapPos_Default,
    Samus_CalcSpritemapPos_Default, Samus_CalcSpritemapPos_Special,
};
pub unsafe fn Samus_CalcSpritemapPos(k: u16, game: &mut Game) -> PairU16 {
    let mut v1: PairU16 = PairU16 { k: 0, j: 0 };
    if (ram_word!(game, ceres_status) as i32 & 0x8000i32) == 0i32 {
        v1 = kSamus_CalcSpritemapPos[*ram_ptr!(game; 0xa1f) as usize]
            .expect("non-null function pointer")(k, game);
    } else {
        let old_x: u16 = ram_word!(game, samus_x_pos);
        let old_y: u16 = ram_word!(game, samus_y_pos);
        Samus_CalcPos_Mode7(game);
        v1 = kSamus_CalcSpritemapPos[*ram_ptr!(game; 0xa1f) as usize]
            .expect("non-null function pointer")(k, game);
        set_ram_word!(game.samus_y_pos = old_y);
        set_ram_word!(game.samus_x_pos = old_x);
    }
    MakePairU16(v1.k, v1.j)
}
pub unsafe fn Samus_CalcSpritemapPos_Default(j: u16, game: &mut Game) -> PairU16 {
    let v1: i32 = *(&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
        as *mut SamusPoseParams)
        .offset(0))
    .y_offset_to_gfx)
        .offset((4i32 * j as i32) as isize) as i8 as i32;
    set_ram_word!(
        game.samus_spritemap_y_pos = (ram_word!(game, samus_y_pos) as i32
            - v1
            - ram_word!(game, layer1_y_pos) as i32) as u16
    );
    set_ram_word!(
        game.samus_spritemap_x_pos =
            (ram_word!(game, samus_x_pos)).wrapping_sub(ram_word!(game, layer1_x_pos))
    );
    MakePairU16(
        ram_word!(game, samus_spritemap_x_pos),
        ram_word!(game, samus_spritemap_y_pos),
    )
}
pub unsafe fn Samus_CalcSpritemapPos_Standing(j: u16, game: &mut Game) -> PairU16 {
    static g_byte_908D28: [u8; 16] = [3, 6, 0, 0, 3, 6, 0, 0, 3, 3, 6, 0, 3, 3, 6, 0];
    let mut v3: PairU16 = PairU16 { k: 0, j: 0 };
    let mut v1: i16 = (j as i32 >> 1i32) as i16;
    if j as i32 >> 1i32 == 0 || v1 as i32 == kPose_9B_FaceF_VariaGravitySuit as i32 {
        if ram_word!(game, samus_anim_frame) as i32 - 2i32 & 0x8000i32 == 0 {
            set_ram_word!(
                game.samus_spritemap_y_pos = (ram_word!(game, samus_y_pos) as i32
                    - 1i32
                    - ram_word!(game, layer1_y_pos) as i32)
                    as u16
            );
            set_ram_word!(
                game.samus_spritemap_x_pos =
                    (ram_word!(game, samus_x_pos)).wrapping_sub(ram_word!(game, layer1_x_pos))
            );
            return MakePairU16(
                (ram_word!(game, samus_x_pos)).wrapping_sub(ram_word!(game, layer1_x_pos)),
                (ram_word!(game, samus_y_pos) as i32 - 1i32 - ram_word!(game, layer1_y_pos) as i32)
                    as u16,
            );
        }
    } else if v1 as i32 - kPose_A4_FaceR_LandJump as i32 & 0x8000i32 == 0
        && v1 as i32 - kPose_A8_FaceR_Grappling as i32 & 0x8000i32 != 0
    {
        let r18: u16 = *((&raw const g_byte_908D28 as *const u8).offset(
            (ram_word!(game, samus_anim_frame) as i32 + 4i32 * (v1 as i32 - 164i32)) as u16
                as isize,
        ) as *mut u16);
        set_ram_word!(
            game.samus_spritemap_y_pos = (ram_word!(game, samus_y_pos) as i32
                - r18 as i32
                - ram_word!(game, layer1_y_pos) as i32)
                as u16
        );
        set_ram_word!(
            game.samus_spritemap_x_pos =
                (ram_word!(game, samus_x_pos)).wrapping_sub(ram_word!(game, layer1_x_pos))
        );
        return MakePairU16(
            (ram_word!(game, samus_x_pos)).wrapping_sub(ram_word!(game, layer1_x_pos)),
            (ram_word!(game, samus_y_pos) as i32
                - r18 as i32
                - ram_word!(game, layer1_y_pos) as i32) as u16,
        );
    }
    v3 = Samus_CalcSpritemapPos_Default(j, game);
    MakePairU16(v3.k, v3.j)
}
pub unsafe fn Samus_CalcSpritemapPos_Crouch(j: u16, game: &mut Game) -> PairU16 {
    static byte_908D80: [i8; 24] = [
        -8, 0, -8, 0, -4, -2, -4, -2, 0, 0, 0, 0, -4, 0, -4, 0, 5, 4, 5, 4, 0, 0, 0, 0,
    ];
    let mut v1: i16 = (j as i32 >> 1i32) as i16;
    if (j as i32 >> 1i32) - kPose_35_FaceR_CrouchTrans as i32 & 0x8000i32 != 0
        || v1 as i32 - kPose_41_FaceL_Morphball_Ground as i32 & 0x8000i32 == 0
    {
        Samus_CalcSpritemapPos_Default(j, game)
    } else {
        let v4: i32 = byte_908D80[(ram_word!(game, samus_anim_frame) as i32
            + 2i32 * (v1 as i32 - 53i32)) as u16 as usize] as i32;
        set_ram_word!(
            game.samus_spritemap_y_pos = (v4 + ram_word!(game, samus_y_pos) as i32
                - ram_word!(game, layer1_y_pos) as i32)
                as u16
        );
        set_ram_word!(
            game.samus_spritemap_x_pos =
                (ram_word!(game, samus_x_pos)).wrapping_sub(ram_word!(game, layer1_x_pos))
        );
        MakePairU16(
            (ram_word!(game, samus_x_pos)).wrapping_sub(ram_word!(game, layer1_x_pos)),
            ram_word!(game, samus_spritemap_y_pos),
        )
    }
}
pub unsafe fn Samus_CalcSpritemapPos_Special(j: u16, game: &mut Game) -> PairU16 {
    static byte_908DEF: [i8; 32] = [
        7, 5, -8, -8, -8, -8, -8, -5, 4, 4, 4, 4, 0, 0, 4, -3, -5, 0, 0, 4, -3, -5, -3, 4, 0, 0, 4,
        0, 0, 4, 0, 0,
    ];
    let mut v2: i32 = 0;
    let mut v1: i32 = j as i32 >> 1i32;
    if v1 == kPose_E8_FaceR_Drained_CrouchFalling as i32
        || v1 == kPose_E9_FaceL_Drained_CrouchFalling as i32
    {
        v2 = byte_908DEF[ram_word!(game, samus_anim_frame) as usize] as i32;
    } else {
        if v1 != kPose_EA_FaceR_Drained_Stand as i32 && v1 != kPose_EB_FaceL_Drained_Stand as i32
            || ((ram_word!(game, samus_anim_frame)) as i16).wrapping_sub(5i16) < 0i16
        {
            return Samus_CalcSpritemapPos_Default(j, game);
        }
        v2 = -3i32;
    }
    set_ram_word!(
        game.samus_spritemap_y_pos = (v2 + ram_word!(game, samus_y_pos) as i32
            - ram_word!(game, layer1_y_pos) as i32) as u16
    );
    set_ram_word!(
        game.samus_spritemap_x_pos =
            (ram_word!(game, samus_x_pos)).wrapping_sub(ram_word!(game, layer1_x_pos))
    );
    MakePairU16(
        (ram_word!(game, samus_x_pos)).wrapping_sub(ram_word!(game, layer1_x_pos)),
        (v2 + ram_word!(game, samus_y_pos) as i32 - ram_word!(game, layer1_y_pos) as i32) as u16,
    )
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
    let r18: u16 = Samus_GetBottom_R18(game);
    match (ram_word!(game, fx_type) as i32 & 0xfi32) >> 1i32 {
        1 | 2 => {
            if (ram_word!(game, lava_acid_y_pos) as i32 & 0x8000i32) == 0i32
                && ram_word!(game, lava_acid_y_pos) as i32 - r18 as i32 & 0x8000i32 != 0
            {
                set_ram_word!(game.liquid_physics_type = kLiquidPhysicsType_LavaAcid as u16);
            } else {
                set_ram_word!(game.liquid_physics_type = 0u16);
            }
        }
        3 => {
            if (ram_word!(game, fx_y_pos) as i32 & 0x8000i32) == 0i32
                && ram_word!(game, fx_y_pos) as i32 - r18 as i32 & 0x8000i32 != 0
                && (ram_word!(game, fx_liquid_options) as i32 & 4i32) == 0i32
            {
                set_ram_word!(game.liquid_physics_type = kLiquidPhysicsType_Water as u16);
            } else {
                set_ram_word!(game.liquid_physics_type = 0u16);
            }
        }
        _ => {
            set_ram_word!(game.liquid_physics_type = 0u16);
        }
    };
}
pub unsafe fn Samus_HandleMovement_X(game: &mut Game) {
    Samus_HandleExtraRunspeedX(game);
    Samus_MoveX(
        Samus_CalcBaseSpeed_X(Samus_DetermineSpeedTableEntryPtr_X(game), game),
        game,
    );
}
pub unsafe fn Samus_MoveX(mut amt: i32, game: &mut Game) {
    let moving_left: bool = if ram_word!(game, samus_x_accel_mode) == 0
        || ram_word!(game, samus_x_accel_mode) == 2u16
    {
        !(*ram_ptr!(game; 0xa1e) != 4u8)
    } else {
        !(*ram_ptr!(game; 0xa1e) != 8u8)
    };
    if moving_left {
        amt = Samus_CalcDisplacementMoveLeft(amt, game);
    } else {
        amt = Samus_CalcDisplacementMoveRight(amt, game);
    }
    if amt < 0i32 {
        Samus_MoveLeft(amt, game);
    } else {
        Samus_MoveRight(amt, game);
    };
}
pub unsafe fn Samus_BombJumpRisingXMovement_(game: &mut Game) {
    let mut amt: i32 = 0;
    let moving_left: bool = if ram_word!(game, knockback_dir) != 0 {
        amt = Samus_CalcBaseSpeed_X(Samus_DetermineSpeedTableEntryPtr_X(game), game);
        (ram_word!(game, knockback_x_dir) == 0)
    } else {
        amt = Samus_CalcBaseSpeed_X(addr_stru_909F25 as u16, game);
        !((ram_word!(game, bomb_jump_dir) as u8) != 1u8)
    };
    if moving_left {
        amt = Samus_CalcDisplacementMoveLeft(amt, game);
    } else {
        amt = Samus_CalcDisplacementMoveRight(amt, game);
    }
    if amt < 0i32 {
        Samus_MoveLeft(amt, game);
    } else {
        Samus_MoveRight(amt, game);
    };
}
pub unsafe fn Samus_BombJumpRisingYMovement_(game: &mut Game) {
    if ram_word!(game, samus_y_dir) == 1u16 {
        if (ram_word!(game, samus_y_speed) as i32 & 0x8000i32) != 0i32 {
            set_ram_words! { game, 0u16; samus_y_subspeed, samus_y_speed }
            set_ram_word!(game.samus_y_dir = 2u16);
            if (ram_word!(game, bomb_jump_dir) as u8) != 2u8 {
                set_ram_word!(game.samus_x_accel_mode = 2u16);
            }
        } else if ram_word!(game, samus_y_speed) as i32 - 1i32 & 0x8000i32 != 0
            && ram_word!(game, samus_input_handler) as i32 != 0x90e91di32 & 0xffffi32
        {
            set_ram_word!(game.samus_input_handler = ((0x90e913i32) as u16));
        }
    }
}
pub unsafe fn Samus_BombJumpFallingXMovement_(game: &mut Game) {
    let mut amt: i32 = Samus_CalcBaseSpeed_X(Samus_DetermineSpeedTableEntryPtr_X(game), game);
    if ram_word!(game, samus_var62) == 1u16 {
        amt = Samus_CalcDisplacementMoveLeft(amt, game);
    } else {
        amt = Samus_CalcDisplacementMoveRight(amt, game);
    }
    if amt < 0i32 {
        Samus_MoveLeft(amt, game);
    } else {
        Samus_MoveRight(amt, game);
    };
}
pub unsafe fn Samus_BombJumpFallingYMovement_(game: &mut Game) {
    if ram_word!(game, samus_y_speed) as i32 - 5i32 & 0x8000i32 != 0 {
        AddToHiLo(
            ram_ptr!(game, u16; 0xb2e),
            ram_ptr!(game, u16; 0xb2c),
            (ram_word!(game, samus_y_accel) as u32) << 16i32
                | ram_word!(game, samus_y_subaccel) as u32,
        );
    }
    Samus_MoveDown(
        ((ram_word!(game, samus_y_speed) as u32) << 16i32
            | ram_word!(game, samus_y_subspeed) as u32) as i32,
        game,
    );
}
pub unsafe fn Samus_JumpingMovement(game: &mut Game) {
    Samus_HandleExtraRunspeedX(game);
    if ram_word!(game, samus_pose) as i32 == kPose_4B_FaceR_Jumptrans as i32
        || ram_word!(game, samus_pose) as i32 == kPose_4C_FaceL_Jumptrans as i32
        || ram_word!(game, samus_pose) as i32 - kPose_55_FaceR_Jumptrans_AimU as i32 & 0x8000i32
            == 0
            && ram_word!(game, samus_pose) as i32 - kPose_5B as i32 & 0x8000i32 != 0
    {
        set_ram_word!(game.samus_x_accel_mode = 0u16);
        Samus_Move_NoBaseSpeed_X(game);
        Samus_MoveExtraY(game);
        return;
    }
    if ram_word!(game, samus_y_dir) == 1u16
        && ((ram_word!(game, button_config_jump_a) as i32
            & ram_word!(game, joypad1_lastkeys) as i32)
            == 0i32
            || (ram_word!(game, samus_y_speed) as i32 & 0x8000i32) != 0i32)
    {
        set_ram_words! { game, 0u16; samus_y_subspeed, samus_y_speed }
        set_ram_word!(game.samus_y_dir = 2u16);
    }
    let pair: Pair_Bool_Amt =
        Samus_CalcBaseSpeed_NoDecel_X(Samus_DetermineSpeedTableEntryPtr_X(game), game);
    let move_horizontally: bool = if *ram_ptr!(game; 0xa1f) == 20u8 {
        if ram_word!(game, samus_x_accel_mode) == 0 {
            set_ram_word!(game.samus_x_accel_mode = 2u16);
        }
        true
    } else {
        (ram_word!(game, samus_x_accel_mode) != 0u16
            || (ram_word!(game, joypad1_lastkeys) as i32 & 0x100i32) != 0i32
            || (ram_word!(game, joypad1_lastkeys) as i32 & 0x200i32) != 0i32)
            || ({
                set_ram_words! { game, 0u16;
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
pub unsafe fn Samus_SpinJumpMovement(game: &mut Game) {
    let mut current_block: u64;
    static g_word_909E9F: u16 = 8u16;
    Samus_HandleExtraRunspeedX(game);
    if ram_word!(game, samus_y_dir) == 1u16
        && ((ram_word!(game, button_config_jump_a) as i32
            & ram_word!(game, joypad1_lastkeys) as i32)
            == 0i32
            || (ram_word!(game, samus_y_speed) as i32 & 0x8000i32) != 0i32)
    {
        set_ram_words! { game, 0u16; samus_y_subspeed, samus_y_speed }
        set_ram_word!(game.samus_y_dir = 2u16);
    }
    let mut pair: Pair_Bool_Amt =
        Samus_CalcBaseSpeed_NoDecel_X(Samus_DetermineSpeedTableEntryPtr_X(game), game);
    if pair.flag as i32 != 0 || ram_word!(game, samus_x_accel_mode) == 1u16 {
        current_block = 15844657471994422988;
    } else {
        if *ram_ptr!(game; 0xa1e) == 4u8 {
            if (ram_word!(game, joypad1_lastkeys) as i32 & kButton_Left as i32) != 0i32 {
                current_block = 15844657471994422988;
            } else {
                current_block = 11746438652493739369;
            }
        } else if (ram_word!(game, joypad1_lastkeys) as i32 & kButton_Right as i32) == 0i32 {
            current_block = 11746438652493739369;
        } else {
            current_block = 15844657471994422988;
        }
        if current_block == 15844657471994422988 {
        } else {
            pair.amt = 0i32;
            set_ram_words! { game, 0u16;
                samus_x_base_speed, samus_x_base_subspeed, samus_collision_flag, samus_x_accel_mode,
            }
            current_block = 7037269245041226537;
        }
    }
    if current_block == 15844657471994422988 && ram_word!(game, samus_x_accel_mode) == 0 {
        set_ram_word!(game.samus_x_accel_mode = 2u16);
    }
    Samus_MoveX(pair.amt, game);
    if Samus_WallJumpCheck((g_word_909E9F as i16 as i32) << 16i32, game) == 0 {
        Samus_MoveY_WithSpeedCalc(game);
    }
}
pub unsafe fn Samus_CheckStartFalling(game: &mut Game) {
    if ram_word!(game, samus_y_dir) == 1u16
        && (ram_word!(game, samus_y_speed) as i32 & 0x8000i32) != 0i32
    {
        set_ram_words! { game, 0u16; samus_y_subspeed, samus_y_speed }
        set_ram_word!(game.samus_y_dir = 2u16);
    }
}
pub unsafe fn Samus_MoveY_WithSpeedCalc(game: &mut Game) {
    let mut amt: i32 = ((ram_word!(game, samus_y_speed) as u32) << 16i32
        | ram_word!(game, samus_y_subspeed) as u32) as i32;
    if ram_word!(game, samus_y_dir) == 2u16 {
        if ram_word!(game, samus_y_speed) != 5u16 {
            AddToHiLo(
                ram_ptr!(game, u16; 0xb2e),
                ram_ptr!(game, u16; 0xb2c),
                (ram_word!(game, samus_y_accel) as u32) << 16i32
                    | ram_word!(game, samus_y_subaccel) as u32,
            );
        }
    } else {
        AddToHiLo(
            ram_ptr!(game, u16; 0xb2e),
            ram_ptr!(game, u16; 0xb2c),
            -((ram_word!(game, samus_y_accel) as i32) << 16i32
                | ram_word!(game, samus_y_subaccel) as i32) as u32,
        );
    }
    if ram_word!(game, samus_y_dir) != 2u16 {
        amt = -amt;
    }
    amt = (amt as u32).wrapping_add(
        (ram_word!(game, extra_samus_y_displacement) as u32) << 16i32
            | ram_word!(game, extra_samus_y_subdisplacement) as u32,
    ) as i32;
    if amt < 0i32 {
        Samus_MoveUp(amt, game);
    } else {
        Samus_MoveDown(amt, game);
    };
}
pub unsafe fn Samus_FallingMovement(game: &mut Game) {
    Samus_HandleExtraRunspeedX(game);
    let pair: Pair_Bool_Amt =
        Samus_CalcBaseSpeed_NoDecel_X(Samus_DetermineSpeedTableEntryPtr_X(game), game);
    if ram_word!(game, samus_x_accel_mode) != 0u16
        || (ram_word!(game, joypad1_lastkeys) as i32 & kButton_Right as i32) != 0i32
        || (ram_word!(game, joypad1_lastkeys) as i32 & kButton_Left as i32) != 0i32
    {
        Samus_MoveX(pair.amt, game);
    } else {
        set_ram_words! { game, 0u16;
            samus_x_base_speed, samus_x_base_subspeed, samus_collision_flag,
        }
    }
    Samus_CheckStartFalling(game);
    Samus_MoveY_WithSpeedCalc(game);
}
pub unsafe fn Samus_MorphedFallingMovement(game: &mut Game) {
    let mut pair: Pair_Bool_Amt =
        Samus_CalcBaseSpeed_NoDecel_X(Samus_DetermineSpeedTableEntryPtr_X(game), game);
    if ram_word!(game, samus_x_accel_mode) == 0
        && (ram_word!(game, joypad1_lastkeys) as i32 & kButton_Right as i32) == 0i32
        && (ram_word!(game, joypad1_lastkeys) as i32 & kButton_Left as i32) == 0i32
    {
        pair.amt = 0i32;
        set_ram_words! { game, 0u16;
            samus_x_base_speed, samus_x_base_subspeed, samus_collision_flag,
        }
    }
    Samus_MoveX(pair.amt, game);
    Samus_CheckStartFalling(game);
    Samus_MoveY_WithSpeedCalc(game);
}
pub unsafe fn Samus_MorphedBouncingMovement(game: &mut Game) {
    let mut pair: Pair_Bool_Amt =
        Samus_CalcBaseSpeed_NoDecel_X(Samus_DetermineSpeedTableEntryPtr_X(game), game);
    if ram_word!(game, samus_x_accel_mode) == 0
        && (ram_word!(game, joypad1_lastkeys) as i32 & kButton_Right as i32) == 0i32
        && (ram_word!(game, joypad1_lastkeys) as i32 & kButton_Left as i32) == 0i32
    {
        pair.amt = 0i32;
        set_ram_words! { game, 0u16;
            samus_x_base_speed, samus_x_base_subspeed, samus_collision_flag,
        }
    }
    Samus_MoveX(pair.amt, game);
    if ram_word!(game, knockback_dir) == 0 {
        if ram_word!(game, extra_samus_y_displacement) != 0u16
            || ram_word!(game, extra_samus_y_subdisplacement) != 0u16
        {
            set_ram_word!(game.samus_y_dir = 2u16);
            set_ram_words! { game, 0u16; samus_y_subspeed, samus_y_speed }
            let amt: i32 = ((ram_word!(game, extra_samus_y_displacement) as u32) << 16i32
                | ram_word!(game, extra_samus_y_subdisplacement) as u32)
                as i32;
            if amt < 0i32 {
                Samus_MoveUp(amt, game);
            } else {
                Samus_MoveDown(amt + ((1i16 as i32) << 16i32), game);
            }
        } else {
            Samus_CheckStartFalling(game);
            Samus_MoveY_WithSpeedCalc(game);
        }
    }
}
pub unsafe fn Samus_Move_NoSpeedCalc_Y(game: &mut Game) {
    let mut amt: i32 = ((ram_word!(game, extra_samus_y_displacement) as u32) << 16i32
        | ram_word!(game, extra_samus_y_subdisplacement) as u32) as i32;
    if amt == 0i32 {
        if ram_word!(game, samus_pos_adjusted_by_slope_flag) == 0 {
            amt = ((ram_word!(game, samus_total_x_speed) as u32) << 16i32
                | ram_word!(game, samus_total_x_subspeed) as u32) as i32;
        }
        Samus_MoveDown(amt + ((1i16 as i32) << 16i32), game);
    } else if amt < 0i32 {
        Samus_MoveUp(amt, game);
    } else {
        Samus_MoveDown(amt + ((1i16 as i32) << 16i32), game);
    };
}
pub unsafe fn Samus_MoveExtraY(game: &mut Game) {
    let amt: i32 = ((ram_word!(game, extra_samus_y_displacement) as u32) << 16i32
        | ram_word!(game, extra_samus_y_subdisplacement) as u32) as i32;
    if amt != 0i32 {
        if amt < 0i32 {
            Samus_MoveUp(amt, game);
        } else {
            Samus_MoveDown(amt + ((1i16 as i32) << 16i32), game);
        }
    }
}
pub unsafe fn Samus_CheckAndMoveY(game: &mut Game) -> u8 {
    if ram_word!(game, samus_y_dir) == 0 {
        return 0u8;
    }
    Samus_CheckStartFalling(game);
    Samus_MoveY_WithSpeedCalc(game);
    1u8
}
pub unsafe fn Samus_MoveY_Simple_(game: &mut Game) -> u8 {
    if ram_word!(game, samus_y_dir) == 0 {
        return 0u8;
    }
    Samus_CheckStartFalling(game);
    Samus_MoveY_WithSpeedCalc(game);
    1u8
}
pub unsafe fn Samus_Move_NoBaseSpeed_X(game: &mut Game) {
    Samus_MoveX(0i32, game);
}
pub unsafe fn Samus_MoveLeft(mut amt: i32, game: &mut Game) {
    amt = -amt;
    let cres: CheckEnemyColl_Result = Samus_CheckSolidEnemyColl(amt, game);
    set_ram_word!(game.samus_collision_flag = cres.collision);
    amt = cres.amt;
    if ram_word!(game, samus_collision_flag) != 0 {
        Samus_ClearXSpeedIfColl(game);
        Samus_MoveLeft_NoColl(amt, game);
        Samus_AlignYPosSlope(game);
    } else {
        amt = Samus_MoveRight_NoSolidColl(-amt, game);
        SetHiLo(
            ram_ptr!(game, u16; 0xdaa),
            ram_ptr!(game, u16; 0xdac),
            amt as u32,
        );
        if (ram_word!(game, samus_collision_direction) as i32 & 1i32) != 0i32 {
            set_ram_word!(game.samus_collision_flag = 0u16);
        }
        Samus_ClearXSpeedIfColl(game);
        Samus_AlignYPosSlope(game);
    };
}
pub unsafe fn Samus_MoveRight(mut amt: i32, game: &mut Game) {
    let cres: CheckEnemyColl_Result = Samus_CheckSolidEnemyColl(amt, game);
    set_ram_word!(game.samus_collision_flag = cres.collision);
    amt = cres.amt;
    if ram_word!(game, samus_collision_flag) != 0 {
        Samus_ClearXSpeedIfColl(game);
        Samus_MoveRight_NoColl(amt, game);
        Samus_AlignYPosSlope(game);
    } else {
        amt = Samus_MoveRight_NoSolidColl(amt, game);
        SetHiLo(
            ram_ptr!(game, u16; 0xdae),
            ram_ptr!(game, u16; 0xdb0),
            amt as u32,
        );
        if (ram_word!(game, samus_collision_direction) as i32 & 1i32) == 0i32 {
            set_ram_word!(game.samus_collision_flag = 0u16);
        }
        Samus_ClearXSpeedIfColl(game);
        Samus_AlignYPosSlope(game);
    };
}
pub unsafe fn Samus_MoveUp(mut amt: i32, game: &mut Game) {
    set_ram_word!(game.samus_collision_direction = 2u16);
    let cres: CheckEnemyColl_Result = Samus_CheckSolidEnemyColl(-amt, game);
    set_ram_word!(game.samus_collision_flag = cres.collision);
    amt = cres.amt;
    if ram_word!(game, samus_collision_flag) != 0 {
        Samus_MoveUp_SetPoseCalcInput(game);
        Samus_MoveUp_NoColl(amt, game);
    } else {
        amt = Samus_MoveDown_NoSolidColl(-amt, game);
        SetHiLo(
            ram_ptr!(game, u16; 0xdb2),
            ram_ptr!(game, u16; 0xdb4),
            amt as u32,
        );
        Samus_MoveUp_SetPoseCalcInput(game);
    };
}
pub unsafe fn Samus_MoveDown(mut amt: i32, game: &mut Game) {
    set_ram_word!(game.samus_collision_direction = 3u16);
    let cres: CheckEnemyColl_Result = Samus_CheckSolidEnemyColl(amt, game);
    set_ram_word!(game.samus_collision_flag = cres.collision);
    amt = cres.amt;
    if ram_word!(game, samus_collision_flag) != 0 {
        Samus_MoveDown_SetPoseCalcInput(game);
        Samus_MoveDown_NoColl(amt, game);
    } else {
        amt = Samus_MoveDown_NoSolidColl(amt, game);
        set_ram_word!(game.projectile_init_speed_samus_moved_down_fract = amt as u16);
        set_ram_word!(game.projectile_init_speed_samus_moved_down = (amt >> 16i32) as u16);
        Samus_MoveDown_SetPoseCalcInput(game);
    };
}
pub unsafe fn Samus_MoveHandler_ReleaseFromGrapple(game: &mut Game) {
    if ram_word!(game, samus_y_dir) == 1u16
        && (ram_word!(game, samus_y_speed) as i32 & 0x8000i32) != 0i32
    {
        set_ram_words! { game, 0u16; samus_y_subspeed, samus_y_speed }
        set_ram_word!(game.samus_y_dir = 2u16);
        set_ram_word!(game.samus_movement_handler = ((0x90a337i32) as u16));
    }
    set_ram_word!(game.samus_x_accel_mode = 2u16);
    let amt: i32 = Samus_CalcBaseSpeed_X(Samus_DetermineGrappleSwingSpeed_X(game), game);
    if ram_word!(game, samus_x_accel_mode) != 0u16
        || (ram_word!(game, joypad1_lastkeys) as i32 & kButton_Right as i32) != 0i32
        || (ram_word!(game, joypad1_lastkeys) as i32 & kButton_Left as i32) != 0i32
    {
        Samus_MoveX(amt, game);
    } else {
        set_ram_words! { game, 0u16;
            samus_x_base_speed, samus_x_base_subspeed, samus_collision_flag,
        }
    }
    Samus_MoveY_WithSpeedCalc(game);
    if ram_word!(game, input_to_pose_calc) != 0 {
        set_ram_word!(game.samus_movement_handler = ((0x90a337i32) as u16));
    }
}
pub unsafe fn Samus_HandleMovement_DrainedCrouching(game: &mut Game) {
    Samus_MoveY_WithSpeedCalc(game);
    if ram_word!(game, samus_collision_flag) != 0 {
        set_ram_word!(game.samus_movement_handler = ((0x90a337i32) as u16));
        set_ram_word!(game.samus_anim_frame_timer = 8u16);
        set_ram_word!(game.samus_anim_frame = 7u16);
        set_ram_words! { game, 0u16; samus_y_subspeed, samus_y_speed }
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallScrollingFinishedHook(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        9475465 => Samus_ScrollFinishedHook_SporeSpawnFight;
    }
}
pub unsafe fn MainScrollingRoutine(game: &mut Game) {
    let mut current_block: u64;
    if ram_word!(game, slow_grabble_scrolling_flag) != 0 {
        if ((ram_word!(game, samus_x_pos) as i32 & 0x8000i32) == 0i32) {
            let mut v0: u16 = 0;
            v0 = (ram_word!(game, samus_x_pos)).wrapping_sub(ram_word!(game, layer1_x_pos));
            if ram_word!(game, samus_x_pos) < ram_word!(game, layer1_x_pos) {
                current_block = 8565863539623487812;
            } else if v0 >= 0xa0u16 {
                update_ram_word!(game.layer1_x_pos, wrapping_add(3u16));
                current_block = 9616406889445156080;
            } else if v0 < 0x60u16 {
                current_block = 8565863539623487812;
            } else {
                current_block = 9616406889445156080;
            }
            if current_block == 8565863539623487812 {
                update_ram_word!(game.layer1_x_pos, wrapping_sub(3u16));
            }
            if (ram_word!(game, samus_y_pos) as i32 & 0x8000i32) == 0i32 {
                let v1: u16 =
                    (ram_word!(game, samus_y_pos)).wrapping_sub(ram_word!(game, layer1_y_pos));
                if ram_word!(game, samus_y_pos) >= ram_word!(game, layer1_y_pos) {
                    if v1 >= 0x90u16 {
                        update_ram_word!(game.layer1_y_pos, wrapping_add(3u16));
                        current_block = 13986801909406869559;
                    } else if v1 >= 0x70u16 {
                        current_block = 13986801909406869559;
                    } else {
                        current_block = 1856101646708284338;
                    }
                } else {
                    current_block = 1856101646708284338;
                }
                if current_block == 13986801909406869559 {
                } else {
                    update_ram_word!(game.layer1_y_pos, wrapping_sub(3u16));
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
    if ram_word!(game, scrolling_finished_hook) != 0 {
        CallScrollingFinishedHook(
            ((ram_word!(game, scrolling_finished_hook)) as u32) | (0x900000u32),
            game,
        );
    }
    set_ram_word!(game.samus_prev_x_pos = ram_word!(game, samus_x_pos));
    set_ram_word!(game.samus_prev_x_subpos = ram_word!(game, samus_x_subpos));
    set_ram_word!(game.samus_prev_y_pos = ram_word!(game, samus_y_pos));
    set_ram_word!(game.samus_prev_y_subpos = ram_word!(game, samus_y_subpos));
}
pub unsafe fn Samus_ScrollFinishedHook_SporeSpawnFight(game: &mut Game) {
    if ram_word!(game, layer1_y_pos) <= 0x1d0u16 {
        set_ram_word!(game.layer1_y_pos = 464u16);
    }
}
pub unsafe fn Samus_HandleScroll_X(game: &mut Game) {
    static kSamus_HandleScroll_X_FaceLeft: [u16; 4] = [0xa0, 0x50, 0x20, 0xe0];
    static kSamus_HandleScroll_X_FaceRight: [u16; 4] = [0x60, 0x40, 0x20, 0xe0];
    if ram_word!(game, samus_prev_x_pos) == ram_word!(game, samus_x_pos) {
        HandleAutoscrolling_X(game);
        return;
    }
    if (ram_word!(game, knockback_dir) != 0u16
        || *ram_ptr!(game; 0xa1f) == 16u8
        || ram_word!(game, samus_x_accel_mode) == 1u16) as i32
        ^ (*ram_ptr!(game; 0xa1e) != 4u8) as i32
        != 0
    {
        set_ram_word!(
            game.ideal_layer1_xpos = (ram_word!(game, samus_x_pos)).wrapping_sub(
                kSamus_HandleScroll_X_FaceRight
                    [(ram_word!(game, camera_distance_index) as i32 >> 1i32) as usize],
            )
        );
    } else {
        set_ram_word!(
            game.ideal_layer1_xpos = (ram_word!(game, samus_x_pos)).wrapping_sub(
                kSamus_HandleScroll_X_FaceLeft
                    [(ram_word!(game, camera_distance_index) as i32 >> 1i32) as usize],
            )
        );
    }
    if ram_word!(game, ideal_layer1_xpos) != ram_word!(game, layer1_x_pos) {
        if ((ram_word!(game, ideal_layer1_xpos)) as i16)
            .wrapping_sub((ram_word!(game, layer1_x_pos)) as i16)
            < 0i16
        {
            AddToHiLo(
                ram_ptr!(game, u16; 0x911),
                ram_ptr!(game, u16; 0x90f),
                -((ram_word!(game, absolute_moved_last_frame_x) as i32) << 16i32
                    | ram_word!(game, absolute_moved_last_frame_x_fract) as i32)
                    as u32,
            );
            HandleScrollingWhenTriggeringScrollLeft(game);
        } else {
            AddToHiLo(
                ram_ptr!(game, u16; 0x911),
                ram_ptr!(game, u16; 0x90f),
                (ram_word!(game, absolute_moved_last_frame_x) as u32) << 16i32
                    | ram_word!(game, absolute_moved_last_frame_x_fract) as u32,
            );
            HandleScrollingWhenTriggeringScrollRight(game);
        }
    }
}
pub unsafe fn Samus_HandleScroll_Y(game: &mut Game) {
    if ram_word!(game, samus_prev_y_pos) == ram_word!(game, samus_y_pos) {
        HandleAutoscrolling_Y(game);
    } else {
        if ram_word!(game, samus_y_dir) == 1u16 {
            set_ram_word!(
                game.ideal_layer1_ypos =
                    (ram_word!(game, samus_y_pos)).wrapping_sub(ram_word!(game, down_scroller))
            );
        } else {
            set_ram_word!(
                game.ideal_layer1_ypos =
                    (ram_word!(game, samus_y_pos)).wrapping_sub(ram_word!(game, up_scroller))
            );
        }
        if ram_word!(game, ideal_layer1_ypos) != ram_word!(game, layer1_y_pos) {
            if ((ram_word!(game, ideal_layer1_ypos)) as i16)
                .wrapping_sub((ram_word!(game, layer1_y_pos)) as i16)
                < 0i16
            {
                AddToHiLo(
                    ram_ptr!(game, u16; 0x915),
                    ram_ptr!(game, u16; 0x913),
                    -((ram_word!(game, absolute_moved_last_frame_y) as i32) << 16i32
                        | ram_word!(game, absolute_moved_last_frame_y_fract) as i32)
                        as u32,
                );
                HandleScrollingWhenTriggeringScrollUp(game);
            } else {
                AddToHiLo(
                    ram_ptr!(game, u16; 0x915),
                    ram_ptr!(game, u16; 0x913),
                    (ram_word!(game, absolute_moved_last_frame_y) as u32) << 16i32
                        | ram_word!(game, absolute_moved_last_frame_y_fract) as u32,
                );
                HandleScrollingWhenTriggeringScrollDown(game);
            }
        }
    };
}
pub unsafe fn Samus_CalcDistanceMoved_X(game: &mut Game) {
    if ((ram_word!(game, samus_x_pos)) as i16)
        .wrapping_sub((ram_word!(game, samus_prev_x_pos)) as i16)
        >= 0i16
    {
        SetHiLo(
            ram_ptr!(game, u16; 0xda2),
            ram_ptr!(game, u16; 0xda4),
            ((ram_word!(game, samus_x_pos) as u32) << 16i32
                | ram_word!(game, samus_x_subpos) as u32)
                .wrapping_sub(
                    (ram_word!(game, samus_prev_x_pos) as u32) << 16i32
                        | ram_word!(game, samus_prev_x_subpos) as u32,
                )
                .wrapping_add(((1i32) << 16i32) as u32),
        );
    } else {
        SetHiLo(
            ram_ptr!(game, u16; 0xda2),
            ram_ptr!(game, u16; 0xda4),
            ((ram_word!(game, samus_prev_x_pos) as u32) << 16i32
                | ram_word!(game, samus_prev_x_subpos) as u32)
                .wrapping_sub(
                    (ram_word!(game, samus_x_pos) as u32) << 16i32
                        | ram_word!(game, samus_x_subpos) as u32,
                )
                .wrapping_add(((1i32) << 16i32) as u32),
        );
    };
}
pub unsafe fn Samus_CalcDistanceMoved_Y(game: &mut Game) {
    if ((ram_word!(game, samus_y_pos)) as i16)
        .wrapping_sub((ram_word!(game, samus_prev_y_pos)) as i16)
        >= 0i16
    {
        SetHiLo(
            ram_ptr!(game, u16; 0xda6),
            ram_ptr!(game, u16; 0xda8),
            ((ram_word!(game, samus_y_pos) as u32) << 16i32
                | ram_word!(game, samus_y_subpos) as u32)
                .wrapping_sub(
                    (ram_word!(game, samus_prev_y_pos) as u32) << 16i32
                        | ram_word!(game, samus_prev_y_subpos) as u32,
                )
                .wrapping_add(((1i32) << 16i32) as u32),
        );
    } else {
        SetHiLo(
            ram_ptr!(game, u16; 0xda6),
            ram_ptr!(game, u16; 0xda8),
            ((ram_word!(game, samus_prev_y_pos) as u32) << 16i32
                | ram_word!(game, samus_prev_y_subpos) as u32)
                .wrapping_sub(
                    (ram_word!(game, samus_y_pos) as u32) << 16i32
                        | ram_word!(game, samus_y_subpos) as u32,
                )
                .wrapping_add(((1i32) << 16i32) as u32),
        );
    };
}
static kSamus_HandleExtraRunspeedX_Tab0: [u16; 3] = [0, 0, 0];
static kSamus_HandleExtraRunspeedX_Tab1: [u16; 3] = [0x1000u16, 0x400u16, 0x400u16];
static kSamus_HandleExtraRunspeedX_Tab2: [u16; 3] = [7, 4, 4];
static kSamus_HandleExtraRunspeedX_Tab3: [u16; 3] = [0, 0, 0];
static kSamus_HandleExtraRunspeedX_Tab4: [u16; 3] = [2, 1, 0];
static kSamus_HandleExtraRunspeedX_Tab5: [u16; 3] = [0, 0, 0];
pub unsafe fn Samus_HandleExtraRunspeedX(game: &mut Game) {
    let mut current_block: u64;
    if (ram_word!(game, equipped_items) as i32 & 0x20i32) == 0i32 {
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
        if *ram_ptr!(game; 0xa1f) != 1u8
            || (ram_word!(game, button_config_run_b) as i32
                & ram_word!(game, joypad1_lastkeys) as i32)
                == 0i32
        {
            current_block = 3594490907146900115;
        } else {
            if (ram_word!(game, equipped_items) as i32 & 0x2000i32) != 0i32 {
                if ram_word!(game, samus_has_momentum_flag) == 0 {
                    set_ram_words! { game, 1u16;
                        samus_has_momentum_flag, special_samus_palette_timer,
                    }
                    set_ram_word!(game.special_samus_palette_frame = 0u16);
                    set_ram_word!(
                        game.speed_boost_counter = crate::rtl::rom_table_word!(game, 0x91b61f, 0)
                    );
                }
                if ((ram_word!(game, samus_x_extra_run_speed)) as i16)
                    .wrapping_sub((kSamus_HandleExtraRunspeedX_Tab2[0i32 as usize]) as i16)
                    >= 0i16
                    && ((ram_word!(game, samus_x_extra_run_subspeed)) as i16)
                        .wrapping_sub((kSamus_HandleExtraRunspeedX_Tab3[0i32 as usize]) as i16)
                        >= 0i16
                {
                    set_ram_word!(
                        game.samus_x_extra_run_speed =
                            kSamus_HandleExtraRunspeedX_Tab2[0i32 as usize]
                    );
                    set_ram_word!(
                        game.samus_x_extra_run_subspeed =
                            kSamus_HandleExtraRunspeedX_Tab3[0i32 as usize]
                    );
                    current_block = 7368539786447134940;
                } else {
                    current_block = 15768484401365413375;
                }
            } else {
                if ram_word!(game, samus_has_momentum_flag) == 0 {
                    set_ram_word!(game.samus_has_momentum_flag = 1u16);
                    set_ram_word!(game.speed_boost_counter = 0u16);
                }
                if ((ram_word!(game, samus_x_extra_run_speed)) as i16)
                    .wrapping_sub((kSamus_HandleExtraRunspeedX_Tab4[0i32 as usize]) as i16)
                    >= 0i16
                    && ((ram_word!(game, samus_x_extra_run_subspeed)) as i16)
                        .wrapping_sub((kSamus_HandleExtraRunspeedX_Tab5[0i32 as usize]) as i16)
                        >= 0i16
                {
                    set_ram_word!(
                        game.samus_x_extra_run_speed =
                            kSamus_HandleExtraRunspeedX_Tab4[0i32 as usize]
                    );
                    set_ram_word!(
                        game.samus_x_extra_run_subspeed =
                            kSamus_HandleExtraRunspeedX_Tab5[0i32 as usize]
                    );
                    current_block = 7368539786447134940;
                } else {
                    current_block = 15768484401365413375;
                }
            }
            if current_block == 7368539786447134940 {
            } else {
                AddToHiLo(
                    ram_ptr!(game, u16; 0xb42),
                    ram_ptr!(game, u16; 0xb44),
                    (kSamus_HandleExtraRunspeedX_Tab0[0i32 as usize] as u32) << 16i32
                        | kSamus_HandleExtraRunspeedX_Tab1[0i32 as usize] as u32,
                );
                current_block = 7368539786447134940;
            }
        }
    }
    if current_block == 3594490907146900115 && ram_word!(game, samus_has_momentum_flag) == 0 {
        set_ram_words! { game, 0u16; samus_x_extra_run_speed, samus_x_extra_run_subspeed }
    }
    if (ram_word!(game, speed_boost_counter) as i32 & 0xff00i32) == 1024i32 {
        set_ram_word!(game.samus_contact_damage_index = 1u16);
    }
}
pub unsafe fn Samus_MoveRight_NoColl(amt: i32, game: &mut Game) {
    AddToHiLo(
        ram_ptr!(game, u16; 0xaf6),
        ram_ptr!(game, u16; 0xaf8),
        amt as u32,
    );
    SetHiLo(
        ram_ptr!(game, u16; 0xdae),
        ram_ptr!(game, u16; 0xdb0),
        amt as u32,
    );
}
pub unsafe fn Samus_MoveLeft_NoColl(amt: i32, game: &mut Game) {
    AddToHiLo(
        ram_ptr!(game, u16; 0xaf6),
        ram_ptr!(game, u16; 0xaf8),
        -amt as u32,
    );
    SetHiLo(
        ram_ptr!(game, u16; 0xdaa),
        ram_ptr!(game, u16; 0xdac),
        -amt as u32,
    );
}
pub unsafe fn Samus_MoveDown_NoColl(amt: i32, game: &mut Game) {
    AddToHiLo(
        ram_ptr!(game, u16; 0xafa),
        ram_ptr!(game, u16; 0xafc),
        amt as u32,
    );
    SetHiLo(
        ram_ptr!(game, u16; 0xdb6),
        ram_ptr!(game, u16; 0xdb8),
        amt as u32,
    );
}
pub unsafe fn Samus_MoveUp_NoColl(amt: i32, game: &mut Game) {
    AddToHiLo(
        ram_ptr!(game, u16; 0xafa),
        ram_ptr!(game, u16; 0xafc),
        -amt as u32,
    );
    SetHiLo(
        ram_ptr!(game, u16; 0xdb2),
        ram_ptr!(game, u16; 0xdb4),
        -amt as u32,
    );
}
pub unsafe fn Samus_InitJump(game: &mut Game) {
    let use_normal_speed: bool;
    static kSamus_InitJump_Speed_Y: [u16; 3] = [4, 1, 2];
    static kSamus_InitJump_Subspeed_Y: [u16; 3] = [0xe000u16, 0xc000u16, 0xc000u16];
    static kSamus_InitJump_Tab2: [u16; 3] = [6, 2, 3];
    static kSamus_InitJump_Tab3: [u16; 3] = [0, 0x8000, 0x8000];
    let mut v0: u16 = 0;
    if (ram_word!(game, equipped_items) as i32 & 0x20i32) != 0i32 {
        use_normal_speed = true;
    } else {
        let mut r18: u16 = Samus_GetBottom_R18(game);
        match liquid_at_height(game, r18) {
            LiquidAtHeight::Water => {
                v0 = 2u16;
                use_normal_speed = false;
            }
            LiquidAtHeight::LavaAcid => {
                v0 = 4u16;
                use_normal_speed = false;
            }
            LiquidAtHeight::None => use_normal_speed = true,
        }
    }
    if use_normal_speed {
        v0 = 0u16;
    }
    let v1: i32 = v0 as i32 >> 1i32;
    if (ram_word!(game, equipped_items) as i32 & 0x100i32) != 0i32 {
        set_ram_word!(game.samus_y_subspeed = kSamus_InitJump_Tab3[v1 as usize]);
        set_ram_word!(game.samus_y_speed = kSamus_InitJump_Tab2[v1 as usize]);
    } else {
        set_ram_word!(game.samus_y_subspeed = kSamus_InitJump_Subspeed_Y[v1 as usize]);
        set_ram_word!(game.samus_y_speed = kSamus_InitJump_Speed_Y[v1 as usize]);
    }
    if (ram_word!(game, equipped_items) as i32 & 0x2000i32) != 0i32 {
        update_ram_word!(
            game.samus_y_subspeed,
            wrapping_add(ram_word!(game, samus_x_extra_run_subspeed))
        );
        update_ram_word!(game.samus_y_speed, |fresh126| (fresh126 as i32
            + (ram_word!(game, samus_x_extra_run_speed) as i32 >> 1i32))
            as u16);
    }
    set_ram_words! { game, 0u16; grapple_walljump_timer, reached_ceres_elevator_fade_timer }
    set_ram_word!(game.samus_y_dir = 1u16);
}
pub unsafe fn Samus_InitWallJump(game: &mut Game) {
    let mut r18: u16 = 0;
    let use_normal_speed: bool;
    static kSamus_InitWallJump_0: [u16; 3] = [4, 0, 2];
    static kSamus_InitWallJump_1: [u16; 3] = [0xa000u16, 0x4000u16, 0xa000u16];
    static kSamus_InitWallJump_2: [u16; 3] = [5, 0, 3];
    static kSamus_InitWallJump_3: [u16; 3] = [0x8000u16, 0x8000u16, 0x8000u16];
    let mut v0: u16 = 0;
    if (ram_word!(game, equipped_items) as i32 & 0x20i32) != 0i32 {
        use_normal_speed = true;
    } else {
        r18 = Samus_GetBottom_R18(game);
        match liquid_at_height(game, r18) {
            LiquidAtHeight::Water => {
                v0 = 2u16;
                use_normal_speed = false;
            }
            LiquidAtHeight::LavaAcid => {
                v0 = 4u16;
                use_normal_speed = false;
            }
            LiquidAtHeight::None => use_normal_speed = true,
        }
    }
    if use_normal_speed {
        v0 = 0u16;
    }
    let v1: i32 = v0 as i32 >> 1i32;
    if (ram_word!(game, equipped_items) as i32 & 0x100i32) != 0i32 {
        set_ram_word!(game.samus_y_subspeed = kSamus_InitWallJump_3[v1 as usize]);
        set_ram_word!(game.samus_y_speed = kSamus_InitWallJump_2[v1 as usize]);
    } else {
        set_ram_word!(game.samus_y_subspeed = kSamus_InitWallJump_1[v1 as usize]);
        set_ram_word!(game.samus_y_speed = kSamus_InitWallJump_0[v1 as usize]);
    }
    if (ram_word!(game, equipped_items) as i32 & 0x2000i32) != 0i32 {
        r18 = (ram_word!(game, samus_x_extra_run_speed) as i32 >> 1i32) as u16;
        update_ram_word!(
            game.samus_y_subspeed,
            wrapping_add(ram_word!(game, samus_x_extra_run_subspeed))
        );
        update_ram_word!(game.samus_y_speed, |fresh152| (fresh152 as i32
            + (ram_word!(game, samus_x_extra_run_speed) as i32 >> 1i32))
            as u16);
    }
    set_ram_words! { game, 0u16; grapple_walljump_timer, reached_ceres_elevator_fade_timer }
    set_ram_word!(game.samus_y_dir = 1u16);
}
static kSamus_SetSpeedForKnockback_Y_Speed: [u16; 3] = [5, 2, 2];
static kSamus_SetSpeedForKnockback_Y_Subspeed: [u16; 3] = [0u16, 0u16, 0u16];
pub unsafe fn Samus_SetSpeedForKnockback_Y(game: &mut Game) {
    let use_normal_speed: bool;
    let mut v0: u16 = 0;
    if (ram_word!(game, equipped_items) as i32 & 0x20i32) == 0i32 {
        let r18: u16 = Samus_GetBottom_R18(game);
        match liquid_at_height(game, r18) {
            LiquidAtHeight::Water => {
                v0 = 2u16;
                use_normal_speed = false;
            }
            LiquidAtHeight::LavaAcid => {
                v0 = 4u16;
                use_normal_speed = false;
            }
            LiquidAtHeight::None => use_normal_speed = true,
        }
    } else {
        use_normal_speed = true;
    }
    if use_normal_speed {
        v0 = 0u16;
    }
    let v1: i32 = v0 as i32 >> 1i32;
    set_ram_word!(game.samus_y_subspeed = kSamus_SetSpeedForKnockback_Y_Subspeed[v1 as usize]);
    set_ram_word!(game.samus_y_speed = kSamus_SetSpeedForKnockback_Y_Speed[v1 as usize]);
    set_ram_words! { game, 0u16; grapple_walljump_timer, reached_ceres_elevator_fade_timer }
    set_ram_word!(game.samus_y_dir = 1u16);
}
pub unsafe fn Samus_InitBombJump(game: &mut Game) {
    let use_normal_speed: bool;
    static kSamus_InitBombJump_Speed: [u16; 3] = [2, 0, 0];
    static kSamus_InitBombJump_Subspeed: [u16; 3] = [0xc000u16, 0x1000u16, 0x1000u16];
    let mut v0: u16 = 0;
    if (ram_word!(game, equipped_items) as i32 & 0x20i32) == 0i32 {
        let r18: u16 = Samus_GetBottom_R18(game);
        match liquid_at_height(game, r18) {
            LiquidAtHeight::Water => {
                v0 = 2u16;
                use_normal_speed = false;
            }
            LiquidAtHeight::LavaAcid => {
                v0 = 4u16;
                use_normal_speed = false;
            }
            LiquidAtHeight::None => use_normal_speed = true,
        }
    } else {
        use_normal_speed = true;
    }
    if use_normal_speed {
        v0 = 0u16;
    }
    let v1: i32 = v0 as i32 >> 1i32;
    set_ram_word!(game.samus_y_subspeed = kSamus_InitBombJump_Subspeed[v1 as usize]);
    set_ram_word!(game.samus_y_speed = kSamus_InitBombJump_Speed[v1 as usize]);
    set_ram_words! { game, 0u16; grapple_walljump_timer, reached_ceres_elevator_fade_timer }
    set_ram_word!(game.samus_y_dir = 1u16);
}
unsafe fn IsGreaterThanQuirked(vhi: u16, vlo: u16, cmphi: u16, cmplo: u16) -> bool {
    if ((vhi) as i16).wrapping_sub((cmphi) as i16) >= 0i16
        && (vhi != cmphi || ((vlo) as i16).wrapping_sub((cmplo) as i16) >= 0i16 && vlo != cmplo)
    {
        return true_0 != 0;
    }
    false_0 != 0
}
pub unsafe fn Samus_CalcBaseSpeed_X(k: u16, game: &mut Game) -> i32 {
    let sste: *mut SamusSpeedTableEntry = get_SamusSpeedTableEntry(k, game);
    if ram_word!(game, samus_x_accel_mode) != 0 {
        let delta: i32 = (if *ram_ptr!(game; 0xb4c) != 0u8 {
            ((Mult8x8(*ram_ptr!(game; 0xb4c), (*sste).decel as u8) as i32 >> 8i32) as u32) << 16i32
                | Mult8x8(
                    *ram_ptr!(game; 0xb4c),
                    *(&raw mut (*sste).decel_sub as *mut u8).offset(1isize),
                ) as u32
        } else {
            ((*sste).decel as u32) << 16i32 | (*sste).decel_sub as u32
        }) as i32;
        AddToHiLo(
            ram_ptr!(game, u16; 0xb46),
            ram_ptr!(game, u16; 0xb48),
            -delta as u32,
        );
        if (ram_word!(game, samus_x_base_speed) as i16) < 0i16 {
            set_ram_words! { game, 0u16;
                samus_x_base_speed, samus_x_base_subspeed, samus_x_accel_mode,
            }
        }
    } else {
        AddToHiLo(
            ram_ptr!(game, u16; 0xb46),
            ram_ptr!(game, u16; 0xb48),
            ((*sste).accel as u32) << 16i32 | (*sste).accel_sub as u32,
        );
        if IsGreaterThanQuirked(
            ram_word!(game, samus_x_base_speed),
            ram_word!(game, samus_x_base_subspeed),
            (*sste).max_speed,
            (*sste).max_speed_sub,
        ) {
            set_ram_word!(game.samus_x_base_speed = (*sste).max_speed);
            set_ram_word!(game.samus_x_base_subspeed = (*sste).max_speed_sub);
        }
    }
    ((ram_word!(game, samus_x_base_speed) as u32) << 16i32
        | ram_word!(game, samus_x_base_subspeed) as u32) as i32
}
unsafe fn Samus_CalcBaseSpeed_NoDecel_X(k: u16, game: &mut Game) -> Pair_Bool_Amt {
    let sste: *mut SamusSpeedTableEntry = get_SamusSpeedTableEntry(k, game);
    let mut rv: bool = false_0 != 0;
    if (ram_word!(game, samus_x_accel_mode) as i32 & 1i32) != 0i32 {
        let delta: i32 = (if *ram_ptr!(game; 0xb4c) != 0u8 {
            ((Mult8x8(*ram_ptr!(game; 0xb4c), (*sste).decel as u8) as i32 >> 8i32) as u32) << 16i32
                | Mult8x8(
                    *ram_ptr!(game; 0xb4c),
                    *(&raw mut (*sste).decel_sub as *mut u8).offset(1isize),
                ) as u32
        } else {
            ((*sste).decel as u32) << 16i32 | (*sste).decel_sub as u32
        }) as i32;
        AddToHiLo(
            ram_ptr!(game, u16; 0xb46),
            ram_ptr!(game, u16; 0xb48),
            -delta as u32,
        );
        if (ram_word!(game, samus_x_base_speed) as i16) < 0i16 {
            set_ram_words! { game, 0u16;
                samus_x_base_speed, samus_x_base_subspeed, samus_x_accel_mode,
            }
        }
    } else {
        AddToHiLo(
            ram_ptr!(game, u16; 0xb46),
            ram_ptr!(game, u16; 0xb48),
            ((*sste).accel as u32) << 16i32 | (*sste).accel_sub as u32,
        );
        if IsGreaterThanQuirked(
            ram_word!(game, samus_x_base_speed),
            ram_word!(game, samus_x_base_subspeed),
            (*sste).max_speed,
            (*sste).max_speed_sub,
        ) {
            set_ram_word!(game.samus_x_base_speed = (*sste).max_speed);
            set_ram_word!(game.samus_x_base_subspeed = (*sste).max_speed_sub);
            rv = true_0 != 0;
        }
    }
    Pair_Bool_Amt {
        flag: rv,
        amt: ((ram_word!(game, samus_x_base_speed) as u32) << 16i32
            | ram_word!(game, samus_x_base_subspeed) as u32) as i32,
    }
}
pub unsafe fn Samus_DetermineSpeedTableEntryPtr_X(game: &mut Game) -> u16 {
    if (ram_word!(game, equipped_items) as i32 & 0x20i32) == 0i32 {
        let r18: u16 = Samus_GetBottom_R18(game);
        match liquid_at_height(game, r18) {
            LiquidAtHeight::Water => {
                set_ram_word!(
                    game.samus_x_speed_table_pointer = addr_kSamusSpeedTable_Water_X as u16
                );
            }
            LiquidAtHeight::LavaAcid => {
                set_ram_word!(
                    game.samus_x_speed_table_pointer = addr_kSamusSpeedTable_LavaAcid_X as u16
                );
            }
            LiquidAtHeight::None => {}
        }
    }
    (ram_word!(game, samus_x_speed_table_pointer) as i32 + 12i32 * *ram_ptr!(game; 0xa1f) as i32)
        as u16
}
pub unsafe fn Samus_DetermineGrappleSwingSpeed_X(game: &mut Game) -> u16 {
    if (ram_word!(game, equipped_items) as i32 & 0x20i32) != 0i32 {
        return addr_stru_909F31 as u16;
    }
    let bottom = Samus_GetBottom_R18(game);
    match liquid_at_height(game, bottom) {
        LiquidAtHeight::None => addr_stru_909F31 as u16,
        LiquidAtHeight::Water => addr_stru_909F3D as u16,
        LiquidAtHeight::LavaAcid => addr_kSamusSpeedTable_Normal_X as u16,
    }
}
pub unsafe fn Samus_DetermineAccel_Y(game: &mut Game) {
    static g_word_909EA1: u16 = 0x1c00u16;
    static g_word_909EA3: u16 = 0x800u16;
    static g_word_909EA5: u16 = 0x900u16;
    static g_word_909EA7: u16 = 0u16;
    static g_word_909EA9: u16 = 0u16;
    static g_word_909EAB: u16 = 0u16;
    if ((ram_word!(game, equipped_items) as i32 & 0x20i32) == 0i32) {
        let mut r18: u16 = Samus_GetBottom_R18(game);
        match liquid_at_height(game, r18) {
            LiquidAtHeight::Water => {
                set_ram_word!(game.samus_y_subaccel = g_word_909EA3);
                set_ram_word!(game.samus_y_accel = g_word_909EA9);
                return;
            }
            LiquidAtHeight::LavaAcid => {
                set_ram_word!(game.samus_y_subaccel = g_word_909EA5);
                set_ram_word!(game.samus_y_accel = g_word_909EAB);
                return;
            }
            LiquidAtHeight::None => {}
        }
    }
    set_ram_word!(game.samus_y_subaccel = g_word_909EA1);
    set_ram_word!(game.samus_y_accel = g_word_909EA7);
}
pub unsafe fn Samus_GrappleWallJumpCheck(amt: i32, game: &mut Game) -> u8 {
    game.set_enemy_index_to_shake(0xffff);
    let direction = match game.samus_pose_x_dir() {
        4 => 0,
        8 => 1,
        _ => return 0,
    };
    game.set_samus_collision_direction(direction);
    let result = Samus_CheckSolidEnemyColl(amt, game);
    if result.collision == 0 {
        WallJumpBlockCollDetect(result.amt, game);
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
pub unsafe fn Samus_WallJumpCheck(amt: i32, game: &mut Game) -> u8 {
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
            distance = WallJumpBlockCollDetect(distance, game);
        } else {
            distance = distance.wrapping_neg();
            WallJumpBlockCollDetect(distance, game);
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
            26u16
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
pub unsafe fn Samus_Movement_07_Unused(game: &mut Game) {
    set_ram_word!(game.input_to_pose_calc = 0u16);
}
static kSamusMovementHandlers: [Option<HandlerFunc>; 28] = callback_table! { HandlerFunc;
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
pub unsafe fn Samus_MovementHandler_Normal(game: &mut Game) {
    if ram_word!(game, time_is_frozen_flag) == 0 {
        kSamusMovementHandlers[*ram_ptr!(game; 0xa1f) as usize].expect("non-null function pointer")(
            game,
        );
        Samus_UpdateSpeedEchoPos(game);
    }
}
pub unsafe fn Samus_Movement_00_Standing(game: &mut Game) {
    if ram_word!(game, samus_pose) != 0u16
        && ram_word!(game, samus_pose) as i32 != kPose_9B_FaceF_VariaGravitySuit as i32
    {
        if (ram_word!(game, samus_pose) as i32 == kPose_01_FaceR_Normal as i32
            || ram_word!(game, samus_pose) as i32 == kPose_02_FaceL_Normal as i32)
            && (ram_word!(game, button_config_shoot_x) as i32
                & ram_word!(game, joypad1_lastkeys) as i32)
                != 0i32
        {
            set_ram_word!(game.samus_anim_frame_timer = 16u16);
            set_ram_word!(game.samus_anim_frame = 0u16);
        }
        Samus_Move_NoBaseSpeed_X(game);
        Samus_Move_NoSpeedCalc_Y(game);
        Samus_CancelSpeedBoost(game);
        set_ram_words! { game, 0u16;
            samus_x_extra_run_speed, samus_x_extra_run_subspeed, samus_x_base_speed,
            samus_x_base_subspeed, samus_x_accel_mode,
        }
    } else {
        if ram_word!(game, elevator_status) != 0 {
            set_ram_word!(game.samus_collision_direction = 2u16);
            Samus_MoveDown_NoSolidColl((1i16 as i32) << 16i32, game);
        }
        set_ram_word!(game.input_to_pose_calc = 0u16);
    };
}
pub unsafe fn Samus_Movement_01_Running(game: &mut Game) {
    static kSamusFootstepFrame: [u8; 10] = [0, 0, 1, 0, 0, 0, 0, 1, 0, 0];
    Samus_HandleMovement_X(game);
    Samus_Move_NoSpeedCalc_Y(game);
    if ram_word!(game, samus_anim_frame_timer) == 1u16
        && kSamusFootstepFrame[ram_word!(game, samus_anim_frame) as usize] != 0
    {
        Samus_FootstepGraphics(game);
        if ram_word!(game, cinematic_function) == 0
            && ram_word!(game, boss_id) == 0
            && ram_word!(game, samus_shine_timer) == 0
            && (ram_word!(game, speed_boost_counter) as i32 & 0x400i32) == 0i32
        {
            QueueSfx3_Max6(6u16, game);
        }
    }
}
pub unsafe fn Samus_Movement_02_NormalJumping(game: &mut Game) {
    Samus_JumpingMovement(game);
}
pub unsafe fn Samus_Movement_03_SpinJumping(game: &mut Game) {
    let skip_jump_restart: bool;
    static kSamusPhys_JumpMinYVelAir: u16 = 0x280u16;
    static kSamusPhys_JumpMaxYVelAir: u16 = 0x500u16;
    static kSamusPhys_JumpMinYVelWater: u16 = 0x80u16;
    static kSamusPhys_JumpMaxYVelWater: u16 = 0x500u16;
    let mut r18: u16 = 0u16;
    if (ram_word!(game, samus_suit_palette_index) as i32 & 4i32) == 0i32 {
        let r20: u16 = Samus_GetTop_R20(game);
        if liquid_at_height(game, r20) != LiquidAtHeight::None {
            r18 = 1u16;
        }
    }
    if r18 == 0 {
        if (ram_word!(game, equipped_items) as i32 & 0x200i32) != 0i32
            && (ram_word!(game, samus_y_dir) == 2u16)
        {
            skip_jump_restart = if ram_word!(game, liquid_physics_type) != 0 {
                (((*((ram_ptr!(game, u16; 0xb2c) as *mut u8).offset(1) as *mut u16)) as i16)
                    .wrapping_sub((kSamusPhys_JumpMinYVelWater) as i16)
                    < 0i16
                    || ((*((ram_ptr!(game, u16; 0xb2c) as *mut u8).offset(1) as *mut u16)) as i16)
                        .wrapping_sub((kSamusPhys_JumpMaxYVelWater) as i16)
                        >= 0i16)
            } else {
                (((*((ram_ptr!(game, u16; 0xb2c) as *mut u8).offset(1) as *mut u16)) as i16)
                    .wrapping_sub((kSamusPhys_JumpMinYVelAir) as i16)
                    < 0i16
                    || ((*((ram_ptr!(game, u16; 0xb2c) as *mut u8).offset(1) as *mut u16)) as i16)
                        .wrapping_sub((kSamusPhys_JumpMaxYVelAir) as i16)
                        >= 0i16)
            };
            if skip_jump_restart {
            } else {
                set_ram_word!(
                    game.UNUSED_word_7E0DFA =
                        (ram_word!(game, UNUSED_word_7E0DFA) as i32 & 0xff00i32 | 1i32) as u16
                );
                if (ram_word!(game, button_config_jump_a) as i32
                    & ram_word!(game, joypad1_newkeys) as i32)
                    != 0i32
                {
                    Samus_InitJump(game);
                }
            }
        }
        if ram_word!(game, samus_pose) as i32 == kPose_81_FaceR_Screwattack as i32
            || ram_word!(game, samus_pose) as i32 == kPose_82_FaceL_Screwattack as i32
        {
            set_ram_word!(game.samus_contact_damage_index = 3u16);
        } else if ram_word!(game, flare_counter) as i32 - 60i32 & 0x8000i32 == 0 {
            set_ram_word!(game.samus_contact_damage_index = 4u16);
        }
    } else if ram_word!(game, samus_anim_frame_timer) == 1u16
        && *(game.g_rom.offset(crate::rtl::lorom_offset(0x90a514)) as *mut u8).offset(ram_word!(
            game,
            samus_anim_frame
        )
            as isize)
            != 0u8
    {
        QueueSfx1_Max6(0x2fu16, game);
    }
    Samus_SpinJumpMovement(game);
}
pub unsafe fn Samus_Movement_04_MorphBallOnGround(game: &mut Game) {
    if ram_word!(game, samus_x_accel_mode) == 0 {
        let use_normal_movement: bool = if *ram_ptr!(game; 0xa1e) == 4u8 {
            !(ram_word!(game, samus_pose) as i32 == kPose_41_FaceL_Morphball_Ground as i32)
        } else {
            !(ram_word!(game, samus_pose) as i32 == kPose_1D_FaceR_Morphball_Ground as i32)
        };
        if use_normal_movement {
        } else {
            Samus_Move_NoBaseSpeed_X(game);
            if Samus_CheckAndMoveY(game) as i32 & 1i32 == 0 {
                Samus_Move_NoSpeedCalc_Y(game);
                Samus_CancelSpeedBoost(game);
                set_ram_words! { game, 0u16;
                    samus_x_extra_run_speed, samus_x_extra_run_subspeed, samus_x_base_speed,
                    samus_x_base_subspeed, samus_x_accel_mode,
                }
            }
            return;
        }
    }
    Samus_HandleMovement_X(game);
    if Samus_CheckAndMoveY(game) as i32 & 1i32 == 0 {
        Samus_Move_NoSpeedCalc_Y(game);
    }
}
pub unsafe fn Samus_Movement_05_Crouching(game: &mut Game) {
    Samus_Move_NoBaseSpeed_X(game);
    Samus_Move_NoSpeedCalc_Y(game);
    set_ram_words! { game, 0u16;
        samus_x_extra_run_speed, samus_x_extra_run_subspeed, samus_x_base_speed,
        samus_x_base_subspeed, samus_x_accel_mode,
    }
}
pub unsafe fn Samus_Movement_06_Falling(game: &mut Game) {
    Samus_FallingMovement(game);
    if (ram_word!(game, samus_pose) as i32 == kPose_29_FaceR_Fall as i32
        || ram_word!(game, samus_pose) as i32 == kPose_2A_FaceL_Fall as i32
        || ram_word!(game, samus_pose) as i32 == kPose_67_FaceR_Fall_Gun as i32
        || ram_word!(game, samus_pose) as i32 == kPose_68_FaceL_Fall_Gun as i32)
        && ram_word!(game, samus_y_speed) as i32 - 5i32 & 0x8000i32 == 0
        && ram_word!(game, samus_anim_frame) as i32 - 5i32 & 0x8000i32 != 0
    {
        set_ram_word!(game.samus_anim_frame_timer = 8u16);
        set_ram_word!(game.samus_anim_frame = 5u16);
    }
}
pub unsafe fn Samus_Movement_08_MorphBallFalling(game: &mut Game) {
    if ram_word!(game, joypad1_lastkeys) as i32 & (kButton_Left as i32 | kButton_Right as i32)
        == 0i32
        && ram_word!(game, samus_x_accel_mode) == 0
    {
        Samus_CancelSpeedBoost(game);
        set_ram_words! { game, 0u16;
            samus_x_extra_run_speed, samus_x_extra_run_subspeed, samus_x_base_speed,
            samus_x_base_subspeed, samus_x_accel_mode,
        }
    }
    if ram_word!(game, used_for_ball_bounce_on_landing) != 0 {
        Samus_MorphedBouncingMovement(game);
    } else {
        Samus_MorphedFallingMovement(game);
    };
}
pub fn Samus_Movement_09_Unused(game: &mut Game) {}
pub unsafe fn Samus_Movement_0A_KnockbackOrCrystalFlashEnding(game: &mut Game) {
    set_ram_word!(game.input_to_pose_calc = 0u16);
    Samus_Move_NoSpeedCalc_Y(game);
}
pub unsafe fn Samus_Movement_0B_Unused(game: &mut Game) {
    set_ram_word!(game.input_to_pose_calc = 0u16);
}
pub unsafe fn Samus_Movement_0C_Unused(game: &mut Game) {
    set_ram_word!(game.input_to_pose_calc = 0u16);
}
pub fn Samus_Movement_0D_Unused(game: &mut Game) {}
pub fn nullsub_13(game: &mut Game) {}
static kSamusCrouchingEtcFuncs: [Option<HandlerFunc>; 12] = callback_table! { unsafe HandlerFunc;
    nullsub_13, nullsub_13, nullsub_13, nullsub_13, SamusCrouchingEtcFunc,
    SamusCrouchingEtcFunc, nullsub_13, nullsub_13, nullsub_13, nullsub_13,
    SamusCrouchingEtcFunc, SamusCrouchingEtcFunc,
};
pub unsafe fn Samus_Movement_0F_CrouchingEtcTransition(game: &mut Game) {
    let apply_movement: bool =
        !(ram_word!(game, samus_pose) as i32 - kPose_F1_FaceR_CrouchTrans_AimU as i32 & 0x8000i32
            != 0)
            || (!(ram_word!(game, samus_pose) as i32 - kPose_DB as i32 & 0x8000i32 == 0)
                && ({
                    kSamusCrouchingEtcFuncs[(ram_word!(game, samus_pose) as i32 - 53i32) as usize]
                        .expect("non-null function pointer")(game);
                    true
                }));
    if apply_movement {
        Samus_Move_NoBaseSpeed_X(game);
        if Samus_MoveY_Simple_(game) as i32 & 1i32 == 0 {
            Samus_Move_NoSpeedCalc_Y(game);
        }
    }
    if ram_word!(game, input_to_pose_calc) == 1025u16 {
        set_ram_words! { game, 0u16;
            samus_y_subspeed, samus_y_speed, samus_y_dir, used_for_ball_bounce_on_landing,
        }
    }
    set_ram_word!(game.input_to_pose_calc = 0u16);
}
pub unsafe fn SamusCrouchingEtcFunc(game: &mut Game) {
    set_ram_word!(game.enable_horiz_slope_coll = 3u16);
    set_ram_word!(game.UNUSEDword_7E0AA4 = 0u16);
}
pub unsafe fn Samus_Movement_0E_TurningAroundOnGround(game: &mut Game) {
    Samus_HandleMovement_X(game);
    Samus_Move_NoSpeedCalc_Y(game);
    Samus_CancelSpeedBoost(game);
    set_ram_words! { game, 0u16;
        samus_x_extra_run_speed, samus_x_extra_run_subspeed, input_to_pose_calc,
    }
}
pub unsafe fn Samus_Movement_10_Moonwalking(game: &mut Game) {
    Samus_HandleMovement_X(game);
    Samus_Move_NoSpeedCalc_Y(game);
}
pub unsafe fn Samus_Movement_11_SpringBallOnGround(game: &mut Game) {
    if ram_word!(game, samus_x_accel_mode) == 0 {
        let use_normal_movement: bool = if *ram_ptr!(game; 0xa1e) == 4u8 {
            !(ram_word!(game, samus_pose) as i32 == kPose_7A_FaceL_Springball_Ground as i32)
        } else {
            !(ram_word!(game, samus_pose) as i32 == kPose_79_FaceR_Springball_Ground as i32)
        };
        if use_normal_movement {
        } else {
            Samus_Move_NoBaseSpeed_X(game);
            if Samus_CheckAndMoveY(game) as i32 & 1i32 == 0 {
                Samus_Move_NoSpeedCalc_Y(game);
                Samus_CancelSpeedBoost(game);
                set_ram_words! { game, 0u16;
                    samus_x_extra_run_speed, samus_x_extra_run_subspeed, samus_x_base_speed,
                    samus_x_base_subspeed, samus_x_accel_mode,
                }
            }
            return;
        }
    }
    Samus_HandleMovement_X(game);
    if Samus_CheckAndMoveY(game) as i32 & 1i32 == 0 {
        Samus_Move_NoSpeedCalc_Y(game);
    }
}
pub unsafe fn Samus_Movement_12_SpringBallInAir(game: &mut Game) {
    if ram_word!(game, used_for_ball_bounce_on_landing) != 0 {
        Samus_MorphedBouncingMovement(game);
    } else {
        Samus_JumpingMovement(game);
    };
}
pub unsafe fn Samus_Movement_13_SpringBallFalling(game: &mut Game) {
    if ram_word!(game, joypad1_lastkeys) as i32 & (kButton_Left as i32 | kButton_Right as i32)
        == 0i32
        && ram_word!(game, samus_x_accel_mode) == 0
    {
        Samus_CancelSpeedBoost(game);
        set_ram_words! { game, 0u16;
            samus_x_extra_run_speed, samus_x_extra_run_subspeed, samus_x_base_speed,
            samus_x_base_subspeed, samus_x_accel_mode,
        }
    }
    if ram_word!(game, used_for_ball_bounce_on_landing) != 0 {
        Samus_MorphedBouncingMovement(game);
    } else {
        Samus_MorphedFallingMovement(game);
    };
}
pub unsafe fn Samus_Movement_14_WallJumping(game: &mut Game) {
    if ram_word!(game, samus_anim_frame) as i32 - 23i32 & 0x8000i32 != 0 {
        if ram_word!(game, samus_anim_frame) as i32 - 3i32 & 0x8000i32 == 0
            && ram_word!(game, flare_counter) as i32 - 60i32 & 0x8000i32 == 0
        {
            set_ram_word!(game.samus_contact_damage_index = 4u16);
        }
    } else {
        set_ram_word!(game.samus_contact_damage_index = 3u16);
    }
    Samus_JumpingMovement(game);
}
pub unsafe fn Samus_Movement_15_RanIntoWall(game: &mut Game) {
    Samus_Move_NoBaseSpeed_X(game);
    Samus_Move_NoSpeedCalc_Y(game);
    Samus_CancelSpeedBoost(game);
    set_ram_words! { game, 0u16;
        samus_x_extra_run_speed, samus_x_extra_run_subspeed, samus_x_base_speed,
        samus_x_base_subspeed, samus_x_accel_mode,
    }
}
pub unsafe fn Samus_Movement_16_Grappling(game: &mut Game) {
    if ram_word!(game, input_to_pose_calc) != 5u16 {
        set_ram_word!(game.input_to_pose_calc = 0u16);
    }
}
pub unsafe fn Samus_Movement_17_TurningAroundJumping(game: &mut Game) {
    Samus_HandleMovement_X(game);
    if Samus_CheckAndMoveY(game) as i32 & 1i32 == 0 {
        Samus_Move_NoSpeedCalc_Y(game);
    }
    Samus_CancelSpeedBoost(game);
    set_ram_words! { game, 0u16;
        samus_x_extra_run_speed, samus_x_extra_run_subspeed, input_to_pose_calc,
    }
}
pub unsafe fn Samus_Movement_18_TurningAroundFalling(game: &mut Game) {
    Samus_HandleMovement_X(game);
    if Samus_CheckAndMoveY(game) as i32 & 1i32 == 0 {
        Samus_Move_NoSpeedCalc_Y(game);
    }
    Samus_CancelSpeedBoost(game);
    set_ram_words! { game, 0u16;
        samus_x_extra_run_speed, samus_x_extra_run_subspeed, input_to_pose_calc,
    }
}
pub unsafe fn Samus_Movement_19_DamageBoost(game: &mut Game) {
    Samus_JumpingMovement(game);
}
pub unsafe fn Samus_Movement_1A_GrabbedByDraygon(game: &mut Game) {
    set_ram_word!(game.input_to_pose_calc = 0u16);
}
pub unsafe fn Samus_Movement_1B_ShinesparkEtc(game: &mut Game) {
    set_ram_word!(game.input_to_pose_calc = 0u16);
}
static kShr0x80: [u8; 8] = [0x80, 0x40, 0x20, 0x10, 8, 4, 2, 1];
static kShr0xFc00: [u16; 8] = [0xfc00, 0x7e00, 0x3f00, 0x1f80, 0xfc0, 0x7e0, 0x3f0, 0x1f8];
unsafe fn MarkMapTileAsExplored(r18: u16, r24: u16, game: &mut Game) {
    let v0: u8 = (((r18) & (0xff00u16)) as i32 >> 8i32) as u8;
    let R34: u16 = (ram_word!(game, room_x_coordinate_on_map) as i32 + v0 as i32 & 0x20i32) as u16;
    let r20: u16 =
        ((ram_word!(game, room_x_coordinate_on_map) as i32 + v0 as i32 & 0x1fi32) >> 3i32) as u16;
    let R22: u16 = (ram_word!(game, room_y_coordinate_on_map) as i32
        + ((r24 as i32 & 0xff00i32) >> 8i32)
        + 1i32) as u16;
    let ref mut fresh13 =
        *ram_ptr!(game; 0x7f7, (r20 as i32 + 4i32 * (R34 as i32 + R22 as i32)) as u16 as isize);
    *fresh13 = (*fresh13)
        | (kShr0x80
            [(ram_word!(game, room_x_coordinate_on_map) as i32 + v0 as i32 & 7i32) as usize]);
}
pub unsafe fn DisableMinimapAndMarkBossRoomAsExplored(game: &mut Game) {
    set_ram_word!(game.debug_disable_minimap = 1u16);
    let mut v0: u16 = 0u16;
    loop {
        let v1: i32 = v0 as i32 >> 1i32;
        set_ram_words! { game, 11295u16;
            hud_tilemap[(v1 + 26i32)], hud_tilemap[(v1 + 58i32)], hud_tilemap[(v1 + 90i32)],
        }
        v0 = (v0).wrapping_add(2u16);
        if (((v0) as i16).wrapping_sub(10i16) >= 0i16) {
            break;
        }
    }
    let mut v2: u16 = 5u16;
    while ram_word!(game, boss_id)
        != (*(game.g_rom.offset(crate::rtl::lorom_offset(0x90a83a))
            as *mut DisableMinimapAndMarkBossRoomAsExploredEnt)
            .offset(v2 as isize))
        .boss_id_
    {
        v2 = v2.wrapping_sub(1);
        if (v2 as i32 & 0x8000i32) != 0i32 {
            return;
        }
    }
    let mut i: i32 = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x90a83a))
        as *mut DisableMinimapAndMarkBossRoomAsExploredEnt)
        .offset(v2 as isize))
    .ptrs as i32;
    loop {
        let v4: *const u16 = RomPtr_90(i as u16, game) as *const u16;
        if (*v4 as i32 & 0x8000i32) != 0i32 {
            break;
        }
        MarkMapTileAsExplored(*v4.offset(0), *v4.offset(1), game);
        i += 4i32;
    }
}
pub fn InitializeMiniMapBroken() {}
pub unsafe fn UpdateMinimap(game: &mut Game) {
    if ram_word!(game, debug_disable_minimap) != 0u16
        || ram_word!(game, samus_x_pos) as i32 >> 4i32
            >= ram_word!(game, room_width_in_blocks) as i32
        || ram_word!(game, samus_y_pos) as i32 >> 4i32
            >= ram_word!(game, room_height_in_blocks) as i32
    {
        return;
    }
    let mut r46: u16 = 0u16;
    let v0: u8 = (((ram_word!(game, samus_x_pos)) & (0xff00u16)) as i32 >> 8i32) as u8;
    let r34: u16 = (ram_word!(game, room_x_coordinate_on_map) as i32 + v0 as i32 & 0x20i32) as u16;
    let r18: u16 = (ram_word!(game, room_x_coordinate_on_map) as i32 + v0 as i32 & 0x1fi32) as u16;
    let v1: u16 = (ram_word!(game, room_x_coordinate_on_map) as i32 + v0 as i32 & 7i32) as u16;
    let r20: u16 =
        ((ram_word!(game, room_x_coordinate_on_map) as i32 + v0 as i32 & 0x1fi32) >> 3i32) as u16;
    let r22: u16 = (ram_word!(game, room_y_coordinate_on_map) as i32
        + ((ram_word!(game, samus_y_pos) as i32 & 0xff00i32) >> 8i32)
        + 1i32) as u16;
    let v2: u16 = (r20 as i32 + 4i32 * (r34 as i32 + r22 as i32)) as u16;
    let ref mut fresh98 = *ram_ptr!(game; 0x7f7, v2 as isize);
    *fresh98 = (*fresh98)
        | (kShr0x80
            [(ram_word!(game, room_x_coordinate_on_map) as i32 + v0 as i32 & 7i32) as usize]);
    let r32: u16 = v1;
    let r30: u16 = v2;
    let mut v3: u16 = (v2).wrapping_sub(4u16);
    let mut v4: i16 = ((v1) as i16).wrapping_sub(2i16);
    if ((v1) as i16).wrapping_sub(2i16) < 0i16 {
        v4 = (v4) & (7i16);
        v3 = v3.wrapping_sub(1);
        r46 = r46.wrapping_add(1);
    }
    let R50: u16 = v3;
    let r52: i32 = 2i32 * v4 as i32;
    let v6: i32 = r52 >> 1i32;
    let mut r24: u16 =
        (kShr0xFc00[v6 as usize]) & (u16::swap_bytes(*(ram_ptr!(game, u16; 0x7f7, v3 as isize))));
    let mut r26: u16 = (kShr0xFc00[v6 as usize])
        & (u16::swap_bytes(*(ram_ptr!(game, u16; 0x7f7, (v3 as i32 + 4i32) as isize))));
    let mut r28: u16 = (kShr0xFc00[v6 as usize])
        & (u16::swap_bytes(*(ram_ptr!(game, u16; 0x7f7, (v3 as i32 + 8i32) as isize))));
    let mut r9: *const u8 = RomPtr_82(
        crate::rtl::rom_table_word!(game, 0x829717, ram_word!(game, area_index) as isize),
        game,
    );
    let r15: *const u8 = r9;
    r9 = r9.offset(v3 as i32 as isize);
    let mut r38: u16 = u16::swap_bytes(*(r9 as *mut u16));
    r9 = r9.offset(4);
    let mut r40: u16 = u16::swap_bytes(*(r9 as *mut u16));
    r9 = r9.offset(4);
    let mut r42: u16 = u16::swap_bytes(*(r9 as *mut u16));
    if (R50 as i32 & 3i32) == 3i32 {
        let mut v10: i16 = (if r46 != 0u16 { r52 >> 1i32 } else { r32 as i32 }) as i16;
        if v10 as i32 - 6i32 & 0x8000i32 == 0 {
            let R48: u8 = (if r34 != 0u16 {
                R50 as i32 - 124i32
            } else {
                R50 as i32 + 125i32
            }) as u8;
            let v0_0: u16 = R48 as u16;
            let mut r44: u16 = 0u16;
            r9 = r15.offset(R48 as i32 as isize);
            *(&raw mut r44 as *mut u8).offset(0) = *ram_ptr!(game; 0x7f7, R48 as isize);
            *(&raw mut r44 as *mut u8).offset(1isize) = *r9;
            if (r34 as u8) == 32u8 {
                *(&raw mut r38 as *mut u8).offset(1isize) =
                    *(&raw mut r44 as *mut u8).offset(1isize);
                *(&raw mut r24 as *mut u8).offset(1isize) = r44 as u8;
            } else {
                *(&raw mut r38 as *mut u8).offset(0) = *(&raw mut r44 as *mut u8).offset(1isize);
                *(&raw mut r24 as *mut u8).offset(0) = r44 as u8;
            }
            *(&raw mut r44 as *mut u8).offset(0) =
                *ram_ptr!(game; 0x7f7, (v0_0 as i32 + 4i32) as isize);
            r9 = r9.offset(4);
            *(&raw mut r44 as *mut u8).offset(1isize) = *r9;
            if (r34 as u8) == 32u8 {
                *(&raw mut r40 as *mut u8).offset(1isize) =
                    *(&raw mut r44 as *mut u8).offset(1isize);
                *(&raw mut r26 as *mut u8).offset(1isize) = r44 as u8;
            } else {
                *(&raw mut r40 as *mut u8).offset(0) = *(&raw mut r44 as *mut u8).offset(1isize);
                *(&raw mut r26 as *mut u8).offset(0) = r44 as u8;
            }
            *(&raw mut r44 as *mut u8).offset(0) =
                *ram_ptr!(game; 0x7f7, (v0_0 as i32 + 8i32) as isize);
            r9 = r9.offset(4);
            *(&raw mut r44 as *mut u8).offset(1isize) = *r9;
            if (r34 as u8) == 32u8 {
                *(&raw mut r42 as *mut u8).offset(1isize) =
                    *(&raw mut r44 as *mut u8).offset(1isize);
                *(&raw mut r28 as *mut u8).offset(1isize) = r44 as u8;
            } else {
                *(&raw mut r42 as *mut u8).offset(0) = *(&raw mut r44 as *mut u8).offset(1isize);
                *(&raw mut r28 as *mut u8).offset(0) = r44 as u8;
            }
        }
    }
    let mut n: i32 = r52 >> 1i32;
    while n != 0 {
        r24 = (r24).wrapping_mul(2u16);
        r38 = (r38).wrapping_mul(2u16);
        r26 = (r26).wrapping_mul(2u16);
        r40 = (r40).wrapping_mul(2u16);
        r28 = (r28).wrapping_mul(2u16);
        r42 = (r42).wrapping_mul(2u16);
        n -= 1;
    }
    UpdateMinimapInside(r18, r22, r34, r30, r32, r38, r24, r40, r26, r42, r28, game);
}
pub unsafe fn UpdateMinimapInside(
    r18: u16,
    r22: u16,
    r34: u16,
    r30: u16,
    r32: u16,
    mut r38: u16,
    mut r24: u16,
    mut r40: u16,
    mut r26: u16,
    mut r42: u16,
    mut r28: u16,
    game: &mut Game,
) {
    let mut v0: u16 = 0;
    let mut v1: i16 = 0;
    let mut v5: i16 = 0;
    let mut v7: i16 = 0;
    let mut v8: i16 = 0;
    *(&raw mut v0 as *mut u8).offset(0) = (((r34).wrapping_add(r22)) as i32 >> 8i32) as u8;
    *(&raw mut v0 as *mut u8).offset(1isize) = ((r34) as u8).wrapping_add((r22) as u8);
    let t: u16 = (r18 as i32 + (v0 as i32 >> 3i32)) as u16;
    if r34 != 0u16 && (t as i32 & 0x1fi32) - 2i32 & 0x8000i32 != 0 {
        v1 = ((t) as i16).wrapping_sub(1026i16);
    } else {
        v1 = ((t) as i16).wrapping_sub(34i16);
    }
    let mut v2: u16 = (2u16).wrapping_mul((v1) as u16);
    let r0: *const u16 = RomPtr(
        Load24(
            (game.g_rom.offset(crate::rtl::lorom_offset(0x82964a)) as *mut LongPtr)
                .offset(ram_word!(game, area_index) as isize),
        ),
        game,
    ) as *const u16;
    let r3: *const u16 = r0.offset(32);
    let r6: *const u16 = r0.offset(64);
    let mut n: i32 = 5i32;
    let mut v3: u16 = 0u16;
    loop {
        let mut v4: bool = r38 as i32 >> 15i32 != 0;
        r38 = (r38).wrapping_mul(2u16);
        if !v4 || {
            v5 = *r0.offset((v2 as i32 >> 1i32) as isize) as i16;
            ram_word!(game, has_area_map) == 0
        } {
            v5 = 31i16;
        }
        let v6: i32 = v3 as i32 >> 1i32;
        set_ram_word!(game.hud_tilemap[(v6 + 26i32)] = (v5 as i32 & 0xc3ffi32 | 0x2c00i32) as u16);
        v4 = r24 as i32 >> 15i32 != 0;
        r24 = (r24).wrapping_mul(2u16);
        if v4 {
            set_ram_word!(
                game.hud_tilemap[(v6 + 26i32)] = (*r0.offset((v2 as i32 >> 1i32) as isize) as i32
                    & 0xc3ffi32
                    | 0x2800i32) as u16
            );
        }
        v4 = r40 as i32 >> 15i32 != 0;
        r40 = (r40).wrapping_mul(2u16);
        if !v4 || {
            v7 = *r3.offset((v2 as i32 >> 1i32) as isize) as i16;
            ram_word!(game, has_area_map) == 0
        } {
            v7 = 31i16;
        }
        set_ram_word!(game.hud_tilemap[(v6 + 58i32)] = (v7 as i32 & 0xc3ffi32 | 0x2c00i32) as u16);
        v4 = r26 as i32 >> 15i32 != 0;
        r26 = (r26).wrapping_mul(2u16);
        if v4 {
            set_ram_word!(
                game.hud_tilemap[(v6 + 58i32)] = (*r3.offset((v2 as i32 >> 1i32) as isize) as i32
                    & 0xc3ffi32
                    | 0x2800i32) as u16
            );
            if n == 3i32 && (ram_word!(game, hud_tilemap[(v6 + 58i32)]) as i32 & 0x1ffi32) == 40i32
            {
                let ref mut fresh99 =
                    *(ram_ptr!(game, u16; 0x7f3) as *mut u8).offset(r30 as i32 as isize);
                *fresh99 = (*fresh99) | (kShr0x80[r32 as usize]);
            }
        }
        v4 = r42 as i32 >> 15i32 != 0;
        r42 = (r42).wrapping_mul(2u16);
        if !v4 || {
            v8 = *r6.offset((v2 as i32 >> 1i32) as isize) as i16;
            ram_word!(game, has_area_map) == 0
        } {
            v8 = 31i16;
        }
        set_ram_word!(game.hud_tilemap[(v6 + 90i32)] = (v8 as i32 & 0xc3ffi32 | 0x2c00i32) as u16);
        v4 = r28 as i32 >> 15i32 != 0;
        r28 = (r28).wrapping_mul(2u16);
        if v4 {
            set_ram_word!(
                game.hud_tilemap[(v6 + 90i32)] = (*r6.offset((v2 as i32 >> 1i32) as isize) as i32
                    & 0xc3ffi32
                    | 0x2800i32) as u16
            );
        }
        v3 = (v3).wrapping_add(2u16);
        v2 = (v2).wrapping_add(2u16);
        if (v2 as i32 & 0x3fi32) == 0i32 {
            v2 = (v2).wrapping_add(1984u16);
        }
        n -= 1;
        if (n == 0) {
            break;
        }
    }
    if (*ram_ptr!(game; 0x5b5) as i32 & 8i32) == 0i32 {
        update_ram_word!(game.hud_tilemap[60], |fresh100| (fresh100) | (0x1c00u16));
    }
}
pub unsafe fn Samus_HandleCooldown(game: &mut Game) {
    if ram_word!(game, time_is_frozen_flag) != 0 {
        set_ram_word!(game.cooldown_timer = 32u16);
    } else if ram_word!(game, cooldown_timer) != 0
        && ((ram_word!(game, cooldown_timer) as i32 & 0x8000i32) != 0i32 || {
            update_ram_word!(game.cooldown_timer, wrapping_sub(1));
            (ram_word!(game, cooldown_timer) as i32 & 0x8000i32) != 0i32
        })
    {
        set_ram_word!(game.cooldown_timer = 0u16);
    }
}
pub unsafe fn Samus_CanFireBeam(game: &mut Game) -> u8 {
    if ram_word!(game, projectile_counter) as i32 - 5i32 & 0x8000i32 == 0
        || (ram_word!(game, cooldown_timer) as u8) != 0u8
    {
        return 0u8;
    }
    set_ram_word!(game.cooldown_timer = 1u16);
    update_ram_word!(game.projectile_counter, wrapping_add(1));
    1u8
}
pub unsafe fn Samus_CanFireSuperMissile(game: &mut Game) -> u8 {
    if ram_word!(game, hud_item_index) != 2u16 {
        if ram_word!(game, projectile_counter) as i32 - 5i32 & 0x8000i32 == 0 {
            return 0u8;
        }
    } else if (ram_word!(game, projectile_counter) as i32 - 4i32 & 0x8000i32 == 0) {
        return 0u8;
    }
    if ram_word!(game, cooldown_timer) as u8 == 0 {
        set_ram_word!(game.cooldown_timer = 1u16);
        update_ram_word!(game.projectile_counter, wrapping_add(1));
        return 1u8;
    }
    0u8
}
pub unsafe fn UpdateBeamTilesAndPalette(game: &mut Game) {
    let v0: u16 = (2i32 * (ram_word!(game, equipped_beams) as i32 & 0xfffi32)) as u16;
    let mut v1: u16 = ram_word!(game, vram_write_queue_tail);
    (*gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game)).size = 256u16;
    v1 = (v1).wrapping_add(2u16);
    (*gVramWriteEntry(v1 as i32, game)).size =
        crate::rtl::rom_table_word!(game, 0x90c3b1, (v0 as i32 >> 1i32) as isize);
    v1 = (v1).wrapping_add(2u16);
    let fresh155 = v1;
    v1 = v1.wrapping_add(1);
    *(&raw mut (*(gVramWriteEntry as unsafe fn(i32, &mut Game) -> *mut VramWriteEntry)(
        fresh155 as i32,
        game,
    ))
    .size as *mut u8)
        .offset(0) = -102i32 as u8;
    (*gVramWriteEntry(v1 as i32, game)).size = addr_unk_606300 as u16;
    set_ram_word!(game.vram_write_queue_tail = (v1).wrapping_add(2u16));
    WriteBeamPalette_Y(v0, game);
}
pub unsafe fn WriteBeamPalette_A(a: u16, game: &mut Game) {
    WriteBeamPalette_Y((2i32 * (a as i32 & 0xfffi32)) as u16, game);
}
pub unsafe fn WriteBeamPalette_Y(j: u16, game: &mut Game) {
    let r0: u16 = crate::rtl::rom_table_word!(game, 0x90c3c9, (j as i32 >> 1i32) as isize);
    let mut v1: u16 = 0u16;
    let mut v2: u16 = 0u16;
    loop {
        *(ram_ptr!(game, u16; 0xc000)).offset(((v2 as i32 >> 1i32) + 224i32) as isize) =
            *(RomPtr_90((r0).wrapping_add(v1), game) as *mut u16);
        v2 = (v2).wrapping_add(2u16);
        v1 = (v1).wrapping_add(2u16);
        if (((v1) as i16).wrapping_sub(32i16) >= 0i16) {
            break;
        }
    }
}
pub unsafe fn LoadProjectilePalette(a: u16, game: &mut Game) {
    let r0: u16 = crate::rtl::rom_table_word!(game, 0x90c3c9, (a as i32 & 0xfffi32) as isize);
    let mut v1: u16 = 0u16;
    let mut v2: u16 = 0u16;
    loop {
        *(ram_ptr!(game, u16; 0xc000)).offset(((v2 as i32 >> 1i32) + 224i32) as isize) =
            *(RomPtr_90((r0).wrapping_add(v1), game) as *mut u16);
        v2 = (v2).wrapping_add(2u16);
        v1 = (v1).wrapping_add(2u16);
        if (((v1) as i16).wrapping_sub(32i16) >= 0i16) {
            break;
        }
    }
}
pub unsafe fn ResetProjectileData(game: &mut Game) {
    let mut v0: u16 = 0u16;
    loop {
        let v1: i32 = v0 as i32 >> 1i32;
        set_ram_words! { game, 0u16;
            projectile_timers[v1], projectile_x_pos[v1], projectile_y_pos[v1], projectile_dir[v1],
            projectile_bomb_x_speed[v1], projectile_bomb_y_speed[v1], projectile_x_radius[v1],
            projectile_y_radius[v1], projectile_type[v1], projectile_damage[v1],
            projectile_bomb_instruction_ptr[v1], projectile_bomb_instruction_timers[v1],
            projectile_variables[v1], projectile_spritemap_pointers[v1],
        }
        set_ram_word!(game.projectile_bomb_pre_instructions[v1] = ((0x90b169i32) as u16));
        v0 = (v0).wrapping_add(2u16);
        if (((v0) as i16).wrapping_sub(20i16) >= 0i16) {
            break;
        }
    }
    set_ram_words! { game, 0u16; bomb_counter, cooldown_timer, projectile_counter, power_bomb_flag }
    if ram_word!(game, hud_auto_cancel_flag) != 0 {
        set_ram_words! { game, 0u16; hud_item_index, samus_auto_cancel_hud_item_index }
    }
    set_ram_words! { game, 0u16;
        speed_echo_xpos[0], speed_echo_xpos[1], speed_echo_xpos[2], speed_echo_xpos[3],
        speed_echo_ypos[0], speed_echo_ypos[1], speed_echo_ypos[2], speed_echo_ypos[3],
        speed_echo_xspeed[0], speed_echo_xspeed[1], speed_echo_xspeed[2], speed_echo_xspeed[3],
        speed_echoes_index,
    }
    if ram_word!(game, samus_special_super_palette_flags) != 0 {
        set_ram_word!(game.samus_special_super_palette_flags = 0u16);
        Samus_LoadSuitTargetPalette(game);
    }
    if ram_word!(game, hyper_beam_flag) != 0 {
        SpawnPalfxObject(addr_stru_8DE1F0 as u16, game);
    }
}
pub unsafe fn ClearProjectile(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    set_ram_words! { game, 0u16;
        projectile_x_pos[v1], projectile_y_pos[v1], projectile_bomb_x_subpos[v1],
        projectile_bomb_y_subpos[v1], projectile_dir[v1], projectile_bomb_x_speed[v1],
        projectile_bomb_y_speed[v1], projectile_x_radius[v1], projectile_y_radius[v1],
        projectile_type[v1], projectile_damage[v1], projectile_bomb_instruction_ptr[v1],
        projectile_bomb_instruction_timers[v1], projectile_variables[v1],
        projectile_spritemap_pointers[v1],
    }
    set_ram_word!(game.projectile_bomb_pre_instructions[v1] = ((0x90b169i32) as u16));
    if ((k) as i16).wrapping_sub(10i16) >= 0i16 {
        if (update_ram_word!(game.bomb_counter, wrapping_sub(1), reread) as i32 & 0x8000i32) != 0i32
        {
            set_ram_word!(game.bomb_counter = 0u16);
        }
    } else {
        if (update_ram_word!(game.projectile_counter, wrapping_sub(1), reread) as i32 & 0x8000i32)
            != 0i32
        {
            set_ram_word!(game.projectile_counter = 0u16);
        }
    };
}
fn move_projectile_to_impact_point(index: u16, direction: usize, game: &mut Game) {
    const DIRECTIONS: [(i8, i8); 10] = [
        (0, -1),
        (1, -1),
        (1, 0),
        (1, 1),
        (0, 1),
        (0, 1),
        (-1, 1),
        (-1, 0),
        (-1, -1),
        (0, -1),
    ];
    const X_POSITION: usize = 0xb64;
    const Y_POSITION: usize = 0xb78;
    const X_RADIUS: usize = 0xbb4;
    const Y_RADIUS: usize = 0xbc8;

    let (x_direction, y_direction) = DIRECTIONS[direction];
    let slot_offset = usize::from(index & !1);
    // Diagonals update X before Y; an unused axis is neither read nor written.
    for (position_base, radius_base, step) in [
        (X_POSITION, X_RADIUS, x_direction),
        (Y_POSITION, Y_RADIUS, y_direction),
    ] {
        if step == 0 {
            continue;
        }
        let address = position_base + slot_offset;
        let position = u16::from_le_bytes([game.g_ram[address], game.g_ram[address + 1]]);
        let radius_address = radius_base + slot_offset;
        let radius =
            u16::from_le_bytes([game.g_ram[radius_address], game.g_ram[radius_address + 1]]);
        let position = if step > 0 {
            position.wrapping_add(radius)
        } else {
            position.wrapping_sub(radius)
        };
        game.g_ram[address..address + 2].copy_from_slice(&position.to_le_bytes());
    }
}
pub unsafe fn KillProjectile(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    if (ram_word!(game, projectile_type[v1]) as i32 & 0xf00i32) != 0i32 {
        if (*((ram_ptr!(game, u16; 0xc18)).offset(v1 as isize) as *mut u8).offset(1isize) as i32
            & 0xfi32)
            - 3i32
            & 0x8000i32
            == 0
        {
            ClearProjectile(k, game);
            return;
        }
    } else {
        move_projectile_to_impact_point(
            k,
            (ram_word!(game, projectile_dir[v1]) as i32 & 0xfi32) as usize,
            game,
        );
    }
    KillProjectileInner(k, game);
    set_ram_word!(
        game.projectile_bomb_pre_instructions[(k as i32 >> 1i32)] = ((0x90b169i32) as u16)
    );
}
crate::game::address_dispatch! {
    pub unsafe fn CallProjPreInstr(ea: u32, k: u16, game: &mut Game) => ea;
        call (k, game); default { Unreachable(game); };
    {
        9482601 => [()]; 9481971 => ProjPreInstr_Beam_NoWaveBeam; 9482058 => ProjPreInstr_Dir0459;
        9482066 => ProjPreInstr_Dir1368; 9482080 => ProjPreInstr_Dir27;
        9482088 => ProjPreInstr_Missile; 9482183 => ProjPreInstr_Missile_Func0459;
        9482191 => ProjPreInstr_Missile_Func1368; 9482205 => ProjPreInstr_Missile_Func27;
        9482213 => ProjPreInstr_SuperMissile; 9482311 => ProjPreInstr_SuperMissile_Func0459;
        9482322 => ProjPreInstr_SuperMissile_Func1368; 9482346 => ProjPreInstr_SuperMissile_Func27;
        9482357 => ProjPreInstr_Func1; 9482393 => ProjPreInstr_Bomb;
        9482414 => ProjPreInstr_PowerBomb; 9482435 => ProjPreInstr_WavePlasmaEtc;
        9482468 => ProjPreInstr_BeamOrIceWave; 9482499 => ProjPreInstr_Wave_Shared;
        9482555 => ProjPreInstr_WavePlasmaEtc_0459; 9482563 => ProjPreInstr_WavePlasmaEtc_1368;
        9482577 => ProjPreInstr_WavePlasmaEtc_27; 9482585 => ProjPreInstr_HyperBeam;
        9490185 => ProjPreInstr_IceSba; 9490298 => ProjPreInstr_IceSba2;
        9491666 => ProjPreInstr_SpeedEcho; 9492371 => ProjPreInstr_PlasmaSba;
        9492449 => ProjPreInstr_PlasmaSbaFunc_0; 9492474 => ProjPreInstr_PlasmaSbaFunc_1;
        9492499 => ProjPreInstr_PlasmaSbaFunc_2; 9492727 => ProjPreInstr_SpreadBomb;
        9493000 => ProjPreInstr_WaveSba; 9493254 => ProjPreInstr_SpazerSba;
        9493660 => ProjPreInstr_EndOfSpazerSba; 9498579 => ProjPreInstr_UnknownProj8027;
    }
}
pub unsafe fn HandleProjectile(game: &mut Game) {
    set_ram_word!(game.projectile_index = 18u16);
    let mut i: i32 = 18i32;
    while i >= 0i32 {
        let v1: i32 = i >> 1i32;
        if ram_word!(game, projectile_bomb_instruction_ptr[v1]) != 0 {
            CallProjPreInstr(
                ((ram_word!(game, projectile_bomb_pre_instructions[v1])) as u32) | (0x900000u32),
                i as u16,
                game,
            );
            RunProjectileInstructions(game);
            i = ram_word!(game, projectile_index) as i32;
        }
        i -= 2i32;
        set_ram_word!(game.projectile_index = i as u16);
    }
}
static kDirToVelMult16_X: [i16; 10] = [0, 16, 16, 16, 0, 0, -16, -16, -16, 0];
static kDirToVelMult16_Y: [i16; 10] = [-16, -16, 0, 16, 16, 16, 16, 0, -16, -16];
static kProjPreInstr_Beam_Funcs: [Option<Func_Y_V>; 10] = callback_table! { unsafe Func_Y_V;
    ProjPreInstr_Dir0459, ProjPreInstr_Dir1368, ProjPreInstr_Dir27, ProjPreInstr_Dir1368,
    ProjPreInstr_Dir0459, ProjPreInstr_Dir0459, ProjPreInstr_Dir1368, ProjPreInstr_Dir27,
    ProjPreInstr_Dir1368, ProjPreInstr_Dir0459,
};
pub unsafe fn ProjPreInstr_Beam_NoWaveBeam(mut k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    if (ram_word!(game, projectile_dir[v1]) as i32 & 0xf0i32) != 0i32 {
        ClearProjectile(k, game);
    } else {
        let ref mut fresh72 = *(ram_ptr!(game, u16; 0xc90)).offset(v1 as isize);
        let fresh73 = *fresh72;
        *fresh72 = (*fresh72).wrapping_sub(1);
        if fresh73 == 1u16 {
            set_ram_word!(game.projectile_timers[v1] = 4u16);
            SpawnProjectileTrail(k, game);
            k = ram_word!(game, projectile_index);
        }
        let v3: i32 = k as i32 >> 1i32;
        let v4: u16 = (2i32 * (ram_word!(game, projectile_dir[v3]) as i32 & 0xfi32)) as u16;
        let v5: i32 = v4 as i32 >> 1i32;
        update_ram_word!(
            game.projectile_bomb_x_speed[v3],
            wrapping_add((kDirToVelMult16_X[v5 as usize]) as u16)
        );
        update_ram_word!(
            game.projectile_bomb_y_speed[v3],
            wrapping_add((kDirToVelMult16_Y[v5 as usize]) as u16)
        );
        kProjPreInstr_Beam_Funcs[v5 as usize].expect("non-null function pointer")(v4, game);
        DeleteProjectileIfFarOffScreen(game);
    };
}
pub unsafe fn ProjPreInstr_Dir0459(k: u16, game: &mut Game) {
    BlockCollNoWaveBeamVert(ram_word!(game, projectile_index), game);
}
pub unsafe fn ProjPreInstr_Dir1368(k: u16, game: &mut Game) {
    let v1: u16 = ram_word!(game, projectile_index);
    if BlockCollNoWaveBeamHoriz(ram_word!(game, projectile_index), game) as i32 & 1i32 == 0 {
        BlockCollNoWaveBeamVert(v1, game);
    }
}
pub unsafe fn ProjPreInstr_Dir27(k: u16, game: &mut Game) {
    BlockCollNoWaveBeamHoriz(ram_word!(game, projectile_index), game);
}
static kProjPreInstr_Missile_Funcs: [Option<Func_Y_V>; 10] = callback_table! { unsafe Func_Y_V;
    ProjPreInstr_Missile_Func0459, ProjPreInstr_Missile_Func1368, ProjPreInstr_Missile_Func27,
    ProjPreInstr_Missile_Func1368, ProjPreInstr_Missile_Func0459, ProjPreInstr_Missile_Func0459,
    ProjPreInstr_Missile_Func1368, ProjPreInstr_Missile_Func27, ProjPreInstr_Missile_Func1368,
    ProjPreInstr_Missile_Func0459,
};
pub unsafe fn ProjPreInstr_Missile(mut k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    if (ram_word!(game, projectile_dir[v1]) as i32 & 0xf0i32) != 0i32 {
        ClearProjectile(k, game);
    } else {
        let ref mut fresh68 = *(ram_ptr!(game, u16; 0xc90)).offset(v1 as isize);
        let fresh69 = *fresh68;
        *fresh68 = (*fresh68).wrapping_sub(1);
        if fresh69 == 1u16 {
            set_ram_word!(game.projectile_timers[v1] = 4u16);
            SpawnProjectileTrail(k, game);
            k = ram_word!(game, projectile_index);
        }
        let v3: i32 = k as i32 >> 1i32;
        let v4: u16 = (2i32 * (ram_word!(game, projectile_dir[v3]) as i32 & 0xfi32)) as u16;
        let v5: i32 = v4 as i32 >> 1i32;
        update_ram_word!(
            game.projectile_bomb_x_speed[v3],
            wrapping_add((kDirToVelMult16_X[v5 as usize]) as u16)
        );
        update_ram_word!(
            game.projectile_bomb_y_speed[v3],
            wrapping_add((kDirToVelMult16_Y[v5 as usize]) as u16)
        );
        Missile_Func1(k, game);
        kProjPreInstr_Missile_Funcs[(v4 as i32 >> 1i32) as usize]
            .expect("non-null function pointer")(v4, game);
        DeleteProjectileIfFarOffScreen(game);
    };
}
pub unsafe fn ProjPreInstr_Missile_Func0459(k: u16, game: &mut Game) {
    BlockCollMissileVert(ram_word!(game, projectile_index), game);
}
pub unsafe fn ProjPreInstr_Missile_Func1368(k: u16, game: &mut Game) {
    let v1: u16 = ram_word!(game, projectile_index);
    if BlockCollMissileHoriz(ram_word!(game, projectile_index), game) == 0 {
        BlockCollMissileVert(v1, game);
    }
}
pub unsafe fn ProjPreInstr_Missile_Func27(k: u16, game: &mut Game) {
    BlockCollMissileHoriz(ram_word!(game, projectile_index), game);
}
static kProjPreInstr_SuperMissile_Funcs: [Option<Func_Y_V>; 10] = callback_table! { unsafe Func_Y_V;
    ProjPreInstr_SuperMissile_Func0459, ProjPreInstr_SuperMissile_Func1368,
    ProjPreInstr_SuperMissile_Func27, ProjPreInstr_SuperMissile_Func1368,
    ProjPreInstr_SuperMissile_Func0459, ProjPreInstr_SuperMissile_Func0459,
    ProjPreInstr_SuperMissile_Func1368, ProjPreInstr_SuperMissile_Func27,
    ProjPreInstr_SuperMissile_Func1368, ProjPreInstr_SuperMissile_Func0459,
};
pub unsafe fn ProjPreInstr_SuperMissile(mut k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    if (ram_word!(game, projectile_dir[v1]) as i32 & 0xf0i32) != 0i32 {
        ClearProjectile(k, game);
    } else {
        let ref mut fresh62 = *(ram_ptr!(game, u16; 0xc90)).offset(v1 as isize);
        let fresh63 = *fresh62;
        *fresh62 = (*fresh62).wrapping_sub(1);
        if fresh63 == 1u16 {
            set_ram_word!(game.projectile_timers[v1] = 2u16);
            SpawnProjectileTrail(k, game);
            k = ram_word!(game, projectile_index);
        }
        let mut v3: i16 = ((ram_word!(game, projectile_dir[(k as i32 >> 1i32)])) as i16) & (0xfi16);
        Missile_Func1(k, game);
        kProjPreInstr_SuperMissile_Funcs[v3 as usize].expect("non-null function pointer")(
            v3 as u16, game,
        );
        if (DeleteProjectileIfFarOffScreen(game) as i32 & 1i32 == 0) {
            return;
        }
    }
    let mut i: i32 = 8i32;
    while i >= 0i32 {
        if (ram_word!(game, projectile_type[(i >> 1i32)]) as i32 & 0xfffi32) == 512i32 {
            ClearProjectile(i as u16, game);
        }
        i -= 2i32;
    }
}
pub unsafe fn ProjPreInstr_SuperMissile_Func0459(k: u16, game: &mut Game) {
    BlockCollMissileVert(ram_word!(game, projectile_index), game);
    SuperMissileBlockCollDetect_Y(game);
}
pub unsafe fn ProjPreInstr_SuperMissile_Func1368(k: u16, game: &mut Game) {
    let v1: u16 = ram_word!(game, projectile_index);
    if BlockCollMissileHoriz(ram_word!(game, projectile_index), game) != 0 {
        SuperMissileBlockCollDetect_X(game);
    } else {
        SuperMissileBlockCollDetect_X(game);
        BlockCollMissileVert(v1, game);
        SuperMissileBlockCollDetect_Y(game);
    };
}
pub unsafe fn ProjPreInstr_SuperMissile_Func27(k: u16, game: &mut Game) {
    BlockCollMissileHoriz(ram_word!(game, projectile_index), game);
    SuperMissileBlockCollDetect_X(game);
}
pub unsafe fn ProjPreInstr_Func1(k: u16, game: &mut Game) {
    if (ram_word!(game, projectile_dir[(k as i32 >> 1i32)]) as i32 & 0xf0i32) != 0i32 {
        ClearProjectile(k, game);
        let mut i: i32 = 8i32;
        while i >= 0i32 {
            if (ram_word!(game, projectile_type[(i >> 1i32)]) as i32 & 0xfffi32) == 512i32 {
                ClearProjectile(i as u16, game);
            }
            i -= 2i32;
        }
    }
}
pub unsafe fn ProjPreInstr_Bomb(k: u16, game: &mut Game) {
    if (ram_word!(game, projectile_dir[(k as i32 >> 1i32)]) as i32 & 0xf0i32) != 0i32 {
        ClearProjectile(k, game);
    } else {
        Bomb_Func2(game);
        BombOrPowerBomb_Func1(k, game);
    };
}
pub unsafe fn ProjPreInstr_PowerBomb(k: u16, game: &mut Game) {
    if (ram_word!(game, projectile_dir[(k as i32 >> 1i32)]) as i32 & 0xf0i32) != 0i32 {
        ClearProjectile(k, game);
    } else {
        PowerBomb_Func3(game);
        BombOrPowerBomb_Func1(k, game);
    };
}
pub unsafe fn ProjPreInstr_WavePlasmaEtc(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    if (ram_word!(game, projectile_dir[v1]) as i32 & 0xf0i32) != 0i32 {
        ClearProjectile(k, game);
    } else {
        let ref mut fresh59 = *(ram_ptr!(game, u16; 0xc90)).offset(v1 as isize);
        let fresh60 = *fresh59;
        *fresh59 = (*fresh59).wrapping_sub(1);
        if fresh60 == 1u16 {
            set_ram_word!(game.projectile_timers[v1] = 4u16);
            SpawnProjectileTrail(k, game);
            ProjPreInstr_Wave_Shared(ram_word!(game, projectile_index), game);
        } else {
            ProjPreInstr_Wave_Shared(k, game);
        }
    };
}
pub unsafe fn ProjPreInstr_BeamOrIceWave(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    if (ram_word!(game, projectile_dir[v1]) as i32 & 0xf0i32) != 0i32 {
        ClearProjectile(k, game);
    } else {
        let ref mut fresh57 = *(ram_ptr!(game, u16; 0xc90)).offset(v1 as isize);
        let fresh58 = *fresh57;
        *fresh57 = (*fresh57).wrapping_sub(1);
        if fresh58 == 1u16 {
            set_ram_word!(game.projectile_timers[v1] = 3u16);
            SpawnProjectileTrail(k, game);
            ProjPreInstr_Wave_Shared(ram_word!(game, projectile_index), game);
        } else {
            ProjPreInstr_Wave_Shared(k, game);
        }
    };
}
static kProjPreInstr_WavePlasmaEtcFuncs: [Option<Func_Y_V>; 10] = callback_table! { unsafe Func_Y_V;
    ProjPreInstr_WavePlasmaEtc_0459, ProjPreInstr_WavePlasmaEtc_1368,
    ProjPreInstr_WavePlasmaEtc_27, ProjPreInstr_WavePlasmaEtc_1368,
    ProjPreInstr_WavePlasmaEtc_0459, ProjPreInstr_WavePlasmaEtc_0459,
    ProjPreInstr_WavePlasmaEtc_1368, ProjPreInstr_WavePlasmaEtc_27,
    ProjPreInstr_WavePlasmaEtc_1368, ProjPreInstr_WavePlasmaEtc_0459,
};
pub unsafe fn ProjPreInstr_Wave_Shared(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    let v2: u16 = (2i32 * (ram_word!(game, projectile_dir[v1]) as i32 & 0xfi32)) as u16;
    let v3: i32 = v2 as i32 >> 1i32;
    update_ram_word!(
        game.projectile_bomb_x_speed[v1],
        wrapping_add((kDirToVelMult16_X[v3 as usize]) as u16)
    );
    update_ram_word!(
        game.projectile_bomb_y_speed[v1],
        wrapping_add((kDirToVelMult16_Y[v3 as usize]) as u16)
    );
    kProjPreInstr_WavePlasmaEtcFuncs[v3 as usize].expect("non-null function pointer")(v2, game);
    DeleteProjectileIfFarOffScreen(game);
}
pub unsafe fn ProjPreInstr_WavePlasmaEtc_0459(k: u16, game: &mut Game) {
    BlockCollWaveBeamVert(ram_word!(game, projectile_index), game);
}
pub unsafe fn ProjPreInstr_WavePlasmaEtc_1368(k: u16, game: &mut Game) {
    let v1: u16 = ram_word!(game, projectile_index);
    if BlockCollWaveBeamHoriz(ram_word!(game, projectile_index), game) as i32 & 1i32 == 0 {
        BlockCollWaveBeamVert(v1, game);
    }
}
pub unsafe fn ProjPreInstr_WavePlasmaEtc_27(k: u16, game: &mut Game) {
    BlockCollWaveBeamHoriz(ram_word!(game, projectile_index), game);
}
pub unsafe fn ProjPreInstr_HyperBeam(k: u16, game: &mut Game) {
    if (ram_word!(game, projectile_dir[(k as i32 >> 1i32)]) as i32 & 0xf0i32) != 0i32 {
        ClearProjectile(k, game);
    } else {
        ProjPreInstr_Wave_Shared(k, game);
    };
}
pub unsafe fn DeleteProjectileIfFarOffScreen(game: &mut Game) -> u8 {
    let mut v0: i16 = ((*(ram_ptr!(game, u16; 0xb64))
        .offset((ram_word!(game, projectile_index) as i32 >> 1i32) as isize))
        as i16)
        .wrapping_sub((ram_word!(game, layer1_x_pos)) as i16);
    if v0 as i32 + 64i32 & 0x8000i32 == 0 && v0 as i32 - 320i32 & 0x8000i32 != 0 {
        let mut v2: i16 = ((*(ram_ptr!(game, u16; 0xb78))
            .offset((ram_word!(game, projectile_index) as i32 >> 1i32) as isize))
            as i16)
            .wrapping_sub((ram_word!(game, layer1_y_pos)) as i16);
        if v2 as i32 + 64i32 & 0x8000i32 == 0 && v2 as i32 - 320i32 & 0x8000i32 != 0 {
            return 0u8;
        }
    }
    ClearProjectile(ram_word!(game, projectile_index), game);
    1u8
}
static kInitializeProjectileSpeed_XY_Diag: [u16; 24] = [
    0x400, 0x2ab, 0x400, 0x2ab, 0x400, 0x2ab, 0x400, 0x2ab, 0x400, 0x2ab, 0x400, 0x2ab, 0x400,
    0x2ab, 0x400, 0x2ab, 0x400, 0x2ab, 0x400, 0x2ab, 0x400, 0x2ab, 0x400, 0x2ab,
];
pub unsafe fn SetInitialProjectileSpeed(r20: u16, game: &mut Game) {
    let v0: i32 = r20 as i32 >> 1i32;
    let v1: u16 = (4i32 * (ram_word!(game, projectile_type[v0]) as i32 & 0xfi32)) as u16;
    let v2: u16 = (2i32 * (ram_word!(game, projectile_dir[v0]) as i32 & 0xfi32)) as u16;
    let mut r22: u16 = 0;
    if v2 == 0 || v2 == 4u16 || v2 == 8u16 || v2 == 10u16 || v2 == 14u16 || v2 == 18u16 {
        r22 = kInitializeProjectileSpeed_XY_Diag[(v1 as i32 >> 1i32) as usize];
    } else {
        if v2 != 2u16 && v2 != 6u16 && v2 != 12u16 && v2 != 16u16 {
            Unreachable(game);
        }
        r22 = kInitializeProjectileSpeed_XY_Diag[((v1 as i32 >> 1i32) + 1i32) as usize];
    }
    InitializeProjectileSpeed(r20, r22, game);
}
pub unsafe fn InitializeProjectileSpeedOfType(r20: u16, game: &mut Game) {
    InitializeProjectileSpeed(r20, 0u16, game);
}
pub unsafe fn InitializeProjectileSpeed(k: u16, r22: u16, game: &mut Game) {
    let mut r18: u16 = 0;
    let kh: i32 = k as i32 >> 1i32;
    set_ram_words! { game, 0u16; projectile_bomb_x_subpos[kh], projectile_bomb_y_subpos[kh] }
    match 2i32 * (ram_word!(game, projectile_dir[kh]) as i32 & 0xfi32) {
        0 | 18 => {
            if ram_word!(game, projectile_init_speed_samus_moved_up) as u8 != 0 {
                r18 = (*((ram_ptr!(game, u16; 0xdb0) as *mut u8).offset(1) as *mut u16) as i32
                    >> 2i32
                    | 0xc000i32) as u16;
            } else {
                r18 = 0u16;
            }
            set_ram_word!(game.projectile_bomb_y_speed[kh] = (r18).wrapping_sub(r22));
            set_ram_word!(game.projectile_bomb_x_speed[kh] = 0u16);
        }
        2 => {
            if ram_word!(game, projectile_init_speed_samus_moved_up) as u8 != 0 {
                r18 = (*((ram_ptr!(game, u16; 0xdb0) as *mut u8).offset(1) as *mut u16) as i32
                    >> 2i32
                    | 0xc000i32) as u16;
            } else {
                r18 = 0u16;
            }
            set_ram_word!(game.projectile_bomb_y_speed[kh] = (r18).wrapping_sub(r22));
            set_ram_word!(
                game.projectile_bomb_x_speed[kh] = (*((ram_ptr!(game, u16; 0xdac) as *mut u8)
                    .offset(1) as *mut u16))
                    .wrapping_add(r22)
            );
        }
        4 => {
            set_ram_word!(game.projectile_bomb_y_speed[kh] = 0u16);
            set_ram_word!(
                game.projectile_bomb_x_speed[kh] = (*((ram_ptr!(game, u16; 0xdac) as *mut u8)
                    .offset(1) as *mut u16))
                    .wrapping_add(r22)
            );
        }
        6 => {
            set_ram_word!(
                game.projectile_bomb_y_speed[kh] = (*((ram_ptr!(game, u16; 0xdb4) as *mut u8)
                    .offset(1) as *mut u16))
                    .wrapping_add(r22)
            );
            set_ram_word!(
                game.projectile_bomb_x_speed[kh] = (*((ram_ptr!(game, u16; 0xdac) as *mut u8)
                    .offset(1) as *mut u16))
                    .wrapping_add(r22)
            );
        }
        8 | 10 => {
            set_ram_word!(
                game.projectile_bomb_y_speed[kh] = (*((ram_ptr!(game, u16; 0xdb4) as *mut u8)
                    .offset(1) as *mut u16))
                    .wrapping_add(r22)
            );
            set_ram_word!(game.projectile_bomb_x_speed[kh] = 0u16);
        }
        12 => {
            set_ram_word!(
                game.projectile_bomb_y_speed[kh] = (*((ram_ptr!(game, u16; 0xdb4) as *mut u8)
                    .offset(1) as *mut u16))
                    .wrapping_add(r22)
            );
            set_ram_word!(
                game.projectile_bomb_x_speed[kh] = (*((ram_ptr!(game, u16; 0xda8) as *mut u8)
                    .offset(1) as *mut u16))
                    .wrapping_sub(r22)
            );
        }
        14 => {
            set_ram_word!(game.projectile_bomb_y_speed[kh] = 0u16);
            set_ram_word!(
                game.projectile_bomb_x_speed[kh] = (*((ram_ptr!(game, u16; 0xda8) as *mut u8)
                    .offset(1) as *mut u16))
                    .wrapping_sub(r22)
            );
        }
        16 => {
            if ram_word!(game, projectile_init_speed_samus_moved_up) as u8 != 0 {
                r18 = (*((ram_ptr!(game, u16; 0xdb0) as *mut u8).offset(1) as *mut u16) as i32
                    >> 2i32
                    | 0xc000i32) as u16;
            } else {
                r18 = 0u16;
            }
            set_ram_word!(game.projectile_bomb_y_speed[kh] = (r18).wrapping_sub(r22));
            set_ram_word!(
                game.projectile_bomb_x_speed[kh] = (*((ram_ptr!(game, u16; 0xda8) as *mut u8)
                    .offset(1) as *mut u16))
                    .wrapping_sub(r22)
            );
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn Missile_Func1(k: u16, game: &mut Game) {
    static word_90C301: u16 = 0x100u16;
    let mut v3: u16 = 0;
    let v1: i32 = k as i32 >> 1i32;
    if (ram_word!(game, projectile_variables[v1]) as i32 & 0xff00i32) != 0i32 {
        if (ram_word!(game, projectile_type[v1]) as i32 & 0x200i32) != 0i32 {
            v3 = addr_kSuperMissileAccelerations2 as u16;
        } else {
            v3 = addr_kSuperMissileAccelerations as u16;
        }
        let v4: *const u8 = RomPtr_90(
            (v3 as i32 + 4i32 * (ram_word!(game, projectile_dir[v1]) as i32 & 0xfi32)) as u16,
            game,
        );
        update_ram_word!(
            game.projectile_bomb_x_speed[v1],
            wrapping_add(*(v4 as *mut u16))
        );
        update_ram_word!(
            game.projectile_bomb_y_speed[v1],
            wrapping_add(*(v4.offset(2) as *mut u16))
        );
    } else {
        let v2: u16 = (word_90C301).wrapping_add(ram_word!(game, projectile_variables[v1]));
        set_ram_word!(game.projectile_variables[v1] = v2);
        if (v2 as i32 & 0xff00i32) != 0i32 {
            InitializeProjectileSpeed(k, ram_word!(game, projectile_variables[v1]), game);
            if (ram_word!(game, projectile_type[v1]) as i32 & 0x200i32) != 0i32 {
                Missile_Func2(game);
            }
        }
    };
}
pub unsafe fn SuperMissileBlockCollDetect_Y(game: &mut Game) {
    let v0: i32 = ram_word!(game, projectile_index) as i32 >> 1i32;
    if (ram_word!(game, projectile_type[v0]) as i32 & 0xf00i32) == 512i32
        || (ram_word!(game, projectile_type[v0]) as i32 & 0xf00i32) == 2048i32
    {
        let v5: u8 = ram_word!(game, projectile_variables[v0]) as u8;
        if *(ram_ptr!(game, u16; 0xc7c) as *mut u8)
            .offset(ram_word!(game, projectile_index) as i32 as isize)
            .offset(1)
            != 0
        {
            let v1: u16 = (abs16(ram_word!(game, projectile_bomb_y_speed[v0]))) & (0xff00u16);
            if v1 as i32 - 2816i32 & 0x8000i32 != 0 {
                let v4: i32 = ram_word!(game, projectile_index) as i32 >> 1i32;
                if (ram_word!(game, projectile_type[v4]) as i32 & 0xf00i32) != 2048i32 {
                    set_ram_word!(
                        game.projectile_y_pos[(v5 as i32 >> 1i32)] =
                            ram_word!(game, projectile_y_pos[v4])
                    );
                    return;
                }
            } else {
                let r18: u16 = ((v1 as i32 >> 8i32) - 10i32) as u16;
                if (ram_word!(game, projectile_bomb_y_speed[v0]) as i32 & 0x8000i32) == 0i32 {
                    let v6: u16 = ram_word!(game, projectile_index);
                    set_ram_word!(
                        game.projectile_y_pos[(v5 as i32 >> 1i32)] =
                            (ram_word!(game, projectile_y_pos[v0])).wrapping_sub(r18)
                    );
                    set_ram_word!(game.projectile_index = v5 as u16);
                    BlockCollMissileVert(v5 as u16, game);
                    set_ram_word!(game.projectile_index = v6);
                    if (ram_word!(game, projectile_type[(v6 as i32 >> 1i32)]) as i32 & 0xf00i32)
                        == 2048i32
                    {
                        ClearProjectile(v5 as u16, game);
                    }
                    return;
                }
                let v7: u16 = ram_word!(game, projectile_index);
                set_ram_word!(
                    game.projectile_y_pos[(v5 as i32 >> 1i32)] = (r18).wrapping_add(
                        *(ram_ptr!(game, u16; 0xb78))
                            .offset((ram_word!(game, projectile_index) as i32 >> 1i32) as isize,),
                    )
                );
                set_ram_word!(game.projectile_index = v5 as u16);
                BlockCollMissileVert(v5 as u16, game);
                set_ram_word!(game.projectile_index = v7);
                if (ram_word!(game, projectile_type[(v7 as i32 >> 1i32)]) as i32 & 0xf00i32)
                    != 2048i32
                {
                    return;
                }
            }
            ClearProjectile(v5 as u16, game);
        }
    }
}
pub unsafe fn Projectile_Func4(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    if *(ram_ptr!(game, u16; 0xc7c) as *mut u8)
        .offset(k as i32 as isize)
        .offset(1)
        != 0
        && (ram_word!(game, projectile_type[v1]) as i32 & 0xf00i32) == 2048i32
    {
        ClearProjectile(ram_word!(game, projectile_variables[v1]) as u8 as u16, game);
    }
}
pub unsafe fn SuperMissileBlockCollDetect_X(game: &mut Game) {
    let v0: i32 = ram_word!(game, projectile_index) as i32 >> 1i32;
    if (ram_word!(game, projectile_type[v0]) as i32 & 0xf00i32) == 512i32
        || (ram_word!(game, projectile_type[v0]) as i32 & 0xf00i32) == 2048i32
    {
        let v5: u8 = ram_word!(game, projectile_variables[v0]) as u8;
        if *(ram_ptr!(game, u16; 0xc7c) as *mut u8)
            .offset(ram_word!(game, projectile_index) as i32 as isize)
            .offset(1)
            != 0
        {
            let v1: u16 = (abs16(ram_word!(game, projectile_bomb_x_speed[v0]))) & (0xff00u16);
            if v1 as i32 - 2816i32 & 0x8000i32 != 0 {
                let v4: i32 = ram_word!(game, projectile_index) as i32 >> 1i32;
                if (ram_word!(game, projectile_type[v4]) as i32 & 0xf00i32) != 2048i32 {
                    set_ram_word!(
                        game.projectile_x_pos[(v5 as i32 >> 1i32)] =
                            ram_word!(game, projectile_x_pos[v4])
                    );
                    return;
                }
            } else {
                let r18: u16 = ((v1 as i32 >> 8i32) - 10i32) as u16;
                if (ram_word!(game, projectile_bomb_x_speed[v0]) as i32 & 0x8000i32) == 0i32 {
                    let v6: u16 = ram_word!(game, projectile_index);
                    set_ram_word!(
                        game.projectile_x_pos[(v5 as i32 >> 1i32)] =
                            (ram_word!(game, projectile_x_pos[v0])).wrapping_sub(r18)
                    );
                    set_ram_word!(game.projectile_index = v5 as u16);
                    BlockCollMissileHoriz(v5 as u16, game);
                    set_ram_word!(game.projectile_index = v6);
                    if (ram_word!(game, projectile_type[(v6 as i32 >> 1i32)]) as i32 & 0xf00i32)
                        == 2048i32
                    {
                        ClearProjectile(v5 as u16, game);
                    }
                    return;
                }
                let v7: u16 = ram_word!(game, projectile_index);
                set_ram_word!(
                    game.projectile_x_pos[(v5 as i32 >> 1i32)] = (r18).wrapping_add(
                        *(ram_ptr!(game, u16; 0xb64))
                            .offset((ram_word!(game, projectile_index) as i32 >> 1i32) as isize,),
                    )
                );
                set_ram_word!(game.projectile_index = v5 as u16);
                BlockCollMissileHoriz(v5 as u16, game);
                set_ram_word!(game.projectile_index = v7);
                if (ram_word!(game, projectile_type[(v7 as i32 >> 1i32)]) as i32 & 0xf00i32)
                    != 2048i32
                {
                    return;
                }
            }
            ClearProjectile(v5 as u16, game);
        }
    }
}
pub unsafe fn ProjInstr_MoveLeftProjectileTrailDown(j: u16, game: &mut Game) {
    update_ram_word!(
        game.projectiletrail_left_y_pos[(j as i32 >> 1i32)],
        wrapping_add(1)
    );
}
pub unsafe fn ProjInstr_MoveRightProjectileTrailDown(j: u16, game: &mut Game) {
    update_ram_word!(
        game.projectiletrail_right_y_pos[(j as i32 >> 1i32)],
        wrapping_add(1)
    );
}
pub unsafe fn ProjInstr_MoveLeftProjectileTrailUp(j: u16, game: &mut Game) {
    update_ram_word!(
        game.projectiletrail_left_y_pos[(j as i32 >> 1i32)],
        wrapping_sub(1)
    );
}
pub unsafe fn SpawnProjectileTrail(k: u16, game: &mut Game) {
    let mut v2: i16 = 0;
    let mut v1: u16 = ram_word!(game, projectile_type[(k as i32 >> 1i32)]);
    if (v1 as i32 & 0xf00i32) != 0i32 {
        let v3: u16 = ((*(&raw mut v1 as *mut u8).offset(1isize)) as u16) & (0xfu16);
        if v3 >= 3u16 {
            return;
        }
        v2 = ((v3) as i16).wrapping_add(31i16);
    } else {
        v2 = ((ram_word!(game, projectile_type[(k as i32 >> 1i32)])) as i16) & (0x3fi16);
    }
    let mut v4: u16 = 34u16;
    while ram_word!(game, projectiletrail_left_instr_timer[(v4 as i32 >> 1i32)]) != 0 {
        v4 = (v4).wrapping_sub(2u16);
        if (v4 as i32 & 0x8000i32) != 0i32 {
            return;
        }
    }
    let v5: i32 = v4 as i32 >> 1i32;
    set_ram_words! { game, 1u16;
        projectiletrail_left_instr_timer[v5], projectiletrail_right_instr_timer[v5],
    }
    let v6: i32 = v2 as i32;
    set_ram_word!(
        game.projectiletrail_left_instr_list_ptr[v5] =
            crate::rtl::rom_table_word!(game, 0x90b5bb, v6 as isize)
    );
    set_ram_word!(
        game.projectiletrail_right_instr_list_ptr[v5] =
            crate::rtl::rom_table_word!(game, 0x90b609, v6 as isize)
    );
    ProjectileTrail_Func5(ram_word!(game, projectile_index), v4, game);
}
crate::game::address_dispatch! {
    pub unsafe fn CallProjInstr(ea: u32, j: u16, game: &mut Game) => ea;
        call (j, game); default { Unreachable(game); };
    {
        9483557 => ProjInstr_MoveLeftProjectileTrailDown;
        9483655 => ProjInstr_MoveRightProjectileTrailDown;
        9483699 => ProjInstr_MoveLeftProjectileTrailUp;
    }
}
pub unsafe fn HandleProjectileTrails(game: &mut Game) {
    let mut v11: *mut OamEnt = null_mut::<OamEnt>();
    let mut v23: *mut OamEnt = null_mut::<OamEnt>();
    let mut v29: *mut OamEnt = null_mut::<OamEnt>();
    let mut v34: *mut OamEnt = null_mut::<OamEnt>();
    let mut v6: u16 = 0;
    let mut v18: u16 = 0;
    if ram_word!(game, time_is_frozen_flag) == 0 {
        let mut skip_first_trail: bool;
        let mut v0: u16 = 34u16;
        loop {
            let mut v1: i32 = v0 as i32 >> 1i32;
            let mut v2: u16 = ram_word!(game, projectiletrail_left_instr_timer[v1]);
            if v2 != 0 {
                let v3: u16 = (v2).wrapping_sub(1u16);
                set_ram_word!(game.projectiletrail_left_instr_timer[v1] = v3);
                skip_first_trail = !(v3 != 0)
                    && ({
                        let mut i: i32 =
                            ram_word!(game, projectiletrail_left_instr_list_ptr[v1]) as i32;
                        loop {
                            let v5: *const u16 = RomPtr_90(i as u16, game) as *const u16;
                            v6 = *v5;
                            if (*v5 as i32 & 0x8000i32) == 0i32 {
                                break;
                            }
                            CallProjInstr(((v6) as u32) | (0x900000u32), v0, game);
                            i += 2i32;
                        }
                        let mut v7: i32 = v0 as i32 >> 1i32;
                        set_ram_word!(game.projectiletrail_left_instr_timer[v7] = v6);
                        !(v6 != 0)
                            || ({
                                set_ram_word!(
                                    game.projectiletrail_left_tile_and_attribs[v7] =
                                        *(RomPtr_90(i as u16, game) as *mut u16).offset(1)
                                );
                                set_ram_word!(
                                    game.projectiletrail_left_instr_list_ptr[v7] =
                                        (i + 4i32) as u16
                                );
                                false
                            })
                    });
                if skip_first_trail {
                } else {
                    let v8: u16 = ram_word!(game, oam_next_ptr);
                    if ((ram_word!(game, oam_next_ptr)) as i16).wrapping_sub(512i16) < 0i16 {
                        let v9: i32 = v0 as i32 >> 1i32;
                        let mut v10: i16 = ((ram_word!(game, projectiletrail_left_x_pos[v9]))
                            as i16)
                            .wrapping_sub((ram_word!(game, layer1_x_pos)) as i16);
                        if (v10 as i32 & 0xff00i32) == 0i32 {
                            v11 = gOamEnt(ram_word!(game, oam_next_ptr) as i32, game);
                            *(&raw mut (*v11).xcoord as *mut u16) = v10 as u16;
                            let mut v12: i16 = ((ram_word!(game, projectiletrail_left_y_pos[v9]))
                                as i16)
                                .wrapping_sub((ram_word!(game, layer1_y_pos)) as i16);
                            if (v12 as i32 & 0xff00i32) == 0i32 {
                                *(&raw mut (*v11).ycoord as *mut u16) = v12 as u16;
                                *(&raw mut (*v11).charnum as *mut u16) =
                                    ram_word!(game, projectiletrail_left_tile_and_attribs[v9]);
                                set_ram_word!(game.oam_next_ptr = (v8).wrapping_add(4u16));
                            }
                        }
                    }
                }
            }
            let v13: i32 = v0 as i32 >> 1i32;
            let v14: u16 = ram_word!(game, projectiletrail_right_instr_timer[v13]);
            if v14 != 0 {
                let v15: u16 = (v14).wrapping_sub(1u16);
                set_ram_word!(game.projectiletrail_right_instr_timer[v13] = v15);
                let draw_second_trail: bool = (v15 != 0)
                    || ({
                        let mut j: u16 = ram_word!(game, projectiletrail_right_instr_list_ptr[v13]);
                        loop {
                            let v17: *const u16 = RomPtr_90(j, game) as *const u16;
                            v18 = *v17;
                            if (*v17 as i32 & 0x8000i32) == 0i32 {
                                break;
                            }
                            CallProjInstr(((v18) as u32) | (0x900000u32), v0, game);
                            j = (j).wrapping_add(2u16);
                        }
                        let mut v19: i32 = 0;
                        v19 = v0 as i32 >> 1i32;
                        set_ram_word!(game.projectiletrail_right_instr_timer[v19] = v18);
                        (v18 != 0)
                            && ({
                                set_ram_word!(
                                    game.projectiletrail_right_tile_and_attribs[v19] =
                                        *(RomPtr_90(j, game) as *mut u16).offset(1)
                                );
                                set_ram_word!(
                                    game.projectiletrail_right_instr_list_ptr[v19] =
                                        (j).wrapping_add(4u16)
                                );
                                true
                            })
                    });
                if draw_second_trail {
                    let v20: u16 = ram_word!(game, oam_next_ptr);
                    if ((ram_word!(game, oam_next_ptr)) as i16).wrapping_sub(512i16) < 0i16 {
                        let v21: i32 = v0 as i32 >> 1i32;
                        let mut v22: i16 = ((ram_word!(game, projectiletrail_right_x_pos[v21]))
                            as i16)
                            .wrapping_sub((ram_word!(game, layer1_x_pos)) as i16);
                        if (v22 as i32 & 0xff00i32) == 0i32 {
                            v23 = gOamEnt(ram_word!(game, oam_next_ptr) as i32, game);
                            *(&raw mut (*v23).xcoord as *mut u16) = v22 as u16;
                            let mut v24: i16 = ((ram_word!(game, projectiletrail_right_y_pos[v21]))
                                as i16)
                                .wrapping_sub((ram_word!(game, layer1_y_pos)) as i16);
                            if (v24 as i32 & 0xff00i32) == 0i32 {
                                *(&raw mut (*v23).ycoord as *mut u16) = v24 as u16;
                                *(&raw mut (*v23).charnum as *mut u16) =
                                    ram_word!(game, projectiletrail_right_tile_and_attribs[v21]);
                                set_ram_word!(game.oam_next_ptr = (v20).wrapping_add(4u16));
                            }
                        }
                    }
                }
            }
            v0 = (v0).wrapping_sub(2u16);
            if (v0 as i32 & 0x8000i32) != 0i32 {
                return;
            }
        }
    }
    let mut k: u16 = 34u16;
    while (k as i32 & 0x8000i32) == 0i32 {
        let v26: u16 = ram_word!(game, oam_next_ptr);
        if ((ram_word!(game, oam_next_ptr)) as i16).wrapping_sub(512i16) < 0i16 {
            let v27: i32 = k as i32 >> 1i32;
            if ram_word!(game, projectiletrail_left_instr_timer[v27]) != 0 {
                let mut v28: i16 = ((ram_word!(game, projectiletrail_left_x_pos[v27])) as i16)
                    .wrapping_sub((ram_word!(game, layer1_x_pos)) as i16);
                if (v28 as i32 & 0xff00i32) == 0i32 {
                    v29 = gOamEnt(ram_word!(game, oam_next_ptr) as i32, game);
                    *(&raw mut (*v29).xcoord as *mut u16) = v28 as u16;
                    let mut v30: i16 = ((ram_word!(game, projectiletrail_left_y_pos[v27])) as i16)
                        .wrapping_sub((ram_word!(game, layer1_y_pos)) as i16);
                    if (v30 as i32 & 0xff00i32) == 0i32 {
                        *(&raw mut (*v29).ycoord as *mut u16) = v30 as u16;
                        *(&raw mut (*v29).charnum as *mut u16) =
                            ram_word!(game, projectiletrail_left_tile_and_attribs[v27]);
                        set_ram_word!(game.oam_next_ptr = (v26).wrapping_add(4u16));
                    }
                }
            }
        }
        let v31: u16 = ram_word!(game, oam_next_ptr);
        if ((ram_word!(game, oam_next_ptr)) as i16).wrapping_sub(512i16) < 0i16 {
            let v32: i32 = k as i32 >> 1i32;
            if ram_word!(game, projectiletrail_right_instr_timer[v32]) != 0 {
                let mut v33: i16 = ((ram_word!(game, projectiletrail_right_x_pos[v32])) as i16)
                    .wrapping_sub((ram_word!(game, layer1_x_pos)) as i16);
                if (v33 as i32 & 0xff00i32) == 0i32 {
                    v34 = gOamEnt(ram_word!(game, oam_next_ptr) as i32, game);
                    *(&raw mut (*v34).xcoord as *mut u16) = v33 as u16;
                    let mut v35: i16 = ((ram_word!(game, projectiletrail_right_y_pos[v32])) as i16)
                        .wrapping_sub((ram_word!(game, layer1_y_pos)) as i16);
                    if (v35 as i32 & 0xff00i32) == 0i32 {
                        *(&raw mut (*v34).ycoord as *mut u16) = v35 as u16;
                        *(&raw mut (*v34).charnum as *mut u16) =
                            ram_word!(game, projectiletrail_right_tile_and_attribs[v32]);
                        set_ram_word!(game.oam_next_ptr = (v31).wrapping_add(4u16));
                    }
                }
            }
        }
        k = (k).wrapping_sub(2u16);
    }
}
pub unsafe fn HudSelectionHandler_NothingOrPowerBombs(game: &mut Game) {
    let fire_uncharged: bool;
    set_ram_word!(game.prev_beam_charge_counter = ram_word!(game, flare_counter));
    if ram_word!(game, hyper_beam_flag) != 0u16
        || (ram_word!(game, equipped_beams) as i32 & 0x1000i32) == 0i32
    {
        if (ram_word!(game, button_config_shoot_x) as i32
            & ram_word!(game, joypad1_lastkeys) as i32)
            != 0i32
        {
            FireUnchargedBeam(game);
        }
        return;
    }
    if ram_word!(game, new_projectile_direction_changed_pose) != 0 {
        fire_uncharged = (ram_word!(game, flare_counter) as i32 - 60i32 & 0x8000i32 != 0);
    } else if (ram_word!(game, button_config_shoot_x) as i32
        & ram_word!(game, joypad1_lastkeys) as i32)
        == 0i32
    {
        if ram_word!(game, flare_counter) == 0 {
            return;
        }
        fire_uncharged = (ram_word!(game, flare_counter) as i32 - 60i32 & 0x8000i32 != 0);
    } else {
        if ram_word!(game, flare_counter) as i32 - 120i32 & 0x8000i32 != 0 {
            if update_ram_word!(game.flare_counter, wrapping_add(1), reread) == 1u16 {
                ClearFlareAnimationState(game);
                FireUnchargedBeam(game);
            }
        } else if FireSba(game) as i32 & 1i32 != 0 {
            set_ram_word!(game.flare_counter = 0u16);
            ClearFlareAnimationState(game);
            Samus_LoadSuitPalette(game);
        }
        return;
    }
    if fire_uncharged {
        set_ram_word!(game.flare_counter = 0u16);
        ClearFlareAnimationState(game);
        FireUnchargedBeam(game);
    } else {
        set_ram_word!(game.flare_counter = 0u16);
        ClearFlareAnimationState(game);
        FireChargedBeam(game);
    }
}
static kProjectileBombPreInstr: [u16; 12] = [
    fnProjPreInstr_Beam_NoWaveBeam as u16,
    fnProjPreInstr_BeamOrIceWave as u16,
    fnProjPreInstr_Beam_NoWaveBeam as u16,
    fnProjPreInstr_BeamOrIceWave as u16,
    fnProjPreInstr_Beam_NoWaveBeam as u16,
    fnProjPreInstr_WavePlasmaEtc as u16,
    fnProjPreInstr_Beam_NoWaveBeam as u16,
    fnProjPreInstr_WavePlasmaEtc as u16,
    fnProjPreInstr_Beam_NoWaveBeam as u16,
    fnProjPreInstr_WavePlasmaEtc as u16,
    fnProjPreInstr_Beam_NoWaveBeam as u16,
    fnProjPreInstr_WavePlasmaEtc as u16,
];
pub unsafe fn FireUnchargedBeam(game: &mut Game) {
    if ram_word!(game, hyper_beam_flag) != 0 {
        FireHyperBeam(game);
        return;
    }
    if Samus_CanFireBeam(game) as i32 & 1i32 != 0 {
        let mut v0: u16 = 0u16;
        while ram_word!(game, projectile_damage[(v0 as i32 >> 1i32)]) != 0 {
            v0 = (v0).wrapping_add(2u16);
            if (((v0) as i16).wrapping_sub(10i16) < 0i16) {
                continue;
            }
            v0 = (v0).wrapping_sub(2u16);
            break;
        }
        if InitProjectilePositionDirection(v0, game) as i32 & 1i32 == 0 {
            set_ram_word!(game.projectile_invincibility_timer = 10u16);
            let mut v1: u16 = v0;
            let v2: i32 = v0 as i32 >> 1i32;
            set_ram_word!(game.projectile_timers[v2] = 4u16);
            let mut v3: i8 = ram_word!(game, equipped_beams) as i8;
            set_ram_word!(
                game.projectile_type[v2] = (ram_word!(game, equipped_beams)) | (0x8000u16)
            );
            QueueSfx1_Max15(
                kUnchargedProjectile_Sfx[(v3 as i32 & 0xfi32) as usize],
                game,
            );
            set_ram_word!(game.play_resume_charging_beam_sfx = 0u16);
            InitializeProjectile(v1, game);
            let initialize_projectile: bool =
                if (ram_word!(game, equipped_beams) as i32 & 0x1000i32) != 0i32
                    || (ram_word!(game, button_config_shoot_x) as i32
                        & ram_word!(game, joypad1_newkeys) as i32)
                        != 0i32
                    || (ram_word!(game, button_config_shoot_x) as i32
                        & ram_word!(game, joypad1_newinput_samusfilter) as i32)
                        != 0i32
                {
                    let mut v6: u16 = ram_word!(game, projectile_type[v2]);
                    set_ram_word!(
                        game.cooldown_timer =
                            kProjectileCooldown_Uncharged[(v6 as i32 & 0x3fi32) as usize] as u16
                    );
                    !((v6 as i32 & 1i32) == 0i32)
                } else {
                    let mut v5: u16 = 0;
                    v5 = ram_word!(game, projectile_type[v2]);
                    set_ram_word!(
                        game.cooldown_timer =
                            kBeamAutoFireCooldowns[(v5 as i32 & 0x3fi32) as usize] as u16
                    );
                    !((v5 as i32 & 1i32) == 0i32)
                };
            if initialize_projectile {
                let mut v4: i32 = 0;
                v4 = v1 as i32 >> 1i32;
                set_ram_words! { game, 0u16;
                    projectile_bomb_x_speed[v4], projectile_bomb_y_speed[v4],
                }
                set_ram_word!(game.projectile_index = v1);
                WaveBeam_CheckColl(v1, game);
            } else {
                set_ram_words! { game, 0u16;
                    projectile_bomb_x_speed[v2], projectile_bomb_y_speed[v2],
                }
                set_ram_word!(game.projectile_index = v1);
                CheckBeamCollByDir(v1, game);
                v1 = ram_word!(game, projectile_index);
                if (*(ram_ptr!(game, u16; 0xc18))
                    .offset((ram_word!(game, projectile_index) as i32 >> 1i32) as isize)
                    as i32
                    & 0xf00i32)
                    != 0i32
                {
                    return;
                }
            }
            set_ram_word!(
                game.projectile_bomb_pre_instructions[(v1 as i32 >> 1i32)] =
                    kProjectileBombPreInstr[(ram_word!(game, projectile_type[(v1 as i32 >> 1i32)])
                        as i32
                        & 0xfi32) as usize]
            );
            SetInitialProjectileSpeed(v1, game);
            return;
        }
    }
    if ram_word!(game, prev_beam_charge_counter) as i32 - 16i32 & 0x8000i32 == 0 {
        set_ram_word!(game.play_resume_charging_beam_sfx = 0u16);
        QueueSfx1_Max15(2u16, game);
    }
}
static kFireChargedBeam_Funcs: [u16; 12] = [
    fnProjPreInstr_Beam_NoWaveBeam as u16,
    fnProjPreInstr_WavePlasmaEtc as u16,
    fnProjPreInstr_Beam_NoWaveBeam as u16,
    fnProjPreInstr_WavePlasmaEtc as u16,
    fnProjPreInstr_Beam_NoWaveBeam as u16,
    fnProjPreInstr_WavePlasmaEtc as u16,
    fnProjPreInstr_Beam_NoWaveBeam as u16,
    fnProjPreInstr_WavePlasmaEtc as u16,
    fnProjPreInstr_Beam_NoWaveBeam as u16,
    fnProjPreInstr_WavePlasmaEtc as u16,
    fnProjPreInstr_Beam_NoWaveBeam as u16,
    fnProjPreInstr_WavePlasmaEtc as u16,
];
pub unsafe fn FireChargedBeam(game: &mut Game) {
    if Samus_CanFireBeam(game) as i32 & 1i32 != 0 {
        let mut v0: u16 = 0u16;
        while ram_word!(game, projectile_damage[(v0 as i32 >> 1i32)]) != 0 {
            v0 = (v0).wrapping_add(2u16);
            if (((v0) as i16).wrapping_sub(10i16) < 0i16) {
                continue;
            }
            v0 = (v0).wrapping_sub(2u16);
            break;
        }
        let mut r20: u16 = v0;
        if InitProjectilePositionDirection(r20, game) as i32 & 1i32 == 0 {
            set_ram_word!(game.projectile_invincibility_timer = 10u16);
            let mut v1: u16 = r20;
            let v2: i32 = r20 as i32 >> 1i32;
            set_ram_word!(game.projectile_timers[v2] = 4u16);
            let v3: u16 = (ram_word!(game, equipped_beams) as i32 & 0x100fi32 | 0x8010i32) as u16;
            set_ram_word!(game.projectile_type[v2] = v3);
            QueueSfx1_Max15(kChargedProjectile_Sfx[(v3 as i32 & 0xfi32) as usize], game);
            set_ram_word!(game.play_resume_charging_beam_sfx = 0u16);
            InitializeProjectile(v1, game);
            let v5: u16 = ram_word!(game, projectile_type[v2]);
            set_ram_word!(
                game.cooldown_timer =
                    kProjectileCooldown_Uncharged[(v5 as i32 & 0x3fi32) as usize] as u16
            );
            let initialize_projectile: bool = if (v5 as i32 & 1i32) != 0i32 {
                let v4: i32 = v1 as i32 >> 1i32;
                set_ram_words! { game, 0u16;
                    projectile_bomb_x_speed[v4], projectile_bomb_y_speed[v4],
                }
                set_ram_word!(game.projectile_index = v1);
                WaveBeam_CheckColl(v1, game);
                true
            } else {
                set_ram_words! { game, 0u16;
                    projectile_bomb_x_speed[v2], projectile_bomb_y_speed[v2],
                }
                set_ram_word!(game.projectile_index = v1);
                CheckBeamCollByDir(v1, game);
                v1 = ram_word!(game, projectile_index);
                !((*(ram_ptr!(game, u16; 0xc18))
                    .offset((ram_word!(game, projectile_index) as i32 >> 1i32) as isize)
                    as i32
                    & 0xf00i32)
                    != 0i32)
            };
            if initialize_projectile {
                r20 = v1;
                set_ram_word!(
                    game.projectile_bomb_pre_instructions[(v1 as i32 >> 1i32)] =
                        kFireChargedBeam_Funcs[(ram_word!(
                            game,
                            projectile_type[(v1 as i32 >> 1i32)]
                        ) as i32
                            & 0xfi32) as usize]
                );
                SetInitialProjectileSpeed(r20, game);
            }
            set_ram_word!(game.charged_shot_glow_timer = 4u16);
            return;
        }
    }
    if ram_word!(game, prev_beam_charge_counter) as i32 - 16i32 & 0x8000i32 == 0 {
        set_ram_word!(game.play_resume_charging_beam_sfx = 0u16);
        QueueSfx1_Max15(2u16, game);
    }
}
static kProjectileOriginOffsets3_X: [i16; 10] = [2, 13, 11, 13, 2, -5, -14, -11, -19, -2];
static kProjectileOriginOffsets3_Y: [i16; 10] = [-8, -13, 1, 4, 13, 13, 4, 1, -19, -8];
static kProjectileOriginOffsets4_X: [i16; 10] = [2, 15, 15, 13, 2, -5, -13, -13, -15, -2];
static kProjectileOriginOffsets4_Y: [i16; 10] = [-8, -16, -2, 1, 13, 13, 1, -2, -16, -8];
pub unsafe fn InitProjectilePositionDirection(r20: u16, game: &mut Game) -> u8 {
    let mut v0: u16 = ram_word!(game, samus_pose);
    let mut direction_shots_fired: u16 = 0;
    if ram_word!(game, new_projectile_direction_changed_pose) != 0 {
        direction_shots_fired = ram_word!(game, new_projectile_direction_changed_pose) as u8 as u16;
        set_ram_word!(game.new_projectile_direction_changed_pose = 0u16);
    } else {
        direction_shots_fired = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
            as *mut SamusPoseParams)
            .offset(v0 as isize))
        .direction_shots_fired as u16;
        if (direction_shots_fired as i32 & 0xf0i32) != 0i32 {
            if direction_shots_fired != 16u16 || {
                *(&raw mut direction_shots_fired as *mut u8).offset(0) =
                    (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
                        as *mut SamusPoseParams)
                        .offset(ram_word!(game, samus_last_different_pose) as isize))
                    .direction_shots_fired;
                (direction_shots_fired as i32 & 0xf0i32) != 0i32
            } {
                update_ram_word!(game.projectile_counter, wrapping_sub(1));
                return 1u8;
            }
            direction_shots_fired = direction_shots_fired as u8 as u16;
            v0 = ram_word!(game, samus_pose);
        }
    }
    let v2: i32 = r20 as i32 >> 1i32;
    set_ram_word!(game.projectile_dir[v2] = direction_shots_fired);
    let r22: u16 = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
        as *mut SamusPoseParams)
        .offset(v0 as isize))
    .y_offset_to_gfx as u16;
    let v3: u16 = (2i32 * (ram_word!(game, projectile_dir[v2]) as i32 & 0xfi32)) as u16;
    if ram_word!(game, samus_pose) as i32 == kPose_75_FaceL_Moonwalk_AimUL as i32
        || ram_word!(game, samus_pose) as i32 == kPose_76_FaceR_Moonwalk_AimUR as i32
        || *ram_ptr!(game; 0xa1f) == 1u8
    {
        let v6: i32 = v3 as i32 >> 1i32;
        set_ram_word!(
            game.projectile_x_pos[v2] = (ram_word!(game, samus_x_pos))
                .wrapping_add((kProjectileOriginOffsets4_X[v6 as usize]) as u16)
        );
        set_ram_word!(
            game.projectile_y_pos[v2] = (ram_word!(game, samus_y_pos) as i32
                + kProjectileOriginOffsets4_Y[v6 as usize] as i32
                - r22 as i32) as u16
        );
        0u8
    } else {
        let v4: i32 = v3 as i32 >> 1i32;
        set_ram_word!(
            game.projectile_x_pos[v2] = (ram_word!(game, samus_x_pos))
                .wrapping_add((kProjectileOriginOffsets3_X[v4 as usize]) as u16)
        );
        set_ram_word!(
            game.projectile_y_pos[v2] = (ram_word!(game, samus_y_pos) as i32
                + kProjectileOriginOffsets3_Y[v4 as usize] as i32
                - r22 as i32) as u16
        );
        0u8
    }
}
pub unsafe fn HandleChargingBeamGfxAudio(game: &mut Game) {
    if ram_word!(game, hyper_beam_flag) != 0 {
        if ram_word!(game, flare_counter) != 0 {
            let mut i: i32 = 4i32;
            while i >= 0i32 {
                let mut v7: bool = *((ram_ptr!(game, u16; 0xcdc) as *mut u8).offset(i as isize)
                    as *mut u16)
                    == 1u16;
                let ref mut fresh17 =
                    *((ram_ptr!(game, u16; 0xcdc) as *mut u8).offset(i as isize) as *mut u16);
                *fresh17 = (*fresh17).wrapping_sub(1);
                let v8: bool = (*fresh17 as i32 & 0x8000i32) != 0i32;
                if v7 || v8 {
                    let ref mut fresh20 =
                        *((ram_ptr!(game, u16; 0xcd6) as *mut u8).offset(i as isize) as *mut u16);
                    let fresh21 = *fresh20;
                    *fresh20 = (*fresh20).wrapping_sub(1);
                    v7 = fresh21 == 1u16;
                    if v7 {
                        if i == 4i32 {
                            set_ram_word!(game.flare_counter = 0u16);
                        }
                    } else {
                        *((ram_ptr!(game, u16; 0xcdc) as *mut u8).offset(i as isize) as *mut u16) =
                            3u16;
                    }
                }
                DrawFlareAnimationComponent(i as u16, game);
                i -= 2i32;
            }
        }
    } else if (ram_word!(game, flare_counter) as i16) > 0i16 {
        if ram_word!(game, flare_counter) == 1u16 {
            set_ram_words! { game, 0u16;
                flare_animation_frame, flare_slow_sparks_anim_frame, flare_fast_sparks_anim_frame,
            }
            set_ram_word!(game.flare_animation_timer = 3u16);
            set_ram_word!(game.flare_slow_sparks_anim_timer = 5u16);
            set_ram_word!(game.flare_fast_sparks_anim_timer = 4u16);
        }
        if ram_word!(game, flare_counter) as i32 - 15i32 & 0x8000i32 == 0 {
            if ram_word!(game, flare_counter) == 16u16 {
                QueueSfx1_Max9(8u16, game);
            }
            let mut v0: u16 = 0u16;
            loop {
                let mut v1: i16 = ((*((ram_ptr!(game, u16; 0xcdc) as *mut u8)
                    .offset(v0 as i32 as isize) as *mut u16))
                    as i16)
                    .wrapping_sub(1i16);
                *((ram_ptr!(game, u16; 0xcdc) as *mut u8).offset(v0 as i32 as isize) as *mut u16) =
                    v1 as u16;
                if v1 < 0i16 {
                    let v2: u16 = (*((ram_ptr!(game, u16; 0xcd6) as *mut u8)
                        .offset(v0 as i32 as isize)
                        as *mut u16))
                        .wrapping_add(1u16);
                    *((ram_ptr!(game, u16; 0xcd6) as *mut u8).offset(v0 as i32 as isize)
                        as *mut u16) = v2;
                    let mut v3: u16 = v2;
                    let r0: *const u8 = RomPtr_90(
                        crate::rtl::rom_table_word!(game, 0x90c481, (v0 as i32 >> 1i32) as isize),
                        game,
                    );
                    let mut v4: i16 = *r0.offset(v2 as isize) as i16;
                    if v4 == 255i16 {
                        *((ram_ptr!(game, u16; 0xcd6) as *mut u8).offset(v0 as i32 as isize)
                            as *mut u16) = 0u16;
                        v3 = 0u16;
                    } else if v4 == 254i16 {
                        let v5: u16 = (*((ram_ptr!(game, u16; 0xcd6) as *mut u8)
                            .offset(v0 as i32 as isize)
                            as *mut u16))
                            .wrapping_sub((*r0.offset((v3 as i32 + 1i32) as isize)) as u16);
                        *((ram_ptr!(game, u16; 0xcd6) as *mut u8).offset(v0 as i32 as isize)
                            as *mut u16) = v5;
                        v3 = v5;
                    }
                    *((ram_ptr!(game, u16; 0xcdc) as *mut u8).offset(v0 as i32 as isize)
                        as *mut u16) = *r0.offset(v3 as isize) as u16;
                }
                let mut v9: u16 = v0;
                DrawFlareAnimationComponent(v0, game);
                if ram_word!(game, flare_counter) as i32 - 30i32 & 0x8000i32 != 0 {
                    break;
                }
                v0 = (v0).wrapping_add(2u16);
                if (((v9) as i16).wrapping_sub(4i16) >= 0i16) {
                    break;
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
    static word_93A225: [u16; 3] = [0, 0x1e, 0x24];
    static word_93A22B: [u16; 3] = [0, 0x2a, 0x30];
    let mut v1: u16 = 0;
    let mut r20: u16 = 0;
    let mut r18: u16 = *(ram_ptr!(game, u16; 0xcd6) as *mut u8).offset(k as i32 as isize) as u16;
    if *ram_ptr!(game; 0xa1e) == 4u8 {
        v1 = word_93A22B[(k as i32 >> 1i32) as usize];
    } else {
        v1 = word_93A225[(k as i32 >> 1i32) as usize];
    }
    let r22: u16 = (r18).wrapping_add(v1);
    let r24: u16 = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
        as *mut SamusPoseParams)
        .offset(ram_word!(game, samus_pose) as isize))
    .y_offset_to_gfx as u16;
    let mut v2: i16 = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
        as *mut SamusPoseParams)
        .offset(ram_word!(game, samus_pose) as isize))
    .direction_shots_fired as i16;
    if v2 != 255i16 && v2 != 16i16 {
        let v3: u16 = (2i32 * (v2 as i32 & 0xfi32)) as u16;
        let old_x: u16 = ram_word!(game, samus_x_pos);
        let old_y: u16 = ram_word!(game, samus_y_pos);
        if (ram_word!(game, ceres_status) as i32 & 0x8000i32) != 0i32 {
            Samus_CalcPos_Mode7(game);
        }
        let v4: i32 = v3 as i32 >> 1i32;
        let mut v5: u16 = 0;
        if *ram_ptr!(game; 0xa1f) == 1u8 {
            r20 = (ram_word!(game, samus_x_pos) as i32
                + kProjectileOriginOffsets2_X[v4 as usize] as i32
                - ram_word!(game, layer1_x_pos) as i32) as u16;
            v5 = (ram_word!(game, samus_y_pos) as i32
                + kProjectileOriginOffsets2_Y[v4 as usize] as i32
                - r24 as i32
                - ram_word!(game, layer1_y_pos) as i32) as u16;
        } else {
            r20 = (ram_word!(game, samus_x_pos) as i32
                + kProjectileOriginOffsets_X[v4 as usize] as i32
                - ram_word!(game, layer1_x_pos) as i32) as u16;
            v5 = (ram_word!(game, samus_y_pos) as i32
                + kProjectileOriginOffsets_Y[v4 as usize] as i32
                - r24 as i32
                - ram_word!(game, layer1_y_pos) as i32) as u16;
        }
        r18 = v5;
        if (v5 as i32 & 0xff00i32) == 0i32 {
            DrawBeamGrappleSpritemap(r22, r20, r18, game);
        }
        if (ram_word!(game, ceres_status) as i32 & 0x8000i32) != 0i32 {
            set_ram_word!(game.samus_y_pos = old_y);
            set_ram_word!(game.samus_x_pos = old_x);
        }
    }
}
pub unsafe fn ClearFlareAnimationState(game: &mut Game) {
    set_ram_words! { game, 0u16;
        flare_animation_frame, flare_slow_sparks_anim_frame, flare_fast_sparks_anim_frame,
        flare_animation_timer, flare_slow_sparks_anim_timer, flare_fast_sparks_anim_timer,
    }
}
pub unsafe fn FireHyperBeam(game: &mut Game) {
    if Samus_CanFireBeam(game) as i32 & 1i32 != 0 {
        let mut v0: u16 = 0u16;
        while ram_word!(game, projectile_damage[(v0 as i32 >> 1i32)]) != 0 {
            v0 = (v0).wrapping_add(2u16);
            if (((v0) as i16).wrapping_sub(10i16) < 0i16) {
                continue;
            }
            v0 = (v0).wrapping_sub(2u16);
            break;
        }
        let mut r20: u16 = v0;
        if InitProjectilePositionDirection(r20, game) as i32 & 1i32 == 0 {
            set_ram_word!(game.projectile_invincibility_timer = 10u16);
            let k: u16 = r20;
            set_ram_word!(game.projectile_type[(r20 as i32 >> 1i32)] = -28648i32 as u16);
            QueueSfx1_Max15(kChargedProjectile_Sfx[8i32 as usize], game);
            set_ram_word!(game.play_resume_charging_beam_sfx = 0u16);
            InitializeProjectile(k, game);
            let v1: i32 = k as i32 >> 1i32;
            set_ram_words! { game, 0u16; projectile_bomb_x_speed[v1], projectile_bomb_y_speed[v1] }
            set_ram_word!(game.projectile_index = k);
            WaveBeam_CheckColl(k, game);
            let v2: u16 = ram_word!(game, projectile_index);
            let v3: i32 = ram_word!(game, projectile_index) as i32 >> 1i32;
            set_ram_word!(game.projectile_damage[v3] = 1000u16);
            set_ram_word!(game.projectile_bomb_pre_instructions[v3] = ((0x90b159i32) as u16));
            r20 = v2;
            SetInitialProjectileSpeed(r20, game);
            set_ram_word!(game.cooldown_timer = 21u16);
            set_ram_word!(game.charged_shot_glow_timer = -32748i32 as u16);
            set_ram_word!(game.flare_animation_frame = 29u16);
            set_ram_words! { game, 5u16;
                flare_slow_sparks_anim_frame, flare_fast_sparks_anim_frame,
            }
            set_ram_words! { game, 3u16;
                flare_animation_timer, flare_slow_sparks_anim_timer, flare_fast_sparks_anim_timer,
            }
            set_ram_word!(game.flare_counter = 0x8000u16);
        }
    }
}
static kCheckBeamCollByDir: [Option<Func_V>; 10] = callback_table! { unsafe Func_V;
    CheckBeamCollByDir_0459, CheckBeamCollByDir_1368, CheckBeamCollByDir_2,
    CheckBeamCollByDir_1368, CheckBeamCollByDir_0459, CheckBeamCollByDir_0459,
    CheckBeamCollByDir_1368, CheckBeamCollByDir_7, CheckBeamCollByDir_1368,
    CheckBeamCollByDir_0459,
};
pub unsafe fn CheckBeamCollByDir(k: u16, game: &mut Game) {
    kCheckBeamCollByDir
        [(ram_word!(game, projectile_dir[(k as i32 >> 1i32)]) as i32 & 0xfi32) as usize]
        .expect("non-null function pointer")(game);
}
pub unsafe fn CheckBeamCollByDir_0459(game: &mut Game) {
    BlockCollNoWaveBeamVert(ram_word!(game, projectile_index), game);
}
pub unsafe fn CheckBeamCollByDir_1368(game: &mut Game) {
    let v0: u16 = ram_word!(game, projectile_index);
    if BlockCollNoWaveBeamHoriz(ram_word!(game, projectile_index), game) as i32 & 1i32 == 0 {
        BlockCollNoWaveBeamVert(v0, game);
    }
}
pub unsafe fn CheckBeamCollByDir_2(game: &mut Game) {
    BlockCollNoWaveBeamHoriz(ram_word!(game, projectile_index), game);
}
pub unsafe fn CheckBeamCollByDir_7(game: &mut Game) {
    let v0: u16 = ram_word!(game, projectile_index);
    *(ram_ptr!(game, u16; 0xbdc))
        .offset((ram_word!(game, projectile_index) as i32 >> 1i32) as isize) = -1i32 as u16;
    BlockCollNoWaveBeamHoriz(v0, game);
}
static kWaveBeam_CheckColl_Funcs: [Option<Func_V>; 10] = callback_table! { unsafe Func_V;
    WaveBeam_CheckColl_0459, WaveBeam_CheckColl_1368, WaveBeam_CheckColl_2,
    WaveBeam_CheckColl_1368, WaveBeam_CheckColl_0459, WaveBeam_CheckColl_0459,
    WaveBeam_CheckColl_1368, WaveBeam_CheckColl_7, WaveBeam_CheckColl_1368,
    WaveBeam_CheckColl_0459,
};
pub unsafe fn WaveBeam_CheckColl(k: u16, game: &mut Game) {
    kWaveBeam_CheckColl_Funcs
        [(ram_word!(game, projectile_dir[(k as i32 >> 1i32)]) as i32 & 0xfi32) as usize]
        .expect("non-null function pointer")(game);
}
pub unsafe fn WaveBeam_CheckColl_0459(game: &mut Game) {
    BlockCollWaveBeamVert(ram_word!(game, projectile_index), game);
}
pub unsafe fn WaveBeam_CheckColl_1368(game: &mut Game) {
    let v0: u16 = ram_word!(game, projectile_index);
    if BlockCollWaveBeamHoriz(ram_word!(game, projectile_index), game) as i32 & 1i32 == 0 {
        BlockCollWaveBeamVert(v0, game);
    }
}
pub unsafe fn WaveBeam_CheckColl_2(game: &mut Game) {
    BlockCollWaveBeamHoriz(ram_word!(game, projectile_index), game);
}
pub unsafe fn WaveBeam_CheckColl_7(game: &mut Game) {
    let v0: u16 = ram_word!(game, projectile_index);
    *(ram_ptr!(game, u16; 0xbdc))
        .offset((ram_word!(game, projectile_index) as i32 >> 1i32) as isize) = -1i32 as u16;
    BlockCollWaveBeamHoriz(v0, game);
}
pub unsafe fn ProjectileReflection(r20: u16, game: &mut Game) {
    let v0: u16 = r20;
    let v1: i32 = r20 as i32 >> 1i32;
    let v2: u16 = ram_word!(game, projectile_type[v1]);
    if (v2 as i32 & 0x100i32) != 0i32 {
        InitializeProjectile(r20, game);
        set_ram_word!(game.projectile_bomb_pre_instructions[v1] = ((0x90af68i32) as u16));
        set_ram_word!(game.projectile_variables[v1] = 240u16);
    } else if (v2 as i32 & 0x200i32) != 0i32 {
        let k: u16 = r20;
        ClearProjectile(
            *((ram_ptr!(game, u16; 0xc7c)).offset(v1 as isize) as *mut u8).offset(0) as u16,
            game,
        );
        InitializeProjectile(k, game);
        let v3: i32 = k as i32 >> 1i32;
        set_ram_word!(game.projectile_bomb_pre_instructions[v3] = ((0x90afe5i32) as u16));
        set_ram_word!(game.projectile_variables[v3] = 240u16);
    } else {
        SetInitialProjectileSpeed(r20, game);
        InitializeProjectile(v0, game);
        set_ram_word!(
            game.projectile_bomb_pre_instructions[v1] = kFireChargedBeam_Funcs
                [(ram_word!(game, projectile_type[v1]) as i32 & 0xfi32) as usize]
        );
    };
}
pub unsafe fn HudSelectionHandler_MissilesOrSuperMissiles(game: &mut Game) {
    let mut current_block: u64;
    if (ram_word!(game, button_config_shoot_x) as i32 & ram_word!(game, joypad1_newkeys) as i32)
        == 0i32
        && (ram_word!(game, button_config_shoot_x) as i32
            & ram_word!(game, joypad1_newinput_samusfilter) as i32)
            == 0i32
        || Samus_CanFireSuperMissile(game) as i32 & 1i32 == 0
    {
        return;
    }
    if ram_word!(game, hud_item_index) != 2u16 {
        if ram_word!(game, samus_missiles) != 0 {
            current_block = 7351195479953500246;
        } else {
            current_block = 12512720045134471270;
        }
    } else if ram_word!(game, samus_super_missiles) == 0 {
        current_block = 12512720045134471270;
    } else {
        current_block = 7351195479953500246;
    }
    if current_block == 7351195479953500246 {
        let mut v0: u16 = 0u16;
        loop {
            if (ram_word!(game, projectile_damage[(v0 as i32 >> 1i32)]) == 0) {
                current_block = 13536709405535804910;
                break;
            }
            v0 = (v0).wrapping_add(2u16);
            if ((v0) as i16).wrapping_sub(10i16) >= 0i16 {
                current_block = 12512720045134471270;
                break;
            }
        }
        if current_block == 12512720045134471270 {
        } else {
            let r20: u16 = v0;
            if InitProjectilePositionDirection(r20, game) as i32 & 1i32 == 0 {
                set_ram_word!(game.projectile_invincibility_timer = 20u16);
                if ram_word!(game, hud_item_index) == 2u16 {
                    update_ram_word!(game.samus_super_missiles, wrapping_sub(1));
                } else {
                    update_ram_word!(game.samus_missiles, wrapping_sub(1));
                }
                let v1: i32 = r20 as i32 >> 1i32;
                set_ram_word!(game.projectile_timers[v1] = 4u16);
                let v3: u16 = ram_word!(game, hud_item_index);
                let r18: u16 = u16::swap_bytes(ram_word!(game, hud_item_index));
                update_ram_word!(game.projectile_type[v1], |fresh80| (fresh80 as i32
                    | (r18 as i32 | 0x8000i32))
                    as u16);
                let v4: u16 = (2i32 * (v3 as i32 & 0xfi32)) as u16;
                if ram_word!(game, cinematic_function) == 0 {
                    QueueSfx1_Max6(kNonBeamProjectile_Sfx[(v4 as i32 >> 1i32) as usize], game);
                }
                InitializeProjectileSpeedOfType(r20, game);
                InitializeProjectile(r20, game);
                let mut v7: u16 = ram_word!(game, projectile_type[v1]);
                if (v7 as i32 & 0x200i32) != 0i32 {
                    set_ram_word!(
                        game.projectile_bomb_pre_instructions[v1] = ((0x90afe5i32) as u16)
                    );
                } else {
                    set_ram_word!(
                        game.projectile_bomb_pre_instructions[v1] = ((0x90af68i32) as u16)
                    );
                }
                set_ram_word!(
                    game.cooldown_timer = kNonBeamProjectileCooldowns
                        [(*(&raw mut v7 as *mut u8).offset(1isize,) as i32 & 0xfi32) as usize]
                        as u16
                );
                if ram_word!(game, samus_auto_cancel_hud_item_index) != 0 {
                    set_ram_words! { game, 0u16; hud_item_index, samus_auto_cancel_hud_item_index }
                    return;
                }
                if ram_word!(game, hud_item_index) == 2u16 {
                    if ram_word!(game, samus_super_missiles) != 0 {
                        return;
                    }
                } else if ram_word!(game, samus_missiles) != 0 {
                    return;
                }
                set_ram_word!(game.hud_item_index = 0u16);
            }
            return;
        }
    }
    update_ram_word!(game.projectile_counter, wrapping_sub(1));
}
pub unsafe fn Missile_Func2(game: &mut Game) {
    let mut v0: u16 = 0u16;
    while ram_word!(game, projectile_damage[(v0 as i32 >> 1i32)]) != 0 {
        v0 = (v0).wrapping_add(2u16);
        if ((v0) as i16).wrapping_sub(10i16) >= 0i16 {
            return;
        }
    }
    let r20: u16 = v0;
    let v1: i32 = v0 as i32 >> 1i32;
    update_ram_word!(game.projectile_type[v1], |fresh66| (fresh66) | (0x8200u16));
    let v2: i32 = ram_word!(game, projectile_index) as i32 >> 1i32;
    set_ram_word!(game.projectile_x_pos[v1] = ram_word!(game, projectile_x_pos[v2]));
    set_ram_word!(game.projectile_y_pos[v1] = ram_word!(game, projectile_y_pos[v2]));
    set_ram_word!(game.projectile_dir[v1] = ram_word!(game, projectile_dir[v2]));
    InitProjectilePositionDirection(r20, game);
    InitializeInstrForSuperMissile(v0, game);
    set_ram_word!(game.projectile_bomb_pre_instructions[v1] = ((0x90b075i32) as u16));
    *(ram_ptr!(game, u16; 0xc7c))
        .offset((ram_word!(game, projectile_index) as i32 >> 1i32) as isize) = (v0 as i32
        + (*(ram_ptr!(game, u16; 0xc7c))
            .offset((ram_word!(game, projectile_index) as i32 >> 1i32) as isize) as i32
            & 0xff00i32))
        as u16;
    update_ram_word!(game.projectile_counter, wrapping_add(1));
}
pub unsafe fn HudSelectionHandler_MorphBall(game: &mut Game) {
    if (ram_word!(game, button_config_shoot_x) as i32 & ram_word!(game, joypad1_lastkeys) as i32)
        != 0i32
    {
        if ram_word!(game, hud_item_index) == 3u16 {
            if (ram_word!(game, power_bomb_flag) as i32 & 0x8000i32) == 0i32
                && HudSelectionHandler_MorphBall_Helper2(game) as i32 & 1i32 != 0
                && ram_word!(game, samus_power_bombs) != 0
            {
                if (update_ram_word!(game.samus_power_bombs, wrapping_sub(1), reread) as i32
                    & 0x8000i32)
                    == 0i32
                {
                    set_ram_word!(game.power_bomb_flag = -1i32 as u16);
                    let mut v2: u16 = 10u16;
                    while ram_word!(game, projectile_type[(v2 as i32 >> 1i32)]) != 0 {
                        v2 = (v2).wrapping_add(2u16);
                        if (((v2) as i16).wrapping_sub(20i16) < 0i16) {
                            continue;
                        }
                        v2 = (v2).wrapping_sub(2u16);
                        break;
                    }
                    let r18: u16 = u16::swap_bytes(ram_word!(game, hud_item_index));
                    let v4: i32 = v2 as i32 >> 1i32;
                    let mut v5: u16 = (r18) | (ram_word!(game, projectile_type[v4]));
                    set_ram_word!(game.projectile_type[v4] = v5);
                    let v6: u8 = *(&raw mut v5 as *mut u8).offset(1isize);
                    set_ram_word!(game.projectile_dir[v4] = 0u16);
                    set_ram_word!(game.projectile_x_pos[v4] = ram_word!(game, samus_x_pos));
                    set_ram_word!(game.projectile_y_pos[v4] = ram_word!(game, samus_y_pos));
                    set_ram_word!(game.projectile_variables[v4] = 60u16);
                    InitializeInstrForMissile(v2, game);
                    set_ram_word!(
                        game.projectile_bomb_pre_instructions[v4] = ((0x90b0aei32) as u16)
                    );
                    set_ram_word!(
                        game.cooldown_timer =
                            kNonBeamProjectileCooldowns[(v6 as i32 & 0xfi32) as usize] as u16
                    );
                    if ram_word!(game, samus_auto_cancel_hud_item_index) != 0 {
                        set_ram_words! { game, 0u16;
                            hud_item_index, samus_auto_cancel_hud_item_index,
                        }
                    } else if ram_word!(game, hud_item_index) == 3u16
                        && ram_word!(game, samus_power_bombs) == 0
                    {
                        set_ram_word!(game.hud_item_index = 0u16);
                    }
                }
            }
        } else if HudSelectionHandler_MorphBall_Helper(game) as i32 & 1i32 != 0 {
            let mut v0: u16 = 10u16;
            while ram_word!(game, projectile_type[(v0 as i32 >> 1i32)]) != 0 {
                v0 = (v0).wrapping_add(2u16);
                if (((v0) as i16).wrapping_sub(20i16) < 0i16) {
                    continue;
                }
                v0 = (v0).wrapping_sub(2u16);
                break;
            }
            let v1: i32 = v0 as i32 >> 1i32;
            set_ram_word!(game.projectile_type[v1] = 1280u16);
            set_ram_word!(game.projectile_dir[v1] = 0u16);
            set_ram_word!(game.projectile_x_pos[v1] = ram_word!(game, samus_x_pos));
            set_ram_word!(game.projectile_y_pos[v1] = ram_word!(game, samus_y_pos));
            set_ram_word!(game.projectile_variables[v1] = 60u16);
            InitializeInstrForMissile(v0, game);
            set_ram_word!(game.projectile_bomb_pre_instructions[v1] = ((0x90b099i32) as u16));
            set_ram_word!(game.cooldown_timer = kNonBeamProjectileCooldowns[5i32 as usize] as u16);
        }
    } else if ram_word!(game, flare_counter) != 0 {
        QueueSfx1_Max9(2u16, game);
        set_ram_word!(game.flare_counter = 0u16);
        ClearFlareAnimationState(game);
        Samus_LoadSuitPalette(game);
    }
}
pub unsafe fn HudSelectionHandler_MorphBall_Helper(game: &mut Game) -> u8 {
    if (ram_word!(game, equipped_items) as i32 & 0x1000i32) != 0i32 {
        if ram_word!(game, flare_counter) as i32 - 60i32 & 0x8000i32 != 0
            || ram_word!(game, bomb_counter) != 0u16
        {
            return HudSelectionHandler_MorphBall_Helper2(game);
        }
        if (ram_word!(game, joypad1_lastkeys) as i32 & kButton_Down as i32) != 0i32
            && (ram_word!(game, bomb_spread_charge_timeout_counter) as i32 & 0xc0i32) - 192i32
                & 0x8000i32
                != 0
        {
            update_ram_word!(game.bomb_spread_charge_timeout_counter, wrapping_add(1));
        } else {
            BombSpread(game);
            Samus_LoadSuitPalette(game);
            QueueSfx1_Max9(2u16, game);
        }
    }
    0u8
}
pub unsafe fn HudSelectionHandler_MorphBall_Helper2(game: &mut Game) -> u8 {
    if (ram_word!(game, button_config_shoot_x) as i32 & ram_word!(game, joypad1_newkeys) as i32)
        == 0i32
        || ram_word!(game, bomb_counter) != 0u16
            && (ram_word!(game, bomb_counter) as i32 - 5i32 & 0x8000i32 == 0
                || (ram_word!(game, cooldown_timer) as u8) != 0u8)
    {
        if ram_word!(game, flare_counter) != 0 {
            QueueSfx1_Max9(2u16, game);
            set_ram_word!(game.flare_counter = 0u16);
            ClearFlareAnimationState(game);
            Samus_LoadSuitPalette(game);
        }
        0u8
    } else {
        update_ram_word!(game.cooldown_timer, wrapping_add(1));
        update_ram_word!(game.bomb_counter, wrapping_add(1));
        1u8
    }
}
pub unsafe fn Bomb_Func2(game: &mut Game) {
    let v0: u16 = ram_word!(game, projectile_index);
    let v1: i32 = ram_word!(game, projectile_index) as i32 >> 1i32;
    let v2: u16 = ram_word!(game, projectile_variables[v1]);
    if v2 != 0 {
        let v3: u16 = (v2).wrapping_sub(1u16);
        set_ram_word!(game.projectile_variables[v1] = v3);
        if v3 != 0 {
            if v3 == 15u16 {
                update_ram_word!(
                    game.projectile_bomb_instruction_ptr[v1],
                    wrapping_add(28u16)
                );
            }
        } else {
            QueueSfx2_Max6(8u16, game);
            InitializeBombExplosion(v0, game);
        }
    }
}
pub unsafe fn PowerBomb_Func3(game: &mut Game) {
    let v0: u16 = ram_word!(game, projectile_index);
    let v1: i32 = ram_word!(game, projectile_index) as i32 >> 1i32;
    let v2: u16 = ram_word!(game, projectile_variables[v1]);
    if v2 != 0 {
        let v3: u16 = (v2).wrapping_sub(1u16);
        set_ram_word!(game.projectile_variables[v1] = v3);
        if v3 != 0 {
            if v3 == 15u16 {
                update_ram_word!(
                    game.projectile_bomb_instruction_ptr[v1],
                    wrapping_add(28u16)
                );
            }
        } else {
            set_ram_word!(game.power_bomb_explosion_x_pos = ram_word!(game, projectile_x_pos[v1]));
            set_ram_word!(game.power_bomb_explosion_y_pos = ram_word!(game, projectile_y_pos[v1]));
            EnableHdmaObjects(game);
            SpawnPowerBombExplosion(game);
            set_ram_word!(game.projectile_variables[(v0 as i32 >> 1i32)] = -1i32 as u16);
        }
    } else if ram_word!(game, power_bomb_flag) == 0 {
        ClearProjectile(ram_word!(game, projectile_index), game);
    }
}
static kRunSwitchedToHudHandler: [Option<Func_U8>; 6] = callback_table! { unsafe Func_U8;
    SwitchToHudHandler_Nothing, SwitchToHudHandler_Missiles, SwitchToHudHandler_SuperMissiles,
    SwitchToHudHandler_PowerBombs, SwitchToHudHandler_Grapple, SwitchToHudHandler_Xray,
};
pub unsafe fn HandleSwitchingHudSelection(game: &mut Game) {
    let mut current_block: u64;
    let mut v0: u16 = 0;
    let mut r22: u16 = 0u16;
    let r18: u16 = ram_word!(game, hud_item_index);
    if (ram_word!(game, button_config_itemcancel_y) as i32
        & ram_word!(game, joypad1_newkeys) as i32)
        != 0i32
    {
        set_ram_word!(game.samus_auto_cancel_hud_item_index = 0u16);
        current_block = 5182151076943748346;
    } else {
        r22 = ((ram_word!(game, button_config_itemcancel_y) as i32
            & ram_word!(game, joypad1_lastkeys) as i32)
            != 0i32) as u16;
        if (ram_word!(game, button_config_itemswitch) as i32
            & ram_word!(game, joypad1_newkeys) as i32)
            == 0i32
        {
            current_block = 16684771485965707775;
        } else {
            v0 = (ram_word!(game, hud_item_index)).wrapping_add(1u16);
            if ram_word!(game, hud_item_index) as i32 - 5i32 & 0x8000i32 == 0 {
                current_block = 5182151076943748346;
            } else {
                current_block = 11963831534727063849;
            }
        }
    }
    if current_block == 5182151076943748346 {
        v0 = 0u16;
        current_block = 11963831534727063849;
    }
    if current_block == 11963831534727063849 {
        set_ram_word!(game.hud_item_index = v0);
        while kRunSwitchedToHudHandler[v0 as usize].expect("non-null function pointer")(game) as i32
            & 1i32
            != 0
        {
            v0 = (ram_word!(game, hud_item_index)).wrapping_add(1u16);
            set_ram_word!(game.hud_item_index = v0);
            if v0 as i32 - 6i32 & 0x8000i32 == 0 {
                v0 = 0u16;
                set_ram_word!(game.hud_item_index = 0u16);
            }
        }
        if r22 != 0 {
            set_ram_word!(game.samus_auto_cancel_hud_item_index = ram_word!(game, hud_item_index));
        } else {
            set_ram_word!(game.samus_auto_cancel_hud_item_index = 0u16);
        }
    }
    if ram_word!(game, hud_item_index) == r18 {
        let mut v1: u16 = (ram_word!(game, hud_item_changed_this_frame)).wrapping_add(1u16);
        if ram_word!(game, hud_item_changed_this_frame) as i32 - 2i32 & 0x8000i32 == 0 {
            v1 = 2u16;
        }
        set_ram_word!(game.hud_item_changed_this_frame = v1);
    } else {
        set_ram_word!(game.hud_item_changed_this_frame = 1u16);
    };
}
pub unsafe fn SwitchToHudHandler_Nothing(game: &mut Game) -> u8 {
    set_ram_word!(game.flare_counter = 0u16);
    ClearFlareAnimationState(game);
    Samus_LoadSuitPalette(game);
    0u8
}
pub unsafe fn SwitchToHudHandler_Missiles(game: &mut Game) -> u8 {
    if ram_word!(game, samus_missiles) == 0 {
        return 1u8;
    }
    set_ram_word!(game.flare_counter = 0u16);
    ClearFlareAnimationState(game);
    Samus_LoadSuitPalette(game);
    0u8
}
pub unsafe fn SwitchToHudHandler_SuperMissiles(game: &mut Game) -> u8 {
    if ram_word!(game, samus_super_missiles) == 0 {
        return 1u8;
    }
    set_ram_word!(game.flare_counter = 0u16);
    ClearFlareAnimationState(game);
    Samus_LoadSuitPalette(game);
    0u8
}
pub unsafe fn SwitchToHudHandler_PowerBombs(game: &mut Game) -> u8 {
    if ram_word!(game, samus_power_bombs) == 0 {
        return 1u8;
    }
    set_ram_word!(game.flare_counter = 0u16);
    ClearFlareAnimationState(game);
    Samus_LoadSuitPalette(game);
    0u8
}
pub unsafe fn SwitchToHudHandler_Grapple(game: &mut Game) -> u8 {
    if (ram_word!(game, equipped_items) as i32 & 0x4000i32) == 0i32 {
        return 1u8;
    }
    if ram_word!(game, grapple_beam_function) as i32 == 0x9bc4f0i32 & 0xffffi32 {
        Samus_LoadSuitPalette(game);
        set_ram_word!(game.flare_counter = 0u16);
        ClearFlareAnimationState(game);
        set_ram_word!(game.grapple_beam_function = ((0x9bc4f0i32) as u16));
    }
    0u8
}
pub unsafe fn SwitchToHudHandler_Xray(game: &mut Game) -> u8 {
    if (ram_word!(game, equipped_items) as i32 & 0x8000i32) == 0i32 {
        return 1u8;
    }
    set_ram_word!(game.flare_counter = 0u16);
    ClearFlareAnimationState(game);
    Samus_LoadSuitPalette(game);
    0u8
}
pub unsafe fn HandleArmCannonOpenState(game: &mut Game) {
    if *ram_ptr!(game; 0xaa7) != 0u8 || UpdateArmCannonIsOpenFlag(game) as i32 & 1i32 != 0 {
        AdvanceArmCannonFrame(game);
    }
    set_ram_word!(
        game.arm_cannon_drawing_mode = *RomPtr_90(
            crate::rtl::rom_table_word!(game, 0x90c7df, ram_word!(game, samus_pose) as isize),
            game,
        )
        .offset(1) as u16
    );
}
pub unsafe fn UpdateArmCannonIsOpenFlag(game: &mut Game) -> u8 {
    static kFlagShouldArmCannonBeOpen: [i8; 6] = [0, 1, 1, 0, 1, 0];
    if ram_word!(game, hud_item_changed_this_frame) as i32 - 2i32 & 0x8000i32 != 0 {
        return 0u8;
    }
    let r18: u16 = kFlagShouldArmCannonBeOpen[ram_word!(game, hud_item_index) as usize] as u16;
    if ((*ram_ptr!(game; 0xaa6)) as u16) == r18 {
        return 0u8;
    }
    if r18 != 0 {
        set_ram_word!(game.arm_cannon_frame = 0u16);
    } else {
        set_ram_word!(game.arm_cannon_frame = 4u16);
    }
    *(ram_ptr!(game, u16; 0xaa6)) = (r18) | (0x100u16);
    1u8
}
pub unsafe fn AdvanceArmCannonFrame(game: &mut Game) {
    if *ram_ptr!(game; 0xaa6) != 0 {
        if ram_word!(game, arm_cannon_frame) as i32 - 2i32 & 0x8000i32 != 0 {
            update_ram_word!(game.arm_cannon_frame, wrapping_add(1));
            return;
        }
        set_ram_word!(game.arm_cannon_frame = 3u16);
    } else {
        if ram_word!(game, arm_cannon_frame) != 1u16
            && ((ram_word!(game, arm_cannon_frame)) as i16).wrapping_sub(1i16) >= 0i16
        {
            update_ram_word!(game.arm_cannon_frame, wrapping_sub(1));
            return;
        }
        set_ram_word!(game.arm_cannon_frame = 0u16);
    }
    *(ram_ptr!(game, u16; 0xaa6)) = *ram_ptr!(game; 0xaa6) as u16;
}
pub unsafe fn Samus_ArmCannon_Draw(game: &mut Game) {
    static kDrawArmCannon_Char: [u16; 10] = [
        0x281f, 0x281f, 0x281f, 0x681f, 0xa81f, 0xe81f, 0x281f, 0x681f, 0x681f, 0x681f,
    ];
    let mut v3: u16 = 0;
    let mut r22: u16 = 0;
    if ram_word!(game, arm_cannon_frame) != 0u16
        && (ram_word!(game, samus_invincibility_timer) == 0
            || (ram_word!(game, nmi_frame_counter_word) as i32 & 1i32) == 0i32)
    {
        let v0: u16 =
            crate::rtl::rom_table_word!(game, 0x90c7df, ram_word!(game, samus_pose) as isize);
        let v1: *const u8 = RomPtr_90(v0, game);
        let v2: i16 = *v1 as i16;
        if (v2 as i32 & 0x80i32) != 0i32 {
            if ram_word!(game, samus_anim_frame) != 0 {
                v3 = (2i32 * (*v1.offset(2) as i32 & 0x7fi32)) as u16;
            } else {
                v3 = (2i32 * (*v1 as i32 & 0x7fi32)) as u16;
            }
            r22 = (v0).wrapping_add(4u16);
        } else {
            v3 = (2u16).wrapping_mul((v2) as u16);
            r22 = (v0).wrapping_add(2u16);
        }
        let r24: u16 = kDrawArmCannon_Char[(v3 as i32 >> 1i32) as usize];
        let v4: *const u8 = RomPtr_90(
            (r22 as i32 + 2i32 * ram_word!(game, samus_anim_frame) as i32) as u16,
            game,
        );
        let r18: u16 = *v4.offset(0) as i8 as u16;
        let r20: u16 = *v4.offset(1) as i8 as u16;
        r22 = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629)) as *mut SamusPoseParams)
            .offset(ram_word!(game, samus_pose) as isize))
        .y_offset_to_gfx as u16;
        let v7: u16 = ram_word!(game, oam_next_ptr);
        let v8: i16 = (r18 as i32 + ram_word!(game, samus_x_pos) as i32
            - ram_word!(game, layer1_x_pos) as i32) as i16;
        if v8 >= 0i16 && v8 as i32 - 256i32 & 0x8000i32 != 0 {
            let v9: *mut OamEnt = gOamEnt(ram_word!(game, oam_next_ptr) as i32, game);
            (*v9).xcoord = v8 as u8;
            let v10: i16 = (r20 as i32 + ram_word!(game, samus_y_pos) as i32
                - r22 as i32
                - ram_word!(game, layer1_y_pos) as i32) as i16;
            if v10 >= 0i16 && v10 as i32 - 256i32 & 0x8000i32 != 0 {
                (*v9).ycoord = v10 as u8;
                *(&raw mut (*v9).charnum as *mut u16) = r24;
                set_ram_word!(game.oam_next_ptr = (v7).wrapping_add(4u16));
            }
        }
        let v11: *const u8 = RomPtr_90(
            crate::rtl::rom_table_word!(game, 0x90c7df, ram_word!(game, samus_pose) as isize),
            game,
        );
        let mut v12: u8 = *v11;
        if (v12 as i32 & 0x80i32) != 0i32 {
            v12 = (if ram_word!(game, samus_anim_frame) != 0u16 {
                *v11.offset(2) as i32
            } else {
                *v11.offset(0) as i32
            }) as u8;
        }
        let v13: u16 = (crate::rtl::rom_table_word!(game, 0x90c7a5, (v12 as i32 & 0x7fi32) as isize)
            as i32
            + 2i32 * ram_word!(game, arm_cannon_frame) as i32) as u16;
        let mut v14: u16 = ram_word!(game, vram_write_queue_tail);
        (*gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game)).size = 32u16;
        v14 = (v14).wrapping_add(2u16);
        let v15: u16 = *(RomPtr_90(v13, game) as *mut u16);
        (*gVramWriteEntry(v14 as i32, game)).size = v15;
        v14 = (v14).wrapping_add(2u16);
        let fresh15 = v14;
        v14 = v14.wrapping_add(1);
        *(&raw mut (*(gVramWriteEntry as unsafe fn(i32, &mut Game) -> *mut VramWriteEntry)(
            fresh15 as i32,
            game,
        ))
        .size as *mut u8)
            .offset(0) = -102i32 as u8;
        (*gVramWriteEntry(v14 as i32, game)).size = addr_unk_6061F0 as u16;
        set_ram_word!(game.vram_write_queue_tail = (v14).wrapping_add(2u16));
    }
}
unsafe fn Projectile_SinLookup_Inner(k: u16, r24: u16) -> u16 {
    let mut prod: u16 = Mult8x8(
        *((&raw const kSinCosTable8bit_Sext as *const i16).offset(64) as *mut u8)
            .offset(k as i32 as isize),
        r24 as u8,
    );
    let r18: u16 = (prod as i32 >> 8i32) as u16;
    prod = Mult8x8(
        *((&raw const kSinCosTable8bit_Sext as *const i16).offset(64) as *mut u8)
            .offset(k as i32 as isize)
            .offset(1),
        r24 as u8,
    );
    (r18).wrapping_add(prod)
}
pub unsafe fn Projectile_SinLookup(j: u16, a: u16) -> Point16U {
    let mut v2: u16 = 0;
    let mut v4: u16 = 0;
    let r24: u16 = a;
    let r26: u16 = j;
    if j as i32 - 128i32 & 0x8000i32 != 0 {
        v2 = Projectile_SinLookup_Inner((2u16).wrapping_mul(j), r24);
    } else {
        v2 = -(Projectile_SinLookup_Inner(
            (2u16).wrapping_mul((((j) as u8).wrapping_add(0x80u8)) as u16),
            r24,
        ) as i32) as u16;
    }
    let v3: i16 = (((r26) as u8).wrapping_sub(64u8)) as i16;
    if v3 as i32 - 128i32 & 0x8000i32 != 0 {
        v4 = Projectile_SinLookup_Inner((2u16).wrapping_mul((v3) as u16), r24);
    } else {
        v4 = -(Projectile_SinLookup_Inner(
            (2u16).wrapping_mul((((v3) as u8).wrapping_add(0x80u8)) as u16),
            r24,
        ) as i32) as u16;
    }
    Point16U { x: v2, y: v4 }
}
static kCostOfSbaInPowerBombs: [u16; 12] = [0, 1, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0];
static kFireSbaFuncs: [Option<Func_V_A>; 12] = callback_table! { unsafe Func_V_A;
    FireSba_ClearCarry, FireSba_FireWave, FireSba_FireIce, FireSba_ClearCarry,
    FireSba_FireSpazer, FireSba_ClearCarry, FireSba_ClearCarry, FireSba_ClearCarry,
    FireSba_FirePlasma, FireSba_ClearCarry, FireSba_ClearCarry, FireSba_ClearCarry,
};
pub unsafe fn FireSba(game: &mut Game) -> u8 {
    if ram_word!(game, hud_item_index) != 3u16 {
        return 0u8;
    }
    let v1: u16 = (2i32 * (ram_word!(game, equipped_beams) as i32 & 0xfi32)) as u16;
    let mut v2: i16 = ((ram_word!(game, samus_power_bombs)) as i16)
        .wrapping_sub((kCostOfSbaInPowerBombs[(v1 as i32 >> 1i32) as usize]) as i16);
    if v2 < 0i16 {
        v2 = 0i16;
    }
    set_ram_word!(game.samus_power_bombs = v2 as u16);
    let rv: u8 =
        kFireSbaFuncs[(v1 as i32 >> 1i32) as usize].expect("non-null function pointer")(game);
    if ram_word!(game, samus_power_bombs) == 0 {
        set_ram_words! { game, 0u16; hud_item_index, samus_auto_cancel_hud_item_index }
    }
    rv
}
pub fn FireSba_ClearCarry(game: &mut Game) -> u8 {
    0u8
}
pub unsafe fn FireSba_FireWave(game: &mut Game) -> u8 {
    static kFireSba_FireWave_X: [i16; 4] = [128, 128, -128, -128];
    static kFireSba_FireWave_Y: [i16; 4] = [128, -128, -128, 128];
    let mut i: i32 = 6i32;
    while i >= 0i32 {
        let v1: i32 = i >> 1i32;
        set_ram_word!(game.projectile_timers[v1] = 4u16);
        set_ram_word!(
            game.projectile_type[v1] =
                (ram_word!(game, equipped_beams) as i32 & 0x100fi32 | 0x8010i32) as u16
        );
        set_ram_word!(game.projectile_dir[v1] = 0u16);
        set_ram_word!(game.projectile_bomb_pre_instructions[v1] = ((0x90da08i32) as u16));
        set_ram_word!(game.projectile_bomb_y_speed[v1] = 600u16);
        set_ram_words! { game, 0u16;
            projectile_bomb_x_speed[v1], projectile_variables[v1], projectile_bomb_x_subpos[v1],
            projectile_bomb_y_subpos[v1],
        }
        set_ram_word!(
            game.projectile_x_pos[v1] = ((kFireSba_FireWave_X[v1 as usize]) as u16)
                .wrapping_add(ram_word!(game, samus_x_pos))
        );
        set_ram_word!(
            game.projectile_y_pos[v1] = ((kFireSba_FireWave_Y[v1 as usize]) as u16)
                .wrapping_add(ram_word!(game, samus_y_pos))
        );
        InitializeSbaProjectile(i as u16, game);
        i -= 2i32;
    }
    set_ram_word!(game.projectile_counter = 4u16);
    set_ram_word!(
        game.cooldown_timer = kProjectileCooldown_Uncharged
            [(ram_word!(game, projectile_type[0]) as i32 & 0x3fi32) as usize]
            as u16
    );
    set_ram_word!(game.used_for_sba_attacksB60 = 4u16);
    QueueSfx1_Max6(0x28u16, game);
    1u8
}
pub unsafe fn FireSba_FireIce(game: &mut Game) -> u8 {
    static kIcePlasmaSbaProjectileOriginAngles: [u16; 8] =
        [0, 0x40, 0x80, 0xc0, 0x20, 0x60, 0xa0, 0xe0];
    if ram_word!(game, projectile_bomb_pre_instructions[0]) as i32 == 0x90cf7ai32 & 0xffffi32
        || ram_word!(game, projectile_bomb_pre_instructions[0]) as i32 == 0x90cf09i32 & 0xffffi32
    {
        return 0u8;
    }
    let mut i: i32 = 6i32;
    while i >= 0i32 {
        let v2: i32 = i >> 1i32;
        set_ram_word!(game.projectile_timers[v2] = 4u16);
        set_ram_word!(
            game.projectile_type[v2] =
                (ram_word!(game, equipped_beams) as i32 & 0x100fi32 | 0x8010i32) as u16
        );
        set_ram_word!(game.projectile_dir[v2] = 0u16);
        set_ram_word!(game.projectile_bomb_pre_instructions[v2] = ((0x90cf09i32) as u16));
        set_ram_word!(
            game.projectile_variables[v2] = kIcePlasmaSbaProjectileOriginAngles[v2 as usize]
        );
        set_ram_word!(game.projectile_bomb_y_speed[v2] = 600u16);
        InitializeProjectile(i as u16, game);
        i -= 2i32;
    }
    set_ram_word!(game.projectile_counter = 4u16);
    set_ram_word!(
        game.cooldown_timer = kProjectileCooldown_Uncharged
            [(ram_word!(game, projectile_type[0]) as i32 & 0x3fi32) as usize]
            as u16
    );
    if *ram_ptr!(game; 0xa1e) == 4u8 {
        set_ram_word!(game.used_for_sba_attacksB60 = -4i32 as u16);
    } else {
        set_ram_word!(game.used_for_sba_attacksB60 = 4u16);
    }
    QueueSfx1_Max6(0x23u16, game);
    1u8
}
pub unsafe fn FireSba_FireSpazer(game: &mut Game) -> u8 {
    static kFireSpazer_Timer: [i16; 4] = [0, 0, 4, 4];
    static kFireSpazer_Yspeed: [i16; 4] = [4, -4, 4, -4];
    let mut i: i32 = 6i32;
    while i >= 0i32 {
        let v1: i32 = i >> 1i32;
        set_ram_word!(game.projectile_timers[v1] = kFireSpazer_Timer[v1 as usize] as u16);
        set_ram_word!(game.projectile_dir[v1] = 5u16);
        set_ram_word!(game.projectile_bomb_pre_instructions[v1] = ((0x90db06i32) as u16));
        set_ram_word!(game.projectile_bomb_x_speed[v1] = 40u16);
        set_ram_word!(game.projectile_bomb_y_speed[v1] = kFireSpazer_Yspeed[v1 as usize] as u16);
        set_ram_words! { game, 0u16;
            projectile_variables[v1], projectile_unk_A[v1], projectile_bomb_x_subpos[v1],
            projectile_bomb_y_subpos[v1],
        }
        if ((i - 4i32) as i16) >= 0i16 {
            set_ram_word!(game.projectile_type[(i >> 1i32)] = -32732i32 as u16);
            InitializeShinesparkEchoOrSpazerSba(i as u16, game);
        } else {
            set_ram_word!(
                game.projectile_type[(i >> 1i32)] =
                    (ram_word!(game, equipped_beams) as i32 & 0x100fi32 | 0x8010i32) as u16
            );
            InitializeSbaProjectile(i as u16, game);
        }
        i -= 2i32;
    }
    set_ram_word!(game.projectile_counter = 4u16);
    set_ram_word!(
        game.cooldown_timer = kProjectileCooldown_Uncharged
            [(ram_word!(game, projectile_type[0]) as i32 & 0x3fi32) as usize]
            as u16
    );
    set_ram_word!(game.used_for_sba_attacksB60 = 0u16);
    QueueSfx1_Max6(0x25u16, game);
    1u8
}
pub unsafe fn FireSba_FirePlasma(game: &mut Game) -> u8 {
    static kIcePlasmaSbaProjectileOriginAngles: [u16; 8] =
        [0, 0x40, 0x80, 0xc0, 0x20, 0x60, 0xa0, 0xe0];
    if ram_word!(game, projectile_bomb_pre_instructions[0]) as i32 == 0x90d793i32 & 0xffffi32 {
        return 0u8;
    }
    let mut i: i32 = 6i32;
    while i >= 0i32 {
        let v2: i32 = i >> 1i32;
        set_ram_word!(
            game.projectile_type[v2] =
                (ram_word!(game, equipped_beams) as i32 & 0x100fi32 | 0x8010i32) as u16
        );
        set_ram_word!(game.projectile_dir[v2] = 0u16);
        set_ram_word!(game.projectile_bomb_pre_instructions[v2] = ((0x90d793i32) as u16));
        set_ram_word!(
            game.projectile_variables[v2] = kIcePlasmaSbaProjectileOriginAngles[v2 as usize]
        );
        set_ram_word!(game.projectile_bomb_x_speed[v2] = 40u16);
        set_ram_word!(game.projectile_bomb_y_speed[v2] = 0u16);
        InitializeSbaProjectile(i as u16, game);
        i -= 2i32;
    }
    set_ram_word!(game.projectile_counter = 4u16);
    set_ram_word!(
        game.cooldown_timer = kProjectileCooldown_Uncharged
            [(ram_word!(game, projectile_type[0]) as i32 & 0x3fi32) as usize]
            as u16
    );
    if *ram_ptr!(game; 0xa1e) == 4u8 {
        set_ram_word!(game.used_for_sba_attacksB60 = -4i32 as u16);
    } else {
        set_ram_word!(game.used_for_sba_attacksB60 = 4u16);
    }
    QueueSfx1_Max6(0x27u16, game);
    1u8
}
pub unsafe fn ProjPreInstr_IceSba(mut k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    if (ram_word!(game, projectile_dir[v1]) as i32 & 0xf0i32) != 0i32 {
        QueueSfx1_Max6(0x24u16, game);
        ClearProjectile(k, game);
    } else {
        let ref mut fresh49 = *(ram_ptr!(game, u16; 0xc90)).offset(v1 as isize);
        let fresh50 = *fresh49;
        *fresh49 = (*fresh49).wrapping_sub(1);
        let mut v2: bool = fresh50 == 1u16;
        if v2 {
            set_ram_word!(game.projectile_timers[v1] = 4u16);
            SpawnProjectileTrail(k, game);
            k = ram_word!(game, projectile_index);
        }
        let v3: i32 = k as i32 >> 1i32;
        let pt: Point16U = Projectile_SinLookup(ram_word!(game, projectile_variables[v3]), 0x20u16);
        set_ram_word!(
            game.projectile_x_pos[v3] = (pt.x).wrapping_add(ram_word!(game, samus_x_pos))
        );
        set_ram_word!(
            game.projectile_y_pos[v3] = (pt.y).wrapping_add(ram_word!(game, samus_y_pos))
        );
        set_ram_word!(
            game.projectile_variables[v3] = (((ram_word!(game, used_for_sba_attacksB60)) as u8)
                .wrapping_add((ram_word!(game, projectile_variables[v3])) as u8,))
                as u16
        );
        let ref mut fresh53 = *(ram_ptr!(game, u16; 0xbf0)).offset(v3 as isize);
        let fresh54 = *fresh53;
        *fresh53 = (*fresh53).wrapping_sub(1);
        v2 = fresh54 == 1u16;
        if v2 {
            set_ram_word!(game.projectile_bomb_pre_instructions[v3] = ((0x90cf7ai32) as u16));
            set_ram_word!(game.projectile_bomb_x_speed[v3] = 40u16);
            QueueSfx1_Max6(0x24u16, game);
        }
        set_ram_word!(game.cooldown_timer = 2u16);
        set_ram_word!(game.flare_counter = 0u16);
    };
}
pub unsafe fn ProjPreInstr_IceSba2(mut k: u16, game: &mut Game) {
    let mut pt: Point16U = Point16U { x: 0, y: 0 };

    let mut v7: i16 = 0;
    let v1: i32 = k as i32 >> 1i32;
    let clear_projectile: bool = ((ram_word!(game, projectile_dir[v1]) as i32 & 0xf0i32) != 0i32)
        || ({
            let ref mut fresh45 = *(ram_ptr!(game, u16; 0xc90)).offset(v1 as isize);
            let fresh46 = *fresh45;
            *fresh45 = (*fresh45).wrapping_sub(1);
            if fresh46 == 1u16 {
                set_ram_word!(game.projectile_timers[v1] = 4u16);
                SpawnProjectileTrail(k, game);
                k = ram_word!(game, projectile_index);
            }
            let mut v3: i32 = 0;
            v3 = k as i32 >> 1i32;
            pt = Projectile_SinLookup(
                ram_word!(game, projectile_variables[v3]),
                ram_word!(game, projectile_bomb_x_speed[v3]),
            );
            pt.x = (pt.x).wrapping_add(ram_word!(game, samus_x_pos));
            set_ram_word!(game.projectile_x_pos[v3] = pt.x);
            let mut v5: i16 = ((pt.x) as i16).wrapping_sub((ram_word!(game, layer1_x_pos)) as i16);
            (v5 as i32 + 32i32 & 0x8000i32 != 0
                || v5 as i32 - 288i32 & 0x8000i32 == 0
                || {
                    let mut v6: i16 =
                        ((pt.y) as i16).wrapping_add((ram_word!(game, samus_y_pos)) as i16);
                    set_ram_word!(
                        game.projectile_y_pos[v3] =
                            (pt.y).wrapping_add(ram_word!(game, samus_y_pos))
                    );
                    v7 = (v6).wrapping_sub((ram_word!(game, layer1_y_pos)) as i16);
                    v7 as i32 - 16i32 & 0x8000i32 != 0
                }
                || v7 as i32 - 256i32 & 0x8000i32 == 0)
                || ({
                    set_ram_word!(
                        game.projectile_variables[v3] =
                            (((ram_word!(game, used_for_sba_attacksB60)) as u8)
                                .wrapping_add((ram_word!(game, projectile_variables[v3])) as u8,))
                                as u16
                    );
                    set_ram_word!(
                        game.projectile_bomb_x_speed[v3] =
                            (((ram_word!(game, projectile_bomb_x_speed[v3])) as u8)
                                .wrapping_add(8u8)) as u16
                    );
                    set_ram_word!(game.cooldown_timer = 2u16);
                    set_ram_word!(game.flare_counter = 0u16);
                    false
                })
        });
    if clear_projectile {
        ClearProjectile(k, game);
    };
}
pub unsafe fn Projectile_Func7_Shinespark(game: &mut Game) {
    set_ram_word!(game.samus_movement_handler = ((0x90d068i32) as u16));
    set_ram_word!(game.samus_y_dir = 1u16);
    set_ram_word!(game.speed_boost_counter = 1024u16);
    set_ram_words! { game, 0u16; samus_y_subspeed, samus_y_speed, knockback_dir }
    set_ram_word!(game.samus_x_extra_run_speed = 8u16);
    set_ram_words! { game, 0u16;
        samus_x_extra_run_subspeed, samus_x_base_speed, samus_x_base_subspeed,
    }
    set_ram_word!(game.substate = 7u16);
    set_ram_words! { game, 0u16; suit_pickup_light_beam_pos, cooldown_timer }
    set_ram_word!(game.timer_for_shinesparks_startstop = 30u16);
    set_ram_word!(game.samus_shine_timer = 60u16);
    set_ram_word!(game.timer_for_shine_timer = 6u16);
    set_ram_words! { game, 0u16; special_samus_palette_frame, bomb_jump_dir }
    if ram_word!(game, flare_counter) != 0 {
        if ram_word!(game, flare_counter) as i32 - 16i32 & 0x8000i32 == 0 {
            QueueSfx1_Max9(2u16, game);
        }
        set_ram_word!(game.flare_counter = 0u16);
        ClearFlareAnimationState(game);
    }
}
pub unsafe fn Samus_MoveHandlerShinesparkWindup(game: &mut Game) {
    let v0: bool = (update_ram_word!(
        game.timer_for_shinesparks_startstop,
        wrapping_sub(1),
        reread
    ) as i32
        & 0x8000i32)
        != 0i32;
    if ram_word!(game, timer_for_shinesparks_startstop) == 0 || v0 {
        if *ram_ptr!(game; 0xa1e) == 4u8 {
            set_ram_word!(game.samus_new_pose_interrupted = 204u16);
        } else {
            set_ram_word!(game.samus_new_pose_interrupted = 203u16);
        }
        set_ram_word!(game.samus_movement_handler = ((0x90d0abi32) as u16));
        set_ram_word!(game.samus_input_handler = ((0x90e90ei32) as u16));
        set_ram_words! { game, 0u16;
            speed_echoes_index, speed_echo_xspeed[0], speed_echo_xspeed[1], speed_echo_xpos[0],
            speed_echo_xpos[1],
        }
        QueueSfx3_Max9(0xfu16, game);
    }
}
pub unsafe fn Samus_MoveHandlerVerticalShinespark(game: &mut Game) {
    set_ram_word!(game.samus_contact_damage_index = 2u16);
    set_ram_word!(game.samus_hurt_flash_counter = 8u16);
    Samus_UpdateSpeedEchoPos(game);
    Samus_ShinesparkMove_Y(game);
    Samus_EndSuperJump(game);
    if ram_word!(game, samus_health) as i32 - 30i32 & 0x8000i32 == 0 && {
        (update_ram_word!(game.samus_health, wrapping_sub(1), reread) as i32 & 0x8000i32) != 0i32
    } {
        set_ram_word!(game.samus_health = 0u16);
    }
}
pub unsafe fn Samus_MoveHandler_Shinespark_Diag(game: &mut Game) {
    set_ram_word!(game.samus_contact_damage_index = 2u16);
    set_ram_word!(game.samus_hurt_flash_counter = 8u16);
    Samus_UpdateSpeedEchoPos(game);
    Samus_ShinesparkMove_X(game);
    Samus_ShinesparkMove_Y(game);
    Samus_EndSuperJump(game);
    if ram_word!(game, samus_health) as i32 - 30i32 & 0x8000i32 == 0 && {
        (update_ram_word!(game.samus_health, wrapping_sub(1), reread) as i32 & 0x8000i32) != 0i32
    } {
        set_ram_word!(game.samus_health = 0u16);
    }
}
pub unsafe fn Samus_MoveHandler_Shinespark_Horiz(game: &mut Game) {
    set_ram_word!(game.samus_contact_damage_index = 2u16);
    set_ram_word!(game.samus_hurt_flash_counter = 8u16);
    Samus_UpdateSpeedEchoPos(game);
    Samus_ShinesparkMove_X(game);
    Samus_EndSuperJump(game);
    if ram_word!(game, samus_health) as i32 - 30i32 & 0x8000i32 == 0 && {
        (update_ram_word!(game.samus_health, wrapping_sub(1), reread) as i32 & 0x8000i32) != 0i32
    } {
        set_ram_word!(game.samus_health = 0u16);
    }
}
unsafe fn Samus_ClampSpeedHi(mut amt: i32, val: i32) -> u32 {
    if (amt >> 16i32) - val & 0x8000i32 == 0 {
        amt = (val) << 16i32 | amt & 0xffffi32;
    }
    amt as u32
}
pub unsafe fn Samus_ShinesparkMove_X(game: &mut Game) {
    set_ram_word!(game.samus_shine_timer = 15u16);
    AddToHiLo(
        ram_ptr!(game, u16; 0xb42),
        ram_ptr!(game, u16; 0xb44),
        (ram_word!(game, samus_y_accel) as u32) << 16i32 | ram_word!(game, samus_y_subaccel) as u32,
    );
    if ram_word!(game, samus_x_extra_run_speed) as i32 - 15i32 & 0x8000i32 == 0 {
        SetHiLo(
            ram_ptr!(game, u16; 0xb42),
            ram_ptr!(game, u16; 0xb44),
            ((15i16 as i32) << 16i32) as u32,
        );
    }
    let mut amt: i32 = 0i32;
    let move_right: bool = if *ram_ptr!(game; 0xa1e) == 4u8 {
        amt = Samus_ClampSpeedHi(-Samus_CalcDisplacementMoveLeft(amt, game), 15i32) as i32;
        let cres: CheckEnemyColl_Result = Samus_CheckSolidEnemyColl(amt, game);
        amt = cres.amt;
        if cres.collision != 0 {
            set_ram_word!(game.samus_collision_flag = cres.collision);
            false
        } else {
            amt = -amt;
            true
        }
    } else {
        amt = Samus_ClampSpeedHi(Samus_CalcDisplacementMoveRight(amt, game), 25i32) as i32;
        let cres_0: CheckEnemyColl_Result = Samus_CheckSolidEnemyColl(amt, game);
        amt = cres_0.amt;
        !(cres_0.collision != 0)
            || ({
                set_ram_word!(game.samus_collision_flag = cres_0.collision);
                false
            })
    };
    if move_right {
        Samus_MoveRight_NoSolidColl(amt, game);
        Samus_AlignYPosSlope(game);
    }
    let mut v4: i16 = ((ram_word!(game, samus_x_pos)) as i16)
        .wrapping_sub((ram_word!(game, samus_prev_x_pos)) as i16);
    if ((ram_word!(game, samus_x_pos)) as i16)
        .wrapping_sub((ram_word!(game, samus_prev_x_pos)) as i16)
        < 0i16
    {
        if v4 as i32 + 15i32 & 0x8000i32 != 0 {
            set_ram_word!(
                game.samus_prev_x_pos = (ram_word!(game, samus_x_pos)).wrapping_add(15u16)
            );
        }
    } else if v4 as i32 - 16i32 & 0x8000i32 == 0 {
        set_ram_word!(game.samus_prev_x_pos = (ram_word!(game, samus_x_pos)).wrapping_sub(15u16));
    }
}
pub unsafe fn Samus_ShinesparkMove_Y(game: &mut Game) {
    set_ram_word!(game.samus_shine_timer = 15u16);
    AddToHiLo(
        ram_ptr!(game, u16; 0xdec),
        ram_ptr!(game, u16; 0xdee),
        (ram_word!(game, samus_y_accel) as u32) << 16i32 | ram_word!(game, samus_y_subaccel) as u32,
    );
    AddToHiLo(
        ram_ptr!(game, u16; 0xb2e),
        ram_ptr!(game, u16; 0xb2c),
        (ram_word!(game, substate) as u32) << 16i32
            | ram_word!(game, suit_pickup_light_beam_pos) as u32,
    );
    let mut amt: i32 = Samus_ClampSpeedHi(
        ((ram_word!(game, samus_y_speed) as u32) << 16i32
            | ram_word!(game, samus_y_subspeed) as u32) as i32,
        14i32,
    ) as i32;
    amt = (amt as u32).wrapping_sub(
        (ram_word!(game, extra_samus_y_displacement) as u32) << 16i32
            | ram_word!(game, extra_samus_y_subdisplacement) as u32,
    ) as i32;
    set_ram_word!(game.samus_collision_direction = 2u16);
    let cres: CheckEnemyColl_Result =
        Samus_CheckSolidEnemyColl(Samus_ClampSpeedHi(amt, 15i32) as i32, game);
    amt = cres.amt;
    if cres.collision != 0 {
        set_ram_word!(game.samus_collision_flag = cres.collision);
    } else {
        Samus_MoveDown_NoSolidColl(-amt, game);
    }
    if ram_word!(game, samus_y_pos) as i32 - ram_word!(game, samus_prev_y_pos) as i32 + 14i32
        & 0x8000i32
        != 0
    {
        set_ram_word!(game.samus_prev_y_pos = (ram_word!(game, samus_y_pos)).wrapping_add(14u16));
    }
}
pub unsafe fn Samus_EndSuperJump(game: &mut Game) -> u8 {
    if ram_word!(game, samus_health) as i32 - 30i32 & 0x8000i32 == 0
        && ram_word!(game, samus_collision_flag) == 0
    {
        return 0u8;
    }
    if *ram_ptr!(game; 0xa1e) == 4u8 {
        set_ram_word!(game.speed_echo_xspeed[0] = 32u16);
        set_ram_word!(game.speed_echo_xspeed[1] = 160u16);
        set_ram_word!(game.speed_echo_xpos[2] = 4u16);
    } else {
        set_ram_word!(game.speed_echo_xspeed[0] = 224u16);
        set_ram_word!(game.speed_echo_xspeed[1] = 96u16);
        set_ram_word!(game.speed_echo_xpos[2] = -4i32 as u16);
    }
    set_ram_words! { game, 0u16;
        samus_y_subspeed, samus_y_speed, samus_x_extra_run_speed, samus_x_extra_run_subspeed,
        speed_boost_counter, UNUSED_word_7E0B1A, samus_y_dir,
    }
    set_ram_word!(game.samus_movement_handler = ((0x90d346i32) as u16));
    set_ram_word!(game.samus_draw_handler = ((0x90ebf3i32) as u16));
    set_ram_word!(game.speed_echoes_index = 0u16);
    set_ram_word!(game.speed_echo_xpos[0] = ram_word!(game, samus_x_pos));
    set_ram_word!(game.speed_echo_xpos[1] = ram_word!(game, samus_x_pos));
    set_ram_word!(game.speed_echo_ypos[0] = ram_word!(game, samus_y_pos));
    set_ram_word!(game.speed_echo_ypos[1] = ram_word!(game, samus_y_pos));
    set_ram_words! { game, 0u16; speed_echo_xspeed[2], samus_hurt_flash_counter }
    QueueSfx1_Max6(0x35u16, game);
    QueueSfx3_Max6(0x10u16, game);
    1u8
}
static kSamus_MoveHandler_ShinesparkCrash: [Option<Func_V>; 3] = callback_table! { unsafe Func_V;
    Samus_MoveHandler_ShinesparkCrash_0, Samus_MoveHandler_ShinesparkCrash_1,
    Samus_MoveHandler_ShinesparkCrash_2,
};
pub unsafe fn Samus_MoveHandler_ShinesparkCrash(game: &mut Game) {
    set_ram_word!(game.samus_shine_timer = 15u16);
    kSamus_MoveHandler_ShinesparkCrash
        [*(ram_ptr!(game, u16; 0xaae) as *mut u8).offset(1isize) as usize]
        .expect("non-null function pointer")(game);
    let mut i: i32 = 2i32;
    while i >= 0i32 {
        let v1: i32 = i >> 1i32;
        let pt: Point16U = Projectile_SinLookup(
            ram_word!(game, speed_echo_xspeed[v1]),
            ram_word!(game, speed_echoes_index) as u8 as u16,
        );
        set_ram_word!(game.speed_echo_xpos[v1] = (pt.x).wrapping_add(ram_word!(game, samus_x_pos)));
        set_ram_word!(game.speed_echo_ypos[v1] = (pt.y).wrapping_add(ram_word!(game, samus_y_pos)));
        i -= 2i32;
    }
}
pub unsafe fn Samus_MoveHandler_ShinesparkCrash_0(game: &mut Game) {
    let mut v0: u16 = (ram_word!(game, speed_echoes_index)).wrapping_add(4u16);
    if ram_word!(game, speed_echoes_index) as i32 - 12i32 & 0x8000i32 == 0 {
        v0 = (v0) | (0x100u16);
    }
    set_ram_word!(game.speed_echoes_index = v0);
}
pub unsafe fn Samus_MoveHandler_ShinesparkCrash_1(game: &mut Game) {
    set_ram_word!(
        game.speed_echo_xspeed[0] = ((*((ram_ptr!(game, u16; 0xab0)).offset(2) as *mut u8)
            .offset(0))
        .wrapping_add(*((ram_ptr!(game, u16; 0xac0)).offset(0) as *mut u8).offset(0),))
            as u16
    );
    set_ram_word!(
        game.speed_echo_xspeed[1] = ((*((ram_ptr!(game, u16; 0xab0)).offset(2) as *mut u8)
            .offset(0))
        .wrapping_add(*((ram_ptr!(game, u16; 0xac0)).offset(1) as *mut u8).offset(0),))
            as u16
    );
    update_ram_word!(game.speed_echo_ypos[2], wrapping_add(4u16));
    if ram_word!(game, speed_echo_ypos[2]) as i32 - 128i32 & 0x8000i32 == 0 {
        set_ram_word!(
            game.speed_echoes_index =
                ((ram_word!(game, speed_echoes_index) as u8) as u16) | (0x200u16)
        );
    }
}
pub unsafe fn Samus_MoveHandler_ShinesparkCrash_2(game: &mut Game) {
    update_ram_word!(game.speed_echoes_index, wrapping_sub(4u16));
    if ram_word!(game, speed_echoes_index) as u8 == 0 {
        set_ram_word!(game.samus_movement_handler = ((0x90d3f3i32) as u16));
        set_ram_word!(game.timer_for_shinesparks_startstop = 30u16);
        set_ram_words! { game, 0u16;
            speed_echoes_index, speed_echo_xspeed[0], speed_echo_xspeed[1],
        }
    }
}
pub unsafe fn Samus_MoveHandler_ShinesparkCrashEchoCircle(game: &mut Game) {
    set_ram_word!(game.samus_shine_timer = 15u16);
    let v0: bool = (update_ram_word!(
        game.timer_for_shinesparks_startstop,
        wrapping_sub(1),
        reread
    ) as i32
        & 0x8000i32)
        != 0i32;
    if ram_word!(game, timer_for_shinesparks_startstop) == 0 || v0 {
        set_ram_word!(game.samus_movement_handler = ((0x90d40di32) as u16));
        set_ram_word!(game.samus_draw_handler = ((0x90eb52i32) as u16));
    }
}
pub unsafe fn Samus_MoveHandler_ShinesparkCrashFinish(game: &mut Game) {
    static kShinesparkCrashFinish_Tab0: [u8; 12] = [
        0, 0x80, 0, 0x80, 0x40, 0xc0, 0x40, 0xc0, 0xe0, 0x60, 0x20, 0xa0,
    ];
    set_ram_word!(game.speed_echoes_index = 0u16);
    if ram_word!(game, projectile_counter) as i32 - 5i32 & 0x8000i32 != 0 {
        if ram_word!(game, projectile_counter) as i32 - 4i32 & 0x8000i32 != 0 {
            update_ram_word!(game.projectile_counter, wrapping_add(1));
            set_ram_word!(game.speed_echo_xspeed[2] = 64u16);
            set_ram_word!(game.speed_echo_xpos[2] = ram_word!(game, samus_x_pos));
            set_ram_word!(game.speed_echo_ypos[2] = ram_word!(game, samus_y_pos));
            set_ram_word!(game.projectile_type[3] = addr_loc_908029 as u16);
            InitializeShinesparkEchoOrSpazerSba(6u16, game);
            set_ram_word!(game.projectile_bomb_pre_instructions[3] = ((0x90d4d2i32) as u16));
            set_ram_word!(
                game.projectile_variables[3] = kShinesparkCrashFinish_Tab0
                    [(2i32 * (ram_word!(game, samus_pose) as i32 - 201i32)) as u16 as usize]
                    as u16
            );
            set_ram_word!(game.projectile_bomb_x_speed[3] = 0u16);
        }
        update_ram_word!(game.projectile_counter, wrapping_add(1));
        set_ram_word!(game.speed_echo_xspeed[3] = 64u16);
        set_ram_word!(game.speed_echo_xpos[3] = ram_word!(game, samus_x_pos));
        set_ram_word!(game.speed_echo_ypos[3] = ram_word!(game, samus_y_pos));
        set_ram_word!(game.projectile_type[4] = addr_loc_908029 as u16);
        InitializeShinesparkEchoOrSpazerSba(8u16, game);
        set_ram_word!(game.projectile_bomb_pre_instructions[4] = ((0x90d4d2i32) as u16));
        set_ram_word!(
            game.projectile_variables[4] =
                kShinesparkCrashFinish_Tab0[((2i32 * (ram_word!(game, samus_pose) as i32 - 201i32))
                    as u16 as i32
                    + 1i32) as usize] as u16
        );
        set_ram_word!(game.projectile_bomb_x_speed[4] = 0u16);
    }
    set_ram_word!(game.cooldown_timer = 0u16);
    set_ram_word!(game.samus_shine_timer = 1u16);
    if *ram_ptr!(game; 0xa1e) == 4u8 {
        set_ram_word!(game.samus_new_pose_transitional = 2u16);
    } else {
        set_ram_word!(game.samus_new_pose_transitional = 1u16);
    }
    set_ram_word!(game.samus_hurt_switch_index = 2u16);
    set_ram_words! { game, 0u16; substate, suit_pickup_light_beam_pos }
}
pub unsafe fn ProjPreInstr_SpeedEcho(k: u16, game: &mut Game) {
    let mut v5: i16 = 0;
    let v1: i32 = k as i32 >> 1i32;
    update_ram_word!(game.projectile_bomb_x_speed[v1], wrapping_add(8u16));
    let pt: Point16U = Projectile_SinLookup(
        ram_word!(game, projectile_variables[v1]),
        *((ram_ptr!(game, u16; 0xbdc)).offset(v1 as isize) as *mut u8).offset(0) as u16,
    );
    let v2: u16 = (pt.x).wrapping_add(ram_word!(game, samus_x_pos));
    *((ram_ptr!(game, u16; 0xaae) as *mut u8).offset(k as i32 as isize) as *mut u16) =
        (pt.x).wrapping_add(ram_word!(game, samus_x_pos));
    set_ram_word!(game.projectile_x_pos[v1] = v2);
    let mut v3: i16 = ((v2) as i16).wrapping_sub((ram_word!(game, layer1_x_pos)) as i16);
    if v3 < 0i16
        || v3 as i32 - 256i32 & 0x8000i32 == 0
        || {
            let mut v4: u16 = (pt.y).wrapping_add(ram_word!(game, samus_y_pos));
            set_ram_word!(
                game.speed_echo_xpos[(v1 + 3i32)] =
                    (pt.y).wrapping_add(ram_word!(game, samus_y_pos))
            );
            set_ram_word!(game.projectile_y_pos[v1] = v4);
            v5 = ((v4) as i16).wrapping_sub((ram_word!(game, layer1_y_pos)) as i16);
            v5 < 0i16
        }
        || v5 as i32 - 256i32 & 0x8000i32 == 0
    {
        set_ram_word!(game.speed_echo_ypos[(v1 + 3i32)] = 0u16);
        *((ram_ptr!(game, u16; 0xaae) as *mut u8).offset(k as i32 as isize) as *mut u16) = 0u16;
        set_ram_word!(game.speed_echo_xpos[(v1 + 3i32)] = 0u16);
        ClearProjectile(k, game);
    }
}
pub unsafe fn Grapple_Func1(game: &mut Game) {
    let mut v0: bool = false;
    if (ram_word!(game, button_config_shoot_x) as i32 & ram_word!(game, joypad1_lastkeys) as i32)
        != 0i32
        && {
            v0 = ((ram_word!(game, grapple_walljump_timer)) as i16).wrapping_sub(1i16) < 0i16;
            update_ram_word!(game.grapple_walljump_timer, wrapping_sub(1));
            ram_word!(game, grapple_walljump_timer) != 0u16
        }
        && !v0
    {
        if (ram_word!(game, grapple_beam_length_delta) as i32 & 0x8000i32) == 0i32 {
            update_ram_word!(
                game.grapple_beam_length,
                wrapping_add(ram_word!(game, grapple_beam_length_delta))
            );
            if ram_word!(game, grapple_beam_length) as i32 - 96i32 & 0x8000i32 == 0 {
                set_ram_word!(game.grapple_beam_length_delta = -16i32 as u16);
            }
        }
        let pt: Point16U = Projectile_SinLookup(
            *(ram_ptr!(game, u16; 0xcfa) as *mut u8).offset(1isize) as u16,
            ram_word!(game, grapple_beam_length),
        );
        set_ram_word!(
            game.grapple_beam_end_x_pos =
                (pt.x).wrapping_add(ram_word!(game, x_pos_of_start_of_grapple_beam))
        );
        set_ram_word!(
            game.grapple_beam_end_y_pos =
                (pt.y).wrapping_add(ram_word!(game, y_pos_of_start_of_grapple_beam))
        );
        update_ram_word!(game.grapple_beam_end_angle16, wrapping_add(2048u16));
        GrappleBeamFunc_BF1B(game);
    } else {
        set_ram_word!(game.grapple_beam_function = ((0x9bc856i32) as u16));
    };
}
pub unsafe fn Hdmaobj_CrystalFlash(game: &mut Game) -> u8 {
    let mut v1: SamusPose = kPose_00_FaceF_Powersuit;
    if ram_word!(game, game_state) as i32 - 40i32 & 0x8000i32 != 0
        && ram_word!(game, joypad1_lastkeys) as i32
            != ram_word!(game, button_config_shoot_x) as i32 | 0x430i32
    {
        return 1u8;
    }
    if ram_word!(game, samus_y_speed) != 0u16
        || ram_word!(game, samus_y_subspeed) != 0u16
        || ram_word!(game, samus_health) as i32 - 51i32 & 0x8000i32 == 0
        || ram_word!(game, samus_reserve_health) != 0u16
        || ram_word!(game, samus_missiles) as i32 - 10i32 & 0x8000i32 != 0
        || ram_word!(game, samus_super_missiles) as i32 - 10i32 & 0x8000i32 != 0
        || ram_word!(game, samus_power_bombs) as i32 - 10i32 & 0x8000i32 != 0
    {
        return 1u8;
    }
    v1 = (if *ram_ptr!(game; 0xa1e) == 4u8 {
        kPose_D4_FaceL_CrystalFlash as i32
    } else {
        kPose_D3_FaceR_CrystalFlash as i32
    }) as SamusPose;
    set_ram_word!(game.samus_pose = v1 as u16);
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    if *ram_ptr!(game; 0xa1f) != 27u8 {
        return 1u8;
    }
    set_ram_word!(game.samus_last_different_pose = ram_word!(game, samus_prev_pose));
    *(ram_ptr!(game, u16; 0xa26)) = *(ram_ptr!(game, u16; 0xa22));
    set_ram_word!(game.samus_prev_pose = ram_word!(game, samus_pose));
    *(ram_ptr!(game, u16; 0xa22)) = *(ram_ptr!(game, u16; 0xa1e));
    set_ram_word!(game.samus_movement_handler = ((0x90d678i32) as u16));
    if ram_word!(game, samus_input_handler) as i32 != 0x90e91di32 & 0xffffi32 {
        set_ram_word!(game.samus_input_handler = ((0x90e90ei32) as u16));
    }
    set_ram_word!(game.timer_for_shinesparks_startstop = 9u16);
    set_ram_word!(game.which_item_to_pickup = 0u16);
    set_ram_word!(game.substate = 10u16);
    set_ram_word!(game.suit_pickup_light_beam_pos = 0u16);
    *(ram_ptr!(game, u16; 0xdf0)) = 0u16;
    set_ram_word!(game.timer_for_shine_timer = 7u16);
    set_ram_word!(game.special_samus_palette_frame = 0u16);
    set_ram_word!(game.samus_shine_timer = 1u16);
    *(ram_ptr!(game, u16; 0xdf2)) = 1u16;
    set_ram_words! { game, 0u16; samus_invincibility_timer, samus_knockback_timer, knockback_dir }
    0u8
}
pub unsafe fn SamusMoveHandler_CrystalFlashStart(game: &mut Game) {
    update_ram_word!(game.samus_y_pos, wrapping_sub(2u16));
    if (update_ram_word!(
        game.timer_for_shinesparks_startstop,
        wrapping_sub(1),
        reread
    ) as i32
        & 0x8000i32)
        != 0i32
    {
        set_ram_word!(game.samus_anim_frame_timer = 3u16);
        set_ram_word!(game.samus_anim_frame = 6u16);
        *(ram_ptr!(game, u16; 0xdf0)) = ram_word!(game, samus_y_pos);
        set_ram_word!(game.samus_movement_handler = ((0x90d6cei32) as u16));
        set_ram_words! { game, 0u16; samus_invincibility_timer, samus_knockback_timer }
        QueueSfx3_Max15(1u16, game);
        set_ram_word!(game.power_bomb_flag = 0u16);
        set_ram_word!(game.power_bomb_explosion_x_pos = ram_word!(game, samus_x_pos));
        set_ram_word!(game.power_bomb_explosion_y_pos = ram_word!(game, samus_y_pos));
        EnableHdmaObjects(game);
        SpawnCrystalFlashHdmaObjs(game);
    }
}
static kSamusMoveHandler_CrystalFlashMainFuncs: [Option<Func_V>; 3] = callback_table! { unsafe Func_V;
    SamusMoveHandler_CrystalFlashMain_0, SamusMoveHandler_CrystalFlashMain_1,
    SamusMoveHandler_CrystalFlashMain_2,
};
pub unsafe fn SamusMoveHandler_CrystalFlashMain(game: &mut Game) {
    kSamusMoveHandler_CrystalFlashMainFuncs[ram_word!(game, which_item_to_pickup) as usize]
        .expect("non-null function pointer")(game);
    set_ram_words! { game, 0u16; samus_invincibility_timer, samus_knockback_timer }
}
pub unsafe fn SamusMoveHandler_CrystalFlashMain_0(game: &mut Game) {
    if (ram_word!(game, nmi_frame_counter_word) as i32 & 7i32) == 0i32 {
        update_ram_word!(game.samus_missiles, wrapping_sub(1));
        Samus_RestoreHealth(0x32u16, game);
        let v0: bool = ((ram_word!(game, substate)) as i16).wrapping_sub(1i16) < 0i16;
        if update_ram_word!(game.substate, wrapping_sub(1), reread) == 0 || v0 {
            set_ram_word!(game.substate = 10u16);
            update_ram_word!(game.which_item_to_pickup, wrapping_add(1));
        }
    }
}
pub unsafe fn SamusMoveHandler_CrystalFlashMain_1(game: &mut Game) {
    if (ram_word!(game, nmi_frame_counter_word) as i32 & 7i32) == 0i32 {
        update_ram_word!(game.samus_super_missiles, wrapping_sub(1));
        Samus_RestoreHealth(0x32u16, game);
        let v0: bool = ((ram_word!(game, substate)) as i16).wrapping_sub(1i16) < 0i16;
        if update_ram_word!(game.substate, wrapping_sub(1), reread) == 0 || v0 {
            set_ram_word!(game.substate = 10u16);
            update_ram_word!(game.which_item_to_pickup, wrapping_add(1));
        }
    }
}
pub unsafe fn SamusMoveHandler_CrystalFlashMain_2(game: &mut Game) {
    if (ram_word!(game, nmi_frame_counter_word) as i32 & 7i32) == 0i32 {
        update_ram_word!(game.samus_power_bombs, wrapping_sub(1));
        Samus_RestoreHealth(0x32u16, game);
        let v0: bool = ((ram_word!(game, substate)) as i16).wrapping_sub(1i16) < 0i16;
        if update_ram_word!(game.substate, wrapping_sub(1), reread) == 0 || v0 {
            set_ram_word!(game.samus_movement_handler = ((0x90d75bi32) as u16));
            set_ram_word!(game.samus_draw_handler = ((0x90eb52i32) as u16));
            set_ram_word!(game.samus_anim_frame_timer = 3u16);
            set_ram_word!(game.samus_anim_frame = 12u16);
        }
    }
}
pub unsafe fn kSamusMoveHandler_CrystalFlashFinish(game: &mut Game) {
    if ram_word!(game, samus_y_pos) != *(ram_ptr!(game, u16; 0xdf0)) {
        update_ram_word!(game.samus_y_pos, wrapping_add(1));
    }
    if *ram_ptr!(game; 0xa1f) == 0 {
        set_ram_word!(game.power_bomb_flag = 0u16);
        set_ram_word!(game.samus_shine_timer = -1i32 as u16);
        set_ram_word!(game.samus_movement_handler = ((0x90a337i32) as u16));
        if ram_word!(game, samus_input_handler) as i32 != 0x90e91di32 & 0xffffi32 {
            set_ram_word!(game.samus_input_handler = ((0x90e913i32) as u16));
            set_ram_words! { game, 0u16; samus_invincibility_timer, samus_knockback_timer }
        }
    }
}
pub unsafe fn ProjPreInstr_PlasmaSba(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    if (ram_word!(game, projectile_dir[v1]) as i32 & 0xf0i32) != 0i32 {
        ClearProjectile(k, game);
    } else {
        set_ram_word!(game.cooldown_timer = 2u16);
        set_ram_word!(game.flare_counter = 0u16);
        let pt: Point16U = Projectile_SinLookup(
            ram_word!(game, projectile_variables[v1]),
            ram_word!(game, projectile_bomb_x_speed[v1]),
        );
        set_ram_word!(
            game.projectile_x_pos[v1] = (pt.x).wrapping_add(ram_word!(game, samus_x_pos))
        );
        set_ram_word!(
            game.projectile_y_pos[v1] = (pt.y).wrapping_add(ram_word!(game, samus_y_pos))
        );
        set_ram_word!(
            game.projectile_variables[v1] = (((ram_word!(game, used_for_sba_attacksB60)) as u8)
                .wrapping_add((ram_word!(game, projectile_variables[v1])) as u8,))
                as u16
        );
        let mut v3: i16 =
            (2i16).wrapping_mul((ram_word!(game, projectile_bomb_y_speed[v1])) as i16);
        if v3 != 0 {
            if v3 == 2i16 {
                ProjPreInstr_PlasmaSbaFunc_1(k, game);
            } else {
                if v3 != 4i16 {
                    Unreachable(game);
                    loop {}
                }
                ProjPreInstr_PlasmaSbaFunc_2(k, game);
            }
        } else {
            ProjPreInstr_PlasmaSbaFunc_0(k, game);
        }
    };
}
pub unsafe fn ProjPreInstr_PlasmaSbaFunc_0(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    let v2: u16 = (((ram_word!(game, projectile_bomb_x_speed[v1])) as u8).wrapping_add(4u8)) as u16;
    set_ram_word!(game.projectile_bomb_x_speed[v1] = v2);
    if v2 as i32 - 192i32 & 0x8000i32 == 0 {
        set_ram_word!(game.projectile_bomb_y_speed[v1] = 1u16);
    }
}
pub unsafe fn ProjPreInstr_PlasmaSbaFunc_1(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    let v2: u16 = (((ram_word!(game, projectile_bomb_x_speed[v1])) as u8).wrapping_sub(4u8)) as u16;
    set_ram_word!(game.projectile_bomb_x_speed[v1] = v2);
    if v2 as i32 - 45i32 & 0x8000i32 != 0 {
        set_ram_word!(game.projectile_bomb_y_speed[v1] = 2u16);
    }
}
pub unsafe fn ProjPreInstr_PlasmaSbaFunc_2(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    if ram_word!(game, projectile_x_pos[v1]) as i32 - ram_word!(game, layer1_x_pos) as i32 + 32i32
        & 0x8000i32
        != 0
        || ram_word!(game, projectile_x_pos[v1]) as i32
            - ram_word!(game, layer1_x_pos) as i32
            - 288i32
            & 0x8000i32
            == 0
        || ram_word!(game, projectile_y_pos[v1]) as i32
            - ram_word!(game, layer1_y_pos) as i32
            - 16i32
            & 0x8000i32
            != 0
        || ram_word!(game, projectile_y_pos[v1]) as i32
            - ram_word!(game, layer1_y_pos) as i32
            - 256i32
            & 0x8000i32
            == 0
    {
        ClearProjectile(j, game);
    } else {
        set_ram_word!(
            game.projectile_bomb_x_speed[v1] =
                (((ram_word!(game, projectile_bomb_x_speed[v1])) as u8).wrapping_add(4u8)) as u16
        );
    };
}
static kBombSpread_Tab0: [u16; 5] = [0x78, 0x6e, 0x64, 0x6e, 0x78];
static kBombSpread_Tab1: [u16; 5] = [0x8100, 0x8080, 0, 0x80, 0x100];
static kBombSpread_Tab2: [u16; 5] = [0, 1, 2, 1, 0];
static kBombSpread_Tab3: [u16; 5] = [0, 0, 0x8000, 0, 0];
pub unsafe fn ProjPreInstr_SpreadBomb(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    if (ram_word!(game, projectile_dir[v1]) as i32 & 0xf0i32) != 0i32 {
        ClearProjectile(k, game);
        return;
    }
    Bomb_Func2(game);
    if ram_word!(game, projectile_variables[v1]) != 0 {
        AddToHiLo(
            (ram_ptr!(game, u16; 0xbf0)).offset(v1 as isize),
            (ram_ptr!(game, u16; 0xc90)).offset(v1 as isize),
            (ram_word!(game, samus_y_accel) as u32) << 16i32
                | ram_word!(game, samus_y_subaccel) as u32,
        );
        AddToHiLo(
            (ram_ptr!(game, u16; 0xb78)).offset(v1 as isize),
            (ram_ptr!(game, u16; 0xba0)).offset(v1 as isize),
            (ram_word!(game, projectile_bomb_y_speed[v1]) as u32) << 16i32
                | ram_word!(game, projectile_timers[v1]) as u32,
        );
        if BlockCollSpreadBomb(k, game) as i32 & 1i32 != 0 {
            AddToHiLo(
                (ram_ptr!(game, u16; 0xb78)).offset(v1 as isize),
                (ram_ptr!(game, u16; 0xba0)).offset(v1 as isize),
                -((ram_word!(game, projectile_bomb_y_speed[v1]) as i32) << 16i32
                    | ram_word!(game, projectile_timers[v1]) as i32) as u32,
            );
            if (ram_word!(game, projectile_bomb_y_speed[v1]) as i32 & 0x8000i32) != 0i32 {
                set_ram_words! { game, 0u16; projectile_bomb_y_speed[v1], projectile_y_radius[v1] }
            } else {
                set_ram_word!(game.projectile_timers[v1] = kBombSpread_Tab3[(v1 - 5i32) as usize]);
                set_ram_word!(
                    game.projectile_bomb_y_speed[v1] = ram_word!(game, projectile_unk_A[v1])
                );
            }
            return;
        }
        let t: u16 = ram_word!(game, projectile_bomb_x_speed[v1]);
        if t as i32 & 0x8000i32 != 0 {
            AddToHiLo(
                (ram_ptr!(game, u16; 0xb64)).offset(v1 as isize),
                (ram_ptr!(game, u16; 0xb8c)).offset(v1 as isize),
                -(((((t) as i16) & (0x7fffi16)) as i32) << 8i32) as u32,
            );
        } else {
            AddToHiLo(
                (ram_ptr!(game, u16; 0xb64)).offset(v1 as isize),
                (ram_ptr!(game, u16; 0xb8c)).offset(v1 as isize),
                ((t as i16 as i32) << 8i32) as u32,
            );
        }
    }
    if BlockCollSpreadBomb(k, game) as i32 & 1i32 != 0 {
        let t_0: u16 = ram_word!(game, projectile_bomb_x_speed[v1]);
        update_ram_word!(game.projectile_bomb_x_speed[v1], |fresh43| (fresh43)
            ^ (0x8000u16));
        if t_0 as i32 & 0x8000i32 == 0 {
            AddToHiLo(
                (ram_ptr!(game, u16; 0xb64)).offset(v1 as isize),
                (ram_ptr!(game, u16; 0xb8c)).offset(v1 as isize),
                -((t_0 as i16 as i32) << 8i32) as u32,
            );
        } else {
            AddToHiLo(
                (ram_ptr!(game, u16; 0xb64)).offset(v1 as isize),
                (ram_ptr!(game, u16; 0xb8c)).offset(v1 as isize),
                (((((t_0) as i16) & (0x7fffi16)) as i32) << 8i32) as u32,
            );
        }
    }
}
pub unsafe fn ProjPreInstr_WaveSba(mut k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    let mut v2: bool = false;
    let mut v3: bool = false;
    if (ram_word!(game, projectile_dir[v1]) as i32 & 0xf0i32) != 0i32
        || {
            v2 = ram_word!(game, projectile_bomb_y_speed[v1]) == 1u16;
            v3 = ((ram_word!(game, projectile_bomb_y_speed[v1])) as i16).wrapping_sub(1i16) < 0i16;
            update_ram_word!(game.projectile_bomb_y_speed[v1], wrapping_sub(1));
            v2 as i32 != 0
        }
        || v3
    {
        QueueSfx1_Max6(0x29u16, game);
        ClearProjectile(k, game);
        return;
    }
    let R34: u16 = ram_word!(game, projectile_bomb_x_speed[v1]);
    let ref mut fresh37 = *(ram_ptr!(game, u16; 0xc90)).offset(v1 as isize);
    let fresh38 = *fresh37;
    *fresh37 = (*fresh37).wrapping_sub(1);
    v2 = fresh38 == 1u16;
    if v2 {
        set_ram_word!(game.projectile_timers[v1] = 4u16);
        SpawnProjectileTrail(k, game);
        k = ram_word!(game, projectile_index);
    }
    if ((ram_word!(game, samus_x_pos)) as i16)
        .wrapping_sub((ram_word!(game, projectile_x_pos[v1])) as i16)
        < 0i16
    {
        if ram_word!(game, projectile_bomb_x_speed[v1]) as i32 + 2047i32 & 0x8000i32 == 0 {
            update_ram_word!(game.projectile_bomb_x_speed[v1], wrapping_sub(64u16));
        }
    } else if ram_word!(game, projectile_bomb_x_speed[v1]) as i32 - 2048i32 & 0x8000i32 != 0 {
        update_ram_word!(game.projectile_bomb_x_speed[v1], wrapping_add(64u16));
    }
    AddToHiLo(
        (ram_ptr!(game, u16; 0xb64)).offset(v1 as isize),
        (ram_ptr!(game, u16; 0xb8c)).offset(v1 as isize),
        ((ram_word!(game, projectile_bomb_x_speed[v1]) as i16 as i32) << 8i32) as u32,
    );
    if ((ram_word!(game, samus_y_pos)) as i16)
        .wrapping_sub((ram_word!(game, projectile_y_pos[v1])) as i16)
        < 0i16
    {
        if ram_word!(game, projectile_variables[v1]) as i32 + 2047i32 & 0x8000i32 == 0 {
            update_ram_word!(game.projectile_variables[v1], wrapping_sub(64u16));
        }
    } else if ram_word!(game, projectile_variables[v1]) as i32 - 2048i32 & 0x8000i32 != 0 {
        update_ram_word!(game.projectile_variables[v1], wrapping_add(64u16));
    }
    AddToHiLo(
        (ram_ptr!(game, u16; 0xb78)).offset(v1 as isize),
        (ram_ptr!(game, u16; 0xba0)).offset(v1 as isize),
        ((ram_word!(game, projectile_variables[v1]) as i16 as i32) << 8i32) as u32,
    );
    if k == 6u16 {
        if (ram_word!(game, projectile_bomb_x_speed[3]) as i32 & 0x8000i32) != 0i32 {
            if (R34 as i32 & 0x8000i32) == 0i32 {
                QueueSfx1_Max6(0x28u16, game);
            }
        } else if (R34 as i32 & 0x8000i32) != 0i32 {
            QueueSfx1_Max6(0x28u16, game);
        }
    }
    set_ram_word!(game.cooldown_timer = 2u16);
    set_ram_word!(game.flare_counter = 0u16);
}
pub unsafe fn BombSpread(game: &mut Game) {
    if ram_word!(game, bomb_functions[0]) as i32 != 0x90d8f7i32 & 0xffffi32 {
        let mut v0: u16 = 10u16;
        loop {
            let v1: i32 = v0 as i32 >> 1i32;
            set_ram_word!(game.projectile_type[v1] = -31488i32 as u16);
            set_ram_word!(game.projectile_dir[v1] = 0u16);
            set_ram_word!(game.projectile_bomb_pre_instructions[v1] = ((0x90d8f7i32) as u16));
            InitializeInstrForMissile(v0, game);
            set_ram_word!(game.projectile_x_pos[v1] = ram_word!(game, samus_x_pos));
            set_ram_word!(game.projectile_bomb_x_subpos[v1] = 0u16);
            set_ram_word!(game.projectile_y_pos[v1] = ram_word!(game, samus_y_pos));
            set_ram_word!(game.projectile_bomb_y_subpos[v1] = 0u16);
            let v2: i32 = ((v0).wrapping_sub(10u16)) as i32 >> 1i32;
            set_ram_word!(game.projectile_bomb_x_speed[v1] = kBombSpread_Tab1[v2 as usize]);
            set_ram_word!(game.projectile_timers[v1] = kBombSpread_Tab3[v2 as usize]);
            let v3: u16 = -((kBombSpread_Tab2[v2 as usize] as i32
                + (ram_word!(game, bomb_spread_charge_timeout_counter) as i32 >> 6i32 & 3i32))
                as i16 as i32) as u16;
            set_ram_word!(game.projectile_bomb_y_speed[v1] = v3);
            set_ram_word!(game.projectile_unk_A[v1] = v3);
            set_ram_word!(game.projectile_variables[v1] = kBombSpread_Tab0[v2 as usize]);
            v0 = (v0).wrapping_add(2u16);
            if (((v0) as i16).wrapping_sub(20i16) >= 0i16) {
                break;
            }
        }
        set_ram_word!(game.bomb_counter = 5u16);
        set_ram_word!(
            game.cooldown_timer =
                kNonBeamProjectileCooldowns[(*((ram_ptr!(game, u16; 0xc18)).offset(5) as *mut u8)
                    .offset(1isize,) as i32
                    & 0xfi32) as usize] as u16
        );
        set_ram_words! { game, 0u16; bomb_spread_charge_timeout_counter, flare_counter }
    }
}
static kProjPreInstr_SpazerSba_FuncsB: [Option<Func_SpazerSba_V>; 3] = callback_table! { unsafe Func_SpazerSba_V;
    ProjPreInstr_SpazerSba_FuncB_0, ProjPreInstr_SpazerSba_FuncB_1,
    ProjPreInstr_SpazerSba_FuncB_2,
};
static kProjPreInstr_SpazerSba_FuncsA: [Option<Func_Y_V>; 4] = callback_table! { unsafe Func_Y_V;
    ProjPreInstr_SpazerSba_FuncA_0, ProjPreInstr_SpazerSba_FuncA_1,
    ProjPreInstr_SpazerSba_FuncA_2, ProjPreInstr_SpazerSba_FuncA_3,
};
pub unsafe fn ProjPreInstr_SpazerSba(mut k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    if (ram_word!(game, projectile_dir[v1]) as i32 & 0xf0i32) != 0i32 {
        kProjPreInstr_SpazerSba_FuncsA[v1 as usize].expect("non-null function pointer")(k, game);
    } else {
        let v2: i32 = k as i32 >> 1i32;
        let ref mut fresh32 = *(ram_ptr!(game, u16; 0xc90)).offset(v2 as isize);
        let fresh33 = *fresh32;
        *fresh32 = (*fresh32).wrapping_sub(1);
        if fresh33 == 1u16 {
            set_ram_word!(game.projectile_timers[v2] = 4u16);
            SpawnProjectileTrail(k, game);
            k = ram_word!(game, projectile_index);
        }
        let v4: i32 = k as i32 >> 1i32;
        let pt: Point16U = Projectile_SinLookup(
            ram_word!(game, projectile_variables[v4]),
            ram_word!(game, projectile_bomb_x_speed[v4]),
        );
        set_ram_word!(
            game.projectile_x_pos[v4] = (pt.x).wrapping_add(ram_word!(game, samus_x_pos))
        );
        kProjPreInstr_SpazerSba_FuncsB
            [(ram_word!(game, projectile_unk_A[v4]) as i32 >> 1i32) as usize]
            .expect("non-null function pointer")(k, pt.y, game);
        set_ram_word!(game.cooldown_timer = 2u16);
        set_ram_word!(game.flare_counter = 0u16);
    };
}
pub unsafe fn ProjPreInstr_SpazerSba_FuncA_0(k: u16, game: &mut Game) {
    ClearProjectile(k, game);
    ClearProjectile(4u16, game);
}
pub unsafe fn ProjPreInstr_SpazerSba_FuncA_1(k: u16, game: &mut Game) {
    ClearProjectile(k, game);
    ClearProjectile(6u16, game);
}
pub unsafe fn ProjPreInstr_SpazerSba_FuncA_2(k: u16, game: &mut Game) {
    ClearProjectile(k, game);
    ClearProjectile(0u16, game);
}
pub unsafe fn ProjPreInstr_SpazerSba_FuncA_3(k: u16, game: &mut Game) {
    ClearProjectile(k, game);
    ClearProjectile(2u16, game);
}
pub unsafe fn ProjPreInstr_SpazerSba_FuncB_0(j: u16, r22: u16, game: &mut Game) {
    static kProjPreInstr_SpazerSba_Yspeed: [i16; 4] = [2, -2, 2, -2];
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.projectile_y_pos[v1] = (r22).wrapping_add(ram_word!(game, samus_y_pos)));
    let v2: u16 = ((*((ram_ptr!(game, u16; 0xbf0)).offset(v1 as isize) as *mut u8).offset(0))
        .wrapping_add(*((ram_ptr!(game, u16; 0xc7c)).offset(v1 as isize) as *mut u8).offset(0)))
        as u16;
    set_ram_word!(game.projectile_variables[v1] = v2);
    if v2 == 128u16 {
        set_ram_word!(game.projectile_bomb_x_speed[v1] = 160u16);
        set_ram_word!(
            game.projectile_bomb_y_speed[v1] = kProjPreInstr_SpazerSba_Yspeed[v1 as usize] as u16
        );
        set_ram_word!(game.projectile_dir[v1] = 0u16);
        set_ram_word!(game.projectile_unk_A[v1] = 2u16);
    }
}
pub unsafe fn ProjPreInstr_SpazerSba_FuncB_1(j: u16, r22: u16, game: &mut Game) {
    static kSpazerProjectileYSpeed: [i16; 4] = [-2, 2, -2, 2];
    let v1: u16 = (r22 as i32 + ram_word!(game, samus_y_pos) as i32 - 114i32) as u16;
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(game.projectile_y_pos[v2] = v1);
    if v1 as i32 - ram_word!(game, layer1_y_pos) as i32 - 16i32 & 0x8000i32 != 0 {
        FireEndOfSpazerSba(j, game);
    } else {
        set_ram_word!(
            game.projectile_variables[v2] =
                ((*((ram_ptr!(game, u16; 0xbf0)).offset(v2 as isize) as *mut u8).offset(0))
                    .wrapping_add(
                        *((ram_ptr!(game, u16; 0xc7c)).offset(v2 as isize) as *mut u8).offset(0),
                    )) as u16
        );
        let v3: u16 = (ram_word!(game, projectile_bomb_x_speed[v2])).wrapping_sub(5u16);
        set_ram_word!(game.projectile_bomb_x_speed[v2] = v3);
        if v3 == 0 {
            set_ram_word!(
                game.projectile_bomb_y_speed[v2] = kSpazerProjectileYSpeed[v2 as usize] as u16
            );
            set_ram_word!(
                game.projectile_variables[v2] = (((ram_word!(game, projectile_variables[v2]))
                    as u8)
                    .wrapping_add(0x80u8)) as u16
            );
            set_ram_word!(game.projectile_unk_A[v2] = 4u16);
            if j == 0 {
                QueueSfx1_Max6(0x26u16, game);
            }
        }
    };
}
pub unsafe fn ProjPreInstr_SpazerSba_FuncB_2(j: u16, r22: u16, game: &mut Game) {
    let v1: u16 = (r22 as i32 + ram_word!(game, samus_y_pos) as i32 - 114i32) as u16;
    set_ram_word!(game.projectile_y_pos[(j as i32 >> 1i32)] = v1);
    if v1 as i32 - ram_word!(game, layer1_y_pos) as i32 - 16i32 & 0x8000i32 != 0 || {
        let mut v2: i32 = j as i32 >> 1i32;
        set_ram_word!(
            game.projectile_variables[v2] =
                ((*((ram_ptr!(game, u16; 0xbf0)).offset(v2 as isize) as *mut u8).offset(0))
                    .wrapping_add(
                        *((ram_ptr!(game, u16; 0xc7c)).offset(v2 as isize) as *mut u8).offset(0),
                    )) as u16
        );
        let mut v3: u16 = (ram_word!(game, projectile_bomb_x_speed[v2])).wrapping_add(5u16);
        set_ram_word!(game.projectile_bomb_x_speed[v2] = v3);
        v3 as i32 - 96i32 & 0x8000i32 == 0
    } {
        FireEndOfSpazerSba(j, game);
    }
}
pub unsafe fn FireEndOfSpazerSba(j: u16, game: &mut Game) {
    static kFireEndOfSpazerSba: [i16; 4] = [16, 16, -16, -16];
    let v1: i32 = j as i32 >> 1i32;
    update_ram_word!(
        game.projectile_x_pos[v1],
        wrapping_add((kFireEndOfSpazerSba[v1 as usize]) as u16)
    );
    set_ram_word!(game.projectile_dir[v1] = 5u16);
    set_ram_word!(game.projectile_timers[v1] = 4u16);
    set_ram_word!(game.projectile_bomb_pre_instructions[v1] = ((0x90dc9ci32) as u16));
    if ((j) as i16).wrapping_sub(4i16) < 0i16 {
        set_ram_word!(game.projectile_type[(j as i32 >> 1i32)] = -32732i32 as u16);
        InitializeShinesparkEchoOrSpazerSba(j, game);
    }
}
pub unsafe fn ProjPreInstr_EndOfSpazerSba(mut k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    let clear_projectile: bool = ((ram_word!(game, projectile_dir[v1]) as i32 & 0xf0i32) != 0i32)
        || ({
            let ref mut fresh30 = *(ram_ptr!(game, u16; 0xc90)).offset(v1 as isize);
            let fresh31 = *fresh30;
            *fresh30 = (*fresh30).wrapping_sub(1);
            if fresh31 == 1u16 {
                set_ram_word!(game.projectile_timers[v1] = 4u16);
                SpawnProjectileTrail(k, game);
                k = ram_word!(game, projectile_index);
            }
            let mut v3: i32 = 0;
            v3 = k as i32 >> 1i32;
            let mut v4: u16 = 0;
            v4 = (ram_word!(game, projectile_y_pos[v3])).wrapping_add(8u16);
            set_ram_word!(game.projectile_y_pos[v3] = v4);
            (v4 as i32 - ram_word!(game, layer1_y_pos) as i32 - 248i32 & 0x8000i32 == 0)
                || ({
                    set_ram_word!(game.cooldown_timer = 2u16);
                    set_ram_word!(game.flare_counter = 0u16);
                    false
                })
        });
    if clear_projectile {
        ClearProjectile(k, game);
    };
}
static kHandleHudSpecificBehaviorAndProjs: [Option<Func_V>; 28] = callback_table! { unsafe Func_V;
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
    if ram_word!(game, samus_pose) as i32 == kPose_00_FaceF_Powersuit as i32
        || ram_word!(game, samus_pose) as i32 == kPose_9B_FaceF_VariaGravitySuit as i32
        || {
            Samus_HandleCooldown(game);
            HandleSwitchingHudSelection(game);
            kHandleHudSpecificBehaviorAndProjs[*ram_ptr!(game; 0xa1f) as usize]
                .expect("non-null function pointer")(game);
            ram_word!(game, time_is_frozen_flag) == 0
        }
    {
        HandleProjectile(game);
    }
}
pub unsafe fn HudSelectionHandler_Normal(game: &mut Game) {
    static kHudSelectionHandler_Normal: [Option<Func_V>; 7] = callback_table! { unsafe Func_V;
        HudSelectionHandler_NothingOrPowerBombs, HudSelectionHandler_MissilesOrSuperMissiles,
        HudSelectionHandler_MissilesOrSuperMissiles, HudSelectionHandler_NothingOrPowerBombs,
        HudSelectionHandler_Grappling, HudSelectionHandler_Xray, HudSelectionHandler_TurningAround,
    };
    let mut v0: u16 = 0;
    if ram_word!(game, grapple_beam_function) as i32 == 0x9bc4f0i32 & 0xffffi32 {
        if ram_word!(game, time_is_frozen_flag) != 0 {
            v0 = 10u16;
        } else {
            v0 = (2u16).wrapping_mul(ram_word!(game, hud_item_index));
        }
    } else {
        v0 = 8u16;
    }
    kHudSelectionHandler_Normal[(v0 as i32 >> 1i32) as usize].expect("non-null function pointer")(
        game,
    );
}
pub unsafe fn HudSelectionHandler_Grappling(game: &mut Game) {
    GrappleBeamHandler(game);
}
pub unsafe fn HudSelectionHandler_TurningAround(game: &mut Game) {
    if ram_word!(game, new_projectile_direction_changed_pose) != 0 {
        HudSelectionHandler_Normal(game);
    } else if ram_word!(game, grapple_beam_function) as i32 != 0x9bc4f0i32 & 0xffffi32 {
        set_ram_word!(game.grapple_beam_function = ((0x9bc856i32) as u16));
    }
}
pub unsafe fn HudSelectionHandler_CrouchEtcTrans(game: &mut Game) {
    static byte_90DDAA: [u8; 12] = [0, 0, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1];
    let handle_normal_selection: bool = (ram_word!(game, samus_pose) as i32
        - kPose_F1_FaceR_CrouchTrans_AimU as i32
        & 0x8000i32
        == 0)
        || ({
            if ram_word!(game, samus_pose) as i32 - kPose_DB as i32 & 0x8000i32 == 0 {
                return;
            }
            !(byte_90DDAA[((ram_word!(game, samus_pose)).wrapping_sub(53u16)) as usize] != 0)
                || ({
                    if ram_word!(game, grapple_beam_function) as i32 != 0x9bc4f0i32 & 0xffffi32 {
                        set_ram_word!(game.grapple_beam_function = ((0x9bc856i32) as u16));
                        HudSelectionHandler_Normal(game);
                    }
                    false
                })
        });
    if handle_normal_selection {
        HudSelectionHandler_Normal(game);
    };
}
pub unsafe fn HudSelectionHandler_JumpEtc(game: &mut Game) {
    if ram_word!(game, grapple_beam_function) as i32 != 0x9bc4f0i32 & 0xffffi32 {
        set_ram_word!(game.grapple_beam_function = ((0x9bc856i32) as u16));
        HudSelectionHandler_Normal(game);
    }
}
pub unsafe fn HudSelectionHandler_Xray(game: &mut Game) {
    if (ram_word!(game, button_config_run_b) as i32 & ram_word!(game, joypad1_lastkeys) as i32)
        != 0i32
    {
        XrayRunHandler(game);
    } else {
        HudSelectionHandler_NothingOrPowerBombs(game);
    };
}
pub unsafe fn HudSelectionHandler_GrabbedByDraygon(game: &mut Game) {
    if ram_word!(game, samus_pose) as i32 == kPose_DB as i32 | kPose_04_FaceL_AimU as i32 {
        HudSelectionHandler_MorphBall(game);
    } else {
        HudSelectionHandler_Normal(game);
    };
}
static kSamusHitInterrupts: [Option<Func_U8>; 28] = callback_table! { unsafe Func_U8;
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
    if ram_word!(game, samus_knockback_timer) != 0 {
        if ram_word!(game, debug_invincibility) as i32 - 7i32 & 0x8000i32 != 0 {
            if ram_word!(game, time_is_frozen_flag) == 0
                && ram_word!(game, knockback_dir) == 0
                && kSamusHitInterrupts[*ram_ptr!(game; 0xa1f) as usize]
                    .expect("non-null function pointer")(game) as i32
                    & 1i32
                    != 0
            {
                set_ram_word!(game.samus_special_transgfx_index = 1u16);
            }
        } else {
            set_ram_words! { game, 0u16; samus_invincibility_timer, samus_knockback_timer }
        }
    } else if ram_word!(game, knockback_dir) != 0 {
        if *ram_ptr!(game; 0xa1f) as i32 == kPose_0A_MoveL_NoAim as i32 {
            if ram_word!(game, flare_counter) as i32 - 16i32 & 0x8000i32 == 0 {
                QueueSfx1_Max6(0x41u16, game);
            }
            if *ram_ptr!(game; 0xa1e) == 4u8 {
                set_ram_word!(game.samus_new_pose_transitional = kPose_2A_FaceL_Fall as u16);
            } else {
                set_ram_word!(game.samus_new_pose_transitional = kPose_29_FaceR_Fall as u16);
            }
        } else {
            if ram_word!(game, samus_hurt_switch_index) == 3u16 {
                set_ram_word!(game.samus_hurt_switch_index = 8u16);
                return;
            }
            set_ram_word!(game.samus_new_pose_transitional = ram_word!(game, samus_pose));
        }
        set_ram_word!(game.samus_hurt_switch_index = 1u16);
    } else if ram_word!(game, bomb_jump_dir) != 0 {
        SetupBombJump(game);
    }
}
pub unsafe fn Samus_HitInterrupt_Shinespark(game: &mut Game) -> u8 {
    if ram_word!(game, samus_pose) as i32 == kPose_E8_FaceR_Drained_CrouchFalling as i32
        || ram_word!(game, samus_pose) as i32 == kPose_E9_FaceL_Drained_CrouchFalling as i32
    {
        set_ram_word!(game.samus_anim_frame_timer = 17u16);
        set_ram_word!(game.samus_anim_frame = 26u16);
    }
    set_ram_words! { game, 0u16; samus_special_transgfx_index, knockback_dir }
    0u8
}
pub unsafe fn Samus_HitInterrupt_KnockbackGrapple(game: &mut Game) -> u8 {
    set_ram_word!(game.samus_special_transgfx_index = 0u16);
    0u8
}
pub unsafe fn Samus_HitInterrupt_Turning(game: &mut Game) -> u8 {
    set_ram_words! { game, 0u16; samus_special_transgfx_index, knockback_dir }
    0u8
}
pub unsafe fn Samus_HitInterrupt_Falling(game: &mut Game) -> u8 {
    if ram_word!(game, frame_handler_gamma) as i32 != 0x90e41bi32 & 0xffffi32 {
        return Samus_HitInterrupt_Stand(game);
    }
    set_ram_words! { game, 0u16; samus_special_transgfx_index, knockback_dir }
    0u8
}
pub unsafe fn Samus_HitInterrupt_Stand(game: &mut Game) -> u8 {
    if *ram_ptr!(game; 0xa1e) == 4u8 {
        set_ram_word!(game.samus_new_pose_interrupted = 84u16);
    } else {
        set_ram_word!(game.samus_new_pose_interrupted = 83u16);
    }
    1u8
}
pub unsafe fn Samus_HitInterrupt_Ball(game: &mut Game) -> u8 {
    set_ram_word!(game.samus_new_pose_interrupted = ram_word!(game, samus_pose));
    1u8
}
pub unsafe fn Samus_HitInterrupt_Unused(game: &mut Game) -> u8 {
    if *ram_ptr!(game; 0xa1e) == 4u8 {
        set_ram_word!(game.samus_new_pose_interrupted = 52u16);
    } else {
        set_ram_word!(game.samus_new_pose_interrupted = 51u16);
    }
    1u8
}
static kSamus_MoveHandler_Knockback: [Option<Func_V>; 6] = callback_table! { unsafe Func_V;
    Samus_MoveHandler_Knockback_0, Samus_MoveHandler_Knockback_Up,
    Samus_MoveHandler_Knockback_Up, Samus_MoveHandler_Knockback_3,
    Samus_MoveHandler_Knockback_Down, Samus_MoveHandler_Knockback_Down,
};
pub unsafe fn Samus_MoveHandler_Knockback(game: &mut Game) {
    kSamus_MoveHandler_Knockback[ram_word!(game, knockback_dir) as usize]
        .expect("non-null function pointer")(game);
    set_ram_word!(game.input_to_pose_calc = 0u16);
}
pub unsafe fn Samus_MoveHandler_Knockback_0(game: &mut Game) {
    InvalidInterrupt_Crash(game);
}
pub unsafe fn Samus_MoveHandler_Knockback_Up(game: &mut Game) {
    Samus_BombJumpRisingXMovement_(game);
    Samus_MoveY_WithSpeedCalc(game);
    Samus_ClearMoveVars(game);
}
pub unsafe fn Samus_MoveHandler_Knockback_3(game: &mut Game) {
    Samus_MoveY_WithSpeedCalc(game);
    Samus_ClearMoveVars(game);
}
pub unsafe fn Samus_MoveHandler_Knockback_Down(game: &mut Game) {
    Samus_BombJumpRisingXMovement_(game);
    Samus_Move_NoSpeedCalc_Y(game);
    Samus_ClearMoveVars(game);
}
pub unsafe fn Samus_ClearMoveVars(game: &mut Game) {
    if ram_word!(game, samus_collision_flag) != 0 {
        set_ram_words! { game, 0u16;
            samus_x_accel_mode, samus_collides_with_solid_enemy, samus_is_falling_flag,
            UNUSED_word_7E0B1A, UNUSED_word_7E0B2A, samus_y_subspeed, samus_y_speed, samus_y_dir,
            UNUSED_word_7E0B38, samus_x_base_speed, samus_x_base_subspeed,
        }
        Samus_AlignBottomWithPrevPose(game);
    }
}
static kSetupBombJump: [Option<Func_U8>; 28] = callback_table! { unsafe Func_U8;
    SetupBombJump_StandCrouch, SetupBombJump_1, SetupBombJump_2, SetupBombJump_2,
    SetupBombJump_4, SetupBombJump_StandCrouch, SetupBombJump_1, SetupBombJump_4,
    SetupBombJump_4, SetupBombJump_4, SetupBombJump_4, SetupBombJump_1, SetupBombJump_1,
    SetupBombJump_1, SetupBombJump_2, SetupBombJump_2, SetupBombJump_1, SetupBombJump_4,
    SetupBombJump_4, SetupBombJump_4, SetupBombJump_1, SetupBombJump_1, SetupBombJump_1,
    SetupBombJump_2, SetupBombJump_2, SetupBombJump_2, SetupBombJump_1A, SetupBombJump_1A,
};
pub unsafe fn SetupBombJump(game: &mut Game) {
    if *(ram_ptr!(game, u16; 0xa56) as *mut u8).offset(1isize) == 0
        && kSetupBombJump[*ram_ptr!(game; 0xa1f) as usize].expect("non-null function pointer")(game)
            as i32
            & 1i32
            != 0
    {
        set_ram_word!(game.samus_special_transgfx_index = 3u16);
    }
}
pub unsafe fn SetupBombJump_StandCrouch(game: &mut Game) -> u8 {
    if ram_word!(game, time_is_frozen_flag) == 0 {
        return SetupBombJump_1(game);
    }
    set_ram_word!(game.bomb_jump_dir = 0u16);
    0u8
}
pub unsafe fn SetupBombJump_1(game: &mut Game) -> u8 {
    if *ram_ptr!(game; 0xa1e) == 4u8 {
        set_ram_word!(game.samus_new_pose_interrupted = kPose_52_FaceL_Jump_NoAim_MoveF as u16);
    } else {
        set_ram_word!(game.samus_new_pose_interrupted = kPose_51_FaceR_Jump_NoAim_MoveF as u16);
    }
    1u8
}
pub unsafe fn SetupBombJump_4(game: &mut Game) -> u8 {
    set_ram_word!(game.samus_new_pose_interrupted = ram_word!(game, samus_pose));
    1u8
}
pub unsafe fn SetupBombJump_2(game: &mut Game) -> u8 {
    set_ram_word!(game.samus_input_handler = ((0x90e913i32) as u16));
    SetupBombJump_1A(game)
}
pub unsafe fn SetupBombJump_1A(game: &mut Game) -> u8 {
    set_ram_word!(game.bomb_jump_dir = 0u16);
    0u8
}
pub unsafe fn Samus_MoveHandler_BombJumpStart(game: &mut Game) {
    Samus_InitBombJump(game);
    set_ram_word!(game.samus_movement_handler = ((0x90e032i32) as u16));
    set_ram_word!(game.input_to_pose_calc = 0u16);
}
static kSamus_MoveHandler_BombJumpMain: [Option<Func_V>; 4] = callback_table! { unsafe Func_V;
    Samus_MoveHandler_Knockback_0, Samus_HorizontalBombJump, Samus_VerticalBombJump,
    Samus_HorizontalBombJump,
};
pub unsafe fn Samus_MoveHandler_BombJumpMain(game: &mut Game) {
    if ram_word!(game, bomb_jump_dir) != 0 {
        kSamus_MoveHandler_BombJumpMain[ram_word!(game, bomb_jump_dir) as u8 as usize]
            .expect("non-null function pointer")(game);
    } else {
        Samus_MoveHandler_BombJumpFunc1(game);
    };
}
pub unsafe fn Samus_HorizontalBombJump(game: &mut Game) {
    Samus_BombJumpRisingXMovement_(game);
    Samus_BombJumpRisingYMovement_(game);
    if ram_word!(game, samus_y_dir) == 2u16 || {
        Samus_MoveY_WithSpeedCalc(game);
        ram_word!(game, samus_collision_flag) != 0u16
    } {
        Samus_MoveHandler_BombJumpFunc1(game);
    }
}
pub unsafe fn Samus_VerticalBombJump(game: &mut Game) {
    Samus_BombJumpRisingYMovement_(game);
    if ram_word!(game, samus_y_dir) == 2u16 || {
        Samus_MoveY_WithSpeedCalc(game);
        ram_word!(game, samus_collision_flag) != 0u16
    } {
        Samus_MoveHandler_BombJumpFunc1(game);
    }
}
pub unsafe fn Samus_MoveHandler_BombJumpFunc1(game: &mut Game) {
    set_ram_word!(game.samus_movement_handler = ((0x90a337i32) as u16));
    if ram_word!(game, samus_input_handler) as i32 != 0x90e91di32 & 0xffffi32 {
        set_ram_word!(game.samus_input_handler = ((0x90e913i32) as u16));
    }
    set_ram_word!(game.bomb_jump_dir = 0u16);
}
crate::game::address_dispatch! {
    pub unsafe fn CallFrameHandlerGamma(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        9494683 => Samus_Func1; 9494725 => Samus_Func2; 9494758 => Samus_Func3;
        9494804 => DrawTimer_; 9494830 => Samus_PushOutOfRidleysWay; 9494984 => Samus_Func4;
        9495201 => Samus_GrabbedByDraygonFrameHandler; 9495422 => [()]; 9495459 => Samus_Func7;
        9495579 => Samus_Func9; 9496846 => [()];
    }
}
pub unsafe fn RunFrameHandlerGamma(game: &mut Game) {
    CallFrameHandlerGamma(
        ((ram_word!(game, frame_handler_gamma)) as u32) | (0x900000u32),
        game,
    );
}
pub unsafe fn Samus_Func1(game: &mut Game) {
    if ram_word!(game, samus_pose) as i32 == kPose_E9_FaceL_Drained_CrouchFalling as i32
        && ram_word!(game, samus_anim_frame) as i32 - 8i32 & 0x8000i32 == 0
        && (ram_word!(game, joypad1_newkeys) as i32 & kButton_Up as i32) != 0i32
    {
        set_ram_word!(game.samus_anim_frame_timer = 1u16);
        set_ram_word!(game.samus_anim_frame = 13u16);
        set_ram_word!(game.frame_handler_gamma = ((0x90e90ei32) as u16));
    }
}
pub unsafe fn Samus_Func2(game: &mut Game) {
    if ram_word!(game, samus_anim_frame) as i32 - 8i32 & 0x8000i32 == 0
        && ram_word!(game, samus_anim_frame) as i32 - 12i32 & 0x8000i32 != 0
        && (ram_word!(game, joypad1_newkeys) as i32 & kButton_Up as i32) != 0i32
    {
        set_ram_word!(game.samus_anim_frame_timer = 1u16);
        set_ram_word!(game.samus_anim_frame = 18u16);
    }
}
pub unsafe fn Samus_Func3(game: &mut Game) {
    if ProcessTimer(game) as i32 & 1i32 != 0 {
        set_ram_word!(game.game_state = kGameState_35_TimeUp as u16);
        let mut i: i32 = 510i32;
        while i >= 0i32 {
            set_ram_word!(game.target_palettes[(i >> 1i32)] = 0x7fffu16);
            i -= 2i32;
        }
        set_ram_word!(game.frame_handler_gamma = ((0x90e114i32) as u16));
        DisablePaletteFx(game);
    }
    if ram_word!(game, timer_status) != 0 {
        DrawTimer(game);
    }
}
pub unsafe fn DrawTimer_(game: &mut Game) {
    DrawTimer(game);
}
pub unsafe fn Samus_SetPushedOutOfCeresRidley(game: &mut Game) {
    set_ram_word!(game.samus_movement_handler = ((0x90e90ei32) as u16));
    set_ram_word!(game.frame_handler_gamma = ((0x90e12ei32) as u16));
}
pub unsafe fn Samus_PushOutOfRidleysWay(game: &mut Game) {
    if *ram_ptr!(game; 0xa1e) == 4u8 {
        set_ram_word!(game.samus_pose = kPose_54_FaceL_Knockback as u16);
    } else {
        set_ram_word!(game.samus_pose = kPose_53_FaceR_Knockback as u16);
    }
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    set_ram_word!(game.samus_last_different_pose = ram_word!(game, samus_prev_pose));
    *(ram_ptr!(game, u16; 0xa26)) = *(ram_ptr!(game, u16; 0xa22));
    set_ram_word!(game.samus_prev_pose = ram_word!(game, samus_pose));
    *(ram_ptr!(game, u16; 0xa22)) = *(ram_ptr!(game, u16; 0xa1e));
    update_ram_word!(game.samus_y_pos, |fresh131| (fresh131 as i32
        - (21i32 - ram_word!(game, samus_y_radius) as i32))
        as u16);
    if ram_word!(game, samus_x_pos) as i32 - ram_word!(game, layer1_x_pos) as i32 - 128i32
        & 0x8000i32
        != 0
    {
        set_ram_word!(game.samus_var62 = 1u16);
    } else {
        set_ram_word!(game.samus_var62 = 2u16);
    }
    set_ram_word!(game.samus_y_speed = 5u16);
    set_ram_words! { game, 0u16; samus_y_subspeed, bomb_jump_dir }
    set_ram_word!(game.frame_handler_gamma = ((0x90e1c8i32) as u16));
    set_ram_word!(game.samus_new_pose = -1i32 as u16);
    set_ram_word!(game.samus_new_pose_interrupted = -1i32 as u16);
    set_ram_word!(game.samus_new_pose_transitional = -1i32 as u16);
    set_ram_words! { game, 0u16;
        samus_momentum_routine_index, samus_special_transgfx_index, samus_hurt_switch_index,
    }
    ProcessTimer(game);
    if ram_word!(game, timer_status) != 0 {
        DrawTimer(game);
    }
}
pub unsafe fn Samus_Func4(game: &mut Game) {
    static off_90E1F7: [Option<Func_V>; 3] = callback_table! { unsafe Func_V;
        None, Samus_Func5, Samus_Func6,
    };
    if ram_word!(game, samus_new_pose) as i32 == kPose_4F_FaceL_Dmgboost as i32
        || ram_word!(game, samus_new_pose) as i32 == kPose_50_FaceR_Dmgboost as i32
    {
        set_ram_word!(game.samus_new_pose = -1i32 as u16);
        set_ram_word!(game.samus_momentum_routine_index = 0u16);
    }
    off_90E1F7[ram_word!(game, samus_var62) as usize].expect("non-null function pointer")(game);
    set_ram_word!(game.input_to_pose_calc = 0u16);
    ProcessTimer(game);
    if ram_word!(game, timer_status) != 0 {
        DrawTimer(game);
    }
}
pub unsafe fn Samus_Func5(game: &mut Game) {
    Samus_BombJumpFallingXMovement_(game);
    if ram_word!(game, samus_collision_flag) != 0 {
        set_ram_word!(game.samus_movement_handler = ((0x90a337i32) as u16));
        set_ram_word!(game.frame_handler_gamma = ((0x90e0e6i32) as u16));
        set_ram_word!(game.samus_var62 = 0u16);
        Samus_ClearMoveVars(game);
    } else {
        Samus_BombJumpFallingYMovement_(game);
    };
}
pub unsafe fn Samus_Func6(game: &mut Game) {
    Samus_BombJumpFallingXMovement_(game);
    if ram_word!(game, samus_collision_flag) != 0 {
        set_ram_word!(game.samus_movement_handler = ((0x90a337i32) as u16));
        set_ram_word!(game.frame_handler_gamma = ((0x90e0e6i32) as u16));
        set_ram_word!(game.samus_var62 = 0u16);
        Samus_ClearMoveVars(game);
    } else {
        Samus_BombJumpFallingYMovement_(game);
    };
}
pub unsafe fn Samus_GrabbedByDraygonFrameHandler(game: &mut Game) {
    if ram_word!(game, grapple_beam_function) as i32 == 0x9bc77ei32 & 0xffffi32 {
        set_ram_word!(game.samus_new_pose = -1i32 as u16);
        set_ram_word!(game.samus_momentum_routine_index = 0u16);
    }
    if ram_word!(game, joypad1_newkeys) as i32
        & (kButton_Up as i32 | kButton_Down as i32 | kButton_Left as i32 | kButton_Right as i32)
        != 0i32
        && ram_word!(game, joypad1_newkeys) as i32
            & (kButton_Up as i32 | kButton_Down as i32 | kButton_Left as i32 | kButton_Right as i32)
            != ram_word!(game, suit_pickup_light_beam_pos) as i32
    {
        set_ram_word!(
            game.suit_pickup_light_beam_pos = (ram_word!(game, joypad1_newkeys) as i32
                & (kButton_Up as i32
                    | kButton_Down as i32
                    | kButton_Left as i32
                    | kButton_Right as i32)) as u16
        );
        if ((update_ram_word!(game.substate, wrapping_add(1), reread)) as i16).wrapping_sub(60i16)
            >= 0i16
        {
            Samus_ReleaseFromDraygon(game);
        }
    }
}
pub unsafe fn Samus_SetGrabbedByDraygonPose(a: u16, game: &mut Game) {
    if (a as i32 & 1i32) != 0i32 {
        set_ram_word!(game.samus_pose = kPose_EC_FaceR_Draygon_NoMove_NoAim as u16);
    } else {
        set_ram_word!(game.samus_pose = kPose_BA_FaceL_Draygon_NoMove_NoAim as u16);
    }
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    set_ram_word!(game.frame_handler_gamma = ((0x90e2a1i32) as u16));
    set_ram_word!(game.samus_movement_handler = ((0x90e90ei32) as u16));
    set_ram_words! { game, 0u16; substate, suit_pickup_light_beam_pos }
    *(ram_ptr!(game, u16; 0xdf0)) = 0u16;
    set_ram_word!(game.samus_last_different_pose = ram_word!(game, samus_prev_pose));
    *(ram_ptr!(game, u16; 0xa26)) = *(ram_ptr!(game, u16; 0xa22));
    set_ram_word!(game.samus_prev_pose = ram_word!(game, samus_pose));
    *(ram_ptr!(game, u16; 0xa22)) = *(ram_ptr!(game, u16; 0xa1e));
    set_ram_word!(game.samus_new_pose = -1i32 as u16);
    set_ram_word!(game.samus_new_pose_interrupted = -1i32 as u16);
    set_ram_word!(game.samus_new_pose_transitional = -1i32 as u16);
    set_ram_words! { game, 0u16;
        samus_momentum_routine_index, samus_special_transgfx_index, samus_hurt_switch_index,
    }
}
pub unsafe fn Samus_ReleaseFromDraygon_(game: &mut Game) {
    Samus_ReleaseFromDraygon(game);
}
pub unsafe fn Samus_ReleaseFromDraygon(game: &mut Game) {
    if *ram_ptr!(game; 0xa1e) == 4u8 {
        set_ram_word!(game.samus_pose = kPose_02_FaceL_Normal as u16);
    } else {
        set_ram_word!(game.samus_pose = kPose_01_FaceR_Normal as u16);
    }
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    set_ram_word!(game.samus_movement_handler = ((0x90a337i32) as u16));
    set_ram_word!(game.frame_handler_gamma = ((0x90e90ei32) as u16));
    set_ram_word!(game.samus_last_different_pose = ram_word!(game, samus_prev_pose));
    *(ram_ptr!(game, u16; 0xa26)) = *(ram_ptr!(game, u16; 0xa22));
    set_ram_word!(game.samus_prev_pose = ram_word!(game, samus_pose));
    *(ram_ptr!(game, u16; 0xa22)) = *(ram_ptr!(game, u16; 0xa1e));
    set_ram_words! { game, 0u16; samus_x_base_speed, samus_x_base_subspeed }
    set_ram_word!(game.samus_new_pose = -1i32 as u16);
    set_ram_word!(game.samus_new_pose_interrupted = -1i32 as u16);
    set_ram_word!(game.samus_new_pose_transitional = -1i32 as u16);
    set_ram_words! { game, 0u16;
        samus_momentum_routine_index, samus_special_transgfx_index, samus_hurt_switch_index,
        samus_y_speed, samus_y_subspeed, samus_y_dir, used_for_ball_bounce_on_landing,
        samus_x_accel_mode,
    }
    set_ram_word!(
        game.samus_grapple_flags =
            (ram_word!(game, samus_grapple_flags) as i32 & 0xfffdi32 | 2i32) as u16
    );
}
pub unsafe fn Samus_Func7(game: &mut Game) {
    Samus_BombJumpFallingXMovement_(game);
    if ram_word!(game, samus_collision_flag) != 0u16 || {
        Samus_BombJumpFallingYMovement_(game);
        ram_word!(game, samus_collision_flag) != 0u16
    } {
        set_ram_word!(game.samus_movement_handler = ((0x90a337i32) as u16));
        set_ram_word!(game.frame_handler_gamma = ((0x90e90ei32) as u16));
        set_ram_word!(game.samus_var62 = 0u16);
        Samus_ClearMoveVars(game);
        set_ram_word!(game.samus_new_pose_interrupted = 65u16);
        set_ram_word!(game.samus_special_transgfx_index = 0u16);
    }
}
pub unsafe fn Samus_Func8(game: &mut Game) {
    set_ram_words! { game, 0u16; samus_y_subspeed, samus_y_speed }
    set_ram_word!(game.samus_movement_handler = ((0x90a337i32) as u16));
    set_ram_word!(game.frame_handler_gamma = ((0x90e90ei32) as u16));
}
pub unsafe fn Samus_Func9(game: &mut Game) {
    if ram_word!(game, samus_y_speed) as i32 - 5i32 & 0x8000i32 != 0 {
        AddToHiLo(
            ram_ptr!(game, u16; 0xb2e),
            ram_ptr!(game, u16; 0xb2c),
            (ram_word!(game, samus_y_accel) as u32) << 16i32
                | ram_word!(game, samus_y_subaccel) as u32,
        );
    }
    if (ram_word!(game, samus_pose) as i32 == kPose_29_FaceR_Fall as i32
        || ram_word!(game, samus_pose) as i32 == kPose_2A_FaceL_Fall as i32
        || ram_word!(game, samus_pose) as i32 == kPose_67_FaceR_Fall_Gun as i32
        || ram_word!(game, samus_pose) as i32 == kPose_68_FaceL_Fall_Gun as i32)
        && ram_word!(game, samus_y_speed) as i32 - 5i32 & 0x8000i32 == 0
    {
        set_ram_word!(game.samus_anim_frame_timer = 16u16);
        set_ram_word!(game.samus_anim_frame = 4u16);
    }
}
unsafe fn Samus_ClampSpeed(amt: i32) -> i32 {
    let mut r18: u16 = (amt >> 16i32) as u16;
    let r20: u16 = amt as u16;
    let v0: i16 = r18 as i16;
    if v0 < 0i16 {
        if v0 as i32 + 15i32 & 0x8000i32 != 0 {
            r18 = -15i32 as u16;
        }
    } else if v0 as i32 - 16i32 & 0x8000i32 == 0 {
        r18 = 15u16;
    }
    (r18 as i32) << 16i32 | r20 as i32
}
pub unsafe fn Samus_CalcDisplacementMoveLeft(mut amt: i32, game: &mut Game) -> i32 {
    amt = Samus_CalcSpeed_X(amt as u32, game) as i32;
    set_ram_word!(game.samus_collision_direction = 0u16);
    amt = ((ram_word!(game, extra_samus_x_displacement) as u32) << 16i32
        | ram_word!(game, extra_samus_x_subdisplacement) as u32)
        .wrapping_sub(amt as u32) as i32;
    Samus_ClampSpeed(amt)
}
pub unsafe fn Samus_CalcDisplacementMoveRight(mut amt: i32, game: &mut Game) -> i32 {
    amt = Samus_CalcSpeed_X(amt as u32, game) as i32;
    set_ram_word!(game.samus_collision_direction = 1u16);
    amt = (amt as u32).wrapping_add(
        (ram_word!(game, extra_samus_x_displacement) as u32) << 16i32
            | ram_word!(game, extra_samus_x_subdisplacement) as u32,
    ) as i32;
    Samus_ClampSpeed(amt)
}
pub unsafe fn Samus_CalcSpeed_X(mut amt: u32, game: &mut Game) -> u32 {
    amt = amt.wrapping_add(
        (ram_word!(game, samus_x_extra_run_speed) as u32) << 16i32
            | ram_word!(game, samus_x_extra_run_subspeed) as u32,
    );
    amt >>= if ram_word!(game, samus_x_speed_divisor) <= 4u16 {
        ram_word!(game, samus_x_speed_divisor) as i32
    } else {
        4i32
    };
    SetHiLo(ram_ptr!(game, u16; 0xdbc), ram_ptr!(game, u16; 0xdbe), amt);
    amt
}
pub unsafe fn Samus_ClearXSpeedIfColl(game: &mut Game) {
    if ram_word!(game, samus_collision_flag) != 0 {
        if ram_word!(game, samus_collision_direction) != 0 {
            set_ram_word!(game.samus_collides_with_solid_enemy = 8u16);
        } else {
            set_ram_word!(game.samus_collides_with_solid_enemy = 4u16);
        }
        Samus_CancelSpeedBoost(game);
        set_ram_words! { game, 0u16;
            samus_x_extra_run_speed, samus_x_extra_run_subspeed, samus_x_base_speed,
            samus_x_base_subspeed, samus_x_accel_mode,
        }
    } else {
        set_ram_words! { game, 0u16; input_to_pose_calc, samus_collides_with_solid_enemy }
    };
}
pub unsafe fn Samus_MoveUp_SetPoseCalcInput(game: &mut Game) {
    if ram_word!(game, samus_collision_flag) != 0 {
        set_ram_word!(game.input_to_pose_calc = 4u16);
    } else {
        set_ram_word!(game.input_to_pose_calc = 0u16);
    };
}
static kSamus_MoveDown_SetPoseCalcInput_Tab0: [u8; 28] = [
    0, 0, 4, 4, 1, 0, 4, 2, 4, 4, 0, 0, 0, 0, 4, 4, 0, 3, 4, 4, 4, 0, 4, 4, 4, 4, 4, 4,
];
static kSamus_MoveDown_SetPoseCalcInput_Tab1: [u8; 28] = [
    4, 4, 0, 0, 4, 4, 0, 4, 1, 2, 0, 4, 4, 0, 4, 4, 4, 4, 3, 3, 0, 4, 4, 4, 4, 0, 4, 4,
];
pub unsafe fn Samus_MoveDown_SetPoseCalcInput(game: &mut Game) {
    if ram_word!(game, samus_collision_flag) != 0 {
        set_ram_word!(game.input_to_pose_calc = 1u16);
        *(ram_ptr!(game, u16; 0xdc6) as *mut u8).offset(1isize) =
            kSamus_MoveDown_SetPoseCalcInput_Tab1[*ram_ptr!(game; 0xa1f) as usize];
    } else if (ram_word!(game, input_to_pose_calc) as u8) != 5u8 {
        set_ram_word!(game.input_to_pose_calc = 2u16);
        *(ram_ptr!(game, u16; 0xdc6) as *mut u8).offset(1isize) =
            kSamus_MoveDown_SetPoseCalcInput_Tab0[*ram_ptr!(game; 0xa1f) as usize];
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallFrameHandlerAlfa(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        9496213 => Samus_FrameHandlerAlfa_Func11; 9496265 => Samus_FrameHandlerAlfa_Func12;
        9496339 => Samus_FrameHandlerAlfa_Func13; 9496781 => [()];
    }
}
pub unsafe fn HandleControllerInputForGamePhysics(game: &mut Game) {
    CallFrameHandlerAlfa(
        ((ram_word!(game, frame_handler_alfa)) as u32) | (0x900000u32),
        game,
    );
}
pub unsafe fn Samus_FrameHandlerAlfa_Func11(game: &mut Game) {
    set_ram_word!(game.samus_new_pose = -1i32 as u16);
    set_ram_word!(game.samus_new_pose_interrupted = -1i32 as u16);
    set_ram_word!(game.samus_new_pose_transitional = -1i32 as u16);
    set_ram_words! { game, 0u16;
        samus_momentum_routine_index, samus_special_transgfx_index, samus_hurt_switch_index,
    }
    Samus_SetRadius(game);
    Samus_CallInputHandler(game);
    Samus_UpdateSuitPaletteIndex(game);
    Samus_DetermineAccel_Y(game);
    BlockInsideDetection(game);
    Samus_HandleHudSpecificBehaviorAndProjs(game);
    Samus_Func10(game);
}
pub unsafe fn Samus_FrameHandlerAlfa_Func12(game: &mut Game) {
    set_ram_word!(game.samus_new_pose = -1i32 as u16);
    set_ram_word!(game.samus_new_pose_interrupted = -1i32 as u16);
    set_ram_word!(game.samus_new_pose_transitional = -1i32 as u16);
    set_ram_words! { game, 0u16;
        samus_momentum_routine_index, samus_special_transgfx_index, samus_hurt_switch_index,
    }
    set_ram_word!(game.controller1_input_for_demo = ram_word!(game, joypad1_lastkeys));
    set_ram_word!(game.controller1_new_input_for_demo = ram_word!(game, joypad1_newkeys));
    set_ram_word!(
        game.demo_backup_prev_controller_input = ram_word!(game, joypad1_input_samusfilter)
    );
    set_ram_word!(
        game.demo_backup_prev_controller_input_new = ram_word!(game, joypad1_newinput_samusfilter)
    );
    Samus_SetRadius(game);
    Samus_UpdateSuitPaletteIndex(game);
    Samus_CallInputHandler(game);
    Samus_DetermineAccel_Y(game);
    BlockInsideDetection(game);
    Samus_HandleHudSpecificBehaviorAndProjs(game);
    Samus_Func10(game);
}
pub unsafe fn Samus_FrameHandlerAlfa_Func13(game: &mut Game) {
    HandleProjectile(game);
    Samus_Func10(game);
}
crate::game::address_dispatch! {
    pub unsafe fn CallFrameHandlerBeta(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        9496357 => Samus_FrameHandlerBeta_Func17; 9496530 => HandleDemoRecorder_3;
        9496565 => Samus_FrameHandlerBeta_Func14; 9496627 => Samus_Func15; 9496682 => Samus_Func16;
        9496746 => Samus_Func18; 9496781 => [()]; 9496790 => [()]; 9496793 => [()];
        9496796 => SetContactDamageIndexAndUpdateMinimap; 9496812 => Samus_Func19;
        9496834 => Samus_LowHealthCheck;
    }
}
pub unsafe fn HandleSamusMovementAndPause(game: &mut Game) {
    CallFrameHandlerBeta(
        ((ram_word!(game, frame_handler_beta)) as u32) | (0x900000u32),
        game,
    );
}
pub unsafe fn Samus_FrameHandlerBeta_Func17(game: &mut Game) {
    set_ram_word!(game.samus_contact_damage_index = 0u16);
    RunSamusMovementHandler(game);
    UpdateMinimap(game);
    RunFrameHandlerGamma(game);
    Samus_Animate(game);
    Samus_HitInterruption(game);
    Samus_HandleTransFromBlockColl(game);
    Samus_HandleTransitions(game);
    Samus_HandlePalette(game);
    Samus_HandlePeriodicDamage(game);
    Samus_PauseCheck(game);
    Samus_LowHealthCheck_(game);
}
pub unsafe fn HandleDemoRecorder_1(game: &mut Game) {
    if (ram_word!(game, joypad2_new_keys) as i32 & 0x8000i32) == 0i32 {
        if ram_word!(game, debug_flag) == 0
            && (ram_word!(game, joypad2_new_keys) as i32 & 0x80i32) != 0i32
        {
            DisableEprojs(game);
            set_ram_word!(game.time_is_frozen_flag = 1u16);
            set_ram_word!(game.frame_handler_alfa = ((0x90e8cdi32) as u16));
            set_ram_word!(game.frame_handler_beta = ((0x90e7d2i32) as u16));
        }
    } else if ram_word!(game, debug_flag) != 0 {
        set_ram_word!(game.samus_draw_handler = ((0x90e90ei32) as u16));
        set_ram_word!(game.debug_flag = 0u16);
    } else {
        set_ram_word!(game.debug_flag = 1u16);
        set_ram_word!(game.samus_draw_handler = ((0x90eb52i32) as u16));
    };
}
pub unsafe fn HandleDemoRecorder_3(game: &mut Game) {
    if (ram_word!(game, joypad2_new_keys) as i32 & 0x80i32) != 0i32 {
        set_ram_word!(game.frame_handler_alfa = ((0x90e695i32) as u16));
        set_ram_word!(game.frame_handler_beta = ((0x90e725i32) as u16));
        EnableEprojs(game);
        set_ram_word!(game.time_is_frozen_flag = 0u16);
    }
}
pub unsafe fn Samus_FrameHandlerBeta_Func14(game: &mut Game) {
    set_ram_word!(game.samus_contact_damage_index = 0u16);
    RunSamusMovementHandler(game);
    UpdateMinimap(game);
    Samus_Animate(game);
    Samus_HitInterruption(game);
    Samus_HandleTransFromBlockColl(game);
    Samus_HandleTransitions(game);
    Samus_HandlePalette(game);
    set_ram_word!(game.joypad1_lastkeys = ram_word!(game, controller1_input_for_demo));
    set_ram_word!(game.joypad1_newkeys = ram_word!(game, controller1_new_input_for_demo));
    set_ram_word!(
        game.joypad1_input_samusfilter = ram_word!(game, demo_backup_prev_controller_input)
    );
    set_ram_word!(
        game.joypad1_newinput_samusfilter = ram_word!(game, demo_backup_prev_controller_input_new)
    );
}
pub unsafe fn Samus_Func15(game: &mut Game) {
    set_ram_word!(game.samus_contact_damage_index = 0u16);
    RunSamusMovementHandler(game);
    Samus_Animate(game);
    Samus_HitInterruption(game);
    Samus_HandleTransFromBlockColl(game);
    Samus_HandleTransitions(game);
    Samus_HandlePalette(game);
    set_ram_word!(game.joypad1_lastkeys = ram_word!(game, controller1_input_for_demo));
    set_ram_word!(game.joypad1_newkeys = ram_word!(game, controller1_new_input_for_demo));
    set_ram_word!(
        game.joypad1_input_samusfilter = ram_word!(game, demo_backup_prev_controller_input)
    );
    set_ram_word!(
        game.joypad1_newinput_samusfilter = ram_word!(game, demo_backup_prev_controller_input_new)
    );
}
pub unsafe fn Samus_Func16(game: &mut Game) {
    Samus_SetRadius(game);
    UpdateMinimap(game);
    Samus_Animate(game);
    set_ram_word!(game.elevator_status = 0u16);
    set_ram_word!(game.samus_prev_y_pos = ram_word!(game, samus_y_pos));
    if PlaySamusFanfare(game) as i32 & 1i32 != 0 {
        if ram_word!(game, debug_invincibility) as i32 - 7i32 & 0x8000i32 != 0
            || (ram_word!(game, joypad2_last) as i32 & 0x8000i32) == 0i32
        {
            set_ram_word!(game.debug_invincibility = 0u16);
        }
        set_ram_word!(game.frame_handler_alfa = ((0x90e695i32) as u16));
        set_ram_word!(game.frame_handler_beta = ((0x90e725i32) as u16));
    }
}
pub unsafe fn Samus_Func18(game: &mut Game) {
    Samus_FrameHandlerBeta_Func17(game);
    if ram_word!(game, frame_handler_gamma) as i32 == 0x90e114i32 & 0xffffi32
        && ram_word!(game, game_state) as i32 != kGameState_35_TimeUp as i32
    {
        set_ram_word!(game.game_state = kGameState_35_TimeUp as u16);
    }
}
pub unsafe fn SetContactDamageIndexAndUpdateMinimap(game: &mut Game) {
    set_ram_word!(game.samus_contact_damage_index = 0u16);
    UpdateMinimap(game);
}
pub unsafe fn Samus_Func19(game: &mut Game) {
    set_ram_word!(game.samus_contact_damage_index = 0u16);
    RunSamusMovementHandler(game);
    UpdateMinimap(game);
    Samus_Animate(game);
}
pub unsafe fn Samus_LowHealthCheck(game: &mut Game) {
    Samus_LowHealthCheck_(game);
}
crate::game::address_dispatch! {
    pub unsafe fn CallSamusInputHandler(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        9496846 => [()]; 9496851 => Samus_InputHandler_E913; 9496856 => Samus_Func20_;
        9496861 => Samus_InputHandler_E91D; 9496870 => HandleAutoJumpHack;
    }
}
pub unsafe fn Samus_CallInputHandler(game: &mut Game) {
    CallSamusInputHandler(
        ((ram_word!(game, samus_input_handler)) as u32) | (0x900000u32),
        game,
    );
}
pub unsafe fn Samus_InputHandler_E913(game: &mut Game) {
    Samus_InputHandler(game);
}
pub unsafe fn Samus_Func20_(game: &mut Game) {
    Samus_Func20(game);
}
pub unsafe fn Samus_InputHandler_E91D(game: &mut Game) {
    DemoObjectInputHandler(game);
    Samus_InputHandler(game);
}
pub unsafe fn HandleAutoJumpHack(game: &mut Game) {
    let v0: u16 = ram_word!(game, joypad1_newkeys);
    if ram_word!(game, autojump_timer) != 0u16
        && ram_word!(game, autojump_timer) as i32 - 9i32 & 0x8000i32 != 0
    {
        update_ram_word!(game.joypad1_newkeys, |fresh26| (fresh26)
            | (ram_word!(game, button_config_jump_a)));
        set_ram_word!(game.autojump_timer = 0u16);
    }
    Samus_InputHandler(game);
    set_ram_word!(game.joypad1_newkeys = v0);
    set_ram_word!(game.samus_input_handler = ((0x90e913i32) as u16));
}
crate::game::address_dispatch! {
    pub unsafe fn CallSamusMovementHandler(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        9475182 => Samus_MoveHandler_ReleaseFromGrapple;
        9475275 => Samus_HandleMovement_DrainedCrouching; 9478967 => Samus_MovementHandler_Normal;
        9490536 => Samus_MoveHandlerShinesparkWindup;
        9490603 => Samus_MoveHandlerVerticalShinespark;
        9490647 => Samus_MoveHandler_Shinespark_Diag; 9490694 => Samus_MoveHandler_Shinespark_Horiz;
        9491270 => Samus_MoveHandler_ShinesparkCrash;
        9491331 => Samus_MoveHandler_ShinesparkCrash_0;
        9491350 => Samus_MoveHandler_ShinesparkCrash_1;
        9491404 => Samus_MoveHandler_ShinesparkCrash_2;
        9491443 => Samus_MoveHandler_ShinesparkCrashEchoCircle;
        9491469 => Samus_MoveHandler_ShinesparkCrashFinish;
        9492088 => SamusMoveHandler_CrystalFlashStart; 9492174 => SamusMoveHandler_CrystalFlashMain;
        9492315 => kSamusMoveHandler_CrystalFlashFinish; 9494328 => Samus_MoveHandler_Knockback;
        9494352 => Samus_MoveHandler_Knockback_0; 9494355 => Samus_MoveHandler_Knockback_Up;
        9494365 => Samus_MoveHandler_Knockback_3; 9494372 => Samus_MoveHandler_Knockback_Down;
        9494565 => Samus_MoveHandler_BombJumpStart; 9494578 => Samus_MoveHandler_BombJumpMain;
        9494653 => Samus_MoveHandler_BombJumpFunc1; 9496846 => [()];
        9496911 => SamusMovementType_Xray; 9498699 => Samus_Func25_ShineSpark;
        9498738 => Samus_MoveHandler_F072;
    }
}
pub unsafe fn RunSamusMovementHandler(game: &mut Game) {
    CallSamusMovementHandler(
        ((ram_word!(game, samus_movement_handler)) as u32) | (0x900000u32),
        game,
    );
}
pub unsafe fn SamusMovementType_Xray(game: &mut Game) {
    let mut v0: u16 = 0;
    if *ram_ptr!(game; 0xa1f) as i32 != kMovementType_0E_TurningAroundOnGround as i32 {
        set_ram_word!(game.samus_anim_frame_timer = 15u16);
        if *ram_ptr!(game; 0xa1e) == 4u8 {
            if ram_word!(game, xray_angle) as i32 - 153i32 & 0x8000i32 != 0 {
                v0 = 4u16;
            } else if ram_word!(game, xray_angle) as i32 - 178i32 & 0x8000i32 != 0 {
                v0 = 3u16;
            } else if ram_word!(game, xray_angle) as i32 - 203i32 & 0x8000i32 != 0 {
                v0 = 2u16;
            } else {
                v0 = ((ram_word!(game, xray_angle) as i32 - 228i32 & 0x8000i32) != 0i32) as u16;
            }
        } else if ram_word!(game, xray_angle) as i32 - 25i32 & 0x8000i32 != 0 {
            v0 = 0u16;
        } else if ram_word!(game, xray_angle) as i32 - 50i32 & 0x8000i32 != 0 {
            v0 = 1u16;
        } else if ram_word!(game, xray_angle) as i32 - 75i32 & 0x8000i32 != 0 {
            v0 = 2u16;
        } else if ram_word!(game, xray_angle) as i32 - 100i32 & 0x8000i32 != 0 {
            v0 = 3u16;
        } else {
            v0 = 4u16;
        }
        set_ram_word!(game.samus_anim_frame = v0);
    }
}
pub unsafe fn Samus_HandlePeriodicDamage(game: &mut Game) {
    if ram_word!(game, time_is_frozen_flag) != 0 {
        let ref mut fresh127 = *(ram_ptr!(game, u16; 0xa4e));
        *fresh127 = 0u16;
        set_ram_word!(game.samus_periodic_damage = *fresh127);
        return;
    }
    let mut t: i32 = ((ram_word!(game, samus_periodic_damage) as u32) << 16i32
        | ram_word!(game, samus_periodic_subdamage) as u32) as i32;
    if t < 0i32 {
        InvalidInterrupt_Crash(game);
        return;
    }
    if (ram_word!(game, equipped_items) as i32 & 0x20i32) != 0i32 {
        t = t >> 2i32 & 0xffff00i32;
    } else if (ram_word!(game, equipped_items) as i32 & 1i32) != 0i32 {
        t = t >> 1i32 & 0xffff00i32;
    }
    AddToHiLo(
        ram_ptr!(game, u16; 0x9c2),
        ram_ptr!(game, u16; 0xa4c),
        -t as u32,
    );
    if (ram_word!(game, samus_health) as i16) < 0i16 {
        let ref mut fresh128 = *(ram_ptr!(game, u16; 0xa4c));
        *fresh128 = 0u16;
        set_ram_word!(game.samus_health = *fresh128);
    }
    let ref mut fresh129 = *(ram_ptr!(game, u16; 0xa4e));
    *fresh129 = 0u16;
    set_ram_word!(game.samus_periodic_damage = *fresh129);
}
pub unsafe fn Samus_PauseCheck(game: &mut Game) {
    if ram_word!(game, power_bomb_flag) == 0
        && ram_word!(game, time_is_frozen_flag) == 0
        && ram_word!(game, door_transition_flag_enemies) == 0
        && ram_word!(game, area_index) != 6u16
        && ram_word!(game, game_state) as i32 == kGameState_8_MainGameplay as i32
        && (ram_word!(game, joypad1_newkeys) as i32 & kButton_Start as i32) != 0i32
    {
        set_ram_words! { game, 1u16; screen_fade_delay, screen_fade_counter }
        set_ram_word!(game.game_state = kGameState_12_Pausing as u16);
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
pub unsafe fn Samus_LowHealthCheck_0(game: &mut Game) {
    Samus_LowHealthCheck_(game);
}
pub unsafe fn Samus_JumpCheck(game: &mut Game) {
    if (ram_word!(game, button_config_jump_a) as i32 & ram_word!(game, joypad1_lastkeys) as i32)
        != 0i32
        && (ram_word!(game, button_config_jump_a) as i32
            & ram_word!(game, joypad1_input_samusfilter) as i32)
            != 0i32
    {
        update_ram_word!(game.autojump_timer, wrapping_add(1));
    } else {
        set_ram_word!(game.autojump_timer = 0u16);
    }
    set_ram_word!(game.joypad1_input_samusfilter = ram_word!(game, joypad1_lastkeys));
    set_ram_word!(game.joypad1_newinput_samusfilter = ram_word!(game, joypad1_newkeys));
    let update_health_reference: bool = (((ram_word!(game, samus_health)) as i16)
        .wrapping_sub((ram_word!(game, samus_prev_health_for_flash)) as i16)
        >= 0i16)
        || ({
            if ram_word!(game, samus_hurt_flash_counter) == 0 {
                set_ram_word!(game.samus_hurt_flash_counter = 1u16);
            }
            (ram_word!(game, debug_invincibility) as i32 - 7i32 & 0x8000i32 != 0)
                || ({
                    set_ram_word!(game.samus_health = ram_word!(game, samus_prev_health_for_flash));
                    false
                })
        });
    if update_health_reference {
        set_ram_word!(game.samus_prev_health_for_flash = ram_word!(game, samus_health));
    };
}
pub unsafe fn Samus_Func10(game: &mut Game) {
    set_ram_words! { game, 0u16;
        projectile_init_speed_samus_moved_left, projectile_init_speed_samus_moved_left_fract,
        projectile_init_speed_samus_moved_right, projectile_init_speed_samus_moved_right_fract,
        projectile_init_speed_samus_moved_up, projectile_init_speed_samus_moved_up_fract,
        projectile_init_speed_samus_moved_down, projectile_init_speed_samus_moved_down_fract,
        samus_anim_frame_skip, new_projectile_direction_changed_pose,
    }
    update_ram_word!(
        game.UNUSED_word_7E0DFA,
        |fresh27| ((fresh27 as i32) << 8i32) as u16
    );
    *(ram_ptr!(game, u16; 0xa10)) = *(ram_ptr!(game, u16; 0xa1e));
}
pub unsafe fn DrawSamusAndProjectiles(game: &mut Game) {
    SamusDrawSprites(game);
    DrawPlayerExplosions2(game);
    Samus_JumpCheck(game);
    Samus_ShootCheck(game);
}
crate::game::address_dispatch! {
    pub unsafe fn CallSamusDrawHandler(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        9497426 => SamusDrawHandler_Default; 9497478 => sub_90EB86; 9496846 | 9497586 => [()];
        9497587 => Samus_DrawHandler_EndOfShinespark; 9497620 => SamusDisplayHandler_UsingElevator;
        9497629 => SamusDisplayHandler_SamusReceivedFatal;
    }
}
pub unsafe fn SamusDrawSprites(game: &mut Game) {
    HandleArmCannonOpenState(game);
    CallSamusDrawHandler(
        ((ram_word!(game, samus_draw_handler)) as u32) | (0x900000u32),
        game,
    );
}
pub unsafe fn SamusDrawHandler_Default(game: &mut Game) {
    HandleChargingBeamGfxAudio(game);
    sub_90EB55(game);
}
pub unsafe fn sub_90EB55(game: &mut Game) {
    if (ram_word!(game, arm_cannon_drawing_mode) as i32 & 0xfi32) != 0i32 {
        if (ram_word!(game, arm_cannon_drawing_mode) as i32 & 0xfi32) == 2i32 {
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
pub unsafe fn sub_90EB86(game: &mut Game) {
    if ram_word!(game, grapple_beam_function) as i32 + 0x37aai32 & 0x8000i32 == 0 {
        sub_90EB55(game);
        return;
    }
    if ram_word!(game, grapple_beam_function) as i32 == 0x9bc703i32 & 0xffffi32
        || ram_word!(game, grapple_beam_function) as i32 == 0x9bc759i32 & 0xffffi32
    {
        GrappleBeamFunc_BF1B(game);
    }
    HandleGrappleBeamFlare(game);
    if (ram_word!(game, arm_cannon_drawing_mode) as i32 & 0xfi32) != 0i32 {
        if (ram_word!(game, arm_cannon_drawing_mode) as i32 & 0xfi32) == 2i32 {
            HandleAtmosphericEffects(game);
            Samus_Draw(game);
            Samus_ArmCannon_Draw(game);
            UpdateGrappleBeamTilesAndIncrFlameCtr(game);
            if ram_word!(game, grapple_beam_length) == 0 {
                return;
            }
        } else {
            HandleAtmosphericEffects(game);
            Samus_ArmCannon_Draw(game);
            Samus_Draw(game);
            UpdateGrappleBeamTilesAndIncrFlameCtr(game);
            if ram_word!(game, grapple_beam_length) == 0 {
                return;
            }
        }
    } else {
        HandleAtmosphericEffects(game);
        Samus_Draw(game);
        UpdateGrappleBeamTilesAndIncrFlameCtr(game);
        if ram_word!(game, grapple_beam_length) == 0 {
            return;
        }
    }
    HandleGrappleBeamGfx(game);
}
pub unsafe fn Samus_DrawHandler_EndOfShinespark(game: &mut Game) {
    Samus_Draw(game);
    let mut i: i32 = 2i32;
    while i >= 0i32 {
        Samus_DrawShinesparkCrashEchoes(i as u16, game);
        i -= 2i32;
    }
}
pub unsafe fn Samus_DrawManyEchoes(game: &mut Game) {
    SamusDrawHandler_Default(game);
    for _ in (0i32..=2i32).step_by(2) {
        Samus_DrawEchoes(game);
    }
}
pub unsafe fn SamusDisplayHandler_UsingElevator(game: &mut Game) {
    if (ram_word!(game, nmi_frame_counter_word) as i32 & 1i32) == 0i32 {
        SamusDisplayHandler_SamusReceivedFatal(game);
    }
}
pub unsafe fn SamusDisplayHandler_SamusReceivedFatal(game: &mut Game) {
    Samus_DrawWhenNotAnimatingOrDying(game);
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
    if (ram_word!(game, equipped_items) as i32 & 0x20i32) != 0i32 {
        set_ram_word!(game.samus_suit_palette_index = 4u16);
    } else if (ram_word!(game, equipped_items) as i32 & 1i32) != 0i32 {
        set_ram_word!(game.samus_suit_palette_index = 2u16);
    } else {
        set_ram_word!(game.samus_suit_palette_index = 0u16);
    };
}
pub unsafe fn MoveSamusWithControlPad(game: &mut Game) {
    if (ram_word!(game, joypad1_lastkeys) as i32 & 0x800i32) != 0i32 {
        Samus_MoveUp((-4i16 as i32) << 16i32, game);
    }
    if (ram_word!(game, joypad1_lastkeys) as i32 & 0x400i32) != 0i32 {
        Samus_MoveDown((4i16 as i32) << 16i32, game);
    }
    if (ram_word!(game, joypad1_lastkeys) as i32 & 0x200i32) != 0i32 {
        Samus_MoveLeft((-4i16 as i32) << 16i32, game);
    }
    if (ram_word!(game, joypad1_lastkeys) as i32 & 0x100i32) != 0i32 {
        Samus_MoveRight((4i16 as i32) << 16i32, game);
    }
}
static byte_90ED50: [u8; 28] = [
    0, 0, 0, 0, 1, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0,
];
pub unsafe fn SwappedAmmoRoutine(game: &mut Game) {
    if byte_90ED50[*ram_ptr!(game; 0xa1f) as usize] != 0 {
        set_ram_word!(game.samus_missiles = (ram_word!(game, samus_x_pos) as i32 >> 4i32) as u16);
        set_ram_word!(
            game.samus_max_missiles = (ram_word!(game, samus_x_pos) as i32 >> 4i32) as u16
        );
        set_ram_word!(
            game.samus_super_missiles = (ram_word!(game, samus_y_pos) as i32 >> 4i32) as u16
        );
        set_ram_word!(
            game.samus_max_super_missiles = (ram_word!(game, samus_y_pos) as i32 >> 4i32) as u16
        );
    }
}
pub unsafe fn UNUSEDsub_90ED6C(game: &mut Game) {
    set_ram_word!(game.samus_missiles = ram_word!(game, game_time_hours));
    set_ram_word!(game.samus_max_missiles = ram_word!(game, game_time_hours));
    set_ram_word!(game.samus_super_missiles = ram_word!(game, game_time_minutes));
    set_ram_word!(game.samus_max_super_missiles = ram_word!(game, game_time_minutes));
    set_ram_word!(game.samus_power_bombs = ram_word!(game, game_time_seconds));
    set_ram_word!(game.samus_max_power_bombs = ram_word!(game, game_time_seconds));
}
static kSamus_FootstepGraphics: [Option<Func_V>; 8] = callback_table! { unsafe Func_V;
    Samus_FootstepGraphics_Crateria, Samus_FootstepGraphics_1, Samus_FootstepGraphics_1,
    Samus_FootstepGraphics_1, Samus_FootstepGraphics_Maridia, Samus_FootstepGraphics_1,
    Samus_FootstepGraphics_1, Samus_FootstepGraphics_1,
};
pub unsafe fn Samus_FootstepGraphics(game: &mut Game) {
    kSamus_FootstepGraphics[ram_word!(game, area_index) as usize]
        .expect("non-null function pointer")(game);
}
static byte_90EDC9: [u8; 16] = [1, 0, 0, 0, 0, 2, 0, 4, 0, 4, 4, 4, 4, 0, 4, 0];
pub unsafe fn Samus_FootstepGraphics_Crateria(game: &mut Game) {
    let use_default_footsteps: bool;
    if !(ram_word!(game, cinematic_function) != 0u16
        || ((ram_word!(game, room_index)) as i16).wrapping_sub(16i16) >= 0i16)
    {
        use_default_footsteps =
            if (byte_90EDC9[ram_word!(game, room_index) as usize] as i32 & 1i32) == 0i32 {
                if (byte_90EDC9[ram_word!(game, room_index) as usize] as i32 & 2i32) != 0i32 {
                    !(ram_word!(game, samus_y_pos) as i32 - 944i32 & 0x8000i32 == 0)
                } else {
                    !((byte_90EDC9[ram_word!(game, room_index) as usize] as i32 & 4i32) != 0i32)
                }
            } else {
                (ram_word!(game, fx_type) != 10u16)
            };
        if use_default_footsteps {
        } else {
            Samus_FootstepGraphics_Maridia(game);
            return;
        }
    }
    Samus_FootstepGraphics_1(game);
}
pub unsafe fn Samus_FootstepGraphics_Maridia(game: &mut Game) {
    let r18: u16 = Samus_GetBottom_R18(game);
    if liquid_at_height(game, r18) != LiquidAtHeight::None {
        return;
    }
    if *ram_ptr!(game; 0xa1e) == 4u8 {
        set_ram_word!(
            game.atmospheric_gfx_x_pos[0] = (ram_word!(game, samus_x_pos)).wrapping_sub(12u16)
        );
        set_ram_word!(
            game.atmospheric_gfx_x_pos[1] = (ram_word!(game, samus_x_pos)).wrapping_add(8u16)
        );
    } else {
        set_ram_word!(
            game.atmospheric_gfx_x_pos[0] = (ram_word!(game, samus_x_pos)).wrapping_add(12u16)
        );
        set_ram_word!(
            game.atmospheric_gfx_x_pos[1] = (ram_word!(game, samus_x_pos)).wrapping_sub(8u16)
        );
    }
    set_ram_word!(
        game.atmospheric_gfx_y_pos[0] = (ram_word!(game, samus_y_pos)).wrapping_add(16u16)
    );
    set_ram_word!(
        game.atmospheric_gfx_y_pos[1] = (ram_word!(game, samus_y_pos)).wrapping_add(16u16)
    );
    set_ram_words! { game, 256u16;
        atmospheric_gfx_frame_and_type[0], atmospheric_gfx_frame_and_type[1],
    }
    set_ram_word!(game.atmospheric_gfx_anim_timer[0] = -32766i32 as u16);
    set_ram_word!(game.atmospheric_gfx_anim_timer[1] = 3u16);
}
pub unsafe fn Samus_FootstepGraphics_1(game: &mut Game) {
    if (ram_word!(game, speed_boost_counter) as i32 & 0xff00i32) == 1024i32 {
        let r18: u16 = Samus_GetBottom_R18(game);
        if liquid_at_height(game, r18) != LiquidAtHeight::None {
            return;
        }
        if *ram_ptr!(game; 0xa1e) == 4u8 {
            set_ram_word!(
                game.atmospheric_gfx_x_pos[0] = (ram_word!(game, samus_x_pos)).wrapping_sub(12u16)
            );
            set_ram_word!(
                game.atmospheric_gfx_x_pos[1] = (ram_word!(game, samus_x_pos)).wrapping_add(8u16)
            );
        } else {
            set_ram_word!(
                game.atmospheric_gfx_x_pos[0] = (ram_word!(game, samus_x_pos)).wrapping_add(12u16)
            );
            set_ram_word!(
                game.atmospheric_gfx_x_pos[1] = (ram_word!(game, samus_x_pos)).wrapping_sub(8u16)
            );
        }
        set_ram_word!(
            game.atmospheric_gfx_y_pos[0] = (ram_word!(game, samus_y_pos)).wrapping_add(16u16)
        );
        set_ram_word!(
            game.atmospheric_gfx_y_pos[1] = (ram_word!(game, samus_y_pos)).wrapping_add(16u16)
        );
        set_ram_words! { game, 1792u16;
            atmospheric_gfx_frame_and_type[0], atmospheric_gfx_frame_and_type[1],
        }
        set_ram_word!(game.atmospheric_gfx_anim_timer[0] = -32766i32 as u16);
        set_ram_word!(game.atmospheric_gfx_anim_timer[1] = 3u16);
    }
}
pub unsafe fn Samus_UpdateSpeedEchoPos(game: &mut Game) {
    if (ram_word!(game, speed_boost_counter) as i32 & 0xff00i32) == 1024i32
        && (ram_word!(game, speed_echoes_index) as i32 & 0x8000i32) == 0i32
        && (ram_word!(game, game_time_frames) as i32 & 3i32) == 0i32
    {
        let v0: u16 = ram_word!(game, speed_echoes_index);
        let v1: i32 = ram_word!(game, speed_echoes_index) as i32 >> 1i32;
        set_ram_word!(game.speed_echo_xpos[v1] = ram_word!(game, samus_x_pos));
        set_ram_word!(game.speed_echo_ypos[v1] = ram_word!(game, samus_y_pos));
        let mut v2: u16 = (v0).wrapping_add(2u16);
        if ((v2) as i16).wrapping_sub(4i16) >= 0i16 {
            v2 = 0u16;
        }
        set_ram_word!(game.speed_echoes_index = v2);
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
pub unsafe fn ProjPreInstr_UnknownProj8027(k: u16, game: &mut Game) {
    static kProjPreInstr_UnknownProj8027_X: [i16; 4] = [-4, -4, 4, 4];
    static kProjPreInstr_UnknownProj8027_Y: [i16; 4] = [4, -4, -4, 4];
    static kProjPreInstr_UnknownProj8027_X2: [i16; 4] = [0x80, 0x80, -0x80, -0x80];
    static kProjPreInstr_UnknownProj8027_Y2: [i16; 4] = [-0x80, 0x80, 0x80, -0x80];
    let v1: i32 = k as i32 >> 1i32;
    update_ram_word!(
        game.projectile_x_pos[v1],
        wrapping_add((kProjPreInstr_UnknownProj8027_X[v1 as usize]) as u16)
    );
    let v2: u16 = ((kProjPreInstr_UnknownProj8027_Y[v1 as usize]) as u16)
        .wrapping_add(ram_word!(game, projectile_y_pos[v1]));
    set_ram_word!(game.projectile_y_pos[v1] = v2);
    if v2 == ram_word!(game, samus_y_pos) {
        if ram_word!(game, projectile_variables[v1]) == 1u16 {
            if k == 6u16 {
                set_ram_word!(game.samus_movement_handler = ((0x90f04bi32) as u16));
            }
            ClearProjectile(k, game);
        } else {
            let v3: i32 = k as i32 >> 1i32;
            update_ram_word!(game.projectile_variables[v3], wrapping_add(1));
            set_ram_word!(game.samus_shine_timer = 180u16);
            set_ram_word!(game.timer_for_shine_timer = 1u16);
            set_ram_word!(game.special_samus_palette_frame = 0u16);
            set_ram_word!(
                game.projectile_x_pos[v3] = ((kProjPreInstr_UnknownProj8027_X2[v3 as usize])
                    as u16)
                    .wrapping_add(ram_word!(game, samus_x_pos))
            );
            set_ram_word!(
                game.projectile_y_pos[v3] = ((kProjPreInstr_UnknownProj8027_Y2[v3 as usize])
                    as u16)
                    .wrapping_add(ram_word!(game, samus_y_pos))
            );
        }
    }
}
pub unsafe fn Samus_Func25_ShineSpark(game: &mut Game) {
    if ram_word!(game, samus_shine_timer) == 0 {
        set_ram_word!(game.special_samus_palette_frame = 6u16);
        set_ram_words! { game, 1u16; special_samus_palette_timer, demo_timer_counter }
        set_ram_word!(game.timer_for_shine_timer = 10u16);
        set_ram_word!(game.samus_shine_timer = 120u16);
        set_ram_word!(game.samus_movement_handler = ((0x90f072i32) as u16));
    }
}
pub unsafe fn Samus_MoveHandler_F072(game: &mut Game) {
    if ram_word!(game, samus_shine_timer) == 0 {
        set_ram_word!(game.frame_handler_alfa = ((0x90e695i32) as u16));
        set_ram_word!(game.samus_movement_handler = ((0x90a337i32) as u16));
    }
}
static kSamusCodeHandlers: [Option<Func_U8>; 32] = callback_table! { unsafe Func_U8;
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
    let code: u16 = kSamusCodeHandlers[(a as i32 & 0x1fi32) as usize]
        .expect("non-null function pointer")(game) as u16;
    if code as i32 & 1i32 == 0 {
        return (code as i32 >> 1i32) as u16;
    }
    set_ram_word!(game.samus_new_pose = -1i32 as u16);
    set_ram_word!(game.samus_new_pose_interrupted = -1i32 as u16);
    set_ram_word!(game.samus_new_pose_transitional = -1i32 as u16);
    set_ram_words! { game, 0u16;
        samus_momentum_routine_index, samus_special_transgfx_index, samus_hurt_switch_index,
    }
    -1i32 as u16
}
pub unsafe fn Samus_UpdatePreviousPose(game: &mut Game) {
    set_ram_word!(game.samus_last_different_pose = ram_word!(game, samus_prev_pose));
    *(ram_ptr!(game, u16; 0xa26)) = *(ram_ptr!(game, u16; 0xa22));
    set_ram_word!(game.samus_prev_pose = ram_word!(game, samus_pose));
    *(ram_ptr!(game, u16; 0xa22)) = *(ram_ptr!(game, u16; 0xa1e));
}
pub fn ClearCarry() -> u8 {
    0u8
}
pub unsafe fn SamusCode_00_LockSamus(game: &mut Game) -> u8 {
    set_ram_word!(game.frame_handler_alfa = ((0x90e713i32) as u16));
    set_ram_word!(game.frame_handler_beta = ((0x90e8dci32) as u16));
    1u8
}
pub unsafe fn SamusCode_01_UnlockSamus(game: &mut Game) -> u8 {
    set_ram_word!(game.frame_handler_alfa = ((0x90e695i32) as u16));
    set_ram_word!(game.frame_handler_beta = ((0x90e725i32) as u16));
    1u8
}
pub unsafe fn SamusCode_02_ReachCeresElevator(game: &mut Game) -> u8 {
    if *ram_ptr!(game; 0xa1e) == 4u8 {
        set_ram_word!(game.samus_pose = kPose_02_FaceL_Normal as u16);
    } else {
        set_ram_word!(game.samus_pose = kPose_01_FaceR_Normal as u16);
    }
    set_ram_word!(game.samus_anim_frame_skip = 0u16);
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    set_ram_word!(game.reached_ceres_elevator_fade_timer = 60u16);
    SamusCode_00_LockSamus(game)
}
pub unsafe fn SamusCode_03(game: &mut Game) -> u8 {
    if ram_word!(game, grapple_beam_function) as i32 != 0x9bc4f0i32 & 0xffffi32 {
        set_ram_word!(game.grapple_beam_function = ((0x9bc8c5i32) as u16));
        return 0u8;
    }
    if *ram_ptr!(game; 0xa1f) as i32 != kMovementType_03_SpinJumping as i32
        && *ram_ptr!(game; 0xa1f) as i32 != kMovementType_14_WallJumping as i32
    {
        return 0u8;
    }
    if *ram_ptr!(game; 0xa1e) as i32 == kMovementType_04_MorphBallOnGround as i32 {
        set_ram_word!(game.samus_pose = kPose_02_FaceL_Normal as u16);
    } else {
        set_ram_word!(game.samus_pose = kPose_01_FaceR_Normal as u16);
    }
    set_ram_word!(game.samus_anim_frame_skip = 0u16);
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    Samus_UpdatePreviousPose(game);
    1u8
}
pub unsafe fn SamusCode_04_06_Common(game: &mut Game) -> u8 {
    set_ram_word!(game.flare_counter = 0u16);
    ClearFlareAnimationState(game);
    Samus_LoadSuitPalette(game);
    0u8
}
pub unsafe fn SamusCode_04(game: &mut Game) -> u8 {
    set_ram_word!(game.samus_charge_palette_index = 0u16);
    SamusCode_04_06_Common(game)
}
pub unsafe fn SamusCode_06_LockToStation(game: &mut Game) -> u8 {
    set_ram_word!(game.frame_handler_alfa = ((0x90e713i32) as u16));
    set_ram_word!(game.frame_handler_beta = ((0x90e8d6i32) as u16));
    if ram_word!(game, flare_counter) as i32 - 15i32 & 0x8000i32 == 0 {
        QueueSfx1_Max15(2u16, game);
    }
    SamusCode_04_06_Common(game)
}
pub unsafe fn SamusCode_07_SetupForElevator(game: &mut Game) -> u8 {
    MakeSamusFaceForward(game);
    set_ram_word!(game.frame_handler_beta = ((0x90e8eci32) as u16));
    set_ram_word!(game.samus_movement_handler = ((0x90a337i32) as u16));
    set_ram_word!(game.samus_draw_handler = ((0x90ec14i32) as u16));
    set_ram_word!(game.samus_input_handler = ((0x90e913i32) as u16));
    set_ram_word!(game.bomb_jump_dir = 0u16);
    1u8
}
pub unsafe fn SamusCode_08_SetupForCeresStart(game: &mut Game) -> u8 {
    set_ram_word!(game.frame_handler_alfa = ((0x90e8cdi32) as u16));
    set_ram_word!(game.frame_handler_beta = ((0x90e8dci32) as u16));
    set_ram_word!(game.samus_pose = kPose_00_FaceF_Powersuit as u16);
    set_ram_word!(game.samus_anim_frame_skip = 0u16);
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    Samus_LoadSuitPalette(game);
    set_ram_word!(game.samus_draw_handler = ((0x90eb52i32) as u16));
    set_ram_word!(game.samus_prev_pose = ram_word!(game, samus_pose));
    set_ram_word!(game.samus_last_different_pose = ram_word!(game, samus_pose));
    *(ram_ptr!(game, u16; 0xa22)) = *(ram_ptr!(game, u16; 0xa1e));
    *(ram_ptr!(game, u16; 0xa26)) = *(ram_ptr!(game, u16; 0xa1e));
    set_ram_word!(game.samus_y_pos = 0u16);
    let platform_parameter =
        SpawnEprojWithGfx(0u16, 0u16, addr_kEproj_CeresElevatorPad as u16, game);
    SpawnEprojWithGfx(
        platform_parameter,
        0u16,
        addr_kEproj_CeresElevatorPlatform as u16,
        game,
    );
    set_ram_word!(game.debug_disable_minimap = 0u16);
    PlayRoomMusicTrackAfterAFrames(0x20u16, game);
    1u8
}
pub unsafe fn SamusCode_08_SetupForZebesStart(game: &mut Game) -> u8 {
    if (ram_word!(game, equipped_items) as i32 & 0x20i32) != 0i32 {
        SpawnPalfxObject(addr_stru_8DE1FC as u16, game);
        set_ram_word!(game.samus_pose = kPose_9B_FaceF_VariaGravitySuit as u16);
    } else if (ram_word!(game, equipped_items) as i32 & 1i32) != 0i32 {
        SpawnPalfxObject(addr_stru_8DE1F8 as u16, game);
        set_ram_word!(game.samus_pose = kPose_9B_FaceF_VariaGravitySuit as u16);
    } else {
        SpawnPalfxObject(addr_stru_8DE1F4 as u16, game);
        set_ram_word!(game.samus_pose = kPose_00_FaceF_Powersuit as u16);
    }
    Samus_LoadSuitPalette(game);
    SamusFunc_F433(game);
    set_ram_word!(game.samus_anim_frame_timer = 3u16);
    set_ram_word!(game.samus_anim_frame = 2u16);
    set_ram_word!(game.substate = 0u16);
    1u8
}
pub unsafe fn SamusCode_0A_ClearDrawHandler(game: &mut Game) -> u8 {
    set_ram_word!(game.samus_draw_handler = ((0x90e90ei32) as u16));
    0u8
}
pub unsafe fn SamusCode_0B_DrawHandlerDefault(game: &mut Game) -> u8 {
    set_ram_word!(game.samus_draw_handler = ((0x90eb52i32) as u16));
    SamusCode_01_UnlockSamus(game)
}
pub unsafe fn SamusCode_0C_UnlockFromMapStation(game: &mut Game) -> u8 {
    SamusFunc_E633(game);
    if ram_word!(game, frame_handler_beta) as i32 == 0x90e8d6i32 & 0xffffi32 {
        set_ram_word!(game.frame_handler_alfa = ((0x90e695i32) as u16));
        set_ram_word!(game.frame_handler_beta = ((0x90e725i32) as u16));
    }
    1u8
}
pub unsafe fn SamusCode_0D_IsGrappleActive(game: &mut Game) -> u8 {
    ((if ram_word!(game, grapple_beam_function) as i32 != 0x9bc4f0i32 & 0xffffi32 {
        2i32
    } else {
        0i32
    }) as u8)
}
pub unsafe fn SamusCode_0D_IsGrappleActive_A(game: &mut Game) -> u8 {
    (ram_word!(game, grapple_beam_function) as i32 != 0x9bc4f0i32 & 0xffffi32) as u8
}
pub unsafe fn SamusCode_0E(game: &mut Game) -> u8 {
    set_ram_word!(game.frame_handler_alfa = ((0x90e695i32) as u16));
    set_ram_word!(game.frame_handler_beta = ((0x90e8aai32) as u16));
    1u8
}
pub unsafe fn SamusCode_0F_EnableTimerHandling(game: &mut Game) -> u8 {
    set_ram_word!(game.frame_handler_gamma = ((0x90e0e6i32) as u16));
    0u8
}
pub unsafe fn SamusCode_10(game: &mut Game) -> u8 {
    if ram_word!(game, frame_handler_beta) as i32 != 0x90e8d9i32 & 0xffffi32 {
        set_ram_word!(game.frame_handler_alfa = ((0x90e695i32) as u16));
        set_ram_word!(game.frame_handler_beta = ((0x90e725i32) as u16));
    }
    1u8
}
pub unsafe fn SamusCode_11_15_Common(game: &mut Game) -> u8 {
    set_ram_word!(game.frame_handler_alfa = ((0x90e713i32) as u16));
    set_ram_word!(game.frame_handler_beta = ((0x90e8cdi32) as u16));
    set_ram_word!(game.samus_draw_handler = ((0x90ec1di32) as u16));
    1u8
}
pub unsafe fn SamusCode_11_SetupForDeath(game: &mut Game) -> u8 {
    DisablePaletteFx(game);
    SamusCode_11_15_Common(game)
}
pub unsafe fn SamusCode_15_CalledBySuitAcquision(game: &mut Game) -> u8 {
    Samus_UpdatePreviousPose(game);
    set_ram_word!(game.flare_counter = 0u16);
    ClearFlareAnimationState(game);
    Samus_LoadSuitPalette(game);
    SamusCode_11_15_Common(game)
}
pub unsafe fn SamusCode_12_SetSuperPaletteFlag1(game: &mut Game) -> u8 {
    set_ram_word!(game.samus_special_super_palette_flags = 1u16);
    0u8
}
pub unsafe fn SamusCode_12_SetSuperPaletteFlag0(game: &mut Game) -> u8 {
    set_ram_word!(game.samus_special_super_palette_flags = 0u16);
    Samus_LoadSuitPalette(game);
    0u8
}
pub unsafe fn SamusCode_14(game: &mut Game) -> u8 {
    if ram_word!(game, samus_health) as i32 - 31i32 & 0x8000i32 != 0 {
        QueueSfx3_Max6(2u16, game);
    }
    if SamusCode_0D_IsGrappleActive_A(game) == 0 {
        if *ram_ptr!(game; 0xa1e) == 3u8 {
            if ram_word!(game, samus_pose) as i32 == kPose_81_FaceR_Screwattack as i32
                || ram_word!(game, samus_pose) as i32 == kPose_82_FaceL_Screwattack as i32
            {
                QueueSfx1_Max6(0x33u16, game);
            } else if ram_word!(game, samus_pose) as i32 == kPose_1B_FaceR_SpaceJump as i32
                || ram_word!(game, samus_pose) as i32 == kPose_1C_FaceL_SpaceJump as i32
            {
                QueueSfx1_Max6(0x3eu16, game);
            } else {
                QueueSfx1_Max6(0x31u16, game);
            }
        }
    } else {
        QueueSfx1_Max6(6u16, game);
    }
    0u8
}
pub unsafe fn SamusCode_05_SetupDrained(game: &mut Game) -> u8 {
    set_ram_word!(game.frame_handler_gamma = ((0x90e09bi32) as u16));
    Samus_SetupForBeingDrained(game)
}
pub unsafe fn Samus_SetupForBeingDrained(game: &mut Game) -> u8 {
    set_ram_word!(game.samus_pose = kPose_54_FaceL_Knockback as u16);
    set_ram_word!(game.samus_anim_frame_skip = 0u16);
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    Samus_UpdatePreviousPose(game);
    set_ram_word!(game.flare_counter = 0u16);
    ClearFlareAnimationState(game);
    Samus_LoadSuitPalette(game);
    set_ram_word!(game.frame_handler_alfa = ((0x90e713i32) as u16));
    set_ram_word!(game.frame_handler_beta = ((0x90e8d9i32) as u16));
    1u8
}
pub unsafe fn SamusCode_18(game: &mut Game) -> u8 {
    set_ram_word!(game.frame_handler_gamma = ((0x90e0c5i32) as u16));
    Samus_SetupForBeingDrained(game)
}
pub unsafe fn SamusCode_16(game: &mut Game) -> u8 {
    set_ram_word!(game.samus_special_super_palette_flags = 0x8000u16);
    set_ram_words! { game, 1u16; special_samus_palette_frame, special_samus_palette_timer }
    set_ram_word!(game.samus_charge_palette_index = 0u16);
    0u8
}
pub unsafe fn SamusCode_17_DisableRainbowSamusAndStandUp(game: &mut Game) -> u8 {
    set_ram_words! { game, 0u16;
        samus_special_super_palette_flags, special_samus_palette_frame, special_samus_palette_timer,
        samus_charge_palette_index,
    }
    Samus_LoadSuitPalette(game);
    set_ram_word!(game.samus_anim_frame_timer = 1u16);
    set_ram_word!(game.samus_anim_frame = 13u16);
    0u8
}
pub unsafe fn SamusCode_17_FreezeDrainedSamus(game: &mut Game) -> u8 {
    set_ram_word!(game.samus_anim_frame_timer = 1u16);
    set_ram_word!(game.samus_anim_frame = 28u16);
    1u8
}
pub unsafe fn SamusCode_1A(game: &mut Game) -> u8 {
    set_ram_word!(game.frame_handler_beta = ((0x90e902i32) as u16));
    0u8
}
pub unsafe fn SamusCode_1B_CheckedLockSamus(game: &mut Game) -> u8 {
    if ram_word!(game, frame_handler_beta) as i32 == 0x90e8d9i32 & 0xffffi32 {
        1u8
    } else {
        SamusCode_00_LockSamus(game)
    }
}
pub unsafe fn SamusCode_1C(game: &mut Game) -> u8 {
    let current_block_5: u64;
    if *ram_ptr!(game; 0xa1f) as i32 == kMovementType_14_WallJumping as i32 {
        if ram_word!(game, samus_anim_frame) as i32 - 23i32 & 0x8000i32 != 0 {
            if ram_word!(game, samus_anim_frame) as i32 - 13i32 & 0x8000i32 != 0 {
                current_block_5 = 15182254099572620017;
            } else {
                current_block_5 = 14206663613128494784;
            }
        } else {
            current_block_5 = 11650488183268122163;
        }
    } else {
        if *ram_ptr!(game; 0xa1f) as i32 != kMovementType_03_SpinJumping as i32 {
            return 0u8;
        }
        if ram_word!(game, samus_pose) as i32 != kPose_81_FaceR_Screwattack as i32
            && ram_word!(game, samus_pose) as i32 != kPose_82_FaceL_Screwattack as i32
        {
            if ram_word!(game, samus_pose) as i32 != kPose_1B_FaceR_SpaceJump as i32
                && ram_word!(game, samus_pose) as i32 != kPose_1C_FaceL_SpaceJump as i32
            {
                current_block_5 = 15182254099572620017;
            } else {
                current_block_5 = 14206663613128494784;
            }
        } else {
            current_block_5 = 11650488183268122163;
        }
    }
    match current_block_5 {
        11650488183268122163 => {}
        15182254099572620017 => {
            QueueSfx1_Max9(0x31u16, game);
            return 0u8;
        }
        _ => {
            QueueSfx1_Max9(0x3eu16, game);
            return 0u8;
        }
    }
    QueueSfx1_Max9(0x33u16, game);
    0u8
}
pub unsafe fn SamusCode_1D_ClearSoundInDoor(game: &mut Game) -> u8 {
    if *ram_ptr!(game; 0xa1f) == 3u8 || *ram_ptr!(game; 0xa1f) == 20u8 {
        QueueSfx1_Max15(0x32u16, game);
        0u8
    } else {
        if (ram_word!(game, button_config_shoot_x) as i32
            & ram_word!(game, joypad1_lastkeys) as i32)
            == 0i32
            && ram_word!(game, flare_counter) as i32 - 16i32 & 0x8000i32 != 0
        {
            QueueSfx1_Max15(2u16, game);
        }
        0u8
    }
}
pub unsafe fn SamusCode_1E(game: &mut Game) -> u8 {
    if ram_word!(game, game_state) == 8u16 {
        if *ram_ptr!(game; 0xa1f) == 3u8 || *ram_ptr!(game; 0xa1f) == 20u8 {
            SamusCode_1C(game);
            return 0u8;
        }
        if ram_word!(game, flare_counter) as i32 - 16i32 & 0x8000i32 == 0 {
            QueueSfx1_Max9(0x41u16, game);
        }
    }
    0u8
}
pub unsafe fn SamusCode_1F(game: &mut Game) -> u8 {
    if ram_word!(game, grapple_beam_function) != (addr_loc_90C4F0 as u16) {
        set_ram_words! { game, 0u16;
            grapple_beam_unkD1E, grapple_beam_unkD20, grapple_beam_direction, grapple_beam_unkD36,
            grapple_walljump_timer, slow_grabble_scrolling_flag, grapple_varCF6, grapple_beam_flags,
        }
        LoadProjectilePalette(ram_word!(game, equipped_beams), game);
        set_ram_word!(game.grapple_beam_function = -15120i32 as u16);
        set_ram_word!(game.samus_draw_handler = ((0x90eb52i32) as u16));
    }
    0u8
}
pub unsafe fn Samus_Func26(game: &mut Game) -> u8 {
    if *ram_ptr!(game; 0xa1f) as i32 != kMovementType_03_SpinJumping as i32
        && *ram_ptr!(game; 0xa1f) as i32 != kMovementType_14_WallJumping as i32
        && (ram_word!(game, button_config_shoot_x) as i32
            & ram_word!(game, joypad1_lastkeys) as i32)
            != 0i32
        && ram_word!(game, flare_counter) as i32 - 16i32 & 0x8000i32 == 0
    {
        QueueSfx1_Max9(0x41u16, game);
    }
    0u8
}
pub unsafe fn Samus_ShootCheck(game: &mut Game) {
    let resume_charge_sound: bool =
        ((ram_word!(game, play_resume_charging_beam_sfx) as i32 & 0x8000i32) != 0i32)
            || ({
                if ram_word!(game, play_resume_charging_beam_sfx) != 0 {
                    if (ram_word!(game, button_config_shoot_x) as i32
                        & ram_word!(game, joypad1_lastkeys) as i32)
                        != 0i32
                    {
                        QueueSfx1_Max9(0x41u16, game);
                    }
                    set_ram_word!(game.play_resume_charging_beam_sfx = 0u16);
                }
                if ram_word!(game, samus_echoes_sound_flag) != 0u16
                    && (ram_word!(game, speed_boost_counter) as i32 & 0x400i32) == 0i32
                {
                    set_ram_word!(game.samus_echoes_sound_flag = 0u16);
                    QueueSfx3_Max15(0x25u16, game);
                }
                ((*ram_ptr!(game; 0xa11) == 3u8 || *ram_ptr!(game; 0xa11) == 20u8)
                    && *ram_ptr!(game; 0xa1f) as i32 != kMovementType_03_SpinJumping as i32
                    && *ram_ptr!(game; 0xa1f) as i32 != kMovementType_14_WallJumping as i32)
                    && ({
                        QueueSfx1_Max15(0x32u16, game);
                        (ram_word!(game, flare_counter) as i32 - 16i32 & 0x8000i32 == 0
                            && (ram_word!(game, button_config_shoot_x) as i32
                                & ram_word!(game, joypad1_lastkeys) as i32)
                                != 0i32)
                    })
            });
    if resume_charge_sound {
        set_ram_word!(game.play_resume_charging_beam_sfx = 1u16);
    }
    if ram_word!(game, enable_debug) != 0 {
        if ram_word!(game, samus_pose) as i32 == kPose_00_FaceF_Powersuit as i32
            || ram_word!(game, samus_pose) as i32 == kPose_9B_FaceF_VariaGravitySuit as i32
        {
            if (ram_word!(game, joypad2_last) as i32 & 0x30i32) == 48i32
                && (ram_word!(game, joypad2_new_keys) as i32 & 0x80i32) != 0i32
            {
                set_ram_word!(game.debug_invincibility = 7u16);
            }
        } else {
            if ram_word!(game, debug_invincibility) as i32 - 7i32 & 0x8000i32 == 0 {
                return;
            }
            set_ram_word!(game.debug_invincibility = 0u16);
        }
    }
    if CheckEventHappened(0xeu16, game) as i32 & 1i32 != 0
        && ram_word!(game, frame_handler_gamma) as i32 == 0x90e114i32 & 0xffffi32
        && ram_word!(game, game_state) as i32 != kGameState_35_TimeUp as i32
    {
        set_ram_word!(game.game_state = kGameState_35_TimeUp as u16);
    }
}
