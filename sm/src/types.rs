//! Shared byte-compatible gameplay, hardware and music types.

use crate::game::enemy_layouts::enemy_layout;
use crate::game_state::Game;

pub type VoidP = u16;

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct LongPtr {
    pub addr: VoidP,
    pub bank: u8,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct OamEnt {
    pub xcoord: u8,
    pub ycoord: u8,
    pub charnum: u8,
    pub flags: u8,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct VramWriteEntry {
    pub size: u16,
    pub src: LongPtr,
    pub vram_dst: u16,
}

pub type Buttons = u32;

pub type CalculateAngleFromXYFunc = unsafe fn(u16, &mut Game) -> u8;

pub type Consts_60 = u32;

pub type Consts_80 = u32;

pub type Consts_81 = u32;

pub type Consts_82 = u32;

pub type Consts_83 = u32;

pub type Consts_84 = u32;

pub type Consts_86 = u32;

pub type Consts_87 = u32;

pub type Consts_88 = u32;

pub type Consts_89 = u32;

pub type Consts_8B = u32;

pub type Consts_8C = u32;

pub type Consts_8D = u32;

pub type Consts_8F = u32;

pub type Consts_90 = u32;

pub type Consts_91 = u32;

pub type Consts_94 = u32;

pub type Consts_9B = u32;

pub type Consts_A0 = u32;

pub type Consts_A2 = u32;

pub type Consts_A3 = u32;

pub type Consts_A4 = u32;

pub type Consts_A5 = u32;

pub type Consts_A6 = u32;

pub type Consts_A7 = u32;

pub type Consts_A8 = u32;

pub type Consts_A9 = u32;

pub type Consts_AA = u32;

pub type Consts_AD = u32;

pub type Consts_B2 = u32;

pub type Consts_B3 = u32;

pub type Consts_B4 = u32;

pub type Consts_B7 = u32;

pub type CoroutineRet = u8;

pub type CrocomireFunc = unsafe fn(u16, *const u16, &mut Game) -> *const u16;

pub type DspReg = u32;

pub type EnemyAiBits = u32;

pub type EnemyProps = u32;

pub type FuncXY_V = unsafe fn(u16, u16, &mut Game) -> ();

pub type Func_CalcHdma = unsafe fn(u16, u16, &mut Game) -> ();

pub type Func_SpazerSba_V = unsafe fn(u16, u16, &mut Game) -> ();

pub type Func_U8 = unsafe fn(&mut Game) -> u8;

pub type Func_V = unsafe fn(&mut Game) -> ();

pub type Func_V_A = unsafe fn(&mut Game) -> u8;

pub type Func_V_Coroutine = unsafe fn(&mut Game) -> CoroutineRet;

pub type Func_Y_V = unsafe fn(u16, &mut Game) -> ();

pub type Func_Y_Y = unsafe fn(u16, &mut Game) -> u16;

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

pub type GLSLScaleType = u32;

pub type GLbitfield = u32;

pub type GLboolean = ::core::ffi::c_uchar;

pub type GLbyte = ::core::ffi::c_schar;

pub type GLchar = ::core::ffi::c_char;

pub type GLdouble = ::core::ffi::c_double;

pub type GLenum = u32;

pub type GLfloat = ::core::ffi::c_float;

pub type GLint = i32;

pub type GLint64 = i64;

pub type GLintptr = ptrdiff_t;

pub type GLshort = ::core::ffi::c_short;

pub type GLsizei = i32;

pub type GLsizeiptr = ptrdiff_t;

pub type GLubyte = ::core::ffi::c_uchar;

pub type GLuint = u32;

pub type GLushort = ::core::ffi::c_ushort;

pub type GameState = u32;

pub type HandlerFunc = unsafe fn(&mut Game) -> ();

pub type LiquidPhysicsType = u32;

pub type LoadingGameState = u32;

pub type PFN_LOADFUNCPOINTERS = Option<unsafe fn() -> i32>;

pub type PpuZbufType = u16;

pub type ProjectileType = u32;

pub type RunFrameFunc = unsafe fn(u16, i32, &mut Game) -> ();

pub type RunMode = u32;

pub type SDL_AudioCallback =
    Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut Uint8, i32) -> ()>;

pub type SDL_AudioDeviceID = Uint32;

pub type SDL_AudioFormat = Uint16;

pub type SDL_FingerID = Sint64;

pub type SDL_GLContext = *mut ::core::ffi::c_void;

pub type SDL_GLattr = u32;

pub type SDL_GestureID = Sint64;

pub type SDL_HitTestResult = u32;

pub type SDL_JoystickID = Sint32;

pub type SDL_JoystickPowerLevel = i32;

pub type SDL_Keycode = Sint32;

pub type SDL_Keymod = u32;

pub type SDL_Scancode = u32;

pub type SDL_TouchID = Sint64;

pub type SDL_bool = u32;

pub type SamusMovementType = u32;

pub type SamusPose = u32;

pub type SaveLoadFunc =
    unsafe fn(*mut ::core::ffi::c_void, *mut ::core::ffi::c_void, size_t, &mut Game) -> ();

pub type Sint16 = i16;

pub type Sint32 = i32;

pub type Sint64 = i64;

pub type SnesRegs = u32;

pub type SyncAllFunc = unsafe fn(&mut Game) -> ();

pub type Uint16 = u16;

pub type Uint32 = u32;

pub type Uint64 = u64;

pub type Uint8 = u8;

pub type _IO_lock_t = ();

pub type __mode_t = u32;

pub type __off64_t = ::core::ffi::c_long;

pub type __off_t = ::core::ffi::c_long;

pub type __time_t = ::core::ffi::c_long;

pub type ogl_LoadStatus = u32;

pub type ptrdiff_t = isize;

pub type size_t = usize;

pub type stbi__uint16 = u16;

pub type stbi__uint32 = u32;

pub type stbi_uc = ::core::ffi::c_uchar;

pub type stbi_us = ::core::ffi::c_ushort;

pub type time_t = __time_t;

pub type CheckEnemyColl_Result = CheckSolidEnemyColl_Result;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CheckSolidEnemyColl_Result {
    pub collision: u16,
    pub amt: i32,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct DoorDef {
    pub room_definition_ptr: VoidP,
    pub door_bitflags: u8,
    pub door_orientation: u8,
    pub x_pos_plm: u8,
    pub y_pos_plm: u8,
    pub x_pos_in_room: u8,
    pub y_pos_in_room: u8,
    pub samus_distance_from_door: u16,
    pub door_setup_code: VoidP,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct EnemyBase {
    pub enemy_ptr: u16,
    pub x_pos: u16,
    pub x_subpos: u16,
    pub y_pos: u16,
    pub y_subpos: u16,
    pub x_width: u16,
    pub y_height: u16,
    pub properties: u16,
    pub extra_properties: u16,
    pub ai_handler_bits: u16,
    pub health: u16,
    pub spritemap_pointer: u16,
    pub timer: u16,
    pub current_instruction: u16,
    pub instruction_timer: u16,
    pub palette_index: u16,
    pub vram_tiles_index: u16,
    pub layer: u16,
    pub flash_timer: u16,
    pub frozen_timer: u16,
    pub invincibility_timer: u16,
    pub shake_timer: u16,
    pub frame_counter: u16,
    pub bank: u8,
    pub field_2F: u8,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct EnemyData {
    pub enemy_ptr: u16,
    pub x_pos: u16,
    pub x_subpos: u16,
    pub y_pos: u16,
    pub y_subpos: u16,
    pub x_width: u16,
    pub y_height: u16,
    pub properties: u16,
    pub extra_properties: u16,
    pub ai_handler_bits: u16,
    pub health: u16,
    pub spritemap_pointer: u16,
    pub timer: u16,
    pub current_instruction: u16,
    pub instruction_timer: u16,
    pub palette_index: u16,
    pub vram_tiles_index: u16,
    pub layer: u16,
    pub flash_timer: u16,
    pub frozen_timer: u16,
    pub invincibility_timer: u16,
    pub shake_timer: u16,
    pub frame_counter: u16,
    pub bank: u8,
    pub field_2F: u8,
    pub ai_var_A: u16,
    pub ai_var_B: u16,
    pub ai_var_C: u16,
    pub ai_var_D: u16,
    pub ai_var_E: u16,
    pub ai_preinstr: u16,
    pub parameter_1: u16,
    pub parameter_2: u16,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct EnemyDef {
    pub tile_data_size: u16,
    pub palette_ptr: VoidP,
    pub health: u16,
    pub damage: u16,
    pub x_radius: u16,
    pub y_radius: u16,
    pub bank: u8,
    pub hurt_ai_time: u8,
    pub hurt_sfx: u16,
    pub boss_fight_value: u16,
    pub ai_init: VoidP,
    pub num_parts: u16,
    pub field_16: u16,
    pub main_ai: VoidP,
    pub grapple_ai: VoidP,
    pub hurt_ai: VoidP,
    pub frozen_ai: VoidP,
    pub time_is_frozen_ai: VoidP,
    pub death_anim: u16,
    pub field_24: u16,
    pub field_26: u16,
    pub powerbomb_reaction: VoidP,
    pub field_2A: VoidP,
    pub field_2C: u16,
    pub field_2E: u16,
    pub touch_ai: VoidP,
    pub shot_ai: VoidP,
    pub field_34: VoidP,
    pub tile_data: LongPtr,
    pub layer: u8,
    pub item_drop_chances_ptr: VoidP,
    pub vulnerability_ptr: VoidP,
    pub name_ptr: VoidP,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct EnemySpawnData {
    pub some_flag: u16,
    pub cause_of_death: u16,
    pub field_4: u16,
    pub vram_tiles_index: u16,
    pub palette_index: u16,
    pub field_A: u16,
    pub field_C: u16,
    pub field_E: u16,
    pub xpos2: u16,
    pub ypos2: u16,
    pub field_14: u16,
    pub field_16: u16,
    pub field_18: u16,
    pub field_1A: u16,
    pub field_1C: u16,
    pub id: u16,
    pub x_pos: u16,
    pub y_pos: u16,
    pub init_param: u16,
    pub properties: u16,
    pub extra_properties: u16,
    pub param_1: u16,
    pub param_2: u16,
    pub name: [u8; 18],
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct EnemyTileset {
    pub enemy_def: VoidP,
    pub vram_dst: u16,
}

enemy_layout! {
    Enemy_Botwoon {
        base: EnemyBase,
        words: [
            botwoon_var_A, botwoon_var_B, botwoon_var_C, botwoon_var_D, botwoon_var_E,
            botwoon_var_F, botwoon_parameter_1, botwoon_parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            botwoon_var_00, botwoon_var_01, botwoon_var_02, botwoon_var_03,
            botwoon_var_04, botwoon_var_05, botwoon_var_06, botwoon_var_07,
            botwoon_var_08, botwoon_var_09, botwoon_var_0A, botwoon_var_0B,
            botwoon_var_0C, botwoon_var_0D, botwoon_var_0E, botwoon_var_0F,
            botwoon_var_10, botwoon_var_11, botwoon_var_12, botwoon_var_13,
            botwoon_var_14, botwoon_var_15, botwoon_var_16, botwoon_var_17,
            botwoon_var_18, botwoon_var_19, botwoon_var_1A, botwoon_var_1B,
            botwoon_var_1C, botwoon_var_1D, botwoon_var_1E, botwoon_var_1F,
        ],
        pad2: [u8; 1984],
        words: [
            botwoon_var_20, botwoon_var_21, botwoon_var_22, botwoon_var_23,
            botwoon_var_24, botwoon_var_25, botwoon_var_26, botwoon_var_27,
            botwoon_var_28, botwoon_var_29, botwoon_var_2A, botwoon_var_2B,
            botwoon_var_2C, botwoon_var_2D, botwoon_var_2E, botwoon_var_2F,
            botwoon_var_30, botwoon_var_31, botwoon_var_32, botwoon_var_33,
            botwoon_var_34, botwoon_var_35, botwoon_var_36, botwoon_var_37,
            botwoon_var_38, botwoon_var_39, botwoon_var_3A, botwoon_var_3B,
            botwoon_var_3C, botwoon_var_3D, botwoon_var_3E, botwoon_var_3F,
        ],
        pad3: [u8; 1984],
        words: [
            botwoon_var_40, botwoon_var_41, botwoon_var_42, botwoon_var_43,
            botwoon_var_44, botwoon_var_45, botwoon_var_46, botwoon_var_47,
            botwoon_var_48, botwoon_var_49, botwoon_var_4A, botwoon_var_4B,
            botwoon_var_4C, botwoon_var_4D, botwoon_var_4E, botwoon_var_4F,
            botwoon_var_50, botwoon_var_51, botwoon_var_52, botwoon_var_53,
            botwoon_var_54, botwoon_var_55, botwoon_var_56, botwoon_var_57,
            botwoon_var_58, botwoon_var_59, botwoon_var_5A, botwoon_var_5B,
            botwoon_var_5C, botwoon_var_5D, botwoon_var_5E, botwoon_var_5F,
        ],
    }
}

enemy_layout! {
    Enemy_MotherBrain {
        base: EnemyBase,
        words: [
            mbn_var_A, mbn_var_B, mbn_var_C, mbn_var_D, mbn_var_E, mbn_var_F,
            mbn_parameter_1, mbn_parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            mbn_var_00, mbn_var_01, mbn_var_02, mbn_var_03, mbn_var_04, mbn_var_05,
            mbn_var_06, mbn_var_07, mbn_var_08, mbn_var_09, mbn_var_0A, mbn_var_0B,
            mbn_var_0C, mbn_var_0D, mbn_var_0E, mbn_var_0F, mbn_var_10, mbn_var_11,
            mbn_var_12, mbn_var_13, mbn_var_14, mbn_var_15, mbn_var_16, mbn_var_17,
            mbn_var_18, mbn_var_19, mbn_var_1A, mbn_var_1B, mbn_var_1C, mbn_var_1D,
            mbn_var_1E, mbn_var_1F,
        ],
        pad2: [u8; 1984],
        words: [
            mbn_var_20, mbn_var_21, mbn_var_22, mbn_var_23, mbn_var_24, mbn_var_25,
            mbn_var_26, mbn_var_27, mbn_var_28, mbn_var_29, mbn_var_2A, mbn_var_2B,
            mbn_var_2C, mbn_var_2D, mbn_var_2E, mbn_var_2F, mbn_var_30, mbn_var_31,
            mbn_var_32, mbn_var_33, mbn_var_34, mbn_var_35, mbn_var_36, mbn_var_37,
            mbn_var_38, mbn_var_39, mbn_var_3A, mbn_var_3B, mbn_var_3C, mbn_var_3D,
            mbn_var_3E, mbn_var_3F,
        ],
    }
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct EprojDef {
    pub init_code_ptr: VoidP,
    pub pre_instr_ptr: VoidP,
    pub instr_list: VoidP,
    pub radius: u16,
    pub properties: u16,
    pub hit_instruction_list: VoidP,
    pub shot_instruction_list: VoidP,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct ExpandingSquareTransitionHdma {
    pub field_0: u8,
    pub field_1: u8,
    pub field_2: u8,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct ExtendedSpriteMap {
    pub xpos: u16,
    pub ypos: u16,
    pub spritemap: VoidP,
    pub hitbox_ptr_: VoidP,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union ExtraEnemyRam7800 {
    pub kraid: Ram7800_Kraid,
    pub pad: [u8; 64],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union ExtraEnemyRam8000 {
    pub pad: [u8; 64],
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct Hitbox {
    pub left: u16,
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
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
    pub xpos: u16,
    pub ypos: u16,
    pub pause_screen_animation_id: u16,
    pub joypad_input: u16,
    pub map_scroll_dir: u16,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct Mode7VramWriteQueue {
    pub field_0: u8,
    pub field_1: u16,
    pub gap3: [u8; 2],
    pub field_5: u16,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct PairU16 {
    pub k: u16,
    pub j: u16,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Pair_Bool_Amt {
    pub flag: bool,
    pub amt: i32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Point16U {
    pub x: u16,
    pub y: u16,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Point32 {
    pub x: u32,
    pub y: u32,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct ProjectileInstr {
    pub timer: u16,
    pub spritemap_ptr: VoidP,
    pub x_radius: u8,
    pub y_radius: u8,
    pub field_6: u16,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union Ram3000 {
    pub pause_menu_map_tilemap: [u16; 1024],
    pub cinematic_bg_tilemap: [u16; 1024],
    pub msgbox: Ram3000_MsgBox,
    pub msgbox_y_scroll_hdma: [u8; 240],
    pub misc: Ram3000_Misc,
    pub menu: Ram3000_Menu,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct Ram3000_Menu {
    pub field_0: [u8; 768],
    pub palette_backup_in_menu: [u16; 256],
    pub backup_of_io_registers_in_gameover: [u8; 54],
    pub field_536: [u8; 202],
    pub menu_tilemap: [u8; 512],
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct Ram3000_Misc {
    pub field_0: [u8; 472],
    pub pause_screen_samus_witeframe_tilemap: [u16; 264],
    pub field_3E8: [u8; 24],
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct Ram3000_MsgBox {
    pub pad: [u8; 188],
    pub msg_box_anim_y_radius_neg: [u16; 30],
    pub msg_box_anim_y_radius: [u16; 30],
    pub field_134: [u8; 204],
    pub tilemap: [u16; 192],
    pub indirect_hdma: [u8; 7],
    pub field_387: [u8; 99],
    pub backup_of_enabled_hdma_channels: u8,
    pub backup_of_bg3_tilemap_and_size: u16,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union Ram3800 {
    pub cinematic_bg_tilemap: [u16; 1024],
    pub equipment_screen_bg1_tilemap: [u16; 1024],
    pub debug_game_over_tilemap: [u16; 1024],
    pub cleared_message_box_bg3_tilemap: [u16; 1024],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union Ram4000 {
    pub xray_tilemaps: [u16; 6144],
    pub bg2_tilemap: [u16; 2048],
    pub decomp_buffer_kraid: [u16; 1024],
    pub bg2_room_select_map_tilemap: [u16; 1024],
    pub intro_japanese_text_tiles: [u16; 768],
    pub backups: Ram4000_Backups,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct Ram4000_Backups {
    pub field_0: [u8; 256],
    pub backup_of_vram_0x5880_msgbox: [u16; 896],
    pub field_800: [u8; 2048],
    pub backup_of_0x3e00_in_kraid_pause: [u8; 1024],
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct Ram7800_Kraid {
    pub kraid_next: u16,
    pub field_2: u16,
    pub field_4: u16,
    pub kraid_thinking: u16,
    pub kraid_min_y_pos_eject: u16,
    pub kraid_mouth_flags: u16,
    pub kraid_healths_8ths: [u16; 8],
    pub field_1C: u16,
    pub kraid_target_x: u16,
    pub kraid_healths_4ths: [u16; 4],
    pub field_28: u16,
    pub kraid_hurt_frame: u16,
    pub kraid_hurt_frame_timer: u16,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Ram8000_Default {
    pub var_20: u16,
    pub var_21: u16,
    pub var_22: u16,
    pub var_23: u16,
    pub var_24: u16,
    pub var_25: u16,
    pub var_26: u16,
    pub var_27: u16,
    pub var_28: u16,
    pub var_29: u16,
    pub var_2A: u16,
    pub var_2B: u16,
    pub var_2C: u16,
    pub var_2D: u16,
    pub var_2E: u16,
    pub var_2F: u16,
    pub var_30: u16,
    pub var_31: u16,
    pub var_32: u16,
    pub var_33: u16,
    pub var_34: u16,
    pub var_35: u16,
    pub var_36: u16,
    pub var_37: u16,
    pub var_38: u16,
    pub var_39: u16,
    pub var_3A: u16,
    pub var_3B: u16,
    pub var_3C: u16,
    pub var_3D: u16,
    pub var_3E: u16,
    pub var_3F: u16,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Rect16U {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct RoomDefHeader {
    pub semiunique_room_number: u8,
    pub area_index_: u8,
    pub x_coordinate_on_map: u8,
    pub y_coordinate_on_map: u8,
    pub width: u8,
    pub height: u8,
    pub up_scroller_: u8,
    pub down_scroller_: u8,
    pub cre_bitset_: u8,
    pub ptr_to_doorout: VoidP,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct RoomDefRoomstate {
    pub compressed_room_map_ptr: LongPtr,
    pub graphics_set: u8,
    pub music_track: u8,
    pub music_control: u8,
    pub room_layer3_fx_ptr: VoidP,
    pub enemy_population_ptr_: VoidP,
    pub enemy_tilesets_ptr: VoidP,
    pub vertical_screen_nudge_limit: u16,
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
    pub x_block: u8,
    pub y_block: u8,
    pub plm_room_argument: u16,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct SamusPoseParams {
    pub pose_x_dir: u8,
    pub movement_type: u8,
    pub new_pose_unless_buttons: u8,
    pub direction_shots_fired: u8,
    pub y_offset_to_gfx: u8,
    pub field_5: u8,
    pub y_radius: u8,
    pub field_7: u8,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct SpawnHardcodedPlmArgs {
    pub block_x: u8,
    pub block_y: u8,
    pub header: VoidP,
}

impl SpawnHardcodedPlmArgs {
    pub const fn new(block_x: u8, block_y: u8, header: VoidP) -> Self {
        Self {
            block_x,
            block_y,
            header,
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct SpawnHdmaObject_Args {
    pub dma_control: u8,
    pub target_register: u8,
    pub instruction_list: VoidP,
}

impl SpawnHdmaObject_Args {
    pub const fn new(dma_control: u8, target_register: u8, instruction_list: VoidP) -> Self {
        Self {
            dma_control,
            target_register,
            instruction_list,
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct StartDmaCopy {
    pub chan: u8,
    pub dmap: u8,
    pub bbad: u8,
    pub a1: LongPtr,
    pub das: VoidP,
}

impl StartDmaCopy {
    const fn new(channel: u8, mode: u8, register: u8, source: u32, size: u16) -> Self {
        Self {
            chan: channel,
            dmap: mode,
            bbad: register,
            a1: LongPtr {
                addr: source as u16,
                bank: (source >> 16) as u8,
            },
            das: size,
        }
    }

    /// Copy alternating low/high VRAM bytes. VMAIN controls address stepping.
    pub const fn vram_words(channel: u8, source: u32, size: u16) -> Self {
        Self::new(
            channel,
            1,
            crate::snes::registers::VMDATAL as u8,
            source,
            size,
        )
    }

    /// Copy only the low byte of each VRAM word.
    pub const fn vram_low_bytes(channel: u8, source: u32, size: u16) -> Self {
        Self::new(
            channel,
            0,
            crate::snes::registers::VMDATAL as u8,
            source,
            size,
        )
    }

    /// Copy only the high byte of each VRAM word.
    pub const fn vram_high_bytes(channel: u8, source: u32, size: u16) -> Self {
        Self::new(
            channel,
            0,
            crate::snes::registers::VMDATAH as u8,
            source,
            size,
        )
    }

    /// Copy bytes through the auto-incrementing WRAM data port.
    pub const fn wram_bytes(channel: u8, source: u32, size: u16) -> Self {
        Self::new(
            channel,
            0,
            crate::snes::registers::WMDATA as u8,
            source,
            size,
        )
    }
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct TileTable {
    pub top_left: u16,
    pub top_right: u16,
    pub bottom_left: u16,
    pub bottom_right: u16,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct TileTables {
    pub tables: [TileTable; 1024],
}

/// Order in which the four destinations of a block are written.
pub(crate) enum TileDrawOrder {
    Rows,
    Columns,
}

impl TileTables {
    /// Draw a block's four tiles with its horizontal and vertical flips.
    ///
    /// Destinations follow `order`. Read each source immediately before its
    /// store: a destination may overlap a source needed by a later tile.
    ///
    /// # Safety
    /// `table` must point to a readable tile table and each destination must
    /// be writable for two bytes. Pointers may overlap and be unaligned.
    #[inline]
    pub(crate) unsafe fn draw_block(
        table: *const Self,
        block: u16,
        destinations: [*mut u16; 4],
        order: TileDrawOrder,
    ) {
        let corners = match order {
            TileDrawOrder::Rows => [0, 1, 2, 3],
            TileDrawOrder::Columns => [0, 2, 1, 3],
        };
        let flip = ((block >> 10) & 3) as usize;
        let tile_flip = (block & 0x0c00) << 4;
        let first = (block & 0x03ff) as usize * 4;
        for (destination, corner) in destinations.into_iter().zip(corners) {
            let source = table.cast::<u16>().add(first + (corner ^ flip));
            let tile = u16::from_le(source.read_unaligned()) ^ tile_flip;
            destination.write_unaligned(tile.to_le());
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct VramReadQueueEnt {
    pub vram_target: u16,
    pub dma_parameters: u16,
    pub src: LongPtr,
    pub size: u16,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct Vulnerability {
    pub power: u8,
    pub wave: u8,
    pub ice: u8,
    pub ice_wave: u8,
    pub spazer: u8,
    pub spazer_wave: u8,
    pub spazer_ice: u8,
    pub spazer_ice_wave: u8,
    pub plasma: u8,
    pub plasma_wave: u8,
    pub plasma_ice: u8,
    pub plasma_ice_wave: u8,
    pub missile: u8,
    pub super_missile: u8,
    pub bomb: u8,
    pub power_bomb: u8,
    pub speed_booster: u8,
    pub shinespark: u8,
    pub screw_attack: u8,
    pub charged_beam: u8,
    pub pseudo_screw_attack: u8,
    pub field_15: u8,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Apu {
    pub spc: *mut Spc,
    pub dsp: *mut Dsp,
    pub ram: [u8; 65536],
    pub romReadable: bool,
    pub dspAdr: u8,
    pub cycles: u32,
    pub inPorts: [u8; 6],
    pub outPorts: [u8; 4],
    pub timer: [Timer; 3],
    pub cpuCyclesLeft: u8,
    pub pad: [u8; 6],
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
    pub hScroll: u16,
    pub vScroll: u16,
    pub tilemapWider: bool,
    pub tilemapHigher: bool,
    pub tilemapAdr: u16,
    pub tileAdr: u16,
    pub bigTiles: bool,
    pub mosaicEnabled: bool,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Cart {
    pub snes: *mut Snes,
    pub type_0: u8,
    pub rom: *mut u8,
    pub romSize: u32,
    pub ram: *mut u8,
    pub ramSize: u32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Channel {
    pub pattern_order_ptr_for_chan: u16,
    pub index: u8,
    pub note_ticks_left: u8,
    pub note_keyoff_ticks_left: u8,
    pub subroutine_num_loops: u8,
    pub volume_fade_ticks: u8,
    pub pan_num_ticks: u8,
    pub pitch_slide_length: u8,
    pub pitch_slide_delay_left: u8,
    pub vibrato_hold_count: u8,
    pub vib_depth: u8,
    pub tremolo_hold_count: u8,
    pub tremolo_depth: u8,
    pub vibrato_change_count: u8,
    pub note_length: u8,
    pub note_gate_off_fixedpt: u8,
    pub channel_volume_master: u8,
    pub instrument_id: u8,
    pub instrument_pitch_base: u16,
    pub saved_pattern_ptr: u16,
    pub pattern_start_ptr: u16,
    pub pitch_envelope_num_ticks: u8,
    pub pitch_envelope_delay: u8,
    pub pitch_envelope_direction: u8,
    pub pitch_envelope_slide_value: u8,
    pub vibrato_count: u8,
    pub vibrato_rate: u8,
    pub vibrato_delay_ticks: u8,
    pub vibrato_fade_num_ticks: u8,
    pub vibrato_fade_add_per_tick: u8,
    pub vibrato_depth_target: u8,
    pub tremolo_count: u8,
    pub tremolo_rate: u8,
    pub tremolo_delay_ticks: u8,
    pub channel_transposition: u8,
    pub channel_volume: u16,
    pub volume_fade_addpertick: u16,
    pub volume_fade_target: u8,
    pub final_volume: u8,
    pub pan_value: u16,
    pub pan_add_per_tick: u16,
    pub pan_target_value: u8,
    pub pan_flag_with_phase_invert: u8,
    pub pitch: u16,
    pub pitch_add_per_tick: u16,
    pub pitch_target: u8,
    pub fine_tune: u8,
    pub cutk: u8,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Cpu {
    pub mem: *mut ::core::ffi::c_void,
    pub memType: u8,
    pub a: u16,
    pub x: u16,
    pub y: u16,
    pub sp: u16,
    pub pc: u16,
    pub dp: u16,
    pub k: u8,
    pub db: u8,
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
    pub cyclesUsed: u8,
    pub spBreakpoint: u16,
    pub in_emu: bool,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Dma {
    pub snes: *mut Snes,
    pub channel: [DmaChannel; 8],
    pub hdmaTimer: u16,
    pub dmaTimer: u32,
    pub dmaBusy: bool,
    pub pad: [u8; 7],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct DmaChannel {
    pub bAdr: u8,
    pub aAdr: u16,
    pub aBank: u8,
    pub size: u16,
    pub indBank: u8,
    pub tableAdr: u16,
    pub repCount: u8,
    pub unusedByte: u8,
    pub dmaActive: bool,
    pub hdmaActive: bool,
    pub mode: u8,
    pub fixed: bool,
    pub decrement: bool,
    pub indirect: bool,
    pub fromB: bool,
    pub unusedBit: bool,
    pub doTransfer: bool,
    pub terminated: bool,
    pub offIndex: u8,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Dsp {
    pub apu_ram: *mut u8,
    pub ram: [u8; 128],
    pub channel: [DspChannel; 8],
    pub dirPage: u16,
    pub evenCycle: bool,
    pub mute: bool,
    pub reset: bool,
    pub masterVolumeL: i8,
    pub masterVolumeR: i8,
    pub noiseSample: i16,
    pub noiseRate: u16,
    pub noiseCounter: u16,
    pub echoWrites: bool,
    pub echoVolumeL: i8,
    pub echoVolumeR: i8,
    pub feedbackVolume: i8,
    pub echoBufferAdr: u16,
    pub echoDelay: u16,
    pub echoRemain: u16,
    pub echoBufferIndex: u16,
    pub firBufferIndex: u8,
    pub firValues: [i8; 8],
    pub firBufferL: [i16; 8],
    pub firBufferR: [i16; 8],
    pub sampleBuffer: [i16; 1068],
    pub sampleOffset: u16,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct DspChannel {
    pub pitch: u16,
    pub pitchCounter: u16,
    pub pitchModulation: bool,
    pub decodeBuffer: [i16; 19],
    pub srcn: u8,
    pub decodeOffset: u16,
    pub previousFlags: u8,
    pub old: i16,
    pub older: i16,
    pub useNoise: bool,
    pub adsrRates: [u16; 4],
    pub rateCounter: u16,
    pub adsrState: u8,
    pub sustainLevel: u16,
    pub useGain: bool,
    pub gainMode: u8,
    pub directGain: bool,
    pub gainValue: u16,
    pub gain: u16,
    pub keyOn: bool,
    pub keyOff: bool,
    pub sampleOut: i16,
    pub volumeL: i8,
    pub volumeR: i8,
    pub echoEnable: bool,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct DspRegWriteHistory {
    pub count: u32,
    pub addr: [u8; 256],
    pub val: [u8; 256],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Input {
    pub snes: *mut Snes,
    pub type_0: u8,
    pub currentState: u16,
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
    pub vram: [u16; 32768],
    pub vramPointer: u16,
    pub vramIncrementOnHigh: bool,
    pub vramIncrement: u16,
    pub vramRemapMode: u8,
    pub vramReadBuffer: u16,
    pub cgram: [u16; 256],
    pub cgramPointer: u8,
    pub cgramSecondWrite: bool,
    pub cgramBuffer: u8,
    pub oam: [u16; 256],
    pub highOam: [u8; 32],
    pub oamAdr: u8,
    pub oamAdrWritten: u8,
    pub oamInHigh: bool,
    pub oamInHighWritten: bool,
    pub oamSecondWrite: bool,
    pub oamBuffer: u8,
    pub objPriority: bool,
    pub objTileAdr1: u16,
    pub objTileAdr2: u16,
    pub objSize: u8,
    pub objPixelBufferXX: [u8; 256],
    pub objPriorityBufferXX: [u8; 256],
    pub timeOver: bool,
    pub rangeOver: bool,
    pub objInterlace: bool,
    pub bgLayer: [BgLayer; 4],
    pub scrollPrev: u8,
    pub scrollPrev2: u8,
    pub mosaicSize: u8,
    pub mosaicStartLine: u8,
    pub layer: [Layer; 5],
    pub m7matrix: [i16; 8],
    pub m7prev: u8,
    pub m7largeField: bool,
    pub m7charFill: bool,
    pub m7xFlip: bool,
    pub m7yFlip: bool,
    pub m7extBg: bool,
    pub m7startX: i32,
    pub m7startY: i32,
    pub windowLayer: [WindowLayer; 6],
    pub window1left: u8,
    pub window1right: u8,
    pub window2left: u8,
    pub window2right: u8,
    pub clipMode: u8,
    pub preventMathMode: u8,
    pub addSubscreen: bool,
    pub subtractColor: bool,
    pub halfColor: bool,
    pub mathEnabled: [bool; 6],
    pub fixedColorR: u8,
    pub fixedColorG: u8,
    pub fixedColorB: u8,
    pub forcedBlank: bool,
    pub brightness: u8,
    pub mode: u8,
    pub bg3priority: bool,
    pub evenFrame: bool,
    pub pseudoHires: bool,
    pub overscan: bool,
    pub frameOverscan: bool,
    pub interlace: bool,
    pub frameInterlace: bool,
    pub directColor: bool,
    pub hCount: u16,
    pub vCount: u16,
    pub hCountSecond: bool,
    pub vCountSecond: bool,
    pub countersLatched: bool,
    pub ppu1openBus: u8,
    pub ppu2openBus: u8,
    pub pixelbuffer_placeholder: u8,
    pub windowsel: u32,
    pub extraLeftCur: u8,
    pub extraRightCur: u8,
    pub extraLeftRight: u8,
    pub screenEnabled: [u8; 2],
    pub screenWindowed: [u8; 2],
    pub mosaicEnabled: u8,
    pub lastBrightnessMult: u8,
    pub lineHasSprites: bool,
    pub bgBuffers: [PpuPixelPrioBufs; 2],
    pub objBuffer: PpuPixelPrioBufs,
    pub renderPitch: u32,
    pub renderBuffer: *mut u8,
    pub brightnessMult: [u8; 63],
    pub brightnessMultHalf: [u8; 64],
    pub mosaicModulo: [u8; 256],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct PpuPixelPrioBufs {
    pub data: [PpuZbufType; 256],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct SfxChan {
    pub voice_bitset: u8,
    pub instr_timer: u8,
    pub subnote: u8,
    pub repeat_pt: u8,
    pub note: u8,
    pub ptr_idx: u8,
    pub voice_index: u8,
    pub voice_index_mult8: u8,
    pub release_timer: u8,
    pub volume: u8,
    pub target_note: u8,
    pub phase_invert: u8,
    pub ptr: u16,
    pub repeat_ctr: u8,
    pub enable_pitch_slide: u8,
    pub release_flag: u8,
    pub enable_pitch_slide_legato: u8,
    pub adsr_settings: u16,
    pub channel_mask: u8,
    pub legato: u8,
    pub disable: u8,
    pub subnote_delta: u8,
    pub update_adsr: u8,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct SfxRegs {
    pub priority: u8,
    pub chan_idx_x2: u8,
    pub chan_voice_bitset_ptr: u16,
    pub cur_sound: u8,
    pub chan_voice_mask_ptr: u16,
    pub chan_voice_index_ptr: u16,
    pub some_value: u8,
    pub cur_ptr: u16,
    pub enabled_voices: u8,
    pub voices_to_setup: u8,
    pub chan_idx: u8,
    pub voices_remaining: u8,
    pub cur_sound_index: u8,
    pub voice_id: u8,
    pub init_flag: u8,
    pub voice_index: u8,
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
    pub runningWhichVersion: u8,
    pub ramAdr: u32,
    pub ram: *mut u8,
    pub padx: [u8; 4],
    pub hPos: u16,
    pub vPos: u16,
    pub frames: u32,
    pub cpuCyclesLeft: u8,
    pub cpuMemOps: u8,
    pub padpad: [u8; 2],
    pub apuCatchupCycles: ::core::ffi::c_double,
    pub hIrqEnabled: bool,
    pub vIrqEnabled: bool,
    pub nmiEnabled: bool,
    pub hTimer: u16,
    pub vTimer: u16,
    pub inNmi: bool,
    pub inIrq: bool,
    pub inVblank: bool,
    pub portAutoReadX: [u16; 4],
    pub autoJoyRead: bool,
    pub autoJoyTimer: u16,
    pub ppuLatch: bool,
    pub multiplyA: u8,
    pub multiplyResult: u16,
    pub divideA: u16,
    pub divideResult: u16,
    pub fastMem: bool,
    pub openBus: u8,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Spc {
    pub apu: *mut Apu,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub pc: u16,
    pub c: bool,
    pub z: bool,
    pub v: bool,
    pub n: bool,
    pub i: bool,
    pub h: bool,
    pub p: bool,
    pub b: bool,
    pub stopped: bool,
    pub cyclesUsed: u8,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct SpcPlayer {
    pub reg_write_history: *mut DspRegWriteHistory,
    pub timer_cycles: u8,
    pub dsp: *mut Dsp,
    pub port_to_snes: [u8; 4],
    pub last_value_from_snes: [u8; 4],
    pub counter_sf0c: u16,
    pub always_zero: u16,
    pub temp_accum: u16,
    pub ttt: u8,
    pub did_affect_volumepitch_flag: u8,
    pub addr0: u16,
    pub addr1: u16,
    pub lfsr_value: u16,
    pub is_chan_on: u8,
    pub fast_forward: u8,
    pub word_20: u16,
    pub music_ptr_toplevel: u16,
    pub block_count: u8,
    pub sfx_timer_accum: u8,
    pub chn: u8,
    pub key_ON: u8,
    pub key_OFF: u8,
    pub cur_chan_bit: u8,
    pub reg_FLG: u8,
    pub reg_NON: u8,
    pub reg_EON: u8,
    pub echo_stored_time: u8,
    pub echo_parameter_EDL: u8,
    pub reg_EFB: u8,
    pub global_transposition: u8,
    pub main_tempo_accum: u8,
    pub tempo: u16,
    pub tempo_fade_num_ticks: u8,
    pub tempo_fade_final: u8,
    pub tempo_fade_add: u16,
    pub master_volume: u16,
    pub master_volume_fade_ticks: u8,
    pub master_volume_fade_target: u8,
    pub master_volume_fade_add_per_tick: u16,
    pub vol_dirty: u8,
    pub percussion_base_id: u8,
    pub echo_volume_left: u16,
    pub echo_volume_right: u16,
    pub echo_volume_fade_add_left: u16,
    pub echo_volume_fade_add_right: u16,
    pub echo_volume_fade_ticks: u8,
    pub echo_volume_fade_target_left: u8,
    pub echo_volume_fade_target_right: u8,
    pub p_echoBuffer: u16,
    pub memset_ptr: u16,
    pub memset_ctr: u8,
    pub disable_sfx2: u8,
    pub byte_4B1: u8,
    pub channel: [Channel; 8],
    pub sfx1: SfxRegs,
    pub sfx2: SfxRegs,
    pub sfx3: SfxRegs,
    pub sfx_chans_1: [SfxChan; 4],
    pub sfx_chans_2: [SfxChan; 2],
    pub sfx_chans_3: [SfxChan; 2],
    pub last_written_edl: u8,
    pub input_ports: [u8; 4],
    pub ram: [u8; 65536],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Timer {
    pub cycles: u8,
    pub divider: u8,
    pub target: u8,
    pub counter: u8,
    pub enabled: bool,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct WindowLayer {
    pub window1enabled: bool,
    pub window2enabled: bool,
    pub window1inversed: bool,
    pub window2inversed: bool,
    pub maskLogic: u8,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ByteArray {
    pub data: *mut u8,
    pub size: size_t,
    pub capacity: size_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Config {
    pub window_width: i32,
    pub window_height: i32,
    pub enhanced_mode7: bool,
    pub new_renderer: bool,
    pub ignore_aspect_ratio: bool,
    pub fullscreen: u8,
    pub window_scale: u8,
    pub enable_audio: bool,
    pub linear_filtering: bool,
    pub output_method: u8,
    pub audio_freq: u16,
    pub audio_channels: u8,
    pub audio_samples: u16,
    pub autosave: bool,
    pub extended_aspect_ratio: u8,
    pub extend_y: bool,
    pub no_sprite_limits: bool,
    pub display_perf_title: bool,
    pub enable_msu: u8,
    pub resume_msu: bool,
    pub disable_frame_delay: bool,
    pub msuvolume: u8,
    pub features0: u32,
    pub link_graphics: *const ::core::ffi::c_char,
    pub memory_buffer: *mut ::core::ffi::c_char,
    pub shader: *const ::core::ffi::c_char,
    pub msu_path: *const ::core::ffi::c_char,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct GlTextureWithSize {
    pub gl_texture: u32,
    pub width: u16,
    pub height: u16,
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
    pub uniform: [u32; 20],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct GlslPass {
    pub filename: *mut ::core::ffi::c_char,
    pub scale_type_x: u8,
    pub scale_type_y: u8,
    pub float_framebuffer: bool,
    pub srgb_framebuffer: bool,
    pub mipmap_input: bool,
    pub scale_x: ::core::ffi::c_float,
    pub scale_y: ::core::ffi::c_float,
    pub wrap_mode: u32,
    pub frame_count_mod: u32,
    pub frame_count: u32,
    pub gl_program: u32,
    pub gl_fbo: u32,
    pub filter: u32,
    pub gl_texture: u32,
    pub width: u16,
    pub height: u16,
    pub unif: GlslUniforms,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct GlslShader {
    pub n_pass: i32,
    pub pass: *mut GlslPass,
    pub first_param: *mut GlslParam,
    pub first_texture: *mut GlslTexture,
    pub gl_vao: *mut u32,
    pub gl_vbo: u32,
    pub frame_count: u32,
    pub max_prev_frame: i32,
    pub prev_frame: [GlTextureWithSize; 8],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct GlslTexture {
    pub next: *mut GlslTexture,
    pub id: *mut ::core::ffi::c_char,
    pub filename: *mut ::core::ffi::c_char,
    pub filter: u32,
    pub gl_texture: u32,
    pub wrap_mode: u32,
    pub mipmap: bool,
    pub width: i32,
    pub height: i32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct GlslTextureUniform {
    pub Texture: i32,
    pub InputSize: i32,
    pub TextureSize: i32,
    pub TexCoord: i32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct GlslUniforms {
    pub Top: GlslTextureUniform,
    pub OutputSize: i32,
    pub FrameCount: i32,
    pub FrameDirection: i32,
    pub LUTTexCoord: i32,
    pub VertexCoord: i32,
    pub Orig: GlslTextureUniform,
    pub Prev: [GlslTextureUniform; 7],
    pub Pass: [GlslTextureUniform; 20],
    pub PassPrev: [GlslTextureUniform; 20],
    pub Texture: [i32; 10],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct RendererFuncs {
    pub Initialize: Option<unsafe fn(*mut SDL_Window, &mut Game) -> bool>,
    pub Destroy: Option<unsafe fn(&mut Game) -> ()>,
    pub BeginDraw: Option<unsafe fn(i32, i32, *mut *mut u8, *mut i32, &mut Game) -> ()>,
    pub EndDraw: Option<unsafe fn(&mut Game) -> ()>,
}

#[repr(C)]
pub struct _IO_FILE {
    _private: [u8; 0],
}

pub type FILE = _IO_FILE;

pub type KeyAction = u32;

pub type OutputMethod = u32;

pub type GamepadButton = i32;

pub type SaveLoadOperation = u32;

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

/// Immutable format/path pointer storage. Every entry points to a static
/// NUL-terminated byte string; dereferencing this table never mutates strings.
#[repr(transparent)]
pub(crate) struct ImmutableCStringPointers<const N: usize>([*const core::ffi::c_char; N]);
// Safety: tables contain only immutable static string addresses, and expose
// no mutable access to either the pointer array or the pointed-to bytes.
unsafe impl<const N: usize> Sync for ImmutableCStringPointers<N> {}
impl<const N: usize> ImmutableCStringPointers<N> {
    /// # Safety
    /// Every pointer must refer to an immutable NUL-terminated byte string
    /// that remains valid for the entire program. Neither its bytes nor the
    /// pointer storage may be mutated through another alias.
    pub(crate) const unsafe fn new(pointers: [*const core::ffi::c_char; N]) -> Self {
        Self(pointers)
    }
}
impl<const N: usize> core::ops::Deref for ImmutableCStringPointers<N> {
    type Target = [*const core::ffi::c_char; N];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
