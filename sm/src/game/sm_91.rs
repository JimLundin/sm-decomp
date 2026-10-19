pub use crate::game::constants::bank_8d::*;
pub use crate::game::constants::bank_8f::*;
pub use crate::game::constants::bank_91::*;
pub use crate::game::constants::bank_9b::*;
pub use crate::game::constants::buttons::*;
pub use crate::game::constants::game_states::*;
pub use crate::game::constants::samus_movement::*;
pub use crate::game::constants::samus_poses::*;
use crate::game::sm_80::{
    InvalidInterrupt_Crash, QueueSfx1_Max6, QueueSfx1_Max9, QueueSfx2_Max6, QueueSfx3_Max6,
    QueueSfx3_Max9,
};
use crate::game::sm_84::{DisablePLMs, EnablePLMs, LoadXrayBlocks};
use crate::game::sm_86::{DisableEprojs, EnableEprojs};
use crate::game::sm_87::{DisableAnimtiles, EnableAnimtiles};
use crate::game::sm_88::SpawnHdmaObject;
use crate::game::sm_8d::{DisablePaletteFx, EnablePaletteFx, SpawnPalfxObject};
use crate::game::sm_90::{liquid_at_height, LiquidAtHeight};
use crate::game::sm_90::{
    CallSomeSamusCode, LoadProjectilePalette, Projectile_Func7_Shinespark,
    Samus_AlignBottomWithPrevPose, Samus_GetBottom_R18, Samus_GetTop_R20, Samus_InitJump,
    Samus_InitWallJump, Samus_SetSpeedForKnockback_Y, UpdateBeamTilesAndPalette,
    WriteBeamPalette_A,
};
use crate::game::sm_94::{Samus_CollDetectChangedPose, Samus_MoveRight_NoSolidColl};
use crate::game::sm_9b::{GrappleBeamFunc_BD95, GrappleBeamFunc_BEEB};
use crate::game::sm_a0::Samus_CheckSolidEnemyColl;
use crate::game_state::Game;
use crate::ram_map::{ram_word, set_ram_word, update_ram_word};
use crate::rtl::{mov24, Mult8x8, RomPtr, Unreachable};
use crate::rtl::{AddToHiLo, SetHiLo};
extern "C" {
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: u32,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;

}
pub use crate::types::{
    __int16_t, __int32_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int32,
    int32_t, int8, int8_t, size_t, uint16, uint16_t, uint32, uint32_t, uint8, uint8_t, Func_U8,
    Func_V, LongPtr, SamusPose, SpawnHdmaObject_Args, VoidP, VramWriteEntry,
};
pub use crate::types::{SamusMovementType, SamusPoseParams};
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct DemoInputObject {
    pub ptr: VoidP,
    pub pre_instr: VoidP,
    pub instr_ptr: VoidP,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct DemoSetDef {
    pub items: uint16,
    pub missiles: uint16,
    pub super_missiles: uint16,
    pub power_bombs: uint16,
    pub health: uint16,
    pub beams: uint16,
    pub equipped_beams_: uint16,
    pub demo_obj: VoidP,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct PoseEntry {
    pub new_input: uint16,
    pub cur_input: uint16,
    pub new_pose: uint16,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct XrayBlockData {
    pub value: uint16,
    pub addr: VoidP,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct SamusCrystalFlashPalTable {
    pub ptr: VoidP,
    pub timer: uint16,
}
pub use crate::types::Buttons;
pub use crate::types::Consts_8F;
pub use crate::types::Consts_91;
pub use crate::types::Consts_9B;
pub use crate::types::GameState;
pub use crate::types::{CheckEnemyColl_Result, CheckSolidEnemyColl_Result};
pub use crate::types::{
    Consts_8D, Ram4000, Ram4000_Backups, TileTable, TileTables, VramReadQueueEnt,
};
#[derive(Copy, Clone)]
#[repr(C)]
pub struct XrayHdmaCtx {
    pub r22: uint16,
    pub r24: uint16,
    pub r26: uint16,
    pub r28: uint16,
    pub r30: uint16,
    pub r32: uint16,
    pub r18_avoid: uint16,
    pub dest: *mut uint16,
}
enum XrayBeamDirection {
    Right,
    Up,
    Down,
    Left,
    Horizontal,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Pair_R18_R20 {
    pub r18: uint16,
    pub r20: uint16,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const true_0: i32 = 1;
pub const false_0: i32 = 0;
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 28] = unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_char; 28]>(*b"uint8 *RomPtr_RAM(uint16_t)\0")
};
#[inline]
unsafe fn RomPtr_RAM(addr: uint16_t, game: &mut Game) -> *mut uint8 {
    '_c2rust_label: {
        if addr < 0x2000u16 {
        } else {
            __assert_fail(
                b"addr < 0x2000\0" as *const u8 as *const ::core::ffi::c_char,
                b"src/sm_rtl.h\0" as *const u8 as *const ::core::ffi::c_char,
                51 as u32,
                __ASSERT_FUNCTION.as_ptr(),
            );
        }
    };
    (&raw mut game.g_ram as *mut uint8).offset(addr as i32 as isize)
}
#[inline]
unsafe fn RomPtr_91(addr: uint16_t, game: &mut Game) -> *const uint8 {
    RomPtr((0x910000u32) | ((addr) as u32), game)
}
#[inline]
unsafe fn RomPtr_9B(addr: uint16_t, game: &mut Game) -> *const uint8 {
    RomPtr((0x9b0000u32) | ((addr) as u32), game)
}
#[inline]
unsafe fn gVramWriteEntry(a: i32, game: &mut Game) -> *mut VramWriteEntry {
    (&raw mut game.g_ram as *mut uint8).offset((0xd0 as i32 + a) as isize) as *mut uint8
        as *mut VramWriteEntry
}
#[inline]
unsafe fn get_PoseEntry(a: uint16, game: &mut Game) -> *mut PoseEntry {
    RomPtr((0x910000u32) | ((a) as u32), game) as *mut PoseEntry
}
pub const fnSamus_MovementHandler_Normal: i32 = 0x90a337;
pub const fnProjPreInstr_Empty: i32 = 0x90b169;
pub const fnSamus_MoveHandlerVerticalShinespark: i32 = 0x90d0ab;
pub const fnSamus_MoveHandler_Shinespark_Diag: i32 = 0x90d0d7;
pub const fnSamus_MoveHandler_Shinespark_Horiz: i32 = 0x90d106;
pub const fnSamus_MoveHandler_Knockback: i32 = 0x90df38;
pub const fnSamus_MoveHandler_BombJumpStart: i32 = 0x90e025;
pub const fnSamus_Func3: i32 = 0x90e0e6;
pub const fnSamus_Func9: i32 = 0x90e41b;
pub const fnSamus_FrameHandlerAlfa_Func11: i32 = 0x90e695;
pub const fnSamus_FrameHandlerAlfa_Func12: i32 = 0x90e6c9;
pub const fnSamus_FrameHandlerAlfa_Func13: i32 = 0x90e713;
pub const fnSamus_FrameHandlerBeta_Func17: i32 = 0x90e725;
pub const fnSamus_FrameHandlerBeta_Func14: i32 = 0x90e7f5;
pub const fnSamus_Func16: i32 = 0x90e86a;
pub const fnEmptyFunction: i32 = 0x90e8cd;
pub const fnj_HandleDemoRecorder_2_0: i32 = 0x90e8d9;
pub const fnSetContactDamageIndexAndUpdateMinimap: i32 = 0x90e8dc;
pub const fnnullsub_152: i32 = 0x90e90e;
pub const fnSamus_InputHandler_E913: i32 = 0x90e913;
pub const fnSamus_Func20_: i32 = 0x90e918;
pub const fnSamus_InputHandler_E91D: i32 = 0x90e91d;
pub const fnHandleAutoJumpHack: i32 = 0x90e926;
pub const fnSamusMovementType_Xray: i32 = 0x90e94f;
pub const fnSamusDrawHandler_Default: i32 = 0x90eb52;
pub const fnSamus_InputHandler: i32 = 0x918000;
pub const fnDemoPreInstr_nullsub_162: i32 = 0x9183bf;
pub const fnDemoInstr_Finish: uint32 = 9536551;
pub const fnDemoInstr_SetPreInstr: uint32 = 9536564;
pub const fnDemoInstr_ClearPreInstr: uint32 = 9536575;
pub const fnDemoInstr_Goto: uint32 = 9536584;
pub const fnDemoInstr_DecTimerAndGoto: uint32 = 9536591;
pub const fnDemoInstr_SetTimer: uint32 = 9536601;
pub const fnDemoInstr_Func2: uint32 = 9537020;
pub const fnDemoPreInstr_864F: uint32 = 9537103;
pub const fnDemoPreInstr_866A: i32 = 0x91866a;
pub const fnDemoInstr_Disable: uint32 = 9537154;
pub const fnDemoInstr_Func3: uint32 = 9537337;
pub const fnDemoPreInstr_CheckLeaveDemo: i32 = 0x918a9b;
pub const fnDemoPreInstr_8AB0: uint32 = 9538224;
pub const fnXray_Func6: uint32 = 9555577;
pub const fnXray_Func6B: uint32 = 9555643;
pub const fnXray_Func7: uint32 = 9555766;
pub const fnXray_Func8: uint32 = 9555774;
pub const fnXray_Func9: i32 = 0x91cf4e;
pub const fnXray_Func10: uint32 = 9555810;
pub const fnXray_Func11: i32 = 0x91cf6f;
pub const fnGrappleBeamFunc_Inactive: i32 = 0x9bc4f0;
static kSamusInputHandlers: [Option<Func_V>; 28] = unsafe {
    [
        Some(Samus_Input_00_Standing as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_01_Running as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_02_NormalJumping as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_03_SpinJumping as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_04_MorphBallOnGround as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_05_Crouching as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_06_Falling as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_07_Unused as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_08_MorphBallFalling as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_09_Unused as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_0A_KnockbackOrCrystalFlashEnding as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_0B_Unused as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_0C_Unused as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_0D_Unused as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_0E_TurningAroundOnGround as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_0F_CrouchingEtcTransition as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_10_Moonwalking as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_11_SpringBallOnGround as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_12_SpringBallInAir as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_13_SpringBallFalling as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_14_WallJumping as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_15_RanIntoWall as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_16_Grappling as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_17_TurningAroundJumping as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_18_TurningAroundFalling as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_19_DamageBoost as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_1A_GrabbedByDraygon as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_1B_ShinesparkEtc as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn Samus_InputHandler(game: &mut Game) {
    kSamusInputHandlers[*(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as usize]
        .expect("non-null function pointer")(game);
}
pub unsafe fn Samus_Input_00_Standing(game: &mut Game) {
    if ram_word!(game, samus_pose) != 0u16
        && ram_word!(game, samus_pose) as i32 != kPose_9B_FaceF_VariaGravitySuit as i32
        || ram_word!(game, elevator_status) == 0
    {
        Samus_LookupTransitionTable(game);
    }
}
pub unsafe fn Samus_Input_01_Running(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_Input_02_NormalJumping(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_Input_03_SpinJumping(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_Input_04_MorphBallOnGround(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub fn Samus_Input_07_Unused(game: &mut Game) {}
pub unsafe fn Samus_Input_05_Crouching(game: &mut Game) {
    if ram_word!(game, time_is_frozen_flag) != 0 {
        Samus_Func20(game);
    } else {
        Samus_LookupTransitionTable(game);
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f) == 0 {
            update_ram_word!(game.samus_y_pos, wrapping_sub(5u16));
            update_ram_word!(game.samus_prev_y_pos, wrapping_sub(5u16));
        }
    };
}
pub unsafe fn Samus_Input_06_Falling(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_Input_08_MorphBallFalling(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub fn Samus_Input_09_Unused(game: &mut Game) {}
pub unsafe fn Samus_Input_0A_KnockbackOrCrystalFlashEnding(game: &mut Game) {
    Samus_LookupTransitionTable(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as i32
        != kMovementType_0A_KnockbackOrCrystalFlashEnding as i32
    {
        Samus_InitJump(game);
        set_ram_word!(game.samus_knockback_timer = 0 as uint16);
    }
}
pub fn Samus_Input_0B_Unused(game: &mut Game) {}
pub unsafe fn Samus_Input_0C_Unused(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_Input_0D_Unused(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_Input_0E_TurningAroundOnGround(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub fn Samus_Input_0F_CrouchingEtcTransition(game: &mut Game) {}
pub unsafe fn Samus_Input_10_Moonwalking(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_Input_11_SpringBallOnGround(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_Input_12_SpringBallInAir(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_Input_13_SpringBallFalling(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_Input_14_WallJumping(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_Input_15_RanIntoWall(game: &mut Game) {
    if ram_word!(game, time_is_frozen_flag) != 0 {
        Samus_Func20(game);
    } else {
        Samus_LookupTransitionTable(game);
    };
}
pub unsafe fn Samus_Input_16_Grappling(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_Input_17_TurningAroundJumping(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_Input_18_TurningAroundFalling(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_Input_19_DamageBoost(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_Input_1A_GrabbedByDraygon(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_Input_1B_ShinesparkEtc(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_LookupTransitionTable(game: &mut Game) {
    if ram_word!(game, joypad1_lastkeys) != 0 {
        let pair: Pair_R18_R20 = TranslateCustomControllerBindingsToDefault(game);
        let mut pe: *mut PoseEntry = get_PoseEntry(
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x919ee2)) as *const uint8 as *mut uint16)
                .offset(ram_word!(game, samus_pose) as isize),
            game,
        );
        if (*pe).new_input == 0xffffu16 {
            return;
        }
        loop {
            if (pair.r18 as i32 & (*pe).new_input as i32) == 0i32
                && (pair.r20 as i32 & (*pe).cur_input as i32) == 0i32
            {
                if (*pe).new_pose != ram_word!(game, samus_pose) {
                    set_ram_word!(game.samus_new_pose = (*pe).new_pose);
                    set_ram_word!(game.bomb_jump_dir = 0 as uint16);
                }
                return;
            }
            pe = pe.offset(1);
            if ((*pe).new_input == 0xffffu16) {
                break;
            }
        }
    }
    set_ram_word!(game.UNUSED_word_7E0A18 = 0 as uint16);
    Samus_Pose_CancelGrapple(game);
}
pub unsafe fn TranslateCustomControllerBindingsToDefault(game: &mut Game) -> Pair_R18_R20 {
    let mut r18: uint16 = (ram_word!(game, joypad1_newkeys) as i32
        & (kButton_Up as i32 | kButton_Down as i32 | kButton_Left as i32 | kButton_Right as i32))
        as uint16;
    let mut r20: uint16 = (ram_word!(game, joypad1_lastkeys) as i32
        & (kButton_Up as i32 | kButton_Down as i32 | kButton_Left as i32 | kButton_Right as i32))
        as uint16;
    let v0: uint16 = ram_word!(game, joypad1_newkeys);
    if (ram_word!(game, button_config_shoot_x) as i32 & ram_word!(game, joypad1_newkeys) as i32)
        != 0i32
    {
        r18 = (r18) | ((kButton_X) as u16);
    }
    if (ram_word!(game, button_config_jump_a) as i32 & v0 as i32) != 0i32 {
        r18 = (r18) | ((kButton_A) as u16);
    }
    if (ram_word!(game, button_config_run_b) as i32 & v0 as i32) != 0i32 {
        r18 = (r18) | ((kButton_B) as u16);
    }
    if (ram_word!(game, button_config_itemcancel_y) as i32 & v0 as i32) != 0i32 {
        r18 = (r18) | ((kButton_Y) as u16);
    }
    if (ram_word!(game, button_config_aim_up_R) as i32 & v0 as i32) != 0i32
        && ram_word!(game, button_config_aim_up_R) as i32 & (kButton_L as i32 | kButton_R as i32)
            != 0i32
    {
        r18 = (r18) | ((kButton_R) as u16);
    }
    if (ram_word!(game, button_config_aim_down_L) as i32 & v0 as i32) != 0i32
        && ram_word!(game, button_config_aim_down_L) as i32 & (kButton_L as i32 | kButton_R as i32)
            != 0i32
    {
        r18 = (r18) | ((kButton_L) as u16);
    }
    let v1: uint16 = ram_word!(game, joypad1_lastkeys);
    if (ram_word!(game, button_config_shoot_x) as i32 & ram_word!(game, joypad1_lastkeys) as i32)
        != 0i32
    {
        r20 = (r20) | ((kButton_X) as u16);
    }
    if (ram_word!(game, button_config_jump_a) as i32 & v1 as i32) != 0i32 {
        r20 = (r20) | ((kButton_A) as u16);
    }
    if (ram_word!(game, button_config_run_b) as i32 & v1 as i32) != 0i32 {
        r20 = (r20) | ((kButton_B) as u16);
    }
    if (ram_word!(game, button_config_itemcancel_y) as i32 & v1 as i32) != 0i32 {
        r20 = (r20) | ((kButton_Y) as u16);
    }
    if (ram_word!(game, button_config_aim_up_R) as i32 & v1 as i32) != 0i32
        && ram_word!(game, button_config_aim_up_R) as i32 & (kButton_L as i32 | kButton_R as i32)
            != 0i32
    {
        r20 = (r20) | ((kButton_R) as u16);
    }
    if (ram_word!(game, button_config_aim_down_L) as i32 & v1 as i32) != 0i32
        && ram_word!(game, button_config_aim_down_L) as i32 & (kButton_L as i32 | kButton_R as i32)
            != 0i32
    {
        r20 = (r20) | ((kButton_L) as u16);
    }
    Pair_R18_R20 {
        r18: !(r18 as i32) as uint16,
        r20: !(r20 as i32) as uint16,
    }
}
pub unsafe fn Samus_Pose_CancelGrapple(game: &mut Game) {
    if Samus_Pose_Func2(game) as i32 & 1 as i32 != 0
        || (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629)) as *const uint8
            as *mut SamusPoseParams)
            .offset(ram_word!(game, samus_pose) as isize))
        .new_pose_unless_buttons
            == 255u8
    {
        set_ram_word!(game.samus_new_pose = ram_word!(game, samus_pose));
    } else {
        set_ram_word!(
            game.samus_new_pose = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
                as *const uint8 as *mut SamusPoseParams)
                .offset(ram_word!(game, samus_pose) as isize))
            .new_pose_unless_buttons as uint16
        );
    };
}
static kSamus_Pose_Func2_Tab: [uint8; 28] = [
    2, 1, 1, 0, 6, 2, 8, 2, 1, 6, 2, 2, 2, 6, 2, 2, 2, 6, 6, 6, 6, 2, 6, 2, 2, 2, 2, 2,
];
pub unsafe fn Samus_Pose_Func2(game: &mut Game) -> uint8 {
    let mut v0: uint16 = kSamus_Pose_Func2_Tab
        [*(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as usize]
        as uint16;
    if (v0 == 1u16) {
        if ram_word!(game, samus_x_base_speed) == 0 && ram_word!(game, samus_x_base_subspeed) == 0 {
            v0 = 2 as uint16;
        } else {
            set_ram_word!(game.samus_momentum_routine_index = 1 as uint16);
            return 1 as uint8;
        }
    }
    set_ram_word!(game.samus_momentum_routine_index = v0);
    0 as uint8
}
pub fn EnableDemoInput(game: &mut Game) {
    game.set_samus_input_handler(fnSamus_InputHandler_E91D as u16);
    game.set_demo_enable(game.demo_enable() | 0x8000);
}
pub unsafe fn DisableDemoInput(game: &mut Game) {
    set_ram_word!(game.samus_input_handler = ((0x90e913i32) as u16));
    update_ram_word!(
        game.demo_enable,
        |fresh1| (fresh1 as i32 & !(0x8000 as i32)) as uint16
    );
}
pub fn ResetDemoData(game: &mut Game) {
    game.set_demo_input_pre_instr(0);
    game.set_demo_input_instr_timer(0);
    game.set_demo_input_instr_ptr(0);
    game.set_demo_timer_counter(0);
    game.set_xray_angle(0);
    game.set_demo_input(0);
    game.set_demo_input_new(0);
    game.set_demo_input_prev(0);
    game.set_demo_input_prev_new(0);
    game.set_demo_enable(0);
}
pub unsafe fn LoadDemoInputObject(angle: u16, object_ptr: u16, game: &mut Game) {
    game.set_xray_angle(angle);
    if object_ptr & 0x8000 == 0 {
        RomPtr_91(object_ptr, game);
    }
    let address = 0x918000 | u32::from(object_ptr);
    game.set_demo_input_pre_instr(
        game.read_rom_word(address, core::mem::offset_of!(DemoInputObject, pre_instr)),
    );
    game.set_demo_input_instr_ptr(
        game.read_rom_word(address, core::mem::offset_of!(DemoInputObject, instr_ptr)),
    );
    game.set_demo_input_instr_timer(1);
    game.set_demo_timer_counter(0);
}
pub unsafe fn DemoObjectInputHandler(game: &mut Game) {
    if game.demo_enable() & 0x8000 != 0 && game.demo_input_instr_ptr() != 0 {
        ProcessDemoInputObject(game);
        game.set_joypad1_input_samusfilter(game.demo_input_prev());
        game.set_joypad1_newinput_samusfilter(game.demo_input_prev_new());
        game.set_joypad1_lastkeys(game.demo_input());
        game.set_demo_input_prev(game.demo_input());
        game.set_joypad1_newkeys(game.demo_input_new());
        game.set_demo_input_prev_new(game.demo_input_new());
    }
}
pub unsafe fn CallDemoPreInstr(address: u32, game: &mut Game) {
    match address {
        0x9183bf => (),
        0x91864f => DemoPreInstr_864F(game),
        0x91866a => DemoPreInstr_866A(game),
        0x918a9b => DemoPreInstr_CheckLeaveDemo(game),
        0x918ab0 => DemoPreInstr_8AB0(game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn CallDemoInstr(address: u32, k: u16, j: u16, game: &mut Game) -> u16 {
    match address {
        0x918427 => DemoInstr_Finish(k, j, game),
        0x918434 => DemoInstr_SetPreInstr(k, j, game),
        0x91843f => DemoInstr_ClearPreInstr(k, j, game),
        0x918448 => DemoInstr_Goto(k, j, game),
        0x91844f => DemoInstr_DecTimerAndGoto(k, j, game),
        0x918459 => DemoInstr_SetTimer(k, j, game),
        0x9185fc => DemoInstr_Func2(k, j, game),
        0x918682 => DemoInstr_Disable(k, j, game),
        0x918739 => DemoInstr_Func3(k, j, game),
        _ => Unreachable(game) as u16,
    }
}
pub unsafe fn ProcessDemoInputObject(game: &mut Game) {
    CallDemoPreInstr(0x910000 | u32::from(game.demo_input_pre_instr()), game);
    let timer = game.demo_input_instr_timer().wrapping_sub(1);
    game.set_demo_input_instr_timer(timer);
    if timer == 0 {
        let mut instruction_ptr = game.demo_input_instr_ptr();
        let record_address = loop {
            if instruction_ptr & 0x8000 == 0 {
                RomPtr_91(instruction_ptr, game);
            }
            let address = 0x918000 | u32::from(instruction_ptr);
            let instruction = game.read_rom_word(address, 0);
            if game.read_rom_word(address, 0) & 0x8000 == 0 {
                break address;
            }
            instruction_ptr = CallDemoInstr(
                0x910000 | u32::from(instruction),
                0,
                instruction_ptr.wrapping_add(2),
                game,
            );
            if instruction_ptr == 0 {
                return;
            }
        };
        game.set_demo_input_instr_timer(game.read_rom_word(record_address, 0));
        if instruction_ptr & 0x8000 == 0 {
            RomPtr_91(instruction_ptr, game);
        }
        game.set_demo_input(game.read_rom_word(record_address, 2));
        game.set_demo_input_new(game.read_rom_word(record_address, 4));
        game.set_demo_input_instr_ptr(instruction_ptr.wrapping_add(6));
    }
}
pub fn DemoInstr_Finish(_k: u16, _j: u16, game: &mut Game) -> u16 {
    game.set_demo_input_instr_ptr(0);
    game.set_demo_input(0);
    game.set_demo_input_new(0);
    0
}
pub unsafe fn DemoInstr_SetPreInstr(k: uint16, j: uint16, game: &mut Game) -> uint16 {
    set_ram_word!(game.demo_input_pre_instr = *(RomPtr_91(j, game) as *mut uint16));
    (j).wrapping_add(2u16)
}
pub unsafe fn DemoInstr_ClearPreInstr(k: uint16, j: uint16, game: &mut Game) -> uint16 {
    set_ram_word!(game.demo_input_pre_instr = 0x8447 as uint16);
    j
}
pub unsafe fn DemoInstr_Goto(k: uint16, j: uint16, game: &mut Game) -> uint16 {
    *(RomPtr_91(j, game) as *mut uint16)
}
pub unsafe fn DemoInstr_DecTimerAndGoto(k: uint16, j: uint16, game: &mut Game) -> uint16 {
    if update_ram_word!(game.demo_timer_counter, wrapping_sub(1), reread) != 0 {
        DemoInstr_Goto(k, j, game)
    } else {
        (j).wrapping_add(2u16)
    }
}
pub unsafe fn DemoInstr_SetTimer(k: uint16, j: uint16, game: &mut Game) -> uint16 {
    set_ram_word!(game.demo_timer_counter = *(RomPtr_91(j, game) as *mut uint16));
    (j).wrapping_add(2u16)
}
pub unsafe fn DemoInstr_Func2(k: uint16, j: uint16, game: &mut Game) -> uint16 {
    set_ram_word!(game.frame_handler_alfa = ((0x90e695i32) as u16));
    set_ram_word!(game.frame_handler_beta = ((0x90e725i32) as u16));
    DisableDemoInput(game);
    j
}
pub unsafe fn DemoPreInstr_864F(game: &mut Game) {
    if ram_word!(game, samus_x_pos) as i32 - 178 as i32 & 0x8000 as i32 != 0 {
        set_ram_word!(game.demo_input_pre_instr = ((0x91866ai32) as u16));
        set_ram_word!(game.demo_input_instr_ptr = 0x8623 as uint16);
        set_ram_word!(game.demo_input_instr_timer = 1 as uint16);
    }
}
pub unsafe fn DemoPreInstr_866A(game: &mut Game) {
    if ram_word!(game, eproj_x_pos[0]) == 0 {
        set_ram_word!(game.demo_input_pre_instr = ((0x9183bfi32) as u16));
        set_ram_word!(game.demo_input_instr_ptr = addr_off_91864B as uint16);
        set_ram_word!(game.demo_input_instr_timer = 1 as uint16);
    }
}
pub unsafe fn DemoInstr_Disable(k: uint16, j: uint16, game: &mut Game) -> uint16 {
    set_ram_word!(game.frame_handler_alfa = ((0x90e8cdi32) as u16));
    set_ram_word!(game.frame_handler_beta = ((0x90e8cdi32) as u16));
    DisableDemoInput(game);
    j
}
pub unsafe fn UNUSED_DemoInstr_Func4(game: &mut Game) {
    set_ram_word!(game.frame_handler_alfa = ((0x90e713i32) as u16));
    set_ram_word!(game.samus_pose = kPose_02_FaceL_Normal as uint16);
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    set_ram_word!(game.samus_last_different_pose = ram_word!(game, samus_prev_pose));
    *((&raw mut game.g_ram as *mut uint8).offset(0xa26) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xa22) as *mut uint16);
    set_ram_word!(game.samus_prev_pose = ram_word!(game, samus_pose));
    *((&raw mut game.g_ram as *mut uint8).offset(0xa22) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1e) as *mut uint16);
    DisableDemoInput(game);
    set_ram_word!(game.samus_input_handler = ((0x90e90ei32) as u16));
}
pub unsafe fn DemoInstr_Func3(k: uint16, j: uint16, game: &mut Game) -> uint16 {
    set_ram_word!(game.frame_handler_alfa = ((0x90e713i32) as u16));
    set_ram_word!(game.samus_pose = kPose_02_FaceL_Normal as uint16);
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    set_ram_word!(game.samus_last_different_pose = ram_word!(game, samus_prev_pose));
    *((&raw mut game.g_ram as *mut uint8).offset(0xa26) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xa22) as *mut uint16);
    set_ram_word!(game.samus_prev_pose = ram_word!(game, samus_pose));
    *((&raw mut game.g_ram as *mut uint8).offset(0xa22) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1e) as *mut uint16);
    DisableDemoInput(game);
    set_ram_word!(game.samus_input_handler = ((0x90e90ei32) as u16));
    j
}
static kDemoSetFuncPtrs_0: [Option<Func_V>; 6] = unsafe {
    [
        Some(DemoSetFunc_0 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_1 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_1 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_2 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_1 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_3 as unsafe fn(&mut Game) -> ()),
    ]
};
static kDemoSetFuncPtrs_1: [Option<Func_V>; 6] = unsafe {
    [
        Some(DemoSetFunc_2 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_2 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_1 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_2 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_1 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_4 as unsafe fn(&mut Game) -> ()),
    ]
};
static kDemoSetFuncPtrs_2: [Option<Func_V>; 6] = unsafe {
    [
        Some(DemoSetFunc_2 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_5 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_1 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_1 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_1 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_1 as unsafe fn(&mut Game) -> ()),
    ]
};
static kDemoSetFuncPtrs_3: [Option<Func_V>; 5] = unsafe {
    [
        Some(DemoSetFunc_6 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_2 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_2 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_1 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_7 as unsafe fn(&mut Game) -> ()),
    ]
};
static kDemoSetFuncPtrs: [&'static [Option<Func_V>]; 4] = [
    &kDemoSetFuncPtrs_0,
    &kDemoSetFuncPtrs_1,
    &kDemoSetFuncPtrs_2,
    &kDemoSetFuncPtrs_3,
];
pub unsafe fn LoadDemoData(game: &mut Game) {
    let scene_offset = game.demo_scene().wrapping_mul(16);
    let set_offset = game.read_rom_word(0x918885, usize::from(game.demo_set()) * 2);
    let definition_offset = scene_offset.wrapping_add(set_offset);
    let definition_address = 0x910000 | u32::from(definition_offset);
    if definition_offset & 0x8000 == 0 {
        // Retain RomPtr's diagnostic/failed-frame flag for invalid windows.
        RomPtr(definition_address, game);
    }
    let definition_address = definition_address | 0x8000;
    game.set_collected_items(
        game.read_rom_word(definition_address, core::mem::offset_of!(DemoSetDef, items)),
    );
    game.set_equipped_items(game.collected_items());
    game.set_samus_max_missiles(game.read_rom_word(
        definition_address,
        core::mem::offset_of!(DemoSetDef, missiles),
    ));
    game.set_samus_missiles(game.samus_max_missiles());
    game.set_samus_max_super_missiles(game.read_rom_word(
        definition_address,
        core::mem::offset_of!(DemoSetDef, super_missiles),
    ));
    game.set_samus_super_missiles(game.samus_max_super_missiles());
    game.set_samus_max_power_bombs(game.read_rom_word(
        definition_address,
        core::mem::offset_of!(DemoSetDef, power_bombs),
    ));
    game.set_samus_power_bombs(game.samus_max_power_bombs());
    game.set_samus_max_health(game.read_rom_word(
        definition_address,
        core::mem::offset_of!(DemoSetDef, health),
    ));
    game.set_samus_health(game.samus_max_health());
    game.set_collected_beams(
        game.read_rom_word(definition_address, core::mem::offset_of!(DemoSetDef, beams)),
    );
    game.set_equipped_beams(game.read_rom_word(
        definition_address,
        core::mem::offset_of!(DemoSetDef, equipped_beams_),
    ));
    game.set_samus_reserve_health(0);
    ResetDemoData(game);
    EnableDemoInput(game);
    if definition_offset & 0x8000 == 0 {
        RomPtr(0x910000 | u32::from(definition_offset), game);
    }
    let demo_obj = game.read_rom_word(
        definition_address,
        core::mem::offset_of!(DemoSetDef, demo_obj),
    );
    LoadDemoInputObject(demo_obj, demo_obj, game);
    let set = usize::from(game.demo_set());
    let scene = usize::from(game.demo_scene());
    let initialize_scene = kDemoSetFuncPtrs[set][scene].expect("non-null function pointer");
    initialize_scene(game);
    game.set_frame_handler_alfa(fnSamus_FrameHandlerAlfa_Func12 as u16);
    game.set_frame_handler_beta(fnSamus_FrameHandlerBeta_Func14 as u16);
    game.set_samus_momentum_routine_index(0);
    game.set_samus_special_transgfx_index(0);
    game.set_samus_hurt_switch_index(0);
    Samus_LoadSuitPalette(game);
    UpdateBeamTilesAndPalette(game);
    game.set_button_config_up(kButton_Up as u16);
    game.set_button_config_down(kButton_Down as u16);
    game.set_button_config_left(kButton_Left as u16);
    game.set_button_config_right(kButton_Right as u16);
    game.set_button_config_shoot_x(kButton_X as u16);
    game.set_button_config_jump_a(kButton_A as u16);
    game.set_button_config_run_b(kButton_B as u16);
    game.set_button_config_itemcancel_y(kButton_Y as u16);
    game.set_button_config_itemswitch(kButton_Select as u16);
    game.set_button_config_aim_up_r(kButton_R as u16);
    game.set_button_config_aim_down_l(kButton_L as u16);
    game.set_unused_word_7e09e8(1);
    game.set_debug_flag(1);
    game.set_moonwalk_flag(0);
    game.set_unused_word_7e0df8(0);
    game.set_unused_word_7e0dfa(0);
    game.set_unused_word_7e0dfc(0);
}
pub unsafe fn DemoSetFunc_0(game: &mut Game) {
    MakeSamusFaceForward(game);
    game.set_samus_draw_handler(fnSamusDrawHandler_Default as u16);
}
pub unsafe fn DemoSetFunc_3(game: &mut Game) {
    DemoSetFunc_Common(kPose_1F_MoveL_Morphball_Ground as u16, game);
}
pub unsafe fn DemoSetFunc_7(game: &mut Game) {
    set_ram_word!(game.samus_health = 20 as uint16);
    DemoSetFunc_2(game);
}
pub unsafe fn DemoSetFunc_2(game: &mut Game) {
    DemoSetFunc_Common(kPose_02_FaceL_Normal as u16, game);
}
pub unsafe fn DemoSetFunc_4(game: &mut Game) {
    DemoSetFunc_Common(kPose_2A_FaceL_Fall as u16, game);
}
pub unsafe fn DemoSetFunc_1(game: &mut Game) {
    DemoSetFunc_Common(kPose_01_FaceR_Normal as u16, game);
}
pub unsafe fn DemoSetFunc_Common(pose: u16, game: &mut Game) {
    game.set_samus_pose(pose);
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    game.set_samus_draw_handler(fnSamusDrawHandler_Default as u16);
}
pub unsafe fn DemoSetFunc_5(game: &mut Game) {
    game.set_samus_draw_handler(fnSamusDrawHandler_Default as u16);
    Projectile_Func7_Shinespark(game);
    game.set_samus_pose(kPose_CD_FaceR_Shinespark_Diag as u16);
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
}
pub unsafe fn DemoSetFunc_6(game: &mut Game) {
    set_ram_word!(game.samus_draw_handler = ((0x90eb52i32) as u16));
    Projectile_Func7_Shinespark(game);
    set_ram_word!(game.samus_pose = kPose_CA_FaceL_Shinespark_Horiz as uint16);
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
}
pub unsafe fn DemoPreInstr_CheckLeaveDemo(game: &mut Game) {
    if ram_word!(game, game_state) as i32 == kGameState_44_TransitionFromDemo as i32 {
        set_ram_word!(game.demo_input_instr_ptr = addr_kDemoInstrs_LeaveDemo as uint16);
        set_ram_word!(game.demo_input_instr_timer = 1 as uint16);
    }
}
pub fn DemoPreInstr_8AB0(game: &mut Game) {
    if game.samus_movement_type() != kMovementType_1A_GrabbedByDraygon as u8 {
        game.set_demo_input_pre_instr(fnDemoPreInstr_CheckLeaveDemo as u16);
        game.set_demo_input_instr_ptr(addr_stru_919346 as u16);
        game.set_demo_input_instr_timer(1);
    }
}
unsafe fn XrayFillUp(dest: *mut uint16, fill_with: uint16, mut v1: uint16) {
    loop {
        *dest.offset((v1 as i32 >> 1 as i32) as isize) = fill_with;
        v1 = (v1).wrapping_sub(2u16);
        if (v1 as i32 & 0x8000 as i32 != 0) {
            break;
        }
    }
}
unsafe fn XrayFillDown(dest: *mut uint16, fill_with: uint16, mut v1: uint16) {
    loop {
        *dest.offset((v1 as i32 >> 1 as i32) as isize) = fill_with;
        v1 = (v1).wrapping_add(2u16);
        if (v1 >= 460u16) {
            break;
        }
    }
}
unsafe fn XrayFillUpRamped(
    dest: *mut uint16,
    mut pos: i32,
    mut left: uint32,
    left_step: uint32,
    mut right: uint32,
    right_step: uint32,
    adjust: i32,
) {
    loop {
        left = left.wrapping_add(left_step);
        right = right.wrapping_add(right_step);
        let left_val: uint32 = if left & !(0xffff as i32) as uint32 != 0 {
            (if (left as int32) < 0 as int32 {
                0 as i32
            } else {
                0xffff as i32
            }) as uint32
        } else {
            left
        };
        let right_val: uint32 = if right & !(0xffff as i32) as uint32 != 0 {
            (if (right as int32) < 0 as int32 {
                0 as i32
            } else {
                0xffff as i32
            }) as uint32
        } else {
            right
        };
        let finished: bool = left & !(0xffff as i32) as uint32 != 0
            && right & !(0xffff as i32) as uint32 != 0
            && ((left_step ^ left | right_step ^ right) >> 31 as i32) == 0u32;
        *dest.offset((pos >> 1 as i32) as isize) =
            (if (right as int32) < 0 as int32 || left as int32 >= 0x10000 as int32 {
                0xff as uint32
            } else {
                left_val >> 8 as i32 | right_val & !(0xff as i32) as uint32
            }) as uint16;
        if finished {
            pos += adjust;
            loop {
                pos -= 2 as i32;
                if (pos < 0i32) {
                    break;
                }
                *dest.offset((pos >> 1 as i32) as isize) = 0xff as uint16;
            }
            return;
        }
        pos -= 2 as i32;
        if (pos < 0i32) {
            break;
        }
    }
}
unsafe fn XrayFillDownRamped(
    dest: *mut uint16,
    mut pos: i32,
    mut left: uint32,
    left_step: uint32,
    mut right: uint32,
    right_step: uint32,
    adjust: i32,
) {
    loop {
        left = left.wrapping_add(left_step);
        right = right.wrapping_add(right_step);
        let left_val: uint32 = if left & !(0xffff as i32) as uint32 != 0 {
            (if (left as int32) < 0 as int32 {
                0 as i32
            } else {
                0xffff as i32
            }) as uint32
        } else {
            left
        };
        let right_val: uint32 = if right & !(0xffff as i32) as uint32 != 0 {
            (if (right as int32) < 0 as int32 {
                0 as i32
            } else {
                0xffff as i32
            }) as uint32
        } else {
            right
        };
        let finished: bool = left & !(0xffff as i32) as uint32 != 0
            && right & !(0xffff as i32) as uint32 != 0
            && ((left_step ^ left | right_step ^ right) >> 31 as i32) == 0u32;
        *dest.offset((pos >> 1 as i32) as isize) =
            (if (right as int32) < 0 as int32 || left as int32 >= 0x10000 as int32 {
                0xff as uint32
            } else {
                left_val >> 8 as i32 | right_val & !(0xff as i32) as uint32
            }) as uint16;
        if finished {
            pos += adjust;
            loop {
                pos += 2 as i32;
                if (pos >= 460i32) {
                    break;
                }
                *dest.offset((pos >> 1 as i32) as isize) = 0xff as uint16;
            }
            return;
        }
        pos += 2 as i32;
        if (pos >= 460i32) {
            break;
        }
    }
}
unsafe fn XrayHdmaFunc_BeamAimedR(ctx: *mut XrayHdmaCtx, game: &mut Game) {
    let p: i32 = 2 as i32 * (*ctx).r24 as i32;
    XrayFillUpRamped(
        (*ctx).dest,
        p - 2 as i32,
        (((*ctx).r22) as u32).wrapping_sub(0x10000u32),
        (*ctx).r30 as uint32,
        65535 as uint32,
        1 as uint32,
        2 as i32,
    );
    XrayFillDownRamped(
        (*ctx).dest,
        p,
        (((*ctx).r22) as u32).wrapping_sub(0x10000u32),
        (*ctx).r32 as uint32,
        65535 as uint32,
        1 as uint32,
        -(2 as i32),
    );
}
unsafe fn XrayHdmaFunc_BeamAimedL(ctx: *mut XrayHdmaCtx, game: &mut Game) {
    let t: i32 = 2 as i32 * (*ctx).r24 as i32;
    XrayFillUpRamped(
        (*ctx).dest,
        t - 2 as i32,
        0 as uint32,
        -(1 as i32) as uint32,
        (((*ctx).r22) as u32).wrapping_add(0x10000u32),
        -((*ctx).r32 as i32) as uint32,
        2 as i32,
    );
    XrayFillDownRamped(
        (*ctx).dest,
        t,
        0 as uint32,
        -(1 as i32) as uint32,
        (((*ctx).r22) as u32).wrapping_add(0x10000u32),
        -((*ctx).r30 as i32) as uint32,
        -(2 as i32),
    );
}
unsafe fn XrayHdmaFunc_BeamAimedU(ctx: *mut XrayHdmaCtx, game: &mut Game) {
    let p: i32 = 2 as i32 * (*ctx).r24 as i32;
    if (*ctx).r26 as i32 - 192 as i32 & 0x8000 as i32 != 0 {
        XrayFillUpRamped(
            (*ctx).dest,
            p - 2 as i32,
            (((*ctx).r22) as u32).wrapping_sub(0x10000u32),
            (*ctx).r30 as uint32,
            (((*ctx).r22) as u32).wrapping_sub(0x10000u32),
            (*ctx).r32 as uint32,
            0 as i32,
        );
    } else if (*ctx).r28 as i32 - 192 as i32 & 0x8000 as i32 != 0 {
        XrayFillUpRamped(
            (*ctx).dest,
            p - 2 as i32,
            (((*ctx).r22) as u32).wrapping_add(0x10000u32),
            -((*ctx).r30 as i32) as uint32,
            (((*ctx).r22) as u32).wrapping_sub(0x10000u32),
            (*ctx).r32 as uint32,
            0 as i32,
        );
    } else {
        XrayFillUpRamped(
            (*ctx).dest,
            p - 2 as i32,
            (((*ctx).r22) as u32).wrapping_add(0x10000u32),
            -((*ctx).r30 as i32) as uint32,
            (((*ctx).r22) as u32).wrapping_add(0x10000u32),
            -((*ctx).r32 as i32) as uint32,
            2 as i32,
        );
    }
    XrayFillDown((*ctx).dest, 0xff as uint16, p as uint16);
}
// Callers supply an even byte offset (2 * row), so the high-edge pass
// overwrites every temporary high byte written by the low-edge pass.
fn xray_onscreen_positive_stores(
    mut write: impl FnMut(u32, u8),
    base: u32,
    start: u16,
    origin: u16,
    low_step: u16,
    high_step: u16,
) {
    let mut low_edge = origin;
    let mut high_edge = origin;
    // Zero doubles as the ROM's no-carry sentinel, even for a carry at row 0.
    let mut carry_row = 0;
    // $91:C87A uses the N flag after CPY #$01CC, including wrapped Y.
    fn continues(index: u16) -> bool {
        index.wrapping_sub(460) & 0x8000 != 0
    }
    fn word(write: &mut impl FnMut(u32, u8), base: u32, index: u16, value: u16) {
        let address = base + u32::from(index);
        let [low, high] = value.to_le_bytes();
        write(address, low);
        write(address + 1, high);
    }

    let mut index = start;
    loop {
        let (value, carry) = low_edge.overflowing_add(low_step);
        low_edge = value;
        if carry {
            carry_row = index;
            loop {
                word(&mut write, base, index, 0x00ff);
                index = index.wrapping_add(2);
                if !continues(index) {
                    break;
                }
            }
            break;
        }
        // The high-edge pass overwrites this word's high byte.
        let value = low_edge >> 8;
        word(&mut write, base, index, value);
        index = index.wrapping_add(2);
        if !continues(index) {
            break;
        }
    }

    index = start.wrapping_add(1);
    loop {
        let (value, carry) = high_edge.overflowing_add(high_step);
        high_edge = value;
        if carry {
            loop {
                write(base + u32::from(index), 0xff);
                index = index.wrapping_add(2);
                if !continues(index) {
                    break;
                }
            }
            break;
        }
        write(base + u32::from(index), (high_edge >> 8) as u8);
        index = index.wrapping_add(2);
        if !continues(index) {
            // The ROM reaches cleanup only when this edge carries.
            return;
        }
    }

    if carry_row != 0 {
        index = carry_row.wrapping_add(2);
        while continues(index) {
            word(&mut write, base, index, 0x00ff);
            index = index.wrapping_add(2);
        }
    }
}
fn xray_offscreen_positive_stores(
    mut write: impl FnMut(u32, u8),
    base: u32,
    start: u16,
    origin: u16,
    low_step: u16,
    high_step: u16,
) {
    // The ROM wraps the 16-bit index before adding the table base.
    // At start zero, cleanup can therefore reach the following WRAM bank.
    fn byte(write: &mut impl FnMut(u32, u8), base: u32, index: i32, value: u8) {
        write(base + u32::from(index as u16), value);
    }
    fn word(write: &mut impl FnMut(u32, u8), base: u32, index: i32, value: u16) {
        let address = base + u32::from(index as u16);
        write(address, value as u8);
        write(address + 1, (value >> 8) as u8);
    }
    fn edge(
        write: &mut impl FnMut(u32, u8),
        base: u32,
        mut index: i32,
        mut value: u16,
        step: u16,
        high: bool,
    ) -> i32 {
        loop {
            let (next, carry) = value.overflowing_add(step);
            value = next;
            if carry {
                break;
            }
            index += 2;
            if index >= 460 {
                return index;
            }
        }
        if high {
            byte(write, base, index, (value >> 8) as u8);
        } else {
            word(write, base, index, value >> 8);
        }
        index += 2;
        if index >= 460 {
            return if high { 459 } else { 458 };
        }
        let crossing = index - 4;
        loop {
            let (next, carry) = value.overflowing_add(step);
            value = next;
            if carry {
                break;
            }
            if high {
                byte(write, base, index, (value >> 8) as u8);
            } else {
                word(write, base, index, value >> 8);
            }
            index += 2;
            if index >= 460 {
                return crossing;
            }
        }
        while index < 460 {
            if high {
                byte(write, base, index, 255);
            } else {
                word(write, base, index, 255);
            }
            index += 2;
        }
        crossing
    }
    // The low edge writes whole words first; the high edge then replaces
    // their high bytes. The cleanup passes depend on both crossing indices.
    let low = edge(&mut write, base, i32::from(start), origin, low_step, false);
    let high = edge(
        &mut write,
        base,
        i32::from(start) + 1,
        origin,
        high_step,
        true,
    );
    let mut index = low;
    loop {
        byte(&mut write, base, index, 0);
        index -= 2;
        if index < high {
            break;
        }
    }
    let mut index = high - 1;
    loop {
        word(&mut write, base, index, 255);
        index -= 2;
        if index < 0 {
            break;
        }
    }
}

unsafe fn XrayHdmaFunc_BeamAimedD(ctx: &XrayHdmaCtx, game: &mut Game) {
    let p: i32 = 2 as i32 * ctx.r24 as i32;
    if ctx.r28 as i32 - 128 as i32 & 0x8000 as i32 != 0 {
        // Current callers pass the $9100 or $9800 table inside this g_ram.
        let base = 0x7e0000u32 + ctx.dest.cast::<u8>().offset_from(game.g_ram.as_ptr()) as u32;
        xray_offscreen_positive_stores(
            |address, value| game.g_ram[(address - 0x7e0000) as usize] = value,
            base,
            ctx.r24.wrapping_mul(2),
            ctx.r22,
            ctx.r32,
            ctx.r30,
        );
        return;
    } else if ctx.r26 as i32 - 128 as i32 & 0x8000 as i32 != 0 {
        XrayFillDownRamped(
            ctx.dest,
            p,
            ((ctx.r22) as u32).wrapping_add(0x10000u32),
            -(ctx.r32 as i32) as uint32,
            ((ctx.r22) as u32).wrapping_sub(0x10000u32),
            ctx.r30 as uint32,
            0 as i32,
        );
    } else {
        XrayFillDownRamped(
            ctx.dest,
            p,
            ((ctx.r22) as u32).wrapping_add(0x10000u32),
            -(ctx.r32 as i32) as uint32,
            ((ctx.r22) as u32).wrapping_add(0x10000u32),
            -(ctx.r30 as i32) as uint32,
            0 as i32,
        );
    }
    XrayFillUp(ctx.dest, 0xff as uint16, (p - 2 as i32) as uint16);
}
unsafe fn XrayHdmaFunc_BeamHoriz(ctx: *mut XrayHdmaCtx, game: &mut Game) {
    let v0: uint16 = (2 as i32 * ((*ctx).r24 as i32 - 1 as i32)) as uint16;
    *(*ctx).dest.offset((v0 as i32 >> 1 as i32) as isize) = 0xff00 as uint16;
    XrayFillUp((*ctx).dest, 0xff as uint16, (v0).wrapping_sub(2u16));
    XrayFillDown((*ctx).dest, 0xff as uint16, (v0).wrapping_add(2u16));
}
unsafe fn XrayHdmaOnScreen_BeamAimedR(ctx: *mut XrayHdmaCtx, game: &mut Game) {
    let p: i32 = 2 as i32 * (*ctx).r24 as i32;
    *(*ctx).dest.offset((p - 2 as i32 >> 1 as i32) as isize) =
        ((*ctx).r22 as i32 >> 8 as i32 | 0xff00 as i32) as uint16;
    XrayFillUpRamped(
        (*ctx).dest,
        p - 4 as i32,
        (*ctx).r22 as uint32,
        (*ctx).r30 as uint32,
        65535 as uint32,
        1 as uint32,
        2 as i32,
    );
    XrayFillDownRamped(
        (*ctx).dest,
        p,
        (*ctx).r22 as uint32,
        (*ctx).r32 as uint32,
        65535 as uint32,
        1 as uint32,
        -(2 as i32),
    );
}
unsafe fn XrayHdmaOnScreen_BeamAimedL(ctx: *mut XrayHdmaCtx, game: &mut Game) {
    let v0: i32 = 2 as i32 * (*ctx).r24 as i32;
    *(*ctx).dest.offset((v0 - 2 as i32 >> 1 as i32) as isize) =
        (((*ctx).r22 as i32 >> 8 as i32) << 8 as i32) as uint16;
    XrayFillUpRamped(
        (*ctx).dest,
        v0 - 4 as i32,
        0 as uint32,
        -(1 as i32) as uint32,
        (*ctx).r22 as uint32,
        -((*ctx).r32 as i32) as uint32,
        2 as i32,
    );
    XrayFillDownRamped(
        (*ctx).dest,
        v0,
        0 as uint32,
        -(1 as i32) as uint32,
        (*ctx).r22 as uint32,
        -((*ctx).r30 as i32) as uint32,
        -(2 as i32),
    );
}
unsafe fn XrayHdmaOnScreen_BeamAimedU(ctx: *mut XrayHdmaCtx, game: &mut Game) {
    let v0: i32 = 2 as i32 * (*ctx).r24 as i32;
    *(*ctx).dest.offset((v0 - 2 as i32 >> 1 as i32) as isize) =
        (((*ctx).r22 as i32 >> 8 as i32) * 0x101 as i32) as uint16;
    if (*ctx).r26 as i32 - 192 as i32 & 0x8000 as i32 != 0 {
        XrayFillUpRamped(
            (*ctx).dest,
            v0 - 4 as i32,
            (*ctx).r22 as uint32,
            (*ctx).r30 as uint32,
            (*ctx).r22 as uint32,
            (*ctx).r32 as uint32,
            0 as i32,
        );
    } else if (*ctx).r28 as i32 - 192 as i32 & 0x8000 as i32 != 0 {
        XrayFillUpRamped(
            (*ctx).dest,
            v0 - 4 as i32,
            (*ctx).r22 as uint32,
            -((*ctx).r30 as i32) as uint32,
            (*ctx).r22 as uint32,
            (*ctx).r32 as uint32,
            0 as i32,
        );
    } else {
        XrayFillUpRamped(
            (*ctx).dest,
            v0 - 4 as i32,
            (*ctx).r22 as uint32,
            -((*ctx).r30 as i32) as uint32,
            (*ctx).r22 as uint32,
            -((*ctx).r32 as i32) as uint32,
            2 as i32,
        );
    }
    XrayFillDown((*ctx).dest, 0xff as uint16, v0 as uint16);
}
fn fill_xray_onscreen_positive_table(
    ram: &mut [u8],
    table: usize,
    start: u16,
    origin: u16,
    low_step: u16,
    high_step: u16,
) {
    // Wrap the byte offset before adding the table base, including at row zero.
    let anchor = table + usize::from(start.wrapping_sub(2));
    ram[anchor..anchor + 2].fill((origin >> 8) as u8);
    xray_onscreen_positive_stores(
        |address, value| ram[address as usize] = value,
        table as u32,
        start,
        origin,
        low_step,
        high_step,
    );

    let mut index = start.wrapping_sub(4);
    loop {
        // The ROM checks the first word even when the initial index is negative.
        let address = table + usize::from(index);
        if u16::from_le_bytes([ram[address], ram[address + 1]]) == 0x00ff {
            return;
        }
        ram[address..address + 2].copy_from_slice(&0x00ffu16.to_le_bytes());
        index = index.wrapping_sub(2);
        if index & 0x8000 != 0 {
            return;
        }
    }
}

unsafe fn XrayHdmaOnScreen_BeamAimedD(ctx: &XrayHdmaCtx, game: &mut Game) {
    if ctx.r28 as i32 - 128 as i32 & 0x8000 as i32 != 0 {
        // Current callers pass the $9100 or $9800 table inside this g_ram.
        let table = ctx.dest.cast::<u8>().offset_from(game.g_ram.as_ptr()) as usize;
        let start = ctx.r24.wrapping_mul(2);
        let origin = ctx.r22;
        let low_step = ctx.r32;
        let high_step = ctx.r30;
        fill_xray_onscreen_positive_table(
            &mut game.g_ram,
            table,
            start,
            origin,
            low_step,
            high_step,
        );
        return;
    }
    let v0: i32 = 2 as i32 * ctx.r24 as i32;
    *ctx.dest.offset((v0 - 2 as i32 >> 1 as i32) as isize) =
        ((ctx.r22 as i32 >> 8 as i32) * 0x101 as i32) as uint16;
    if ctx.r26 as i32 - 128 as i32 & 0x8000 as i32 != 0 {
        XrayFillDownRamped(
            ctx.dest,
            v0,
            ctx.r22 as uint32,
            -(ctx.r32 as i32) as uint32,
            ctx.r22 as uint32,
            ctx.r30 as uint32,
            0 as i32,
        );
    } else {
        XrayFillDownRamped(
            ctx.dest,
            v0,
            ctx.r22 as uint32,
            -(ctx.r32 as i32) as uint32,
            ctx.r22 as uint32,
            -(ctx.r30 as i32) as uint32,
            0 as i32,
        );
    }
    XrayFillUp(ctx.dest, 0xff as uint16, (v0 - 4 as i32) as uint16);
}
unsafe fn XrayHdmaOnScreen_BeamHoriz(ctx: *mut XrayHdmaCtx, game: &mut Game) {
    let v0: uint16 = (2 as i32 * ((*ctx).r24 as i32 - 1 as i32)) as uint16;
    let t: uint32 = ((*ctx).r22 as i32 >> 8 as i32) as uint32;
    *(*ctx).dest.offset((v0 as i32 >> 1 as i32) as isize) = (if (*ctx).r18_avoid == 64u16 {
        t | 0xff00 as uint32
    } else {
        t << 8 as i32
    }) as uint16;
    XrayFillUp((*ctx).dest, 0xff as uint16, (v0).wrapping_sub(2u16));
    XrayFillDown((*ctx).dest, 0xff as uint16, (v0).wrapping_add(2u16));
}
pub unsafe fn CalculateXrayHdmaTableInner(
    relative_x: u16,
    relative_y: u16,
    angle: u16,
    width: u16,
    off_screen: bool,
    dest_addr: *mut u16,
    game: &mut Game,
) {
    fn gradient_index(angle: u16) -> usize {
        let half_turn_offset = angle.wrapping_sub(128);
        usize::from(if half_turn_offset & 0x8000 != 0 {
            angle
        } else {
            half_turn_offset
        })
    }

    // Preserve the SNES sign-bit tests; angle 256 stays distinct from zero.
    let mut lower_angle = angle.wrapping_sub(width);
    if lower_angle & 0x8000 != 0 {
        lower_angle = lower_angle.wrapping_add(256);
    }
    let mut upper_angle = angle.wrapping_add(width);
    if upper_angle.wrapping_sub(257) & 0x8000 == 0 {
        upper_angle = upper_angle.wrapping_sub(256);
    }
    let lower_gradient = game.read_rom_word(0x91c9d4, gradient_index(lower_angle) * 2);
    let upper_gradient = game.read_rom_word(0x91c9d4, gradient_index(upper_angle) * 2);
    let mut ctx = XrayHdmaCtx {
        r22: relative_x << 8,
        r24: relative_y,
        r26: lower_angle,
        r28: upper_angle,
        r30: lower_gradient,
        r32: upper_gradient,
        r18_avoid: angle,
        dest: dest_addr,
    };
    let direction = if width == 0 && (angle == 64 || angle == 192) {
        XrayBeamDirection::Horizontal
    } else if lower_angle.wrapping_sub(128) & 0x8000 != 0 {
        if lower_angle.wrapping_sub(64) & 0x8000 == 0 {
            XrayBeamDirection::Down
        } else if upper_angle.wrapping_sub(64) & 0x8000 == 0 {
            XrayBeamDirection::Right
        } else {
            XrayBeamDirection::Up
        }
    } else if lower_angle.wrapping_sub(192) & 0x8000 == 0 {
        XrayBeamDirection::Up
    } else if upper_angle.wrapping_sub(192) & 0x8000 == 0 {
        XrayBeamDirection::Left
    } else {
        XrayBeamDirection::Down
    };
    match (off_screen, direction) {
        (true, XrayBeamDirection::Right) => XrayHdmaFunc_BeamAimedR(&raw mut ctx, game),
        (true, XrayBeamDirection::Up) => XrayHdmaFunc_BeamAimedU(&raw mut ctx, game),
        (true, XrayBeamDirection::Down) => XrayHdmaFunc_BeamAimedD(&ctx, game),
        (true, XrayBeamDirection::Left) => XrayHdmaFunc_BeamAimedL(&raw mut ctx, game),
        (true, XrayBeamDirection::Horizontal) => XrayHdmaFunc_BeamHoriz(&raw mut ctx, game),
        (false, XrayBeamDirection::Right) => XrayHdmaOnScreen_BeamAimedR(&raw mut ctx, game),
        (false, XrayBeamDirection::Up) => XrayHdmaOnScreen_BeamAimedU(&raw mut ctx, game),
        (false, XrayBeamDirection::Down) => XrayHdmaOnScreen_BeamAimedD(&ctx, game),
        (false, XrayBeamDirection::Left) => XrayHdmaOnScreen_BeamAimedL(&raw mut ctx, game),
        (false, XrayBeamDirection::Horizontal) => XrayHdmaOnScreen_BeamHoriz(&raw mut ctx, game),
    }
}
pub unsafe fn XrayRunHandler(game: &mut Game) {
    if ram_word!(game, time_is_frozen_flag) == 0
        && (ram_word!(game, button_config_run_b) as i32 & ram_word!(game, joypad1_lastkeys) as i32)
            != 0i32
        && Xray_Initialize(game) as i32 & 1 as i32 != 0
    {
        SpawnHdmaObject(
            0x91 as uint8,
            game.g_rom.offset(crate::rtl::lorom_offset(0x91caf2)) as *const uint8
                as *mut SpawnHdmaObject_Args as *const ::core::ffi::c_void,
            game,
        );
    }
}
pub unsafe fn Xray_SetupStage1_FreezeTimeBackup(k: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa78) as *mut uint16 as *mut uint8).offset(0) =
        1 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16 as *mut uint8)
        .offset(k as uint8 as i32 as isize) = ram_word!(game, reg_BG2HOFS) as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16 as *mut uint8)
        .offset(k as uint8 as i32 as isize)
        .offset(1) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1920) as *mut uint16 as *mut uint8)
        .offset(k as uint8 as i32 as isize) = ram_word!(game, reg_BG2VOFS) as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1920) as *mut uint16 as *mut uint8)
        .offset(k as uint8 as i32 as isize)
        .offset(1) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        );
    *((&raw mut game.g_ram as *mut uint8).offset(0x192c) as *mut uint16 as *mut uint8)
        .offset(k as uint8 as i32 as isize) = *(&raw mut game.g_ram as *mut uint8).offset(0x59);
}
pub unsafe fn Xray_SetupStage2_ReadBg1_2ndScreen(game: &mut Game) {
    let v0: uint16 = ram_word!(game, vram_read_queue_tail);
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340) as *mut VramReadQueueEnt)
        .offset(0))
    .vram_target as *mut uint8)
        .offset(v0 as i32 as isize) as *mut uint16) =
        (((*(&raw mut game.g_ram as *mut uint8).offset(0x58) as i32 & 0xfc as i32) << 8 as i32)
            + 1024 as i32) as uint16;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340) as *mut VramReadQueueEnt)
        .offset(0))
    .dma_parameters as *mut uint8)
        .offset(v0 as i32 as isize) as *mut uint16) = 129 as uint16;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340) as *mut VramReadQueueEnt)
        .offset(0))
    .dma_parameters as *mut uint8)
        .offset(v0 as i32 as isize)
        .offset(1) as *mut uint16) = 57 as uint16;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340) as *mut VramReadQueueEnt)
        .offset(0))
    .src
    .addr as *mut uint8)
        .offset(v0 as i32 as isize) as *mut VoidP) =
        (((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000 as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long
            + 0x2800 as ::core::ffi::c_long) as VoidP;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340) as *mut VramReadQueueEnt)
        .offset(0))
    .src
    .bank)
        .offset(v0 as i32 as isize) as *mut uint16) = 126 as uint16;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340) as *mut VramReadQueueEnt)
        .offset(0))
    .size as *mut uint8)
        .offset(v0 as i32 as isize) as *mut uint16) = 2048 as uint16;
    set_ram_word!(game.vram_read_queue_tail = (v0).wrapping_add(9u16));
}
pub unsafe fn Xray_SetupStage3_ReadBg1_1stScreen(game: &mut Game) {
    let v0: uint16 = ram_word!(game, vram_read_queue_tail);
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340) as *mut VramReadQueueEnt)
        .offset(0))
    .vram_target as *mut uint8)
        .offset(v0 as i32 as isize) as *mut uint16) =
        ((*(&raw mut game.g_ram as *mut uint8).offset(0x58) as i32 & 0xfc as i32) << 8 as i32)
            as uint16;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340) as *mut VramReadQueueEnt)
        .offset(0))
    .dma_parameters as *mut uint8)
        .offset(v0 as i32 as isize) as *mut uint16) = 129 as uint16;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340) as *mut VramReadQueueEnt)
        .offset(0))
    .dma_parameters as *mut uint8)
        .offset(v0 as i32 as isize)
        .offset(1) as *mut uint16) = 57 as uint16;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340) as *mut VramReadQueueEnt)
        .offset(0))
    .src
    .addr as *mut uint8)
        .offset(v0 as i32 as isize) as *mut VoidP) =
        (((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000 as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long
            + 0x2000 as ::core::ffi::c_long) as VoidP;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340) as *mut VramReadQueueEnt)
        .offset(0))
    .src
    .bank)
        .offset(v0 as i32 as isize) as *mut uint16) = 126 as uint16;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340) as *mut VramReadQueueEnt)
        .offset(0))
    .size as *mut uint8)
        .offset(v0 as i32 as isize) as *mut uint16) = 2048 as uint16;
    set_ram_word!(game.vram_read_queue_tail = (v0).wrapping_add(9u16));
}
pub unsafe fn Xray_SetupStage4(game: &mut Game) {
    let mut r24: uint16 = (4 as i32
        * (ram_word!(game, layer1_y_pos) as i32 + ram_word!(game, bg1_y_offset) as i32
            & 0xf0 as i32)) as uint16;
    r24 = (r24 as i32
        + ((ram_word!(game, layer1_x_pos) as i32 + ram_word!(game, bg1_x_offset) as i32
            & 0xf0 as i32)
            >> 3 as i32)) as uint16;
    r24 = (r24 as i32
        + 4 as i32
            * (ram_word!(game, layer1_x_pos) as i32 + ram_word!(game, bg1_x_offset) as i32
                & 0x100 as i32)) as uint16;
    let mut r22: uint16 = 0 as uint16;
    let mut m: i32 = 16 as i32;
    loop {
        let mut n: i32 = 16 as i32;
        let mut r26: uint16 = (r24) & (0x7e0u16);
        let mut r28: uint16 = (r24) & (0x1fu16);
        let mut r30: uint16 = 0 as uint16;
        loop {
            if r30 as i32 + r28 as i32 - 32 as i32 & 0x8000 as i32 == 0 {
                r26 = (r26 as i32 + 1024 as i32 & 0x7e0 as i32) as uint16;
                r28 = 0 as uint16;
                r30 = 0 as uint16;
            }
            let v0: uint16 = (r30 as i32 + r28 as i32 + r26 as i32) as uint16;
            let v11: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x4000)
                as *mut Ram4000))
                .xray_tilemaps[(v0 as i32 + 4096 as i32) as usize];
            let v10: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x4000)
                as *mut Ram4000))
                .xray_tilemaps[(v0 as i32 + 4097 as i32) as usize];
            let v9: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x4000)
                as *mut Ram4000))
                .xray_tilemaps[(v0 as i32 + 4128 as i32) as usize];
            let v1: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x4000)
                as *mut Ram4000))
                .xray_tilemaps[(v0 as i32 + 4129 as i32) as usize];
            *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
                .xray_tilemaps as *mut uint16)
                .offset(33) as *mut uint16 as *mut uint8)
                .offset(r22 as i32 as isize) as *mut uint16) = v1;
            *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
                .xray_tilemaps as *mut uint16)
                .offset(32) as *mut uint16 as *mut uint8)
                .offset(r22 as i32 as isize) as *mut uint16) = v9;
            *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
                .xray_tilemaps as *mut uint16)
                .offset(1) as *mut uint16 as *mut uint8)
                .offset(r22 as i32 as isize) as *mut uint16) = v10;
            *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
                .xray_tilemaps as *mut uint16 as *mut uint8)
                .offset(r22 as i32 as isize) as *mut uint16) = v11;
            r22 = (r22).wrapping_add(4u16);
            r30 = (r30).wrapping_add(2u16);
            n -= 1;
            if (n == 0) {
                break;
            }
        }
        Xray_SetupStage4_Func1(r22, r26, r28, r30, game);
        r24 = ((r24 as i32 & 0x400 as i32) + (r24 as i32 + 64 as i32 & 0x3ff as i32)) as uint16;
        r22 = (r22).wrapping_add(64u16);
        m -= 1;
        if (m == 0) {
            break;
        }
    }
    '_c2rust_label: {
        if ((ram_word!(game, layer1_y_pos) as i32 >> 12 as i32) as uint16) == 0u16 {
        } else {
            __assert_fail(
                b"(uint16)(layer1_y_pos >> 12) == 0\0" as *const u8 as *const ::core::ffi::c_char,
                b"src/sm_91.c\0" as *const u8 as *const ::core::ffi::c_char,
                921 as u32,
                __ASSERT_FUNCTION.as_ptr(),
            );
        }
    };
    let mut r34: uint16 = (Mult8x8(
        (ram_word!(game, layer1_y_pos) as i32 >> 4 as i32) as uint8,
        ram_word!(game, room_width_in_blocks) as uint8,
    ) as i32
        + (ram_word!(game, layer1_x_pos) as i32 >> 4 as i32)) as uint16;
    r22 = 0 as uint16;
    m = 16 as i32;
    loop {
        Xray_SetupStage4_Func2(r22, r34, game);
        let mut n_0: i32 = 16 as i32;
        let mut r36: uint16 = r34;
        loop {
            Xray_SetupStage4_Func3(n_0 as uint16, r22, r36, game);
            r22 = (r22).wrapping_add(4u16);
            r36 = (r36).wrapping_add(1u16);
            n_0 -= 1;
            if (n_0 == 0) {
                break;
            }
        }
        let v12: uint16 = r22;
        r22 = (r22).wrapping_add(1984u16);
        Xray_SetupStage4_Func3(0 as uint16, r22, r36, game);
        r22 = (v12).wrapping_add(64u16);
        r34 = (r34).wrapping_add(ram_word!(game, room_width_in_blocks));
        m -= 1;
        if (m == 0) {
            break;
        }
    }
    LoadXrayBlocks(game);
    let v7: uint16 = ram_word!(game, vram_read_queue_tail);
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340) as *mut VramReadQueueEnt)
        .offset(0))
    .vram_target as *mut uint8)
        .offset(v7 as i32 as isize) as *mut uint16) =
        ((*(&raw mut game.g_ram as *mut uint8).offset(0x59) as i32 & 0xfc as i32) << 8 as i32)
            as uint16;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340) as *mut VramReadQueueEnt)
        .offset(0))
    .dma_parameters as *mut uint8)
        .offset(v7 as i32 as isize) as *mut uint16) = 129 as uint16;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340) as *mut VramReadQueueEnt)
        .offset(0))
    .dma_parameters as *mut uint8)
        .offset(v7 as i32 as isize)
        .offset(1) as *mut uint16) = 57 as uint16;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340) as *mut VramReadQueueEnt)
        .offset(0))
    .src
    .addr as *mut uint8)
        .offset(v7 as i32 as isize) as *mut VoidP) = 0x5000 as VoidP;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340) as *mut VramReadQueueEnt)
        .offset(0))
    .src
    .bank)
        .offset(v7 as i32 as isize) as *mut uint16) = 126 as uint16;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340) as *mut VramReadQueueEnt)
        .offset(0))
    .size as *mut uint8)
        .offset(v7 as i32 as isize) as *mut uint16) = 2048 as uint16;
    set_ram_word!(game.vram_read_queue_tail = (v7).wrapping_add(9u16));
}
pub unsafe fn Xray_SetupStage4_Func1(
    dst_r22: uint16,
    r26: uint16,
    r28: uint16,
    r30: uint16,
    game: &mut Game,
) {
    let mut v0: int16 = 0;
    let mut R32: uint16 = r26;
    v0 = ((r30) as i16).wrapping_add((r28) as i16);
    if r30 as i32 + r28 as i32 - 32 as i32 & 0x8000 as i32 == 0 {
        R32 = (R32 as i32 + 1024 as i32 & 0x7e0 as i32) as uint16;
        v0 = 0 as int16;
    }
    let v1: uint16 = (R32).wrapping_add((v0) as u16);
    let v6: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
        .xray_tilemaps[(v1 as i32 + 4096 as i32) as usize];
    let v5: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
        .xray_tilemaps[(v1 as i32 + 4097 as i32) as usize];
    let v4: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
        .xray_tilemaps[(v1 as i32 + 4128 as i32) as usize];
    let v3: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
        .xray_tilemaps[(v1 as i32 + 4129 as i32) as usize];
    let v2: uint16 = (dst_r22).wrapping_add(1984u16);
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(33) as *mut uint16 as *mut uint8)
        .offset(v2 as i32 as isize) as *mut uint16) = v3;
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(32) as *mut uint16 as *mut uint8)
        .offset(v2 as i32 as isize) as *mut uint16) = v4;
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(1) as *mut uint16 as *mut uint8)
        .offset(v2 as i32 as isize) as *mut uint16) = v5;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
        .xray_tilemaps as *mut uint16 as *mut uint8)
        .offset(v2 as i32 as isize) as *mut uint16) = v6;
}
unsafe fn Xray_GetXrayedBlock(k: uint16, game: &mut Game) -> *const uint8 {
    let mut value: uint16 = 0;
    let bts: uint16 = *(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402)
        .offset(k as isize) as uint16;
    let r40: uint16 = (ram_word!(game, level_data[k])) & (0xf000u16);
    let mut i: i32 = 0 as i32;
    loop {
        value = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91d2d6)) as *const uint8
            as *mut XrayBlockData)
            .offset(i as isize))
        .value;
        if value == 0xffffu16 {
            break;
        }
        if value == r40 {
            let mut p: *const uint8 = RomPtr_91(
                (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91d2d6)) as *const uint8
                    as *mut XrayBlockData)
                    .offset(i as isize))
                .addr,
                game,
            );
            loop {
                value = *(p as *mut uint16);
                if value == 0xffffu16 {
                    break;
                }
                if value == 0xff00u16 || value == bts {
                    return RomPtr_91(*(p.offset(2) as *mut uint16), game);
                }
                p = p.offset(4);
            }
            return ::core::ptr::null::<uint8>();
        }
        i += 1;
    }
    ::core::ptr::null::<uint8>()
}
pub unsafe fn Xray_SetupStage4_Func2(dst_r22: uint16, r34: uint16, game: &mut Game) {
    if r34 == 0u16 {
        return;
    }
    let jp: *const uint8 = Xray_GetXrayedBlock((r34).wrapping_sub(1u16), game);
    let dst: *mut uint16 = (&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x4000)
        as *mut Ram4000))
        .xray_tilemaps as *mut uint16 as *mut uint8)
        .offset(dst_r22 as i32 as isize) as *mut uint16;
    if jp.is_null() {
        return;
    }
    if *(jp as *mut uint16) as i32 != 0x91cf4e as i32 & 0xffff as i32 {
        if *(jp as *mut uint16) as i32 != 0x91cf6f as i32 & 0xffff as i32 {
            return;
        }
        let src: *mut TileTable = (&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
            as *mut TileTables))
            .tables as *mut TileTable)
            .offset(*(jp.offset(8) as *mut uint16) as i32 as isize);
        *dst.offset(64) = (*src).top_left;
        *dst.offset(65) = (*src).top_right;
        *dst.offset(96) = (*src).bottom_left;
        *dst.offset(97) = (*src).bottom_right;
    }
    let src_0: *mut TileTable = (&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
        as *mut TileTables))
        .tables as *mut TileTable)
        .offset(*(jp.offset(4) as *mut uint16) as i32 as isize);
    *dst.offset(0) = (*src_0).top_left;
    *dst.offset(1) = (*src_0).top_right;
    *dst.offset(32) = (*src_0).bottom_left;
    *dst.offset(33) = (*src_0).bottom_right;
}
unsafe fn Xray_CombinedMove(dst_r22: uint16, r36: uint16, which_dir: bool, game: &mut Game) {
    let mut y: uint16 = (r36 as i32 / ram_word!(game, room_width_in_blocks) as i32) as uint16;
    let mut x: uint16 = (r36 as i32 % ram_word!(game, room_width_in_blocks) as i32) as uint16;
    let mut r48: uint16 = (ram_word!(game, level_data[r36])) & (0xf000u16);
    let mut step: i32 = *(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402)
        .offset(r36 as isize) as int8 as i32;
    if step != 0 {
        let mut variable: *mut uint16 = if which_dir { &raw mut y } else { &raw mut x };
        loop {
            if ((*variable as i32 + step) as int16) < 0i16 {
                Xray_Func13(dst_r22, 0xff as uint16, game);
                return;
            }
            *variable = (*variable as i32 + step) as uint16;
            let pos: i32 = y as i32 * ram_word!(game, room_width_in_blocks) as i32 + x as i32;
            step = *(&raw mut game.g_ram as *mut uint8)
                .offset(0x16402)
                .offset(pos as isize) as int8 as i32;
            r48 = (ram_word!(game, level_data[pos])) & (0xf000u16);
            if r48 == 0xd000u16 {
                variable = &raw mut y;
            } else {
                if r48 != 0x5000u16 {
                    break;
                }
                if step < 0i32 {
                    variable = &raw mut x;
                }
            }
        }
    }
    if (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91d2d6)) as *const uint8
        as *mut XrayBlockData)
        .offset(1))
    .value
        == r48
    {
        let mut t: *mut uint16 = RomPtr_91(
            (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91d2d6)) as *const uint8
                as *mut XrayBlockData)
                .offset(1))
            .addr,
            game,
        ) as *mut uint16;
        while *t.offset(0) != 0xffffu16 {
            if *t.offset(0) == 0xff00u16 || *t.offset(0) as i32 == step & 0xff as i32 {
                Xray_Func12(
                    dst_r22,
                    RomPtr_91((*t.offset(1)).wrapping_add(2u16), game),
                    game,
                );
                return;
            }
            t = t.offset(2);
        }
    }
}
unsafe fn Xray_Func7(dst_r22: uint16, jp: *const uint8, game: &mut Game) {
    Xray_Func12(dst_r22, jp.offset(2), game);
}
unsafe fn Xray_Func8(dst_r22: uint16, jp: *const uint8, game: &mut Game) {
    if ram_word!(game, area_index) == 1u16 {
        Xray_Func12(dst_r22, jp.offset(2), game);
    }
}
unsafe fn Xray_Func9(r18: uint16, dst_r22: uint16, jp: *const uint8, game: &mut Game) {
    Xray_Func12(dst_r22, jp.offset(2), game);
    if r18 != 1u16 {
        Xray_Func14(dst_r22, jp.offset(4), game);
    }
}
unsafe fn Xray_Func10(dst_r22: uint16, jp: *const uint8, game: &mut Game) {
    Xray_Func12(dst_r22, jp.offset(2), game);
    Xray_Func15(dst_r22, jp.offset(2), game);
}
unsafe fn Xray_Func11(r18: uint16, dst_r22: uint16, jp: *const uint8, game: &mut Game) {
    Xray_Func12(dst_r22, jp.offset(2), game);
    if r18 != 1u16 {
        Xray_Func14(dst_r22, jp.offset(4), game);
    }
    Xray_Func15(dst_r22, jp.offset(6), game);
    if r18 != 1u16 {
        let v5: uint16 = *(jp.offset(8) as *mut uint16);
        let top_left: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
            as *mut TileTables))
            .tables[v5 as usize]
            .top_left;
        let top_right: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
            as *mut TileTables))
            .tables[v5 as usize]
            .top_right;
        let bottom_left: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
            as *mut TileTables))
            .tables[v5 as usize]
            .bottom_left;
        let bottom_right: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
            as *mut TileTables))
            .tables[v5 as usize]
            .bottom_right;
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
            .xray_tilemaps as *mut uint16)
            .offset(99) as *mut uint16 as *mut uint8)
            .offset(dst_r22 as i32 as isize) as *mut uint16) = bottom_right;
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
            .xray_tilemaps as *mut uint16)
            .offset(98) as *mut uint16 as *mut uint8)
            .offset(dst_r22 as i32 as isize) as *mut uint16) = bottom_left;
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
            .xray_tilemaps as *mut uint16)
            .offset(67) as *mut uint16 as *mut uint8)
            .offset(dst_r22 as i32 as isize) as *mut uint16) = top_right;
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
            .xray_tilemaps as *mut uint16)
            .offset(66) as *mut uint16 as *mut uint8)
            .offset(dst_r22 as i32 as isize) as *mut uint16) = top_left;
    }
}
unsafe fn CallXrayFunc(
    ea: uint32,
    jp: *const uint8,
    r18: uint16,
    r22: uint16,
    r36: uint16,
    game: &mut Game,
) {
    match ea {
        9555577 => {
            Xray_CombinedMove(r22, r36, true_0 != 0, game);
        }
        9555643 => {
            Xray_CombinedMove(r22, r36, false_0 != 0, game);
        }
        9555766 => Xray_Func7(r22, jp, game),
        9555774 => Xray_Func8(r22, jp, game),
        9555790 => Xray_Func9(r18, r22, jp, game),
        9555810 => Xray_Func10(r22, jp, game),
        9555823 => Xray_Func11(r18, r22, jp, game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn Xray_SetupStage4_Func3(r18: uint16, r22: uint16, r36: uint16, game: &mut Game) {
    let jp: *const uint8 = Xray_GetXrayedBlock(r36, game);
    if !jp.is_null() {
        CallXrayFunc(
            ((*(jp as *mut uint16)) as u32) | (0x910000u32),
            jp,
            r18,
            r22,
            r36,
            game,
        );
    }
}
unsafe fn Xray_Func12(dst_r22: uint16, jp: *const uint8, game: &mut Game) {
    Xray_Func13(dst_r22, *(jp as *mut uint16), game);
}
unsafe fn Xray_Func13(dst_r22: uint16, a: uint16, game: &mut Game) {
    let top_left: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
        as *mut TileTables))
        .tables[a as usize]
        .top_left;
    let top_right: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
        as *mut TileTables))
        .tables[a as usize]
        .top_right;
    let bottom_left: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
        as *mut TileTables))
        .tables[a as usize]
        .bottom_left;
    let bottom_right: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
        as *mut TileTables))
        .tables[a as usize]
        .bottom_right;
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(33) as *mut uint16 as *mut uint8)
        .offset(dst_r22 as i32 as isize) as *mut uint16) = bottom_right;
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(32) as *mut uint16 as *mut uint8)
        .offset(dst_r22 as i32 as isize) as *mut uint16) = bottom_left;
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(1) as *mut uint16 as *mut uint8)
        .offset(dst_r22 as i32 as isize) as *mut uint16) = top_right;
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(0) as *mut uint16 as *mut uint8)
        .offset(dst_r22 as i32 as isize) as *mut uint16) = top_left;
}
unsafe fn Xray_Func14(dst_r22: uint16, jp: *const uint8, game: &mut Game) {
    let a: uint16 = *(jp as *mut uint16);
    let top_left: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
        as *mut TileTables))
        .tables[a as usize]
        .top_left;
    let top_right: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
        as *mut TileTables))
        .tables[a as usize]
        .top_right;
    let bottom_left: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
        as *mut TileTables))
        .tables[a as usize]
        .bottom_left;
    let bottom_right: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
        as *mut TileTables))
        .tables[a as usize]
        .bottom_right;
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(35) as *mut uint16 as *mut uint8)
        .offset(dst_r22 as i32 as isize) as *mut uint16) = bottom_right;
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(34) as *mut uint16 as *mut uint8)
        .offset(dst_r22 as i32 as isize) as *mut uint16) = bottom_left;
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(3) as *mut uint16 as *mut uint8)
        .offset(dst_r22 as i32 as isize) as *mut uint16) = top_right;
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(2) as *mut uint16 as *mut uint8)
        .offset(dst_r22 as i32 as isize) as *mut uint16) = top_left;
}
unsafe fn Xray_Func15(dst_r22: uint16, jp: *const uint8, game: &mut Game) {
    let a: uint16 = *(jp as *mut uint16);
    let top_left: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
        as *mut TileTables))
        .tables[a as usize]
        .top_left;
    let top_right: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
        as *mut TileTables))
        .tables[a as usize]
        .top_right;
    let bottom_left: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
        as *mut TileTables))
        .tables[a as usize]
        .bottom_left;
    let bottom_right: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
        as *mut TileTables))
        .tables[a as usize]
        .bottom_right;
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(97) as *mut uint16 as *mut uint8)
        .offset(dst_r22 as i32 as isize) as *mut uint16) = bottom_right;
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(96) as *mut uint16 as *mut uint8)
        .offset(dst_r22 as i32 as isize) as *mut uint16) = bottom_left;
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(65) as *mut uint16 as *mut uint8)
        .offset(dst_r22 as i32 as isize) as *mut uint16) = top_right;
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(64) as *mut uint16 as *mut uint8)
        .offset(dst_r22 as i32 as isize) as *mut uint16) = top_left;
}
unsafe fn Xray_Func16(dst_r22: uint16, a: uint16, game: &mut Game) {
    let top_left: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
        as *mut TileTables))
        .tables[a as usize]
        .top_left;
    let top_right: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
        as *mut TileTables))
        .tables[a as usize]
        .top_right;
    let bottom_left: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
        as *mut TileTables))
        .tables[a as usize]
        .bottom_left;
    let bottom_right: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
        as *mut TileTables))
        .tables[a as usize]
        .bottom_right;
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(1) as *mut uint16 as *mut uint8)
        .offset(dst_r22 as i32 as isize) as *mut uint16) = bottom_right;
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(0) as *mut uint16 as *mut uint8)
        .offset(dst_r22 as i32 as isize) as *mut uint16) = bottom_left;
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(33) as *mut uint16 as *mut uint8)
        .offset(dst_r22 as i32 as isize) as *mut uint16) = top_right;
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(32) as *mut uint16 as *mut uint8)
        .offset(dst_r22 as i32 as isize) as *mut uint16) = top_left;
}
pub unsafe fn LoadBlockToXrayTilemap(a: uint16, k: uint16, j: uint16, game: &mut Game) {
    let R24: uint16 = (k as i32 - (ram_word!(game, layer1_x_pos) as i32 >> 4 as i32)) as uint16;
    let R26: uint16 = (j as i32 - (ram_word!(game, layer1_y_pos) as i32 >> 4 as i32)) as uint16;
    if R24 as i32 & 0x8000 as i32 == 0
        && R24 as i32 - 16 as i32 & 0x8000 as i32 != 0
        && R26 as i32 & 0x8000 as i32 == 0
        && R26 as i32 - 16 as i32 & 0x8000 as i32 != 0
    {
        let R22: uint16 = (4 as i32 * (R24 as i32 + 32 as i32 * R26 as i32)) as uint16;
        if (a as i32 & 0x800 as i32) != 0i32 {
            Xray_Func16(R22, (a) & (0x3ffu16), game);
        } else {
            Xray_Func13(R22, (a) & (0x3ffu16), game);
        }
    }
}
pub unsafe fn Xray_SetupStage5(game: &mut Game) {
    let mut v1: u32 = 0;
    if CanXrayShowBlocks(game) {
        if ram_word!(game, earthquake_timer) != 0 {
            set_ram_word!(
                game.reg_BG1HOFS =
                    (ram_word!(game, layer1_x_pos)).wrapping_add(ram_word!(game, bg1_x_offset))
            );
            set_ram_word!(
                game.reg_BG1VOFS =
                    (ram_word!(game, layer1_y_pos)).wrapping_add(ram_word!(game, bg1_y_offset))
            );
        }
        set_ram_word!(game.reg_BG2HOFS = (ram_word!(game, reg_BG1HOFS)) & (0xfu16));
        set_ram_word!(game.reg_BG2VOFS = (ram_word!(game, reg_BG1VOFS)) & (0xfu16));
        *(&raw mut game.g_ram as *mut uint8).offset(0x59) = 73 as uint8;
    }
    let v0: uint16 = ram_word!(game, vram_read_queue_tail);
    v1 = ram_word!(game, vram_read_queue_tail) as u32;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340) as *mut VramReadQueueEnt)
        .offset(0))
    .vram_target as *mut uint8)
        .offset(ram_word!(game, vram_read_queue_tail) as i32 as isize) as *mut uint16) =
        (((*(&raw mut game.g_ram as *mut uint8).offset(0x59) as i32 & 0xfc as i32) << 8 as i32)
            + 1024 as i32) as uint16;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340) as *mut VramReadQueueEnt)
        .offset(0))
    .dma_parameters as *mut uint8)
        .offset(v1 as isize)
        .offset(1) as *mut uint16) = 57 as uint16;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340) as *mut VramReadQueueEnt)
        .offset(0))
    .src
    .addr as *mut uint8)
        .offset(v0 as i32 as isize) as *mut VoidP) = 0x5800 as VoidP;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340) as *mut VramReadQueueEnt)
        .offset(0))
    .src
    .bank)
        .offset(v0 as i32 as isize) as *mut uint16) = 126 as uint16;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340) as *mut VramReadQueueEnt)
        .offset(0))
    .size as *mut uint8)
        .offset(v0 as i32 as isize) as *mut uint16) = 2048 as uint16;
    set_ram_word!(game.vram_read_queue_tail = (v0).wrapping_add(9u16));
}
pub unsafe fn CanXrayShowBlocks(game: &mut Game) -> bool {
    if ram_word!(game, room_ptr) as i32 == addr_kRoom_a66a as i32
        || ram_word!(game, room_ptr) as i32 == addr_kRoom_cefb as i32
    {
        return false_0 != 0;
    }
    if ram_word!(game, fx_type) == 0x24u16 {
        return false_0 != 0;
    }
    if ram_word!(game, boss_id) == 3u16
        || ram_word!(game, boss_id) == 6u16
        || ram_word!(game, boss_id) == 7u16
        || ram_word!(game, boss_id) == 8u16
        || ram_word!(game, boss_id) == 10u16
    {
        return false_0 != 0;
    }
    true_0 != 0
}
pub unsafe fn Xray_SetupStage6(game: &mut Game) {
    let mut v1: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    if CanXrayShowBlocks(game) {
        let v0: uint16 = ram_word!(game, vram_write_queue_tail);
        v1 = gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game);
        (*v1).size = 2048 as uint16;
        (*v1).src.addr = ((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000
            as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8)
            as ::core::ffi::c_long as VoidP;
        *(&raw mut (*v1).src.bank as *mut uint16) = 126 as uint16;
        (*v1).vram_dst = ((*(&raw mut game.g_ram as *mut uint8).offset(0x59) as i32 & 0xfc as i32)
            << 8 as i32) as uint16;
        set_ram_word!(game.vram_write_queue_tail = (v0).wrapping_add(7u16));
    }
}
pub unsafe fn Xray_SetupStage7(game: &mut Game) {
    let mut v1: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    if CanXrayShowBlocks(game) {
        let v0: uint16 = ram_word!(game, vram_write_queue_tail);
        v1 = gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game);
        (*v1).size = 2048 as uint16;
        (*v1).src.addr = 0x4800 as VoidP;
        *(&raw mut (*v1).src.bank as *mut uint16) = 126 as uint16;
        (*v1).vram_dst =
            (((*(&raw mut game.g_ram as *mut uint8).offset(0x59) as i32 & 0xfc as i32) << 8 as i32)
                + 1024 as i32) as uint16;
        set_ram_word!(game.vram_write_queue_tail = (v0).wrapping_add(7u16));
    }
    mov24(
        (&raw mut game.g_ram as *mut uint8).offset(0xa88) as *mut LongPtr,
        0x9800e4 as uint32,
    );
    mov24(
        (&raw mut game.g_ram as *mut uint8).offset(0xa8b) as *mut LongPtr,
        0x98c8e4 as uint32,
    );
    mov24(
        (&raw mut game.g_ram as *mut uint8).offset(0xa8e) as *mut LongPtr,
        0x999098 as uint32,
    );
    set_ram_word!(game.hdma_var_1 = 0 as uint16);
    set_ram_word!(game.demo_input_pre_instr = 0 as uint16);
    set_ram_word!(game.demo_input_instr_timer = 0 as uint16);
    set_ram_word!(game.demo_input_instr_ptr = 0 as uint16);
    set_ram_word!(game.demo_input = 0 as uint16);
    set_ram_word!(game.demo_input_new = 0 as uint16);
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8 {
        set_ram_word!(game.xray_angle = 192 as uint16);
    } else {
        set_ram_word!(game.xray_angle = 64 as uint16);
    };
}
pub unsafe fn HdmaobjPreInstr_XraySetup(k: uint16, game: &mut Game) {
    let current_block: u64;
    let mut v1: uint16 = 0;
    v1 = 4096 as uint16;
    if ram_word!(game, fx_type) == 36u16 {
        if (*(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) as i32 & 0x1f as i32)
            - 7 as i32
            & 0x8000 as i32
            == 0
        {
            current_block = 5035836855854403712;
        } else {
            current_block = 4263953485050074566;
        }
    } else {
        v1 = 0x2000 as uint16;
        if CanXrayShowBlocks(game) {
            v1 = 0x4000 as uint16;
            current_block = 4263953485050074566;
        } else {
            current_block = 5035836855854403712;
        }
    }
    if current_block == 4263953485050074566 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) as *mut uint8
            as *mut uint16) = 0x27 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) as *mut uint8
            as *mut uint16) = 0x47 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) as *mut uint8
            as *mut uint16) = 0x87 as uint16;
    }
    update_ram_word!(game.fx_layer_blending_config_c, |fresh37| (fresh37) | (v1));
}
pub unsafe fn Xray_SetupStage8_SetBackdropColor(game: &mut Game) {
    set_ram_word!(game.palette_buffer[0] = 3171 as uint16);
}
pub unsafe fn GameState_28_Unused_(game: &mut Game) {
    Unreachable(game);
}
pub unsafe fn VariaSuitPickup(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xdf0) = 48 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xdf1) = 80 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xdf2) = 0x80 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xdf3) = 0 as uint8;
    Samus_CancelSpeedBoost(game);
    SetHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0xb42) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0xb44) as *mut uint16,
        0 as uint32,
    );
    set_ram_word!(game.samus_x_base_speed = 0 as uint16);
    set_ram_word!(game.samus_x_base_subspeed = 0 as uint16);
    set_ram_word!(game.samus_y_subspeed = 0 as uint16);
    set_ram_word!(game.samus_y_speed = 0 as uint16);
    set_ram_word!(game.samus_y_dir = 0 as uint16);
    set_ram_word!(game.used_for_ball_bounce_on_landing = 0 as uint16);
    set_ram_word!(game.samus_x_accel_mode = 0 as uint16);
    set_ram_word!(game.elevator_status = 0 as uint16);
    set_ram_word!(game.substate = 0 as uint16);
    set_ram_word!(game.suit_pickup_light_beam_pos = 0 as uint16);
    set_ram_word!(game.suit_pickup_light_beam_widening_speed = 256 as uint16);
    let mut i: i32 = 510 as i32;
    while i >= 0i32 {
        set_ram_word!(game.hdma_table_1[(i >> 1i32)] = 255 as uint16);
        i -= 2 as i32;
    }
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f) == 3u8
        || *(&raw mut game.g_ram as *mut uint8).offset(0xa1f) == 20u8
    {
        QueueSfx1_Max9(0x32 as uint16, game);
    }
    if (ram_word!(game, equipped_items) as i32 & 0x20 as i32) != 0i32 {
        set_ram_word!(game.samus_pose = kPose_9B_FaceF_VariaGravitySuit as uint16);
    } else {
        set_ram_word!(game.samus_pose = kPose_00_FaceF_Powersuit as uint16);
    }
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    CallSomeSamusCode(0x15 as uint16, game);
    set_ram_word!(game.samus_x_pos = (ram_word!(game, layer1_x_pos)).wrapping_add(120u16));
    set_ram_word!(game.samus_prev_x_pos = (ram_word!(game, layer1_x_pos)).wrapping_add(120u16));
    set_ram_word!(game.samus_y_pos = (ram_word!(game, layer1_y_pos)).wrapping_add(136u16));
    set_ram_word!(game.samus_prev_y_pos = (ram_word!(game, layer1_y_pos)).wrapping_add(136u16));
    QueueSfx2_Max6(0x56 as uint16, game);
    static unk_91D59B: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0x41 as uint8, 0x26 as uint8, 0xd5a2 as VoidP);
    SpawnHdmaObject(
        0x91 as uint8,
        &raw const unk_91D59B as *const ::core::ffi::c_void,
        game,
    );
}
pub unsafe fn GravitySuitPickup(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xdf0) = 48 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xdf1) = 73 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xdf2) = -(112 as i32) as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xdf3) = 1 as uint8;
    Samus_CancelSpeedBoost(game);
    SetHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0xb42) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0xb44) as *mut uint16,
        0 as uint32,
    );
    set_ram_word!(game.samus_x_base_speed = 0 as uint16);
    set_ram_word!(game.samus_x_base_subspeed = 0 as uint16);
    set_ram_word!(game.samus_y_subspeed = 0 as uint16);
    set_ram_word!(game.samus_y_speed = 0 as uint16);
    set_ram_word!(game.samus_y_dir = 0 as uint16);
    set_ram_word!(game.used_for_ball_bounce_on_landing = 0 as uint16);
    set_ram_word!(game.samus_x_accel_mode = 0 as uint16);
    set_ram_word!(game.elevator_status = 0 as uint16);
    set_ram_word!(game.substate = 0 as uint16);
    set_ram_word!(game.suit_pickup_light_beam_pos = 0 as uint16);
    set_ram_word!(game.suit_pickup_light_beam_widening_speed = 256 as uint16);
    let mut i: i32 = 510 as i32;
    while i >= 0i32 {
        set_ram_word!(game.hdma_table_1[(i >> 1i32)] = 255 as uint16);
        i -= 2 as i32;
    }
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as i32
        == kMovementType_03_SpinJumping as i32
        || *(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as i32
            == kMovementType_14_WallJumping as i32
    {
        QueueSfx1_Max9(0x32 as uint16, game);
    }
    if (ram_word!(game, equipped_items) as i32 & 1 as i32) != 0i32 {
        set_ram_word!(game.samus_pose = kPose_9B_FaceF_VariaGravitySuit as uint16);
    } else {
        set_ram_word!(game.samus_pose = kPose_00_FaceF_Powersuit as uint16);
    }
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    CallSomeSamusCode(0x15 as uint16, game);
    set_ram_word!(game.samus_x_pos = (ram_word!(game, layer1_x_pos)).wrapping_add(120u16));
    set_ram_word!(game.samus_prev_x_pos = (ram_word!(game, layer1_x_pos)).wrapping_add(120u16));
    set_ram_word!(game.samus_y_pos = (ram_word!(game, layer1_y_pos)).wrapping_add(136u16));
    set_ram_word!(game.samus_prev_y_pos = (ram_word!(game, layer1_y_pos)).wrapping_add(136u16));
    QueueSfx2_Max6(0x56 as uint16, game);
    static unk_91D673: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0x41 as uint8, 0x26 as uint8, 0xd67a as VoidP);
    SpawnHdmaObject(
        0x91 as uint8,
        &raw const unk_91D673 as *const ::core::ffi::c_void,
        game,
    );
}
pub unsafe fn InitializeSuitPickupHdma(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 19 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 19 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 4 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6d) = 4 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6e) = 16 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x60) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x61) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x62) = 32 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x71) = 51 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) =
        *(&raw mut game.g_ram as *mut uint8).offset(0xdf0);
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) =
        *(&raw mut game.g_ram as *mut uint8).offset(0xdf1);
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) =
        *(&raw mut game.g_ram as *mut uint8).offset(0xdf2);
    mov24(
        (&raw mut game.g_ram as *mut uint8).offset(0xa88) as *mut LongPtr,
        0x9800e4 as uint32,
    );
    mov24(
        (&raw mut game.g_ram as *mut uint8).offset(0xa8b) as *mut LongPtr,
        0x98c8e4 as uint32,
    );
    mov24(
        (&raw mut game.g_ram as *mut uint8).offset(0xa8e) as *mut LongPtr,
        0x999098 as uint32,
    );
    set_ram_word!(game.hdma_var_1 = 0 as uint16);
}
static off_91D72D: [Option<Func_U8>; 11] = unsafe {
    [
        Some(Samus_HandleScrewAttackSpeedBoostingPals as unsafe fn(&mut Game) -> uint8),
        Some(Samus_SpeedBoosterShinePals as unsafe fn(&mut Game) -> uint8),
        ::core::mem::transmute::<Option<unsafe fn(&mut Game) -> ()>, Option<Func_U8>>(Some(
            HandleMiscSamusPalette as unsafe fn(&mut Game) -> (),
        )),
        None,
        None,
        None,
        Some(Samus_HandleShinesparkingPals as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleCrystalFlashPals as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleXrayPals as unsafe fn(&mut Game) -> uint8),
        Some(HandleVisorPalette as unsafe fn(&mut Game) -> uint8),
        Some(nullsub_164 as unsafe fn(&mut Game) -> uint8),
    ]
};
pub unsafe fn Samus_HandlePalette(game: &mut Game) {
    if (ram_word!(game, samus_special_super_palette_flags) as i32 & 0x8000 as i32) == 0i32
        && (HandleBeamChargePalettes(game) as i32 & 1 as i32 != 0
            || off_91D72D[ram_word!(game, timer_for_shine_timer) as usize]
                .expect("non-null function pointer")(game) as i32
                & 1 as i32
                == 0)
    {
        CopyToSamusSuitPalette(
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x91d727)) as *const uint8 as *mut uint16)
                .offset((ram_word!(game, samus_suit_palette_index) as i32 >> 1 as i32) as isize),
            game,
        );
    }
    HandleMiscSamusPalette(game);
}
pub unsafe fn HandleBeamChargePalettes(game: &mut Game) -> uint8 {
    if ram_word!(game, charged_shot_glow_timer) != 0 {
        if ram_word!(game, hyper_beam_flag) != 0 {
            if (ram_word!(game, charged_shot_glow_timer) as i32 & 1 as i32) == 0i32 {
                if (ram_word!(game, charged_shot_glow_timer) as i32 & 0x1e as i32) == 0i32 {
                    set_ram_word!(game.charged_shot_glow_timer = 0 as uint16);
                    return 1 as uint8;
                }
                CopyToSamusSuitPalette(
                    *(game.g_rom.offset(crate::rtl::lorom_offset(0x91d829)) as *const uint8
                        as *mut uint16)
                        .offset(
                            ((ram_word!(game, charged_shot_glow_timer) as i32 & 0x1e as i32)
                                >> 1 as i32) as isize,
                        ),
                    game,
                );
            }
            update_ram_word!(game.charged_shot_glow_timer, wrapping_sub(1));
            return 0 as uint8;
        }
        if update_ram_word!(game.charged_shot_glow_timer, wrapping_sub(1), reread) != 0 {
            let mut i: i32 = 28 as i32;
            while i >= 0i32 {
                *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                    .offset(((i >> 1 as i32) + 193 as i32) as isize) = 1023 as uint16;
                i -= 2 as i32;
            }
            0 as uint8
        } else {
            1 as uint8
        }
    } else if ram_word!(game, grapple_beam_function) as i32 == 0x9bc4f0 as i32 & 0xffff as i32
        && ram_word!(game, flare_counter) != 0u16
        && ram_word!(game, flare_counter) as i32 - 60 as i32 & 0x8000 as i32 == 0
    {
        let mut R36: uint16 = 0;
        if ram_word!(game, samus_contact_damage_index) == 4u16 {
            R36 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x91d7ff)) as *const uint8
                as *mut uint16)
                .offset((ram_word!(game, samus_suit_palette_index) as i32 >> 1 as i32) as isize);
        } else {
            R36 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x91d7d5)) as *const uint8
                as *mut uint16)
                .offset((ram_word!(game, samus_suit_palette_index) as i32 >> 1 as i32) as isize);
        }
        CopyToSamusSuitPalette(
            *(RomPtr_91(
                (R36).wrapping_add(ram_word!(game, samus_charge_palette_index)),
                game,
            ) as *mut uint16),
            game,
        );
        let mut v1: uint16 = (ram_word!(game, samus_charge_palette_index)).wrapping_add(2u16);
        if ram_word!(game, samus_charge_palette_index) as i32 - 10 as i32 & 0x8000 as i32 == 0 {
            v1 = 0 as uint16;
        }
        set_ram_word!(game.samus_charge_palette_index = v1);
        0 as uint8
    } else {
        set_ram_word!(game.samus_charge_palette_index = 0 as uint16);
        HandleVisorPalette(game)
    }
}
pub unsafe fn HandleVisorPalette(game: &mut Game) -> uint8 {
    if ram_word!(game, timer_for_shine_timer) == 8u16 {
        return 0 as uint8;
    }
    if ram_word!(game, fx_layer_blending_config_a) == 40u16
        || ram_word!(game, fx_layer_blending_config_a) == 42u16
    {
        let mut v1: uint16 = (ram_word!(game, samus_visor_palette_timer_index)).wrapping_sub(1u16);
        set_ram_word!(game.samus_visor_palette_timer_index = v1);
        if v1 as uint8 != 0 {
            return 0 as uint8;
        }
        set_ram_word!(game.samus_visor_palette_timer_index = (v1) | (5u16));
        set_ram_word!(
            game.palette_buffer[196] = *(game.g_rom.offset(crate::rtl::lorom_offset(0x9ba3c0))
                as *const uint8 as *mut uint16)
                .offset(
                    (*(&raw mut v1 as *mut uint8).offset(
                        (::core::mem::size_of::<uint16>())
                            .wrapping_div(::core::mem::size_of::<uint8>())
                            .wrapping_sub(1_usize) as isize,
                    ) as i32
                        >> 1 as i32) as isize,
                )
        );
        let v2: i32 = *(&raw mut v1 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        ) as i32
            + 2 as i32;
        if v2 - 12 as i32 & 0x8000 as i32 != 0 {
            set_ram_word!(
                game.samus_visor_palette_timer_index = (u16::swap_bytes(v2 as uint16))
                    | ((ram_word!(game, samus_visor_palette_timer_index) as uint8) as u16)
            );
        } else {
            set_ram_word!(
                game.samus_visor_palette_timer_index =
                    ((ram_word!(game, samus_visor_palette_timer_index) as uint8) as u16)
                        | (0x600u16)
            );
        }
        0 as uint8
    } else {
        set_ram_word!(game.samus_visor_palette_timer_index = 1537 as uint16);
        0 as uint8
    }
}
pub unsafe fn HandleMiscSamusPalette(game: &mut Game) {
    let mut current_block: u64;
    if ram_word!(game, samus_special_super_palette_flags) == 0 {
        let mut v0: uint16 = ram_word!(game, samus_hurt_flash_counter);
        if ram_word!(game, samus_hurt_flash_counter) == 0 {
            return;
        }
        if ram_word!(game, samus_hurt_flash_counter) == 2u16 {
            if ram_word!(game, cinematic_function) == 0
                && (ram_word!(game, frame_handler_beta) as i32 != 0x90e8d9 as i32 & 0xffff as i32
                    || ram_word!(game, samus_pose) as i32 != kPose_54_FaceL_Knockback as i32)
            {
                QueueSfx1_Max6(0x35 as uint16, game);
                current_block = 4767789648742619593;
            } else {
                v0 = ram_word!(game, samus_hurt_flash_counter);
                current_block = 15619007995458559411;
            }
        } else {
            current_block = 15619007995458559411;
        }
        if current_block == 15619007995458559411 {
            if v0 as i32 - 7 as i32 & 0x8000 as i32 == 0 {
                current_block = 11050875288958768710;
            } else if (v0 as i32 & 1 as i32) != 0i32 {
                CopyToSamusSuitPalette(addr_word_9BA380 as uint16, game);
                current_block = 11050875288958768710;
            } else {
                current_block = 4767789648742619593;
            }
        }
        if current_block == 4767789648742619593 {
            if ram_word!(game, cinematic_function) != 0 {
                CopyToSamusSuitPalette(addr_word_9BA3A0 as uint16, game);
            } else {
                Samus_LoadSuitPalette(game);
            }
        }
        let v1: uint16 = (ram_word!(game, samus_hurt_flash_counter)).wrapping_add(1u16);
        set_ram_word!(game.samus_hurt_flash_counter = v1);
        if v1 == 40u16 {
            if ram_word!(game, grapple_beam_function) as i32 == 0x9bc4f0 as i32 & 0xffff as i32 {
                if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as i32
                    == kMovementType_03_SpinJumping as i32
                    || *(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as i32
                        == kMovementType_14_WallJumping as i32
                {
                    CallSomeSamusCode(0x1c as uint16, game);
                } else if ram_word!(game, flare_counter) as i32 - 16 as i32 & 0x8000 as i32 == 0
                    && (ram_word!(game, button_config_shoot_x) as i32
                        & ram_word!(game, joypad1_lastkeys) as i32)
                        != 0i32
                {
                    set_ram_word!(game.play_resume_charging_beam_sfx = 1 as uint16);
                }
            } else if ram_word!(game, grapple_beam_function) as i32 + 0x37aa as i32 & 0x8000 as i32
                != 0
            {
                QueueSfx1_Max9(6 as uint16, game);
            }
        } else if v1 as i32 - 60 as i32 & 0x8000 as i32 == 0 {
            set_ram_word!(game.samus_hurt_flash_counter = 0 as uint16);
        }
    } else {
        if (ram_word!(game, samus_special_super_palette_flags) as i32 & 0x8000 as i32) != 0i32 {
            CopyToSamusSuitPalette(
                *(game.g_rom.offset(crate::rtl::lorom_offset(0x91d99e)) as *const uint8
                    as *mut uint16)
                    .offset(ram_word!(game, samus_charge_palette_index) as isize),
                game,
            );
            let v2: bool =
                (update_ram_word!(game.special_samus_palette_timer, wrapping_sub(1), reread)
                    as i32
                    & 0x8000 as i32)
                    != 0i32;
            if ram_word!(game, special_samus_palette_timer) == 0 || v2 {
                set_ram_word!(
                    game.special_samus_palette_timer = ram_word!(game, special_samus_palette_frame)
                );
                if update_ram_word!(game.samus_charge_palette_index, wrapping_add(1), reread) as i32
                    - 10 as i32
                    & 0x8000 as i32
                    == 0
                {
                    set_ram_word!(game.samus_charge_palette_index = 0 as uint16);
                }
            }
        } else {
            if (ram_word!(game, samus_special_super_palette_flags) as i32 & 1 as i32) != 0i32 {
                CopyToSamusSuitPalette(
                    *(game.g_rom.offset(crate::rtl::lorom_offset(0x91d998)) as *const uint8
                        as *mut uint16)
                        .offset(
                            (ram_word!(game, samus_suit_palette_index) as i32 >> 1 as i32) as isize,
                        ),
                    game,
                );
            } else {
                Samus_LoadSuitPalette(game);
            }
            update_ram_word!(game.samus_special_super_palette_flags, wrapping_add(1));
        }
    }
}
pub unsafe fn Samus_HandleScrewAttackSpeedBoostingPals(game: &mut Game) -> uint8 {
    let current_block: u64;
    if (ram_word!(game, samus_suit_palette_index) as i32 & 4 as i32) == 0i32 {
        let r20: uint16 = Samus_GetTop_R20(game);
        if liquid_at_height(game, r20) != LiquidAtHeight::None {
            return 1 as uint8;
        }
    }
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as i32
        == kMovementType_03_SpinJumping as i32
    {
        if (ram_word!(game, equipped_items) as i32 & 8 as i32) == 0i32 {
            current_block = 14068947459426067103;
        } else if ram_word!(game, samus_anim_frame) != 0 {
            if ram_word!(game, samus_anim_frame) as i32 - 27 as i32 & 0x8000 as i32 == 0 {
                return 0 as uint8;
            }
            current_block = 8831408221741692167;
        } else {
            current_block = 7431622773599773796;
        }
    } else if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as i32
        == kMovementType_14_WallJumping as i32
    {
        if (ram_word!(game, equipped_items) as i32 & 8 as i32) == 0i32 {
            return 1 as uint8;
        }
        if ram_word!(game, samus_anim_frame) as i32 - 3 as i32 & 0x8000 as i32 == 0 {
            current_block = 8831408221741692167;
        } else {
            current_block = 7431622773599773796;
        }
    } else {
        current_block = 14068947459426067103;
    }
    match current_block {
        8831408221741692167 => {
            let R36: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x91da4a))
                as *const uint8 as *mut uint16)
                .offset((ram_word!(game, samus_suit_palette_index) as i32 >> 1 as i32) as isize);
            let v1: uint16 = *(RomPtr_91(
                (R36).wrapping_add(ram_word!(game, special_samus_palette_frame)),
                game,
            ) as *mut uint16);
            CopyToSamusSuitPalette(v1, game);
            let mut v2: uint16 = (ram_word!(game, special_samus_palette_frame)).wrapping_add(2u16);
            if ram_word!(game, special_samus_palette_frame) >= 10u16 {
                v2 = 0 as uint16;
            }
            set_ram_word!(game.special_samus_palette_frame = v2);
            1 as uint8
        }
        7431622773599773796 => {
            set_ram_word!(game.special_samus_palette_frame = 0 as uint16);
            1 as uint8
        }
        _ => {
            if (ram_word!(game, speed_boost_counter) as i32 & 0xff00 as i32) != 1024i32 {
                return 1 as uint8;
            }
            let v3: bool =
                (update_ram_word!(game.special_samus_palette_timer, wrapping_sub(1), reread)
                    as i32
                    & 0x8000 as i32)
                    != 0i32;
            if ram_word!(game, special_samus_palette_timer) == 0 || v3 {
                set_ram_word!(game.special_samus_palette_timer = 4 as uint16);
                let R36_0: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x91daa9))
                    as *const uint8 as *mut uint16)
                    .offset(
                        (ram_word!(game, samus_suit_palette_index) as i32 >> 1 as i32) as isize,
                    );
                if ram_word!(game, special_samus_palette_frame) > 6u16 {
                    set_ram_word!(game.special_samus_palette_frame = 6 as uint16);
                }
                let v4: uint16 = *(RomPtr_91(
                    (R36_0).wrapping_add(ram_word!(game, special_samus_palette_frame)),
                    game,
                ) as *mut uint16);
                CopyToSamusSuitPalette(v4, game);
                let mut v5: uint16 =
                    (ram_word!(game, special_samus_palette_frame)).wrapping_add(2u16);
                if ram_word!(game, special_samus_palette_frame) >= 6u16 {
                    v5 = 6 as uint16;
                }
                set_ram_word!(game.special_samus_palette_frame = v5);
            }
            1 as uint8
        }
    }
}
pub unsafe fn Samus_SpeedBoosterShinePals(game: &mut Game) -> uint8 {
    let mut v0: uint16 = ram_word!(game, samus_shine_timer);
    if ram_word!(game, samus_shine_timer) == 170u16 {
        let v4: uint16 = ram_word!(game, samus_shine_timer);
        QueueSfx3_Max9(0xc as uint16, game);
        v0 = v4;
    }
    set_ram_word!(game.samus_shine_timer = (v0).wrapping_sub(1u16));
    if ((v0) as i16).wrapping_sub(1i16) <= 0i16 {
        set_ram_word!(game.special_samus_palette_frame = 0 as uint16);
        set_ram_word!(game.timer_for_shine_timer = 0 as uint16);
        0 as uint8
    } else {
        let R36: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x91db10)) as *const uint8
            as *mut uint16)
            .offset((ram_word!(game, samus_suit_palette_index) as i32 >> 1 as i32) as isize);
        let v1: uint16 = *(RomPtr_91(
            (R36).wrapping_add(ram_word!(game, special_samus_palette_frame)),
            game,
        ) as *mut uint16);
        CopyToSamusSuitPalette(v1, game);
        let mut v2: uint16 = (ram_word!(game, special_samus_palette_frame)).wrapping_add(2u16);
        if ram_word!(game, special_samus_palette_frame) as i32 - 10 as i32 & 0x8000 as i32 == 0 {
            v2 = 0 as uint16;
        }
        set_ram_word!(game.special_samus_palette_frame = v2);
        1 as uint8
    }
}
pub unsafe fn Samus_HandleShinesparkingPals(game: &mut Game) -> uint8 {
    let v0: bool = (update_ram_word!(game.samus_shine_timer, wrapping_sub(1), reread) as i32
        & 0x8000 as i32)
        != 0i32;
    if ram_word!(game, samus_shine_timer) == 0 || v0 {
        set_ram_word!(game.timer_for_shine_timer = 0 as uint16);
        set_ram_word!(game.special_samus_palette_frame = 0 as uint16);
        0 as uint8
    } else {
        let R36: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x91db75)) as *const uint8
            as *mut uint16)
            .offset((ram_word!(game, samus_suit_palette_index) as i32 >> 1 as i32) as isize);
        let v1: uint16 = *(RomPtr_91(
            (R36).wrapping_add(ram_word!(game, special_samus_palette_frame)),
            game,
        ) as *mut uint16);
        CopyToSamusSuitPalette(v1, game);
        let mut v2: uint16 = (ram_word!(game, special_samus_palette_frame)).wrapping_add(2u16);
        if ram_word!(game, special_samus_palette_frame) as i32 - 6 as i32 & 0x8000 as i32 == 0 {
            v2 = 0 as uint16;
        }
        set_ram_word!(game.special_samus_palette_frame = v2);
        1 as uint8
    }
}
pub unsafe fn Samus_HandleCrystalFlashPals(game: &mut Game) -> uint8 {
    if (ram_word!(game, samus_shine_timer) as i32 & 0x8000 as i32) != 0i32 {
        WriteBeamPalette_A(ram_word!(game, equipped_beams), game);
        set_ram_word!(game.timer_for_shine_timer = 0 as uint16);
        set_ram_word!(game.special_samus_palette_frame = 0 as uint16);
        set_ram_word!(game.special_samus_palette_timer = 0 as uint16);
        set_ram_word!(game.samus_shine_timer = 0 as uint16);
        0 as uint8
    } else {
        if (update_ram_word!(game.samus_shine_timer, wrapping_sub(1), reread) as int16) <= 0i16 {
            set_ram_word!(game.samus_shine_timer = 5 as uint16);
            Samus_Copy6PalColors(
                *(game.g_rom.offset(crate::rtl::lorom_offset(0x91dc28)) as *const uint8
                    as *mut uint16)
                    .offset(
                        (ram_word!(game, special_samus_palette_frame) as i32 >> 1 as i32) as isize,
                    ),
                game,
            );
            let mut v0: uint16 = (ram_word!(game, special_samus_palette_frame)).wrapping_add(2u16);
            if ram_word!(game, special_samus_palette_frame) as i32 - 10 as i32 & 0x8000 as i32 == 0
            {
                v0 = 0 as uint16;
            }
            set_ram_word!(game.special_samus_palette_frame = v0);
        }
        let ref mut fresh15 = *((&raw mut game.g_ram as *mut uint8).offset(0xdf2) as *mut uint16);
        *fresh15 = (*fresh15).wrapping_sub(1);
        let v1: bool = (*fresh15 as int16) < 0i16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0xdf2) as *mut uint16) == 0 || v1 {
            *((&raw mut game.g_ram as *mut uint8).offset(0xdf2) as *mut uint16) =
                *((&raw mut (*(game.g_rom.offset(
                    crate::rtl::lorom_offset(0x91dc00),
                ) as *const uint8 as *mut SamusCrystalFlashPalTable)
                    .offset(0))
                .timer as *mut uint8)
                    .offset(
                        ram_word!(game, special_samus_palette_timer)
                            as i32 as isize,
                    ) as *mut uint16);
            Samus_Copy10PalColors(
                *((&raw mut (*(game.g_rom.offset(
                    crate::rtl::lorom_offset(0x91dc00),
                ) as *const uint8 as *mut SamusCrystalFlashPalTable)
                    .offset(0))
                .ptr as *mut uint8)
                    .offset(
                        ram_word!(game, special_samus_palette_timer)
                            as i32 as isize,
                    ) as *mut VoidP),
                game,
            );
            let mut v2: uint16 = (ram_word!(game, special_samus_palette_timer)).wrapping_add(4u16);
            if ram_word!(game, special_samus_palette_timer) as i32 - 36 as i32 & 0x8000 as i32 == 0
            {
                v2 = 0 as uint16;
            }
            set_ram_word!(game.special_samus_palette_timer = v2);
        }
        1 as uint8
    }
}
pub unsafe fn Samus_Copy10PalColors(v0: uint16, game: &mut Game) {
    memcpy(
        ((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(224)
            as *mut uint16 as *mut ::core::ffi::c_void,
        RomPtr_9B(v0, game) as *const ::core::ffi::c_void,
        20 as size_t,
    );
}
pub unsafe fn Samus_Copy6PalColors(j: uint16, game: &mut Game) {
    memcpy(
        ((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(234)
            as *mut uint16 as *mut ::core::ffi::c_void,
        RomPtr_9B(j, game) as *const ::core::ffi::c_void,
        12 as size_t,
    );
}
pub unsafe fn Samus_HandleXrayPals(game: &mut Game) -> uint8 {
    if (ram_word!(game, demo_timer_counter) as i32 & 0x8000 as i32) != 0i32 {
        set_ram_word!(game.timer_for_shine_timer = 0 as uint16);
        set_ram_word!(game.special_samus_palette_frame = 0 as uint16);
        set_ram_word!(game.special_samus_palette_timer = 0 as uint16);
        set_ram_word!(game.demo_timer_counter = 0 as uint16);
        0 as uint8
    } else {
        if ram_word!(game, demo_timer_counter) == 0 {
            if ram_word!(game, demo_input_pre_instr) as i32 - 2 as i32 & 0x8000 as i32 != 0 {
                let v0: bool =
                    (update_ram_word!(game.special_samus_palette_timer, wrapping_sub(1), reread)
                        as i32
                        & 0x8000 as i32)
                        != 0i32;
                if ram_word!(game, special_samus_palette_timer) == 0 || v0 {
                    set_ram_word!(game.special_samus_palette_timer = 5 as uint16);
                    set_ram_word!(
                        game.palette_buffer[196] =
                            *(game.g_rom.offset(crate::rtl::lorom_offset(0x9ba3c0)) as *const uint8
                                as *mut uint16)
                                .offset(
                                    (ram_word!(game, special_samus_palette_frame) as i32
                                        >> 1 as i32) as isize,
                                )
                    );
                    if ram_word!(game, special_samus_palette_frame) as i32 - 4 as i32
                        & 0x8000 as i32
                        != 0
                    {
                        update_ram_word!(game.special_samus_palette_frame, wrapping_add(2u16));
                    }
                }
                return 1 as uint8;
            }
            set_ram_word!(game.special_samus_palette_frame = 6 as uint16);
            set_ram_word!(game.special_samus_palette_timer = 1 as uint16);
            set_ram_word!(game.demo_timer_counter = 1 as uint16);
        }
        let v2: bool = (update_ram_word!(game.special_samus_palette_timer, wrapping_sub(1), reread)
            as i32
            & 0x8000 as i32)
            != 0i32;
        if ram_word!(game, special_samus_palette_timer) != 0u16 && !v2 {
            return 1 as uint8;
        }
        set_ram_word!(game.special_samus_palette_timer = 5 as uint16);
        set_ram_word!(
            game.palette_buffer[196] =
                *(game.g_rom.offset(crate::rtl::lorom_offset(0x9ba3c0)) as *const uint8
                    as *mut uint16)
                    .offset(
                        (ram_word!(game, special_samus_palette_frame) as i32 >> 1 as i32) as isize,
                    )
        );
        let mut v3: uint16 = (ram_word!(game, special_samus_palette_frame)).wrapping_add(2u16);
        if ram_word!(game, special_samus_palette_frame) as i32 - 10 as i32 & 0x8000 as i32 == 0 {
            v3 = 6 as uint16;
        }
        set_ram_word!(game.special_samus_palette_frame = v3);
        1 as uint8
    }
}
pub fn nullsub_164(game: &mut Game) -> uint8 {
    0 as uint8
}
pub unsafe fn CopyToSamusSuitPalette(k: uint16, game: &mut Game) {
    memcpy(
        ((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(192)
            as *mut uint16 as *mut ::core::ffi::c_void,
        RomPtr_9B(k, game) as *mut uint16 as *const ::core::ffi::c_void,
        32 as size_t,
    );
}
pub unsafe fn CopyToSamusSuitTargetPalette(k: uint16, game: &mut Game) {
    memcpy(
        ((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16).offset(192)
            as *mut uint16 as *mut ::core::ffi::c_void,
        RomPtr_9B(k, game) as *mut uint16 as *const ::core::ffi::c_void,
        32 as size_t,
    );
}
pub unsafe fn Samus_CancelSpeedBoost(game: &mut Game) {
    if ram_word!(game, samus_has_momentum_flag) != 0 {
        set_ram_word!(game.samus_has_momentum_flag = 0 as uint16);
        set_ram_word!(game.speed_boost_counter = 0 as uint16);
        set_ram_word!(game.special_samus_palette_frame = 0 as uint16);
        set_ram_word!(game.special_samus_palette_timer = 0 as uint16);
        if (ram_word!(game, equipped_items) as i32 & 0x20 as i32) != 0i32 {
            CopyToSamusSuitPalette(addr_kSamusPalette_GravitySuit as uint16, game);
        } else if (ram_word!(game, equipped_items) as i32 & 1 as i32) != 0i32 {
            CopyToSamusSuitPalette(addr_kSamusPalette_VariaSuit as uint16, game);
        } else {
            CopyToSamusSuitPalette(addr_kSamusPalette_PowerSuit as uint16, game);
        }
    }
    if (ram_word!(game, speed_echoes_index) as i32 & 0x8000 as i32) == 0i32 {
        set_ram_word!(game.speed_echoes_index = -(1 as i32) as uint16);
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8 {
            set_ram_word!(game.speed_echo_xspeed[0] = -(8 as i32) as uint16);
            set_ram_word!(game.speed_echo_xspeed[1] = -(8 as i32) as uint16);
        } else {
            set_ram_word!(game.speed_echo_xspeed[0] = 8 as uint16);
            set_ram_word!(game.speed_echo_xspeed[1] = 8 as uint16);
        }
    }
}
pub unsafe fn Samus_LoadSuitPalette(game: &mut Game) {
    if (ram_word!(game, equipped_items) as i32 & 0x20 as i32) != 0i32 {
        CopyToSamusSuitPalette(addr_kSamusPalette_GravitySuit as uint16, game);
    } else if (ram_word!(game, equipped_items) as i32 & 1 as i32) != 0i32 {
        CopyToSamusSuitPalette(addr_kSamusPalette_VariaSuit as uint16, game);
    } else {
        CopyToSamusSuitPalette(addr_kSamusPalette_PowerSuit as uint16, game);
    };
}
pub unsafe fn Samus_LoadSuitTargetPalette(game: &mut Game) {
    if (ram_word!(game, equipped_items) as i32 & 0x20 as i32) != 0i32 {
        CopyToSamusSuitTargetPalette(addr_kSamusPalette_GravitySuit as uint16, game);
    } else if (ram_word!(game, equipped_items) as i32 & 1 as i32) != 0i32 {
        CopyToSamusSuitTargetPalette(addr_kSamusPalette_VariaSuit as uint16, game);
    } else {
        CopyToSamusSuitTargetPalette(addr_kSamusPalette_PowerSuit as uint16, game);
    };
}
pub fn Samus_RestoreHealth(amount: u16, game: &mut Game) {
    let health = game.samus_health().wrapping_add(amount);
    game.set_samus_health(health);
    if (health.wrapping_sub(game.samus_max_health()) as i16) >= 0 {
        let mut reserve = game
            .samus_reserve_health()
            .wrapping_add(health)
            .wrapping_sub(game.samus_max_health());
        if (reserve.wrapping_sub(game.samus_max_reserve_health()) as i16) >= 0 {
            reserve = game.samus_max_reserve_health();
        }
        game.set_samus_reserve_health(reserve);
        if reserve != 0 && game.reserve_health_mode() == 0 {
            game.set_reserve_health_mode(1);
        }
        game.set_samus_health(game.samus_max_health());
    }
}
pub unsafe fn Samus_DealDamage(amount: u16, game: &mut Game) {
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
pub fn Samus_RestoreMissiles(amount: u16, game: &mut Game) {
    let missiles = game.samus_missiles().wrapping_add(amount);
    game.set_samus_missiles(missiles);
    if (missiles.wrapping_sub(game.samus_max_missiles()) as i16) >= 0 {
        if game.samus_max_missiles().wrapping_sub(99) & 0x8000 != 0 {
            let reserve = game
                .samus_reserve_missiles()
                .wrapping_add(missiles.wrapping_sub(game.samus_max_missiles()));
            game.set_samus_reserve_missiles(reserve);
            if (game
                .samus_reserve_missiles()
                .wrapping_sub(game.samus_max_missiles()) as i16)
                >= 0
            {
                game.set_samus_reserve_missiles(game.samus_max_missiles());
            }
        } else {
            let reserve = game
                .samus_reserve_missiles()
                .wrapping_add(missiles.wrapping_sub(game.samus_max_missiles()));
            game.set_samus_reserve_missiles(reserve);
            if game.samus_reserve_missiles().wrapping_sub(99) & 0x8000 == 0 {
                game.set_samus_reserve_missiles(99);
            }
        }
        game.set_samus_missiles(game.samus_max_missiles());
    }
}
pub fn Samus_RestoreSuperMissiles(amount: u16, game: &mut Game) {
    let ammo = game.samus_super_missiles().wrapping_add(amount);
    game.set_samus_super_missiles(ammo);
    if (ammo.wrapping_sub(game.samus_max_super_missiles()) as i16) >= 0
        && ammo != game.samus_max_super_missiles()
    {
        game.set_samus_super_missiles(game.samus_max_super_missiles());
    }
}
pub fn Samus_RestorePowerBombs(amount: u16, game: &mut Game) {
    let ammo = game.samus_power_bombs().wrapping_add(amount);
    game.set_samus_power_bombs(ammo);
    if (ammo.wrapping_sub(game.samus_max_power_bombs()) as i16) >= 0
        && ammo != game.samus_max_power_bombs()
    {
        game.set_samus_power_bombs(game.samus_max_power_bombs());
    }
}
pub unsafe fn Samus_Initialize(game: &mut Game) {
    let debug_invincibility = game.debug_invincibility();
    // Clear the original Samus workspace from its last byte to its first.
    for address in (0xa02..=0xe0b).rev() {
        game.g_ram[address] = 0;
    }
    if game.game_state() != kGameState_40_TransitionToDemo as u16 {
        game.set_frame_handler_alfa(fnEmptyFunction as u16);
        if game.loading_game_state() == kGameState_34_CeresGoesBoom as u16 {
            game.set_frame_handler_beta(fnSetContactDamageIndexAndUpdateMinimap as u16);
            game.set_samus_draw_handler(fnSamusDrawHandler_Default as u16);
            game.set_samus_momentum_routine_index(u16::MAX);
            game.set_samus_special_transgfx_index(0);
            game.set_samus_hurt_switch_index(0);
            Samus_LoadSuitPalette(game);
            game.set_samus_input_handler(fnSamus_InputHandler_E913 as u16);
        } else {
            game.set_frame_handler_beta(fnSamus_Func16 as u16);
            game.set_samus_draw_handler(fnSamusDrawHandler_Default as u16);
            game.set_samus_momentum_routine_index(0);
            game.set_samus_special_transgfx_index(0);
            game.set_samus_hurt_switch_index(0);
            game.set_samus_input_handler(fnSamus_InputHandler_E913 as u16);
            game.set_debug_invincibility(debug_invincibility);
        }
    }
    game.set_samus_new_pose(u16::MAX);
    game.set_samus_new_pose_interrupted(u16::MAX);
    game.set_samus_new_pose_transitional(u16::MAX);
    if game.area_index() == 6 {
        game.set_frame_handler_gamma(fnSamus_Func3 as u16);
    } else {
        game.set_frame_handler_gamma(fnnullsub_152 as u16);
    }
    game.set_samus_movement_handler(fnSamus_MovementHandler_Normal as u16);
    game.set_unused_word_7e0a5e(0u16.wrapping_sub(2764));
    game.set_samus_prev_health_for_flash(50);
    game.set_samus_visor_palette_timer_index(1537);
    for slot in 0..10 {
        game.set_projectile_bomb_pre_instruction(slot, fnProjPreInstr_Empty as u16);
    }
    game.set_grapple_beam_function(fnGrappleBeamFunc_Inactive as u16);
    game.set_enable_horiz_slope_coll(3);
    game.set_samus_hurt_flash_counter(0);
    game.set_samus_special_super_palette_flags(0);
    game.set_absolute_moved_last_frame_x_fract(0);
    game.set_absolute_moved_last_frame_x(1);
    game.set_absolute_moved_last_frame_y_fract(0);
    game.set_absolute_moved_last_frame_y(1);
    // Initialize the hdma_table_1 from its last word to its first.
    for word in game.g_ram[0x9800..0x9a00].chunks_exact_mut(2).rev() {
        word.copy_from_slice(&255u16.to_le_bytes());
    }
    game.set_samus_y_subaccel(0x1c00);
    game.set_samus_y_accel(0);
    game.set_fx_y_pos(u16::MAX);
    game.set_lava_acid_y_pos(u16::MAX);
    UpdateBeamTilesAndPalette(game);
    game.set_cinematic_function(0);
    game.set_samus_pose(kPose_00_FaceF_Powersuit as u16);
    game.set_samus_pose_direction_and_movement_type(0, 0);
    game.set_samus_prev_pose(0);
    game.set_samus_prev_pose_direction_and_movement_type(0, 0);
    game.set_samus_last_different_pose(0);
    game.set_samus_last_different_pose_direction_and_movement_type(0, 0);
    game.set_enemy_index_to_shake(u16::MAX);
    game.set_hud_item_index(0);
    game.set_samus_auto_cancel_hud_item_index(0);
    game.set_samus_invincibility_timer(0);
    game.set_samus_knockback_timer(0);
    game.set_samus_hurt_flash_counter(0);
    game.set_debug_invincibility(0);
    if game.game_state() == kGameState_40_TransitionToDemo as u16 {
        LoadDemoData(game);
    }
    game.set_samus_prev_health_for_flash(game.samus_health());
}
pub unsafe fn Xray_Initialize(game: &mut Game) -> uint8 {
    static byte_91E291: [uint8; 28] = [
        1, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
    ];
    if ram_word!(game, cooldown_timer) == 7u16
        && ram_word!(game, bomb_counter) == 5u16
        && ram_word!(game, samus_x_speed_divisor) == 2u16
        || ram_word!(game, samus_pose) as i32 - kPose_A4_FaceR_LandJump as i32 & 0x8000 as i32 == 0
            && (ram_word!(game, samus_pose) as i32 - kPose_A8_FaceR_Grappling as i32
                & 0x8000 as i32
                != 0
                || ram_word!(game, samus_pose) as i32 - kPose_E0_FaceR_LandJump_AimU as i32
                    & 0x8000 as i32
                    == 0
                    && ram_word!(game, samus_pose) as i32
                        - kPose_E8_FaceR_Drained_CrouchFalling as i32
                        & 0x8000 as i32
                        != 0)
        || ram_word!(game, game_state) != 8u16
        || ram_word!(game, power_bomb_explosion_status) != 0u16
        || ram_word!(game, samus_y_speed) != 0u16
        || ram_word!(game, samus_y_subspeed) != 0u16
        || byte_91E291[*(&raw mut game.g_ram as *mut uint8).offset(0xa11) as usize] == 0
    {
        return 0 as uint8;
    }
    if byte_91E291[*(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as usize] == 1u8 {
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8 {
            set_ram_word!(game.samus_new_pose_interrupted = kPose_D6_FaceL_Xray_Stand as uint16);
        } else {
            set_ram_word!(game.samus_new_pose_interrupted = kPose_D5_FaceR_Xray_Stand as uint16);
        }
    } else {
        if byte_91E291[*(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as usize] != 2u8 {
            return 0 as uint8;
        }
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8 {
            set_ram_word!(game.samus_new_pose_interrupted = kPose_DA_FaceL_Xray_Crouch as uint16);
        } else {
            set_ram_word!(game.samus_new_pose_interrupted = kPose_D9_FaceR_Xray_Crouch as uint16);
        }
    }
    set_ram_word!(game.time_is_frozen_flag = 1 as uint16);
    set_ram_word!(game.samus_special_transgfx_index = 5 as uint16);
    let mut i: i32 = 510 as i32;
    while i >= 0i32 {
        set_ram_word!(game.hdma_table_1[(i >> 1i32)] = 255 as uint16);
        i -= 2 as i32;
    }
    DisableEprojs(game);
    DisablePLMs(game);
    DisableAnimtiles(game);
    DisablePaletteFx(game);
    mov24(
        (&raw mut game.g_ram as *mut uint8).offset(0xa88) as *mut LongPtr,
        0x980001 as uint32,
    );
    *(((&raw mut game.g_ram as *mut uint8).offset(0xa8a) as *mut uint16 as *mut uint8).offset(1)
        as *mut uint16) = 0 as uint16;
    set_ram_word!(game.demo_input_prev = -(26424 as i32) as uint16);
    mov24(
        (&raw mut game.g_ram as *mut uint8).offset(0xa8e) as *mut LongPtr,
        0x999098 as uint32,
    );
    set_ram_word!(game.hdma_var_1 = 0 as uint16);
    set_ram_word!(game.demo_input_pre_instr = 0 as uint16);
    set_ram_word!(game.demo_input_instr_timer = 0 as uint16);
    set_ram_word!(game.demo_input_instr_ptr = 0 as uint16);
    set_ram_word!(game.demo_input = 0 as uint16);
    set_ram_word!(game.demo_input_new = 0 as uint16);
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8 {
        set_ram_word!(game.xray_angle = 192 as uint16);
    } else {
        set_ram_word!(game.xray_angle = 64 as uint16);
    }
    1 as uint8
}
pub unsafe fn ResponsibleForXrayStandupGlitch(game: &mut Game) {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as i32
        == kMovementType_05_Crouching as i32
    {
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8 {
            set_ram_word!(game.samus_pose = kPose_28_FaceL_Crouch as uint16);
        } else {
            set_ram_word!(game.samus_pose = kPose_27_FaceR_Crouch as uint16);
        }
    } else if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) as i32
        == kPose_04_FaceL_AimU as i32
    {
        set_ram_word!(game.samus_pose = kPose_02_FaceL_Normal as uint16);
    } else {
        set_ram_word!(game.samus_pose = kPose_01_FaceR_Normal as uint16);
    }
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    set_ram_word!(game.samus_last_different_pose = ram_word!(game, samus_prev_pose));
    *((&raw mut game.g_ram as *mut uint8).offset(0xa26) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xa22) as *mut uint16);
    set_ram_word!(game.samus_prev_pose = ram_word!(game, samus_pose));
    *((&raw mut game.g_ram as *mut uint8).offset(0xa22) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1e) as *mut uint16);
    set_ram_word!(game.samus_movement_handler = ((0x90a337i32) as u16));
    set_ram_word!(game.samus_input_handler = ((0x90e913i32) as u16));
    set_ram_word!(game.demo_timer_counter = -(1 as i32) as uint16);
    let r18: uint16 = (((*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629)) as *const uint8
        as *mut SamusPoseParams)
        .offset(ram_word!(game, samus_pose) as isize))
    .y_radius) as u16)
        .wrapping_sub(ram_word!(game, samus_y_radius));
    if (r18 as i32 & 0x8000 as i32) == 0i32 {
        update_ram_word!(game.samus_y_pos, wrapping_sub(r18));
        set_ram_word!(game.samus_prev_y_pos = ram_word!(game, samus_y_pos));
    }
    EnableEprojs(game);
    EnablePLMs(game);
    EnableAnimtiles(game);
    EnablePaletteFx(game);
}
pub unsafe fn MakeSamusFaceForward(game: &mut Game) {
    if (ram_word!(game, equipped_items) as i32 & 0x20 as i32) != 0i32
        || (ram_word!(game, equipped_items) as i32 & 1 as i32) != 0i32
    {
        set_ram_word!(game.samus_pose = kPose_9B_FaceF_VariaGravitySuit as uint16);
    } else {
        set_ram_word!(game.samus_pose = kPose_00_FaceF_Powersuit as uint16);
    }
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    set_ram_word!(game.samus_last_different_pose = ram_word!(game, samus_prev_pose));
    *((&raw mut game.g_ram as *mut uint8).offset(0xa26) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xa22) as *mut uint16);
    set_ram_word!(game.samus_prev_pose = ram_word!(game, samus_pose));
    *((&raw mut game.g_ram as *mut uint8).offset(0xa22) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1e) as *mut uint16);
    if ram_word!(game, samus_y_radius) != 24u16 {
        update_ram_word!(game.samus_y_pos, wrapping_sub(3u16));
        set_ram_word!(game.samus_prev_y_pos = ram_word!(game, samus_y_pos));
    }
    set_ram_word!(game.frame_handler_alfa = ((0x90e713i32) as u16));
    set_ram_word!(game.frame_handler_beta = ((0x90e8dci32) as u16));
    set_ram_word!(game.samus_new_pose = -(1 as i32) as uint16);
    set_ram_word!(game.samus_new_pose_interrupted = -(1 as i32) as uint16);
    set_ram_word!(game.samus_new_pose_transitional = -(1 as i32) as uint16);
    set_ram_word!(game.samus_momentum_routine_index = 0 as uint16);
    set_ram_word!(game.samus_special_transgfx_index = 0 as uint16);
    set_ram_word!(game.samus_hurt_switch_index = 0 as uint16);
    CallSomeSamusCode(0x1f as uint16, game);
    SetHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0xb42) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0xb44) as *mut uint16,
        0 as uint32,
    );
    set_ram_word!(game.samus_x_base_speed = 0 as uint16);
    set_ram_word!(game.samus_x_base_subspeed = 0 as uint16);
    set_ram_word!(game.samus_y_subspeed = 0 as uint16);
    set_ram_word!(game.samus_y_speed = 0 as uint16);
    set_ram_word!(game.samus_y_dir = 0 as uint16);
    set_ram_word!(game.used_for_ball_bounce_on_landing = 0 as uint16);
    set_ram_word!(game.samus_x_accel_mode = 0 as uint16);
    set_ram_word!(game.flare_counter = 0 as uint16);
    set_ram_word!(game.flare_animation_frame = 0 as uint16);
    set_ram_word!(game.flare_slow_sparks_anim_frame = 0 as uint16);
    set_ram_word!(game.flare_fast_sparks_anim_frame = 0 as uint16);
    set_ram_word!(game.flare_animation_timer = 0 as uint16);
    set_ram_word!(game.flare_slow_sparks_anim_timer = 0 as uint16);
    set_ram_word!(game.flare_fast_sparks_anim_timer = 0 as uint16);
    Samus_LoadSuitPalette(game);
}
static kSomeMotherBrainScripts: [Option<Func_U8>; 5] = unsafe {
    [
        Some(SomeMotherBrainScripts_0 as unsafe fn(&mut Game) -> uint8),
        Some(SomeMotherBrainScripts_1 as unsafe fn(&mut Game) -> uint8),
        Some(SomeMotherBrainScripts_2 as unsafe fn(&mut Game) -> uint8),
        Some(SomeMotherBrainScripts_3_EnableHyperBeam as unsafe fn(&mut Game) -> uint8),
        Some(SomeMotherBrainScripts_4 as unsafe fn(&mut Game) -> uint8),
    ]
};
pub unsafe fn SomeMotherBrainScripts(a: uint16, game: &mut Game) {
    if kSomeMotherBrainScripts[a as usize].expect("non-null function pointer")(game) as i32
        & 1 as i32
        != 0
    {
        set_ram_word!(game.samus_last_different_pose = ram_word!(game, samus_prev_pose));
        *((&raw mut game.g_ram as *mut uint8).offset(0xa26) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xa22) as *mut uint16);
        set_ram_word!(game.samus_prev_pose = ram_word!(game, samus_pose));
        *((&raw mut game.g_ram as *mut uint8).offset(0xa22) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1e) as *mut uint16);
        set_ram_word!(game.samus_new_pose = -(1 as i32) as uint16);
        set_ram_word!(game.samus_new_pose_interrupted = -(1 as i32) as uint16);
        set_ram_word!(game.samus_new_pose_transitional = -(1 as i32) as uint16);
        set_ram_word!(game.samus_momentum_routine_index = 0 as uint16);
        set_ram_word!(game.samus_special_transgfx_index = 0 as uint16);
        set_ram_word!(game.samus_hurt_switch_index = 0 as uint16);
    }
}
pub unsafe fn SomeMotherBrainScripts_0(game: &mut Game) -> uint8 {
    update_ram_word!(game.samus_y_pos, |fresh27| (fresh27 as i32
        - (21 as i32 - ram_word!(game, samus_y_radius) as i32))
        as uint16);
    set_ram_word!(game.samus_prev_y_pos = ram_word!(game, samus_y_pos));
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8 {
        set_ram_word!(game.samus_pose = kPose_E9_FaceL_Drained_CrouchFalling as uint16);
    } else {
        set_ram_word!(game.samus_pose = kPose_E8_FaceR_Drained_CrouchFalling as uint16);
    }
    set_ram_word!(game.samus_anim_frame_skip = 2 as uint16);
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    set_ram_word!(
        game.samus_y_radius = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
            as *const uint8 as *mut SamusPoseParams)
            .offset(ram_word!(game, samus_pose) as isize))
        .y_radius as uint16
    );
    set_ram_word!(game.samus_x_base_speed = 0 as uint16);
    set_ram_word!(game.samus_x_base_subspeed = 0 as uint16);
    set_ram_word!(game.samus_y_subspeed = 0 as uint16);
    set_ram_word!(game.samus_y_speed = 0 as uint16);
    set_ram_word!(game.samus_y_dir = 2 as uint16);
    set_ram_word!(game.flare_counter = 0 as uint16);
    set_ram_word!(game.flare_animation_frame = 0 as uint16);
    set_ram_word!(game.flare_slow_sparks_anim_frame = 0 as uint16);
    set_ram_word!(game.flare_fast_sparks_anim_frame = 0 as uint16);
    set_ram_word!(game.flare_animation_timer = 0 as uint16);
    set_ram_word!(game.flare_slow_sparks_anim_timer = 0 as uint16);
    set_ram_word!(game.flare_fast_sparks_anim_timer = 0 as uint16);
    Samus_LoadSuitPalette(game);
    1 as uint8
}
pub unsafe fn SomeMotherBrainScripts_1(game: &mut Game) -> uint8 {
    set_ram_word!(game.samus_anim_frame_timer = 16 as uint16);
    set_ram_word!(game.samus_anim_frame = 0 as uint16);
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8 {
        set_ram_word!(game.samus_pose = kPose_EB_FaceL_Drained_Stand as uint16);
    } else {
        set_ram_word!(game.samus_pose = kPose_EA_FaceR_Drained_Stand as uint16);
    }
    set_ram_word!(game.frame_handler_gamma = ((0x90e90ei32) as u16));
    1 as uint8
}
pub unsafe fn SomeMotherBrainScripts_2(game: &mut Game) -> uint8 {
    if ram_word!(game, samus_pose) as i32 == kPose_E8_FaceR_Drained_CrouchFalling as i32
        || ram_word!(game, samus_pose) as i32 == kPose_E9_FaceL_Drained_CrouchFalling as i32
    {
        set_ram_word!(game.samus_anim_frame_timer = 1 as uint16);
        set_ram_word!(game.samus_anim_frame = 13 as uint16);
    } else if ram_word!(game, samus_pose) as i32 == kPose_EA_FaceR_Drained_Stand as i32
        || ram_word!(game, samus_pose) as i32 == kPose_EB_FaceL_Drained_Stand as i32
    {
        set_ram_word!(game.samus_anim_frame_timer = 1 as uint16);
        set_ram_word!(game.samus_anim_frame = 4 as uint16);
    }
    set_ram_word!(
        game.samus_y_radius = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
            as *const uint8 as *mut SamusPoseParams)
            .offset(ram_word!(game, samus_pose) as isize))
        .y_radius as uint16
    );
    set_ram_word!(game.samus_x_base_speed = 0 as uint16);
    set_ram_word!(game.samus_x_base_subspeed = 0 as uint16);
    set_ram_word!(game.samus_y_subspeed = 0 as uint16);
    set_ram_word!(game.samus_y_speed = 0 as uint16);
    set_ram_word!(game.samus_y_dir = 2 as uint16);
    1 as uint8
}
pub unsafe fn SomeMotherBrainScripts_3_EnableHyperBeam(game: &mut Game) -> uint8 {
    set_ram_word!(game.equipped_beams = 4105 as uint16);
    UpdateBeamTilesAndPalette(game);
    SpawnPalfxObject(addr_stru_8DE1F0 as uint16, game);
    set_ram_word!(game.hyper_beam_flag = ((0x918000i32) as u16));
    set_ram_word!(game.play_resume_charging_beam_sfx = 0 as uint16);
    0 as uint8
}
pub unsafe fn SomeMotherBrainScripts_4(game: &mut Game) -> uint8 {
    set_ram_word!(game.samus_anim_frame_timer = 16 as uint16);
    set_ram_word!(game.samus_anim_frame = 8 as uint16);
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8 {
        set_ram_word!(game.samus_pose = kPose_E9_FaceL_Drained_CrouchFalling as uint16);
    } else {
        set_ram_word!(game.samus_pose = kPose_E8_FaceR_Drained_CrouchFalling as uint16);
    }
    1 as uint8
}
pub fn nullsub_17(game: &mut Game) {}
static off_91E6E1: [Option<Func_V>; 28] = unsafe {
    [
        Some(SamusFunc_E633_0 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(SamusFunc_E633_3 as unsafe fn(&mut Game) -> ()),
        Some(SamusFunc_E633_4 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(SamusFunc_E633_4 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(SamusFunc_E633_17 as unsafe fn(&mut Game) -> ()),
        Some(SamusFunc_E633_17 as unsafe fn(&mut Game) -> ()),
        Some(SamusFunc_E633_17 as unsafe fn(&mut Game) -> ()),
        Some(SamusFunc_E633_20 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn SamusFunc_E633(game: &mut Game) {
    off_91E6E1[*(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as usize]
        .expect("non-null function pointer")(game);
    if (ram_word!(game, equipped_items) as i32 & 0x2000 as i32) != 0i32 {
        if ram_word!(game, samus_has_momentum_flag) != 0u16
            && ram_word!(game, speed_boost_counter) == 0
        {
            set_ram_word!(game.special_samus_palette_timer = ram_word!(game, speed_boost_counter));
            set_ram_word!(game.special_samus_palette_frame = 0 as uint16);
            set_ram_word!(
                game.speed_boost_counter = *(game.g_rom.offset(crate::rtl::lorom_offset(0x91b61f))
                    as *const uint8 as *mut uint16)
                    .offset(0)
            );
        }
    } else {
        set_ram_word!(game.speed_echoes_index = 0 as uint16);
        set_ram_word!(game.speed_echo_xspeed[0] = 0 as uint16);
        set_ram_word!(game.speed_echo_xspeed[1] = 0 as uint16);
        set_ram_word!(game.samus_has_momentum_flag = 0 as uint16);
        set_ram_word!(game.speed_boost_counter = 0 as uint16);
        set_ram_word!(game.special_samus_palette_frame = 0 as uint16);
        set_ram_word!(game.special_samus_palette_timer = 0 as uint16);
        set_ram_word!(game.speed_echo_xpos[0] = 0 as uint16);
        set_ram_word!(game.speed_echo_xpos[1] = 0 as uint16);
        set_ram_word!(game.speed_echo_ypos[0] = 0 as uint16);
        set_ram_word!(game.speed_echo_ypos[1] = 0 as uint16);
    }
    if ram_word!(game, grapple_beam_function) as i32 == 0x9bc4f0 as i32 & 0xffff as i32 {
        if (ram_word!(game, equipped_beams) as i32 & 0x1000 as i32) != 0i32 {
            if ram_word!(game, flare_counter) as i32 - 16 as i32 & 0x8000 as i32 == 0 {
                QueueSfx1_Max6(0x41 as uint16, game);
            }
        } else {
            set_ram_word!(game.flare_counter = 0 as uint16);
            set_ram_word!(game.flare_animation_frame = 0 as uint16);
            set_ram_word!(game.flare_slow_sparks_anim_frame = 0 as uint16);
            set_ram_word!(game.flare_fast_sparks_anim_frame = 0 as uint16);
            set_ram_word!(game.flare_animation_timer = 0 as uint16);
            set_ram_word!(game.flare_slow_sparks_anim_timer = 0 as uint16);
            set_ram_word!(game.flare_fast_sparks_anim_timer = 0 as uint16);
        }
    } else {
        LoadProjectilePalette(2 as uint16, game);
        QueueSfx1_Max6(6 as uint16, game);
    }
    Samus_LoadSuitPalette(game);
    if ram_word!(game, samus_health) as i32 - 31 as i32 & 0x8000 as i32 != 0 {
        QueueSfx3_Max6(2 as uint16, game);
    }
}
pub unsafe fn Samus_UpdatePreviousPose_0(game: &mut Game) {
    set_ram_word!(game.samus_last_different_pose = ram_word!(game, samus_prev_pose));
    *((&raw mut game.g_ram as *mut uint8).offset(0xa26) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xa22) as *mut uint16);
    set_ram_word!(game.samus_prev_pose = ram_word!(game, samus_pose));
    *((&raw mut game.g_ram as *mut uint8).offset(0xa22) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1e) as *mut uint16);
}
pub unsafe fn SamusFunc_E633_0(game: &mut Game) {
    let current_block_4: u64;
    if ram_word!(game, samus_pose) != 0 {
        if ram_word!(game, samus_pose) as i32 == kPose_9B_FaceF_VariaGravitySuit as i32
            && (ram_word!(game, equipped_items) as i32 & 1 as i32) == 0i32
            && (ram_word!(game, equipped_items) as i32 & 0x20 as i32) == 0i32
        {
            set_ram_word!(game.samus_pose = kPose_00_FaceF_Powersuit as uint16);
            current_block_4 = 5176481699505674939;
        } else {
            current_block_4 = 7351195479953500246;
        }
    } else if (ram_word!(game, equipped_items) as i32 & 1 as i32) != 0i32
        || (ram_word!(game, equipped_items) as i32 & 0x20 as i32) != 0i32
    {
        set_ram_word!(game.samus_pose = kPose_9B_FaceF_VariaGravitySuit as uint16);
        current_block_4 = 5176481699505674939;
    } else {
        current_block_4 = 7351195479953500246;
    }
    if current_block_4 == 5176481699505674939 {
        SamusFunc_F433(game);
        Samus_SetAnimationFrameIfPoseChanged(game);
        Samus_UpdatePreviousPose_0(game);
    };
}
pub unsafe fn SamusFunc_E633_3(game: &mut Game) {
    let current_block: u64;
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa22) as *mut uint16) = 260 as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa22) as *mut uint16) = 264 as uint16;
    }
    if ram_word!(game, samus_pose) as i32 != kPose_81_FaceR_Screwattack as i32
        && ram_word!(game, samus_pose) as i32 != kPose_82_FaceL_Screwattack as i32
    {
        if ram_word!(game, samus_pose) as i32 != kPose_1B_FaceR_SpaceJump as i32
            && ram_word!(game, samus_pose) as i32 != kPose_1C_FaceL_SpaceJump as i32
        {
            current_block = 14317573491659972910;
        } else if (ram_word!(game, equipped_items) as i32 & 8 as i32) != 0i32 {
            if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8 {
                set_ram_word!(game.samus_pose = kPose_82_FaceL_Screwattack as uint16);
            } else {
                set_ram_word!(game.samus_pose = kPose_81_FaceR_Screwattack as uint16);
            }
            current_block = 14317573491659972910;
        } else {
            current_block = 5564999883274278710;
        }
    } else if (ram_word!(game, equipped_items) as i32 & 8 as i32) == 0i32 {
        current_block = 5564999883274278710;
    } else {
        current_block = 14317573491659972910;
    }
    if current_block == 5564999883274278710 {
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8 {
            set_ram_word!(game.samus_pose = kPose_1A_FaceL_SpinJump as uint16);
        } else {
            set_ram_word!(game.samus_pose = kPose_19_FaceR_SpinJump as uint16);
        }
    }
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa22) as *mut uint16) = 772 as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa22) as *mut uint16) = 776 as uint16;
    }
    Samus_UpdatePreviousPose_0(game);
}
pub unsafe fn SamusFunc_E633_4(game: &mut Game) {
    if (ram_word!(game, equipped_items) as i32 & 2 as i32) != 0i32 {
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8 {
            set_ram_word!(game.samus_pose = kPose_7A_FaceL_Springball_Ground as uint16);
        } else {
            set_ram_word!(game.samus_pose = kPose_79_FaceR_Springball_Ground as uint16);
        }
        SamusFunc_F433(game);
        Samus_SetAnimationFrameIfPoseChanged(game);
        Samus_UpdatePreviousPose_0(game);
    }
}
pub unsafe fn SamusFunc_E633_17(game: &mut Game) {
    if (ram_word!(game, equipped_items) as i32 & 2 as i32) == 0i32 {
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8 {
            set_ram_word!(game.samus_pose = kPose_41_FaceL_Morphball_Ground as uint16);
        } else {
            set_ram_word!(game.samus_pose = kPose_1D_FaceR_Morphball_Ground as uint16);
        }
        SamusFunc_F433(game);
        Samus_SetAnimationFrameIfPoseChanged(game);
        Samus_UpdatePreviousPose_0(game);
    }
}
pub unsafe fn SamusFunc_E633_20(game: &mut Game) {
    if (ram_word!(game, equipped_items) as i32 & 8 as i32) != 0i32 {
        set_ram_word!(game.samus_anim_frame = 23 as uint16);
    } else {
        set_ram_word!(game.samus_anim_frame = 3 as uint16);
    };
}
pub fn nullsub_18(game: &mut Game) {}
static kSamus_HandleTransFromBlockColl: [Option<Func_V>; 6] = unsafe {
    [
        Some(nullsub_18 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransFromBlockColl_1 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransFromBlockColl_2 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransFromBlockColl_3 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransFromBlockColl_4 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransFromBlockColl_5 as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn Samus_HandleTransFromBlockColl(game: &mut Game) {
    if ram_word!(game, input_to_pose_calc) != 0 {
        kSamus_HandleTransFromBlockColl[ram_word!(game, input_to_pose_calc) as uint8 as usize]
            .expect("non-null function pointer")(game);
    }
}
pub unsafe fn Samus_HandleTransFromBlockColl_3(game: &mut Game) {
    set_ram_word!(game.samus_new_pose = ram_word!(game, samus_pose));
    set_ram_word!(game.samus_momentum_routine_index = 5 as uint16);
}
pub unsafe fn Samus_HandleTransFromBlockColl_4(game: &mut Game) {
    set_ram_word!(game.samus_new_pose = ram_word!(game, samus_pose));
    set_ram_word!(game.samus_momentum_routine_index = 5 as uint16);
}
pub fn nullsub_18_U8(game: &mut Game) -> uint8 {
    0 as uint8
}
static off_91E951: [Option<Func_U8>; 6] = unsafe {
    [
        Some(Samus_HandleTransFromBlockColl_1_0 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransFromBlockColl_1_1 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransFromBlockColl_1_2 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransFromBlockColl_1_3 as unsafe fn(&mut Game) -> uint8),
        Some(nullsub_18_U8 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransFromBlockColl_1_5 as unsafe fn(&mut Game) -> uint8),
    ]
};
pub unsafe fn Samus_HandleTransFromBlockColl_2(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xdc6) as *mut uint16 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1_usize) as isize,
    ) != 4u8
    {
        let v0: uint16 = (4u16).wrapping_mul(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xdc6) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1_usize) as isize,
                )) as u16,
        );
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8 {
            set_ram_word!(
                game.samus_new_pose = *(game.g_rom.offset(crate::rtl::lorom_offset(0x91e921))
                    as *const uint8 as *mut uint16)
                    .offset(((v0 as i32 >> 1 as i32) + 1 as i32) as isize,)
            );
        } else {
            set_ram_word!(
                game.samus_new_pose = *(game.g_rom.offset(crate::rtl::lorom_offset(0x91e921))
                    as *const uint8 as *mut uint16)
                    .offset((v0 as i32 >> 1 as i32) as isize)
            );
        }
        set_ram_word!(game.samus_momentum_routine_index = 5 as uint16);
    }
}
pub unsafe fn Samus_HandleTransFromBlockColl_1(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xdc6) as *mut uint16 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1_usize) as isize,
    ) != 4u8
    {
        if off_91E951[*((&raw mut game.g_ram as *mut uint8).offset(0xdc6) as *mut uint16
            as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1_usize) as isize,
            ) as usize]
            .expect("non-null function pointer")(game) as i32
            & 1 as i32
            != 0
        {
            set_ram_word!(game.samus_momentum_routine_index = 0 as uint16);
        } else {
            set_ram_word!(game.samus_momentum_routine_index = 5 as uint16);
        }
    }
}
pub unsafe fn Samus_HandleTransFromBlockColl_1_0(game: &mut Game) -> uint8 {
    let mut v0: int16 = 0;
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa23) as i32
        == kMovementType_03_SpinJumping as i32
        || *(&raw mut game.g_ram as *mut uint8).offset(0xa23) as i32
            == kMovementType_14_WallJumping as i32
    {
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8 {
            set_ram_word!(game.samus_new_pose = kPose_A7_FaceL_LandSpinJump as uint16);
        } else {
            set_ram_word!(game.samus_new_pose = kPose_A6_FaceR_LandSpinJump as uint16);
        }
        0 as uint8
    } else {
        v0 = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629)) as *const uint8
            as *mut SamusPoseParams)
            .offset(ram_word!(game, samus_pose) as isize))
        .direction_shots_fired as int16;
        if v0 == 255i16 {
            if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8 {
                set_ram_word!(game.samus_new_pose = kPose_A5_FaceL_LandJump as uint16);
            } else {
                set_ram_word!(game.samus_new_pose = kPose_A4_FaceR_LandJump as uint16);
            }
            0 as uint8
        } else {
            if !(v0 != 2i16 && v0 != 7i16) {
                if (ram_word!(game, button_config_shoot_x) as i32
                    & ram_word!(game, joypad1_lastkeys) as i32)
                    == 0i32
                {
                    v0 = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629)) as *const uint8
                        as *mut SamusPoseParams)
                        .offset(ram_word!(game, samus_pose) as isize))
                    .direction_shots_fired as int16;
                } else {
                    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8 {
                        set_ram_word!(game.samus_new_pose = kPose_E7_FaceL_LandJump_Fire as uint16);
                    } else {
                        set_ram_word!(game.samus_new_pose = kPose_E6_FaceR_LandJump_Fire as uint16);
                    }
                    return 0 as uint8;
                }
            }
            set_ram_word!(
                game.samus_new_pose = *(game.g_rom.offset(crate::rtl::lorom_offset(0x91e9f3))
                    as *const uint8 as *mut uint16)
                    .offset(v0 as isize)
            );
            0 as uint8
        }
    }
}
pub unsafe fn Samus_HandleTransFromBlockColl_1_1(game: &mut Game) -> uint8 {
    let mut v0: int16 = 0;
    v0 = (2i16).wrapping_mul((ram_word!(game, used_for_ball_bounce_on_landing)) as i16);
    if 2 as i32 * ram_word!(game, used_for_ball_bounce_on_landing) as i32 != 0 {
        if v0 == 2i16 {
            set_ram_word!(game.samus_new_pose = ram_word!(game, samus_pose));
            return 0 as uint8;
        }
        if v0 != 4i16 {
            Unreachable(game);
        }
    } else if ram_word!(game, samus_y_speed) as i32 - 3 as i32 & 0x8000 as i32 == 0 {
        set_ram_word!(game.samus_new_pose = ram_word!(game, samus_pose));
        return 0 as uint8;
    }
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8 {
        set_ram_word!(game.samus_new_pose = kPose_41_FaceL_Morphball_Ground as uint16);
    } else {
        set_ram_word!(game.samus_new_pose = kPose_1D_FaceR_Morphball_Ground as uint16);
    }
    0 as uint8
}
pub unsafe fn Samus_HandleTransFromBlockColl_1_2(game: &mut Game) -> uint8 {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8 {
        set_ram_word!(game.samus_new_pose = 66 as uint16);
    } else {
        set_ram_word!(game.samus_new_pose = 32 as uint16);
    }
    0 as uint8
}
pub unsafe fn Samus_HandleTransFromBlockColl_1_3(game: &mut Game) -> uint8 {
    let mut v1: int16 = 0;
    if (ram_word!(game, button_config_jump_a) as i32 & ram_word!(game, joypad1_lastkeys) as i32)
        != 0i32
    {
        set_ram_word!(game.samus_new_pose = ram_word!(game, samus_pose));
        return 0 as uint8;
    }
    v1 = (2i16).wrapping_mul((ram_word!(game, used_for_ball_bounce_on_landing) as uint8) as i16);
    if v1 != 0 {
        if v1 == 2i16 {
            set_ram_word!(game.samus_new_pose = ram_word!(game, samus_pose));
            return 0 as uint8;
        }
        if v1 != 4i16 {
            Unreachable(game);
        }
    } else if ram_word!(game, samus_y_speed) as i32 - 3 as i32 & 0x8000 as i32 == 0 {
        set_ram_word!(game.samus_new_pose = ram_word!(game, samus_pose));
        return 0 as uint8;
    }
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8 {
        set_ram_word!(game.samus_new_pose = kPose_7A_FaceL_Springball_Ground as uint16);
    } else {
        set_ram_word!(game.samus_new_pose = 121 as uint16);
    }
    0 as uint8
}
pub unsafe fn Samus_HandleTransFromBlockColl_1_5(game: &mut Game) -> uint8 {
    set_ram_word!(game.samus_new_pose = ram_word!(game, samus_pose));
    0 as uint8
}
pub unsafe fn Samus_HandleTransFromBlockColl_5(game: &mut Game) {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8 {
        set_ram_word!(game.samus_new_pose = kPose_84_FaceL_Walljump as uint16);
    } else {
        set_ram_word!(game.samus_new_pose = kPose_83_FaceR_Walljump as uint16);
    }
    set_ram_word!(game.samus_momentum_routine_index = 5 as uint16);
}
pub unsafe fn Samus_CheckWalkedIntoSomething(game: &mut Game) {
    let mut cres_0: CheckEnemyColl_Result = CheckSolidEnemyColl_Result {
        collision: 0,
        amt: 0,
    };
    let mut cres: CheckEnemyColl_Result = CheckSolidEnemyColl_Result {
        collision: 0,
        amt: 0,
    };
    let current_block: u64;
    let mut amt: int32 = 0;
    if ram_word!(game, samus_collides_with_solid_enemy) != 0u16
        && *(&raw mut game.g_ram as *mut uint8).offset(0xa1f) == 1u8
    {
        set_ram_word!(
            game.samus_new_pose = *(game.g_rom.offset(crate::rtl::lorom_offset(0x91eb74))
                as *const uint8 as *mut uint16)
                .offset(
                    *(&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
                        as *const uint8 as *mut SamusPoseParams)
                        .offset(0))
                    .direction_shots_fired)
                        .offset((8 as i32 * ram_word!(game, samus_pose) as i32) as isize,)
                        as isize,
                )
        );
        set_ram_word!(game.samus_collides_with_solid_enemy = 0 as uint16);
        return;
    }
    if ram_word!(game, samus_new_pose) == 0xffffu16
        || (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629)) as *const uint8
            as *mut SamusPoseParams)
            .offset(ram_word!(game, samus_new_pose) as isize))
        .movement_type
            != 1u8
    {
        set_ram_word!(game.samus_collides_with_solid_enemy = 0 as uint16);
        return;
    }
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) != 4u8 {
        set_ram_word!(game.samus_collision_direction = 1 as uint16);
        amt = (1 as int16 as i32) << 16 as i32;
        cres = Samus_CheckSolidEnemyColl(amt, game);
        if cres.collision == 0 {
            set_ram_word!(game.samus_collision_direction = 1 as uint16);
            amt = (1 as int16 as i32) << 16 as i32;
            current_block = 12513788902857388954;
        } else {
            current_block = 8208193451455325075;
        }
    } else {
        set_ram_word!(game.samus_collision_direction = 0 as uint16);
        amt = (1 as int16 as i32) << 16 as i32;
        cres_0 = Samus_CheckSolidEnemyColl(amt, game);
        if cres_0.collision != 0 {
            current_block = 8208193451455325075;
        } else {
            amt = (-(1 as i32) as int16 as i32) << 16 as i32;
            set_ram_word!(game.samus_collision_direction = 0 as uint16);
            current_block = 12513788902857388954;
        }
    }
    if current_block == 12513788902857388954 {
        Samus_MoveRight_NoSolidColl(amt, game);
        if (ram_word!(game, samus_collision_flag) == 0) {
            set_ram_word!(game.samus_collides_with_solid_enemy = 0 as uint16);
            return;
        }
    }
    set_ram_word!(
        game.samus_new_pose =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x91eb74)) as *const uint8 as *mut uint16)
                .offset(
                    *(&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
                        as *const uint8 as *mut SamusPoseParams)
                        .offset(0))
                    .direction_shots_fired)
                        .offset((8 as i32 * ram_word!(game, samus_new_pose) as i32) as isize,)
                        as isize,
                )
    );
    set_ram_word!(game.samus_collides_with_solid_enemy = 0 as uint16);
}
pub fn nullsub_19(game: &mut Game) {}
pub fn nullsub_20(game: &mut Game) {}
pub fn nullsub_21(game: &mut Game) {}
pub fn nullsub_22(game: &mut Game) {}
static kSamus_HandleTransitionsA: [Option<Func_V>; 9] = unsafe {
    [
        Some(nullsub_18 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsA_1 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsA_2 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_21 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_22 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsA_5 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsA_6 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsA_7 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsA_8 as unsafe fn(&mut Game) -> ()),
    ]
};
static kSamus_HandleTransitionsB: [Option<Func_V>; 11] = unsafe {
    [
        Some(nullsub_18 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsB_1 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsB_2 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsB_3 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsB_4 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsB_5 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_19 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_20 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsB_8 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsB_9 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsB_10 as unsafe fn(&mut Game) -> ()),
    ]
};
static kSamus_HandleTransitionsC: [Option<Func_V>; 9] = unsafe {
    [
        Some(nullsub_18 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsC_1 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsC_2 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsC_3 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsC_4 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsC_5 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsC_6 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsC_7 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsC_8 as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn Samus_HandleTransitions(game: &mut Game) {
    let mut current_block: u64;
    if (ram_word!(game, samus_new_pose_transitional) as i32 & 0x8000 as i32) == 0i32 {
        if ram_word!(game, samus_hurt_switch_index) != 3u16 {
            if ram_word!(game, samus_hurt_switch_index) == 1u16 {
                current_block = 5743517142682009007;
            } else {
                current_block = 6038634297008360266;
            }
        } else if ram_word!(game, samus_special_transgfx_index) != 9u16 {
            current_block = 6038634297008360266;
        } else {
            current_block = 13536709405535804910;
        }
        if current_block == 13536709405535804910 {
        } else {
            if current_block == 6038634297008360266 {
                set_ram_word!(game.samus_pose = ram_word!(game, samus_new_pose_transitional));
                SamusFunc_F433(game);
                Samus_SetAnimationFrameIfPoseChanged(game);
            }
            kSamus_HandleTransitionsC[ram_word!(game, samus_hurt_switch_index) as usize]
                .expect("non-null function pointer")(game);
            current_block = 10476341332908994874;
        }
    } else {
        current_block = 13536709405535804910;
    }
    if current_block == 13536709405535804910 {
        if (ram_word!(game, samus_new_pose_interrupted) as i32 & 0x8000 as i32) == 0i32 {
            set_ram_word!(game.samus_pose = ram_word!(game, samus_new_pose_interrupted));
            if SamusFunc_F404(game) as i32 & 1 as i32 == 0 {
                kSamus_HandleTransitionsB[ram_word!(game, samus_special_transgfx_index) as usize]
                    .expect("non-null function pointer")(game);
            }
            current_block = 10476341332908994874;
        } else {
            Samus_CheckWalkedIntoSomething(game);
            if (ram_word!(game, samus_new_pose) as i32 & 0x8000 as i32) == 0i32 {
                set_ram_word!(game.samus_pose = ram_word!(game, samus_new_pose));
                if SamusFunc_F404(game) as i32 & 1 as i32 == 0 {
                    kSamus_HandleTransitionsA
                        [ram_word!(game, samus_momentum_routine_index) as usize]
                        .expect("non-null function pointer")(game);
                }
                current_block = 10476341332908994874;
            } else {
                current_block = 10093773643351532501;
            }
        }
    }
    if current_block == 10476341332908994874 {
        set_ram_word!(game.samus_last_different_pose = ram_word!(game, samus_prev_pose));
        *((&raw mut game.g_ram as *mut uint8).offset(0xa26) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xa22) as *mut uint16);
        set_ram_word!(game.samus_prev_pose = ram_word!(game, samus_pose));
        *((&raw mut game.g_ram as *mut uint8).offset(0xa22) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1e) as *mut uint16);
    }
    set_ram_word!(game.input_to_pose_calc = 0 as uint16);
}
pub unsafe fn Samus_HandleTransitionsA_1(game: &mut Game) {
    if ram_word!(game, samus_x_base_speed) != 0u16 || ram_word!(game, samus_x_base_subspeed) != 0u16
    {
        AddToHiLo(
            (&raw mut game.g_ram as *mut uint8).offset(0xb46) as *mut uint16,
            (&raw mut game.g_ram as *mut uint8).offset(0xb48) as *mut uint16,
            (ram_word!(game, samus_x_extra_run_speed) as uint32) << 16 as i32
                | ram_word!(game, samus_x_extra_run_subspeed) as uint32,
        );
        set_ram_word!(game.samus_x_accel_mode = 2 as uint16);
        Samus_CancelSpeedBoost(game);
        SetHiLo(
            (&raw mut game.g_ram as *mut uint8).offset(0xb42) as *mut uint16,
            (&raw mut game.g_ram as *mut uint8).offset(0xb44) as *mut uint16,
            0 as uint32,
        );
        SamusFunc_EC80(game);
    } else {
        Samus_HandleTransitionsA_2(game);
    };
}
pub unsafe fn Samus_HandleTransitionsA_6(game: &mut Game) {
    set_ram_word!(game.samus_x_accel_mode = 0 as uint16);
    set_ram_word!(game.samus_x_base_speed = 0 as uint16);
    set_ram_word!(game.samus_x_base_subspeed = 0 as uint16);
    Samus_HandleTransitionsA_8(game);
}
pub unsafe fn Samus_HandleTransitionsA_8(game: &mut Game) {
    Samus_CancelSpeedBoost(game);
    SetHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0xb42) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0xb44) as *mut uint16,
        0 as uint32,
    );
    SamusFunc_EC80(game);
}
pub unsafe fn Samus_HandleTransitionsA_2(game: &mut Game) {
    set_ram_word!(game.samus_x_accel_mode = 0 as uint16);
    Samus_CancelSpeedBoost(game);
}
static word_91ED36: [uint16; 12] = [5, 5, 9, 9, 0, 0, 0, 0, 0, 0, 0, 0];
pub unsafe fn Samus_HandleTransitionsA_7(game: &mut Game) {
    let mut amt: int32 = 0;
    let mut current_block_11: u64;
    if ram_word!(game, samus_pose) as i32 - kPose_DB as i32 & 0x8000 as i32 != 0 {
        amt = (word_91ED36[(ram_word!(game, samus_pose) as i32 - 53 as i32) as usize] as int16
            as i32)
            << 16 as i32;
        if amt != 0 {
            current_block_11 = 7426089221157146073;
        } else {
            current_block_11 = 735147466149431745;
        }
    } else if ram_word!(game, samus_pose) as i32 - kPose_F1_FaceR_CrouchTrans_AimU as i32
        & 0x8000 as i32
        == 0
        && ram_word!(game, samus_pose) as i32 - kPose_F7_FaceR_StandTrans_AimU as i32
            & 0x8000 as i32
            != 0
    {
        amt = (5 as int16 as i32) << 16 as i32;
        current_block_11 = 7426089221157146073;
    } else {
        current_block_11 = 1856101646708284338;
    }
    if current_block_11 == 7426089221157146073 {
        set_ram_word!(
            game.samus_y_radius = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
                as *const uint8 as *mut SamusPoseParams)
                .offset(ram_word!(game, samus_pose) as isize))
            .y_radius as uint16
        );
        amt = Samus_CollDetectChangedPose(amt, game);
        current_block_11 = 735147466149431745;
    }
    if current_block_11 == 735147466149431745 {
        update_ram_word!(
            game.samus_y_pos,
            |fresh40| (fresh40 as i32 + (amt >> 16 as i32) as i32) as uint16
        );
        set_ram_word!(game.samus_prev_y_pos = ram_word!(game, samus_y_pos));
        if ram_word!(game, used_for_ball_bounce_on_landing) != 0 {
            set_ram_word!(game.used_for_ball_bounce_on_landing = 0 as uint16);
            set_ram_word!(game.samus_y_subspeed = 0 as uint16);
            set_ram_word!(game.samus_y_speed = 0 as uint16);
            set_ram_word!(game.samus_y_dir = 0 as uint16);
        }
    };
}
static kSamus_HandleTransitionsB_1: [Option<Func_U8>; 28] = unsafe {
    [
        Some(Samus_HandleTransitionsB_1_0 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_0 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_0 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_0 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_4 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_0 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_6 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_7 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_4 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_4 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_10 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_11 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_11 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_0 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_11 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_11 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_0 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_4 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_4 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_4 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_0 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_0 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_10 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_11 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_11 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_11 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_11 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_11 as unsafe fn(&mut Game) -> uint8),
    ]
};
pub unsafe fn Samus_HandleTransitionsB_1(game: &mut Game) {
    kSamus_HandleTransitionsB_1[*(&raw mut game.g_ram as *mut uint8).offset(0xa23) as usize]
        .expect("non-null function pointer")(game);
    Samus_SetSpeedForKnockback_Y(game);
    set_ram_word!(game.bomb_jump_dir = 0 as uint16);
    set_ram_word!(game.samus_contact_damage_index = 0 as uint16);
    set_ram_word!(game.samus_hurt_flash_counter = 1 as uint16);
}
pub fn Samus_HandleTransitionsB_1_10(game: &mut Game) -> uint8 {
    0 as uint8
}
pub fn Samus_HandleTransitionsB_1_11(game: &mut Game) -> uint8 {
    0 as uint8
}
pub unsafe fn Samus_HandleTransitionsB_1_6(game: &mut Game) -> uint8 {
    if ram_word!(game, frame_handler_gamma) as i32 == 0x90e41b as i32 & 0xffff as i32 {
        0 as uint8
    } else {
        Samus_HandleTransitionsB_1_0(game)
    }
}
pub unsafe fn Samus_HandleTransitionsB_1_0(game: &mut Game) -> uint8 {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8 {
        if ram_word!(game, knockback_x_dir) != 0 {
            if (ram_word!(game, joypad1_lastkeys) as i32 & 0x200 as i32) != 0i32 {
                set_ram_word!(game.knockback_dir = 5 as uint16);
            } else {
                set_ram_word!(game.knockback_dir = 2 as uint16);
            }
        } else if (ram_word!(game, joypad1_lastkeys) as i32 & 0x200 as i32) != 0i32 {
            set_ram_word!(game.knockback_dir = 4 as uint16);
        } else {
            set_ram_word!(game.knockback_dir = 1 as uint16);
        }
    } else if ram_word!(game, knockback_x_dir) != 0 {
        if (ram_word!(game, joypad1_lastkeys) as i32 & 0x100 as i32) != 0i32 {
            set_ram_word!(game.knockback_dir = 5 as uint16);
        } else {
            set_ram_word!(game.knockback_dir = 2 as uint16);
        }
    } else if (ram_word!(game, joypad1_lastkeys) as i32 & 0x100 as i32) != 0i32 {
        set_ram_word!(game.knockback_dir = 4 as uint16);
    } else {
        set_ram_word!(game.knockback_dir = 1 as uint16);
    }
    set_ram_word!(game.samus_movement_handler = ((0x90df38i32) as u16));
    1 as uint8
}
pub unsafe fn Samus_HandleTransitionsB_1_4(game: &mut Game) -> uint8 {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8 {
        set_ram_word!(game.knockback_dir = 1 as uint16);
    } else {
        set_ram_word!(game.knockback_dir = 2 as uint16);
    }
    set_ram_word!(game.samus_movement_handler = ((0x90df38i32) as u16));
    0 as uint8
}
pub unsafe fn Samus_HandleTransitionsB_1_7(game: &mut Game) -> uint8 {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8 {
        set_ram_word!(game.knockback_dir = 1 as uint16);
    } else {
        set_ram_word!(game.knockback_dir = 2 as uint16);
    }
    set_ram_word!(game.samus_movement_handler = ((0x90df38i32) as u16));
    1 as uint8
}
pub unsafe fn Samus_HandleTransitionsB_2(game: &mut Game) {
    set_ram_word!(game.knockback_dir = 0 as uint16);
    set_ram_word!(game.samus_movement_handler = ((0x90a337i32) as u16));
    set_ram_word!(game.samus_y_speed = 0 as uint16);
    set_ram_word!(game.samus_y_subspeed = 0 as uint16);
    set_ram_word!(game.samus_y_dir = 0 as uint16);
    Samus_AlignBottomWithPrevPose(game);
}
pub unsafe fn Samus_HandleTransitionsB_3(game: &mut Game) {
    set_ram_word!(
        game.bomb_jump_dir = ((ram_word!(game, bomb_jump_dir) as uint8) as u16) | (0x800u16)
    );
    set_ram_word!(game.samus_movement_handler = ((0x90e025i32) as u16));
    if ram_word!(game, samus_input_handler) as i32 != 0x90e91d as i32 & 0xffff as i32 {
        set_ram_word!(game.samus_input_handler = ((0x90e90ei32) as u16));
    }
}
pub unsafe fn Samus_HandleTransitionsB_4(game: &mut Game) {
    Samus_InitJump(game);
}
pub unsafe fn Samus_HandleTransitionsB_5(game: &mut Game) {
    let current_block_21: u64;
    match *(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as i32 {
        5 => {
            if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8 {
                set_ram_word!(game.xray_angle = 192 as uint16);
            } else {
                set_ram_word!(game.xray_angle = 64 as uint16);
            }
            current_block_21 = 619521094565907383;
        }
        0 | 21 | 1 => {
            if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8 {
                set_ram_word!(game.xray_angle = 192 as uint16);
            } else {
                set_ram_word!(game.xray_angle = 64 as uint16);
            }
            current_block_21 = 619521094565907383;
        }
        _ => {
            current_block_21 = 5948590327928692120;
        }
    }
    if current_block_21 == 619521094565907383 {
        set_ram_word!(game.samus_anim_frame = 2 as uint16);
        set_ram_word!(game.samus_anim_frame_timer = 63 as uint16);
        set_ram_word!(game.samus_movement_handler = ((0x90e94fi32) as u16));
        set_ram_word!(game.samus_input_handler = ((0x90e918i32) as u16));
        set_ram_word!(game.timer_for_shine_timer = 8 as uint16);
        set_ram_word!(game.special_samus_palette_timer = 1 as uint16);
        set_ram_word!(game.special_samus_palette_frame = 0 as uint16);
        set_ram_word!(game.samus_shine_timer = 0 as uint16);
        set_ram_word!(game.flare_counter = 0 as uint16);
        set_ram_word!(game.flare_animation_frame = 0 as uint16);
        set_ram_word!(game.flare_slow_sparks_anim_frame = 0 as uint16);
        set_ram_word!(game.flare_fast_sparks_anim_frame = 0 as uint16);
        set_ram_word!(game.flare_animation_timer = 0 as uint16);
        set_ram_word!(game.flare_slow_sparks_anim_timer = 0 as uint16);
        set_ram_word!(game.flare_fast_sparks_anim_timer = 0 as uint16);
        QueueSfx1_Max6(9 as uint16, game);
    };
}
pub unsafe fn Samus_HandleTransitionsB_8(game: &mut Game) {
    update_ram_word!(game.samus_y_pos, wrapping_sub(5u16));
    set_ram_word!(game.samus_prev_y_pos = ram_word!(game, samus_y_pos));
    set_ram_word!(game.frame_handler_alfa = ((0x90e8cdi32) as u16));
}
pub unsafe fn Samus_HandleTransitionsB_9(game: &mut Game) {
    let mut v0: int16 = 0;
    let mut v1: int16 = 0;
    GrappleBeamFunc_BD95(game);
    v0 = ((ram_word!(game, samus_x_pos)) as i16)
        .wrapping_sub((ram_word!(game, samus_prev_x_pos)) as i16);
    if ((ram_word!(game, samus_x_pos)) as i16)
        .wrapping_sub((ram_word!(game, samus_prev_x_pos)) as i16)
        < 0i16
    {
        if v0 as i32 + 12 as i32 & 0x8000 as i32 != 0 {
            set_ram_word!(
                game.samus_prev_x_pos = (ram_word!(game, samus_x_pos)).wrapping_add(12u16)
            );
        }
    } else if v0 as i32 - 13 as i32 & 0x8000 as i32 == 0 {
        set_ram_word!(game.samus_prev_x_pos = (ram_word!(game, samus_x_pos)).wrapping_sub(12u16));
    }
    v1 = ((ram_word!(game, samus_y_pos)) as i16)
        .wrapping_sub((ram_word!(game, samus_prev_y_pos)) as i16);
    if ((ram_word!(game, samus_y_pos)) as i16)
        .wrapping_sub((ram_word!(game, samus_prev_y_pos)) as i16)
        < 0i16
    {
        if v1 as i32 + 12 as i32 & 0x8000 as i32 != 0 {
            set_ram_word!(
                game.samus_prev_y_pos = (ram_word!(game, samus_y_pos)).wrapping_add(12u16)
            );
        }
    } else if v1 as i32 - 13 as i32 & 0x8000 as i32 == 0 {
        set_ram_word!(game.samus_prev_y_pos = (ram_word!(game, samus_y_pos)).wrapping_sub(12u16));
    }
    Samus_CancelSpeedBoost(game);
    set_ram_word!(game.samus_x_base_speed = 0 as uint16);
    set_ram_word!(game.samus_x_base_subspeed = 0 as uint16);
    set_ram_word!(game.samus_y_subspeed = 0 as uint16);
    set_ram_word!(game.samus_y_speed = 0 as uint16);
    SetHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0xb42) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0xb44) as *mut uint16,
        0 as uint32,
    );
}
pub unsafe fn Samus_HandleTransitionsB_10(game: &mut Game) {
    GrappleBeamFunc_BEEB(game);
    Samus_HandleTransitionsB_9B(game);
}
pub unsafe fn Samus_HandleTransitionsB_9B(game: &mut Game) {
    let mut v0: int16 = 0;
    let mut v1: int16 = 0;
    v0 = ((ram_word!(game, samus_x_pos)) as i16)
        .wrapping_sub((ram_word!(game, samus_prev_x_pos)) as i16);
    if ((ram_word!(game, samus_x_pos)) as i16)
        .wrapping_sub((ram_word!(game, samus_prev_x_pos)) as i16)
        < 0i16
    {
        if v0 as i32 + 12 as i32 & 0x8000 as i32 != 0 {
            set_ram_word!(
                game.samus_prev_x_pos = (ram_word!(game, samus_x_pos)).wrapping_add(12u16)
            );
        }
    } else if v0 as i32 - 13 as i32 & 0x8000 as i32 == 0 {
        set_ram_word!(game.samus_prev_x_pos = (ram_word!(game, samus_x_pos)).wrapping_sub(12u16));
    }
    v1 = ((ram_word!(game, samus_y_pos)) as i16)
        .wrapping_sub((ram_word!(game, samus_prev_y_pos)) as i16);
    if ((ram_word!(game, samus_y_pos)) as i16)
        .wrapping_sub((ram_word!(game, samus_prev_y_pos)) as i16)
        < 0i16
    {
        if v1 as i32 + 12 as i32 & 0x8000 as i32 != 0 {
            set_ram_word!(
                game.samus_prev_y_pos = (ram_word!(game, samus_y_pos)).wrapping_add(12u16)
            );
        }
    } else if v1 as i32 - 13 as i32 & 0x8000 as i32 == 0 {
        set_ram_word!(game.samus_prev_y_pos = (ram_word!(game, samus_y_pos)).wrapping_sub(12u16));
    }
    Samus_CancelSpeedBoost(game);
    set_ram_word!(game.samus_x_base_speed = 0 as uint16);
    set_ram_word!(game.samus_x_base_subspeed = 0 as uint16);
    set_ram_word!(game.samus_y_subspeed = 0 as uint16);
    set_ram_word!(game.samus_y_speed = 0 as uint16);
    SetHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0xb42) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0xb44) as *mut uint16,
        0 as uint32,
    );
}
pub fn nullsub_23(game: &mut Game) {}
static kSamus_HandleTransitionsA_5: [Option<Func_V>; 7] = unsafe {
    [
        Some(sub_91EFC3 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsA_5_1 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsA_5_2 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_23 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsA_5_4 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsA_5_5 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsA_5_6 as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn Samus_HandleTransitionsA_5(game: &mut Game) {
    kSamus_HandleTransitionsA_5[ram_word!(game, input_to_pose_calc) as uint8 as usize]
        .expect("non-null function pointer")(game);
}
pub unsafe fn Samus_HandleTransitionsA_5_4(game: &mut Game) {
    set_ram_word!(game.samus_y_subspeed = 0 as uint16);
    set_ram_word!(game.samus_y_speed = 0 as uint16);
    set_ram_word!(game.UNUSED_word_7E0B1A = 0 as uint16);
    set_ram_word!(game.samus_y_dir = 2 as uint16);
}
pub unsafe fn Samus_HandleTransitionsA_5_2(game: &mut Game) {
    if ram_word!(game, samus_y_dir) != 1u16 {
        set_ram_word!(game.used_for_ball_bounce_on_landing = 0 as uint16);
        set_ram_word!(game.samus_y_subspeed = 0 as uint16);
        set_ram_word!(game.samus_y_speed = 0 as uint16);
        set_ram_word!(game.samus_is_falling_flag = 1 as uint16);
        set_ram_word!(game.samus_y_dir = 2 as uint16);
    }
    set_ram_word!(game.UNUSED_word_7E0A18 = 0 as uint16);
}
pub fn sub_91EFC3(game: &mut Game) {}
pub fn sub_91EFC3_rv(game: &mut Game) -> uint8 {
    0 as uint8
}
static kSamus_HandleTransitionsA_5_1: [Option<Func_U8>; 6] = unsafe {
    [
        Some(Samus_HandleTransitionsA_5_1_0 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_MorphBallBounceNoSpringballTrans as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsA_5_1_2 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_MorphBallBounceSpringballTrans as unsafe fn(&mut Game) -> uint8),
        Some(sub_91EFC3_rv as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsA_5_1_5 as unsafe fn(&mut Game) -> uint8),
    ]
};
pub unsafe fn Samus_HandleTransitionsA_5_1(game: &mut Game) {
    HandleLandingSoundEffectsAndGfx(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xdc6) as *mut uint16 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1_usize) as isize,
    ) == 4u8
    {
        SamusFunc_F1D3(game);
    } else if kSamus_HandleTransitionsA_5_1[*((&raw mut game.g_ram as *mut uint8).offset(0xdc6)
        as *mut uint16 as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        ) as usize]
        .expect("non-null function pointer")(game) as i32
        & 1 as i32
        == 0
    {
        set_ram_word!(game.UNUSED_word_7E0A18 = 0 as uint16);
        set_ram_word!(game.samus_x_accel_mode = 0 as uint16);
        set_ram_word!(game.samus_x_base_speed = 0 as uint16);
        set_ram_word!(game.samus_x_base_subspeed = 0 as uint16);
        SamusFunc_F1D3(game);
    }
}
pub unsafe fn HandleLandingSoundEffectsAndGfx(game: &mut Game) {
    if (*(&raw mut game.g_ram as *mut uint8).offset(0xa23) as i32
        == kMovementType_03_SpinJumping as i32
        || *(&raw mut game.g_ram as *mut uint8).offset(0xa23) as i32
            == kMovementType_14_WallJumping as i32)
        && ram_word!(game, cinematic_function) == 0
    {
        if ram_word!(game, samus_prev_pose) as i32 == kPose_81_FaceR_Screwattack as i32
            || ram_word!(game, samus_prev_pose) as i32 == kPose_82_FaceL_Screwattack as i32
        {
            QueueSfx1_Max6(0x34 as uint16, game);
        } else {
            QueueSfx1_Max6(0x32 as uint16, game);
        }
    }
    if ram_word!(game, samus_y_speed) != 0u16
        && ram_word!(game, samus_y_speed) as i32 - 5 as i32 & 0x8000 as i32 == 0
    {
        if ram_word!(game, cinematic_function) == 0 {
            QueueSfx3_Max6(4 as uint16, game);
        }
    } else {
        if ram_word!(game, samus_y_subspeed) == 0 {
            return;
        }
        if ram_word!(game, cinematic_function) == 0 {
            QueueSfx3_Max6(5 as uint16, game);
        }
    }
    HandleLandingGraphics(game);
}
static off_91F0AE: [Option<Func_V>; 8] = unsafe {
    [
        Some(HandleLandingGraphics_Crateria as unsafe fn(&mut Game) -> ()),
        Some(HandleLandingGraphics_Brinstar as unsafe fn(&mut Game) -> ()),
        Some(HandleLandingGraphics_Norfair as unsafe fn(&mut Game) -> ()),
        Some(HandleLandingGraphics_Norfair as unsafe fn(&mut Game) -> ()),
        Some(HandleLandingGraphics_Maridia as unsafe fn(&mut Game) -> ()),
        Some(HandleLandingGraphics_Tourian as unsafe fn(&mut Game) -> ()),
        Some(HandleLandingGraphics_Ceres as unsafe fn(&mut Game) -> ()),
        Some(HandleLandingGraphics_Ceres as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn HandleLandingGraphics(game: &mut Game) {
    off_91F0AE[ram_word!(game, area_index) as usize].expect("non-null function pointer")(game);
}
pub unsafe fn HandleLandingGraphics_Ceres(game: &mut Game) {
    set_ram_word!(game.atmospheric_gfx_frame_and_type[2] = 0 as uint16);
    set_ram_word!(game.atmospheric_gfx_frame_and_type[3] = 0 as uint16);
}
static g_byte_91F0F3: [uint8; 17] = [1, 0, 0, 0, 0, 2, 0, 4, 0, 4, 4, 4, 4, 0, 4, 0, 0];
pub unsafe fn HandleLandingGraphics_Crateria(game: &mut Game) {
    let current_block: u64;
    if (ram_word!(game, cinematic_function) == 0) {
        if ram_word!(game, room_index) == 28u16 {
            HandleLandingGraphics_Norfair(game);
            return;
        }
        if (((ram_word!(game, room_index)) as i16).wrapping_sub(16i16) < 0i16) {
            if (g_byte_91F0F3[ram_word!(game, room_index) as usize] as i32 & 1 as i32) == 0i32 {
                if (g_byte_91F0F3[ram_word!(game, room_index) as usize] as i32 & 2 as i32) != 0i32 {
                    if ram_word!(game, samus_y_pos) as i32 - 944 as i32 & 0x8000 as i32 == 0 {
                        current_block = 18265313692946425332;
                    } else {
                        current_block = 4562832191328891483;
                    }
                } else if (g_byte_91F0F3[ram_word!(game, room_index) as usize] as i32 & 4 as i32)
                    != 0i32
                {
                    current_block = 18265313692946425332;
                } else {
                    current_block = 4562832191328891483;
                }
            } else if ram_word!(game, fx_type) != 10u16 {
                current_block = 4562832191328891483;
            } else {
                current_block = 18265313692946425332;
            }
            if current_block == 4562832191328891483 {
            } else {
                HandleLandingGraphics_Maridia(game);
                return;
            }
        }
    }
    HandleLandingGraphics_Ceres(game);
}
pub unsafe fn HandleLandingGraphics_Maridia(game: &mut Game) {
    let bottom: uint16 = Samus_GetBottom_R18(game);
    if liquid_at_height(game, bottom) != LiquidAtHeight::None {
        return;
    }
    set_ram_word!(game.atmospheric_gfx_frame_and_type[2] = 256 as uint16);
    set_ram_word!(game.atmospheric_gfx_frame_and_type[3] = 256 as uint16);
    set_ram_word!(game.atmospheric_gfx_anim_timer[2] = 3 as uint16);
    set_ram_word!(game.atmospheric_gfx_anim_timer[3] = 3 as uint16);
    set_ram_word!(
        game.atmospheric_gfx_x_pos[2] = (ram_word!(game, samus_x_pos)).wrapping_add(4u16)
    );
    set_ram_word!(
        game.atmospheric_gfx_x_pos[3] = (ram_word!(game, samus_x_pos)).wrapping_sub(3u16)
    );
    set_ram_word!(game.atmospheric_gfx_y_pos[2] = (bottom).wrapping_sub(4u16));
    set_ram_word!(game.atmospheric_gfx_y_pos[3] = (bottom).wrapping_sub(4u16));
}
pub unsafe fn HandleLandingGraphics_Norfair(game: &mut Game) {
    let bottom: uint16 = Samus_GetBottom_R18(game);
    if liquid_at_height(game, bottom) != LiquidAtHeight::None {
        return;
    }
    set_ram_word!(game.atmospheric_gfx_frame_and_type[2] = 1536 as uint16);
    set_ram_word!(game.atmospheric_gfx_frame_and_type[3] = 1536 as uint16);
    set_ram_word!(game.atmospheric_gfx_anim_timer[2] = 3 as uint16);
    set_ram_word!(game.atmospheric_gfx_anim_timer[3] = 3 as uint16);
    set_ram_word!(
        game.atmospheric_gfx_x_pos[2] = (ram_word!(game, samus_x_pos)).wrapping_add(8u16)
    );
    set_ram_word!(
        game.atmospheric_gfx_x_pos[3] = (ram_word!(game, samus_x_pos)).wrapping_sub(8u16)
    );
    set_ram_word!(game.atmospheric_gfx_y_pos[2] = bottom);
    set_ram_word!(game.atmospheric_gfx_y_pos[3] = bottom);
}
pub unsafe fn HandleLandingGraphics_Brinstar(game: &mut Game) {
    if ram_word!(game, room_index) == 8u16 {
        HandleLandingGraphics_Norfair(game);
    } else {
        HandleLandingGraphics_Tourian(game);
    };
}
pub unsafe fn HandleLandingGraphics_Tourian(game: &mut Game) {
    if ram_word!(game, room_index) as i32 - 5 as i32 & 0x8000 as i32 == 0
        && (ram_word!(game, room_index) as i32 - 9 as i32 & 0x8000 as i32 != 0
            || ram_word!(game, room_index) == 11u16)
    {
        HandleLandingGraphics_Norfair(game);
    } else {
        set_ram_word!(game.atmospheric_gfx_frame_and_type[2] = 0 as uint16);
        set_ram_word!(game.atmospheric_gfx_frame_and_type[3] = 0 as uint16);
    };
}
pub unsafe fn SamusFunc_F1D3(game: &mut Game) {
    set_ram_word!(game.samus_is_falling_flag = 0 as uint16);
    set_ram_word!(game.UNUSED_word_7E0B1A = 0 as uint16);
    set_ram_word!(game.UNUSED_word_7E0B2A = 0 as uint16);
    set_ram_word!(game.samus_y_subspeed = 0 as uint16);
    set_ram_word!(game.samus_y_speed = 0 as uint16);
    set_ram_word!(game.samus_y_dir = 0 as uint16);
    set_ram_word!(game.UNUSED_word_7E0B38 = 0 as uint16);
    set_ram_word!(game.used_for_ball_bounce_on_landing = 0 as uint16);
}
pub unsafe fn Samus_HandleTransitionsA_5_1_0(game: &mut Game) -> uint8 {
    if ram_word!(game, samus_input_handler) as i32 != 0x90e91d as i32 & 0xffff as i32 {
        set_ram_word!(game.samus_input_handler = ((0x90e926i32) as u16));
    }
    0 as uint8
}
static g_word_909EB5: uint16 = 1 as uint16;
static g_word_909EB7: uint16 = 0 as uint16;
pub unsafe fn Samus_MorphBallBounceNoSpringballTrans(game: &mut Game) -> uint8 {
    let mut v0: int16 = 0;
    v0 = (2i16).wrapping_mul((ram_word!(game, used_for_ball_bounce_on_landing)) as i16);
    if 2 as i32 * ram_word!(game, used_for_ball_bounce_on_landing) as i32 != 0 {
        if v0 == 2i16 {
            update_ram_word!(game.used_for_ball_bounce_on_landing, wrapping_add(1));
            set_ram_word!(game.samus_y_dir = 1 as uint16);
            set_ram_word!(game.samus_y_subspeed = g_word_909EB7);
            set_ram_word!(game.samus_y_speed = (g_word_909EB5).wrapping_sub(1u16));
            return 1 as uint8;
        }
        if v0 != 4i16 {
            Unreachable(game);
        }
    } else if ram_word!(game, samus_y_speed) as i32 - 3 as i32 & 0x8000 as i32 == 0 {
        update_ram_word!(game.used_for_ball_bounce_on_landing, wrapping_add(1));
        set_ram_word!(game.samus_y_dir = 1 as uint16);
        set_ram_word!(game.samus_y_subspeed = g_word_909EB7);
        set_ram_word!(game.samus_y_speed = g_word_909EB5);
        return 1 as uint8;
    }
    set_ram_word!(game.used_for_ball_bounce_on_landing = 0 as uint16);
    set_ram_word!(game.samus_y_dir = 0 as uint16);
    set_ram_word!(game.samus_y_subspeed = 0 as uint16);
    set_ram_word!(game.samus_y_speed = 0 as uint16);
    0 as uint8
}
pub unsafe fn Samus_HandleTransitionsA_5_1_2(game: &mut Game) -> uint8 {
    set_ram_word!(game.used_for_ball_bounce_on_landing = 0 as uint16);
    set_ram_word!(game.enable_horiz_slope_coll = 3 as uint16);
    0 as uint8
}
pub unsafe fn Samus_MorphBallBounceSpringballTrans(game: &mut Game) -> uint8 {
    let mut v1: int16 = 0;
    if (ram_word!(game, button_config_jump_a) as i32 & ram_word!(game, joypad1_lastkeys) as i32)
        != 0i32
    {
        set_ram_word!(game.used_for_ball_bounce_on_landing = 0 as uint16);
        Samus_InitJump(game);
        return 1 as uint8;
    }
    v1 = (2i16).wrapping_mul((ram_word!(game, used_for_ball_bounce_on_landing) as uint8) as i16);
    if v1 != 0 {
        if v1 == 2i16 {
            set_ram_word!(game.used_for_ball_bounce_on_landing = 1538 as uint16);
            set_ram_word!(game.samus_y_dir = 1 as uint16);
            set_ram_word!(game.samus_y_subspeed = g_word_909EB7);
            set_ram_word!(game.samus_y_speed = (g_word_909EB5).wrapping_sub(1u16));
            return 1 as uint8;
        }
        if v1 != 4i16 {
            Unreachable(game);
        }
    } else if ram_word!(game, samus_y_speed) as i32 - 3 as i32 & 0x8000 as i32 == 0 {
        set_ram_word!(game.used_for_ball_bounce_on_landing = 1537 as uint16);
        set_ram_word!(game.samus_y_dir = 1 as uint16);
        set_ram_word!(game.samus_y_subspeed = g_word_909EB7);
        set_ram_word!(game.samus_y_speed = g_word_909EB5);
        return 1 as uint8;
    }
    set_ram_word!(game.used_for_ball_bounce_on_landing = 0 as uint16);
    set_ram_word!(game.samus_y_dir = 0 as uint16);
    set_ram_word!(game.samus_y_subspeed = 0 as uint16);
    set_ram_word!(game.samus_y_speed = 0 as uint16);
    0 as uint8
}
pub unsafe fn Samus_HandleTransitionsA_5_1_5(game: &mut Game) -> uint8 {
    set_ram_word!(game.used_for_ball_bounce_on_landing = 0 as uint16);
    0 as uint8
}
pub unsafe fn Samus_HandleTransitionsA_5_5(game: &mut Game) {
    set_ram_word!(game.samus_x_accel_mode = 0 as uint16);
    set_ram_word!(game.samus_collides_with_solid_enemy = 0 as uint16);
    set_ram_word!(game.samus_is_falling_flag = 0 as uint16);
    set_ram_word!(game.UNUSED_word_7E0B1A = 0 as uint16);
    set_ram_word!(game.samus_x_base_speed = 0 as uint16);
    set_ram_word!(game.samus_x_base_subspeed = 0 as uint16);
    set_ram_word!(game.UNUSED_word_7E0A18 = 0 as uint16);
    QueueSfx3_Max6(5 as uint16, game);
}
pub unsafe fn Samus_HandleTransitionsA_5_6(game: &mut Game) {
    if ram_word!(game, samus_collides_with_solid_enemy) != 0
        && *(&raw mut game.g_ram as *mut uint8).offset(0xa23) == 9u8
    {
        set_ram_word!(
            game.enable_horiz_slope_coll =
                (*(&raw mut game.g_ram as *mut uint8).offset(0xa1e) != 4u8) as uint16
        );
        set_ram_word!(game.UNUSED_word_7E0A18 = 0 as uint16);
    }
}
pub unsafe fn Samus_HandleTransitionsC_1(game: &mut Game) {
    set_ram_word!(game.knockback_dir = 0 as uint16);
    set_ram_word!(game.samus_movement_handler = ((0x90a337i32) as u16));
    set_ram_word!(game.used_for_ball_bounce_on_landing = 0 as uint16);
    set_ram_word!(game.samus_y_subspeed = 0 as uint16);
    set_ram_word!(game.samus_y_speed = 0 as uint16);
    set_ram_word!(game.samus_is_falling_flag = 1 as uint16);
    set_ram_word!(game.samus_y_dir = 2 as uint16);
    Samus_AlignBottomWithPrevPose(game);
    if ram_word!(game, samus_input_handler) as i32 != 0x90e91d as i32 & 0xffff as i32 {
        set_ram_word!(game.samus_input_handler = ((0x90e913i32) as u16));
    }
}
pub unsafe fn Samus_HandleTransitionsC_2(game: &mut Game) {
    Samus_AlignBottomWithPrevPose(game);
    set_ram_word!(game.samus_movement_handler = ((0x90a337i32) as u16));
    if ram_word!(game, game_state) as i32 == kGameState_42_PlayingDemo as i32 {
        set_ram_word!(game.samus_input_handler = ((0x90e91di32) as u16));
    } else {
        set_ram_word!(game.samus_input_handler = ((0x90e913i32) as u16));
    };
}
pub unsafe fn Samus_HandleTransitionsC_3(game: &mut Game) {
    set_ram_word!(game.used_for_ball_bounce_on_landing = 0 as uint16);
    update_ram_word!(
        game.samus_anim_frame_timer,
        wrapping_add(ram_word!(game, samus_anim_frame_buffer))
    );
}
pub unsafe fn Samus_HandleTransitionsC_4(game: &mut Game) {
    set_ram_word!(game.samus_x_pos = (ram_word!(game, layer1_x_pos)).wrapping_add(128u16));
    set_ram_word!(game.samus_prev_x_pos = (ram_word!(game, layer1_x_pos)).wrapping_add(128u16));
    set_ram_word!(game.samus_y_pos = (ram_word!(game, layer1_y_pos)).wrapping_add(128u16));
    set_ram_word!(game.samus_prev_y_pos = (ram_word!(game, layer1_y_pos)).wrapping_add(128u16));
}
pub unsafe fn Samus_HandleTransitionsC_5(game: &mut Game) {
    update_ram_word!(game.samus_y_pos, wrapping_add(5u16));
    set_ram_word!(game.samus_prev_y_pos = ram_word!(game, samus_y_pos));
}
pub unsafe fn Samus_HandleTransitionsC_6(game: &mut Game) {
    HandleJumpTransition(game);
}
pub unsafe fn Samus_HandleTransitionsC_7(game: &mut Game) {
    let mut v0: int16 = 0;
    let mut v1: int16 = 0;
    v0 = ((ram_word!(game, samus_x_pos)) as i16)
        .wrapping_sub((ram_word!(game, samus_prev_x_pos)) as i16);
    if ((ram_word!(game, samus_x_pos)) as i16)
        .wrapping_sub((ram_word!(game, samus_prev_x_pos)) as i16)
        < 0i16
    {
        if v0 as i32 + 12 as i32 & 0x8000 as i32 != 0 {
            set_ram_word!(
                game.samus_prev_x_pos = (ram_word!(game, samus_x_pos)).wrapping_add(12u16)
            );
        }
    } else if v0 as i32 - 13 as i32 & 0x8000 as i32 == 0 {
        set_ram_word!(game.samus_prev_x_pos = (ram_word!(game, samus_x_pos)).wrapping_sub(12u16));
    }
    v1 = ((ram_word!(game, samus_y_pos)) as i16)
        .wrapping_sub((ram_word!(game, samus_prev_y_pos)) as i16);
    if ((ram_word!(game, samus_y_pos)) as i16)
        .wrapping_sub((ram_word!(game, samus_prev_y_pos)) as i16)
        < 0i16
    {
        if v1 as i32 + 12 as i32 & 0x8000 as i32 != 0 {
            set_ram_word!(
                game.samus_prev_y_pos = (ram_word!(game, samus_y_pos)).wrapping_add(12u16)
            );
        }
    } else if v1 as i32 - 13 as i32 & 0x8000 as i32 == 0 {
        set_ram_word!(game.samus_prev_y_pos = (ram_word!(game, samus_y_pos)).wrapping_sub(12u16));
    }
}
pub unsafe fn Samus_HandleTransitionsC_8(game: &mut Game) {
    Samus_HandleTransitionsC_1(game);
    Samus_HandleTransitionsC_3(game);
}
pub unsafe fn SamusFunc_F404(game: &mut Game) -> uint8 {
    let mut v1: SamusPose = kPose_00_FaceF_Powersuit;
    v1 = ram_word!(game, samus_pose) as SamusPose;
    if ram_word!(game, samus_pose) != ram_word!(game, samus_prev_pose) {
        HandleCollDueToChangedPose(game);
        SamusFunc_F433(game);
        HandleJumpTransition(game);
        Samus_SetAnimationFrameIfPoseChanged(game);
        set_ram_word!(game.samus_anim_frame_skip = 0 as uint16);
    }
    (v1 != ((ram_word!(game, samus_pose)) as u32)) as uint8
}
pub unsafe fn SamusFunc_F433(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa1e) as *mut uint16) =
        *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629)) as *const uint8
            as *mut SamusPoseParams)
            .offset(0))
        .pose_x_dir)
            .offset((8 as i32 * ram_word!(game, samus_pose) as i32) as isize)
            as *mut uint16);
    SamusFunc_F468(game);
    if (*(&raw mut game.g_ram as *mut uint8).offset(0xa23) as i32
        == kMovementType_03_SpinJumping as i32
        || *(&raw mut game.g_ram as *mut uint8).offset(0xa23) as i32
            == kMovementType_14_WallJumping as i32)
        && (ram_word!(game, equipped_items) as i32 & 8 as i32) != 0i32
    {
        Samus_LoadSuitPalette(game);
    }
}
static off_91F4A2: [Option<Func_U8>; 28] = unsafe {
    [
        Some(SamusFunc_F468_Standing as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_Running as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_NormalJump as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_SpinJump as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_MorphBall as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_Crouching as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_Falling as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_Unused as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_MorphBall as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_Unused as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_Unused as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_Unused as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_Unused as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_Unused as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_TurningAroundOnGround as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_CrouchTransEtc as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_Moonwalking as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_Springball as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_Springball as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_Springball as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_WallJumping as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_Unused as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_Unused as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_TurnAroundJumping as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_TurnAroundFalling as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_DamageBoost as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_Unused as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_Shinespark as unsafe fn(&mut Game) -> uint8),
    ]
};
pub unsafe fn SamusFunc_F468(game: &mut Game) {
    if off_91F4A2[*(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as usize]
        .expect("non-null function pointer")(game) as i32
        & 1 as i32
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1e) as *mut uint16) =
            *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629)) as *const uint8
                as *mut SamusPoseParams)
                .offset(0))
            .pose_x_dir)
                .offset((8 as i32 * ram_word!(game, samus_pose) as i32) as isize)
                as *mut uint16);
        if (*((&raw mut game.g_ram as *mut uint8).offset(0xa1e) as *mut uint16) as i32
            & 0xff00 as i32)
            == 3584i32
        {
            off_91F4A2[14 as i32 as usize].expect("non-null function pointer")(game);
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1e) as *mut uint16) =
                *((&raw mut (*(game.g_rom.offset(
                    crate::rtl::lorom_offset(0x91b629),
                ) as *const uint8 as *mut SamusPoseParams)
                    .offset(0))
                .pose_x_dir)
                    .offset(
                        (8 as i32
                            * ram_word!(game, samus_pose)
                                as i32) as isize,
                    ) as *mut uint16);
        }
    }
}
pub fn SamusFunc_F468_Unused(game: &mut Game) -> uint8 {
    0 as uint8
}
pub unsafe fn SamusFunc_F468_Standing(game: &mut Game) -> uint8 {
    if ((*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629)) as *const uint8
        as *mut SamusPoseParams)
        .offset(ram_word!(game, samus_pose) as isize))
    .direction_shots_fired
        == 0
        || (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629)) as *const uint8
            as *mut SamusPoseParams)
            .offset(ram_word!(game, samus_pose) as isize))
        .direction_shots_fired
            == 9u8)
        && ((*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629)) as *const uint8
            as *mut SamusPoseParams)
            .offset(ram_word!(game, samus_prev_pose) as isize))
        .direction_shots_fired
            == 0
            || (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629)) as *const uint8
                as *mut SamusPoseParams)
                .offset(ram_word!(game, samus_prev_pose) as isize))
            .direction_shots_fired
                == 9u8)
    {
        set_ram_word!(game.samus_anim_frame_skip = 1 as uint16);
    }
    0 as uint8
}
pub unsafe fn SamusFunc_F468_Running(game: &mut Game) -> uint8 {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa23) == 1u8 {
        set_ram_word!(game.samus_anim_frame_skip = ((0x918000i32) as u16));
    }
    if ram_word!(game, UNUSED_word_7E0DF8) == 0 {
        return 0 as uint8;
    }
    if ram_word!(game, samus_pose) as i32
        == kPose_44_FaceL_Turn_Crouch as i32 | kPose_01_FaceR_Normal as i32
    {
        set_ram_word!(game.samus_pose = kPose_25_FaceR_Turn_Stand as uint16);
        return 1 as uint8;
    }
    if ram_word!(game, samus_pose) as i32
        == kPose_44_FaceL_Turn_Crouch as i32 | kPose_02_FaceL_Normal as i32
    {
        set_ram_word!(game.samus_pose = kPose_26_FaceL_Turn_Stand as uint16);
        return 1 as uint8;
    }
    0 as uint8
}
pub unsafe fn SamusFunc_F468_NormalJump(game: &mut Game) -> uint8 {
    let mut current_block: u64;
    if ram_word!(game, samus_pose) as i32 != kPose_4E_FaceL_Jump_NoAim_NoMove_NoGun as i32 {
        if ram_word!(game, samus_pose) as i32 == kPose_4D_FaceR_Jump_NoAim_NoMove_NoGun as i32
            || ram_word!(game, samus_pose) as i32 == kPose_15_FaceR_Jump_AimU as i32
        {
            current_block = 10566234459686315488;
        } else if ram_word!(game, samus_pose) as i32 != kPose_16_FaceL_Jump_AimU as i32
            && ram_word!(game, samus_pose) as i32 != kPose_6A_FaceL_Jump_AimUL as i32
        {
            if ram_word!(game, samus_pose) as i32 != kPose_69_FaceR_Jump_AimUR as i32 {
                current_block = 10164294398894088689;
            } else {
                current_block = 10566234459686315488;
            }
        } else {
            current_block = 1917311967535052937;
        }
        match current_block {
            10164294398894088689 => {}
            1917311967535052937 => {}
            _ => {
                if ram_word!(game, samus_shine_timer) != 0 {
                    set_ram_word!(game.samus_pose = kPose_C7_FaceR_ShinesparkWindup_Vert as uint16);
                    current_block = 13140864810214251092;
                } else {
                    current_block = 10164294398894088689;
                }
            }
        }
    } else {
        current_block = 1917311967535052937;
    }
    if current_block == 1917311967535052937 {
        if ram_word!(game, samus_shine_timer) != 0 {
            set_ram_word!(game.samus_pose = kPose_C8_FaceL_ShinesparkWindup_Vert as uint16);
            current_block = 13140864810214251092;
        } else {
            current_block = 10164294398894088689;
        }
    }
    if current_block == 13140864810214251092 {
        Projectile_Func7_Shinespark(game);
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa23) as i32
            == kMovementType_02_NormalJumping as i32
        {
            set_ram_word!(
                game.samus_prev_y_pos = update_ram_word!(game.samus_y_pos, wrapping_sub(1), reread)
            );
        }
        1 as uint8
    } else {
        if ram_word!(game, samus_x_extra_run_speed) != 0u16
            || ram_word!(game, samus_x_extra_run_subspeed) != 0u16
        {
            set_ram_word!(game.samus_x_accel_mode = 2 as uint16);
        } else {
            set_ram_word!(game.samus_x_accel_mode = 0 as uint16);
        }
        if (ram_word!(game, samus_pose) as i32 == kPose_15_FaceR_Jump_AimU as i32
            || ram_word!(game, samus_pose) as i32 == kPose_16_FaceL_Jump_AimU as i32)
            && (ram_word!(game, samus_prev_pose) as i32 == kPose_55_FaceR_Jumptrans_AimU as i32
                || ram_word!(game, samus_prev_pose) as i32 == kPose_56_FaceL_Jumptrans_AimU as i32)
        {
            set_ram_word!(game.samus_anim_frame_skip = 1 as uint16);
        }
        if (ram_word!(game, button_config_shoot_x) as i32 & ram_word!(game, joypad1_newkeys) as i32)
            != 0i32
        {
            set_ram_word!(
                game.new_projectile_direction_changed_pose =
                    (((*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629)) as *const uint8
                        as *mut SamusPoseParams)
                        .offset(ram_word!(game, samus_pose) as isize))
                    .direction_shots_fired) as u16)
                        | (0x8000u16)
            );
        }
        0 as uint8
    }
}
pub unsafe fn SamusFunc_F468_Crouching(game: &mut Game) -> uint8 {
    if (ram_word!(game, samus_pose) as i32 == kPose_85_FaceR_Crouch_AimU as i32
        || ram_word!(game, samus_pose) as i32 == kPose_86_FaceL_Crouch_AimU as i32)
        && (ram_word!(game, samus_prev_pose) as i32 == kPose_F1_FaceR_CrouchTrans_AimU as i32
            || ram_word!(game, samus_prev_pose) as i32 == kPose_F2_FaceL_CrouchTrans_AimU as i32)
    {
        set_ram_word!(game.samus_anim_frame_skip = 1 as uint16);
    }
    0 as uint8
}
pub unsafe fn SamusFunc_F468_Falling(game: &mut Game) -> uint8 {
    if ram_word!(game, samus_x_extra_run_speed) != 0u16
        || ram_word!(game, samus_x_extra_run_subspeed) != 0u16
    {
        set_ram_word!(game.samus_x_accel_mode = 2 as uint16);
    } else {
        set_ram_word!(game.samus_x_accel_mode = 0 as uint16);
    }
    0 as uint8
}
pub unsafe fn SamusFunc_F468_SpinJump(game: &mut Game) -> uint8 {
    let mut current_block: u64;
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa23) as i32
        == kMovementType_03_SpinJumping as i32
        || *(&raw mut game.g_ram as *mut uint8).offset(0xa23) as i32
            == kMovementType_14_WallJumping as i32
    {
        set_ram_word!(game.samus_anim_frame_skip = 1 as uint16);
        if (*(&raw mut game.g_ram as *mut uint8).offset(0xa22) as i32 & 0xf as i32) == 8i32 {
            if *((&raw mut game.g_ram as *mut uint8).offset(0xa1e) as *mut uint16) != 772u16 {
                current_block = 12119026790781926931;
            } else {
                current_block = 820271813250567934;
            }
        } else if (*(&raw mut game.g_ram as *mut uint8).offset(0xa22) as i32 & 0xf as i32) != 4i32
            || *((&raw mut game.g_ram as *mut uint8).offset(0xa1e) as *mut uint16) != 776u16
        {
            current_block = 12119026790781926931;
        } else {
            current_block = 820271813250567934;
        }
        if current_block == 12119026790781926931 {
        } else {
            AddToHiLo(
                (&raw mut game.g_ram as *mut uint8).offset(0xb46) as *mut uint16,
                (&raw mut game.g_ram as *mut uint8).offset(0xb48) as *mut uint16,
                (ram_word!(game, samus_x_extra_run_speed) as uint32) << 16 as i32
                    | ram_word!(game, samus_x_extra_run_subspeed) as uint32,
            );
            Samus_CancelSpeedBoost(game);
            SetHiLo(
                (&raw mut game.g_ram as *mut uint8).offset(0xb42) as *mut uint16,
                (&raw mut game.g_ram as *mut uint8).offset(0xb44) as *mut uint16,
                0 as uint32,
            );
            set_ram_word!(game.samus_x_accel_mode = 1 as uint16);
        }
    }
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8 {
        if (ram_word!(game, equipped_items) as i32 & 0x20 as i32) == 0i32 {
            let r20: uint16 = Samus_GetTop_R20(game);
            if liquid_at_height(game, r20) != LiquidAtHeight::None {
                return 0 as uint8;
            }
        }
        if (ram_word!(game, equipped_items) as i32 & 8 as i32) != 0i32 {
            set_ram_word!(game.samus_pose = kPose_82_FaceL_Screwattack as uint16);
            current_block = 131178307184372633;
        } else {
            if (ram_word!(game, equipped_items) as i32 & 0x200 as i32) != 0i32 {
                QueueSfx1_Max6(0x3e as uint16, game);
                set_ram_word!(game.samus_pose = kPose_1C_FaceL_SpaceJump as uint16);
                return 0 as uint8;
            }
            if ram_word!(game, samus_anim_frame_skip) == 0
                && ram_word!(game, cinematic_function) == 0
            {
                current_block = 807399161326926732;
            } else {
                return 0 as uint8;
            }
        }
    } else {
        if (ram_word!(game, equipped_items) as i32 & 0x20 as i32) == 0i32 {
            let r20_0: uint16 = Samus_GetTop_R20(game);
            if liquid_at_height(game, r20_0) != LiquidAtHeight::None {
                return 0 as uint8;
            }
        }
        if (ram_word!(game, equipped_items) as i32 & 8 as i32) != 0i32 {
            set_ram_word!(game.samus_pose = kPose_81_FaceR_Screwattack as uint16);
            current_block = 131178307184372633;
        } else {
            if (ram_word!(game, equipped_items) as i32 & 0x200 as i32) == 0i32 {
                if ram_word!(game, samus_anim_frame_skip) != 0u16
                    || ram_word!(game, cinematic_function) != 0u16
                {
                    return 0 as uint8;
                }
            } else {
                QueueSfx1_Max6(0x3e as uint16, game);
                set_ram_word!(game.samus_pose = kPose_1B_FaceR_SpaceJump as uint16);
                return 0 as uint8;
            }
            current_block = 807399161326926732;
        }
    }
    if current_block == 807399161326926732 {
        QueueSfx1_Max6(0x31 as uint16, game);
        0 as uint8
    } else {
        if ram_word!(game, samus_anim_frame_skip) == 0 {
            QueueSfx1_Max6(0x33 as uint16, game);
        }
        0 as uint8
    }
}
static off_91F790: [Option<Func_U8>; 12] = unsafe {
    [
        Some(Samus_CrouchTrans as unsafe fn(&mut Game) -> uint8),
        Some(Samus_CrouchTrans as unsafe fn(&mut Game) -> uint8),
        Some(Samus_MorphTrans as unsafe fn(&mut Game) -> uint8),
        Some(Samus_MorphTrans as unsafe fn(&mut Game) -> uint8),
        Some(MaybeUnused_sub_91F7F4 as unsafe fn(&mut Game) -> uint8),
        Some(MaybeUnused_sub_91F840 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_StandOrUnmorphTrans as unsafe fn(&mut Game) -> uint8),
        Some(Samus_StandOrUnmorphTrans as unsafe fn(&mut Game) -> uint8),
        Some(Samus_StandOrUnmorphTrans as unsafe fn(&mut Game) -> uint8),
        Some(Samus_StandOrUnmorphTrans as unsafe fn(&mut Game) -> uint8),
        Some(Samus_StandOrUnmorphTrans as unsafe fn(&mut Game) -> uint8),
        Some(Samus_StandOrUnmorphTrans as unsafe fn(&mut Game) -> uint8),
    ]
};
static off_91F7A8: [Option<Func_U8>; 4] = unsafe {
    [
        Some(Samus_MorphTrans as unsafe fn(&mut Game) -> uint8),
        Some(Samus_MorphTrans as unsafe fn(&mut Game) -> uint8),
        Some(Samus_StandOrUnmorphTrans as unsafe fn(&mut Game) -> uint8),
        Some(Samus_StandOrUnmorphTrans as unsafe fn(&mut Game) -> uint8),
    ]
};
pub unsafe fn SamusFunc_F468_CrouchTransEtc(game: &mut Game) -> uint8 {
    let mut v0: uint16 = 0;
    's_43: {
        if ram_word!(game, samus_pose) as i32 - kPose_F1_FaceR_CrouchTrans_AimU as i32
            & 0x8000 as i32
            != 0
        {
            if ram_word!(game, samus_pose) as i32 - kPose_DB as i32 & 0x8000 as i32 != 0 {
                v0 = (2 as i32 * (ram_word!(game, samus_pose) as i32 - 53 as i32)) as uint16;
            } else {
                return off_91F7A8[(ram_word!(game, samus_pose) as i32 - 219 as i32) as usize]
                    .expect("non-null function pointer")(game);
            }
        } else if ram_word!(game, samus_pose) as i32 - kPose_F7_FaceR_StandTrans_AimU as i32
            & 0x8000 as i32
            != 0
        {
            v0 = 0 as uint16;
        } else {
            set_ram_word!(game.samus_momentum_routine_index = 7 as uint16);
            break 's_43;
        }
        set_ram_word!(game.samus_momentum_routine_index = 7 as uint16);
        return off_91F790[(v0 as i32 >> 1 as i32) as usize].expect("non-null function pointer")(
            game,
        );
    }
    0 as uint8
}
pub unsafe fn Samus_CrouchTrans(game: &mut Game) -> uint8 {
    if (ram_word!(game, speed_boost_counter) as i32 & 0xff00 as i32) - 1024 as i32 & 0x8000 as i32
        == 0
    {
        set_ram_word!(game.samus_shine_timer = 180 as uint16);
        set_ram_word!(game.timer_for_shine_timer = 1 as uint16);
        set_ram_word!(game.special_samus_palette_frame = 0 as uint16);
    }
    0 as uint8
}
pub fn Samus_StandOrUnmorphTrans(game: &mut Game) -> uint8 {
    0 as uint8
}
pub unsafe fn Samus_MorphTrans(game: &mut Game) -> uint8 {
    if (ram_word!(game, equipped_items) as i32 & 4 as i32) != 0i32 {
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa23) as i32
            == kMovementType_03_SpinJumping as i32
        {
            set_ram_word!(game.samus_x_accel_mode = 2 as uint16);
        }
        set_ram_word!(game.bomb_spread_charge_timeout_counter = 0 as uint16);
        0 as uint8
    } else {
        set_ram_word!(game.samus_pose = ram_word!(game, samus_prev_pose));
        1 as uint8
    }
}
pub unsafe fn MaybeUnused_sub_91F7F4(game: &mut Game) -> uint8 {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa23) as i32
        == kMovementType_08_MorphBallFalling as i32
        || *(&raw mut game.g_ram as *mut uint8).offset(0xa23) as i32
            == kMovementType_13_SpringBallFalling as i32
    {
        if (ram_word!(game, equipped_items) as i32 & 2 as i32) != 0i32 {
            set_ram_word!(game.samus_pose = kPose_7D_FaceR_Springball_Fall as uint16);
        } else {
            set_ram_word!(game.samus_pose = kPose_31_FaceR_Morphball_Air as uint16);
        }
    } else if (ram_word!(game, equipped_items) as i32 & 2 as i32) != 0i32 {
        set_ram_word!(game.samus_pose = kPose_79_FaceR_Springball_Ground as uint16);
    } else {
        set_ram_word!(game.samus_pose = kPose_1D_FaceR_Morphball_Ground as uint16);
    }
    1 as uint8
}
pub unsafe fn MaybeUnused_sub_91F840(game: &mut Game) -> uint8 {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa23) as i32
        == kMovementType_08_MorphBallFalling as i32
        || *(&raw mut game.g_ram as *mut uint8).offset(0xa23) as i32
            == kMovementType_13_SpringBallFalling as i32
    {
        if (ram_word!(game, equipped_items) as i32 & 2 as i32) != 0i32 {
            set_ram_word!(game.samus_pose = kPose_7E_FaceL_Springball_Fall as uint16);
        } else {
            set_ram_word!(game.samus_pose = kPose_32_FaceL_Morphball_Air as uint16);
        }
    } else if (ram_word!(game, equipped_items) as i32 & 2 as i32) != 0i32 {
        set_ram_word!(game.samus_pose = kPose_7A_FaceL_Springball_Ground as uint16);
    } else {
        set_ram_word!(game.samus_pose = kPose_41_FaceL_Morphball_Ground as uint16);
    }
    1 as uint8
}
pub unsafe fn SamusFunc_F468_Moonwalking(game: &mut Game) -> uint8 {
    if ram_word!(game, moonwalk_flag) != 0 {
        return 0 as uint8;
    }
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8 {
        set_ram_word!(game.samus_pose = kPose_25_FaceR_Turn_Stand as uint16);
    } else {
        set_ram_word!(game.samus_pose = kPose_26_FaceL_Turn_Stand as uint16);
    }
    1 as uint8
}
pub unsafe fn SamusFunc_F468_DamageBoost(game: &mut Game) -> uint8 {
    SamusFunc_F468_DamageBoost_(game)
}
pub unsafe fn MaybeUnused_sub_91F8B0(game: &mut Game) -> uint8 {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8 {
        set_ram_word!(game.samus_pose = kPose_54_FaceL_Knockback as uint16);
    } else {
        set_ram_word!(game.samus_pose = kPose_53_FaceR_Knockback as uint16);
    }
    1 as uint8
}
pub unsafe fn SamusFunc_F468_DamageBoost_(game: &mut Game) -> uint8 {
    set_ram_word!(game.samus_movement_handler = ((0x90a337i32) as u16));
    0 as uint8
}
pub unsafe fn SamusFunc_F468_TurningAroundOnGround(game: &mut Game) -> uint8 {
    if ram_word!(game, samus_prev_pose) != 0u16
        && ram_word!(game, samus_prev_pose) as i32 != kPose_9B_FaceF_VariaGravitySuit as i32
    {
        let v0: uint16 = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629)) as *const uint8
            as *mut SamusPoseParams)
            .offset(ram_word!(game, samus_prev_pose) as isize))
        .direction_shots_fired as uint16;
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa23) as i32
            == kMovementType_10_Moonwalking as i32
        {
            set_ram_word!(game.new_projectile_direction_changed_pose = (v0) | (0x100u16));
            if (ram_word!(game, button_config_jump_a) as i32
                & ram_word!(game, joypad1_lastkeys) as i32)
                != 0i32
            {
                set_ram_word!(
                    game.samus_pose = *(game.g_rom.offset(crate::rtl::lorom_offset(0x91f9ea))
                        as *const uint8 as *mut uint8)
                        .offset(v0 as isize) as uint16
                );
            } else {
                set_ram_word!(
                    game.samus_pose = *(game.g_rom.offset(crate::rtl::lorom_offset(0x91f9c2))
                        as *const uint8 as *mut uint8)
                        .offset(v0 as isize) as uint16
                );
            }
        } else if *(&raw mut game.g_ram as *mut uint8).offset(0xa23) == 5u8 {
            set_ram_word!(
                game.samus_pose = *(game.g_rom.offset(crate::rtl::lorom_offset(0x91f9cc))
                    as *const uint8 as *mut uint8)
                    .offset(v0 as isize) as uint16
            );
        } else {
            set_ram_word!(
                game.samus_pose = *(game.g_rom.offset(crate::rtl::lorom_offset(0x91f9c2))
                    as *const uint8 as *mut uint8)
                    .offset(v0 as isize) as uint16
            );
        }
    }
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0xb46) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0xb48) as *mut uint16,
        (ram_word!(game, samus_x_extra_run_speed) as uint32) << 16 as i32
            | ram_word!(game, samus_x_extra_run_subspeed) as uint32,
    );
    SetHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0xb42) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0xb44) as *mut uint16,
        0 as uint32,
    );
    set_ram_word!(game.samus_x_accel_mode = 1 as uint16);
    1 as uint8
}
pub unsafe fn SamusFunc_F468_TurnAroundJumping(game: &mut Game) -> uint8 {
    set_ram_word!(
        game.samus_pose = *(game.g_rom.offset(crate::rtl::lorom_offset(0x91f9d6)) as *const uint8
            as *mut uint8)
            .offset(
                (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629)) as *const uint8
                    as *mut SamusPoseParams)
                    .offset(ram_word!(game, samus_prev_pose) as isize))
                .direction_shots_fired as isize,
            ) as uint16
    );
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0xb46) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0xb48) as *mut uint16,
        (ram_word!(game, samus_x_extra_run_speed) as uint32) << 16 as i32
            | ram_word!(game, samus_x_extra_run_subspeed) as uint32,
    );
    SetHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0xb42) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0xb44) as *mut uint16,
        0 as uint32,
    );
    set_ram_word!(game.samus_x_accel_mode = 1 as uint16);
    1 as uint8
}
pub unsafe fn SamusFunc_F468_TurnAroundFalling(game: &mut Game) -> uint8 {
    set_ram_word!(
        game.samus_pose = *(game.g_rom.offset(crate::rtl::lorom_offset(0x91f9e0)) as *const uint8
            as *mut uint8)
            .offset(
                (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629)) as *const uint8
                    as *mut SamusPoseParams)
                    .offset(ram_word!(game, samus_prev_pose) as isize))
                .direction_shots_fired as isize,
            ) as uint16
    );
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0xb46) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0xb48) as *mut uint16,
        (ram_word!(game, samus_x_extra_run_speed) as uint32) << 16 as i32
            | ram_word!(game, samus_x_extra_run_subspeed) as uint32,
    );
    SetHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0xb42) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0xb44) as *mut uint16,
        0 as uint32,
    );
    set_ram_word!(game.samus_x_accel_mode = 1 as uint16);
    1 as uint8
}
pub unsafe fn SamusFunc_F468_MorphBall(game: &mut Game) -> uint8 {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa23) as i32
        == kMovementType_04_MorphBallOnGround as i32
        || *(&raw mut game.g_ram as *mut uint8).offset(0xa23) as i32
            == kMovementType_08_MorphBallFalling as i32
    {
        set_ram_word!(game.samus_anim_frame_skip = ((0x918000i32) as u16));
    }
    SamusFunc_FA0A(game);
    0 as uint8
}
pub unsafe fn SamusFunc_FA0A(game: &mut Game) {
    let current_block: u64;
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa22) == 8u8 {
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) != 4u8 {
            return;
        }
        current_block = 17179679302217393232;
    } else if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 8u8 {
        current_block = 17179679302217393232;
    } else {
        current_block = 7351195479953500246;
    }
    if current_block == 17179679302217393232 {
        AddToHiLo(
            (&raw mut game.g_ram as *mut uint8).offset(0xb46) as *mut uint16,
            (&raw mut game.g_ram as *mut uint8).offset(0xb48) as *mut uint16,
            (ram_word!(game, samus_x_extra_run_speed) as uint32) << 16 as i32
                | ram_word!(game, samus_x_extra_run_subspeed) as uint32,
        );
        Samus_CancelSpeedBoost(game);
        SetHiLo(
            (&raw mut game.g_ram as *mut uint8).offset(0xb42) as *mut uint16,
            (&raw mut game.g_ram as *mut uint8).offset(0xb44) as *mut uint16,
            0 as uint32,
        );
        set_ram_word!(game.samus_x_accel_mode = 1 as uint16);
    };
}
pub unsafe fn SamusFunc_F468_Springball(game: &mut Game) -> uint8 {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa23) as i32
        == kMovementType_11_SpringBallOnGround as i32
        || *(&raw mut game.g_ram as *mut uint8).offset(0xa23) as i32
            == kMovementType_12_SpringBallInAir as i32
        || *(&raw mut game.g_ram as *mut uint8).offset(0xa23) as i32
            == kMovementType_13_SpringBallFalling as i32
    {
        set_ram_word!(game.samus_anim_frame_skip = ((0x918000i32) as u16));
    }
    SamusFunc_FA0A(game);
    0 as uint8
}
pub unsafe fn SamusFunc_F468_WallJumping(game: &mut Game) -> uint8 {
    let bottom: uint16 = Samus_GetBottom_R18(game);
    if liquid_at_height(game, bottom) != LiquidAtHeight::None {
        return 0 as uint8;
    }
    set_ram_word!(game.atmospheric_gfx_frame_and_type[3] = 1536 as uint16);
    set_ram_word!(game.atmospheric_gfx_anim_timer[3] = 3 as uint16);
    set_ram_word!(game.atmospheric_gfx_y_pos[3] = bottom);
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) != 8u8 {
        set_ram_word!(
            game.atmospheric_gfx_x_pos[3] = (ram_word!(game, samus_x_pos)).wrapping_add(6u16)
        );
        return 0 as uint8;
    }
    set_ram_word!(
        game.atmospheric_gfx_x_pos[3] = (ram_word!(game, samus_x_pos)).wrapping_sub(6u16)
    );
    0 as uint8
}
static kSamusFunc_F468_Shinespark: [uint16; 6] = [
    fnSamus_MoveHandler_Shinespark_Horiz as uint16,
    fnSamus_MoveHandler_Shinespark_Horiz as uint16,
    fnSamus_MoveHandlerVerticalShinespark as uint16,
    fnSamus_MoveHandlerVerticalShinespark as uint16,
    fnSamus_MoveHandler_Shinespark_Diag as uint16,
    fnSamus_MoveHandler_Shinespark_Diag as uint16,
];
pub unsafe fn SamusFunc_F468_Shinespark(game: &mut Game) -> uint8 {
    if ram_word!(game, samus_pose) as i32 - kPose_CF_FaceR_Ranintowall_AimUR as i32 & 0x8000 as i32
        != 0
    {
        set_ram_word!(
            game.samus_movement_handler = kSamusFunc_F468_Shinespark
                [(ram_word!(game, samus_pose) as i32 - 201 as i32) as usize]
        );
        set_ram_word!(game.samus_input_handler = ((0x90e90ei32) as u16));
        set_ram_word!(game.speed_echoes_index = 0 as uint16);
        set_ram_word!(game.speed_echo_xspeed[0] = 0 as uint16);
        set_ram_word!(game.speed_echo_xspeed[1] = 0 as uint16);
        set_ram_word!(game.speed_echo_xpos[0] = 0 as uint16);
        set_ram_word!(game.speed_echo_xpos[1] = 0 as uint16);
        QueueSfx3_Max9(0xf as uint16, game);
    }
    0 as uint8
}
static kSamusPhys_AnimDelayInWater: uint16 = 3 as uint16;
static kSamusPhys_AnimDelayInAcid: uint16 = 2 as uint16;
pub unsafe fn Samus_SetAnimationFrameIfPoseChanged(game: &mut Game) {
    let mut t: uint16 = 0;
    if (ram_word!(game, equipped_items) as i32 & 0x20 as i32) != 0i32 {
        t = ram_word!(game, samus_x_speed_divisor);
    } else {
        let r: uint16 = (ram_word!(game, samus_y_pos) as i32
            + (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629)) as *const uint8
                as *mut SamusPoseParams)
                .offset(ram_word!(game, samus_pose) as isize))
            .y_radius as i32
            - 1 as i32) as uint16;
        t = match liquid_at_height(game, r) {
            LiquidAtHeight::Water => kSamusPhys_AnimDelayInWater,
            LiquidAtHeight::LavaAcid => kSamusPhys_AnimDelayInAcid,
            LiquidAtHeight::None => {
                ram_word!(game, samus_x_speed_divisor)
            }
        };
    }
    if (ram_word!(game, samus_anim_frame_skip) as i32 & 0x8000 as i32) == 0i32
        && ram_word!(game, samus_pose) != ram_word!(game, samus_prev_pose)
    {
        set_ram_word!(game.samus_anim_frame = ram_word!(game, samus_anim_frame_skip));
        set_ram_word!(
            game.samus_anim_frame_timer = (t).wrapping_add(
                (*RomPtr_91(
                    (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b010)) as *const uint8
                        as *mut uint16)
                        .offset(ram_word!(game, samus_pose) as isize))
                    .wrapping_add(ram_word!(game, samus_anim_frame_skip)),
                    game,
                )) as u16,
            )
        );
    }
}
pub unsafe fn SamusFunc_EC80(game: &mut Game) {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa23) as i32 != kMovementType_06_Falling as i32
        && *(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as i32
            == kMovementType_06_Falling as i32
    {
        set_ram_word!(game.samus_y_subspeed = 0 as uint16);
        set_ram_word!(game.samus_y_speed = 0 as uint16);
        set_ram_word!(game.samus_y_dir = 2 as uint16);
    }
}
pub fn nullsub_24(game: &mut Game) {}
pub fn nullsub_25(game: &mut Game) {}
static kHandleJumpTrans: [Option<Func_V>; 28] = unsafe {
    [
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(HandleJumpTransition_NormalJump as unsafe fn(&mut Game) -> ()),
        Some(HandleJumpTransition_SpinJump as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(UNUSED_sub_91FC42 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(HandleJumpTransition_SpringBallInAir as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(HandleJumpTransition_WallJump as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_25 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn HandleJumpTransition(game: &mut Game) {
    kHandleJumpTrans[*(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as usize]
        .expect("non-null function pointer")(game);
}
pub unsafe fn HandleJumpTransition_WallJump(game: &mut Game) {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa23) as i32
        != kMovementType_14_WallJumping as i32
    {
        Samus_InitWallJump(game);
    }
}
pub unsafe fn HandleJumpTransition_SpringBallInAir(game: &mut Game) {
    if ram_word!(game, samus_pose) as i32 == kPose_7F_FaceR_Springball_Air as i32 {
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa23) as i32
            != kMovementType_11_SpringBallOnGround as i32
        {
            return;
        }
    } else if !(ram_word!(game, samus_pose) as i32 == kPose_80_FaceL_Springball_Air as i32
        && *(&raw mut game.g_ram as *mut uint8).offset(0xa23) as i32
            == kMovementType_11_SpringBallOnGround as i32)
    {
        return;
    }
    Samus_InitJump(game);
}
pub unsafe fn UNUSED_sub_91FC42(game: &mut Game) {
    if ram_word!(game, samus_pose) as i32
        == kPose_44_FaceL_Turn_Crouch as i32 | kPose_01_FaceR_Normal as i32 | 0x20 as i32
    {
        if ram_word!(game, samus_prev_pose) != 100u16 {
            return;
        }
    } else if !(ram_word!(game, samus_pose) as i32
        == kPose_44_FaceL_Turn_Crouch as i32 | kPose_02_FaceL_Normal as i32 | 0x20 as i32
        && ram_word!(game, samus_prev_pose) == 99u16)
    {
        return;
    }
    Samus_InitJump(game);
}
pub unsafe fn HandleJumpTransition_NormalJump(game: &mut Game) {
    if ram_word!(game, samus_pose) as i32 == kPose_4B_FaceR_Jumptrans as i32
        || ram_word!(game, samus_pose) as i32 == kPose_4C_FaceL_Jumptrans as i32
        || ram_word!(game, samus_pose) as i32 - kPose_55_FaceR_Jumptrans_AimU as i32 & 0x8000 as i32
            == 0
            && ram_word!(game, samus_pose) as i32 - kPose_5B as i32 & 0x8000 as i32 != 0
    {
        if ram_word!(game, samus_prev_pose) as i32 == kPose_27_FaceR_Crouch as i32
            || ram_word!(game, samus_prev_pose) as i32 == kPose_28_FaceL_Crouch as i32
        {
            update_ram_word!(game.samus_y_pos, wrapping_sub(10u16));
        }
        Samus_InitJump(game);
    }
}
pub unsafe fn HandleJumpTransition_SpinJump(game: &mut Game) {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa23) as i32
        != kMovementType_03_SpinJumping as i32
        && *(&raw mut game.g_ram as *mut uint8).offset(0xa23) as i32
            != kMovementType_14_WallJumping as i32
    {
        Samus_InitJump(game);
    }
}
pub unsafe fn Samus_Func20(game: &mut Game) {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f) == 14u8 {
        if ram_word!(game, samus_anim_frame) == 2u16
            && ram_word!(game, samus_anim_frame_timer) == 1u16
        {
            if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8 {
                if ram_word!(game, samus_pose) as i32 == kPose_25_FaceR_Turn_Stand as i32 {
                    set_ram_word!(game.samus_pose = kPose_D6_FaceL_Xray_Stand as uint16);
                } else {
                    set_ram_word!(game.samus_pose = kPose_DA_FaceL_Xray_Crouch as uint16);
                }
            } else if ram_word!(game, samus_pose) as i32 == kPose_26_FaceL_Turn_Stand as i32 {
                set_ram_word!(game.samus_pose = kPose_D5_FaceR_Xray_Stand as uint16);
            } else {
                set_ram_word!(game.samus_pose = kPose_D9_FaceR_Xray_Crouch as uint16);
            }
            SamusFunc_F433(game);
            Samus_SetAnimationFrameIfPoseChanged(game);
            set_ram_word!(game.samus_last_different_pose = ram_word!(game, samus_prev_pose));
            *((&raw mut game.g_ram as *mut uint8).offset(0xa26) as *mut uint16) =
                *((&raw mut game.g_ram as *mut uint8).offset(0xa22) as *mut uint16);
            set_ram_word!(game.samus_prev_pose = ram_word!(game, samus_pose));
            *((&raw mut game.g_ram as *mut uint8).offset(0xa22) as *mut uint16) =
                *((&raw mut game.g_ram as *mut uint8).offset(0xa1e) as *mut uint16);
        }
    } else {
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8 {
            if (ram_word!(game, button_config_right) as i32
                & ram_word!(game, joypad1_lastkeys) as i32)
                == 0i32
            {
                return;
            }
            set_ram_word!(game.xray_angle = (256u16).wrapping_sub(ram_word!(game, xray_angle)));
            if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f) == 5u8 {
                set_ram_word!(game.samus_pose = kPose_44_FaceL_Turn_Crouch as uint16);
            } else {
                set_ram_word!(game.samus_pose = kPose_26_FaceL_Turn_Stand as uint16);
            }
        } else {
            if (ram_word!(game, button_config_left) as i32
                & ram_word!(game, joypad1_lastkeys) as i32)
                == 0i32
            {
                return;
            }
            set_ram_word!(game.xray_angle = (256u16).wrapping_sub(ram_word!(game, xray_angle)));
            if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as i32
                == kMovementType_05_Crouching as i32
            {
                set_ram_word!(game.samus_pose = kPose_43_FaceR_Turn_Crouch as uint16);
            } else {
                set_ram_word!(game.samus_pose = kPose_25_FaceR_Turn_Stand as uint16);
            }
        }
        SamusFunc_F433(game);
        Samus_SetAnimationFrameIfPoseChanged(game);
        set_ram_word!(game.samus_last_different_pose = ram_word!(game, samus_prev_pose));
        *((&raw mut game.g_ram as *mut uint8).offset(0xa26) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xa22) as *mut uint16);
        set_ram_word!(game.samus_prev_pose = ram_word!(game, samus_pose));
        *((&raw mut game.g_ram as *mut uint8).offset(0xa22) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1e) as *mut uint16);
    };
}
static off_91FE8A: [Option<Func_U8>; 4] = unsafe {
    [
        Some(HandleCollDueToChangedPose_Solid_NoColl as unsafe fn(&mut Game) -> uint8),
        Some(HandleCollDueToChangedPose_Solid_CollAbove as unsafe fn(&mut Game) -> uint8),
        Some(HandleCollDueToChangedPose_Solid_CollBelow as unsafe fn(&mut Game) -> uint8),
        Some(HandleCollDueToChangedPose_Solid_CollBoth as unsafe fn(&mut Game) -> uint8),
    ]
};
static off_91FE92: [Option<Func_U8>; 4] = unsafe {
    [
        Some(HandleCollDueToChangedPose_Block_NoColl as unsafe fn(&mut Game) -> uint8),
        Some(HandleCollDueToChangedPose_Block_CollAbove as unsafe fn(&mut Game) -> uint8),
        Some(HandleCollDueToChangedPose_Block_CollBelow as unsafe fn(&mut Game) -> uint8),
        Some(HandleCollDueToChangedPose_Block_CollBoth as unsafe fn(&mut Game) -> uint8),
    ]
};
pub unsafe fn HandleCollDueToChangedPose(game: &mut Game) {
    let mut cres: CheckEnemyColl_Result = CheckSolidEnemyColl_Result {
        collision: 0,
        amt: 0,
    };
    let mut amt: int32 = 0;
    let mut v0: int16 = 0;
    if ram_word!(game, samus_pose) == 0
        || ram_word!(game, samus_pose) as i32 == kPose_9B_FaceF_VariaGravitySuit as i32
    {
        return;
    }
    set_ram_word!(game.solid_enemy_collision_flags = 0 as uint16);
    set_ram_word!(game.block_collision_flags = 0 as uint16);
    v0 = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629)) as *const uint8
        as *mut SamusPoseParams)
        .offset(ram_word!(game, samus_prev_pose) as isize))
    .y_radius as int16;
    if v0 as i32
        - (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629)) as *const uint8
            as *mut SamusPoseParams)
            .offset(ram_word!(game, samus_pose) as isize))
        .y_radius as i32
        & 0x8000 as i32
        == 0
    {
        return;
    }
    set_ram_word!(
        game.samus_y_radius = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
            as *const uint8 as *mut SamusPoseParams)
            .offset(ram_word!(game, samus_prev_pose) as isize))
        .y_radius as uint16
    );
    set_ram_word!(
        game.samus_y_radius_diff = (((*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
            as *const uint8 as *mut SamusPoseParams)
            .offset(ram_word!(game, samus_pose) as isize))
        .y_radius) as u16)
            .wrapping_sub((v0) as u16)
    );
    set_ram_word!(game.samus_collision_direction = 2 as uint16);
    cres = Samus_CheckSolidEnemyColl(
        (ram_word!(game, samus_y_radius_diff) as int16 as int32) << 16 as i32,
        game,
    );
    set_ram_word!(game.samus_collision_flag = cres.collision);
    amt = cres.amt;
    if ram_word!(game, samus_collision_flag) != 0 {
        set_ram_word!(game.solid_enemy_collision_flags = 1 as uint16);
    }
    set_ram_word!(game.samus_space_to_move_up_enemy = (amt >> 16 as i32) as uint16);
    set_ram_word!(game.samus_collision_direction = 3 as uint16);
    cres = Samus_CheckSolidEnemyColl(
        (ram_word!(game, samus_y_radius_diff) as int16 as int32) << 16 as i32,
        game,
    );
    set_ram_word!(game.samus_collision_flag = cres.collision);
    amt = cres.amt;
    if ram_word!(game, samus_collision_flag) != 0 {
        update_ram_word!(game.solid_enemy_collision_flags, |fresh11| (fresh11)
            | (2u16));
    }
    set_ram_word!(game.samus_space_to_move_down_enemy = (amt >> 16 as i32) as uint16);
    if off_91FE8A[ram_word!(game, solid_enemy_collision_flags) as usize]
        .expect("non-null function pointer")(game)
        != 0
    {
        set_ram_word!(game.samus_pose = ram_word!(game, samus_prev_pose));
        return;
    }
    amt = Samus_CollDetectChangedPose(
        (-(ram_word!(game, samus_y_radius_diff) as i32) as int16 as int32) << 16 as i32,
        game,
    );
    if ram_word!(game, samus_collision_flag) != 0 {
        set_ram_word!(game.block_collision_flags = 1 as uint16);
    }
    set_ram_word!(game.samus_space_to_move_up_blocks = (amt >> 16 as i32) as uint16);
    amt = Samus_CollDetectChangedPose(
        (ram_word!(game, samus_y_radius_diff) as int16 as int32) << 16 as i32,
        game,
    );
    if ram_word!(game, samus_collision_flag) != 0 {
        update_ram_word!(game.block_collision_flags, |fresh12| (fresh12) | (2u16));
    }
    set_ram_word!(game.samus_space_to_move_down_blocks = (amt >> 16 as i32) as uint16);
    if off_91FE92[ram_word!(game, block_collision_flags) as usize]
        .expect("non-null function pointer")(game)
        != 0
    {
        set_ram_word!(game.samus_pose = ram_word!(game, samus_prev_pose));
    }
}
pub fn HandleCollDueToChangedPose_Solid_NoColl(game: &mut Game) -> uint8 {
    0 as uint8
}
pub fn HandleCollDueToChangedPose_Solid_CollBoth(game: &mut Game) -> uint8 {
    0 as uint8
}
pub unsafe fn HandleCollDueToChangedPose_Solid_CollAbove(game: &mut Game) -> uint8 {
    let v1: uint16 = ram_word!(game, samus_y_radius);
    set_ram_word!(
        game.samus_y_radius = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
            as *const uint8 as *mut SamusPoseParams)
            .offset(ram_word!(game, samus_pose) as isize))
        .y_radius as uint16
    );
    set_ram_word!(game.samus_collision_direction = 3 as uint16);
    let cres: CheckEnemyColl_Result = Samus_CheckSolidEnemyColl(
        ((((ram_word!(game, samus_y_radius_diff)) as i16)
            .wrapping_sub((ram_word!(game, samus_space_to_move_up_enemy)) as i16))
            as int32)
            << 16 as i32,
        game,
    );
    set_ram_word!(game.samus_y_radius = v1);
    set_ram_word!(game.samus_collision_flag = cres.collision);
    if ram_word!(game, samus_collision_flag) != 0 {
        return 1 as uint8;
    }
    set_ram_word!(game.samus_space_to_move_up_enemy = (cres.amt >> 16 as i32) as uint16);
    0 as uint8
}
pub unsafe fn HandleCollDueToChangedPose_Solid_CollBelow(game: &mut Game) -> uint8 {
    let v1: uint16 = ram_word!(game, samus_y_radius);
    set_ram_word!(
        game.samus_y_radius = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
            as *const uint8 as *mut SamusPoseParams)
            .offset(ram_word!(game, samus_pose) as isize))
        .y_radius as uint16
    );
    set_ram_word!(game.samus_collision_direction = 2 as uint16);
    let cres: CheckEnemyColl_Result = Samus_CheckSolidEnemyColl(
        ((((ram_word!(game, samus_y_radius_diff)) as i16)
            .wrapping_sub((ram_word!(game, samus_space_to_move_down_enemy)) as i16))
            as int32)
            << 16 as i32,
        game,
    );
    set_ram_word!(game.samus_y_radius = v1);
    set_ram_word!(game.samus_collision_flag = cres.collision);
    if ram_word!(game, samus_collision_flag) != 0 {
        return 1 as uint8;
    }
    set_ram_word!(game.samus_space_to_move_down_enemy = (cres.amt >> 16 as i32) as uint16);
    0 as uint8
}
pub unsafe fn HandleCollDueToChangedPose_Block_CollAbove(game: &mut Game) -> uint8 {
    let amt: int32 = Samus_CollDetectChangedPose(
        ((((ram_word!(game, samus_y_radius_diff)) as i16)
            .wrapping_sub((ram_word!(game, samus_space_to_move_up_blocks)) as i16))
            as int32)
            << 16 as i32,
        game,
    );
    if ram_word!(game, samus_collision_flag) != 0 {
        return 1 as uint8;
    }
    if (ram_word!(game, solid_enemy_collision_flags) as i32 & 2 as i32) != 0i32 {
        return HandleCollDueToChangedPose_Block_CollBoth(game);
    }
    update_ram_word!(
        game.samus_y_pos,
        |fresh5| (fresh5 as i32 + (amt >> 16 as i32) as i32) as uint16
    );
    set_ram_word!(game.samus_prev_y_pos = ram_word!(game, samus_y_pos));
    0 as uint8
}
pub unsafe fn HandleCollDueToChangedPose_Block_CollBelow(game: &mut Game) -> uint8 {
    let amt: int32 = Samus_CollDetectChangedPose(
        ((((ram_word!(game, samus_space_to_move_down_blocks)) as i16)
            .wrapping_sub((ram_word!(game, samus_y_radius_diff)) as i16)) as int32)
            << 16 as i32,
        game,
    );
    if ram_word!(game, samus_collision_flag) != 0 {
        return 1 as uint8;
    }
    if (ram_word!(game, solid_enemy_collision_flags) as i32 & 1 as i32) != 0i32 {
        return HandleCollDueToChangedPose_Block_CollBoth(game);
    }
    update_ram_word!(
        game.samus_y_pos,
        |fresh7| (fresh7 as i32 - (amt >> 16 as i32) as i32) as uint16
    );
    set_ram_word!(game.samus_prev_y_pos = ram_word!(game, samus_y_pos));
    0 as uint8
}
pub unsafe fn HandleCollDueToChangedPose_Block_NoColl(game: &mut Game) -> uint8 {
    match ram_word!(game, solid_enemy_collision_flags) as i32 {
        0 => 0 as uint8,
        1 => {
            update_ram_word!(
                game.samus_y_pos,
                wrapping_add(ram_word!(game, samus_space_to_move_up_enemy))
            );
            set_ram_word!(game.samus_prev_y_pos = ram_word!(game, samus_y_pos));
            0 as uint8
        }
        2 => {
            update_ram_word!(
                game.samus_y_pos,
                wrapping_sub(ram_word!(game, samus_space_to_move_down_enemy))
            );
            set_ram_word!(game.samus_prev_y_pos = ram_word!(game, samus_y_pos));
            0 as uint8
        }
        3 => HandleCollDueToChangedPose_Block_CollBoth(game),
        _ => {
            Unreachable(game);
            0 as uint8
        }
    }
}
pub unsafe fn HandleCollDueToChangedPose_Block_CollBoth(game: &mut Game) -> uint8 {
    if ram_word!(game, samus_y_radius) as i32 - 8 as i32 & 0x8000 as i32 != 0 {
        return 1 as uint8;
    }
    set_ram_word!(
        game.samus_pose = (if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8 {
            kPose_28_FaceL_Crouch as i32
        } else {
            kPose_27_FaceR_Crouch as i32
        }) as uint16
    );
    let r18: uint16 = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629)) as *const uint8
        as *mut SamusPoseParams)
        .offset(ram_word!(game, samus_pose) as isize))
    .y_radius as uint16;
    if ram_word!(game, samus_y_radius) as i32 - r18 as i32 & 0x8000 as i32 != 0 {
        update_ram_word!(game.samus_y_pos, |fresh6| (fresh6 as i32
            + (ram_word!(game, samus_y_radius) as i32 - r18 as i32))
            as uint16);
        set_ram_word!(game.samus_prev_y_pos = ram_word!(game, samus_y_pos));
    }
    0 as uint8
}
