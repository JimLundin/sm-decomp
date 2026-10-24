use crate::game::player_transitions as pt;

use crate::game::actors::api as actor_api;

pub use crate::game::prelude::*;
use crate::game::session_runtime::Runtime;
use crate::game::{
    player_collision::operations as player_collision, session_runtime::Runtime as CollisionRuntime,
};

pub use crate::game::constants::bank_8d::*;
pub use crate::game::constants::bank_8f::*;
pub use crate::game::constants::bank_9b::*;
pub use crate::game::constants::bank_91::*;

use crate::game::animated_tiles;
use crate::game::enemy_projectiles::{DisableEprojs, EnableEprojs};
use crate::game::palette_effects;
use crate::game::room_mechanisms::set_enabled;

use crate::game::sm_80::{
    InvalidInterrupt_Crash, QueueSfx1_Max6, QueueSfx1_Max9, QueueSfx2_Max6, QueueSfx3_Max6,
};
use crate::game::sm_88::SpawnHdmaObject;
use crate::game::sm_90::{CallSomeSamusCode, LoadProjectilePalette, UpdateBeamTilesAndPalette};

crate::typed_struct! {
    [
        #[derive(Copy, Clone)]
        #[repr(C, packed)]
    ];
    pub struct DemoInputObject {
        pub ptr, pre_instr, instr_ptr: VoidP;
    }

    pub struct DemoSetDef {
        pub items, missiles, super_missiles, power_bombs, health, beams, equipped_beams_: u16;
        pub demo_obj: VoidP;
    }

    pub struct PoseEntry {
        pub new_input, cur_input, new_pose: u16;
    }


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
const RomPtr_91: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x91>;
const RomPtr_9B: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x9b>;
#[inline]
unsafe fn get_PoseEntry(a: u16, game: &mut Game) -> *mut PoseEntry {
    unsafe { RomPtr((0x910000u32) | ((a) as u32), game) as *mut PoseEntry }
}
static kSamusInputHandlers: [Func_V; 28] = callback_table! { nonnull Func_V;
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
    unsafe {
        kSamusInputHandlers[*ram_ptr!(game; 0xa1f) as usize](game);
    }
}
pub unsafe fn Samus_Input_00_Standing(game: &mut Game) {
    unsafe {
        if word!(game.samus_pose) != 0
            && word!(game.samus_pose) as i32 != kPose_9B_FaceF_VariaGravitySuit as i32
            || word!(game.elevator_status) == 0
        {
            Samus_LookupTransitionTable(game);
        }
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
    unsafe {
        if word!(game.time_is_frozen_flag) != 0 {
            Samus_Func20(game);
        } else {
            Samus_LookupTransitionTable(game);
            if *ram_ptr!(game; 0xa1f) == 0 {
                word!(game.samus_y_pos -= 5);
                word!(game.samus_prev_y_pos -= 5);
            }
        };
    }
}
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    Samus_Input_06_Falling => Samus_LookupTransitionTable(game);
    Samus_Input_08_MorphBallFalling => Samus_LookupTransitionTable(game);
}
pub fn Samus_Input_09_Unused(game: &mut Game) {}
pub unsafe fn Samus_Input_0A_KnockbackOrCrystalFlashEnding(game: &mut Game) {
    unsafe {
        Samus_LookupTransitionTable(game);
        if *ram_ptr!(game; 0xa1f) as i32 != kMovementType_0A_KnockbackOrCrystalFlashEnding as i32 {
            pt::initialize_jump(&mut Runtime::new(game), pt::state::JumpKind::Normal);
            word!(game.samus_knockback_timer = 0);
        }
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
    unsafe {
        if word!(game.time_is_frozen_flag) != 0 {
            Samus_Func20(game);
        } else {
            Samus_LookupTransitionTable(game);
        };
    }
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
    unsafe {
        if word!(game.joypad1_lastkeys) != 0 {
            let pair: Pair_R18_R20 = TranslateCustomControllerBindingsToDefault(game);
            let mut pe = get_PoseEntry(
                crate::rtl::rom_table_word!(game, 0x919ee2, word!(game.samus_pose) as isize),
                game,
            );
            if (*pe).new_input == 0xffff {
                return;
            }
            do_until! { ((*pe).new_input == 0xffff);
                if (pair.r18 & (*pe).new_input) == 0 && (pair.r20 & (*pe).cur_input) == 0 {
                    if (*pe).new_pose != word!(game.samus_pose) {
                        word! {game; samus_new_pose = (*pe).new_pose; bomb_jump_dir = 0; }
                    }
                    return;
                }
                pe = pe.offset(1);
            }
        }
        word!(game.UNUSED_word_7E0A18 = 0);
        Samus_Pose_CancelGrapple(game);
    }
}

macro_rules! translate_configured_buttons {
    ($game:ident, $input_field:ident, $translated_buttons:ident, $input_buttons:ident) => {
        if (word!($game.button_config_shoot_x) & word!($game.$input_field)) != 0 {
            $translated_buttons = ($translated_buttons) | ((kButton_X) as u16);
        }
        if (word!($game.button_config_jump_a) & $input_buttons) != 0 {
            $translated_buttons = ($translated_buttons) | ((kButton_A) as u16);
        }
        if (word!($game.button_config_run_b) & $input_buttons) != 0 {
            $translated_buttons = ($translated_buttons) | ((kButton_B) as u16);
        }
        if (word!($game.button_config_itemcancel_y) & $input_buttons) != 0 {
            $translated_buttons = ($translated_buttons) | ((kButton_Y) as u16);
        }
        if (word!($game.button_config_aim_up_R) & $input_buttons) != 0
            && word!($game.button_config_aim_up_R) as i32 & (kButton_L as i32 | kButton_R as i32)
                != 0
        {
            $translated_buttons = ($translated_buttons) | ((kButton_R) as u16);
        }
        if (word!($game.button_config_aim_down_L) & $input_buttons) != 0
            && word!($game.button_config_aim_down_L) as i32 & (kButton_L as i32 | kButton_R as i32)
                != 0
        {
            $translated_buttons = ($translated_buttons) | ((kButton_L) as u16);
        }
    };
}

pub unsafe fn TranslateCustomControllerBindingsToDefault(game: &mut Game) -> Pair_R18_R20 {
    unsafe {
        let mut r18: u16 = (word!(game.joypad1_newkeys) as i32
            & (kButton_Up as i32
                | kButton_Down as i32
                | kButton_Left as i32
                | kButton_Right as i32)) as u16;
        let mut r20: u16 = (word!(game.joypad1_lastkeys) as i32
            & (kButton_Up as i32
                | kButton_Down as i32
                | kButton_Left as i32
                | kButton_Right as i32)) as u16;
        let v0: u16 = word!(game.joypad1_newkeys);
        translate_configured_buttons!(game, joypad1_newkeys, r18, v0);
        let v1: u16 = word!(game.joypad1_lastkeys);
        translate_configured_buttons!(game, joypad1_lastkeys, r20, v1);
        Pair_R18_R20 {
            r18: !(r18 as i32) as u16,
            r20: !(r20 as i32) as u16,
        }
    }
}
pub unsafe fn Samus_Pose_CancelGrapple(game: &mut Game) {
    unsafe {
        if Samus_Pose_Func2(game) & 1 != 0
            || (*(rom_ptr!(game, SamusPoseParams; 0x91b629))
                .offset(word!(game.samus_pose) as isize))
            .new_pose_unless_buttons
                == 255
        {
            word!(game.samus_new_pose = word!(game.samus_pose));
        } else {
            word!(
                game.samus_new_pose = (*(rom_ptr!(game, SamusPoseParams; 0x91b629))
                    .offset(word!(game.samus_pose) as isize))
                .new_pose_unless_buttons as u16
            );
        };
    }
}
static kSamus_Pose_Func2_Tab: [u8; 28] = [
    2, 1, 1, 0, 6, 2, 8, 2, 1, 6, 2, 2, 2, 6, 2, 2, 2, 6, 6, 6, 6, 2, 6, 2, 2, 2, 2, 2,
];
pub unsafe fn Samus_Pose_Func2(game: &mut Game) -> u8 {
    unsafe {
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
}
pub fn EnableDemoInput(game: &mut Game) {
    game.set_samus_input_handler(fnSamus_InputHandler_E91D as u16);
    game.set_demo_enable(game.demo_enable() | 0x8000);
}
pub unsafe fn DisableDemoInput(game: &mut Game) {
    unsafe {
        word!(game.samus_input_handler = ((0x90e913i32) as u16));
        word!(game.demo_enable &= !(0x8000));
    }
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
    unsafe {
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
}
pub unsafe fn DemoObjectInputHandler(game: &mut Game) {
    unsafe {
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
}
crate::game::callback_dispatch! {
    pub unsafe fn CallDemoPreInstr(address: u32, game: &mut Game) => address;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [0x91864f] DemoPreInstr_864F [] {
        if word!(game.samus_x_pos) as i32 - 178i32 & 0x8000 != 0 {
            word! {game; demo_input_pre_instr = ((0x91866ai32) as u16);
                demo_input_instr_ptr = 0x8623; demo_input_instr_timer = 1; }
        }
    };
    [0x91866a] DemoPreInstr_866A [] {
        if word!(game.eproj_x_pos[0]) == 0 {
            word! {game; demo_input_pre_instr = ((0x9183bfi32) as u16);
                demo_input_instr_ptr = addr_off_91864B as u16; demo_input_instr_timer = 1; }
        }
    };
    [0x918a9b] DemoPreInstr_CheckLeaveDemo [] {
        if word!(game.game_state) as i32 == kGameState_44_TransitionFromDemo as i32 {
            word!(game.demo_input_instr_ptr = addr_kDemoInstrs_LeaveDemo as u16);
            word!(game.demo_input_instr_timer = 1);
        }
    };
    @functions [pub fn (game: &mut Game)];
    [0x918ab0] DemoPreInstr_8AB0 [] {
        if game.samus_movement_type() != kMovementType_1A_GrabbedByDraygon as u8 {
            game.set_demo_input_pre_instr(fnDemoPreInstr_CheckLeaveDemo as u16);
            game.set_demo_input_instr_ptr(addr_stru_919346 as u16);
            game.set_demo_input_instr_timer(1);
        }
    };
    @external {
        0x9183bf => [()];
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallDemoInstr(address: u32, k: u16, j: u16, game: &mut Game) -> u16 => address;
    call (k, j, game); default Unreachable(game) as u16;
    @functions [pub fn (_k: u16, _j: u16, game: &mut Game) -> u16];
    [0x918427] DemoInstr_Finish [] {
        game.set_demo_input_instr_ptr(0);
        game.set_demo_input(0);
        game.set_demo_input_new(0);
        0
    };
    @functions [pub unsafe fn (k: u16, j: u16, game: &mut Game) -> u16];
    [0x918434] DemoInstr_SetPreInstr [(j).wrapping_add(2)] {
        word!(game.demo_input_pre_instr = *(RomPtr_91(j, game) as *mut u16));
    };
    [0x91843f] DemoInstr_ClearPreInstr [j] { word!(game.demo_input_pre_instr = 0x8447); };
    [0x918448] DemoInstr_Goto [*(RomPtr_91(j, game) as *mut u16)] { };
    [0x91844f] DemoInstr_DecTimerAndGoto [] {
        if word!(game.demo_timer_counter -= 1, reread) != 0 {
            DemoInstr_Goto(k, j, game)
        } else {
            (j).wrapping_add(2)
        }
    };
    [0x918459] DemoInstr_SetTimer [(j).wrapping_add(2)] {
        word!(game.demo_timer_counter = *(RomPtr_91(j, game) as *mut u16));
    };
    [0x9185fc] DemoInstr_Func2 [j] {
        word!(game.frame_handler_alfa = ((0x90e695i32) as u16));
        word!(game.frame_handler_beta = ((0x90e725i32) as u16));
        DisableDemoInput(game);
    };
    @functions [pub unsafe fn (k: u16, j: u16, game: &mut Game) -> u16];
    [0x918682] DemoInstr_Disable [j] {
        word!(game.frame_handler_alfa = ((0x90e8cdi32) as u16));
        word!(game.frame_handler_beta = ((0x90e8cdi32) as u16));
        DisableDemoInput(game);
    };
    [0x918739] DemoInstr_Func3 [j] {
        word!(game.frame_handler_alfa = ((0x90e713i32) as u16));
        word!(game.samus_pose = kPose_02_FaceL_Normal as u16);
        pt::finish_pose_change(&mut Runtime::new(game));
        DisableDemoInput(game);
        word!(game.samus_input_handler = ((0x90e90ei32) as u16));
    };
}
pub unsafe fn ProcessDemoInputObject(game: &mut Game) {
    unsafe {
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
}
static kDemoSetFuncPtrs_0: [Func_V; 6] = callback_table! { nonnull Func_V;
    DemoSetFunc_0, DemoSetFunc_1, DemoSetFunc_1, DemoSetFunc_2, DemoSetFunc_1, DemoSetFunc_3,
};
static kDemoSetFuncPtrs_1: [Func_V; 6] = callback_table! { nonnull Func_V;
    DemoSetFunc_2, DemoSetFunc_2, DemoSetFunc_1, DemoSetFunc_2, DemoSetFunc_1, DemoSetFunc_4,
};
static kDemoSetFuncPtrs_2: [Func_V; 6] = callback_table! { nonnull Func_V;
    DemoSetFunc_2, DemoSetFunc_5, DemoSetFunc_1, DemoSetFunc_1, DemoSetFunc_1, DemoSetFunc_1,
};
static kDemoSetFuncPtrs_3: [Func_V; 5] = callback_table! { nonnull Func_V;
    DemoSetFunc_6, DemoSetFunc_2, DemoSetFunc_2, DemoSetFunc_1, DemoSetFunc_7,
};
static kDemoSetFuncPtrs: [&'static [Func_V]; 4] = [
    &kDemoSetFuncPtrs_0,
    &kDemoSetFuncPtrs_1,
    &kDemoSetFuncPtrs_2,
    &kDemoSetFuncPtrs_3,
];
pub unsafe fn LoadDemoData(game: &mut Game) {
    unsafe {
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
        let initialize_scene = kDemoSetFuncPtrs[set][scene];
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
}
pub unsafe fn DemoSetFunc_0(game: &mut Game) {
    unsafe {
        MakeSamusFaceForward(game);
        game.set_samus_draw_handler(fnSamusDrawHandler_Default as u16);
    }
}
pub unsafe fn DemoSetFunc_3(game: &mut Game) {
    unsafe {
        DemoSetFunc_Common(kPose_1F_MoveL_Morphball_Ground as u16, game);
    }
}
pub unsafe fn DemoSetFunc_7(game: &mut Game) {
    unsafe {
        word!(game.samus_health = 20);
        DemoSetFunc_2(game);
    }
}
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    DemoSetFunc_2 => DemoSetFunc_Common(kPose_02_FaceL_Normal as u16, game);
    DemoSetFunc_4 => DemoSetFunc_Common(kPose_2A_FaceL_Fall as u16, game);
    DemoSetFunc_1 => DemoSetFunc_Common(kPose_01_FaceR_Normal as u16, game);
}
pub unsafe fn DemoSetFunc_Common(pose: u16, game: &mut Game) {
    unsafe {
        game.set_samus_pose(pose);
        pt::refresh_pose(&mut Runtime::new(game));
        game.set_samus_draw_handler(fnSamusDrawHandler_Default as u16);
    }
}
pub unsafe fn DemoSetFunc_5(game: &mut Game) {
    unsafe {
        game.set_samus_draw_handler(fnSamusDrawHandler_Default as u16);
        pt::start_shinespark(&mut Runtime::new(game));
        game.set_samus_pose(kPose_CD_FaceR_Shinespark_Diag as u16);
        pt::refresh_pose(&mut Runtime::new(game));
    }
}
pub unsafe fn DemoSetFunc_6(game: &mut Game) {
    unsafe {
        word!(game.samus_draw_handler = ((0x90eb52i32) as u16));
        pt::start_shinespark(&mut Runtime::new(game));
        word!(game.samus_pose = kPose_CA_FaceL_Shinespark_Horiz as u16);
        pt::refresh_pose(&mut Runtime::new(game));
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
pub unsafe fn VariaSuitPickup(game: &mut Game) {
    unsafe {
        *ram_ptr!(game; 0xdf0) = 48;
        *ram_ptr!(game; 0xdf1) = 80;
        *ram_ptr!(game; 0xdf2) = 0x80;
        *ram_ptr!(game; 0xdf3) = 0;
        Samus_CancelSpeedBoost(game);
        words!(game.samus_x_extra_run_speed, samus_x_extra_run_subspeed = 0);
        word! { game, 0;
            samus_x_base_speed, samus_x_base_subspeed, samus_y_subspeed, samus_y_speed, samus_y_dir,
            used_for_ball_bounce_on_landing, samus_x_accel_mode, elevator_status, substate,
            suit_pickup_light_beam_pos,
        }
        word!(game.suit_pickup_light_beam_widening_speed = 256);
        word_transfers!(game, descending, 256; le(hdma_table_1, 255) <- value(255));
        if *ram_ptr!(game; 0xa1f) == 3 || *ram_ptr!(game; 0xa1f) == 20 {
            QueueSfx1_Max9(0x32, game);
        }
        if (word!(game.equipped_items) & 0x20) != 0 {
            word!(game.samus_pose = kPose_9B_FaceF_VariaGravitySuit as u16);
        } else {
            word!(game.samus_pose = kPose_00_FaceF_Powersuit as u16);
        }
        pt::refresh_pose(&mut Runtime::new(game));
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
}
pub unsafe fn GravitySuitPickup(game: &mut Game) {
    unsafe {
        *ram_ptr!(game; 0xdf0) = 48;
        *ram_ptr!(game; 0xdf1) = 73;
        *ram_ptr!(game; 0xdf2) = -112i32 as u8;
        *ram_ptr!(game; 0xdf3) = 1;
        Samus_CancelSpeedBoost(game);
        words!(game.samus_x_extra_run_speed, samus_x_extra_run_subspeed = 0);
        word! { game, 0;
            samus_x_base_speed, samus_x_base_subspeed, samus_y_subspeed, samus_y_speed, samus_y_dir,
            used_for_ball_bounce_on_landing, samus_x_accel_mode, elevator_status, substate,
            suit_pickup_light_beam_pos,
        }
        word!(game.suit_pickup_light_beam_widening_speed = 256);
        word_transfers!(game, descending, 256; le(hdma_table_1, 255) <- value(255));
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
        pt::refresh_pose(&mut Runtime::new(game));
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
}
pub unsafe fn InitializeSuitPickupHdma(game: &mut Game) {
    unsafe {
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
// Shared by unrelated transition/collision aliases below as well as old palette mode10.
pub fn nullsub_164(_game: &mut Game) -> u8 {
    0
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
pub unsafe fn Samus_Initialize(game: &mut Game) {
    unsafe {
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
pub unsafe fn MakeSamusFaceForward(game: &mut Game) {
    unsafe {
        if (word!(game.equipped_items) & 0x20) != 0 || (word!(game.equipped_items) & 1) != 0 {
            word!(game.samus_pose = kPose_9B_FaceF_VariaGravitySuit as u16);
        } else {
            word!(game.samus_pose = kPose_00_FaceF_Powersuit as u16);
        }
        pt::finish_pose_change(&mut Runtime::new(game));
        if word!(game.samus_y_radius) != 24 {
            word!(game.samus_y_pos -= 3);
            word!(game.samus_prev_y_pos = word!(game.samus_y_pos));
        }
        word! {game; frame_handler_alfa = ((0x90e713i32) as u16);
        frame_handler_beta = ((0x90e8dci32) as u16); samus_new_pose = -1i32 as u16;
        samus_new_pose_interrupted = -1i32 as u16; samus_new_pose_transitional = -1i32 as u16; }
        word! { game, 0;
            samus_momentum_routine_index, samus_special_transgfx_index, samus_hurt_switch_index,
        }
        CallSomeSamusCode(0x1f, game);
        words!(game.samus_x_extra_run_speed, samus_x_extra_run_subspeed = 0);
        word! { game, 0;
            samus_x_base_speed, samus_x_base_subspeed, samus_y_subspeed, samus_y_speed, samus_y_dir,
            used_for_ball_bounce_on_landing, samus_x_accel_mode, flare_counter, flare_animation_frame,
            flare_slow_sparks_anim_frame, flare_fast_sparks_anim_frame, flare_animation_timer,
            flare_slow_sparks_anim_timer, flare_fast_sparks_anim_timer,
        }
        Samus_LoadSuitPalette(game);
    }
}
static kSomeMotherBrainScripts: [Func_U8; 5] = callback_table! { nonnull Func_U8;
    SomeMotherBrainScripts_0, SomeMotherBrainScripts_1, SomeMotherBrainScripts_2,
    SomeMotherBrainScripts_3_EnableHyperBeam, SomeMotherBrainScripts_4,
};
pub unsafe fn SomeMotherBrainScripts(a: u16, game: &mut Game) {
    unsafe {
        if kSomeMotherBrainScripts[a as usize](game) & 1 != 0 {
            pt::record_pose_history(Runtime::new(game).session_mut());
            word! {game; samus_new_pose = -1i32 as u16; samus_new_pose_interrupted = -1i32 as u16;
            samus_new_pose_transitional = -1i32 as u16; }
            word! { game, 0;
                samus_momentum_routine_index, samus_special_transgfx_index, samus_hurt_switch_index,
            }
        }
    }
}
pub unsafe fn SomeMotherBrainScripts_0(game: &mut Game) -> u8 {
    unsafe {
        word!(
            checked_i32,
            game.samus_y_pos -= (21 - word!(game.samus_y_radius) as i32)
        );
        word!(game.samus_prev_y_pos = word!(game.samus_y_pos));
        if *ram_ptr!(game; 0xa1e) == 4 {
            word!(game.samus_pose = kPose_E9_FaceL_Drained_CrouchFalling as u16);
        } else {
            word!(game.samus_pose = kPose_E8_FaceR_Drained_CrouchFalling as u16);
        }
        word!(game.samus_anim_frame_skip = 2);
        pt::refresh_pose(&mut Runtime::new(game));
        word!(
            game.samus_y_radius = (*(rom_ptr!(game, SamusPoseParams; 0x91b629))
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
}
pub unsafe fn SomeMotherBrainScripts_1(game: &mut Game) -> u8 {
    unsafe {
        word! {game; samus_anim_frame_timer = 16; samus_anim_frame = 0; }
        if *ram_ptr!(game; 0xa1e) == 4 {
            word!(game.samus_pose = kPose_EB_FaceL_Drained_Stand as u16);
        } else {
            word!(game.samus_pose = kPose_EA_FaceR_Drained_Stand as u16);
        }
        word!(game.frame_handler_gamma = ((0x90e90ei32) as u16));
        1
    }
}
pub unsafe fn SomeMotherBrainScripts_2(game: &mut Game) -> u8 {
    unsafe {
        if word!(game.samus_pose) as i32 == kPose_E8_FaceR_Drained_CrouchFalling as i32
            || word!(game.samus_pose) as i32 == kPose_E9_FaceL_Drained_CrouchFalling as i32
        {
            word! {game; samus_anim_frame_timer = 1; samus_anim_frame = 13; }
        } else if word!(game.samus_pose) as i32 == kPose_EA_FaceR_Drained_Stand as i32
            || word!(game.samus_pose) as i32 == kPose_EB_FaceL_Drained_Stand as i32
        {
            word! {game; samus_anim_frame_timer = 1; samus_anim_frame = 4; }
        }
        word!(
            game.samus_y_radius = (*(rom_ptr!(game, SamusPoseParams; 0x91b629))
                .offset(word!(game.samus_pose) as isize))
            .y_radius as u16
        );
        word! { game, 0;
            samus_x_base_speed, samus_x_base_subspeed, samus_y_subspeed, samus_y_speed,
        }
        word!(game.samus_y_dir = 2);
        1
    }
}
pub unsafe fn SomeMotherBrainScripts_3_EnableHyperBeam(game: &mut Game) -> u8 {
    unsafe {
        word!(game.equipped_beams = 4105);
        UpdateBeamTilesAndPalette(game);
        palette_effects::spawn(addr_stru_8DE1F0 as u16, game);
        word! {game; hyper_beam_flag = ((0x918000i32) as u16); play_resume_charging_beam_sfx = 0; }
        0
    }
}
pub unsafe fn SomeMotherBrainScripts_4(game: &mut Game) -> u8 {
    unsafe {
        word! {game; samus_anim_frame_timer = 16; samus_anim_frame = 8; }
        if *ram_ptr!(game; 0xa1e) == 4 {
            word!(game.samus_pose = kPose_E9_FaceL_Drained_CrouchFalling as u16);
        } else {
            word!(game.samus_pose = kPose_E8_FaceR_Drained_CrouchFalling as u16);
        }
        1
    }
}
pub fn nullsub_17(game: &mut Game) {}
static off_91E6E1: [Func_V; 28] = callback_table! { nonnull Func_V;
    SamusFunc_E633_0, nullsub_17, nullsub_17, SamusFunc_E633_3, SamusFunc_E633_4, nullsub_17,
    nullsub_17, nullsub_17, SamusFunc_E633_4, nullsub_17, nullsub_17, nullsub_17, nullsub_17,
    nullsub_17, nullsub_17, nullsub_17, nullsub_17, SamusFunc_E633_17, SamusFunc_E633_17,
    SamusFunc_E633_17, SamusFunc_E633_20, nullsub_17, nullsub_17, nullsub_17, nullsub_17,
    nullsub_17, nullsub_17, nullsub_17,
};
pub unsafe fn SamusFunc_E633(game: &mut Game) {
    unsafe {
        off_91E6E1[*ram_ptr!(game; 0xa1f) as usize](game);
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
}
pub unsafe fn SamusFunc_E633_0(game: &mut Game) {
    unsafe {
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
            pt::finish_pose_change(&mut Runtime::new(game));
        };
    }
}
pub unsafe fn SamusFunc_E633_3(game: &mut Game) {
    unsafe {
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
        pt::refresh_pose(&mut Runtime::new(game));
        if *ram_ptr!(game; 0xa1e) == 4 {
            *(ram_ptr!(game, u16; 0xa22)) = 772;
        } else {
            *(ram_ptr!(game, u16; 0xa22)) = 776;
        }
        pt::record_pose_history(Runtime::new(game).session_mut());
    }
}
pub unsafe fn SamusFunc_E633_4(game: &mut Game) {
    unsafe {
        if (word!(game.equipped_items) & 2) != 0 {
            if *ram_ptr!(game; 0xa1e) == 4 {
                word!(game.samus_pose = kPose_7A_FaceL_Springball_Ground as u16);
            } else {
                word!(game.samus_pose = kPose_79_FaceR_Springball_Ground as u16);
            }
            pt::finish_pose_change(&mut Runtime::new(game));
        }
    }
}
pub unsafe fn SamusFunc_E633_17(game: &mut Game) {
    unsafe {
        if (word!(game.equipped_items) & 2) == 0 {
            if *ram_ptr!(game; 0xa1e) == 4 {
                word!(game.samus_pose = kPose_41_FaceL_Morphball_Ground as u16);
            } else {
                word!(game.samus_pose = kPose_1D_FaceR_Morphball_Ground as u16);
            }
            pt::finish_pose_change(&mut Runtime::new(game));
        }
    }
}
pub unsafe fn SamusFunc_E633_20(game: &mut Game) {
    unsafe {
        if (word!(game.equipped_items) & 8) != 0 {
            word!(game.samus_anim_frame = 23);
        } else {
            word!(game.samus_anim_frame = 3);
        };
    }
}
pub fn nullsub_18(game: &mut Game) {}
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
