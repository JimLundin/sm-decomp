use crate::c_library::{__assert_fail, memcpy};
use crate::game::callback_tables::callback_table;
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
use crate::ram_map::ram_ptr;
use crate::ram_map::word;
use crate::rtl::{mov24, Mult8x8, RomPtr, Unreachable};
use crate::rtl::{AddToHiLo, SetHiLo};
pub use crate::types::{
    size_t, Func_U8, Func_V, LongPtr, SamusPose, SpawnHdmaObject_Args, VoidP, VramWriteEntry,
};
pub use crate::types::{SamusMovementType, SamusPoseParams};
use ::core::ffi::{c_char, c_long, c_void};
use ::core::mem::transmute;
use ::core::ptr::{null, null_mut};
crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C, packed)]
    pub struct DemoInputObject {
        pub ptr, pre_instr, instr_ptr: VoidP;
    }
}
crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C, packed)]
    pub struct DemoSetDef {
        pub items, missiles, super_missiles, power_bombs, health, beams, equipped_beams_: u16;
        pub demo_obj: VoidP;
    }
}
crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C, packed)]
    pub struct PoseEntry {
        pub new_input, cur_input, new_pose: u16;
    }
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct XrayBlockData {
    pub value: u16,
    pub addr: VoidP,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct SamusCrystalFlashPalTable {
    pub ptr: VoidP,
    pub timer: u16,
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
crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C)]
    pub struct XrayHdmaCtx {
        pub r22, r24, r26, r28, r30, r32, r18_avoid: u16;
        pub dest: *mut u16;
    }
}
enum XrayBeamDirection {
    Right,
    Up,
    Down,
    Left,
    Horizontal,
}
crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C)]
    pub struct Pair_R18_R20 {
        pub r18, r20: u16;
    }
}
pub const NULL: *mut c_void = null_mut::<c_void>();
crate::game::constants::typed_constants! { i32;
    true_0 = 1, false_0 = 0,
}
pub const __ASSERT_FUNCTION: [c_char; 28] =
    unsafe { transmute::<[u8; 28], [c_char; 28]>(*b"uint8 *RomPtr_RAM(uint16_t)\0") };
#[inline]
unsafe fn RomPtr_RAM(addr: u16, game: &mut Game) -> *mut u8 {
    crate::native_assertion! {
        '_c2rust_label: addr < 0x2000;
        b"addr < 0x2000\0", b"src/sm_rtl.h\0", 51, pointer __ASSERT_FUNCTION.as_ptr()
    };
    ram_ptr!(game; addr as i32 as isize)
}
const RomPtr_91: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x91>;
const RomPtr_9B: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x9b>;
crate::ram_map::ram_pointer_functions! {
    gVramWriteEntry(a: i32) -> VramWriteEntry = [(0xd0 + a) as isize];
}
#[inline]
unsafe fn get_PoseEntry(a: u16, game: &mut Game) -> *mut PoseEntry {
    RomPtr((0x910000u32) | ((a) as u32), game) as *mut PoseEntry
}
crate::game::constants::typed_constants! { i32;
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
}
crate::game::constants::typed_constants! { i32;
    fnDemoPreInstr_CheckLeaveDemo = 0x918a9b,
}
crate::game::constants::typed_constants! { i32;
    fnGrappleBeamFunc_Inactive = 0x9bc4f0,
}
static kSamusInputHandlers: [Option<Func_V>; 28] = callback_table! { unsafe Func_V;
    Samus_Input_00_Standing, Samus_Input_01_Running, Samus_Input_02_NormalJumping,
    Samus_Input_03_SpinJumping, Samus_Input_04_MorphBallOnGround, Samus_Input_05_Crouching,
    Samus_Input_06_Falling, Samus_Input_07_Unused, Samus_Input_08_MorphBallFalling,
    Samus_Input_09_Unused, Samus_Input_0A_KnockbackOrCrystalFlashEnding, Samus_Input_0B_Unused,
    Samus_Input_0C_Unused, Samus_Input_0D_Unused, Samus_Input_0E_TurningAroundOnGround,
    Samus_Input_0F_CrouchingEtcTransition, Samus_Input_10_Moonwalking,
    Samus_Input_11_SpringBallOnGround, Samus_Input_12_SpringBallInAir,
    Samus_Input_13_SpringBallFalling, Samus_Input_14_WallJumping, Samus_Input_15_RanIntoWall,
    Samus_Input_16_Grappling, Samus_Input_17_TurningAroundJumping,
    Samus_Input_18_TurningAroundFalling, Samus_Input_19_DamageBoost,
    Samus_Input_1A_GrabbedByDraygon, Samus_Input_1B_ShinesparkEtc,
};
pub unsafe fn Samus_InputHandler(game: &mut Game) {
    kSamusInputHandlers[*ram_ptr!(game; 0xa1f) as usize].expect("non-null function pointer")(game);
}
pub unsafe fn Samus_Input_00_Standing(game: &mut Game) {
    if word!(game.samus_pose) != 0
        && word!(game.samus_pose) as i32 != kPose_9B_FaceF_VariaGravitySuit as i32
        || word!(game.elevator_status) == 0
    {
        Samus_LookupTransitionTable(game);
    }
}
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    Samus_Input_01_Running => Samus_LookupTransitionTable(game);
    Samus_Input_02_NormalJumping => Samus_LookupTransitionTable(game);
    Samus_Input_03_SpinJumping => Samus_LookupTransitionTable(game);
    Samus_Input_04_MorphBallOnGround => Samus_LookupTransitionTable(game);
}
pub fn Samus_Input_07_Unused(game: &mut Game) {}
pub unsafe fn Samus_Input_05_Crouching(game: &mut Game) {
    if word!(game.time_is_frozen_flag) != 0 {
        Samus_Func20(game);
    } else {
        Samus_LookupTransitionTable(game);
        if *ram_ptr!(game; 0xa1f) == 0 {
            word!(game.samus_y_pos, wrapping_sub(5));
            word!(game.samus_prev_y_pos, wrapping_sub(5));
        }
    };
}
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    Samus_Input_06_Falling => Samus_LookupTransitionTable(game);
    Samus_Input_08_MorphBallFalling => Samus_LookupTransitionTable(game);
}
pub fn Samus_Input_09_Unused(game: &mut Game) {}
pub unsafe fn Samus_Input_0A_KnockbackOrCrystalFlashEnding(game: &mut Game) {
    Samus_LookupTransitionTable(game);
    if *ram_ptr!(game; 0xa1f) as i32 != kMovementType_0A_KnockbackOrCrystalFlashEnding as i32 {
        Samus_InitJump(game);
        word!(game.samus_knockback_timer = 0);
    }
}
pub fn Samus_Input_0B_Unused(game: &mut Game) {}
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    Samus_Input_0C_Unused => Samus_LookupTransitionTable(game);
    Samus_Input_0D_Unused => Samus_LookupTransitionTable(game);
    Samus_Input_0E_TurningAroundOnGround => Samus_LookupTransitionTable(game);
}
pub fn Samus_Input_0F_CrouchingEtcTransition(game: &mut Game) {}
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    Samus_Input_10_Moonwalking => Samus_LookupTransitionTable(game);
    Samus_Input_11_SpringBallOnGround => Samus_LookupTransitionTable(game);
    Samus_Input_12_SpringBallInAir => Samus_LookupTransitionTable(game);
    Samus_Input_13_SpringBallFalling => Samus_LookupTransitionTable(game);
    Samus_Input_14_WallJumping => Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_Input_15_RanIntoWall(game: &mut Game) {
    if word!(game.time_is_frozen_flag) != 0 {
        Samus_Func20(game);
    } else {
        Samus_LookupTransitionTable(game);
    };
}
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    Samus_Input_16_Grappling => Samus_LookupTransitionTable(game);
    Samus_Input_17_TurningAroundJumping => Samus_LookupTransitionTable(game);
    Samus_Input_18_TurningAroundFalling => Samus_LookupTransitionTable(game);
    Samus_Input_19_DamageBoost => Samus_LookupTransitionTable(game);
    Samus_Input_1A_GrabbedByDraygon => Samus_LookupTransitionTable(game);
    Samus_Input_1B_ShinesparkEtc => Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_LookupTransitionTable(game: &mut Game) {
    if word!(game.joypad1_lastkeys) != 0 {
        let pair: Pair_R18_R20 = TranslateCustomControllerBindingsToDefault(game);
        let mut pe = get_PoseEntry(
            crate::rtl::rom_table_word!(game, 0x919ee2, word!(game.samus_pose) as isize),
            game,
        );
        if (*pe).new_input == 0xffff {
            return;
        }
        loop {
            if (pair.r18 & (*pe).new_input) == 0 && (pair.r20 & (*pe).cur_input) == 0 {
                if (*pe).new_pose != word!(game.samus_pose) {
                    word!(game.samus_new_pose = (*pe).new_pose);
                    word!(game.bomb_jump_dir = 0);
                }
                return;
            }
            pe = pe.offset(1);
            if ((*pe).new_input == 0xffff) {
                break;
            }
        }
    }
    word!(game.UNUSED_word_7E0A18 = 0);
    Samus_Pose_CancelGrapple(game);
}
pub unsafe fn TranslateCustomControllerBindingsToDefault(game: &mut Game) -> Pair_R18_R20 {
    let mut r18: u16 = (word!(game.joypad1_newkeys) as i32
        & (kButton_Up as i32 | kButton_Down as i32 | kButton_Left as i32 | kButton_Right as i32))
        as u16;
    let mut r20: u16 = (word!(game.joypad1_lastkeys) as i32
        & (kButton_Up as i32 | kButton_Down as i32 | kButton_Left as i32 | kButton_Right as i32))
        as u16;
    let v0: u16 = word!(game.joypad1_newkeys);
    if (word!(game.button_config_shoot_x) & word!(game.joypad1_newkeys)) != 0 {
        r18 = (r18) | ((kButton_X) as u16);
    }
    if (word!(game.button_config_jump_a) & v0) != 0 {
        r18 = (r18) | ((kButton_A) as u16);
    }
    if (word!(game.button_config_run_b) & v0) != 0 {
        r18 = (r18) | ((kButton_B) as u16);
    }
    if (word!(game.button_config_itemcancel_y) & v0) != 0 {
        r18 = (r18) | ((kButton_Y) as u16);
    }
    if (word!(game.button_config_aim_up_R) & v0) != 0
        && word!(game.button_config_aim_up_R) as i32 & (kButton_L as i32 | kButton_R as i32) != 0
    {
        r18 = (r18) | ((kButton_R) as u16);
    }
    if (word!(game.button_config_aim_down_L) & v0) != 0
        && word!(game.button_config_aim_down_L) as i32 & (kButton_L as i32 | kButton_R as i32) != 0
    {
        r18 = (r18) | ((kButton_L) as u16);
    }
    let v1: u16 = word!(game.joypad1_lastkeys);
    if (word!(game.button_config_shoot_x) & word!(game.joypad1_lastkeys)) != 0 {
        r20 = (r20) | ((kButton_X) as u16);
    }
    if (word!(game.button_config_jump_a) & v1) != 0 {
        r20 = (r20) | ((kButton_A) as u16);
    }
    if (word!(game.button_config_run_b) & v1) != 0 {
        r20 = (r20) | ((kButton_B) as u16);
    }
    if (word!(game.button_config_itemcancel_y) & v1) != 0 {
        r20 = (r20) | ((kButton_Y) as u16);
    }
    if (word!(game.button_config_aim_up_R) & v1) != 0
        && word!(game.button_config_aim_up_R) as i32 & (kButton_L as i32 | kButton_R as i32) != 0
    {
        r20 = (r20) | ((kButton_R) as u16);
    }
    if (word!(game.button_config_aim_down_L) & v1) != 0
        && word!(game.button_config_aim_down_L) as i32 & (kButton_L as i32 | kButton_R as i32) != 0
    {
        r20 = (r20) | ((kButton_L) as u16);
    }
    Pair_R18_R20 {
        r18: !(r18 as i32) as u16,
        r20: !(r20 as i32) as u16,
    }
}
pub unsafe fn Samus_Pose_CancelGrapple(game: &mut Game) {
    if Samus_Pose_Func2(game) & 1 != 0
        || (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629)) as *mut SamusPoseParams)
            .offset(word!(game.samus_pose) as isize))
        .new_pose_unless_buttons
            == 255
    {
        word!(game.samus_new_pose = word!(game.samus_pose));
    } else {
        word!(
            game.samus_new_pose = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
                as *mut SamusPoseParams)
                .offset(word!(game.samus_pose) as isize))
            .new_pose_unless_buttons as u16
        );
    };
}
static kSamus_Pose_Func2_Tab: [u8; 28] = [
    2, 1, 1, 0, 6, 2, 8, 2, 1, 6, 2, 2, 2, 6, 2, 2, 2, 6, 6, 6, 6, 2, 6, 2, 2, 2, 2, 2,
];
pub unsafe fn Samus_Pose_Func2(game: &mut Game) -> u8 {
    let mut v0: u16 = kSamus_Pose_Func2_Tab[*ram_ptr!(game; 0xa1f) as usize] as u16;
    if (v0 == 1) {
        if word!(game.samus_x_base_speed) == 0 && word!(game.samus_x_base_subspeed) == 0 {
            v0 = 2;
        } else {
            word!(game.samus_momentum_routine_index = 1);
            return 1;
        }
    }
    word!(game.samus_momentum_routine_index = v0);
    0
}
pub fn EnableDemoInput(game: &mut Game) {
    game.set_samus_input_handler(fnSamus_InputHandler_E91D as u16);
    game.set_demo_enable(game.demo_enable() | 0x8000);
}
pub unsafe fn DisableDemoInput(game: &mut Game) {
    word!(game.samus_input_handler = ((0x90e913i32) as u16));
    word!(game.demo_enable, |fresh1| (fresh1 & !(0x8000)));
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
crate::game::address_dispatch! {
    pub unsafe fn CallDemoPreInstr(address: u32, game: &mut Game) => address;
        call (game); default { Unreachable(game); };
    {
        0x9183bf => [()]; 0x91864f => DemoPreInstr_864F; 0x91866a => DemoPreInstr_866A;
        0x918a9b => DemoPreInstr_CheckLeaveDemo; 0x918ab0 => DemoPreInstr_8AB0;
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallDemoInstr(address: u32, k: u16, j: u16, game: &mut Game) -> u16 => address;
        call (k, j, game); default Unreachable(game) as u16;
    {
        0x918427 => DemoInstr_Finish; 0x918434 => DemoInstr_SetPreInstr;
        0x91843f => DemoInstr_ClearPreInstr; 0x918448 => DemoInstr_Goto;
        0x91844f => DemoInstr_DecTimerAndGoto; 0x918459 => DemoInstr_SetTimer;
        0x9185fc => DemoInstr_Func2; 0x918682 => DemoInstr_Disable; 0x918739 => DemoInstr_Func3;
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
crate::instruction_callbacks! { pub unsafe fn (k: u16, j: u16, game: &mut Game) -> u16;
    DemoInstr_SetPreInstr [(j).wrapping_add(2)] {
        word!(game.demo_input_pre_instr = *(RomPtr_91(j, game) as *mut u16));
    };
    DemoInstr_ClearPreInstr [j] { word!(game.demo_input_pre_instr = 0x8447); };
    DemoInstr_Goto [*(RomPtr_91(j, game) as *mut u16)] { };
    DemoInstr_DecTimerAndGoto [] {
        if word!(game.demo_timer_counter, wrapping_sub(1), reread) != 0 {
            DemoInstr_Goto(k, j, game)
        } else {
            (j).wrapping_add(2)
        }
    };
    DemoInstr_SetTimer [(j).wrapping_add(2)] {
        word!(game.demo_timer_counter = *(RomPtr_91(j, game) as *mut u16));
    };
    DemoInstr_Func2 [j] {
        word!(game.frame_handler_alfa = ((0x90e695i32) as u16));
        word!(game.frame_handler_beta = ((0x90e725i32) as u16));
        DisableDemoInput(game);
    };
}
pub unsafe fn DemoPreInstr_864F(game: &mut Game) {
    if word!(game.samus_x_pos) as i32 - 178i32 & 0x8000 != 0 {
        word!(game.demo_input_pre_instr = ((0x91866ai32) as u16));
        word!(game.demo_input_instr_ptr = 0x8623);
        word!(game.demo_input_instr_timer = 1);
    }
}
pub unsafe fn DemoPreInstr_866A(game: &mut Game) {
    if word!(game.eproj_x_pos[0]) == 0 {
        word!(game.demo_input_pre_instr = ((0x9183bfi32) as u16));
        word!(game.demo_input_instr_ptr = addr_off_91864B as u16);
        word!(game.demo_input_instr_timer = 1);
    }
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, j: u16, game: &mut Game) -> u16;
    DemoInstr_Disable [j] {
        word!(game.frame_handler_alfa = ((0x90e8cdi32) as u16));
        word!(game.frame_handler_beta = ((0x90e8cdi32) as u16));
        DisableDemoInput(game);
    };
    DemoInstr_Func3 [j] {
        word!(game.frame_handler_alfa = ((0x90e713i32) as u16));
        word!(game.samus_pose = kPose_02_FaceL_Normal as u16);
        SamusFunc_F433(game);
        Samus_SetAnimationFrameIfPoseChanged(game);
        word!(game.samus_last_different_pose = word!(game.samus_prev_pose));
        *(ram_ptr!(game, u16; 0xa26)) = *(ram_ptr!(game, u16; 0xa22));
        word!(game.samus_prev_pose = word!(game.samus_pose));
        *(ram_ptr!(game, u16; 0xa22)) = *(ram_ptr!(game, u16; 0xa1e));
        DisableDemoInput(game);
        word!(game.samus_input_handler = ((0x90e90ei32) as u16));
    };
}
static kDemoSetFuncPtrs_0: [Option<Func_V>; 6] = callback_table! { unsafe Func_V;
    DemoSetFunc_0, DemoSetFunc_1, DemoSetFunc_1, DemoSetFunc_2, DemoSetFunc_1, DemoSetFunc_3,
};
static kDemoSetFuncPtrs_1: [Option<Func_V>; 6] = callback_table! { unsafe Func_V;
    DemoSetFunc_2, DemoSetFunc_2, DemoSetFunc_1, DemoSetFunc_2, DemoSetFunc_1, DemoSetFunc_4,
};
static kDemoSetFuncPtrs_2: [Option<Func_V>; 6] = callback_table! { unsafe Func_V;
    DemoSetFunc_2, DemoSetFunc_5, DemoSetFunc_1, DemoSetFunc_1, DemoSetFunc_1, DemoSetFunc_1,
};
static kDemoSetFuncPtrs_3: [Option<Func_V>; 5] = callback_table! { unsafe Func_V;
    DemoSetFunc_6, DemoSetFunc_2, DemoSetFunc_2, DemoSetFunc_1, DemoSetFunc_7,
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
    word!(game.samus_health = 20);
    DemoSetFunc_2(game);
}
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    DemoSetFunc_2 => DemoSetFunc_Common(kPose_02_FaceL_Normal as u16, game);
    DemoSetFunc_4 => DemoSetFunc_Common(kPose_2A_FaceL_Fall as u16, game);
    DemoSetFunc_1 => DemoSetFunc_Common(kPose_01_FaceR_Normal as u16, game);
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
    word!(game.samus_draw_handler = ((0x90eb52i32) as u16));
    Projectile_Func7_Shinespark(game);
    word!(game.samus_pose = kPose_CA_FaceL_Shinespark_Horiz as u16);
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
}
pub unsafe fn DemoPreInstr_CheckLeaveDemo(game: &mut Game) {
    if word!(game.game_state) as i32 == kGameState_44_TransitionFromDemo as i32 {
        word!(game.demo_input_instr_ptr = addr_kDemoInstrs_LeaveDemo as u16);
        word!(game.demo_input_instr_timer = 1);
    }
}
pub fn DemoPreInstr_8AB0(game: &mut Game) {
    if game.samus_movement_type() != kMovementType_1A_GrabbedByDraygon as u8 {
        game.set_demo_input_pre_instr(fnDemoPreInstr_CheckLeaveDemo as u16);
        game.set_demo_input_instr_ptr(addr_stru_919346 as u16);
        game.set_demo_input_instr_timer(1);
    }
}
unsafe fn XrayFillUp(dest: *mut u16, fill_with: u16, mut v1: u16) {
    loop {
        *dest.offset((v1 >> 1) as isize) = fill_with;
        v1 = (v1).wrapping_sub(2);
        if (v1 & 0x8000 != 0) {
            break;
        }
    }
}
unsafe fn XrayFillDown(dest: *mut u16, fill_with: u16, mut v1: u16) {
    loop {
        *dest.offset((v1 >> 1) as isize) = fill_with;
        v1 = (v1).wrapping_add(2);
        if (v1 >= 460) {
            break;
        }
    }
}
unsafe fn XrayFillUpRamped(
    dest: *mut u16,
    mut pos: i32,
    mut left: u32,
    left_step: u32,
    mut right: u32,
    right_step: u32,
    adjust: i32,
) {
    loop {
        left = left.wrapping_add(left_step);
        right = right.wrapping_add(right_step);
        let left_val: u32 = if left & !(0xffffi32) as u32 != 0 {
            (if (left as i32) < 0 { 0 } else { 0xffff }) as u32
        } else {
            left
        };
        let right_val: u32 = if right & !(0xffffi32) as u32 != 0 {
            (if (right as i32) < 0 { 0 } else { 0xffff }) as u32
        } else {
            right
        };
        let finished: bool = left & !(0xffffi32) as u32 != 0
            && right & !(0xffffi32) as u32 != 0
            && ((left_step ^ left | right_step ^ right) >> 31) == 0;
        *dest.offset((pos >> 1) as isize) = (if (right as i32) < 0 || left as i32 >= 0x10000 {
            0xff
        } else {
            left_val >> 8 | right_val & !(0xffi32) as u32
        }) as u16;
        if finished {
            pos += adjust;
            loop {
                pos -= 2;
                if (pos < 0) {
                    break;
                }
                *dest.offset((pos >> 1) as isize) = 0xff;
            }
            return;
        }
        pos -= 2;
        if (pos < 0) {
            break;
        }
    }
}
unsafe fn XrayFillDownRamped(
    dest: *mut u16,
    mut pos: i32,
    mut left: u32,
    left_step: u32,
    mut right: u32,
    right_step: u32,
    adjust: i32,
) {
    loop {
        left = left.wrapping_add(left_step);
        right = right.wrapping_add(right_step);
        let left_val: u32 = if left & !(0xffffi32) as u32 != 0 {
            (if (left as i32) < 0 { 0 } else { 0xffff }) as u32
        } else {
            left
        };
        let right_val: u32 = if right & !(0xffffi32) as u32 != 0 {
            (if (right as i32) < 0 { 0 } else { 0xffff }) as u32
        } else {
            right
        };
        let finished: bool = left & !(0xffffi32) as u32 != 0
            && right & !(0xffffi32) as u32 != 0
            && ((left_step ^ left | right_step ^ right) >> 31) == 0;
        *dest.offset((pos >> 1) as isize) = (if (right as i32) < 0 || left as i32 >= 0x10000 {
            0xff
        } else {
            left_val >> 8 | right_val & !(0xffi32) as u32
        }) as u16;
        if finished {
            pos += adjust;
            loop {
                pos += 2;
                if (pos >= 460) {
                    break;
                }
                *dest.offset((pos >> 1) as isize) = 0xff;
            }
            return;
        }
        pos += 2;
        if (pos >= 460) {
            break;
        }
    }
}
unsafe fn XrayHdmaFunc_BeamAimedR(ctx: *mut XrayHdmaCtx, game: &mut Game) {
    let p: i32 = 2 * (*ctx).r24 as i32;
    XrayFillUpRamped(
        (*ctx).dest,
        p - 2i32,
        (((*ctx).r22) as u32).wrapping_sub(0x10000),
        (*ctx).r30 as u32,
        65535,
        1,
        2,
    );
    XrayFillDownRamped(
        (*ctx).dest,
        p,
        (((*ctx).r22) as u32).wrapping_sub(0x10000),
        (*ctx).r32 as u32,
        65535,
        1,
        -2i32,
    );
}
unsafe fn XrayHdmaFunc_BeamAimedL(ctx: *mut XrayHdmaCtx, game: &mut Game) {
    let t: i32 = 2 * (*ctx).r24 as i32;
    XrayFillUpRamped(
        (*ctx).dest,
        t - 2i32,
        0,
        -1i32 as u32,
        (((*ctx).r22) as u32).wrapping_add(0x10000),
        -((*ctx).r32 as i32) as u32,
        2,
    );
    XrayFillDownRamped(
        (*ctx).dest,
        t,
        0,
        -1i32 as u32,
        (((*ctx).r22) as u32).wrapping_add(0x10000),
        -((*ctx).r30 as i32) as u32,
        -2i32,
    );
}
unsafe fn XrayHdmaFunc_BeamAimedU(ctx: *mut XrayHdmaCtx, game: &mut Game) {
    let p: i32 = 2 * (*ctx).r24 as i32;
    if (*ctx).r26 as i32 - 192i32 & 0x8000 != 0 {
        XrayFillUpRamped(
            (*ctx).dest,
            p - 2i32,
            (((*ctx).r22) as u32).wrapping_sub(0x10000),
            (*ctx).r30 as u32,
            (((*ctx).r22) as u32).wrapping_sub(0x10000),
            (*ctx).r32 as u32,
            0,
        );
    } else if (*ctx).r28 as i32 - 192i32 & 0x8000 != 0 {
        XrayFillUpRamped(
            (*ctx).dest,
            p - 2i32,
            (((*ctx).r22) as u32).wrapping_add(0x10000),
            -((*ctx).r30 as i32) as u32,
            (((*ctx).r22) as u32).wrapping_sub(0x10000),
            (*ctx).r32 as u32,
            0,
        );
    } else {
        XrayFillUpRamped(
            (*ctx).dest,
            p - 2i32,
            (((*ctx).r22) as u32).wrapping_add(0x10000),
            -((*ctx).r30 as i32) as u32,
            (((*ctx).r22) as u32).wrapping_add(0x10000),
            -((*ctx).r32 as i32) as u32,
            2,
        );
    }
    XrayFillDown((*ctx).dest, 0xff, p as u16);
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
    let p: i32 = 2 * ctx.r24 as i32;
    if ctx.r28 as i32 - 128i32 & 0x8000 != 0 {
        // Current callers pass the $9100 or $9800 table inside this g_ram.
        let base = 0x7e0000 + ctx.dest.cast::<u8>().offset_from(game.g_ram.as_ptr()) as u32;
        xray_offscreen_positive_stores(
            |address, value| game.g_ram[(address - 0x7e0000) as usize] = value,
            base,
            ctx.r24.wrapping_mul(2),
            ctx.r22,
            ctx.r32,
            ctx.r30,
        );
        return;
    } else if ctx.r26 as i32 - 128i32 & 0x8000 != 0 {
        XrayFillDownRamped(
            ctx.dest,
            p,
            ((ctx.r22) as u32).wrapping_add(0x10000),
            -(ctx.r32 as i32) as u32,
            ((ctx.r22) as u32).wrapping_sub(0x10000),
            ctx.r30 as u32,
            0,
        );
    } else {
        XrayFillDownRamped(
            ctx.dest,
            p,
            ((ctx.r22) as u32).wrapping_add(0x10000),
            -(ctx.r32 as i32) as u32,
            ((ctx.r22) as u32).wrapping_add(0x10000),
            -(ctx.r30 as i32) as u32,
            0,
        );
    }
    XrayFillUp(ctx.dest, 0xff, (p - 2i32) as u16);
}
unsafe fn XrayHdmaFunc_BeamHoriz(ctx: *mut XrayHdmaCtx, game: &mut Game) {
    let v0: u16 = (2 * ((*ctx).r24 as i32 - 1i32)) as u16;
    *(*ctx).dest.offset((v0 >> 1) as isize) = 0xff00;
    XrayFillUp((*ctx).dest, 0xff, (v0).wrapping_sub(2));
    XrayFillDown((*ctx).dest, 0xff, (v0).wrapping_add(2));
}
unsafe fn XrayHdmaOnScreen_BeamAimedR(ctx: *mut XrayHdmaCtx, game: &mut Game) {
    let p: i32 = 2 * (*ctx).r24 as i32;
    *(*ctx).dest.offset((p - 2i32 >> 1) as isize) = ((*ctx).r22 as i32 >> 8 | 0xff00) as u16;
    XrayFillUpRamped(
        (*ctx).dest,
        p - 4i32,
        (*ctx).r22 as u32,
        (*ctx).r30 as u32,
        65535,
        1,
        2,
    );
    XrayFillDownRamped(
        (*ctx).dest,
        p,
        (*ctx).r22 as u32,
        (*ctx).r32 as u32,
        65535,
        1,
        -2i32,
    );
}
unsafe fn XrayHdmaOnScreen_BeamAimedL(ctx: *mut XrayHdmaCtx, game: &mut Game) {
    let v0: i32 = 2 * (*ctx).r24 as i32;
    *(*ctx).dest.offset((v0 - 2i32 >> 1) as isize) = (((*ctx).r22 as i32 >> 8) << 8) as u16;
    XrayFillUpRamped(
        (*ctx).dest,
        v0 - 4i32,
        0,
        -1i32 as u32,
        (*ctx).r22 as u32,
        -((*ctx).r32 as i32) as u32,
        2,
    );
    XrayFillDownRamped(
        (*ctx).dest,
        v0,
        0,
        -1i32 as u32,
        (*ctx).r22 as u32,
        -((*ctx).r30 as i32) as u32,
        -2i32,
    );
}
unsafe fn XrayHdmaOnScreen_BeamAimedU(ctx: *mut XrayHdmaCtx, game: &mut Game) {
    let v0: i32 = 2 * (*ctx).r24 as i32;
    *(*ctx).dest.offset((v0 - 2i32 >> 1) as isize) = (((*ctx).r22 as i32 >> 8) * 0x101) as u16;
    if (*ctx).r26 as i32 - 192i32 & 0x8000 != 0 {
        XrayFillUpRamped(
            (*ctx).dest,
            v0 - 4i32,
            (*ctx).r22 as u32,
            (*ctx).r30 as u32,
            (*ctx).r22 as u32,
            (*ctx).r32 as u32,
            0,
        );
    } else if (*ctx).r28 as i32 - 192i32 & 0x8000 != 0 {
        XrayFillUpRamped(
            (*ctx).dest,
            v0 - 4i32,
            (*ctx).r22 as u32,
            -((*ctx).r30 as i32) as u32,
            (*ctx).r22 as u32,
            (*ctx).r32 as u32,
            0,
        );
    } else {
        XrayFillUpRamped(
            (*ctx).dest,
            v0 - 4i32,
            (*ctx).r22 as u32,
            -((*ctx).r30 as i32) as u32,
            (*ctx).r22 as u32,
            -((*ctx).r32 as i32) as u32,
            2,
        );
    }
    XrayFillDown((*ctx).dest, 0xff, v0 as u16);
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
    if ctx.r28 as i32 - 128i32 & 0x8000 != 0 {
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
    let v0: i32 = 2 * ctx.r24 as i32;
    *ctx.dest.offset((v0 - 2i32 >> 1) as isize) = ((ctx.r22 as i32 >> 8) * 0x101) as u16;
    if ctx.r26 as i32 - 128i32 & 0x8000 != 0 {
        XrayFillDownRamped(
            ctx.dest,
            v0,
            ctx.r22 as u32,
            -(ctx.r32 as i32) as u32,
            ctx.r22 as u32,
            ctx.r30 as u32,
            0,
        );
    } else {
        XrayFillDownRamped(
            ctx.dest,
            v0,
            ctx.r22 as u32,
            -(ctx.r32 as i32) as u32,
            ctx.r22 as u32,
            -(ctx.r30 as i32) as u32,
            0,
        );
    }
    XrayFillUp(ctx.dest, 0xff, (v0 - 4i32) as u16);
}
unsafe fn XrayHdmaOnScreen_BeamHoriz(ctx: *mut XrayHdmaCtx, game: &mut Game) {
    let v0: u16 = (2 * ((*ctx).r24 as i32 - 1i32)) as u16;
    let t: u32 = ((*ctx).r22 >> 8) as u32;
    *(*ctx).dest.offset((v0 >> 1) as isize) = (if (*ctx).r18_avoid == 64 {
        t | 0xff00
    } else {
        t << 8
    }) as u16;
    XrayFillUp((*ctx).dest, 0xff, (v0).wrapping_sub(2));
    XrayFillDown((*ctx).dest, 0xff, (v0).wrapping_add(2));
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
    if word!(game.time_is_frozen_flag) == 0
        && (word!(game.button_config_run_b) & word!(game.joypad1_lastkeys)) != 0
        && Xray_Initialize(game) & 1 != 0
    {
        SpawnHdmaObject(
            0x91,
            game.g_rom.offset(crate::rtl::lorom_offset(0x91caf2)) as *mut SpawnHdmaObject_Args
                as *const c_void,
            game,
        );
    }
}
pub unsafe fn Xray_SetupStage1_FreezeTimeBackup(k: u16, game: &mut Game) {
    *(ram_ptr!(game, u16; 0xa78) as *mut u8).offset(0) = 1;
    *(ram_ptr!(game, u16; 0x1914) as *mut u8).offset(k as u8 as i32 as isize) =
        word!(game.reg_BG2HOFS) as u8;
    *(ram_ptr!(game, u16; 0x1914) as *mut u8)
        .offset(k as u8 as i32 as isize)
        .offset(1) = *(ram_ptr!(game, u16; 0xb5) as *mut u8).offset(1);
    *(ram_ptr!(game, u16; 0x1920) as *mut u8).offset(k as u8 as i32 as isize) =
        word!(game.reg_BG2VOFS) as u8;
    *(ram_ptr!(game, u16; 0x1920) as *mut u8)
        .offset(k as u8 as i32 as isize)
        .offset(1) = *(ram_ptr!(game, u16; 0xb7) as *mut u8).offset(1);
    *(ram_ptr!(game, u16; 0x192c) as *mut u8).offset(k as u8 as i32 as isize) =
        *ram_ptr!(game; 0x59);
}
pub unsafe fn Xray_SetupStage2_ReadBg1_2ndScreen(game: &mut Game) {
    let v0: u16 = word!(game.vram_read_queue_tail);
    *((&raw mut (*(ram_ptr!(game, VramReadQueueEnt; 0x340)).offset(0)).vram_target as *mut u8)
        .offset(v0 as i32 as isize) as *mut u16) =
        (((*ram_ptr!(game; 0x58) as i32 & 0xfc) << 8) + 1024) as u16;
    *((&raw mut (*(ram_ptr!(game, VramReadQueueEnt; 0x340)).offset(0)).dma_parameters as *mut u8)
        .offset(v0 as i32 as isize) as *mut u16) = 129;
    *((&raw mut (*(ram_ptr!(game, VramReadQueueEnt; 0x340)).offset(0)).dma_parameters as *mut u8)
        .offset(v0 as i32 as isize)
        .offset(1) as *mut u16) = 57;
    *((&raw mut (*(ram_ptr!(game, VramReadQueueEnt; 0x340)).offset(0))
        .src
        .addr as *mut u8)
        .offset(v0 as i32 as isize) as *mut VoidP) = ((ram_ptr!(game, Ram4000; 0x4000) as *mut u8)
        .offset_from(&raw mut game.g_ram as *mut u8)
        as c_long
        + 0x2800 as c_long) as VoidP;
    *((&raw mut (*(ram_ptr!(game, VramReadQueueEnt; 0x340)).offset(0))
        .src
        .bank)
        .offset(v0 as i32 as isize) as *mut u16) = 126;
    *((&raw mut (*(ram_ptr!(game, VramReadQueueEnt; 0x340)).offset(0)).size as *mut u8)
        .offset(v0 as i32 as isize) as *mut u16) = 2048;
    word!(game.vram_read_queue_tail = (v0).wrapping_add(9));
}
pub unsafe fn Xray_SetupStage3_ReadBg1_1stScreen(game: &mut Game) {
    let v0: u16 = word!(game.vram_read_queue_tail);
    *((&raw mut (*(ram_ptr!(game, VramReadQueueEnt; 0x340)).offset(0)).vram_target as *mut u8)
        .offset(v0 as i32 as isize) as *mut u16) =
        ((*ram_ptr!(game; 0x58) as i32 & 0xfc) << 8) as u16;
    *((&raw mut (*(ram_ptr!(game, VramReadQueueEnt; 0x340)).offset(0)).dma_parameters as *mut u8)
        .offset(v0 as i32 as isize) as *mut u16) = 129;
    *((&raw mut (*(ram_ptr!(game, VramReadQueueEnt; 0x340)).offset(0)).dma_parameters as *mut u8)
        .offset(v0 as i32 as isize)
        .offset(1) as *mut u16) = 57;
    *((&raw mut (*(ram_ptr!(game, VramReadQueueEnt; 0x340)).offset(0))
        .src
        .addr as *mut u8)
        .offset(v0 as i32 as isize) as *mut VoidP) = ((ram_ptr!(game, Ram4000; 0x4000) as *mut u8)
        .offset_from(&raw mut game.g_ram as *mut u8)
        as c_long
        + 0x2000 as c_long) as VoidP;
    *((&raw mut (*(ram_ptr!(game, VramReadQueueEnt; 0x340)).offset(0))
        .src
        .bank)
        .offset(v0 as i32 as isize) as *mut u16) = 126;
    *((&raw mut (*(ram_ptr!(game, VramReadQueueEnt; 0x340)).offset(0)).size as *mut u8)
        .offset(v0 as i32 as isize) as *mut u16) = 2048;
    word!(game.vram_read_queue_tail = (v0).wrapping_add(9));
}
pub unsafe fn Xray_SetupStage4(game: &mut Game) {
    let mut r24: u16 =
        (4 * (word!(game.layer1_y_pos) as i32 + word!(game.bg1_y_offset) as i32 & 0xf0)) as u16;
    r24 = (r24 as i32
        + ((word!(game.layer1_x_pos) as i32 + word!(game.bg1_x_offset) as i32 & 0xf0) >> 3))
        as u16;
    r24 = (r24 as i32
        + 4 * (word!(game.layer1_x_pos) as i32 + word!(game.bg1_x_offset) as i32 & 0x100))
        as u16;
    let mut r22: u16 = 0;
    let mut m: i32 = 16;
    loop {
        let mut r26: u16 = (r24) & (0x7e0u16);
        let mut r28: u16 = (r24) & (0x1fu16);
        let mut r30: u16 = 0;
        for _ in 0..16 {
            if r30 as i32 + r28 as i32 - 32i32 & 0x8000 == 0 {
                r26 = (r26 as i32 + 1024 & 0x7e0) as u16;
                r28 = 0;
                r30 = 0;
            }
            let v0: u16 = (r30 as i32 + r28 as i32 + r26 as i32) as u16;
            let v11: u16 =
                (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps[(v0 as i32 + 4096) as usize];
            let v10: u16 =
                (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps[(v0 as i32 + 4097) as usize];
            let v9: u16 =
                (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps[(v0 as i32 + 4128) as usize];
            let v1: u16 =
                (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps[(v0 as i32 + 4129) as usize];
            *(((&raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps as *mut u16).offset(33)
                as *mut u8)
                .offset(r22 as i32 as isize) as *mut u16) = v1;
            *(((&raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps as *mut u16).offset(32)
                as *mut u8)
                .offset(r22 as i32 as isize) as *mut u16) = v9;
            *(((&raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps as *mut u16).offset(1)
                as *mut u8)
                .offset(r22 as i32 as isize) as *mut u16) = v10;
            *((&raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps as *mut u16 as *mut u8)
                .offset(r22 as i32 as isize) as *mut u16) = v11;
            r22 = (r22).wrapping_add(4);
            r30 = (r30).wrapping_add(2);
        }
        Xray_SetupStage4_Func1(r22, r26, r28, r30, game);
        r24 = ((r24 as i32 & 0x400) + (r24 as i32 + 64 & 0x3ff)) as u16;
        r22 = (r22).wrapping_add(64);
        m -= 1;
        if (m == 0) {
            break;
        }
    }
    crate::native_assertion! {
        '_c2rust_label: ((word!(game.layer1_y_pos) as i32 >> 12) as u16) == 0;
        b"(uint16)(layer1_y_pos >> 12) == 0\0", b"src/sm_91.c\0", 921,
        pointer __ASSERT_FUNCTION.as_ptr()
    };
    let mut r34: u16 = (Mult8x8(
        (word!(game.layer1_y_pos) >> 4) as u8,
        word!(game.room_width_in_blocks) as u8,
    ) as i32
        + (word!(game.layer1_x_pos) as i32 >> 4)) as u16;
    r22 = 0;
    m = 16;
    loop {
        Xray_SetupStage4_Func2(r22, r34, game);
        let mut n_0: i32 = 16;
        let mut r36: u16 = r34;
        loop {
            Xray_SetupStage4_Func3(n_0 as u16, r22, r36, game);
            r22 = (r22).wrapping_add(4);
            r36 = (r36).wrapping_add(1);
            n_0 -= 1;
            if (n_0 == 0) {
                break;
            }
        }
        crate::update_place! { previous v12: u16 = r22 => (r22).wrapping_add(1984); }
        Xray_SetupStage4_Func3(0, r22, r36, game);
        r22 = (v12).wrapping_add(64);
        r34 = (r34).wrapping_add(word!(game.room_width_in_blocks));
        m -= 1;
        if (m == 0) {
            break;
        }
    }
    LoadXrayBlocks(game);
    let v7: u16 = word!(game.vram_read_queue_tail);
    *((&raw mut (*(ram_ptr!(game, VramReadQueueEnt; 0x340)).offset(0)).vram_target as *mut u8)
        .offset(v7 as i32 as isize) as *mut u16) =
        ((*ram_ptr!(game; 0x59) as i32 & 0xfc) << 8) as u16;
    *((&raw mut (*(ram_ptr!(game, VramReadQueueEnt; 0x340)).offset(0)).dma_parameters as *mut u8)
        .offset(v7 as i32 as isize) as *mut u16) = 129;
    *((&raw mut (*(ram_ptr!(game, VramReadQueueEnt; 0x340)).offset(0)).dma_parameters as *mut u8)
        .offset(v7 as i32 as isize)
        .offset(1) as *mut u16) = 57;
    *((&raw mut (*(ram_ptr!(game, VramReadQueueEnt; 0x340)).offset(0))
        .src
        .addr as *mut u8)
        .offset(v7 as i32 as isize) as *mut VoidP) = 0x5000 as VoidP;
    *((&raw mut (*(ram_ptr!(game, VramReadQueueEnt; 0x340)).offset(0))
        .src
        .bank)
        .offset(v7 as i32 as isize) as *mut u16) = 126;
    *((&raw mut (*(ram_ptr!(game, VramReadQueueEnt; 0x340)).offset(0)).size as *mut u8)
        .offset(v7 as i32 as isize) as *mut u16) = 2048;
    word!(game.vram_read_queue_tail = (v7).wrapping_add(9));
}
pub unsafe fn Xray_SetupStage4_Func1(dst_r22: u16, r26: u16, r28: u16, r30: u16, game: &mut Game) {
    let mut R32: u16 = r26;
    let mut v0: i16 = ((r30) as i16).wrapping_add((r28) as i16);
    if r30 as i32 + r28 as i32 - 32i32 & 0x8000 == 0 {
        R32 = (R32 as i32 + 1024 & 0x7e0) as u16;
        v0 = 0;
    }
    let v1: u16 = (R32).wrapping_add((v0) as u16);
    let v6: u16 = (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps[(v1 as i32 + 4096) as usize];
    let v5: u16 = (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps[(v1 as i32 + 4097) as usize];
    let v4: u16 = (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps[(v1 as i32 + 4128) as usize];
    let v3: u16 = (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps[(v1 as i32 + 4129) as usize];
    let v2: u16 = (dst_r22).wrapping_add(1984);
    *(((&raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps as *mut u16).offset(33)
        as *mut u8)
        .offset(v2 as i32 as isize) as *mut u16) = v3;
    *(((&raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps as *mut u16).offset(32)
        as *mut u8)
        .offset(v2 as i32 as isize) as *mut u16) = v4;
    *(((&raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps as *mut u16).offset(1)
        as *mut u8)
        .offset(v2 as i32 as isize) as *mut u16) = v5;
    *((&raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps as *mut u16 as *mut u8)
        .offset(v2 as i32 as isize) as *mut u16) = v6;
}
unsafe fn Xray_GetXrayedBlock(k: u16, game: &mut Game) -> *const u8 {
    let bts: u16 = *ram_ptr!(game; 0x16402, k as isize) as u16;
    let r40: u16 = (word!(game.level_data[k])) & (0xf000u16);
    let mut i: i32 = 0;
    loop {
        let mut value: u16 = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91d2d6))
            as *mut XrayBlockData)
            .offset(i as isize))
        .value;
        if value == 0xffff {
            break;
        }
        if value == r40 {
            let mut p = RomPtr_91(
                (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91d2d6)) as *mut XrayBlockData)
                    .offset(i as isize))
                .addr,
                game,
            );
            loop {
                value = *(p as *mut u16);
                if value == 0xffff {
                    break;
                }
                if value == 0xff00 || value == bts {
                    return RomPtr_91(*(p.offset(2) as *mut u16), game);
                }
                p = p.offset(4);
            }
            return null::<u8>();
        }
        i += 1;
    }
    null::<u8>()
}
pub unsafe fn Xray_SetupStage4_Func2(dst_r22: u16, r34: u16, game: &mut Game) {
    if r34 == 0 {
        return;
    }
    let jp = Xray_GetXrayedBlock((r34).wrapping_sub(1), game);
    let dst = (&raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps as *mut u16 as *mut u8)
        .offset(dst_r22 as i32 as isize) as *mut u16;
    if jp.is_null() {
        return;
    }
    if *(jp as *mut u16) as i32 != 0x91cf4e & 0xffff {
        if *(jp as *mut u16) as i32 != 0x91cf6f & 0xffff {
            return;
        }
        let src = (&raw mut (*(ram_ptr!(game, TileTables; 0xa000))).tables as *mut TileTable)
            .offset(*(jp.offset(8) as *mut u16) as i32 as isize);
        *dst.offset(64) = (*src).top_left;
        *dst.offset(65) = (*src).top_right;
        *dst.offset(96) = (*src).bottom_left;
        *dst.offset(97) = (*src).bottom_right;
    }
    let src_0 = (&raw mut (*(ram_ptr!(game, TileTables; 0xa000))).tables as *mut TileTable)
        .offset(*(jp.offset(4) as *mut u16) as i32 as isize);
    *dst.offset(0) = (*src_0).top_left;
    *dst.offset(1) = (*src_0).top_right;
    *dst.offset(32) = (*src_0).bottom_left;
    *dst.offset(33) = (*src_0).bottom_right;
}
unsafe fn Xray_CombinedMove(dst_r22: u16, r36: u16, which_dir: bool, game: &mut Game) {
    let mut y: u16 = (r36 as i32 / word!(game.room_width_in_blocks) as i32) as u16;
    let mut x: u16 = (r36 as i32 % word!(game.room_width_in_blocks) as i32) as u16;
    let mut r48: u16 = (word!(game.level_data[r36])) & (0xf000u16);
    let mut step: i32 = *ram_ptr!(game; 0x16402, r36 as isize) as i8 as i32;
    if step != 0 {
        let mut variable = if which_dir { &raw mut y } else { &raw mut x };
        loop {
            if ((*variable as i32 + step) as i16) < 0 {
                Xray_Func13(dst_r22, 0xff, game);
                return;
            }
            *variable = (*variable as i32 + step) as u16;
            let pos: i32 = y as i32 * word!(game.room_width_in_blocks) as i32 + x as i32;
            step = *ram_ptr!(game; 0x16402, pos as isize) as i8 as i32;
            r48 = (word!(game.level_data[pos])) & (0xf000u16);
            if r48 == 0xd000 {
                variable = &raw mut y;
            } else {
                if r48 != 0x5000 {
                    break;
                }
                if step < 0 {
                    variable = &raw mut x;
                }
            }
        }
    }
    if (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91d2d6)) as *mut XrayBlockData).offset(1))
        .value
        == r48
    {
        let mut t = RomPtr_91(
            (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91d2d6)) as *mut XrayBlockData)
                .offset(1))
            .addr,
            game,
        ) as *mut u16;
        while *t.offset(0) != 0xffff {
            if *t.offset(0) == 0xff00 || *t.offset(0) as i32 == step & 0xff {
                Xray_Func12(
                    dst_r22,
                    RomPtr_91((*t.offset(1)).wrapping_add(2), game),
                    game,
                );
                return;
            }
            t = t.offset(2);
        }
    }
}
unsafe fn Xray_Func7(dst_r22: u16, jp: *const u8, game: &mut Game) {
    Xray_Func12(dst_r22, jp.offset(2), game);
}
unsafe fn Xray_Func8(dst_r22: u16, jp: *const u8, game: &mut Game) {
    if word!(game.area_index) == 1 {
        Xray_Func12(dst_r22, jp.offset(2), game);
    }
}
unsafe fn Xray_Func9(r18: u16, dst_r22: u16, jp: *const u8, game: &mut Game) {
    Xray_Func12(dst_r22, jp.offset(2), game);
    if r18 != 1 {
        Xray_Func14(dst_r22, jp.offset(4), game);
    }
}
unsafe fn Xray_Func10(dst_r22: u16, jp: *const u8, game: &mut Game) {
    Xray_Func12(dst_r22, jp.offset(2), game);
    Xray_Func15(dst_r22, jp.offset(2), game);
}
unsafe fn Xray_Func11(r18: u16, dst_r22: u16, jp: *const u8, game: &mut Game) {
    Xray_Func12(dst_r22, jp.offset(2), game);
    if r18 != 1 {
        Xray_Func14(dst_r22, jp.offset(4), game);
    }
    Xray_Func15(dst_r22, jp.offset(6), game);
    if r18 != 1 {
        let v5: u16 = *(jp.offset(8) as *mut u16);
        let top_left: u16 = (*(ram_ptr!(game, TileTables; 0xa000))).tables[v5 as usize].top_left;
        let top_right: u16 = (*(ram_ptr!(game, TileTables; 0xa000))).tables[v5 as usize].top_right;
        let bottom_left: u16 =
            (*(ram_ptr!(game, TileTables; 0xa000))).tables[v5 as usize].bottom_left;
        let bottom_right: u16 =
            (*(ram_ptr!(game, TileTables; 0xa000))).tables[v5 as usize].bottom_right;
        *(((&raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps as *mut u16).offset(99)
            as *mut u8)
            .offset(dst_r22 as i32 as isize) as *mut u16) = bottom_right;
        *(((&raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps as *mut u16).offset(98)
            as *mut u8)
            .offset(dst_r22 as i32 as isize) as *mut u16) = bottom_left;
        *(((&raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps as *mut u16).offset(67)
            as *mut u8)
            .offset(dst_r22 as i32 as isize) as *mut u16) = top_right;
        *(((&raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps as *mut u16).offset(66)
            as *mut u8)
            .offset(dst_r22 as i32 as isize) as *mut u16) = top_left;
    }
}
crate::game::address_dispatch! {
    unsafe fn CallXrayFunc(ea: u32, jp: *const u8, r18: u16, r22: u16, r36: u16, game: &mut Game) => ea;
        call (r22, jp, game); default { Unreachable(game); };
    {
        9555577 => [{
                    Xray_CombinedMove(r22, r36, true_0 != 0, game);
                }];
        9555643 => [{
                    Xray_CombinedMove(r22, r36, false_0 != 0, game);
                }];
        9555766 => Xray_Func7; 9555774 => Xray_Func8; 9555790 => [Xray_Func9(r18, r22, jp, game)];
        9555810 => Xray_Func10; 9555823 => [Xray_Func11(r18, r22, jp, game)];
    }
}
pub unsafe fn Xray_SetupStage4_Func3(r18: u16, r22: u16, r36: u16, game: &mut Game) {
    let jp = Xray_GetXrayedBlock(r36, game);
    if !jp.is_null() {
        CallXrayFunc(
            ((*(jp as *mut u16)) as u32) | (0x910000u32),
            jp,
            r18,
            r22,
            r36,
            game,
        );
    }
}
unsafe fn Xray_Func12(dst_r22: u16, jp: *const u8, game: &mut Game) {
    Xray_Func13(dst_r22, *(jp as *mut u16), game);
}
unsafe fn Xray_Func13(dst_r22: u16, a: u16, game: &mut Game) {
    let top_left: u16 = (*(ram_ptr!(game, TileTables; 0xa000))).tables[a as usize].top_left;
    let top_right: u16 = (*(ram_ptr!(game, TileTables; 0xa000))).tables[a as usize].top_right;
    let bottom_left: u16 = (*(ram_ptr!(game, TileTables; 0xa000))).tables[a as usize].bottom_left;
    let bottom_right: u16 = (*(ram_ptr!(game, TileTables; 0xa000))).tables[a as usize].bottom_right;
    *(((&raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps as *mut u16).offset(33)
        as *mut u8)
        .offset(dst_r22 as i32 as isize) as *mut u16) = bottom_right;
    *(((&raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps as *mut u16).offset(32)
        as *mut u8)
        .offset(dst_r22 as i32 as isize) as *mut u16) = bottom_left;
    *(((&raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps as *mut u16).offset(1)
        as *mut u8)
        .offset(dst_r22 as i32 as isize) as *mut u16) = top_right;
    *(((&raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps as *mut u16).offset(0)
        as *mut u8)
        .offset(dst_r22 as i32 as isize) as *mut u16) = top_left;
}
unsafe fn Xray_Func14(dst_r22: u16, jp: *const u8, game: &mut Game) {
    let a: u16 = *(jp as *mut u16);
    let top_left: u16 = (*(ram_ptr!(game, TileTables; 0xa000))).tables[a as usize].top_left;
    let top_right: u16 = (*(ram_ptr!(game, TileTables; 0xa000))).tables[a as usize].top_right;
    let bottom_left: u16 = (*(ram_ptr!(game, TileTables; 0xa000))).tables[a as usize].bottom_left;
    let bottom_right: u16 = (*(ram_ptr!(game, TileTables; 0xa000))).tables[a as usize].bottom_right;
    *(((&raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps as *mut u16).offset(35)
        as *mut u8)
        .offset(dst_r22 as i32 as isize) as *mut u16) = bottom_right;
    *(((&raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps as *mut u16).offset(34)
        as *mut u8)
        .offset(dst_r22 as i32 as isize) as *mut u16) = bottom_left;
    *(((&raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps as *mut u16).offset(3)
        as *mut u8)
        .offset(dst_r22 as i32 as isize) as *mut u16) = top_right;
    *(((&raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps as *mut u16).offset(2)
        as *mut u8)
        .offset(dst_r22 as i32 as isize) as *mut u16) = top_left;
}
unsafe fn Xray_Func15(dst_r22: u16, jp: *const u8, game: &mut Game) {
    let a: u16 = *(jp as *mut u16);
    let top_left: u16 = (*(ram_ptr!(game, TileTables; 0xa000))).tables[a as usize].top_left;
    let top_right: u16 = (*(ram_ptr!(game, TileTables; 0xa000))).tables[a as usize].top_right;
    let bottom_left: u16 = (*(ram_ptr!(game, TileTables; 0xa000))).tables[a as usize].bottom_left;
    let bottom_right: u16 = (*(ram_ptr!(game, TileTables; 0xa000))).tables[a as usize].bottom_right;
    *(((&raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps as *mut u16).offset(97)
        as *mut u8)
        .offset(dst_r22 as i32 as isize) as *mut u16) = bottom_right;
    *(((&raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps as *mut u16).offset(96)
        as *mut u8)
        .offset(dst_r22 as i32 as isize) as *mut u16) = bottom_left;
    *(((&raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps as *mut u16).offset(65)
        as *mut u8)
        .offset(dst_r22 as i32 as isize) as *mut u16) = top_right;
    *(((&raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps as *mut u16).offset(64)
        as *mut u8)
        .offset(dst_r22 as i32 as isize) as *mut u16) = top_left;
}
unsafe fn Xray_Func16(dst_r22: u16, a: u16, game: &mut Game) {
    let top_left: u16 = (*(ram_ptr!(game, TileTables; 0xa000))).tables[a as usize].top_left;
    let top_right: u16 = (*(ram_ptr!(game, TileTables; 0xa000))).tables[a as usize].top_right;
    let bottom_left: u16 = (*(ram_ptr!(game, TileTables; 0xa000))).tables[a as usize].bottom_left;
    let bottom_right: u16 = (*(ram_ptr!(game, TileTables; 0xa000))).tables[a as usize].bottom_right;
    *(((&raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps as *mut u16).offset(1)
        as *mut u8)
        .offset(dst_r22 as i32 as isize) as *mut u16) = bottom_right;
    *(((&raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps as *mut u16).offset(0)
        as *mut u8)
        .offset(dst_r22 as i32 as isize) as *mut u16) = bottom_left;
    *(((&raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps as *mut u16).offset(33)
        as *mut u8)
        .offset(dst_r22 as i32 as isize) as *mut u16) = top_right;
    *(((&raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps as *mut u16).offset(32)
        as *mut u8)
        .offset(dst_r22 as i32 as isize) as *mut u16) = top_left;
}
pub unsafe fn LoadBlockToXrayTilemap(a: u16, k: u16, j: u16, game: &mut Game) {
    let R24: u16 = (k as i32 - (word!(game.layer1_x_pos) as i32 >> 4)) as u16;
    let R26: u16 = (j as i32 - (word!(game.layer1_y_pos) as i32 >> 4)) as u16;
    if R24 & 0x8000 == 0
        && R24 as i32 - 16i32 & 0x8000 != 0
        && R26 & 0x8000 == 0
        && R26 as i32 - 16i32 & 0x8000 != 0
    {
        let R22: u16 = (4 * (R24 as i32 + 32 * R26 as i32)) as u16;
        if (a & 0x800) != 0 {
            Xray_Func16(R22, (a) & (0x3ffu16), game);
        } else {
            Xray_Func13(R22, (a) & (0x3ffu16), game);
        }
    }
}
pub unsafe fn Xray_SetupStage5(game: &mut Game) {
    if CanXrayShowBlocks(game) {
        if word!(game.earthquake_timer) != 0 {
            word!(
                game.reg_BG1HOFS =
                    (word!(game.layer1_x_pos)).wrapping_add(word!(game.bg1_x_offset))
            );
            word!(
                game.reg_BG1VOFS =
                    (word!(game.layer1_y_pos)).wrapping_add(word!(game.bg1_y_offset))
            );
        }
        word!(game.reg_BG2HOFS = (word!(game.reg_BG1HOFS)) & (0xfu16));
        word!(game.reg_BG2VOFS = (word!(game.reg_BG1VOFS)) & (0xfu16));
        *ram_ptr!(game; 0x59) = 73;
    }
    let v0: u16 = word!(game.vram_read_queue_tail);
    let mut v1: u32 = word!(game.vram_read_queue_tail) as u32;
    *((&raw mut (*(ram_ptr!(game, VramReadQueueEnt; 0x340)).offset(0)).vram_target as *mut u8)
        .offset(word!(game.vram_read_queue_tail) as i32 as isize) as *mut u16) =
        (((*ram_ptr!(game; 0x59) as i32 & 0xfc) << 8) + 1024) as u16;
    *((&raw mut (*(ram_ptr!(game, VramReadQueueEnt; 0x340)).offset(0)).dma_parameters as *mut u8)
        .offset(v1 as isize)
        .offset(1) as *mut u16) = 57;
    *((&raw mut (*(ram_ptr!(game, VramReadQueueEnt; 0x340)).offset(0))
        .src
        .addr as *mut u8)
        .offset(v0 as i32 as isize) as *mut VoidP) = 0x5800 as VoidP;
    *((&raw mut (*(ram_ptr!(game, VramReadQueueEnt; 0x340)).offset(0))
        .src
        .bank)
        .offset(v0 as i32 as isize) as *mut u16) = 126;
    *((&raw mut (*(ram_ptr!(game, VramReadQueueEnt; 0x340)).offset(0)).size as *mut u8)
        .offset(v0 as i32 as isize) as *mut u16) = 2048;
    word!(game.vram_read_queue_tail = (v0).wrapping_add(9));
}
pub unsafe fn CanXrayShowBlocks(game: &mut Game) -> bool {
    if word!(game.room_ptr) as i32 == addr_kRoom_a66a as i32
        || word!(game.room_ptr) as i32 == addr_kRoom_cefb as i32
    {
        return false_0 != 0;
    }
    if word!(game.fx_type) == 0x24 {
        return false_0 != 0;
    }
    if word!(game.boss_id) == 3
        || word!(game.boss_id) == 6
        || word!(game.boss_id) == 7
        || word!(game.boss_id) == 8
        || word!(game.boss_id) == 10
    {
        return false_0 != 0;
    }
    true_0 != 0
}
pub unsafe fn Xray_SetupStage6(game: &mut Game) {
    let mut v1 = null_mut::<VramWriteEntry>();
    if CanXrayShowBlocks(game) {
        let v0: u16 = word!(game.vram_write_queue_tail);
        v1 = gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
        (*v1).size = 2048;
        (*v1).src.addr = (ram_ptr!(game, Ram4000; 0x4000) as *mut u8)
            .offset_from(&raw mut game.g_ram as *mut u8) as c_long
            as VoidP;
        *(&raw mut (*v1).src.bank as *mut u16) = 126;
        (*v1).vram_dst = ((*ram_ptr!(game; 0x59) as i32 & 0xfc) << 8) as u16;
        word!(game.vram_write_queue_tail = (v0).wrapping_add(7));
    }
}
pub unsafe fn Xray_SetupStage7(game: &mut Game) {
    let mut v1 = null_mut::<VramWriteEntry>();
    if CanXrayShowBlocks(game) {
        let v0: u16 = word!(game.vram_write_queue_tail);
        v1 = gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
        (*v1).size = 2048;
        (*v1).src.addr = 0x4800 as VoidP;
        *(&raw mut (*v1).src.bank as *mut u16) = 126;
        (*v1).vram_dst = (((*ram_ptr!(game; 0x59) as i32 & 0xfc) << 8) + 1024) as u16;
        word!(game.vram_write_queue_tail = (v0).wrapping_add(7));
    }
    mov24(ram_ptr!(game, LongPtr; 0xa88), 0x9800e4);
    mov24(ram_ptr!(game, LongPtr; 0xa8b), 0x98c8e4);
    mov24(ram_ptr!(game, LongPtr; 0xa8e), 0x999098);
    word! { game, 0;
        hdma_var_1, demo_input_pre_instr, demo_input_instr_timer, demo_input_instr_ptr, demo_input,
        demo_input_new,
    }
    if *ram_ptr!(game; 0xa1e) == 4 {
        word!(game.xray_angle = 192);
    } else {
        word!(game.xray_angle = 64);
    };
}
pub unsafe fn HdmaobjPreInstr_XraySetup(k: u16, game: &mut Game) {
    let mut v1: u16 = 4096;
    let update_window_masks: bool = if word!(game.fx_type) == 36 {
        !((*ram_ptr!(game; 0x74, 0) as i32 & 0x1f) - 7i32 & 0x8000 == 0)
    } else {
        v1 = 0x2000;
        (CanXrayShowBlocks(game))
            && ({
                v1 = 0x4000;
                true
            })
    };
    if update_window_masks {
        *(ram_ptr!(game, u16; 0x74, 0)) = 0x27;
        *(ram_ptr!(game, u16; 0x74, 1)) = 0x47;
        *(ram_ptr!(game, u16; 0x74, 2)) = 0x87;
    }
    word!(game.fx_layer_blending_config_c, |fresh37| (fresh37) | (v1));
}
pub unsafe fn Xray_SetupStage8_SetBackdropColor(game: &mut Game) {
    word!(game.palette_buffer[0] = 3171);
}
pub unsafe fn GameState_28_Unused_(game: &mut Game) {
    Unreachable(game);
}
pub unsafe fn VariaSuitPickup(game: &mut Game) {
    *ram_ptr!(game; 0xdf0) = 48;
    *ram_ptr!(game; 0xdf1) = 80;
    *ram_ptr!(game; 0xdf2) = 0x80;
    *ram_ptr!(game; 0xdf3) = 0;
    Samus_CancelSpeedBoost(game);
    SetHiLo(ram_ptr!(game, u16; 0xb42), ram_ptr!(game, u16; 0xb44), 0);
    word! { game, 0;
        samus_x_base_speed, samus_x_base_subspeed, samus_y_subspeed, samus_y_speed, samus_y_dir,
        used_for_ball_bounce_on_landing, samus_x_accel_mode, elevator_status, substate,
        suit_pickup_light_beam_pos,
    }
    word!(game.suit_pickup_light_beam_widening_speed = 256);
    let mut i: i32 = 510;
    while i >= 0 {
        word!(game.hdma_table_1[(i >> 1)] = 255);
        i -= 2;
    }
    if *ram_ptr!(game; 0xa1f) == 3 || *ram_ptr!(game; 0xa1f) == 20 {
        QueueSfx1_Max9(0x32, game);
    }
    if (word!(game.equipped_items) & 0x20) != 0 {
        word!(game.samus_pose = kPose_9B_FaceF_VariaGravitySuit as u16);
    } else {
        word!(game.samus_pose = kPose_00_FaceF_Powersuit as u16);
    }
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    CallSomeSamusCode(0x15, game);
    word!(game.samus_x_pos = (word!(game.layer1_x_pos)).wrapping_add(120));
    word!(game.samus_prev_x_pos = (word!(game.layer1_x_pos)).wrapping_add(120));
    word!(game.samus_y_pos = (word!(game.layer1_y_pos)).wrapping_add(136));
    word!(game.samus_prev_y_pos = (word!(game.layer1_y_pos)).wrapping_add(136));
    QueueSfx2_Max6(0x56, game);
    static unk_91D59B: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0x41, 0x26, 0xd5a2 as VoidP);
    SpawnHdmaObject(0x91, &raw const unk_91D59B as *const c_void, game);
}
pub unsafe fn GravitySuitPickup(game: &mut Game) {
    *ram_ptr!(game; 0xdf0) = 48;
    *ram_ptr!(game; 0xdf1) = 73;
    *ram_ptr!(game; 0xdf2) = -112i32 as u8;
    *ram_ptr!(game; 0xdf3) = 1;
    Samus_CancelSpeedBoost(game);
    SetHiLo(ram_ptr!(game, u16; 0xb42), ram_ptr!(game, u16; 0xb44), 0);
    word! { game, 0;
        samus_x_base_speed, samus_x_base_subspeed, samus_y_subspeed, samus_y_speed, samus_y_dir,
        used_for_ball_bounce_on_landing, samus_x_accel_mode, elevator_status, substate,
        suit_pickup_light_beam_pos,
    }
    word!(game.suit_pickup_light_beam_widening_speed = 256);
    let mut i: i32 = 510;
    while i >= 0 {
        word!(game.hdma_table_1[(i >> 1)] = 255);
        i -= 2;
    }
    if *ram_ptr!(game; 0xa1f) as i32 == kMovementType_03_SpinJumping as i32
        || *ram_ptr!(game; 0xa1f) as i32 == kMovementType_14_WallJumping as i32
    {
        QueueSfx1_Max9(0x32, game);
    }
    if (word!(game.equipped_items) & 1) != 0 {
        word!(game.samus_pose = kPose_9B_FaceF_VariaGravitySuit as u16);
    } else {
        word!(game.samus_pose = kPose_00_FaceF_Powersuit as u16);
    }
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    CallSomeSamusCode(0x15, game);
    word!(game.samus_x_pos = (word!(game.layer1_x_pos)).wrapping_add(120));
    word!(game.samus_prev_x_pos = (word!(game.layer1_x_pos)).wrapping_add(120));
    word!(game.samus_y_pos = (word!(game.layer1_y_pos)).wrapping_add(136));
    word!(game.samus_prev_y_pos = (word!(game.layer1_y_pos)).wrapping_add(136));
    QueueSfx2_Max6(0x56, game);
    static unk_91D673: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0x41, 0x26, 0xd67a as VoidP);
    SpawnHdmaObject(0x91, &raw const unk_91D673 as *const c_void, game);
}
pub unsafe fn InitializeSuitPickupHdma(game: &mut Game) {
    *ram_ptr!(game; 0x69) = 19;
    *ram_ptr!(game; 0x6c) = 19;
    *ram_ptr!(game; 0x6b) = 4;
    *ram_ptr!(game; 0x6d) = 4;
    *ram_ptr!(game; 0x6e) = 16;
    *ram_ptr!(game; 0x60) = 0;
    *ram_ptr!(game; 0x61) = 2;
    *ram_ptr!(game; 0x62) = 32;
    *ram_ptr!(game; 0x71) = 51;
    *ram_ptr!(game; 0x74, 0) = *ram_ptr!(game; 0xdf0);
    *ram_ptr!(game; 0x74, 1) = *ram_ptr!(game; 0xdf1);
    *ram_ptr!(game; 0x74, 2) = *ram_ptr!(game; 0xdf2);
    mov24(ram_ptr!(game, LongPtr; 0xa88), 0x9800e4);
    mov24(ram_ptr!(game, LongPtr; 0xa8b), 0x98c8e4);
    mov24(ram_ptr!(game, LongPtr; 0xa8e), 0x999098);
    word!(game.hdma_var_1 = 0);
}
static off_91D72D: [Option<Func_U8>; 11] = callback_table! { unsafe Func_U8;
    Samus_HandleScrewAttackSpeedBoostingPals, Samus_SpeedBoosterShinePals,
    { transmute::<Option<unsafe fn(&mut Game) -> ()>, Option<Func_U8>>(Some(
                HandleMiscSamusPalette as unsafe fn(&mut Game) -> (),
            )) },
    None, None, None, Samus_HandleShinesparkingPals, Samus_HandleCrystalFlashPals,
    Samus_HandleXrayPals, HandleVisorPalette, nullsub_164,
};
pub unsafe fn Samus_HandlePalette(game: &mut Game) {
    if (word!(game.samus_special_super_palette_flags) & 0x8000) == 0
        && (HandleBeamChargePalettes(game) & 1 != 0
            || off_91D72D[word!(game.timer_for_shine_timer) as usize]
                .expect("non-null function pointer")(game)
                & 1
                == 0)
    {
        CopyToSamusSuitPalette(
            crate::rtl::rom_table_word!(
                game,
                0x91d727,
                (word!(game.samus_suit_palette_index) >> 1) as isize
            ),
            game,
        );
    }
    HandleMiscSamusPalette(game);
}
pub unsafe fn HandleBeamChargePalettes(game: &mut Game) -> u8 {
    if word!(game.charged_shot_glow_timer) != 0 {
        if word!(game.hyper_beam_flag) != 0 {
            if (word!(game.charged_shot_glow_timer) & 1) == 0 {
                if (word!(game.charged_shot_glow_timer) & 0x1e) == 0 {
                    word!(game.charged_shot_glow_timer = 0);
                    return 1;
                }
                CopyToSamusSuitPalette(
                    crate::rtl::rom_table_word!(
                        game,
                        0x91d829,
                        ((word!(game.charged_shot_glow_timer) as i32 & 0x1e) >> 1) as isize,
                    ),
                    game,
                );
            }
            word!(game.charged_shot_glow_timer, wrapping_sub(1));
            return 0;
        }
        if word!(game.charged_shot_glow_timer, wrapping_sub(1), reread) != 0 {
            let mut i: i32 = 28;
            while i >= 0 {
                *(ram_ptr!(game, u16; 0xc000)).offset(((i >> 1) + 193) as isize) = 1023;
                i -= 2;
            }
            0
        } else {
            1
        }
    } else if word!(game.grapple_beam_function) as i32 == 0x9bc4f0 & 0xffff
        && word!(game.flare_counter) != 0
        && word!(game.flare_counter) as i32 - 60i32 & 0x8000 == 0
    {
        let mut R36: u16 = 0;
        if word!(game.samus_contact_damage_index) == 4 {
            R36 = crate::rtl::rom_table_word!(
                game,
                0x91d7ff,
                (word!(game.samus_suit_palette_index) >> 1) as isize
            );
        } else {
            R36 = crate::rtl::rom_table_word!(
                game,
                0x91d7d5,
                (word!(game.samus_suit_palette_index) >> 1) as isize
            );
        }
        CopyToSamusSuitPalette(
            *(RomPtr_91(
                (R36).wrapping_add(word!(game.samus_charge_palette_index)),
                game,
            ) as *mut u16),
            game,
        );
        let mut v1: u16 = (word!(game.samus_charge_palette_index)).wrapping_add(2);
        if word!(game.samus_charge_palette_index) as i32 - 10i32 & 0x8000 == 0 {
            v1 = 0;
        }
        word!(game.samus_charge_palette_index = v1);
        0
    } else {
        word!(game.samus_charge_palette_index = 0);
        HandleVisorPalette(game)
    }
}
pub unsafe fn HandleVisorPalette(game: &mut Game) -> u8 {
    if word!(game.timer_for_shine_timer) == 8 {
        return 0;
    }
    if word!(game.fx_layer_blending_config_a) == 40 || word!(game.fx_layer_blending_config_a) == 42
    {
        let mut v1: u16 = (word!(game.samus_visor_palette_timer_index)).wrapping_sub(1);
        word!(game.samus_visor_palette_timer_index = v1);
        if v1 as u8 != 0 {
            return 0;
        }
        word!(game.samus_visor_palette_timer_index = (v1) | (5u16));
        word!(
            game.palette_buffer[196] = crate::rtl::rom_table_word!(
                game,
                0x9ba3c0,
                (*(&raw mut v1 as *mut u8).offset(1,) >> 1) as isize,
            )
        );
        let v2: i32 = *(&raw mut v1 as *mut u8).offset(1) as i32 + 2;
        if v2 - 12i32 & 0x8000 != 0 {
            word!(
                game.samus_visor_palette_timer_index = (u16::swap_bytes(v2 as u16))
                    | ((word!(game.samus_visor_palette_timer_index) as u8) as u16)
            );
        } else {
            word!(
                game.samus_visor_palette_timer_index =
                    ((word!(game.samus_visor_palette_timer_index) as u8) as u16) | (0x600u16)
            );
        }
        0
    } else {
        word!(game.samus_visor_palette_timer_index = 1537);
        0
    }
}
pub unsafe fn HandleMiscSamusPalette(game: &mut Game) {
    let mut current_block: u64;
    if word!(game.samus_special_super_palette_flags) == 0 {
        let mut v0: u16 = word!(game.samus_hurt_flash_counter);
        if word!(game.samus_hurt_flash_counter) == 0 {
            return;
        }
        if word!(game.samus_hurt_flash_counter) == 2 {
            if word!(game.cinematic_function) == 0
                && (word!(game.frame_handler_beta) as i32 != 0x90e8d9 & 0xffff
                    || word!(game.samus_pose) as i32 != kPose_54_FaceL_Knockback as i32)
            {
                QueueSfx1_Max6(0x35, game);
                current_block = 4767789648742619593;
            } else {
                v0 = word!(game.samus_hurt_flash_counter);
                current_block = 15619007995458559411;
            }
        } else {
            current_block = 15619007995458559411;
        }
        if current_block == 15619007995458559411 {
            if v0 as i32 - 7i32 & 0x8000 == 0 {
                current_block = 11050875288958768710;
            } else if (v0 & 1) != 0 {
                CopyToSamusSuitPalette(addr_word_9BA380 as u16, game);
                current_block = 11050875288958768710;
            } else {
                current_block = 4767789648742619593;
            }
        }
        if current_block == 4767789648742619593 {
            if word!(game.cinematic_function) != 0 {
                CopyToSamusSuitPalette(addr_word_9BA3A0 as u16, game);
            } else {
                Samus_LoadSuitPalette(game);
            }
        }
        let v1: u16 = (word!(game.samus_hurt_flash_counter)).wrapping_add(1);
        word!(game.samus_hurt_flash_counter = v1);
        if v1 == 40 {
            if word!(game.grapple_beam_function) as i32 == 0x9bc4f0 & 0xffff {
                if *ram_ptr!(game; 0xa1f) as i32 == kMovementType_03_SpinJumping as i32
                    || *ram_ptr!(game; 0xa1f) as i32 == kMovementType_14_WallJumping as i32
                {
                    CallSomeSamusCode(0x1c, game);
                } else if word!(game.flare_counter) as i32 - 16i32 & 0x8000 == 0
                    && (word!(game.button_config_shoot_x) & word!(game.joypad1_lastkeys)) != 0
                {
                    word!(game.play_resume_charging_beam_sfx = 1);
                }
            } else if word!(game.grapple_beam_function) as i32 + 0x37aa & 0x8000 != 0 {
                QueueSfx1_Max9(6, game);
            }
        } else if v1 as i32 - 60i32 & 0x8000 == 0 {
            word!(game.samus_hurt_flash_counter = 0);
        }
    } else {
        if (word!(game.samus_special_super_palette_flags) & 0x8000) != 0 {
            CopyToSamusSuitPalette(
                crate::rtl::rom_table_word!(
                    game,
                    0x91d99e,
                    word!(game.samus_charge_palette_index) as isize
                ),
                game,
            );
            let v2: bool =
                (word!(game.special_samus_palette_timer, wrapping_sub(1), reread) & 0x8000) != 0;
            if word!(game.special_samus_palette_timer) == 0 || v2 {
                word!(game.special_samus_palette_timer = word!(game.special_samus_palette_frame));
                if word!(game.samus_charge_palette_index, wrapping_add(1), reread) as i32 - 10i32
                    & 0x8000
                    == 0
                {
                    word!(game.samus_charge_palette_index = 0);
                }
            }
        } else {
            if (word!(game.samus_special_super_palette_flags) & 1) != 0 {
                CopyToSamusSuitPalette(
                    crate::rtl::rom_table_word!(
                        game,
                        0x91d998,
                        (word!(game.samus_suit_palette_index) >> 1) as isize,
                    ),
                    game,
                );
            } else {
                Samus_LoadSuitPalette(game);
            }
            word!(game.samus_special_super_palette_flags, wrapping_add(1));
        }
    }
}
pub unsafe fn Samus_HandleScrewAttackSpeedBoostingPals(game: &mut Game) -> u8 {
    let current_block: u64;
    if (word!(game.samus_suit_palette_index) & 4) == 0 {
        let r20: u16 = Samus_GetTop_R20(game);
        if liquid_at_height(game, r20) != LiquidAtHeight::None {
            return 1;
        }
    }
    if *ram_ptr!(game; 0xa1f) as i32 == kMovementType_03_SpinJumping as i32 {
        if (word!(game.equipped_items) & 8) == 0 {
            current_block = 14068947459426067103;
        } else if word!(game.samus_anim_frame) != 0 {
            if word!(game.samus_anim_frame) as i32 - 27i32 & 0x8000 == 0 {
                return 0;
            }
            current_block = 8831408221741692167;
        } else {
            current_block = 7431622773599773796;
        }
    } else if *ram_ptr!(game; 0xa1f) as i32 == kMovementType_14_WallJumping as i32 {
        if (word!(game.equipped_items) & 8) == 0 {
            return 1;
        }
        if word!(game.samus_anim_frame) as i32 - 3i32 & 0x8000 == 0 {
            current_block = 8831408221741692167;
        } else {
            current_block = 7431622773599773796;
        }
    } else {
        current_block = 14068947459426067103;
    }
    match current_block {
        8831408221741692167 => {
            let R36: u16 = crate::rtl::rom_table_word!(
                game,
                0x91da4a,
                (word!(game.samus_suit_palette_index) >> 1) as isize
            );
            let v1: u16 = *(RomPtr_91(
                (R36).wrapping_add(word!(game.special_samus_palette_frame)),
                game,
            ) as *mut u16);
            CopyToSamusSuitPalette(v1, game);
            let mut v2: u16 = (word!(game.special_samus_palette_frame)).wrapping_add(2);
            if word!(game.special_samus_palette_frame) >= 10 {
                v2 = 0;
            }
            word!(game.special_samus_palette_frame = v2);
            1
        }
        7431622773599773796 => {
            word!(game.special_samus_palette_frame = 0);
            1
        }
        _ => {
            if (word!(game.speed_boost_counter) & 0xff00) != 1024 {
                return 1;
            }
            let v3: bool =
                (word!(game.special_samus_palette_timer, wrapping_sub(1), reread) & 0x8000) != 0;
            if word!(game.special_samus_palette_timer) == 0 || v3 {
                word!(game.special_samus_palette_timer = 4);
                let R36_0: u16 = crate::rtl::rom_table_word!(
                    game,
                    0x91daa9,
                    (word!(game.samus_suit_palette_index) >> 1) as isize
                );
                if word!(game.special_samus_palette_frame) > 6 {
                    word!(game.special_samus_palette_frame = 6);
                }
                let v4: u16 = *(RomPtr_91(
                    (R36_0).wrapping_add(word!(game.special_samus_palette_frame)),
                    game,
                ) as *mut u16);
                CopyToSamusSuitPalette(v4, game);
                let mut v5: u16 = (word!(game.special_samus_palette_frame)).wrapping_add(2);
                if word!(game.special_samus_palette_frame) >= 6 {
                    v5 = 6;
                }
                word!(game.special_samus_palette_frame = v5);
            }
            1
        }
    }
}
pub unsafe fn Samus_SpeedBoosterShinePals(game: &mut Game) -> u8 {
    let mut v0: u16 = word!(game.samus_shine_timer);
    if word!(game.samus_shine_timer) == 170 {
        let v4: u16 = word!(game.samus_shine_timer);
        QueueSfx3_Max9(0xc, game);
        v0 = v4;
    }
    word!(game.samus_shine_timer = (v0).wrapping_sub(1));
    if ((v0) as i16).wrapping_sub(1) <= 0 {
        word! { game, 0; special_samus_palette_frame, timer_for_shine_timer }
        0
    } else {
        let R36: u16 = crate::rtl::rom_table_word!(
            game,
            0x91db10,
            (word!(game.samus_suit_palette_index) >> 1) as isize
        );
        let v1: u16 = *(RomPtr_91(
            (R36).wrapping_add(word!(game.special_samus_palette_frame)),
            game,
        ) as *mut u16);
        CopyToSamusSuitPalette(v1, game);
        let mut v2: u16 = (word!(game.special_samus_palette_frame)).wrapping_add(2);
        if word!(game.special_samus_palette_frame) as i32 - 10i32 & 0x8000 == 0 {
            v2 = 0;
        }
        word!(game.special_samus_palette_frame = v2);
        1
    }
}
pub unsafe fn Samus_HandleShinesparkingPals(game: &mut Game) -> u8 {
    let v0: bool = (word!(game.samus_shine_timer, wrapping_sub(1), reread) & 0x8000) != 0;
    if word!(game.samus_shine_timer) == 0 || v0 {
        word! { game, 0; timer_for_shine_timer, special_samus_palette_frame }
        0
    } else {
        let R36: u16 = crate::rtl::rom_table_word!(
            game,
            0x91db75,
            (word!(game.samus_suit_palette_index) >> 1) as isize
        );
        let v1: u16 = *(RomPtr_91(
            (R36).wrapping_add(word!(game.special_samus_palette_frame)),
            game,
        ) as *mut u16);
        CopyToSamusSuitPalette(v1, game);
        let mut v2: u16 = (word!(game.special_samus_palette_frame)).wrapping_add(2);
        if word!(game.special_samus_palette_frame) as i32 - 6i32 & 0x8000 == 0 {
            v2 = 0;
        }
        word!(game.special_samus_palette_frame = v2);
        1
    }
}
pub unsafe fn Samus_HandleCrystalFlashPals(game: &mut Game) -> u8 {
    if (word!(game.samus_shine_timer) & 0x8000) != 0 {
        WriteBeamPalette_A(word!(game.equipped_beams), game);
        word! { game, 0;
            timer_for_shine_timer, special_samus_palette_frame, special_samus_palette_timer,
            samus_shine_timer,
        }
        0
    } else {
        if (word!(game.samus_shine_timer, wrapping_sub(1), reread) as i16) <= 0 {
            word!(game.samus_shine_timer = 5);
            Samus_Copy6PalColors(
                crate::rtl::rom_table_word!(
                    game,
                    0x91dc28,
                    (word!(game.special_samus_palette_frame) >> 1) as isize
                ),
                game,
            );
            let mut v0: u16 = (word!(game.special_samus_palette_frame)).wrapping_add(2);
            if word!(game.special_samus_palette_frame) as i32 - 10i32 & 0x8000 == 0 {
                v0 = 0;
            }
            word!(game.special_samus_palette_frame = v0);
        }
        crate::update_place! { reference fresh15 = *(ram_ptr!(game, u16; 0xdf2)) => (*fresh15).wrapping_sub(1); }
        let v1: bool = (*fresh15 as i16) < 0;
        if *(ram_ptr!(game, u16; 0xdf2)) == 0 || v1 {
            *(ram_ptr!(game, u16; 0xdf2)) =
                *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91dc00))
                    as *mut SamusCrystalFlashPalTable)
                    .offset(0))
                .timer as *mut u8)
                    .offset(word!(game.special_samus_palette_timer) as i32 as isize)
                    as *mut u16);
            Samus_Copy10PalColors(
                *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91dc00))
                    as *mut SamusCrystalFlashPalTable)
                    .offset(0))
                .ptr as *mut u8)
                    .offset(word!(game.special_samus_palette_timer) as i32 as isize)
                    as *mut VoidP),
                game,
            );
            let mut v2: u16 = (word!(game.special_samus_palette_timer)).wrapping_add(4);
            if word!(game.special_samus_palette_timer) as i32 - 36i32 & 0x8000 == 0 {
                v2 = 0;
            }
            word!(game.special_samus_palette_timer = v2);
        }
        1
    }
}
pub unsafe fn Samus_Copy10PalColors(v0: u16, game: &mut Game) {
    memcpy(
        (ram_ptr!(game, u16; 0xc000)).offset(224) as *mut c_void,
        RomPtr_9B(v0, game) as *const c_void,
        20 as size_t,
    );
}
pub unsafe fn Samus_Copy6PalColors(j: u16, game: &mut Game) {
    memcpy(
        (ram_ptr!(game, u16; 0xc000)).offset(234) as *mut c_void,
        RomPtr_9B(j, game) as *const c_void,
        12 as size_t,
    );
}
pub unsafe fn Samus_HandleXrayPals(game: &mut Game) -> u8 {
    if (word!(game.demo_timer_counter) & 0x8000) != 0 {
        word! { game, 0;
            timer_for_shine_timer, special_samus_palette_frame, special_samus_palette_timer,
            demo_timer_counter,
        }
        0
    } else {
        if word!(game.demo_timer_counter) == 0 {
            if word!(game.demo_input_pre_instr) as i32 - 2i32 & 0x8000 != 0 {
                let v0: bool = (word!(game.special_samus_palette_timer, wrapping_sub(1), reread)
                    & 0x8000)
                    != 0;
                if word!(game.special_samus_palette_timer) == 0 || v0 {
                    word!(game.special_samus_palette_timer = 5);
                    word!(
                        game.palette_buffer[196] = crate::rtl::rom_table_word!(
                            game,
                            0x9ba3c0,
                            (word!(game.special_samus_palette_frame) >> 1) as isize,
                        )
                    );
                    if word!(game.special_samus_palette_frame) as i32 - 4i32 & 0x8000 != 0 {
                        word!(game.special_samus_palette_frame, wrapping_add(2));
                    }
                }
                return 1;
            }
            word!(game.special_samus_palette_frame = 6);
            word! { game, 1; special_samus_palette_timer, demo_timer_counter }
        }
        let v2: bool =
            (word!(game.special_samus_palette_timer, wrapping_sub(1), reread) & 0x8000) != 0;
        if word!(game.special_samus_palette_timer) != 0 && !v2 {
            return 1;
        }
        word!(game.special_samus_palette_timer = 5);
        word!(
            game.palette_buffer[196] = crate::rtl::rom_table_word!(
                game,
                0x9ba3c0,
                (word!(game.special_samus_palette_frame) >> 1) as isize,
            )
        );
        let mut v3: u16 = (word!(game.special_samus_palette_frame)).wrapping_add(2);
        if word!(game.special_samus_palette_frame) as i32 - 10i32 & 0x8000 == 0 {
            v3 = 6;
        }
        word!(game.special_samus_palette_frame = v3);
        1
    }
}
pub fn nullsub_164(game: &mut Game) -> u8 {
    0
}
crate::forward_calls! { pub unsafe fn (k: u16, game: &mut Game);
    CopyToSamusSuitPalette => memcpy(
        (ram_ptr!(game, u16; 0xc000)).offset(192) as *mut c_void,
        RomPtr_9B(k, game) as *mut u16 as *const c_void,
        32 as size_t,
    );
    CopyToSamusSuitTargetPalette => memcpy(
        (ram_ptr!(game, u16; 0xc200)).offset(192) as *mut c_void,
        RomPtr_9B(k, game) as *mut u16 as *const c_void,
        32 as size_t,
    );
}
pub unsafe fn Samus_CancelSpeedBoost(game: &mut Game) {
    if word!(game.samus_has_momentum_flag) != 0 {
        word! { game, 0;
            samus_has_momentum_flag, speed_boost_counter, special_samus_palette_frame,
            special_samus_palette_timer,
        }
        if (word!(game.equipped_items) & 0x20) != 0 {
            CopyToSamusSuitPalette(addr_kSamusPalette_GravitySuit as u16, game);
        } else if (word!(game.equipped_items) & 1) != 0 {
            CopyToSamusSuitPalette(addr_kSamusPalette_VariaSuit as u16, game);
        } else {
            CopyToSamusSuitPalette(addr_kSamusPalette_PowerSuit as u16, game);
        }
    }
    if (word!(game.speed_echoes_index) & 0x8000) == 0 {
        word!(game.speed_echoes_index = -1i32 as u16);
        if *ram_ptr!(game; 0xa1e) == 4 {
            word!(game.speed_echo_xspeed[0] = -8i32 as u16);
            word!(game.speed_echo_xspeed[1] = -8i32 as u16);
        } else {
            word! { game, 8; speed_echo_xspeed[0], speed_echo_xspeed[1] }
        }
    }
}
pub unsafe fn Samus_LoadSuitPalette(game: &mut Game) {
    if (word!(game.equipped_items) & 0x20) != 0 {
        CopyToSamusSuitPalette(addr_kSamusPalette_GravitySuit as u16, game);
    } else if (word!(game.equipped_items) & 1) != 0 {
        CopyToSamusSuitPalette(addr_kSamusPalette_VariaSuit as u16, game);
    } else {
        CopyToSamusSuitPalette(addr_kSamusPalette_PowerSuit as u16, game);
    };
}
pub unsafe fn Samus_LoadSuitTargetPalette(game: &mut Game) {
    if (word!(game.equipped_items) & 0x20) != 0 {
        CopyToSamusSuitTargetPalette(addr_kSamusPalette_GravitySuit as u16, game);
    } else if (word!(game.equipped_items) & 1) != 0 {
        CopyToSamusSuitTargetPalette(addr_kSamusPalette_VariaSuit as u16, game);
    } else {
        CopyToSamusSuitTargetPalette(addr_kSamusPalette_PowerSuit as u16, game);
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
pub unsafe fn Xray_Initialize(game: &mut Game) -> u8 {
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
                    && word!(game.samus_pose) as i32 - kPose_E8_FaceR_Drained_CrouchFalling as i32
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
    word!(game.time_is_frozen_flag = 1);
    word!(game.samus_special_transgfx_index = 5);
    let mut i: i32 = 510;
    while i >= 0 {
        word!(game.hdma_table_1[(i >> 1)] = 255);
        i -= 2;
    }
    DisableEprojs(game);
    DisablePLMs(game);
    DisableAnimtiles(game);
    DisablePaletteFx(game);
    mov24(ram_ptr!(game, LongPtr; 0xa88), 0x980001);
    *((ram_ptr!(game, u16; 0xa8a) as *mut u8).offset(1) as *mut u16) = 0;
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
pub unsafe fn ResponsibleForXrayStandupGlitch(game: &mut Game) {
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
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    word!(game.samus_last_different_pose = word!(game.samus_prev_pose));
    *(ram_ptr!(game, u16; 0xa26)) = *(ram_ptr!(game, u16; 0xa22));
    word!(game.samus_prev_pose = word!(game.samus_pose));
    *(ram_ptr!(game, u16; 0xa22)) = *(ram_ptr!(game, u16; 0xa1e));
    word!(game.samus_movement_handler = ((0x90a337i32) as u16));
    word!(game.samus_input_handler = ((0x90e913i32) as u16));
    word!(game.demo_timer_counter = -1i32 as u16);
    let r18: u16 = (((*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
        as *mut SamusPoseParams)
        .offset(word!(game.samus_pose) as isize))
    .y_radius) as u16)
        .wrapping_sub(word!(game.samus_y_radius));
    if (r18 & 0x8000) == 0 {
        word!(game.samus_y_pos, wrapping_sub(r18));
        word!(game.samus_prev_y_pos = word!(game.samus_y_pos));
    }
    EnableEprojs(game);
    EnablePLMs(game);
    EnableAnimtiles(game);
    EnablePaletteFx(game);
}
pub unsafe fn MakeSamusFaceForward(game: &mut Game) {
    if (word!(game.equipped_items) & 0x20) != 0 || (word!(game.equipped_items) & 1) != 0 {
        word!(game.samus_pose = kPose_9B_FaceF_VariaGravitySuit as u16);
    } else {
        word!(game.samus_pose = kPose_00_FaceF_Powersuit as u16);
    }
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    word!(game.samus_last_different_pose = word!(game.samus_prev_pose));
    *(ram_ptr!(game, u16; 0xa26)) = *(ram_ptr!(game, u16; 0xa22));
    word!(game.samus_prev_pose = word!(game.samus_pose));
    *(ram_ptr!(game, u16; 0xa22)) = *(ram_ptr!(game, u16; 0xa1e));
    if word!(game.samus_y_radius) != 24 {
        word!(game.samus_y_pos, wrapping_sub(3));
        word!(game.samus_prev_y_pos = word!(game.samus_y_pos));
    }
    word!(game.frame_handler_alfa = ((0x90e713i32) as u16));
    word!(game.frame_handler_beta = ((0x90e8dci32) as u16));
    word!(game.samus_new_pose = -1i32 as u16);
    word!(game.samus_new_pose_interrupted = -1i32 as u16);
    word!(game.samus_new_pose_transitional = -1i32 as u16);
    word! { game, 0;
        samus_momentum_routine_index, samus_special_transgfx_index, samus_hurt_switch_index,
    }
    CallSomeSamusCode(0x1f, game);
    SetHiLo(ram_ptr!(game, u16; 0xb42), ram_ptr!(game, u16; 0xb44), 0);
    word! { game, 0;
        samus_x_base_speed, samus_x_base_subspeed, samus_y_subspeed, samus_y_speed, samus_y_dir,
        used_for_ball_bounce_on_landing, samus_x_accel_mode, flare_counter, flare_animation_frame,
        flare_slow_sparks_anim_frame, flare_fast_sparks_anim_frame, flare_animation_timer,
        flare_slow_sparks_anim_timer, flare_fast_sparks_anim_timer,
    }
    Samus_LoadSuitPalette(game);
}
static kSomeMotherBrainScripts: [Option<Func_U8>; 5] = callback_table! { unsafe Func_U8;
    SomeMotherBrainScripts_0, SomeMotherBrainScripts_1, SomeMotherBrainScripts_2,
    SomeMotherBrainScripts_3_EnableHyperBeam, SomeMotherBrainScripts_4,
};
pub unsafe fn SomeMotherBrainScripts(a: u16, game: &mut Game) {
    if kSomeMotherBrainScripts[a as usize].expect("non-null function pointer")(game) & 1 != 0 {
        word!(game.samus_last_different_pose = word!(game.samus_prev_pose));
        *(ram_ptr!(game, u16; 0xa26)) = *(ram_ptr!(game, u16; 0xa22));
        word!(game.samus_prev_pose = word!(game.samus_pose));
        *(ram_ptr!(game, u16; 0xa22)) = *(ram_ptr!(game, u16; 0xa1e));
        word!(game.samus_new_pose = -1i32 as u16);
        word!(game.samus_new_pose_interrupted = -1i32 as u16);
        word!(game.samus_new_pose_transitional = -1i32 as u16);
        word! { game, 0;
            samus_momentum_routine_index, samus_special_transgfx_index, samus_hurt_switch_index,
        }
    }
}
pub unsafe fn SomeMotherBrainScripts_0(game: &mut Game) -> u8 {
    word!(
        game.samus_y_pos,
        |fresh27| (fresh27 as i32 - (21 - word!(game.samus_y_radius) as i32)) as u16
    );
    word!(game.samus_prev_y_pos = word!(game.samus_y_pos));
    if *ram_ptr!(game; 0xa1e) == 4 {
        word!(game.samus_pose = kPose_E9_FaceL_Drained_CrouchFalling as u16);
    } else {
        word!(game.samus_pose = kPose_E8_FaceR_Drained_CrouchFalling as u16);
    }
    word!(game.samus_anim_frame_skip = 2);
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    word!(
        game.samus_y_radius = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
            as *mut SamusPoseParams)
            .offset(word!(game.samus_pose) as isize))
        .y_radius as u16
    );
    word! { game, 0;
        samus_x_base_speed, samus_x_base_subspeed, samus_y_subspeed, samus_y_speed,
    }
    word!(game.samus_y_dir = 2);
    word! { game, 0;
        flare_counter, flare_animation_frame, flare_slow_sparks_anim_frame,
        flare_fast_sparks_anim_frame, flare_animation_timer, flare_slow_sparks_anim_timer,
        flare_fast_sparks_anim_timer,
    }
    Samus_LoadSuitPalette(game);
    1
}
pub unsafe fn SomeMotherBrainScripts_1(game: &mut Game) -> u8 {
    word!(game.samus_anim_frame_timer = 16);
    word!(game.samus_anim_frame = 0);
    if *ram_ptr!(game; 0xa1e) == 4 {
        word!(game.samus_pose = kPose_EB_FaceL_Drained_Stand as u16);
    } else {
        word!(game.samus_pose = kPose_EA_FaceR_Drained_Stand as u16);
    }
    word!(game.frame_handler_gamma = ((0x90e90ei32) as u16));
    1
}
pub unsafe fn SomeMotherBrainScripts_2(game: &mut Game) -> u8 {
    if word!(game.samus_pose) as i32 == kPose_E8_FaceR_Drained_CrouchFalling as i32
        || word!(game.samus_pose) as i32 == kPose_E9_FaceL_Drained_CrouchFalling as i32
    {
        word!(game.samus_anim_frame_timer = 1);
        word!(game.samus_anim_frame = 13);
    } else if word!(game.samus_pose) as i32 == kPose_EA_FaceR_Drained_Stand as i32
        || word!(game.samus_pose) as i32 == kPose_EB_FaceL_Drained_Stand as i32
    {
        word!(game.samus_anim_frame_timer = 1);
        word!(game.samus_anim_frame = 4);
    }
    word!(
        game.samus_y_radius = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
            as *mut SamusPoseParams)
            .offset(word!(game.samus_pose) as isize))
        .y_radius as u16
    );
    word! { game, 0;
        samus_x_base_speed, samus_x_base_subspeed, samus_y_subspeed, samus_y_speed,
    }
    word!(game.samus_y_dir = 2);
    1
}
pub unsafe fn SomeMotherBrainScripts_3_EnableHyperBeam(game: &mut Game) -> u8 {
    word!(game.equipped_beams = 4105);
    UpdateBeamTilesAndPalette(game);
    SpawnPalfxObject(addr_stru_8DE1F0 as u16, game);
    word!(game.hyper_beam_flag = ((0x918000i32) as u16));
    word!(game.play_resume_charging_beam_sfx = 0);
    0
}
pub unsafe fn SomeMotherBrainScripts_4(game: &mut Game) -> u8 {
    word!(game.samus_anim_frame_timer = 16);
    word!(game.samus_anim_frame = 8);
    if *ram_ptr!(game; 0xa1e) == 4 {
        word!(game.samus_pose = kPose_E9_FaceL_Drained_CrouchFalling as u16);
    } else {
        word!(game.samus_pose = kPose_E8_FaceR_Drained_CrouchFalling as u16);
    }
    1
}
pub fn nullsub_17(game: &mut Game) {}
static off_91E6E1: [Option<Func_V>; 28] = callback_table! { unsafe Func_V;
    SamusFunc_E633_0, nullsub_17, nullsub_17, SamusFunc_E633_3, SamusFunc_E633_4, nullsub_17,
    nullsub_17, nullsub_17, SamusFunc_E633_4, nullsub_17, nullsub_17, nullsub_17, nullsub_17,
    nullsub_17, nullsub_17, nullsub_17, nullsub_17, SamusFunc_E633_17, SamusFunc_E633_17,
    SamusFunc_E633_17, SamusFunc_E633_20, nullsub_17, nullsub_17, nullsub_17, nullsub_17,
    nullsub_17, nullsub_17, nullsub_17,
};
pub unsafe fn SamusFunc_E633(game: &mut Game) {
    off_91E6E1[*ram_ptr!(game; 0xa1f) as usize].expect("non-null function pointer")(game);
    if (word!(game.equipped_items) & 0x2000) != 0 {
        if word!(game.samus_has_momentum_flag) != 0 && word!(game.speed_boost_counter) == 0 {
            word!(game.special_samus_palette_timer = word!(game.speed_boost_counter));
            word!(game.special_samus_palette_frame = 0);
            word!(game.speed_boost_counter = crate::rtl::rom_table_word!(game, 0x91b61f, 0));
        }
    } else {
        word! { game, 0;
            speed_echoes_index, speed_echo_xspeed[0], speed_echo_xspeed[1], samus_has_momentum_flag,
            speed_boost_counter, special_samus_palette_frame, special_samus_palette_timer,
            speed_echo_xpos[0], speed_echo_xpos[1], speed_echo_ypos[0], speed_echo_ypos[1],
        }
    }
    if word!(game.grapple_beam_function) as i32 == 0x9bc4f0 & 0xffff {
        if (word!(game.equipped_beams) & 0x1000) != 0 {
            if word!(game.flare_counter) as i32 - 16i32 & 0x8000 == 0 {
                QueueSfx1_Max6(0x41, game);
            }
        } else {
            word! { game, 0;
                flare_counter, flare_animation_frame, flare_slow_sparks_anim_frame,
                flare_fast_sparks_anim_frame, flare_animation_timer, flare_slow_sparks_anim_timer,
                flare_fast_sparks_anim_timer,
            }
        }
    } else {
        LoadProjectilePalette(2, game);
        QueueSfx1_Max6(6, game);
    }
    Samus_LoadSuitPalette(game);
    if word!(game.samus_health) as i32 - 31i32 & 0x8000 != 0 {
        QueueSfx3_Max6(2, game);
    }
}
pub unsafe fn Samus_UpdatePreviousPose_0(game: &mut Game) {
    word!(game.samus_last_different_pose = word!(game.samus_prev_pose));
    *(ram_ptr!(game, u16; 0xa26)) = *(ram_ptr!(game, u16; 0xa22));
    word!(game.samus_prev_pose = word!(game.samus_pose));
    *(ram_ptr!(game, u16; 0xa22)) = *(ram_ptr!(game, u16; 0xa1e));
}
pub unsafe fn SamusFunc_E633_0(game: &mut Game) {
    let refresh_pose: bool = if word!(game.samus_pose) != 0 {
        (word!(game.samus_pose) as i32 == kPose_9B_FaceF_VariaGravitySuit as i32
            && (word!(game.equipped_items) & 1) == 0
            && (word!(game.equipped_items) & 0x20) == 0)
            && ({
                word!(game.samus_pose = kPose_00_FaceF_Powersuit as u16);
                true
            })
    } else {
        ((word!(game.equipped_items) & 1) != 0 || (word!(game.equipped_items) & 0x20) != 0)
            && ({
                word!(game.samus_pose = kPose_9B_FaceF_VariaGravitySuit as u16);
                true
            })
    };
    if refresh_pose {
        SamusFunc_F433(game);
        Samus_SetAnimationFrameIfPoseChanged(game);
        Samus_UpdatePreviousPose_0(game);
    };
}
pub unsafe fn SamusFunc_E633_3(game: &mut Game) {
    if *ram_ptr!(game; 0xa1e) == 4 {
        *(ram_ptr!(game, u16; 0xa22)) = 260;
    } else {
        *(ram_ptr!(game, u16; 0xa22)) = 264;
    }
    let start_spin_jump: bool = if word!(game.samus_pose) as i32
        != kPose_81_FaceR_Screwattack as i32
        && word!(game.samus_pose) as i32 != kPose_82_FaceL_Screwattack as i32
    {
        !(word!(game.samus_pose) as i32 != kPose_1B_FaceR_SpaceJump as i32
            && word!(game.samus_pose) as i32 != kPose_1C_FaceL_SpaceJump as i32)
            && (!((word!(game.equipped_items) & 8) != 0)
                || ({
                    if *ram_ptr!(game; 0xa1e) == 4 {
                        word!(game.samus_pose = kPose_82_FaceL_Screwattack as u16);
                    } else {
                        word!(game.samus_pose = kPose_81_FaceR_Screwattack as u16);
                    }
                    false
                }))
    } else {
        ((word!(game.equipped_items) & 8) == 0)
    };
    if start_spin_jump {
        if *ram_ptr!(game; 0xa1e) == 4 {
            word!(game.samus_pose = kPose_1A_FaceL_SpinJump as u16);
        } else {
            word!(game.samus_pose = kPose_19_FaceR_SpinJump as u16);
        }
    }
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    if *ram_ptr!(game; 0xa1e) == 4 {
        *(ram_ptr!(game, u16; 0xa22)) = 772;
    } else {
        *(ram_ptr!(game, u16; 0xa22)) = 776;
    }
    Samus_UpdatePreviousPose_0(game);
}
pub unsafe fn SamusFunc_E633_4(game: &mut Game) {
    if (word!(game.equipped_items) & 2) != 0 {
        if *ram_ptr!(game; 0xa1e) == 4 {
            word!(game.samus_pose = kPose_7A_FaceL_Springball_Ground as u16);
        } else {
            word!(game.samus_pose = kPose_79_FaceR_Springball_Ground as u16);
        }
        SamusFunc_F433(game);
        Samus_SetAnimationFrameIfPoseChanged(game);
        Samus_UpdatePreviousPose_0(game);
    }
}
pub unsafe fn SamusFunc_E633_17(game: &mut Game) {
    if (word!(game.equipped_items) & 2) == 0 {
        if *ram_ptr!(game; 0xa1e) == 4 {
            word!(game.samus_pose = kPose_41_FaceL_Morphball_Ground as u16);
        } else {
            word!(game.samus_pose = kPose_1D_FaceR_Morphball_Ground as u16);
        }
        SamusFunc_F433(game);
        Samus_SetAnimationFrameIfPoseChanged(game);
        Samus_UpdatePreviousPose_0(game);
    }
}
pub unsafe fn SamusFunc_E633_20(game: &mut Game) {
    if (word!(game.equipped_items) & 8) != 0 {
        word!(game.samus_anim_frame = 23);
    } else {
        word!(game.samus_anim_frame = 3);
    };
}
pub fn nullsub_18(game: &mut Game) {}
static kSamus_HandleTransFromBlockColl: [Option<Func_V>; 6] = callback_table! { unsafe Func_V;
    nullsub_18, Samus_HandleTransFromBlockColl_1, Samus_HandleTransFromBlockColl_2,
    Samus_HandleTransFromBlockColl_3, Samus_HandleTransFromBlockColl_4,
    Samus_HandleTransFromBlockColl_5,
};
pub unsafe fn Samus_HandleTransFromBlockColl(game: &mut Game) {
    if word!(game.input_to_pose_calc) != 0 {
        kSamus_HandleTransFromBlockColl[word!(game.input_to_pose_calc) as u8 as usize]
            .expect("non-null function pointer")(game);
    }
}
pub unsafe fn Samus_HandleTransFromBlockColl_3(game: &mut Game) {
    word!(game.samus_new_pose = word!(game.samus_pose));
    word!(game.samus_momentum_routine_index = 5);
}
pub use self::nullsub_164 as nullsub_18_U8;
pub use self::Samus_HandleTransFromBlockColl_3 as Samus_HandleTransFromBlockColl_4;
static off_91E951: [Option<Func_U8>; 6] = callback_table! { unsafe Func_U8;
    Samus_HandleTransFromBlockColl_1_0, Samus_HandleTransFromBlockColl_1_1,
    Samus_HandleTransFromBlockColl_1_2, Samus_HandleTransFromBlockColl_1_3, nullsub_18_U8,
    Samus_HandleTransFromBlockColl_1_5,
};
pub unsafe fn Samus_HandleTransFromBlockColl_2(game: &mut Game) {
    if *(ram_ptr!(game, u16; 0xdc6) as *mut u8).offset(1) != 4 {
        let v0: u16 =
            (4u16).wrapping_mul((*(ram_ptr!(game, u16; 0xdc6) as *mut u8).offset(1)) as u16);
        if *ram_ptr!(game; 0xa1e) == 4 {
            word!(
                game.samus_new_pose =
                    crate::rtl::rom_table_word!(game, 0x91e921, ((v0 as i32 >> 1) + 1) as isize,)
            );
        } else {
            word!(
                game.samus_new_pose =
                    crate::rtl::rom_table_word!(game, 0x91e921, (v0 >> 1) as isize)
            );
        }
        word!(game.samus_momentum_routine_index = 5);
    }
}
pub unsafe fn Samus_HandleTransFromBlockColl_1(game: &mut Game) {
    if *(ram_ptr!(game, u16; 0xdc6) as *mut u8).offset(1) != 4 {
        if off_91E951[*(ram_ptr!(game, u16; 0xdc6) as *mut u8).offset(1isize) as usize]
            .expect("non-null function pointer")(game)
            & 1
            != 0
        {
            word!(game.samus_momentum_routine_index = 0);
        } else {
            word!(game.samus_momentum_routine_index = 5);
        }
    }
}
pub unsafe fn Samus_HandleTransFromBlockColl_1_0(game: &mut Game) -> u8 {
    if *ram_ptr!(game; 0xa23) as i32 == kMovementType_03_SpinJumping as i32
        || *ram_ptr!(game; 0xa23) as i32 == kMovementType_14_WallJumping as i32
    {
        if *ram_ptr!(game; 0xa1e) == 4 {
            word!(game.samus_new_pose = kPose_A7_FaceL_LandSpinJump as u16);
        } else {
            word!(game.samus_new_pose = kPose_A6_FaceR_LandSpinJump as u16);
        }
        0
    } else {
        let mut v0: i16 = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
            as *mut SamusPoseParams)
            .offset(word!(game.samus_pose) as isize))
        .direction_shots_fired as i16;
        if v0 == 255 {
            if *ram_ptr!(game; 0xa1e) == 4 {
                word!(game.samus_new_pose = kPose_A5_FaceL_LandJump as u16);
            } else {
                word!(game.samus_new_pose = kPose_A4_FaceR_LandJump as u16);
            }
            0
        } else {
            if !(v0 != 2 && v0 != 7) {
                if (word!(game.button_config_shoot_x) & word!(game.joypad1_lastkeys)) == 0 {
                    v0 = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
                        as *mut SamusPoseParams)
                        .offset(word!(game.samus_pose) as isize))
                    .direction_shots_fired as i16;
                } else {
                    if *ram_ptr!(game; 0xa1e) == 4 {
                        word!(game.samus_new_pose = kPose_E7_FaceL_LandJump_Fire as u16);
                    } else {
                        word!(game.samus_new_pose = kPose_E6_FaceR_LandJump_Fire as u16);
                    }
                    return 0;
                }
            }
            word!(game.samus_new_pose = crate::rtl::rom_table_word!(game, 0x91e9f3, v0 as isize));
            0
        }
    }
}
pub unsafe fn Samus_HandleTransFromBlockColl_1_1(game: &mut Game) -> u8 {
    let mut v0: i16 = (2i16).wrapping_mul((word!(game.used_for_ball_bounce_on_landing)) as i16);
    if 2 * word!(game.used_for_ball_bounce_on_landing) as i32 != 0 {
        if v0 == 2 {
            word!(game.samus_new_pose = word!(game.samus_pose));
            return 0;
        }
        if v0 != 4 {
            Unreachable(game);
        }
    } else if word!(game.samus_y_speed) as i32 - 3i32 & 0x8000 == 0 {
        word!(game.samus_new_pose = word!(game.samus_pose));
        return 0;
    }
    if *ram_ptr!(game; 0xa1e) == 4 {
        word!(game.samus_new_pose = kPose_41_FaceL_Morphball_Ground as u16);
    } else {
        word!(game.samus_new_pose = kPose_1D_FaceR_Morphball_Ground as u16);
    }
    0
}
pub unsafe fn Samus_HandleTransFromBlockColl_1_2(game: &mut Game) -> u8 {
    if *ram_ptr!(game; 0xa1e) == 4 {
        word!(game.samus_new_pose = 66);
    } else {
        word!(game.samus_new_pose = 32);
    }
    0
}
pub unsafe fn Samus_HandleTransFromBlockColl_1_3(game: &mut Game) -> u8 {
    if (word!(game.button_config_jump_a) & word!(game.joypad1_lastkeys)) != 0 {
        word!(game.samus_new_pose = word!(game.samus_pose));
        return 0;
    }
    let mut v1: i16 =
        (2i16).wrapping_mul((word!(game.used_for_ball_bounce_on_landing) as u8) as i16);
    if v1 != 0 {
        if v1 == 2 {
            word!(game.samus_new_pose = word!(game.samus_pose));
            return 0;
        }
        if v1 != 4 {
            Unreachable(game);
        }
    } else if word!(game.samus_y_speed) as i32 - 3i32 & 0x8000 == 0 {
        word!(game.samus_new_pose = word!(game.samus_pose));
        return 0;
    }
    if *ram_ptr!(game; 0xa1e) == 4 {
        word!(game.samus_new_pose = kPose_7A_FaceL_Springball_Ground as u16);
    } else {
        word!(game.samus_new_pose = 121);
    }
    0
}
pub unsafe fn Samus_HandleTransFromBlockColl_1_5(game: &mut Game) -> u8 {
    word!(game.samus_new_pose = word!(game.samus_pose));
    0
}
pub unsafe fn Samus_HandleTransFromBlockColl_5(game: &mut Game) {
    if *ram_ptr!(game; 0xa1e) == 4 {
        word!(game.samus_new_pose = kPose_84_FaceL_Walljump as u16);
    } else {
        word!(game.samus_new_pose = kPose_83_FaceR_Walljump as u16);
    }
    word!(game.samus_momentum_routine_index = 5);
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

    let mut amt: i32 = 0;
    if word!(game.samus_collides_with_solid_enemy) != 0 && *ram_ptr!(game; 0xa1f) == 1 {
        word!(
            game.samus_new_pose = crate::rtl::rom_table_word!(
                game,
                0x91eb74,
                *(&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
                    as *mut SamusPoseParams)
                    .offset(0))
                .direction_shots_fired)
                    .offset((8 * word!(game.samus_pose) as i32) as isize,) as isize,
            )
        );
        word!(game.samus_collides_with_solid_enemy = 0);
        return;
    }
    if word!(game.samus_new_pose) == 0xffff
        || (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629)) as *mut SamusPoseParams)
            .offset(word!(game.samus_new_pose) as isize))
        .movement_type
            != 1
    {
        word!(game.samus_collides_with_solid_enemy = 0);
        return;
    }
    let move_right: bool = if *ram_ptr!(game; 0xa1e) != 4 {
        word!(game.samus_collision_direction = 1);
        amt = (1i16 as i32) << 16;
        cres = Samus_CheckSolidEnemyColl(amt, game);
        (cres.collision == 0)
            && ({
                word!(game.samus_collision_direction = 1);
                amt = (1i16 as i32) << 16;
                true
            })
    } else {
        word!(game.samus_collision_direction = 0);
        amt = (1i16 as i32) << 16;
        cres_0 = Samus_CheckSolidEnemyColl(amt, game);
        !(cres_0.collision != 0)
            && ({
                amt = (-1i16 as i32) << 16;
                word!(game.samus_collision_direction = 0);
                true
            })
    };
    if move_right {
        Samus_MoveRight_NoSolidColl(amt, game);
        if (word!(game.samus_collision_flag) == 0) {
            word!(game.samus_collides_with_solid_enemy = 0);
            return;
        }
    }
    word!(
        game.samus_new_pose = crate::rtl::rom_table_word!(
            game,
            0x91eb74,
            *(&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
                as *mut SamusPoseParams)
                .offset(0))
            .direction_shots_fired)
                .offset((8 * word!(game.samus_new_pose) as i32) as isize,) as isize,
        )
    );
    word!(game.samus_collides_with_solid_enemy = 0);
}
pub fn nullsub_19(game: &mut Game) {}
pub fn nullsub_20(game: &mut Game) {}
pub fn nullsub_21(game: &mut Game) {}
pub fn nullsub_22(game: &mut Game) {}
static kSamus_HandleTransitionsA: [Option<Func_V>; 9] = callback_table! { unsafe Func_V;
    nullsub_18, Samus_HandleTransitionsA_1, Samus_HandleTransitionsA_2, nullsub_21, nullsub_22,
    Samus_HandleTransitionsA_5, Samus_HandleTransitionsA_6, Samus_HandleTransitionsA_7,
    Samus_HandleTransitionsA_8,
};
static kSamus_HandleTransitionsB: [Option<Func_V>; 11] = callback_table! { unsafe Func_V;
    nullsub_18, Samus_HandleTransitionsB_1, Samus_HandleTransitionsB_2,
    Samus_HandleTransitionsB_3, Samus_HandleTransitionsB_4, Samus_HandleTransitionsB_5,
    nullsub_19, nullsub_20, Samus_HandleTransitionsB_8, Samus_HandleTransitionsB_9,
    Samus_HandleTransitionsB_10,
};
static kSamus_HandleTransitionsC: [Option<Func_V>; 9] = callback_table! { unsafe Func_V;
    nullsub_18, Samus_HandleTransitionsC_1, Samus_HandleTransitionsC_2,
    Samus_HandleTransitionsC_3, Samus_HandleTransitionsC_4, Samus_HandleTransitionsC_5,
    Samus_HandleTransitionsC_6, Samus_HandleTransitionsC_7, Samus_HandleTransitionsC_8,
};
pub unsafe fn Samus_HandleTransitions(game: &mut Game) {
    let mut current_block: u64;
    if (word!(game.samus_new_pose_transitional) & 0x8000) == 0 {
        if word!(game.samus_hurt_switch_index) != 3 {
            if word!(game.samus_hurt_switch_index) == 1 {
                current_block = 5743517142682009007;
            } else {
                current_block = 6038634297008360266;
            }
        } else if word!(game.samus_special_transgfx_index) != 9 {
            current_block = 6038634297008360266;
        } else {
            current_block = 13536709405535804910;
        }
        if current_block == 13536709405535804910 {
        } else {
            if current_block == 6038634297008360266 {
                word!(game.samus_pose = word!(game.samus_new_pose_transitional));
                SamusFunc_F433(game);
                Samus_SetAnimationFrameIfPoseChanged(game);
            }
            kSamus_HandleTransitionsC[word!(game.samus_hurt_switch_index) as usize]
                .expect("non-null function pointer")(game);
            current_block = 10476341332908994874;
        }
    } else {
        current_block = 13536709405535804910;
    }
    if current_block == 13536709405535804910 {
        if (word!(game.samus_new_pose_interrupted) & 0x8000) == 0 {
            word!(game.samus_pose = word!(game.samus_new_pose_interrupted));
            if SamusFunc_F404(game) & 1 == 0 {
                kSamus_HandleTransitionsB[word!(game.samus_special_transgfx_index) as usize]
                    .expect("non-null function pointer")(game);
            }
            current_block = 10476341332908994874;
        } else {
            Samus_CheckWalkedIntoSomething(game);
            if (word!(game.samus_new_pose) & 0x8000) == 0 {
                word!(game.samus_pose = word!(game.samus_new_pose));
                if SamusFunc_F404(game) & 1 == 0 {
                    kSamus_HandleTransitionsA[word!(game.samus_momentum_routine_index) as usize]
                        .expect("non-null function pointer")(game);
                }
                current_block = 10476341332908994874;
            } else {
                current_block = 10093773643351532501;
            }
        }
    }
    if current_block == 10476341332908994874 {
        word!(game.samus_last_different_pose = word!(game.samus_prev_pose));
        *(ram_ptr!(game, u16; 0xa26)) = *(ram_ptr!(game, u16; 0xa22));
        word!(game.samus_prev_pose = word!(game.samus_pose));
        *(ram_ptr!(game, u16; 0xa22)) = *(ram_ptr!(game, u16; 0xa1e));
    }
    word!(game.input_to_pose_calc = 0);
}
pub unsafe fn Samus_HandleTransitionsA_1(game: &mut Game) {
    if word!(game.samus_x_base_speed) != 0 || word!(game.samus_x_base_subspeed) != 0 {
        AddToHiLo(
            ram_ptr!(game, u16; 0xb46),
            ram_ptr!(game, u16; 0xb48),
            (word!(game.samus_x_extra_run_speed) as u32) << 16
                | word!(game.samus_x_extra_run_subspeed) as u32,
        );
        word!(game.samus_x_accel_mode = 2);
        Samus_CancelSpeedBoost(game);
        SetHiLo(ram_ptr!(game, u16; 0xb42), ram_ptr!(game, u16; 0xb44), 0);
        SamusFunc_EC80(game);
    } else {
        Samus_HandleTransitionsA_2(game);
    };
}
pub unsafe fn Samus_HandleTransitionsA_6(game: &mut Game) {
    word! { game, 0; samus_x_accel_mode, samus_x_base_speed, samus_x_base_subspeed }
    Samus_HandleTransitionsA_8(game);
}
pub unsafe fn Samus_HandleTransitionsA_8(game: &mut Game) {
    Samus_CancelSpeedBoost(game);
    SetHiLo(ram_ptr!(game, u16; 0xb42), ram_ptr!(game, u16; 0xb44), 0);
    SamusFunc_EC80(game);
}
pub unsafe fn Samus_HandleTransitionsA_2(game: &mut Game) {
    word!(game.samus_x_accel_mode = 0);
    Samus_CancelSpeedBoost(game);
}
static word_91ED36: [u16; 12] = [5, 5, 9, 9, 0, 0, 0, 0, 0, 0, 0, 0];
pub unsafe fn Samus_HandleTransitionsA_7(game: &mut Game) {
    let mut amt: i32 = 0;
    let mut current_block_11: u64;
    if word!(game.samus_pose) as i32 - kPose_DB as i32 & 0x8000 != 0 {
        amt = (word_91ED36[(word!(game.samus_pose) as i32 - 53i32) as usize] as i16 as i32) << 16;
        if amt != 0 {
            current_block_11 = 7426089221157146073;
        } else {
            current_block_11 = 735147466149431745;
        }
    } else if word!(game.samus_pose) as i32 - kPose_F1_FaceR_CrouchTrans_AimU as i32 & 0x8000 == 0
        && word!(game.samus_pose) as i32 - kPose_F7_FaceR_StandTrans_AimU as i32 & 0x8000 != 0
    {
        amt = (5i16 as i32) << 16;
        current_block_11 = 7426089221157146073;
    } else {
        current_block_11 = 1856101646708284338;
    }
    if current_block_11 == 7426089221157146073 {
        word!(
            game.samus_y_radius = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
                as *mut SamusPoseParams)
                .offset(word!(game.samus_pose) as isize))
            .y_radius as u16
        );
        amt = Samus_CollDetectChangedPose(amt, game);
        current_block_11 = 735147466149431745;
    }
    if current_block_11 == 735147466149431745 {
        word!(game.samus_y_pos, |fresh40| (fresh40 as i32 + (amt >> 16))
            as u16);
        word!(game.samus_prev_y_pos = word!(game.samus_y_pos));
        if word!(game.used_for_ball_bounce_on_landing) != 0 {
            word! { game, 0;
                used_for_ball_bounce_on_landing, samus_y_subspeed, samus_y_speed, samus_y_dir,
            }
        }
    };
}
static kSamus_HandleTransitionsB_1: [Option<Func_U8>; 28] = callback_table! { unsafe Func_U8;
    Samus_HandleTransitionsB_1_0, Samus_HandleTransitionsB_1_0, Samus_HandleTransitionsB_1_0,
    Samus_HandleTransitionsB_1_0, Samus_HandleTransitionsB_1_4, Samus_HandleTransitionsB_1_0,
    Samus_HandleTransitionsB_1_6, Samus_HandleTransitionsB_1_7, Samus_HandleTransitionsB_1_4,
    Samus_HandleTransitionsB_1_4, Samus_HandleTransitionsB_1_10, Samus_HandleTransitionsB_1_11,
    Samus_HandleTransitionsB_1_11, Samus_HandleTransitionsB_1_0, Samus_HandleTransitionsB_1_11,
    Samus_HandleTransitionsB_1_11, Samus_HandleTransitionsB_1_0, Samus_HandleTransitionsB_1_4,
    Samus_HandleTransitionsB_1_4, Samus_HandleTransitionsB_1_4, Samus_HandleTransitionsB_1_0,
    Samus_HandleTransitionsB_1_0, Samus_HandleTransitionsB_1_10, Samus_HandleTransitionsB_1_11,
    Samus_HandleTransitionsB_1_11, Samus_HandleTransitionsB_1_11, Samus_HandleTransitionsB_1_11,
    Samus_HandleTransitionsB_1_11,
};
pub unsafe fn Samus_HandleTransitionsB_1(game: &mut Game) {
    kSamus_HandleTransitionsB_1[*ram_ptr!(game; 0xa23) as usize]
        .expect("non-null function pointer")(game);
    Samus_SetSpeedForKnockback_Y(game);
    word! { game, 0; bomb_jump_dir, samus_contact_damage_index }
    word!(game.samus_hurt_flash_counter = 1);
}
pub use self::nullsub_164 as Samus_HandleTransitionsB_1_10;
pub use self::nullsub_164 as Samus_HandleTransitionsB_1_11;
pub unsafe fn Samus_HandleTransitionsB_1_6(game: &mut Game) -> u8 {
    if word!(game.frame_handler_gamma) as i32 == 0x90e41b & 0xffff {
        0
    } else {
        Samus_HandleTransitionsB_1_0(game)
    }
}
pub unsafe fn Samus_HandleTransitionsB_1_0(game: &mut Game) -> u8 {
    if *ram_ptr!(game; 0xa1e) == 4 {
        if word!(game.knockback_x_dir) != 0 {
            if (word!(game.joypad1_lastkeys) & 0x200) != 0 {
                word!(game.knockback_dir = 5);
            } else {
                word!(game.knockback_dir = 2);
            }
        } else if (word!(game.joypad1_lastkeys) & 0x200) != 0 {
            word!(game.knockback_dir = 4);
        } else {
            word!(game.knockback_dir = 1);
        }
    } else if word!(game.knockback_x_dir) != 0 {
        if (word!(game.joypad1_lastkeys) & 0x100) != 0 {
            word!(game.knockback_dir = 5);
        } else {
            word!(game.knockback_dir = 2);
        }
    } else if (word!(game.joypad1_lastkeys) & 0x100) != 0 {
        word!(game.knockback_dir = 4);
    } else {
        word!(game.knockback_dir = 1);
    }
    word!(game.samus_movement_handler = ((0x90df38i32) as u16));
    1
}
pub unsafe fn Samus_HandleTransitionsB_1_4(game: &mut Game) -> u8 {
    if *ram_ptr!(game; 0xa1e) == 4 {
        word!(game.knockback_dir = 1);
    } else {
        word!(game.knockback_dir = 2);
    }
    word!(game.samus_movement_handler = ((0x90df38i32) as u16));
    0
}
pub unsafe fn Samus_HandleTransitionsB_1_7(game: &mut Game) -> u8 {
    if *ram_ptr!(game; 0xa1e) == 4 {
        word!(game.knockback_dir = 1);
    } else {
        word!(game.knockback_dir = 2);
    }
    word!(game.samus_movement_handler = ((0x90df38i32) as u16));
    1
}
pub unsafe fn Samus_HandleTransitionsB_2(game: &mut Game) {
    word!(game.knockback_dir = 0);
    word!(game.samus_movement_handler = ((0x90a337i32) as u16));
    word! { game, 0; samus_y_speed, samus_y_subspeed, samus_y_dir }
    Samus_AlignBottomWithPrevPose(game);
}
pub unsafe fn Samus_HandleTransitionsB_3(game: &mut Game) {
    word!(game.bomb_jump_dir = ((word!(game.bomb_jump_dir) as u8) as u16) | (0x800u16));
    word!(game.samus_movement_handler = ((0x90e025i32) as u16));
    if word!(game.samus_input_handler) as i32 != 0x90e91d & 0xffff {
        word!(game.samus_input_handler = ((0x90e90ei32) as u16));
    }
}
pub unsafe fn Samus_HandleTransitionsB_4(game: &mut Game) {
    Samus_InitJump(game);
}
pub unsafe fn Samus_HandleTransitionsB_5(game: &mut Game) {
    let begin_shinespark: bool;
    match *ram_ptr!(game; 0xa1f) as i32 {
        5 => {
            if *ram_ptr!(game; 0xa1e) == 4 {
                word!(game.xray_angle = 192);
            } else {
                word!(game.xray_angle = 64);
            }
            begin_shinespark = true;
        }
        0 | 21 | 1 => {
            if *ram_ptr!(game; 0xa1e) == 4 {
                word!(game.xray_angle = 192);
            } else {
                word!(game.xray_angle = 64);
            }
            begin_shinespark = true;
        }
        _ => begin_shinespark = false,
    }
    if begin_shinespark {
        word!(game.samus_anim_frame = 2);
        word!(game.samus_anim_frame_timer = 63);
        word!(game.samus_movement_handler = ((0x90e94fi32) as u16));
        word!(game.samus_input_handler = ((0x90e918i32) as u16));
        word!(game.timer_for_shine_timer = 8);
        word!(game.special_samus_palette_timer = 1);
        word! { game, 0;
            special_samus_palette_frame, samus_shine_timer, flare_counter, flare_animation_frame,
            flare_slow_sparks_anim_frame, flare_fast_sparks_anim_frame, flare_animation_timer,
            flare_slow_sparks_anim_timer, flare_fast_sparks_anim_timer,
        }
        QueueSfx1_Max6(9, game);
    };
}
pub unsafe fn Samus_HandleTransitionsB_8(game: &mut Game) {
    word!(game.samus_y_pos, wrapping_sub(5));
    word!(game.samus_prev_y_pos = word!(game.samus_y_pos));
    word!(game.frame_handler_alfa = ((0x90e8cdi32) as u16));
}
pub unsafe fn Samus_HandleTransitionsB_9(game: &mut Game) {
    GrappleBeamFunc_BD95(game);
    let mut v0: i16 =
        ((word!(game.samus_x_pos)) as i16).wrapping_sub((word!(game.samus_prev_x_pos)) as i16);
    if ((word!(game.samus_x_pos)) as i16).wrapping_sub((word!(game.samus_prev_x_pos)) as i16) < 0 {
        if v0 as i32 + 12 & 0x8000 != 0 {
            word!(game.samus_prev_x_pos = (word!(game.samus_x_pos)).wrapping_add(12));
        }
    } else if v0 as i32 - 13i32 & 0x8000 == 0 {
        word!(game.samus_prev_x_pos = (word!(game.samus_x_pos)).wrapping_sub(12));
    }
    let mut v1: i16 =
        ((word!(game.samus_y_pos)) as i16).wrapping_sub((word!(game.samus_prev_y_pos)) as i16);
    if ((word!(game.samus_y_pos)) as i16).wrapping_sub((word!(game.samus_prev_y_pos)) as i16) < 0 {
        if v1 as i32 + 12 & 0x8000 != 0 {
            word!(game.samus_prev_y_pos = (word!(game.samus_y_pos)).wrapping_add(12));
        }
    } else if v1 as i32 - 13i32 & 0x8000 == 0 {
        word!(game.samus_prev_y_pos = (word!(game.samus_y_pos)).wrapping_sub(12));
    }
    Samus_CancelSpeedBoost(game);
    word! { game, 0;
        samus_x_base_speed, samus_x_base_subspeed, samus_y_subspeed, samus_y_speed,
    }
    SetHiLo(ram_ptr!(game, u16; 0xb42), ram_ptr!(game, u16; 0xb44), 0);
}
pub unsafe fn Samus_HandleTransitionsB_10(game: &mut Game) {
    GrappleBeamFunc_BEEB(game);
    Samus_HandleTransitionsB_9B(game);
}
pub unsafe fn Samus_HandleTransitionsB_9B(game: &mut Game) {
    let mut v0: i16 =
        ((word!(game.samus_x_pos)) as i16).wrapping_sub((word!(game.samus_prev_x_pos)) as i16);
    if ((word!(game.samus_x_pos)) as i16).wrapping_sub((word!(game.samus_prev_x_pos)) as i16) < 0 {
        if v0 as i32 + 12 & 0x8000 != 0 {
            word!(game.samus_prev_x_pos = (word!(game.samus_x_pos)).wrapping_add(12));
        }
    } else if v0 as i32 - 13i32 & 0x8000 == 0 {
        word!(game.samus_prev_x_pos = (word!(game.samus_x_pos)).wrapping_sub(12));
    }
    let mut v1: i16 =
        ((word!(game.samus_y_pos)) as i16).wrapping_sub((word!(game.samus_prev_y_pos)) as i16);
    if ((word!(game.samus_y_pos)) as i16).wrapping_sub((word!(game.samus_prev_y_pos)) as i16) < 0 {
        if v1 as i32 + 12 & 0x8000 != 0 {
            word!(game.samus_prev_y_pos = (word!(game.samus_y_pos)).wrapping_add(12));
        }
    } else if v1 as i32 - 13i32 & 0x8000 == 0 {
        word!(game.samus_prev_y_pos = (word!(game.samus_y_pos)).wrapping_sub(12));
    }
    Samus_CancelSpeedBoost(game);
    word! { game, 0;
        samus_x_base_speed, samus_x_base_subspeed, samus_y_subspeed, samus_y_speed,
    }
    SetHiLo(ram_ptr!(game, u16; 0xb42), ram_ptr!(game, u16; 0xb44), 0);
}
pub fn nullsub_23(game: &mut Game) {}
static kSamus_HandleTransitionsA_5: [Option<Func_V>; 7] = callback_table! { unsafe Func_V;
    sub_91EFC3, Samus_HandleTransitionsA_5_1, Samus_HandleTransitionsA_5_2, nullsub_23,
    Samus_HandleTransitionsA_5_4, Samus_HandleTransitionsA_5_5, Samus_HandleTransitionsA_5_6,
};
pub unsafe fn Samus_HandleTransitionsA_5(game: &mut Game) {
    kSamus_HandleTransitionsA_5[word!(game.input_to_pose_calc) as u8 as usize]
        .expect("non-null function pointer")(game);
}
pub unsafe fn Samus_HandleTransitionsA_5_4(game: &mut Game) {
    word! { game, 0; samus_y_subspeed, samus_y_speed, UNUSED_word_7E0B1A }
    word!(game.samus_y_dir = 2);
}
pub unsafe fn Samus_HandleTransitionsA_5_2(game: &mut Game) {
    if word!(game.samus_y_dir) != 1 {
        word! { game, 0;
            used_for_ball_bounce_on_landing, samus_y_subspeed, samus_y_speed,
        }
        word!(game.samus_is_falling_flag = 1);
        word!(game.samus_y_dir = 2);
    }
    word!(game.UNUSED_word_7E0A18 = 0);
}
pub fn sub_91EFC3(game: &mut Game) {}
pub use self::nullsub_164 as sub_91EFC3_rv;
static kSamus_HandleTransitionsA_5_1: [Option<Func_U8>; 6] = callback_table! { unsafe Func_U8;
    Samus_HandleTransitionsA_5_1_0, Samus_MorphBallBounceNoSpringballTrans,
    Samus_HandleTransitionsA_5_1_2, Samus_MorphBallBounceSpringballTrans, sub_91EFC3_rv,
    Samus_HandleTransitionsA_5_1_5,
};
pub unsafe fn Samus_HandleTransitionsA_5_1(game: &mut Game) {
    HandleLandingSoundEffectsAndGfx(game);
    if *(ram_ptr!(game, u16; 0xdc6) as *mut u8).offset(1) == 4 {
        SamusFunc_F1D3(game);
    } else if kSamus_HandleTransitionsA_5_1
        [*(ram_ptr!(game, u16; 0xdc6) as *mut u8).offset(1isize) as usize]
        .expect("non-null function pointer")(game)
        & 1
        == 0
    {
        word! { game, 0;
            UNUSED_word_7E0A18, samus_x_accel_mode, samus_x_base_speed, samus_x_base_subspeed,
        }
        SamusFunc_F1D3(game);
    }
}
pub unsafe fn HandleLandingSoundEffectsAndGfx(game: &mut Game) {
    if (*ram_ptr!(game; 0xa23) as i32 == kMovementType_03_SpinJumping as i32
        || *ram_ptr!(game; 0xa23) as i32 == kMovementType_14_WallJumping as i32)
        && word!(game.cinematic_function) == 0
    {
        if word!(game.samus_prev_pose) as i32 == kPose_81_FaceR_Screwattack as i32
            || word!(game.samus_prev_pose) as i32 == kPose_82_FaceL_Screwattack as i32
        {
            QueueSfx1_Max6(0x34, game);
        } else {
            QueueSfx1_Max6(0x32, game);
        }
    }
    if word!(game.samus_y_speed) != 0 && word!(game.samus_y_speed) as i32 - 5i32 & 0x8000 == 0 {
        if word!(game.cinematic_function) == 0 {
            QueueSfx3_Max6(4, game);
        }
    } else {
        if word!(game.samus_y_subspeed) == 0 {
            return;
        }
        if word!(game.cinematic_function) == 0 {
            QueueSfx3_Max6(5, game);
        }
    }
    HandleLandingGraphics(game);
}
static off_91F0AE: [Option<Func_V>; 8] = callback_table! { unsafe Func_V;
    HandleLandingGraphics_Crateria, HandleLandingGraphics_Brinstar,
    HandleLandingGraphics_Norfair, HandleLandingGraphics_Norfair, HandleLandingGraphics_Maridia,
    HandleLandingGraphics_Tourian, HandleLandingGraphics_Ceres, HandleLandingGraphics_Ceres,
};
pub unsafe fn HandleLandingGraphics(game: &mut Game) {
    off_91F0AE[word!(game.area_index) as usize].expect("non-null function pointer")(game);
}
pub unsafe fn HandleLandingGraphics_Ceres(game: &mut Game) {
    word! { game, 0;
        atmospheric_gfx_frame_and_type[2], atmospheric_gfx_frame_and_type[3],
    }
}
static g_byte_91F0F3: [u8; 17] = [1, 0, 0, 0, 0, 2, 0, 4, 0, 4, 4, 4, 4, 0, 4, 0, 0];
pub unsafe fn HandleLandingGraphics_Crateria(game: &mut Game) {
    let use_default_landing: bool;
    if (word!(game.cinematic_function) == 0) {
        if word!(game.room_index) == 28 {
            HandleLandingGraphics_Norfair(game);
            return;
        }
        if (((word!(game.room_index)) as i16).wrapping_sub(16) < 0) {
            use_default_landing = if (g_byte_91F0F3[word!(game.room_index) as usize] & 1) == 0 {
                if (g_byte_91F0F3[word!(game.room_index) as usize] & 2) != 0 {
                    !(word!(game.samus_y_pos) as i32 - 944i32 & 0x8000 == 0)
                } else {
                    !((g_byte_91F0F3[word!(game.room_index) as usize] & 4) != 0)
                }
            } else {
                (word!(game.fx_type) != 10)
            };
            if use_default_landing {
            } else {
                HandleLandingGraphics_Maridia(game);
                return;
            }
        }
    }
    HandleLandingGraphics_Ceres(game);
}
pub unsafe fn HandleLandingGraphics_Maridia(game: &mut Game) {
    let bottom: u16 = Samus_GetBottom_R18(game);
    if liquid_at_height(game, bottom) != LiquidAtHeight::None {
        return;
    }
    word! { game, 256;
        atmospheric_gfx_frame_and_type[2], atmospheric_gfx_frame_and_type[3],
    }
    word! { game, 3; atmospheric_gfx_anim_timer[2], atmospheric_gfx_anim_timer[3] }
    word!(game.atmospheric_gfx_x_pos[2] = (word!(game.samus_x_pos)).wrapping_add(4));
    word!(game.atmospheric_gfx_x_pos[3] = (word!(game.samus_x_pos)).wrapping_sub(3));
    word!(game.atmospheric_gfx_y_pos[2] = (bottom).wrapping_sub(4));
    word!(game.atmospheric_gfx_y_pos[3] = (bottom).wrapping_sub(4));
}
pub unsafe fn HandleLandingGraphics_Norfair(game: &mut Game) {
    let bottom: u16 = Samus_GetBottom_R18(game);
    if liquid_at_height(game, bottom) != LiquidAtHeight::None {
        return;
    }
    word! { game, 1536;
        atmospheric_gfx_frame_and_type[2], atmospheric_gfx_frame_and_type[3],
    }
    word! { game, 3; atmospheric_gfx_anim_timer[2], atmospheric_gfx_anim_timer[3] }
    word!(game.atmospheric_gfx_x_pos[2] = (word!(game.samus_x_pos)).wrapping_add(8));
    word!(game.atmospheric_gfx_x_pos[3] = (word!(game.samus_x_pos)).wrapping_sub(8));
    word!(game.atmospheric_gfx_y_pos[2] = bottom);
    word!(game.atmospheric_gfx_y_pos[3] = bottom);
}
pub unsafe fn HandleLandingGraphics_Brinstar(game: &mut Game) {
    if word!(game.room_index) == 8 {
        HandleLandingGraphics_Norfair(game);
    } else {
        HandleLandingGraphics_Tourian(game);
    };
}
pub unsafe fn HandleLandingGraphics_Tourian(game: &mut Game) {
    if word!(game.room_index) as i32 - 5i32 & 0x8000 == 0
        && (word!(game.room_index) as i32 - 9i32 & 0x8000 != 0 || word!(game.room_index) == 11)
    {
        HandleLandingGraphics_Norfair(game);
    } else {
        word! { game, 0;
            atmospheric_gfx_frame_and_type[2], atmospheric_gfx_frame_and_type[3],
        }
    };
}
pub unsafe fn SamusFunc_F1D3(game: &mut Game) {
    word! { game, 0;
        samus_is_falling_flag, UNUSED_word_7E0B1A, UNUSED_word_7E0B2A, samus_y_subspeed,
        samus_y_speed, samus_y_dir, UNUSED_word_7E0B38, used_for_ball_bounce_on_landing,
    }
}
pub unsafe fn Samus_HandleTransitionsA_5_1_0(game: &mut Game) -> u8 {
    if word!(game.samus_input_handler) as i32 != 0x90e91d & 0xffff {
        word!(game.samus_input_handler = ((0x90e926i32) as u16));
    }
    0
}
static g_word_909EB5: u16 = 1;
static g_word_909EB7: u16 = 0;
pub unsafe fn Samus_MorphBallBounceNoSpringballTrans(game: &mut Game) -> u8 {
    let mut v0: i16 = (2i16).wrapping_mul((word!(game.used_for_ball_bounce_on_landing)) as i16);
    if 2 * word!(game.used_for_ball_bounce_on_landing) as i32 != 0 {
        if v0 == 2 {
            word!(game.used_for_ball_bounce_on_landing, wrapping_add(1));
            word!(game.samus_y_dir = 1);
            word!(game.samus_y_subspeed = g_word_909EB7);
            word!(game.samus_y_speed = (g_word_909EB5).wrapping_sub(1));
            return 1;
        }
        if v0 != 4 {
            Unreachable(game);
        }
    } else if word!(game.samus_y_speed) as i32 - 3i32 & 0x8000 == 0 {
        word!(game.used_for_ball_bounce_on_landing, wrapping_add(1));
        word!(game.samus_y_dir = 1);
        word!(game.samus_y_subspeed = g_word_909EB7);
        word!(game.samus_y_speed = g_word_909EB5);
        return 1;
    }
    word! { game, 0;
        used_for_ball_bounce_on_landing, samus_y_dir, samus_y_subspeed, samus_y_speed,
    }
    0
}
pub unsafe fn Samus_HandleTransitionsA_5_1_2(game: &mut Game) -> u8 {
    word!(game.used_for_ball_bounce_on_landing = 0);
    word!(game.enable_horiz_slope_coll = 3);
    0
}
pub unsafe fn Samus_MorphBallBounceSpringballTrans(game: &mut Game) -> u8 {
    if (word!(game.button_config_jump_a) & word!(game.joypad1_lastkeys)) != 0 {
        word!(game.used_for_ball_bounce_on_landing = 0);
        Samus_InitJump(game);
        return 1;
    }
    let mut v1: i16 =
        (2i16).wrapping_mul((word!(game.used_for_ball_bounce_on_landing) as u8) as i16);
    if v1 != 0 {
        if v1 == 2 {
            word!(game.used_for_ball_bounce_on_landing = 1538);
            word!(game.samus_y_dir = 1);
            word!(game.samus_y_subspeed = g_word_909EB7);
            word!(game.samus_y_speed = (g_word_909EB5).wrapping_sub(1));
            return 1;
        }
        if v1 != 4 {
            Unreachable(game);
        }
    } else if word!(game.samus_y_speed) as i32 - 3i32 & 0x8000 == 0 {
        word!(game.used_for_ball_bounce_on_landing = 1537);
        word!(game.samus_y_dir = 1);
        word!(game.samus_y_subspeed = g_word_909EB7);
        word!(game.samus_y_speed = g_word_909EB5);
        return 1;
    }
    word! { game, 0;
        used_for_ball_bounce_on_landing, samus_y_dir, samus_y_subspeed, samus_y_speed,
    }
    0
}
pub unsafe fn Samus_HandleTransitionsA_5_1_5(game: &mut Game) -> u8 {
    word!(game.used_for_ball_bounce_on_landing = 0);
    0
}
pub unsafe fn Samus_HandleTransitionsA_5_5(game: &mut Game) {
    word! { game, 0;
        samus_x_accel_mode, samus_collides_with_solid_enemy, samus_is_falling_flag,
        UNUSED_word_7E0B1A, samus_x_base_speed, samus_x_base_subspeed, UNUSED_word_7E0A18,
    }
    QueueSfx3_Max6(5, game);
}
pub unsafe fn Samus_HandleTransitionsA_5_6(game: &mut Game) {
    if word!(game.samus_collides_with_solid_enemy) != 0 && *ram_ptr!(game; 0xa23) == 9 {
        word!(game.enable_horiz_slope_coll = (*ram_ptr!(game; 0xa1e) != 4) as u16);
        word!(game.UNUSED_word_7E0A18 = 0);
    }
}
pub unsafe fn Samus_HandleTransitionsC_1(game: &mut Game) {
    word!(game.knockback_dir = 0);
    word!(game.samus_movement_handler = ((0x90a337i32) as u16));
    word! { game, 0; used_for_ball_bounce_on_landing, samus_y_subspeed, samus_y_speed }
    word!(game.samus_is_falling_flag = 1);
    word!(game.samus_y_dir = 2);
    Samus_AlignBottomWithPrevPose(game);
    if word!(game.samus_input_handler) as i32 != 0x90e91d & 0xffff {
        word!(game.samus_input_handler = ((0x90e913i32) as u16));
    }
}
pub unsafe fn Samus_HandleTransitionsC_2(game: &mut Game) {
    Samus_AlignBottomWithPrevPose(game);
    word!(game.samus_movement_handler = ((0x90a337i32) as u16));
    if word!(game.game_state) as i32 == kGameState_42_PlayingDemo as i32 {
        word!(game.samus_input_handler = ((0x90e91di32) as u16));
    } else {
        word!(game.samus_input_handler = ((0x90e913i32) as u16));
    };
}
pub unsafe fn Samus_HandleTransitionsC_3(game: &mut Game) {
    word!(game.used_for_ball_bounce_on_landing = 0);
    word!(
        game.samus_anim_frame_timer,
        wrapping_add(word!(game.samus_anim_frame_buffer))
    );
}
pub unsafe fn Samus_HandleTransitionsC_4(game: &mut Game) {
    word!(game.samus_x_pos = (word!(game.layer1_x_pos)).wrapping_add(128));
    word!(game.samus_prev_x_pos = (word!(game.layer1_x_pos)).wrapping_add(128));
    word!(game.samus_y_pos = (word!(game.layer1_y_pos)).wrapping_add(128));
    word!(game.samus_prev_y_pos = (word!(game.layer1_y_pos)).wrapping_add(128));
}
pub unsafe fn Samus_HandleTransitionsC_5(game: &mut Game) {
    word!(game.samus_y_pos, wrapping_add(5));
    word!(game.samus_prev_y_pos = word!(game.samus_y_pos));
}
pub unsafe fn Samus_HandleTransitionsC_6(game: &mut Game) {
    HandleJumpTransition(game);
}
pub unsafe fn Samus_HandleTransitionsC_7(game: &mut Game) {
    let mut v0: i16 =
        ((word!(game.samus_x_pos)) as i16).wrapping_sub((word!(game.samus_prev_x_pos)) as i16);
    if ((word!(game.samus_x_pos)) as i16).wrapping_sub((word!(game.samus_prev_x_pos)) as i16) < 0 {
        if v0 as i32 + 12 & 0x8000 != 0 {
            word!(game.samus_prev_x_pos = (word!(game.samus_x_pos)).wrapping_add(12));
        }
    } else if v0 as i32 - 13i32 & 0x8000 == 0 {
        word!(game.samus_prev_x_pos = (word!(game.samus_x_pos)).wrapping_sub(12));
    }
    let mut v1: i16 =
        ((word!(game.samus_y_pos)) as i16).wrapping_sub((word!(game.samus_prev_y_pos)) as i16);
    if ((word!(game.samus_y_pos)) as i16).wrapping_sub((word!(game.samus_prev_y_pos)) as i16) < 0 {
        if v1 as i32 + 12 & 0x8000 != 0 {
            word!(game.samus_prev_y_pos = (word!(game.samus_y_pos)).wrapping_add(12));
        }
    } else if v1 as i32 - 13i32 & 0x8000 == 0 {
        word!(game.samus_prev_y_pos = (word!(game.samus_y_pos)).wrapping_sub(12));
    }
}
pub unsafe fn Samus_HandleTransitionsC_8(game: &mut Game) {
    Samus_HandleTransitionsC_1(game);
    Samus_HandleTransitionsC_3(game);
}
pub unsafe fn SamusFunc_F404(game: &mut Game) -> u8 {
    let mut v1: SamusPose = kPose_00_FaceF_Powersuit;
    v1 = word!(game.samus_pose) as SamusPose;
    if word!(game.samus_pose) != word!(game.samus_prev_pose) {
        HandleCollDueToChangedPose(game);
        SamusFunc_F433(game);
        HandleJumpTransition(game);
        Samus_SetAnimationFrameIfPoseChanged(game);
        word!(game.samus_anim_frame_skip = 0);
    }
    (v1 != ((word!(game.samus_pose)) as u32)) as u8
}
pub unsafe fn SamusFunc_F433(game: &mut Game) {
    *(ram_ptr!(game, u16; 0xa1e)) =
        *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
            as *mut SamusPoseParams)
            .offset(0))
        .pose_x_dir)
            .offset((8 * word!(game.samus_pose) as i32) as isize) as *mut u16);
    SamusFunc_F468(game);
    if (*ram_ptr!(game; 0xa23) as i32 == kMovementType_03_SpinJumping as i32
        || *ram_ptr!(game; 0xa23) as i32 == kMovementType_14_WallJumping as i32)
        && (word!(game.equipped_items) & 8) != 0
    {
        Samus_LoadSuitPalette(game);
    }
}
static off_91F4A2: [Option<Func_U8>; 28] = callback_table! { unsafe Func_U8;
    SamusFunc_F468_Standing, SamusFunc_F468_Running, SamusFunc_F468_NormalJump,
    SamusFunc_F468_SpinJump, SamusFunc_F468_MorphBall, SamusFunc_F468_Crouching,
    SamusFunc_F468_Falling, SamusFunc_F468_Unused, SamusFunc_F468_MorphBall,
    SamusFunc_F468_Unused, SamusFunc_F468_Unused, SamusFunc_F468_Unused, SamusFunc_F468_Unused,
    SamusFunc_F468_Unused, SamusFunc_F468_TurningAroundOnGround, SamusFunc_F468_CrouchTransEtc,
    SamusFunc_F468_Moonwalking, SamusFunc_F468_Springball, SamusFunc_F468_Springball,
    SamusFunc_F468_Springball, SamusFunc_F468_WallJumping, SamusFunc_F468_Unused,
    SamusFunc_F468_Unused, SamusFunc_F468_TurnAroundJumping, SamusFunc_F468_TurnAroundFalling,
    SamusFunc_F468_DamageBoost, SamusFunc_F468_Unused, SamusFunc_F468_Shinespark,
};
pub unsafe fn SamusFunc_F468(game: &mut Game) {
    if off_91F4A2[*ram_ptr!(game; 0xa1f) as usize].expect("non-null function pointer")(game) & 1
        != 0
    {
        *(ram_ptr!(game, u16; 0xa1e)) =
            *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
                as *mut SamusPoseParams)
                .offset(0))
            .pose_x_dir)
                .offset((8 * word!(game.samus_pose) as i32) as isize) as *mut u16);
        if (*(ram_ptr!(game, u16; 0xa1e)) & 0xff00) == 3584 {
            off_91F4A2[14i32 as usize].expect("non-null function pointer")(game);
            *(ram_ptr!(game, u16; 0xa1e)) =
                *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
                    as *mut SamusPoseParams)
                    .offset(0))
                .pose_x_dir)
                    .offset((8 * word!(game.samus_pose) as i32) as isize)
                    as *mut u16);
        }
    }
}
pub use self::nullsub_164 as SamusFunc_F468_Unused;
pub unsafe fn SamusFunc_F468_Standing(game: &mut Game) -> u8 {
    if ((*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629)) as *mut SamusPoseParams)
        .offset(word!(game.samus_pose) as isize))
    .direction_shots_fired
        == 0
        || (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629)) as *mut SamusPoseParams)
            .offset(word!(game.samus_pose) as isize))
        .direction_shots_fired
            == 9)
        && ((*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629)) as *mut SamusPoseParams)
            .offset(word!(game.samus_prev_pose) as isize))
        .direction_shots_fired
            == 0
            || (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629)) as *mut SamusPoseParams)
                .offset(word!(game.samus_prev_pose) as isize))
            .direction_shots_fired
                == 9)
    {
        word!(game.samus_anim_frame_skip = 1);
    }
    0
}
pub unsafe fn SamusFunc_F468_Running(game: &mut Game) -> u8 {
    if *ram_ptr!(game; 0xa23) == 1 {
        word!(game.samus_anim_frame_skip = ((0x918000i32) as u16));
    }
    if word!(game.UNUSED_word_7E0DF8) == 0 {
        return 0;
    }
    if word!(game.samus_pose) as i32
        == kPose_44_FaceL_Turn_Crouch as i32 | kPose_01_FaceR_Normal as i32
    {
        word!(game.samus_pose = kPose_25_FaceR_Turn_Stand as u16);
        return 1;
    }
    if word!(game.samus_pose) as i32
        == kPose_44_FaceL_Turn_Crouch as i32 | kPose_02_FaceL_Normal as i32
    {
        word!(game.samus_pose = kPose_26_FaceL_Turn_Stand as u16);
        return 1;
    }
    0
}
pub unsafe fn SamusFunc_F468_NormalJump(game: &mut Game) -> u8 {
    let mut current_block: u64;
    if word!(game.samus_pose) as i32 != kPose_4E_FaceL_Jump_NoAim_NoMove_NoGun as i32 {
        if word!(game.samus_pose) as i32 == kPose_4D_FaceR_Jump_NoAim_NoMove_NoGun as i32
            || word!(game.samus_pose) as i32 == kPose_15_FaceR_Jump_AimU as i32
        {
            current_block = 10566234459686315488;
        } else if word!(game.samus_pose) as i32 != kPose_16_FaceL_Jump_AimU as i32
            && word!(game.samus_pose) as i32 != kPose_6A_FaceL_Jump_AimUL as i32
        {
            if word!(game.samus_pose) as i32 != kPose_69_FaceR_Jump_AimUR as i32 {
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
                if word!(game.samus_shine_timer) != 0 {
                    word!(game.samus_pose = kPose_C7_FaceR_ShinesparkWindup_Vert as u16);
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
        if word!(game.samus_shine_timer) != 0 {
            word!(game.samus_pose = kPose_C8_FaceL_ShinesparkWindup_Vert as u16);
            current_block = 13140864810214251092;
        } else {
            current_block = 10164294398894088689;
        }
    }
    if current_block == 13140864810214251092 {
        Projectile_Func7_Shinespark(game);
        if *ram_ptr!(game; 0xa23) as i32 == kMovementType_02_NormalJumping as i32 {
            word!(game.samus_prev_y_pos = word!(game.samus_y_pos, wrapping_sub(1), reread));
        }
        1
    } else {
        if word!(game.samus_x_extra_run_speed) != 0 || word!(game.samus_x_extra_run_subspeed) != 0 {
            word!(game.samus_x_accel_mode = 2);
        } else {
            word!(game.samus_x_accel_mode = 0);
        }
        if (word!(game.samus_pose) as i32 == kPose_15_FaceR_Jump_AimU as i32
            || word!(game.samus_pose) as i32 == kPose_16_FaceL_Jump_AimU as i32)
            && (word!(game.samus_prev_pose) as i32 == kPose_55_FaceR_Jumptrans_AimU as i32
                || word!(game.samus_prev_pose) as i32 == kPose_56_FaceL_Jumptrans_AimU as i32)
        {
            word!(game.samus_anim_frame_skip = 1);
        }
        if (word!(game.button_config_shoot_x) & word!(game.joypad1_newkeys)) != 0 {
            word!(
                game.new_projectile_direction_changed_pose =
                    (((*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
                        as *mut SamusPoseParams)
                        .offset(word!(game.samus_pose) as isize))
                    .direction_shots_fired) as u16)
                        | (0x8000u16)
            );
        }
        0
    }
}
pub unsafe fn SamusFunc_F468_Crouching(game: &mut Game) -> u8 {
    if (word!(game.samus_pose) as i32 == kPose_85_FaceR_Crouch_AimU as i32
        || word!(game.samus_pose) as i32 == kPose_86_FaceL_Crouch_AimU as i32)
        && (word!(game.samus_prev_pose) as i32 == kPose_F1_FaceR_CrouchTrans_AimU as i32
            || word!(game.samus_prev_pose) as i32 == kPose_F2_FaceL_CrouchTrans_AimU as i32)
    {
        word!(game.samus_anim_frame_skip = 1);
    }
    0
}
pub unsafe fn SamusFunc_F468_Falling(game: &mut Game) -> u8 {
    if word!(game.samus_x_extra_run_speed) != 0 || word!(game.samus_x_extra_run_subspeed) != 0 {
        word!(game.samus_x_accel_mode = 2);
    } else {
        word!(game.samus_x_accel_mode = 0);
    }
    0
}
pub unsafe fn SamusFunc_F468_SpinJump(game: &mut Game) -> u8 {
    let mut current_block: u64;
    if *ram_ptr!(game; 0xa23) as i32 == kMovementType_03_SpinJumping as i32
        || *ram_ptr!(game; 0xa23) as i32 == kMovementType_14_WallJumping as i32
    {
        word!(game.samus_anim_frame_skip = 1);
        if (*ram_ptr!(game; 0xa22) & 0xf) == 8 {
            if *(ram_ptr!(game, u16; 0xa1e)) != 772 {
                current_block = 12119026790781926931;
            } else {
                current_block = 820271813250567934;
            }
        } else if (*ram_ptr!(game; 0xa22) & 0xf) != 4 || *(ram_ptr!(game, u16; 0xa1e)) != 776 {
            current_block = 12119026790781926931;
        } else {
            current_block = 820271813250567934;
        }
        if current_block == 12119026790781926931 {
        } else {
            AddToHiLo(
                ram_ptr!(game, u16; 0xb46),
                ram_ptr!(game, u16; 0xb48),
                (word!(game.samus_x_extra_run_speed) as u32) << 16
                    | word!(game.samus_x_extra_run_subspeed) as u32,
            );
            Samus_CancelSpeedBoost(game);
            SetHiLo(ram_ptr!(game, u16; 0xb42), ram_ptr!(game, u16; 0xb44), 0);
            word!(game.samus_x_accel_mode = 1);
        }
    }
    if *ram_ptr!(game; 0xa1e) == 4 {
        if (word!(game.equipped_items) & 0x20) == 0 {
            let r20: u16 = Samus_GetTop_R20(game);
            if liquid_at_height(game, r20) != LiquidAtHeight::None {
                return 0;
            }
        }
        if (word!(game.equipped_items) & 8) != 0 {
            word!(game.samus_pose = kPose_82_FaceL_Screwattack as u16);
            current_block = 131178307184372633;
        } else {
            if (word!(game.equipped_items) & 0x200) != 0 {
                QueueSfx1_Max6(0x3e, game);
                word!(game.samus_pose = kPose_1C_FaceL_SpaceJump as u16);
                return 0;
            }
            if word!(game.samus_anim_frame_skip) == 0 && word!(game.cinematic_function) == 0 {
                current_block = 807399161326926732;
            } else {
                return 0;
            }
        }
    } else {
        if (word!(game.equipped_items) & 0x20) == 0 {
            let r20_0: u16 = Samus_GetTop_R20(game);
            if liquid_at_height(game, r20_0) != LiquidAtHeight::None {
                return 0;
            }
        }
        if (word!(game.equipped_items) & 8) != 0 {
            word!(game.samus_pose = kPose_81_FaceR_Screwattack as u16);
            current_block = 131178307184372633;
        } else {
            if (word!(game.equipped_items) & 0x200) == 0 {
                if word!(game.samus_anim_frame_skip) != 0 || word!(game.cinematic_function) != 0 {
                    return 0;
                }
            } else {
                QueueSfx1_Max6(0x3e, game);
                word!(game.samus_pose = kPose_1B_FaceR_SpaceJump as u16);
                return 0;
            }
            current_block = 807399161326926732;
        }
    }
    if current_block == 807399161326926732 {
        QueueSfx1_Max6(0x31, game);
        0
    } else {
        if word!(game.samus_anim_frame_skip) == 0 {
            QueueSfx1_Max6(0x33, game);
        }
        0
    }
}
static off_91F790: [Option<Func_U8>; 12] = callback_table! { unsafe Func_U8;
    Samus_CrouchTrans, Samus_CrouchTrans, Samus_MorphTrans, Samus_MorphTrans,
    MaybeUnused_sub_91F7F4, MaybeUnused_sub_91F840, Samus_StandOrUnmorphTrans,
    Samus_StandOrUnmorphTrans, Samus_StandOrUnmorphTrans, Samus_StandOrUnmorphTrans,
    Samus_StandOrUnmorphTrans, Samus_StandOrUnmorphTrans,
};
static off_91F7A8: [Option<Func_U8>; 4] = callback_table! { unsafe Func_U8;
    Samus_MorphTrans, Samus_MorphTrans, Samus_StandOrUnmorphTrans, Samus_StandOrUnmorphTrans,
};
pub unsafe fn SamusFunc_F468_CrouchTransEtc(game: &mut Game) -> u8 {
    let mut v0: u16 = 0;
    's_43: {
        if word!(game.samus_pose) as i32 - kPose_F1_FaceR_CrouchTrans_AimU as i32 & 0x8000 != 0 {
            if word!(game.samus_pose) as i32 - kPose_DB as i32 & 0x8000 != 0 {
                v0 = (2 * (word!(game.samus_pose) as i32 - 53i32)) as u16;
            } else {
                return off_91F7A8[(word!(game.samus_pose) as i32 - 219i32) as usize]
                    .expect("non-null function pointer")(game);
            }
        } else if word!(game.samus_pose) as i32 - kPose_F7_FaceR_StandTrans_AimU as i32 & 0x8000
            != 0
        {
            v0 = 0;
        } else {
            word!(game.samus_momentum_routine_index = 7);
            break 's_43;
        }
        word!(game.samus_momentum_routine_index = 7);
        return off_91F790[(v0 >> 1) as usize].expect("non-null function pointer")(game);
    }
    0
}
pub unsafe fn Samus_CrouchTrans(game: &mut Game) -> u8 {
    if (word!(game.speed_boost_counter) as i32 & 0xff00) - 1024i32 & 0x8000 == 0 {
        word!(game.samus_shine_timer = 180);
        word!(game.timer_for_shine_timer = 1);
        word!(game.special_samus_palette_frame = 0);
    }
    0
}
pub use self::nullsub_164 as Samus_StandOrUnmorphTrans;
pub unsafe fn Samus_MorphTrans(game: &mut Game) -> u8 {
    if (word!(game.equipped_items) & 4) != 0 {
        if *ram_ptr!(game; 0xa23) as i32 == kMovementType_03_SpinJumping as i32 {
            word!(game.samus_x_accel_mode = 2);
        }
        word!(game.bomb_spread_charge_timeout_counter = 0);
        0
    } else {
        word!(game.samus_pose = word!(game.samus_prev_pose));
        1
    }
}
pub unsafe fn MaybeUnused_sub_91F7F4(game: &mut Game) -> u8 {
    if *ram_ptr!(game; 0xa23) as i32 == kMovementType_08_MorphBallFalling as i32
        || *ram_ptr!(game; 0xa23) as i32 == kMovementType_13_SpringBallFalling as i32
    {
        if (word!(game.equipped_items) & 2) != 0 {
            word!(game.samus_pose = kPose_7D_FaceR_Springball_Fall as u16);
        } else {
            word!(game.samus_pose = kPose_31_FaceR_Morphball_Air as u16);
        }
    } else if (word!(game.equipped_items) & 2) != 0 {
        word!(game.samus_pose = kPose_79_FaceR_Springball_Ground as u16);
    } else {
        word!(game.samus_pose = kPose_1D_FaceR_Morphball_Ground as u16);
    }
    1
}
pub unsafe fn MaybeUnused_sub_91F840(game: &mut Game) -> u8 {
    if *ram_ptr!(game; 0xa23) as i32 == kMovementType_08_MorphBallFalling as i32
        || *ram_ptr!(game; 0xa23) as i32 == kMovementType_13_SpringBallFalling as i32
    {
        if (word!(game.equipped_items) & 2) != 0 {
            word!(game.samus_pose = kPose_7E_FaceL_Springball_Fall as u16);
        } else {
            word!(game.samus_pose = kPose_32_FaceL_Morphball_Air as u16);
        }
    } else if (word!(game.equipped_items) & 2) != 0 {
        word!(game.samus_pose = kPose_7A_FaceL_Springball_Ground as u16);
    } else {
        word!(game.samus_pose = kPose_41_FaceL_Morphball_Ground as u16);
    }
    1
}
pub unsafe fn SamusFunc_F468_Moonwalking(game: &mut Game) -> u8 {
    if word!(game.moonwalk_flag) != 0 {
        return 0;
    }
    if *ram_ptr!(game; 0xa1e) == 4 {
        word!(game.samus_pose = kPose_25_FaceR_Turn_Stand as u16);
    } else {
        word!(game.samus_pose = kPose_26_FaceL_Turn_Stand as u16);
    }
    1
}
pub unsafe fn SamusFunc_F468_DamageBoost(game: &mut Game) -> u8 {
    SamusFunc_F468_DamageBoost_(game)
}
pub unsafe fn SamusFunc_F468_DamageBoost_(game: &mut Game) -> u8 {
    word!(game.samus_movement_handler = ((0x90a337i32) as u16));
    0
}
pub unsafe fn SamusFunc_F468_TurningAroundOnGround(game: &mut Game) -> u8 {
    if word!(game.samus_prev_pose) != 0
        && word!(game.samus_prev_pose) as i32 != kPose_9B_FaceF_VariaGravitySuit as i32
    {
        let v0: u16 = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
            as *mut SamusPoseParams)
            .offset(word!(game.samus_prev_pose) as isize))
        .direction_shots_fired as u16;
        if *ram_ptr!(game; 0xa23) as i32 == kMovementType_10_Moonwalking as i32 {
            word!(game.new_projectile_direction_changed_pose = (v0) | (0x100u16));
            if (word!(game.button_config_jump_a) & word!(game.joypad1_lastkeys)) != 0 {
                word!(
                    game.samus_pose = *(game.g_rom.offset(crate::rtl::lorom_offset(0x91f9ea))
                        as *mut u8)
                        .offset(v0 as isize) as u16
                );
            } else {
                word!(
                    game.samus_pose = *(game.g_rom.offset(crate::rtl::lorom_offset(0x91f9c2))
                        as *mut u8)
                        .offset(v0 as isize) as u16
                );
            }
        } else if *ram_ptr!(game; 0xa23) == 5 {
            word!(
                game.samus_pose = *(game.g_rom.offset(crate::rtl::lorom_offset(0x91f9cc))
                    as *mut u8)
                    .offset(v0 as isize) as u16
            );
        } else {
            word!(
                game.samus_pose = *(game.g_rom.offset(crate::rtl::lorom_offset(0x91f9c2))
                    as *mut u8)
                    .offset(v0 as isize) as u16
            );
        }
    }
    AddToHiLo(
        ram_ptr!(game, u16; 0xb46),
        ram_ptr!(game, u16; 0xb48),
        (word!(game.samus_x_extra_run_speed) as u32) << 16
            | word!(game.samus_x_extra_run_subspeed) as u32,
    );
    SetHiLo(ram_ptr!(game, u16; 0xb42), ram_ptr!(game, u16; 0xb44), 0);
    word!(game.samus_x_accel_mode = 1);
    1
}
pub unsafe fn SamusFunc_F468_TurnAroundJumping(game: &mut Game) -> u8 {
    word!(
        game.samus_pose = *(game.g_rom.offset(crate::rtl::lorom_offset(0x91f9d6)) as *mut u8)
            .offset(
                (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629)) as *mut SamusPoseParams)
                    .offset(word!(game.samus_prev_pose) as isize))
                .direction_shots_fired as isize,
            ) as u16
    );
    AddToHiLo(
        ram_ptr!(game, u16; 0xb46),
        ram_ptr!(game, u16; 0xb48),
        (word!(game.samus_x_extra_run_speed) as u32) << 16
            | word!(game.samus_x_extra_run_subspeed) as u32,
    );
    SetHiLo(ram_ptr!(game, u16; 0xb42), ram_ptr!(game, u16; 0xb44), 0);
    word!(game.samus_x_accel_mode = 1);
    1
}
pub unsafe fn SamusFunc_F468_TurnAroundFalling(game: &mut Game) -> u8 {
    word!(
        game.samus_pose = *(game.g_rom.offset(crate::rtl::lorom_offset(0x91f9e0)) as *mut u8)
            .offset(
                (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629)) as *mut SamusPoseParams)
                    .offset(word!(game.samus_prev_pose) as isize))
                .direction_shots_fired as isize,
            ) as u16
    );
    AddToHiLo(
        ram_ptr!(game, u16; 0xb46),
        ram_ptr!(game, u16; 0xb48),
        (word!(game.samus_x_extra_run_speed) as u32) << 16
            | word!(game.samus_x_extra_run_subspeed) as u32,
    );
    SetHiLo(ram_ptr!(game, u16; 0xb42), ram_ptr!(game, u16; 0xb44), 0);
    word!(game.samus_x_accel_mode = 1);
    1
}
pub unsafe fn SamusFunc_F468_MorphBall(game: &mut Game) -> u8 {
    if *ram_ptr!(game; 0xa23) as i32 == kMovementType_04_MorphBallOnGround as i32
        || *ram_ptr!(game; 0xa23) as i32 == kMovementType_08_MorphBallFalling as i32
    {
        word!(game.samus_anim_frame_skip = ((0x918000i32) as u16));
    }
    SamusFunc_FA0A(game);
    0
}
pub unsafe fn SamusFunc_FA0A(game: &mut Game) {
    let apply_extra_run_speed: bool = if *ram_ptr!(game; 0xa22) == 8 {
        if *ram_ptr!(game; 0xa1e) != 4 {
            return;
        }
        true
    } else {
        (*ram_ptr!(game; 0xa1e) == 8)
    };
    if apply_extra_run_speed {
        AddToHiLo(
            ram_ptr!(game, u16; 0xb46),
            ram_ptr!(game, u16; 0xb48),
            (word!(game.samus_x_extra_run_speed) as u32) << 16
                | word!(game.samus_x_extra_run_subspeed) as u32,
        );
        Samus_CancelSpeedBoost(game);
        SetHiLo(ram_ptr!(game, u16; 0xb42), ram_ptr!(game, u16; 0xb44), 0);
        word!(game.samus_x_accel_mode = 1);
    };
}
pub unsafe fn SamusFunc_F468_Springball(game: &mut Game) -> u8 {
    if *ram_ptr!(game; 0xa23) as i32 == kMovementType_11_SpringBallOnGround as i32
        || *ram_ptr!(game; 0xa23) as i32 == kMovementType_12_SpringBallInAir as i32
        || *ram_ptr!(game; 0xa23) as i32 == kMovementType_13_SpringBallFalling as i32
    {
        word!(game.samus_anim_frame_skip = ((0x918000i32) as u16));
    }
    SamusFunc_FA0A(game);
    0
}
pub unsafe fn SamusFunc_F468_WallJumping(game: &mut Game) -> u8 {
    let bottom: u16 = Samus_GetBottom_R18(game);
    if liquid_at_height(game, bottom) != LiquidAtHeight::None {
        return 0;
    }
    word!(game.atmospheric_gfx_frame_and_type[3] = 1536);
    word!(game.atmospheric_gfx_anim_timer[3] = 3);
    word!(game.atmospheric_gfx_y_pos[3] = bottom);
    if *ram_ptr!(game; 0xa1e) != 8 {
        word!(game.atmospheric_gfx_x_pos[3] = (word!(game.samus_x_pos)).wrapping_add(6));
        return 0;
    }
    word!(game.atmospheric_gfx_x_pos[3] = (word!(game.samus_x_pos)).wrapping_sub(6));
    0
}
static kSamusFunc_F468_Shinespark: [u16; 6] = [
    fnSamus_MoveHandler_Shinespark_Horiz as u16,
    fnSamus_MoveHandler_Shinespark_Horiz as u16,
    fnSamus_MoveHandlerVerticalShinespark as u16,
    fnSamus_MoveHandlerVerticalShinespark as u16,
    fnSamus_MoveHandler_Shinespark_Diag as u16,
    fnSamus_MoveHandler_Shinespark_Diag as u16,
];
pub unsafe fn SamusFunc_F468_Shinespark(game: &mut Game) -> u8 {
    if word!(game.samus_pose) as i32 - kPose_CF_FaceR_Ranintowall_AimUR as i32 & 0x8000 != 0 {
        word!(
            game.samus_movement_handler =
                kSamusFunc_F468_Shinespark[(word!(game.samus_pose) as i32 - 201i32) as usize]
        );
        word!(game.samus_input_handler = ((0x90e90ei32) as u16));
        word! { game, 0;
            speed_echoes_index, speed_echo_xspeed[0], speed_echo_xspeed[1], speed_echo_xpos[0],
            speed_echo_xpos[1],
        }
        QueueSfx3_Max9(0xf, game);
    }
    0
}
static kSamusPhys_AnimDelayInWater: u16 = 3;
static kSamusPhys_AnimDelayInAcid: u16 = 2;
pub unsafe fn Samus_SetAnimationFrameIfPoseChanged(game: &mut Game) {
    let mut t: u16 = 0;
    if (word!(game.equipped_items) & 0x20) != 0 {
        t = word!(game.samus_x_speed_divisor);
    } else {
        let r: u16 = (word!(game.samus_y_pos) as i32
            + (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629)) as *mut SamusPoseParams)
                .offset(word!(game.samus_pose) as isize))
            .y_radius as i32
            - 1i32) as u16;
        t = match liquid_at_height(game, r) {
            LiquidAtHeight::Water => kSamusPhys_AnimDelayInWater,
            LiquidAtHeight::LavaAcid => kSamusPhys_AnimDelayInAcid,
            LiquidAtHeight::None => {
                word!(game.samus_x_speed_divisor)
            }
        };
    }
    if (word!(game.samus_anim_frame_skip) & 0x8000) == 0
        && word!(game.samus_pose) != word!(game.samus_prev_pose)
    {
        word!(game.samus_anim_frame = word!(game.samus_anim_frame_skip));
        word!(
            game.samus_anim_frame_timer = (t).wrapping_add(
                (*RomPtr_91(
                    (crate::rtl::rom_table_word!(game, 0x91b010, word!(game.samus_pose) as isize))
                        .wrapping_add(word!(game.samus_anim_frame_skip)),
                    game,
                )) as u16,
            )
        );
    }
}
pub unsafe fn SamusFunc_EC80(game: &mut Game) {
    if *ram_ptr!(game; 0xa23) as i32 != kMovementType_06_Falling as i32
        && *ram_ptr!(game; 0xa1f) as i32 == kMovementType_06_Falling as i32
    {
        word! { game, 0; samus_y_subspeed, samus_y_speed }
        word!(game.samus_y_dir = 2);
    }
}
pub fn nullsub_24(game: &mut Game) {}
pub fn nullsub_25(game: &mut Game) {}
static kHandleJumpTrans: [Option<Func_V>; 28] = callback_table! { unsafe Func_V;
    nullsub_24, nullsub_24, HandleJumpTransition_NormalJump, HandleJumpTransition_SpinJump,
    nullsub_24, nullsub_24, nullsub_24, nullsub_24, nullsub_24, nullsub_24, nullsub_24,
    nullsub_24, nullsub_24, UNUSED_sub_91FC42, nullsub_24, nullsub_24, nullsub_24, nullsub_24,
    HandleJumpTransition_SpringBallInAir, nullsub_24, HandleJumpTransition_WallJump, nullsub_24,
    nullsub_24, nullsub_24, nullsub_24, nullsub_25, nullsub_24, nullsub_24,
};
pub unsafe fn HandleJumpTransition(game: &mut Game) {
    kHandleJumpTrans[*ram_ptr!(game; 0xa1f) as usize].expect("non-null function pointer")(game);
}
pub unsafe fn HandleJumpTransition_WallJump(game: &mut Game) {
    if *ram_ptr!(game; 0xa23) as i32 != kMovementType_14_WallJumping as i32 {
        Samus_InitWallJump(game);
    }
}
pub unsafe fn HandleJumpTransition_SpringBallInAir(game: &mut Game) {
    if word!(game.samus_pose) as i32 == kPose_7F_FaceR_Springball_Air as i32 {
        if *ram_ptr!(game; 0xa23) as i32 != kMovementType_11_SpringBallOnGround as i32 {
            return;
        }
    } else if !(word!(game.samus_pose) as i32 == kPose_80_FaceL_Springball_Air as i32
        && *ram_ptr!(game; 0xa23) as i32 == kMovementType_11_SpringBallOnGround as i32)
    {
        return;
    }
    Samus_InitJump(game);
}
pub unsafe fn UNUSED_sub_91FC42(game: &mut Game) {
    if word!(game.samus_pose) as i32
        == kPose_44_FaceL_Turn_Crouch as i32 | kPose_01_FaceR_Normal as i32 | 0x20
    {
        if word!(game.samus_prev_pose) != 100 {
            return;
        }
    } else if !(word!(game.samus_pose) as i32
        == kPose_44_FaceL_Turn_Crouch as i32 | kPose_02_FaceL_Normal as i32 | 0x20
        && word!(game.samus_prev_pose) == 99)
    {
        return;
    }
    Samus_InitJump(game);
}
pub unsafe fn HandleJumpTransition_NormalJump(game: &mut Game) {
    if word!(game.samus_pose) as i32 == kPose_4B_FaceR_Jumptrans as i32
        || word!(game.samus_pose) as i32 == kPose_4C_FaceL_Jumptrans as i32
        || word!(game.samus_pose) as i32 - kPose_55_FaceR_Jumptrans_AimU as i32 & 0x8000 == 0
            && word!(game.samus_pose) as i32 - kPose_5B as i32 & 0x8000 != 0
    {
        if word!(game.samus_prev_pose) as i32 == kPose_27_FaceR_Crouch as i32
            || word!(game.samus_prev_pose) as i32 == kPose_28_FaceL_Crouch as i32
        {
            word!(game.samus_y_pos, wrapping_sub(10));
        }
        Samus_InitJump(game);
    }
}
pub unsafe fn HandleJumpTransition_SpinJump(game: &mut Game) {
    if *ram_ptr!(game; 0xa23) as i32 != kMovementType_03_SpinJumping as i32
        && *ram_ptr!(game; 0xa23) as i32 != kMovementType_14_WallJumping as i32
    {
        Samus_InitJump(game);
    }
}
pub unsafe fn Samus_Func20(game: &mut Game) {
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
            SamusFunc_F433(game);
            Samus_SetAnimationFrameIfPoseChanged(game);
            word!(game.samus_last_different_pose = word!(game.samus_prev_pose));
            *(ram_ptr!(game, u16; 0xa26)) = *(ram_ptr!(game, u16; 0xa22));
            word!(game.samus_prev_pose = word!(game.samus_pose));
            *(ram_ptr!(game, u16; 0xa22)) = *(ram_ptr!(game, u16; 0xa1e));
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
        SamusFunc_F433(game);
        Samus_SetAnimationFrameIfPoseChanged(game);
        word!(game.samus_last_different_pose = word!(game.samus_prev_pose));
        *(ram_ptr!(game, u16; 0xa26)) = *(ram_ptr!(game, u16; 0xa22));
        word!(game.samus_prev_pose = word!(game.samus_pose));
        *(ram_ptr!(game, u16; 0xa22)) = *(ram_ptr!(game, u16; 0xa1e));
    };
}
static off_91FE8A: [Option<Func_U8>; 4] = callback_table! { unsafe Func_U8;
    HandleCollDueToChangedPose_Solid_NoColl, HandleCollDueToChangedPose_Solid_CollAbove,
    HandleCollDueToChangedPose_Solid_CollBelow, HandleCollDueToChangedPose_Solid_CollBoth,
};
static off_91FE92: [Option<Func_U8>; 4] = callback_table! { unsafe Func_U8;
    HandleCollDueToChangedPose_Block_NoColl, HandleCollDueToChangedPose_Block_CollAbove,
    HandleCollDueToChangedPose_Block_CollBelow, HandleCollDueToChangedPose_Block_CollBoth,
};
pub unsafe fn HandleCollDueToChangedPose(game: &mut Game) {
    let mut cres: CheckEnemyColl_Result = CheckSolidEnemyColl_Result {
        collision: 0,
        amt: 0,
    };
    if word!(game.samus_pose) == 0
        || word!(game.samus_pose) as i32 == kPose_9B_FaceF_VariaGravitySuit as i32
    {
        return;
    }
    word! { game, 0; solid_enemy_collision_flags, block_collision_flags }
    let mut v0: i16 = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
        as *mut SamusPoseParams)
        .offset(word!(game.samus_prev_pose) as isize))
    .y_radius as i16;
    if v0 as i32
        - (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629)) as *mut SamusPoseParams)
            .offset(word!(game.samus_pose) as isize))
        .y_radius as i32
        & 0x8000
        == 0
    {
        return;
    }
    word!(
        game.samus_y_radius = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
            as *mut SamusPoseParams)
            .offset(word!(game.samus_prev_pose) as isize))
        .y_radius as u16
    );
    word!(
        game.samus_y_radius_diff = (((*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
            as *mut SamusPoseParams)
            .offset(word!(game.samus_pose) as isize))
        .y_radius) as u16)
            .wrapping_sub((v0) as u16)
    );
    word!(game.samus_collision_direction = 2);
    cres = Samus_CheckSolidEnemyColl((word!(game.samus_y_radius_diff) as i16 as i32) << 16, game);
    word!(game.samus_collision_flag = cres.collision);
    let mut amt: i32 = cres.amt;
    if word!(game.samus_collision_flag) != 0 {
        word!(game.solid_enemy_collision_flags = 1);
    }
    word!(game.samus_space_to_move_up_enemy = (amt >> 16) as u16);
    word!(game.samus_collision_direction = 3);
    cres = Samus_CheckSolidEnemyColl((word!(game.samus_y_radius_diff) as i16 as i32) << 16, game);
    word!(game.samus_collision_flag = cres.collision);
    amt = cres.amt;
    if word!(game.samus_collision_flag) != 0 {
        word!(game.solid_enemy_collision_flags, |fresh11| (fresh11)
            | (2u16));
    }
    word!(game.samus_space_to_move_down_enemy = (amt >> 16) as u16);
    if off_91FE8A[word!(game.solid_enemy_collision_flags) as usize]
        .expect("non-null function pointer")(game)
        != 0
    {
        word!(game.samus_pose = word!(game.samus_prev_pose));
        return;
    }
    amt = Samus_CollDetectChangedPose(
        (-(word!(game.samus_y_radius_diff) as i32) as i16 as i32) << 16,
        game,
    );
    if word!(game.samus_collision_flag) != 0 {
        word!(game.block_collision_flags = 1);
    }
    word!(game.samus_space_to_move_up_blocks = (amt >> 16) as u16);
    amt = Samus_CollDetectChangedPose((word!(game.samus_y_radius_diff) as i16 as i32) << 16, game);
    if word!(game.samus_collision_flag) != 0 {
        word!(game.block_collision_flags, |fresh12| (fresh12) | (2u16));
    }
    word!(game.samus_space_to_move_down_blocks = (amt >> 16) as u16);
    if off_91FE92[word!(game.block_collision_flags) as usize].expect("non-null function pointer")(
        game,
    ) != 0
    {
        word!(game.samus_pose = word!(game.samus_prev_pose));
    }
}
pub use self::nullsub_164 as HandleCollDueToChangedPose_Solid_NoColl;
pub use self::nullsub_164 as HandleCollDueToChangedPose_Solid_CollBoth;
pub unsafe fn HandleCollDueToChangedPose_Solid_CollAbove(game: &mut Game) -> u8 {
    let v1: u16 = word!(game.samus_y_radius);
    word!(
        game.samus_y_radius = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
            as *mut SamusPoseParams)
            .offset(word!(game.samus_pose) as isize))
        .y_radius as u16
    );
    word!(game.samus_collision_direction = 3);
    let cres: CheckEnemyColl_Result = Samus_CheckSolidEnemyColl(
        ((((word!(game.samus_y_radius_diff)) as i16)
            .wrapping_sub((word!(game.samus_space_to_move_up_enemy)) as i16)) as i32)
            << 16,
        game,
    );
    word!(game.samus_y_radius = v1);
    word!(game.samus_collision_flag = cres.collision);
    if word!(game.samus_collision_flag) != 0 {
        return 1;
    }
    word!(game.samus_space_to_move_up_enemy = (cres.amt >> 16) as u16);
    0
}
pub unsafe fn HandleCollDueToChangedPose_Solid_CollBelow(game: &mut Game) -> u8 {
    let v1: u16 = word!(game.samus_y_radius);
    word!(
        game.samus_y_radius = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
            as *mut SamusPoseParams)
            .offset(word!(game.samus_pose) as isize))
        .y_radius as u16
    );
    word!(game.samus_collision_direction = 2);
    let cres: CheckEnemyColl_Result = Samus_CheckSolidEnemyColl(
        ((((word!(game.samus_y_radius_diff)) as i16)
            .wrapping_sub((word!(game.samus_space_to_move_down_enemy)) as i16)) as i32)
            << 16,
        game,
    );
    word!(game.samus_y_radius = v1);
    word!(game.samus_collision_flag = cres.collision);
    if word!(game.samus_collision_flag) != 0 {
        return 1;
    }
    word!(game.samus_space_to_move_down_enemy = (cres.amt >> 16) as u16);
    0
}
pub unsafe fn HandleCollDueToChangedPose_Block_CollAbove(game: &mut Game) -> u8 {
    let amt: i32 = Samus_CollDetectChangedPose(
        ((((word!(game.samus_y_radius_diff)) as i16)
            .wrapping_sub((word!(game.samus_space_to_move_up_blocks)) as i16)) as i32)
            << 16,
        game,
    );
    if word!(game.samus_collision_flag) != 0 {
        return 1;
    }
    if (word!(game.solid_enemy_collision_flags) & 2) != 0 {
        return HandleCollDueToChangedPose_Block_CollBoth(game);
    }
    word!(game.samus_y_pos, |fresh5| (fresh5 as i32 + (amt >> 16))
        as u16);
    word!(game.samus_prev_y_pos = word!(game.samus_y_pos));
    0
}
pub unsafe fn HandleCollDueToChangedPose_Block_CollBelow(game: &mut Game) -> u8 {
    let amt: i32 = Samus_CollDetectChangedPose(
        ((((word!(game.samus_space_to_move_down_blocks)) as i16)
            .wrapping_sub((word!(game.samus_y_radius_diff)) as i16)) as i32)
            << 16,
        game,
    );
    if word!(game.samus_collision_flag) != 0 {
        return 1;
    }
    if (word!(game.solid_enemy_collision_flags) & 1) != 0 {
        return HandleCollDueToChangedPose_Block_CollBoth(game);
    }
    word!(game.samus_y_pos, |fresh7| (fresh7 as i32 - (amt >> 16))
        as u16);
    word!(game.samus_prev_y_pos = word!(game.samus_y_pos));
    0
}
pub unsafe fn HandleCollDueToChangedPose_Block_NoColl(game: &mut Game) -> u8 {
    match word!(game.solid_enemy_collision_flags) as i32 {
        0 => 0,
        1 => {
            word!(
                game.samus_y_pos,
                wrapping_add(word!(game.samus_space_to_move_up_enemy))
            );
            word!(game.samus_prev_y_pos = word!(game.samus_y_pos));
            0
        }
        2 => {
            word!(
                game.samus_y_pos,
                wrapping_sub(word!(game.samus_space_to_move_down_enemy))
            );
            word!(game.samus_prev_y_pos = word!(game.samus_y_pos));
            0
        }
        3 => HandleCollDueToChangedPose_Block_CollBoth(game),
        _ => {
            Unreachable(game);
            0
        }
    }
}
pub unsafe fn HandleCollDueToChangedPose_Block_CollBoth(game: &mut Game) -> u8 {
    if word!(game.samus_y_radius) as i32 - 8i32 & 0x8000 != 0 {
        return 1;
    }
    word!(
        game.samus_pose = (if *ram_ptr!(game; 0xa1e) == 4 {
            kPose_28_FaceL_Crouch as i32
        } else {
            kPose_27_FaceR_Crouch as i32
        }) as u16
    );
    let r18: u16 = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
        as *mut SamusPoseParams)
        .offset(word!(game.samus_pose) as isize))
    .y_radius as u16;
    if word!(game.samus_y_radius) as i32 - r18 as i32 & 0x8000 != 0 {
        word!(game.samus_y_pos, |fresh6| (fresh6 as i32
            + (word!(game.samus_y_radius) as i32 - r18 as i32))
            as u16);
        word!(game.samus_prev_y_pos = word!(game.samus_y_pos));
    }
    0
}
