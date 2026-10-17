//! Shared byte-compatible gameplay, hardware and music types.

use crate::game_state::Game;
pub type uint8 = u8;
pub type uint16 = u16;
pub type VoidP = uint16;

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct LongPtr {
    pub addr: VoidP,
    pub bank: uint8,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct OamEnt {
    pub xcoord: uint8,
    pub ycoord: uint8,
    pub charnum: uint8,
    pub flags: uint8,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct VramWriteEntry {
    pub size: uint16,
    pub src: LongPtr,
    pub vram_dst: uint16,
}

pub type Buttons = ::core::ffi::c_uint;

pub type CalculateAngleFromXYFunc = unsafe fn(uint16, &mut Game) -> uint8;

pub type Consts_60 = ::core::ffi::c_uint;

pub type Consts_80 = ::core::ffi::c_uint;

pub type Consts_81 = ::core::ffi::c_uint;

pub type Consts_82 = ::core::ffi::c_uint;

pub type Consts_83 = ::core::ffi::c_uint;

pub type Consts_84 = ::core::ffi::c_uint;

pub type Consts_86 = ::core::ffi::c_uint;

pub type Consts_87 = ::core::ffi::c_uint;

pub type Consts_88 = ::core::ffi::c_uint;

pub type Consts_89 = ::core::ffi::c_uint;

pub type Consts_8B = ::core::ffi::c_uint;

pub type Consts_8C = ::core::ffi::c_uint;

pub type Consts_8D = ::core::ffi::c_uint;

pub type Consts_8F = ::core::ffi::c_uint;

pub type Consts_90 = ::core::ffi::c_uint;

pub type Consts_91 = ::core::ffi::c_uint;

pub type Consts_94 = ::core::ffi::c_uint;

pub type Consts_9B = ::core::ffi::c_uint;

pub type Consts_A0 = ::core::ffi::c_uint;

pub type Consts_A2 = ::core::ffi::c_uint;

pub type Consts_A3 = ::core::ffi::c_uint;

pub type Consts_A4 = ::core::ffi::c_uint;

pub type Consts_A5 = ::core::ffi::c_uint;

pub type Consts_A6 = ::core::ffi::c_uint;

pub type Consts_A7 = ::core::ffi::c_uint;

pub type Consts_A8 = ::core::ffi::c_uint;

pub type Consts_A9 = ::core::ffi::c_uint;

pub type Consts_AA = ::core::ffi::c_uint;

pub type Consts_AD = ::core::ffi::c_uint;

pub type Consts_B2 = ::core::ffi::c_uint;

pub type Consts_B3 = ::core::ffi::c_uint;

pub type Consts_B4 = ::core::ffi::c_uint;

pub type Consts_B7 = ::core::ffi::c_uint;

pub type CoroutineRet = uint8_t;

pub type CrocomireFunc = unsafe fn(uint16, *const uint16, &mut Game) -> *const uint16;

pub type DspReg = ::core::ffi::c_uint;

pub type EnemyAiBits = ::core::ffi::c_uint;

pub type EnemyProps = ::core::ffi::c_uint;

pub type FuncXY_V = unsafe fn(uint16, uint16, &mut Game) -> ();

pub type Func_AnimDelay = unsafe fn(*const uint8, &mut Game) -> bool;

pub type Func_CalcHdma = unsafe fn(uint16, uint16, &mut Game) -> ();

pub type Func_SpazerSba_V = unsafe fn(uint16, uint16, &mut Game) -> ();

pub type Func_U8 = unsafe fn(&mut Game) -> uint8;

pub type Func_V = unsafe fn(&mut Game) -> ();

pub type Func_V_A = unsafe fn(&mut Game) -> uint8;

pub type Func_V_Coroutine = unsafe fn(&mut Game) -> CoroutineRet;

pub type Func_Y_V = unsafe fn(uint16, &mut Game) -> ();

pub type Func_Y_Y = unsafe fn(uint16, &mut Game) -> uint16;

pub type GLDEBUGPROC = Option<
    unsafe extern "C" fn(
        GLenum,
        GLenum,
        GLuint,
        GLenum,
        GLsizei,
        *const GLchar,
        *const ::core::ffi::c_void,
    ) -> (),
>;

pub type GLSLScaleType = ::core::ffi::c_uint;

pub type GLbitfield = ::core::ffi::c_uint;

pub type GLboolean = ::core::ffi::c_uchar;

pub type GLbyte = ::core::ffi::c_schar;

pub type GLchar = ::core::ffi::c_char;

pub type GLdouble = ::core::ffi::c_double;

pub type GLenum = ::core::ffi::c_uint;

pub type GLfloat = ::core::ffi::c_float;

pub type GLint = ::core::ffi::c_int;

pub type GLint64 = int64_t;

pub type GLintptr = ptrdiff_t;

pub type GLshort = ::core::ffi::c_short;

pub type GLsizei = ::core::ffi::c_int;

pub type GLsizeiptr = ptrdiff_t;

pub type GLubyte = ::core::ffi::c_uchar;

pub type GLuint = ::core::ffi::c_uint;

pub type GLushort = ::core::ffi::c_ushort;

pub type GameState = ::core::ffi::c_uint;

pub type HandlerFunc = unsafe fn(&mut Game) -> ();

pub type LiquidPhysicsType = ::core::ffi::c_uint;

pub type LoadingGameState = ::core::ffi::c_uint;

pub type PFN_LOADFUNCPOINTERS = Option<unsafe fn() -> ::core::ffi::c_int>;

pub type PpuZbufType = uint16_t;

pub type ProjectileType = ::core::ffi::c_uint;

pub type RunFrameFunc = unsafe fn(uint16, ::core::ffi::c_int, &mut Game) -> ();

pub type RunMode = ::core::ffi::c_uint;

pub type SDL_AudioCallback =
    Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut Uint8, ::core::ffi::c_int) -> ()>;

pub type SDL_AudioDeviceID = Uint32;

pub type SDL_AudioFormat = Uint16;

pub type SDL_FingerID = Sint64;

pub type SDL_GLContext = *mut ::core::ffi::c_void;

pub type SDL_GLattr = ::core::ffi::c_uint;

pub type SDL_GestureID = Sint64;

pub type SDL_HitTestResult = ::core::ffi::c_uint;

pub type SDL_JoystickID = Sint32;

pub type SDL_JoystickPowerLevel = ::core::ffi::c_int;

pub type SDL_Keycode = Sint32;

pub type SDL_Keymod = ::core::ffi::c_uint;

pub type SDL_Scancode = ::core::ffi::c_uint;

pub type SDL_TouchID = Sint64;

pub type SDL_bool = ::core::ffi::c_uint;

pub type SamusMovementType = ::core::ffi::c_uint;

pub type SamusPose = ::core::ffi::c_uint;

pub type SaveLoadFunc =
    unsafe fn(*mut ::core::ffi::c_void, *mut ::core::ffi::c_void, size_t, &mut Game) -> ();

pub type Sint16 = int16_t;

pub type Sint32 = int32_t;

pub type Sint64 = int64_t;

pub type SnesRegs = ::core::ffi::c_uint;

pub type SyncAllFunc = unsafe fn(&mut Game) -> ();

pub type Uint16 = uint16_t;

pub type Uint32 = uint32_t;

pub type Uint64 = uint64_t;

pub type Uint8 = uint8_t;

pub type _IO_lock_t = ();

pub type __int16_t = i16;

pub type __int32_t = i32;

pub type __int64_t = i64;

pub type __int8_t = i8;

pub type __mode_t = ::core::ffi::c_uint;

pub type __off64_t = ::core::ffi::c_long;

pub type __off_t = ::core::ffi::c_long;

pub type __time_t = ::core::ffi::c_long;

pub type __uint16_t = u16;

pub type __uint32_t = u32;

pub type __uint64_t = u64;

pub type __uint8_t = u8;

pub type int16 = int16_t;

pub type int16_t = __int16_t;

pub type int32 = int32_t;

pub type int32_t = __int32_t;

pub type int64_t = __int64_t;

pub type int8 = int8_t;

pub type int8_t = __int8_t;

pub type ogl_LoadStatus = ::core::ffi::c_uint;

pub type ptrdiff_t = isize;

pub type size_t = usize;

pub type stbi__uint16 = uint16_t;

pub type stbi__uint32 = uint32_t;

pub type stbi_uc = ::core::ffi::c_uchar;

pub type stbi_us = ::core::ffi::c_ushort;

pub type time_t = __time_t;

pub type uint = ::core::ffi::c_uint;

pub type uint16_t = __uint16_t;

pub type uint32 = uint32_t;

pub type uint32_t = __uint32_t;

pub type uint64 = uint64_t;

pub type uint64_t = __uint64_t;

pub type uint8_t = __uint8_t;

pub type uintptr_t = usize;

pub type CheckEnemyColl_Result = CheckSolidEnemyColl_Result;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CheckSolidEnemyColl_Result {
    pub collision: uint16,
    pub amt: int32,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct DoorDef {
    pub room_definition_ptr: VoidP,
    pub door_bitflags: uint8,
    pub door_orientation: uint8,
    pub x_pos_plm: uint8,
    pub y_pos_plm: uint8,
    pub x_pos_in_room: uint8,
    pub y_pos_in_room: uint8,
    pub samus_distance_from_door: uint16,
    pub door_setup_code: VoidP,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct EnemyBase {
    pub enemy_ptr: uint16,
    pub x_pos: uint16,
    pub x_subpos: uint16,
    pub y_pos: uint16,
    pub y_subpos: uint16,
    pub x_width: uint16,
    pub y_height: uint16,
    pub properties: uint16,
    pub extra_properties: uint16,
    pub ai_handler_bits: uint16,
    pub health: uint16,
    pub spritemap_pointer: uint16,
    pub timer: uint16,
    pub current_instruction: uint16,
    pub instruction_timer: uint16,
    pub palette_index: uint16,
    pub vram_tiles_index: uint16,
    pub layer: uint16,
    pub flash_timer: uint16,
    pub frozen_timer: uint16,
    pub invincibility_timer: uint16,
    pub shake_timer: uint16,
    pub frame_counter: uint16,
    pub bank: uint8,
    pub field_2F: uint8,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct EnemyData {
    pub enemy_ptr: uint16,
    pub x_pos: uint16,
    pub x_subpos: uint16,
    pub y_pos: uint16,
    pub y_subpos: uint16,
    pub x_width: uint16,
    pub y_height: uint16,
    pub properties: uint16,
    pub extra_properties: uint16,
    pub ai_handler_bits: uint16,
    pub health: uint16,
    pub spritemap_pointer: uint16,
    pub timer: uint16,
    pub current_instruction: uint16,
    pub instruction_timer: uint16,
    pub palette_index: uint16,
    pub vram_tiles_index: uint16,
    pub layer: uint16,
    pub flash_timer: uint16,
    pub frozen_timer: uint16,
    pub invincibility_timer: uint16,
    pub shake_timer: uint16,
    pub frame_counter: uint16,
    pub bank: uint8,
    pub field_2F: uint8,
    pub ai_var_A: uint16,
    pub ai_var_B: uint16,
    pub ai_var_C: uint16,
    pub ai_var_D: uint16,
    pub ai_var_E: uint16,
    pub ai_preinstr: uint16,
    pub parameter_1: uint16,
    pub parameter_2: uint16,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct EnemyDef {
    pub tile_data_size: uint16,
    pub palette_ptr: VoidP,
    pub health: uint16,
    pub damage: uint16,
    pub x_radius: uint16,
    pub y_radius: uint16,
    pub bank: uint8,
    pub hurt_ai_time: uint8,
    pub hurt_sfx: uint16,
    pub boss_fight_value: uint16,
    pub ai_init: VoidP,
    pub num_parts: uint16,
    pub field_16: uint16,
    pub main_ai: VoidP,
    pub grapple_ai: VoidP,
    pub hurt_ai: VoidP,
    pub frozen_ai: VoidP,
    pub time_is_frozen_ai: VoidP,
    pub death_anim: uint16,
    pub field_24: uint16,
    pub field_26: uint16,
    pub powerbomb_reaction: VoidP,
    pub field_2A: VoidP,
    pub field_2C: uint16,
    pub field_2E: uint16,
    pub touch_ai: VoidP,
    pub shot_ai: VoidP,
    pub field_34: VoidP,
    pub tile_data: LongPtr,
    pub layer: uint8,
    pub item_drop_chances_ptr: VoidP,
    pub vulnerability_ptr: VoidP,
    pub name_ptr: VoidP,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct EnemySpawnData {
    pub some_flag: uint16,
    pub cause_of_death: uint16,
    pub field_4: uint16,
    pub vram_tiles_index: uint16,
    pub palette_index: uint16,
    pub field_A: uint16,
    pub field_C: uint16,
    pub field_E: uint16,
    pub xpos2: uint16,
    pub ypos2: uint16,
    pub field_14: uint16,
    pub field_16: uint16,
    pub field_18: uint16,
    pub field_1A: uint16,
    pub field_1C: uint16,
    pub id: uint16,
    pub x_pos: uint16,
    pub y_pos: uint16,
    pub init_param: uint16,
    pub properties: uint16,
    pub extra_properties: uint16,
    pub param_1: uint16,
    pub param_2: uint16,
    pub name: [uint8; 18],
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct EnemyTileset {
    pub enemy_def: VoidP,
    pub vram_dst: uint16,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_Botwoon {
    pub base: EnemyBase,
    pub botwoon_var_A: uint16,
    pub botwoon_var_B: uint16,
    pub botwoon_var_C: uint16,
    pub botwoon_var_D: uint16,
    pub botwoon_var_E: uint16,
    pub botwoon_var_F: uint16,
    pub botwoon_parameter_1: uint16,
    pub botwoon_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub botwoon_var_00: uint16,
    pub botwoon_var_01: uint16,
    pub botwoon_var_02: uint16,
    pub botwoon_var_03: uint16,
    pub botwoon_var_04: uint16,
    pub botwoon_var_05: uint16,
    pub botwoon_var_06: uint16,
    pub botwoon_var_07: uint16,
    pub botwoon_var_08: uint16,
    pub botwoon_var_09: uint16,
    pub botwoon_var_0A: uint16,
    pub botwoon_var_0B: uint16,
    pub botwoon_var_0C: uint16,
    pub botwoon_var_0D: uint16,
    pub botwoon_var_0E: uint16,
    pub botwoon_var_0F: uint16,
    pub botwoon_var_10: uint16,
    pub botwoon_var_11: uint16,
    pub botwoon_var_12: uint16,
    pub botwoon_var_13: uint16,
    pub botwoon_var_14: uint16,
    pub botwoon_var_15: uint16,
    pub botwoon_var_16: uint16,
    pub botwoon_var_17: uint16,
    pub botwoon_var_18: uint16,
    pub botwoon_var_19: uint16,
    pub botwoon_var_1A: uint16,
    pub botwoon_var_1B: uint16,
    pub botwoon_var_1C: uint16,
    pub botwoon_var_1D: uint16,
    pub botwoon_var_1E: uint16,
    pub botwoon_var_1F: uint16,
    pub pad2: [uint8; 1984],
    pub botwoon_var_20: uint16,
    pub botwoon_var_21: uint16,
    pub botwoon_var_22: uint16,
    pub botwoon_var_23: uint16,
    pub botwoon_var_24: uint16,
    pub botwoon_var_25: uint16,
    pub botwoon_var_26: uint16,
    pub botwoon_var_27: uint16,
    pub botwoon_var_28: uint16,
    pub botwoon_var_29: uint16,
    pub botwoon_var_2A: uint16,
    pub botwoon_var_2B: uint16,
    pub botwoon_var_2C: uint16,
    pub botwoon_var_2D: uint16,
    pub botwoon_var_2E: uint16,
    pub botwoon_var_2F: uint16,
    pub botwoon_var_30: uint16,
    pub botwoon_var_31: uint16,
    pub botwoon_var_32: uint16,
    pub botwoon_var_33: uint16,
    pub botwoon_var_34: uint16,
    pub botwoon_var_35: uint16,
    pub botwoon_var_36: uint16,
    pub botwoon_var_37: uint16,
    pub botwoon_var_38: uint16,
    pub botwoon_var_39: uint16,
    pub botwoon_var_3A: uint16,
    pub botwoon_var_3B: uint16,
    pub botwoon_var_3C: uint16,
    pub botwoon_var_3D: uint16,
    pub botwoon_var_3E: uint16,
    pub botwoon_var_3F: uint16,
    pub pad3: [uint8; 1984],
    pub botwoon_var_40: uint16,
    pub botwoon_var_41: uint16,
    pub botwoon_var_42: uint16,
    pub botwoon_var_43: uint16,
    pub botwoon_var_44: uint16,
    pub botwoon_var_45: uint16,
    pub botwoon_var_46: uint16,
    pub botwoon_var_47: uint16,
    pub botwoon_var_48: uint16,
    pub botwoon_var_49: uint16,
    pub botwoon_var_4A: uint16,
    pub botwoon_var_4B: uint16,
    pub botwoon_var_4C: uint16,
    pub botwoon_var_4D: uint16,
    pub botwoon_var_4E: uint16,
    pub botwoon_var_4F: uint16,
    pub botwoon_var_50: uint16,
    pub botwoon_var_51: uint16,
    pub botwoon_var_52: uint16,
    pub botwoon_var_53: uint16,
    pub botwoon_var_54: uint16,
    pub botwoon_var_55: uint16,
    pub botwoon_var_56: uint16,
    pub botwoon_var_57: uint16,
    pub botwoon_var_58: uint16,
    pub botwoon_var_59: uint16,
    pub botwoon_var_5A: uint16,
    pub botwoon_var_5B: uint16,
    pub botwoon_var_5C: uint16,
    pub botwoon_var_5D: uint16,
    pub botwoon_var_5E: uint16,
    pub botwoon_var_5F: uint16,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_MotherBrain {
    pub base: EnemyBase,
    pub mbn_var_A: uint16,
    pub mbn_var_B: uint16,
    pub mbn_var_C: uint16,
    pub mbn_var_D: uint16,
    pub mbn_var_E: uint16,
    pub mbn_var_F: uint16,
    pub mbn_parameter_1: uint16,
    pub mbn_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub mbn_var_00: uint16,
    pub mbn_var_01: uint16,
    pub mbn_var_02: uint16,
    pub mbn_var_03: uint16,
    pub mbn_var_04: uint16,
    pub mbn_var_05: uint16,
    pub mbn_var_06: uint16,
    pub mbn_var_07: uint16,
    pub mbn_var_08: uint16,
    pub mbn_var_09: uint16,
    pub mbn_var_0A: uint16,
    pub mbn_var_0B: uint16,
    pub mbn_var_0C: uint16,
    pub mbn_var_0D: uint16,
    pub mbn_var_0E: uint16,
    pub mbn_var_0F: uint16,
    pub mbn_var_10: uint16,
    pub mbn_var_11: uint16,
    pub mbn_var_12: uint16,
    pub mbn_var_13: uint16,
    pub mbn_var_14: uint16,
    pub mbn_var_15: uint16,
    pub mbn_var_16: uint16,
    pub mbn_var_17: uint16,
    pub mbn_var_18: uint16,
    pub mbn_var_19: uint16,
    pub mbn_var_1A: uint16,
    pub mbn_var_1B: uint16,
    pub mbn_var_1C: uint16,
    pub mbn_var_1D: uint16,
    pub mbn_var_1E: uint16,
    pub mbn_var_1F: uint16,
    pub pad2: [uint8; 1984],
    pub mbn_var_20: uint16,
    pub mbn_var_21: uint16,
    pub mbn_var_22: uint16,
    pub mbn_var_23: uint16,
    pub mbn_var_24: uint16,
    pub mbn_var_25: uint16,
    pub mbn_var_26: uint16,
    pub mbn_var_27: uint16,
    pub mbn_var_28: uint16,
    pub mbn_var_29: uint16,
    pub mbn_var_2A: uint16,
    pub mbn_var_2B: uint16,
    pub mbn_var_2C: uint16,
    pub mbn_var_2D: uint16,
    pub mbn_var_2E: uint16,
    pub mbn_var_2F: uint16,
    pub mbn_var_30: uint16,
    pub mbn_var_31: uint16,
    pub mbn_var_32: uint16,
    pub mbn_var_33: uint16,
    pub mbn_var_34: uint16,
    pub mbn_var_35: uint16,
    pub mbn_var_36: uint16,
    pub mbn_var_37: uint16,
    pub mbn_var_38: uint16,
    pub mbn_var_39: uint16,
    pub mbn_var_3A: uint16,
    pub mbn_var_3B: uint16,
    pub mbn_var_3C: uint16,
    pub mbn_var_3D: uint16,
    pub mbn_var_3E: uint16,
    pub mbn_var_3F: uint16,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct EprojDef {
    pub init_code_ptr: VoidP,
    pub pre_instr_ptr: VoidP,
    pub instr_list: VoidP,
    pub radius: uint16,
    pub properties: uint16,
    pub hit_instruction_list: VoidP,
    pub shot_instruction_list: VoidP,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct ExpandingSquareTransitionHdma {
    pub field_0: uint8,
    pub field_1: uint8,
    pub field_2: uint8,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct ExtendedSpriteMap {
    pub xpos: uint16,
    pub ypos: uint16,
    pub spritemap: VoidP,
    pub hitbox_ptr_: VoidP,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union ExtraEnemyRam7800 {
    pub kraid: Ram7800_Kraid,
    pub pad: [uint8; 64],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union ExtraEnemyRam8000 {
    pub pad: [uint8; 64],
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct Hitbox {
    pub left: uint16,
    pub top: uint16,
    pub right: uint16,
    pub bottom: uint16,
    pub func_ptr: VoidP,
    pub func_ptrA: VoidP,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct MapIconDataPointers {
    pub crateria: VoidP,
    pub brinstar: VoidP,
    pub norfair: VoidP,
    pub wrecked_ship: VoidP,
    pub maraidia: VoidP,
    pub tourian: VoidP,
    pub ceres: VoidP,
    pub debug: VoidP,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct MapScrollArrowData {
    pub xpos: uint16,
    pub ypos: uint16,
    pub pause_screen_animation_id: uint16,
    pub joypad_input: uint16,
    pub map_scroll_dir: uint16,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct Mode7VramWriteQueue {
    pub field_0: uint8,
    pub field_1: uint16,
    pub gap3: [uint8; 2],
    pub field_5: uint16,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct PairU16 {
    pub k: uint16,
    pub j: uint16,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Pair_Bool_Amt {
    pub flag: bool,
    pub amt: int32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Point16U {
    pub x: uint16,
    pub y: uint16,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Point32 {
    pub x: uint32,
    pub y: uint32,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct ProjectileInstr {
    pub timer: uint16,
    pub spritemap_ptr: VoidP,
    pub x_radius: uint8,
    pub y_radius: uint8,
    pub field_6: uint16,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union Ram3000 {
    pub pause_menu_map_tilemap: [uint16; 1024],
    pub cinematic_bg_tilemap: [uint16; 1024],
    pub msgbox: Ram3000_MsgBox,
    pub msgbox_y_scroll_hdma: [uint8; 240],
    pub misc: Ram3000_Misc,
    pub menu: Ram3000_Menu,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct Ram3000_Menu {
    pub field_0: [uint8; 768],
    pub palette_backup_in_menu: [uint16; 256],
    pub backup_of_io_registers_in_gameover: [uint8; 54],
    pub field_536: [uint8; 202],
    pub menu_tilemap: [uint8; 512],
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct Ram3000_Misc {
    pub field_0: [uint8; 472],
    pub pause_screen_samus_witeframe_tilemap: [uint16; 264],
    pub field_3E8: [uint8; 24],
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct Ram3000_MsgBox {
    pub pad: [uint8; 188],
    pub msg_box_anim_y_radius_neg: [uint16; 30],
    pub msg_box_anim_y_radius: [uint16; 30],
    pub field_134: [uint8; 204],
    pub tilemap: [uint16; 192],
    pub indirect_hdma: [uint8; 7],
    pub field_387: [uint8; 99],
    pub backup_of_enabled_hdma_channels: uint8,
    pub backup_of_bg3_tilemap_and_size: uint16,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union Ram3800 {
    pub cinematic_bg_tilemap: [uint16; 1024],
    pub equipment_screen_bg1_tilemap: [uint16; 1024],
    pub debug_game_over_tilemap: [uint16; 1024],
    pub cleared_message_box_bg3_tilemap: [uint16; 1024],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union Ram4000 {
    pub xray_tilemaps: [uint16; 6144],
    pub bg2_tilemap: [uint16; 2048],
    pub decomp_buffer_kraid: [uint16; 1024],
    pub bg2_room_select_map_tilemap: [uint16; 1024],
    pub intro_japanese_text_tiles: [uint16; 768],
    pub backups: Ram4000_Backups,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct Ram4000_Backups {
    pub field_0: [uint8; 256],
    pub backup_of_vram_0x5880_msgbox: [uint16; 896],
    pub field_800: [uint8; 2048],
    pub backup_of_0x3e00_in_kraid_pause: [uint8; 1024],
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct Ram7800_Kraid {
    pub kraid_next: uint16,
    pub field_2: uint16,
    pub field_4: uint16,
    pub kraid_thinking: uint16,
    pub kraid_min_y_pos_eject: uint16,
    pub kraid_mouth_flags: uint16,
    pub kraid_healths_8ths: [uint16; 8],
    pub field_1C: uint16,
    pub kraid_target_x: uint16,
    pub kraid_healths_4ths: [uint16; 4],
    pub field_28: uint16,
    pub kraid_hurt_frame: uint16,
    pub kraid_hurt_frame_timer: uint16,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Ram8000_Default {
    pub var_20: uint16,
    pub var_21: uint16,
    pub var_22: uint16,
    pub var_23: uint16,
    pub var_24: uint16,
    pub var_25: uint16,
    pub var_26: uint16,
    pub var_27: uint16,
    pub var_28: uint16,
    pub var_29: uint16,
    pub var_2A: uint16,
    pub var_2B: uint16,
    pub var_2C: uint16,
    pub var_2D: uint16,
    pub var_2E: uint16,
    pub var_2F: uint16,
    pub var_30: uint16,
    pub var_31: uint16,
    pub var_32: uint16,
    pub var_33: uint16,
    pub var_34: uint16,
    pub var_35: uint16,
    pub var_36: uint16,
    pub var_37: uint16,
    pub var_38: uint16,
    pub var_39: uint16,
    pub var_3A: uint16,
    pub var_3B: uint16,
    pub var_3C: uint16,
    pub var_3D: uint16,
    pub var_3E: uint16,
    pub var_3F: uint16,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Rect16U {
    pub x: uint16,
    pub y: uint16,
    pub w: uint16,
    pub h: uint16,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct RoomDefHeader {
    pub semiunique_room_number: uint8,
    pub area_index_: uint8,
    pub x_coordinate_on_map: uint8,
    pub y_coordinate_on_map: uint8,
    pub width: uint8,
    pub height: uint8,
    pub up_scroller_: uint8,
    pub down_scroller_: uint8,
    pub cre_bitset_: uint8,
    pub ptr_to_doorout: VoidP,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct RoomDefRoomstate {
    pub compressed_room_map_ptr: LongPtr,
    pub graphics_set: uint8,
    pub music_track: uint8,
    pub music_control: uint8,
    pub room_layer3_fx_ptr: VoidP,
    pub enemy_population_ptr_: VoidP,
    pub enemy_tilesets_ptr: VoidP,
    pub vertical_screen_nudge_limit: uint16,
    pub rdf_scroll_ptr: VoidP,
    pub xray_special_casing_ptr: VoidP,
    pub main_code_ptr: VoidP,
    pub room_plm_header_ptr: VoidP,
    pub bg_data_ptr: VoidP,
    pub room_setup_code: VoidP,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct RoomPlmEntry {
    pub plm_header_ptr_: VoidP,
    pub x_block: uint8,
    pub y_block: uint8,
    pub plm_room_argument: uint16,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct SamusPoseParams {
    pub pose_x_dir: uint8,
    pub movement_type: uint8,
    pub new_pose_unless_buttons: uint8,
    pub direction_shots_fired: uint8,
    pub y_offset_to_gfx: uint8,
    pub field_5: uint8,
    pub y_radius: uint8,
    pub field_7: uint8,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct SpawnHardcodedPlmArgs {
    pub field_0: uint8,
    pub field_1: uint8,
    pub field_2: VoidP,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct SpawnHdmaObject_Args {
    pub field_0: uint8,
    pub field_1: uint8,
    pub addr: VoidP,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct StartDmaCopy {
    pub chan: uint8,
    pub dmap: uint8,
    pub bbad: uint8,
    pub a1: LongPtr,
    pub das: VoidP,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct TileTable {
    pub top_left: uint16,
    pub top_right: uint16,
    pub bottom_left: uint16,
    pub bottom_right: uint16,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct TileTables {
    pub tables: [TileTable; 1024],
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct VramReadQueueEnt {
    pub vram_target: uint16,
    pub dma_parameters: uint16,
    pub src: LongPtr,
    pub size: uint16,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct Vulnerability {
    pub power: uint8,
    pub wave: uint8,
    pub ice: uint8,
    pub ice_wave: uint8,
    pub spazer: uint8,
    pub spazer_wave: uint8,
    pub spazer_ice: uint8,
    pub spazer_ice_wave: uint8,
    pub plasma: uint8,
    pub plasma_wave: uint8,
    pub plasma_ice: uint8,
    pub plasma_ice_wave: uint8,
    pub missile: uint8,
    pub super_missile: uint8,
    pub bomb: uint8,
    pub power_bomb: uint8,
    pub speed_booster: uint8,
    pub shinespark: uint8,
    pub screw_attack: uint8,
    pub charged_beam: uint8,
    pub pseudo_screw_attack: uint8,
    pub field_15: uint8,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Apu {
    pub spc: *mut Spc,
    pub dsp: *mut Dsp,
    pub ram: [uint8_t; 65536],
    pub romReadable: bool,
    pub dspAdr: uint8_t,
    pub cycles: uint32_t,
    pub inPorts: [uint8_t; 6],
    pub outPorts: [uint8_t; 4],
    pub timer: [Timer; 3],
    pub cpuCyclesLeft: uint8_t,
    pub pad: [uint8_t; 6],
    pub c2rust_unnamed: ApuHistory,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union ApuHistory {
    pub hist: DspRegWriteHistory,
    pub padpad: *mut ::core::ffi::c_void,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct BgLayer {
    pub hScroll: uint16_t,
    pub vScroll: uint16_t,
    pub tilemapWider: bool,
    pub tilemapHigher: bool,
    pub tilemapAdr: uint16_t,
    pub tileAdr: uint16_t,
    pub bigTiles: bool,
    pub mosaicEnabled: bool,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Cart {
    pub snes: *mut Snes,
    pub type_0: uint8_t,
    pub rom: *mut uint8_t,
    pub romSize: uint32_t,
    pub ram: *mut uint8_t,
    pub ramSize: uint32_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Channel {
    pub pattern_order_ptr_for_chan: uint16,
    pub index: uint8,
    pub note_ticks_left: uint8,
    pub note_keyoff_ticks_left: uint8,
    pub subroutine_num_loops: uint8,
    pub volume_fade_ticks: uint8,
    pub pan_num_ticks: uint8,
    pub pitch_slide_length: uint8,
    pub pitch_slide_delay_left: uint8,
    pub vibrato_hold_count: uint8,
    pub vib_depth: uint8,
    pub tremolo_hold_count: uint8,
    pub tremolo_depth: uint8,
    pub vibrato_change_count: uint8,
    pub note_length: uint8,
    pub note_gate_off_fixedpt: uint8,
    pub channel_volume_master: uint8,
    pub instrument_id: uint8,
    pub instrument_pitch_base: uint16,
    pub saved_pattern_ptr: uint16,
    pub pattern_start_ptr: uint16,
    pub pitch_envelope_num_ticks: uint8,
    pub pitch_envelope_delay: uint8,
    pub pitch_envelope_direction: uint8,
    pub pitch_envelope_slide_value: uint8,
    pub vibrato_count: uint8,
    pub vibrato_rate: uint8,
    pub vibrato_delay_ticks: uint8,
    pub vibrato_fade_num_ticks: uint8,
    pub vibrato_fade_add_per_tick: uint8,
    pub vibrato_depth_target: uint8,
    pub tremolo_count: uint8,
    pub tremolo_rate: uint8,
    pub tremolo_delay_ticks: uint8,
    pub channel_transposition: uint8,
    pub channel_volume: uint16,
    pub volume_fade_addpertick: uint16,
    pub volume_fade_target: uint8,
    pub final_volume: uint8,
    pub pan_value: uint16,
    pub pan_add_per_tick: uint16,
    pub pan_target_value: uint8,
    pub pan_flag_with_phase_invert: uint8,
    pub pitch: uint16,
    pub pitch_add_per_tick: uint16,
    pub pitch_target: uint8,
    pub fine_tune: uint8,
    pub cutk: uint8,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Cpu {
    pub mem: *mut ::core::ffi::c_void,
    pub memType: uint8_t,
    pub a: uint16_t,
    pub x: uint16_t,
    pub y: uint16_t,
    pub sp: uint16_t,
    pub pc: uint16_t,
    pub dp: uint16_t,
    pub k: uint8_t,
    pub db: uint8_t,
    pub c: bool,
    pub z: bool,
    pub v: bool,
    pub n: bool,
    pub i: bool,
    pub d: bool,
    pub xf: bool,
    pub mf: bool,
    pub e: bool,
    pub irqWanted: bool,
    pub nmiWanted: bool,
    pub waiting: bool,
    pub stopped: bool,
    pub cyclesUsed: uint8_t,
    pub spBreakpoint: uint16_t,
    pub in_emu: bool,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Dma {
    pub snes: *mut Snes,
    pub channel: [DmaChannel; 8],
    pub hdmaTimer: uint16_t,
    pub dmaTimer: uint32_t,
    pub dmaBusy: bool,
    pub pad: [uint8_t; 7],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct DmaChannel {
    pub bAdr: uint8_t,
    pub aAdr: uint16_t,
    pub aBank: uint8_t,
    pub size: uint16_t,
    pub indBank: uint8_t,
    pub tableAdr: uint16_t,
    pub repCount: uint8_t,
    pub unusedByte: uint8_t,
    pub dmaActive: bool,
    pub hdmaActive: bool,
    pub mode: uint8_t,
    pub fixed: bool,
    pub decrement: bool,
    pub indirect: bool,
    pub fromB: bool,
    pub unusedBit: bool,
    pub doTransfer: bool,
    pub terminated: bool,
    pub offIndex: uint8_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Dsp {
    pub apu_ram: *mut uint8_t,
    pub ram: [uint8_t; 128],
    pub channel: [DspChannel; 8],
    pub dirPage: uint16_t,
    pub evenCycle: bool,
    pub mute: bool,
    pub reset: bool,
    pub masterVolumeL: int8_t,
    pub masterVolumeR: int8_t,
    pub noiseSample: int16_t,
    pub noiseRate: uint16_t,
    pub noiseCounter: uint16_t,
    pub echoWrites: bool,
    pub echoVolumeL: int8_t,
    pub echoVolumeR: int8_t,
    pub feedbackVolume: int8_t,
    pub echoBufferAdr: uint16_t,
    pub echoDelay: uint16_t,
    pub echoRemain: uint16_t,
    pub echoBufferIndex: uint16_t,
    pub firBufferIndex: uint8_t,
    pub firValues: [int8_t; 8],
    pub firBufferL: [int16_t; 8],
    pub firBufferR: [int16_t; 8],
    pub sampleBuffer: [int16_t; 1068],
    pub sampleOffset: uint16_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct DspChannel {
    pub pitch: uint16_t,
    pub pitchCounter: uint16_t,
    pub pitchModulation: bool,
    pub decodeBuffer: [int16_t; 19],
    pub srcn: uint8_t,
    pub decodeOffset: uint16_t,
    pub previousFlags: uint8_t,
    pub old: int16_t,
    pub older: int16_t,
    pub useNoise: bool,
    pub adsrRates: [uint16_t; 4],
    pub rateCounter: uint16_t,
    pub adsrState: uint8_t,
    pub sustainLevel: uint16_t,
    pub useGain: bool,
    pub gainMode: uint8_t,
    pub directGain: bool,
    pub gainValue: uint16_t,
    pub gain: uint16_t,
    pub keyOn: bool,
    pub keyOff: bool,
    pub sampleOut: int16_t,
    pub volumeL: int8_t,
    pub volumeR: int8_t,
    pub echoEnable: bool,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct DspRegWriteHistory {
    pub count: uint32_t,
    pub addr: [uint8_t; 256],
    pub val: [uint8_t; 256],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Input {
    pub snes: *mut Snes,
    pub type_0: uint8_t,
    pub currentState: uint16_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Layer {
    pub mainScreenEnabled: bool,
    pub subScreenEnabled: bool,
    pub mainScreenWindowed: bool,
    pub subScreenWindowed: bool,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Ppu {
    pub snes: *mut Snes,
    pub vram: [uint16_t; 32768],
    pub vramPointer: uint16_t,
    pub vramIncrementOnHigh: bool,
    pub vramIncrement: uint16_t,
    pub vramRemapMode: uint8_t,
    pub vramReadBuffer: uint16_t,
    pub cgram: [uint16_t; 256],
    pub cgramPointer: uint8_t,
    pub cgramSecondWrite: bool,
    pub cgramBuffer: uint8_t,
    pub oam: [uint16_t; 256],
    pub highOam: [uint8_t; 32],
    pub oamAdr: uint8_t,
    pub oamAdrWritten: uint8_t,
    pub oamInHigh: bool,
    pub oamInHighWritten: bool,
    pub oamSecondWrite: bool,
    pub oamBuffer: uint8_t,
    pub objPriority: bool,
    pub objTileAdr1: uint16_t,
    pub objTileAdr2: uint16_t,
    pub objSize: uint8_t,
    pub objPixelBufferXX: [uint8_t; 256],
    pub objPriorityBufferXX: [uint8_t; 256],
    pub timeOver: bool,
    pub rangeOver: bool,
    pub objInterlace: bool,
    pub bgLayer: [BgLayer; 4],
    pub scrollPrev: uint8_t,
    pub scrollPrev2: uint8_t,
    pub mosaicSize: uint8_t,
    pub mosaicStartLine: uint8_t,
    pub layer: [Layer; 5],
    pub m7matrix: [int16_t; 8],
    pub m7prev: uint8_t,
    pub m7largeField: bool,
    pub m7charFill: bool,
    pub m7xFlip: bool,
    pub m7yFlip: bool,
    pub m7extBg: bool,
    pub m7startX: int32_t,
    pub m7startY: int32_t,
    pub windowLayer: [WindowLayer; 6],
    pub window1left: uint8_t,
    pub window1right: uint8_t,
    pub window2left: uint8_t,
    pub window2right: uint8_t,
    pub clipMode: uint8_t,
    pub preventMathMode: uint8_t,
    pub addSubscreen: bool,
    pub subtractColor: bool,
    pub halfColor: bool,
    pub mathEnabled: [bool; 6],
    pub fixedColorR: uint8_t,
    pub fixedColorG: uint8_t,
    pub fixedColorB: uint8_t,
    pub forcedBlank: bool,
    pub brightness: uint8_t,
    pub mode: uint8_t,
    pub bg3priority: bool,
    pub evenFrame: bool,
    pub pseudoHires: bool,
    pub overscan: bool,
    pub frameOverscan: bool,
    pub interlace: bool,
    pub frameInterlace: bool,
    pub directColor: bool,
    pub hCount: uint16_t,
    pub vCount: uint16_t,
    pub hCountSecond: bool,
    pub vCountSecond: bool,
    pub countersLatched: bool,
    pub ppu1openBus: uint8_t,
    pub ppu2openBus: uint8_t,
    pub pixelbuffer_placeholder: uint8_t,
    pub windowsel: uint32_t,
    pub extraLeftCur: uint8_t,
    pub extraRightCur: uint8_t,
    pub extraLeftRight: uint8_t,
    pub screenEnabled: [uint8_t; 2],
    pub screenWindowed: [uint8_t; 2],
    pub mosaicEnabled: uint8_t,
    pub lastBrightnessMult: uint8_t,
    pub lineHasSprites: bool,
    pub bgBuffers: [PpuPixelPrioBufs; 2],
    pub objBuffer: PpuPixelPrioBufs,
    pub renderPitch: uint32_t,
    pub renderBuffer: *mut uint8_t,
    pub brightnessMult: [uint8_t; 63],
    pub brightnessMultHalf: [uint8_t; 64],
    pub mosaicModulo: [uint8_t; 256],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct PpuPixelPrioBufs {
    pub data: [PpuZbufType; 256],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct SfxChan {
    pub voice_bitset: uint8,
    pub instr_timer: uint8,
    pub subnote: uint8,
    pub repeat_pt: uint8,
    pub note: uint8,
    pub ptr_idx: uint8,
    pub voice_index: uint8,
    pub voice_index_mult8: uint8,
    pub release_timer: uint8,
    pub volume: uint8,
    pub target_note: uint8,
    pub phase_invert: uint8,
    pub ptr: uint16,
    pub repeat_ctr: uint8,
    pub enable_pitch_slide: uint8,
    pub release_flag: uint8,
    pub enable_pitch_slide_legato: uint8,
    pub adsr_settings: uint16,
    pub channel_mask: uint8,
    pub legato: uint8,
    pub disable: uint8,
    pub subnote_delta: uint8,
    pub update_adsr: uint8,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct SfxRegs {
    pub priority: uint8,
    pub chan_idx_x2: uint8,
    pub chan_voice_bitset_ptr: uint16,
    pub cur_sound: uint8,
    pub chan_voice_mask_ptr: uint16,
    pub chan_voice_index_ptr: uint16,
    pub some_value: uint8,
    pub cur_ptr: uint16,
    pub enabled_voices: uint8,
    pub voices_to_setup: uint8,
    pub chan_idx: uint8,
    pub voices_remaining: uint8,
    pub cur_sound_index: uint8,
    pub voice_id: uint8,
    pub init_flag: uint8,
    pub voice_index: uint8,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Snes {
    pub cpu: *mut Cpu,
    pub apu: *mut Apu,
    pub ppu: *mut Ppu,
    pub snes_ppu: *mut Ppu,
    pub my_ppu: *mut Ppu,
    pub dma: *mut Dma,
    pub cart: *mut Cart,
    pub input1: *mut Input,
    pub input2: *mut Input,
    pub debug_cycles: bool,
    pub debug_apu_cycles: bool,
    pub disableRender: bool,
    pub runningWhichVersion: uint8_t,
    pub ramAdr: uint32_t,
    pub ram: *mut uint8_t,
    pub padx: [uint8_t; 4],
    pub hPos: uint16_t,
    pub vPos: uint16_t,
    pub frames: uint32_t,
    pub cpuCyclesLeft: uint8_t,
    pub cpuMemOps: uint8_t,
    pub padpad: [uint8_t; 2],
    pub apuCatchupCycles: ::core::ffi::c_double,
    pub hIrqEnabled: bool,
    pub vIrqEnabled: bool,
    pub nmiEnabled: bool,
    pub hTimer: uint16_t,
    pub vTimer: uint16_t,
    pub inNmi: bool,
    pub inIrq: bool,
    pub inVblank: bool,
    pub portAutoReadX: [uint16_t; 4],
    pub autoJoyRead: bool,
    pub autoJoyTimer: uint16_t,
    pub ppuLatch: bool,
    pub multiplyA: uint8_t,
    pub multiplyResult: uint16_t,
    pub divideA: uint16_t,
    pub divideResult: uint16_t,
    pub fastMem: bool,
    pub openBus: uint8_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Spc {
    pub apu: *mut Apu,
    pub a: uint8_t,
    pub x: uint8_t,
    pub y: uint8_t,
    pub sp: uint8_t,
    pub pc: uint16_t,
    pub c: bool,
    pub z: bool,
    pub v: bool,
    pub n: bool,
    pub i: bool,
    pub h: bool,
    pub p: bool,
    pub b: bool,
    pub stopped: bool,
    pub cyclesUsed: uint8_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct SpcPlayer {
    pub reg_write_history: *mut DspRegWriteHistory,
    pub timer_cycles: uint8,
    pub dsp: *mut Dsp,
    pub port_to_snes: [uint8; 4],
    pub last_value_from_snes: [uint8; 4],
    pub counter_sf0c: uint16,
    pub always_zero: uint16,
    pub temp_accum: uint16,
    pub ttt: uint8,
    pub did_affect_volumepitch_flag: uint8,
    pub addr0: uint16,
    pub addr1: uint16,
    pub lfsr_value: uint16,
    pub is_chan_on: uint8,
    pub fast_forward: uint8,
    pub word_20: uint16,
    pub music_ptr_toplevel: uint16,
    pub block_count: uint8,
    pub sfx_timer_accum: uint8,
    pub chn: uint8,
    pub key_ON: uint8,
    pub key_OFF: uint8,
    pub cur_chan_bit: uint8,
    pub reg_FLG: uint8,
    pub reg_NON: uint8,
    pub reg_EON: uint8,
    pub echo_stored_time: uint8,
    pub echo_parameter_EDL: uint8,
    pub reg_EFB: uint8,
    pub global_transposition: uint8,
    pub main_tempo_accum: uint8,
    pub tempo: uint16,
    pub tempo_fade_num_ticks: uint8,
    pub tempo_fade_final: uint8,
    pub tempo_fade_add: uint16,
    pub master_volume: uint16,
    pub master_volume_fade_ticks: uint8,
    pub master_volume_fade_target: uint8,
    pub master_volume_fade_add_per_tick: uint16,
    pub vol_dirty: uint8,
    pub percussion_base_id: uint8,
    pub echo_volume_left: uint16,
    pub echo_volume_right: uint16,
    pub echo_volume_fade_add_left: uint16,
    pub echo_volume_fade_add_right: uint16,
    pub echo_volume_fade_ticks: uint8,
    pub echo_volume_fade_target_left: uint8,
    pub echo_volume_fade_target_right: uint8,
    pub p_echoBuffer: uint16,
    pub memset_ptr: uint16,
    pub memset_ctr: uint8,
    pub disable_sfx2: uint8,
    pub byte_4B1: uint8,
    pub channel: [Channel; 8],
    pub sfx1: SfxRegs,
    pub sfx2: SfxRegs,
    pub sfx3: SfxRegs,
    pub sfx_chans_1: [SfxChan; 4],
    pub sfx_chans_2: [SfxChan; 2],
    pub sfx_chans_3: [SfxChan; 2],
    pub last_written_edl: uint8,
    pub input_ports: [uint8; 4],
    pub ram: [uint8; 65536],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Timer {
    pub cycles: uint8_t,
    pub divider: uint8_t,
    pub target: uint8_t,
    pub counter: uint8_t,
    pub enabled: bool,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct WindowLayer {
    pub window1enabled: bool,
    pub window2enabled: bool,
    pub window1inversed: bool,
    pub window2inversed: bool,
    pub maskLogic: uint8_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ByteArray {
    pub data: *mut uint8,
    pub size: size_t,
    pub capacity: size_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Config {
    pub window_width: ::core::ffi::c_int,
    pub window_height: ::core::ffi::c_int,
    pub enhanced_mode7: bool,
    pub new_renderer: bool,
    pub ignore_aspect_ratio: bool,
    pub fullscreen: uint8,
    pub window_scale: uint8,
    pub enable_audio: bool,
    pub linear_filtering: bool,
    pub output_method: uint8,
    pub audio_freq: uint16,
    pub audio_channels: uint8,
    pub audio_samples: uint16,
    pub autosave: bool,
    pub extended_aspect_ratio: uint8,
    pub extend_y: bool,
    pub no_sprite_limits: bool,
    pub display_perf_title: bool,
    pub enable_msu: uint8,
    pub resume_msu: bool,
    pub disable_frame_delay: bool,
    pub msuvolume: uint8,
    pub features0: uint32,
    pub link_graphics: *const ::core::ffi::c_char,
    pub memory_buffer: *mut ::core::ffi::c_char,
    pub shader: *const ::core::ffi::c_char,
    pub msu_path: *const ::core::ffi::c_char,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct GlTextureWithSize {
    pub gl_texture: uint,
    pub width: uint16,
    pub height: uint16,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct GlslParam {
    pub next: *mut GlslParam,
    pub id: *mut ::core::ffi::c_char,
    pub has_value: bool,
    pub value: ::core::ffi::c_float,
    pub min: ::core::ffi::c_float,
    pub max: ::core::ffi::c_float,
    pub uniform: [uint; 20],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct GlslPass {
    pub filename: *mut ::core::ffi::c_char,
    pub scale_type_x: uint8,
    pub scale_type_y: uint8,
    pub float_framebuffer: bool,
    pub srgb_framebuffer: bool,
    pub mipmap_input: bool,
    pub scale_x: ::core::ffi::c_float,
    pub scale_y: ::core::ffi::c_float,
    pub wrap_mode: uint,
    pub frame_count_mod: uint,
    pub frame_count: uint,
    pub gl_program: uint,
    pub gl_fbo: uint,
    pub filter: uint,
    pub gl_texture: uint,
    pub width: uint16,
    pub height: uint16,
    pub unif: GlslUniforms,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct GlslShader {
    pub n_pass: ::core::ffi::c_int,
    pub pass: *mut GlslPass,
    pub first_param: *mut GlslParam,
    pub first_texture: *mut GlslTexture,
    pub gl_vao: *mut uint,
    pub gl_vbo: uint,
    pub frame_count: uint,
    pub max_prev_frame: ::core::ffi::c_int,
    pub prev_frame: [GlTextureWithSize; 8],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct GlslTexture {
    pub next: *mut GlslTexture,
    pub id: *mut ::core::ffi::c_char,
    pub filename: *mut ::core::ffi::c_char,
    pub filter: uint,
    pub gl_texture: uint,
    pub wrap_mode: uint,
    pub mipmap: bool,
    pub width: ::core::ffi::c_int,
    pub height: ::core::ffi::c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct GlslTextureUniform {
    pub Texture: ::core::ffi::c_int,
    pub InputSize: ::core::ffi::c_int,
    pub TextureSize: ::core::ffi::c_int,
    pub TexCoord: ::core::ffi::c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct GlslUniforms {
    pub Top: GlslTextureUniform,
    pub OutputSize: ::core::ffi::c_int,
    pub FrameCount: ::core::ffi::c_int,
    pub FrameDirection: ::core::ffi::c_int,
    pub LUTTexCoord: ::core::ffi::c_int,
    pub VertexCoord: ::core::ffi::c_int,
    pub Orig: GlslTextureUniform,
    pub Prev: [GlslTextureUniform; 7],
    pub Pass: [GlslTextureUniform; 20],
    pub PassPrev: [GlslTextureUniform; 20],
    pub Texture: [::core::ffi::c_int; 10],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct RendererFuncs {
    pub Initialize: Option<unsafe fn(*mut SDL_Window, &mut Game) -> bool>,
    pub Destroy: Option<unsafe fn(&mut Game) -> ()>,
    pub BeginDraw: Option<
        unsafe fn(
            ::core::ffi::c_int,
            ::core::ffi::c_int,
            *mut *mut uint8,
            *mut ::core::ffi::c_int,
            &mut Game,
        ) -> (),
    >,
    pub EndDraw: Option<unsafe fn(&mut Game) -> ()>,
}

#[repr(C)]
pub struct _IO_FILE {
    _private: [u8; 0],
}

pub type FILE = _IO_FILE;

pub type KeyAction = ::core::ffi::c_uint;

pub type OutputMethod = ::core::ffi::c_uint;

pub type GamepadButton = ::core::ffi::c_int;

pub type SaveLoadOperation = ::core::ffi::c_uint;

#[cfg(all(feature = "frontend", not(sm_verify_dll)))]
pub use sdl2_sys::SDL_Window;

#[cfg(any(not(feature = "frontend"), sm_verify_dll))]
#[repr(C)]
pub struct SDL_Window {
    _private: [u8; 0],
}

#[repr(C)]
pub struct _IO_codecvt {
    _private: [u8; 0],
}

#[repr(C)]
pub struct _IO_marker {
    _private: [u8; 0],
}

#[repr(C)]
pub struct _IO_wide_data {
    _private: [u8; 0],
}
