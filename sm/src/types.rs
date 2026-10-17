//! Shared byte-compatible gameplay types.

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

pub type CalculateAngleFromXYFunc = unsafe extern "C" fn(uint16) -> uint8;

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

pub type CrocomireFunc = unsafe extern "C" fn(uint16, *const uint16) -> *const uint16;

pub type DspReg = ::core::ffi::c_uint;

pub type EnemyAiBits = ::core::ffi::c_uint;

pub type EnemyProps = ::core::ffi::c_uint;

pub type FuncXY_V = unsafe extern "C" fn(uint16, uint16) -> ();

pub type Func_AnimDelay = unsafe extern "C" fn(*const uint8) -> bool;

pub type Func_CalcHdma = unsafe extern "C" fn(uint16, uint16) -> ();

pub type Func_SpazerSba_V = unsafe extern "C" fn(uint16, uint16) -> ();

pub type Func_U8 = unsafe extern "C" fn() -> uint8;

pub type Func_V = unsafe extern "C" fn() -> ();

pub type Func_V_A = unsafe extern "C" fn() -> uint8;

pub type Func_V_Coroutine = unsafe extern "C" fn() -> CoroutineRet;

pub type Func_Y_V = unsafe extern "C" fn(uint16) -> ();

pub type Func_Y_Y = unsafe extern "C" fn(uint16) -> uint16;

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

pub type HandlerFunc = unsafe extern "C" fn() -> ();

pub type LiquidPhysicsType = ::core::ffi::c_uint;

pub type LoadingGameState = ::core::ffi::c_uint;

pub type PFN_LOADFUNCPOINTERS = Option<unsafe extern "C" fn() -> ::core::ffi::c_int>;

pub type PpuZbufType = uint16_t;

pub type ProjectileType = ::core::ffi::c_uint;

pub type RunFrameFunc = unsafe extern "C" fn(uint16, ::core::ffi::c_int) -> ();

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
    unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut ::core::ffi::c_void, size_t) -> ();

pub type Sint16 = int16_t;

pub type Sint32 = int32_t;

pub type Sint64 = int64_t;

pub type SnesRegs = ::core::ffi::c_uint;

pub type SyncAllFunc = unsafe extern "C" fn() -> ();

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
