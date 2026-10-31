//! Shared byte-compatible gameplay, hardware and music types.

use crate::game_state::Game;
use ::core::ffi::{c_char, c_double, c_long, c_uchar, c_ushort, c_void};
pub use sm_game::actors::content_records::{EnemyDef, ExtendedSpriteMap, Hitbox, LongPtr};
pub use sm_game::actors::layouts::EnemyBase;

crate::type_aliases! {
    pub VoidP, PpuZbufType, Uint16, stbi__uint16 = u16;
}

crate::typed_struct! {
    [
        #[derive(Copy, Clone)]
        #[repr(C, packed)]
    ];
    pub struct VramWriteEntry {
        pub size: u16;
        pub src: LongPtr;
        pub vram_dst: u16;
    }

    pub struct DoorDef {
        pub room_definition_ptr: VoidP;
        pub door_bitflags, door_orientation, x_pos_plm, y_pos_plm, x_pos_in_room, y_pos_in_room: u8;
        pub samus_distance_from_door: u16;
        pub door_setup_code: VoidP;
    }

    pub struct EnemyData {
        pub enemy_ptr, x_pos, x_subpos, y_pos, y_subpos, x_width, y_height, properties,
            extra_properties, ai_handler_bits, health, spritemap_pointer, timer,
            current_instruction, instruction_timer, palette_index, vram_tiles_index, layer,
            flash_timer, frozen_timer, invincibility_timer, shake_timer, frame_counter: u16;
        pub bank, field_2f: u8;
        pub ai_var_a, ai_var_b, ai_var_c, ai_var_d, ai_var_e, ai_preinstr, parameter_1, parameter_2:
            u16;
    }

    pub struct EnemySpawnData {
        pub some_flag, cause_of_death, field_4, vram_tiles_index, palette_index, field_A, field_C,
            field_E, xpos2, ypos2, field_14, field_16, field_18, field_1A, field_1c, id, x_pos,
            y_pos, init_param, properties, extra_properties, param_1, param_2: u16;
        pub name: [u8; 18];
    }

    pub struct EnemyTileset {
        pub enemy_def: VoidP;
        pub vram_dst: u16;
    }

    pub struct EprojDef {
        pub init_code_ptr, pre_instr_ptr, instr_list: VoidP;
        pub radius, properties: u16;
        pub hit_instruction_list, shot_instruction_list: VoidP;
    }

    pub struct ExpandingSquareTransitionHdma {
        pub field_0, field_1, field_2: u8;
    }

    pub struct MapIconDataPointers {
        pub crateria, brinstar, norfair, wrecked_ship, maraidia, tourian, ceres, debug: VoidP;
    }

    pub struct MapScrollArrowData {
        pub xpos, ypos, pause_screen_animation_id, joypad_input, map_scroll_dir: u16;
    }

    pub struct ProjectileInstr {
        pub timer: u16;
        pub spritemap_ptr: VoidP;
        pub x_radius, y_radius: u8;
        pub field_6: u16;
    }

    pub struct Ram3000_Menu {
        pub field_0: [u8; 768];
        pub palette_backup_in_menu: [u16; 256];
        pub backup_of_io_registers_in_gameover: [u8; 54];
        pub field_536: [u8; 202];
        pub menu_tilemap: [u8; 512];
    }

    pub struct Ram3000_Misc {
        pub field_0: [u8; 472];
        pub pause_screen_samus_witeframe_tilemap: [u16; 264];
        pub field_3E8: [u8; 24];
    }

    pub struct Ram3000_MsgBox {
        pub pad: [u8; 188];
        pub msg_box_anim_y_radius_neg, msg_box_anim_y_radius: [u16; 30];
        pub field_134: [u8; 204];
        pub tilemap: [u16; 192];
        pub indirect_hdma: [u8; 7];
        pub field_387: [u8; 99];
        pub backup_of_enabled_hdma_channels: u8;
        pub backup_of_bg3_tilemap_and_size: u16;
    }

    pub struct Ram4000_Backups {
        pub field_0: [u8; 256];
        pub backup_of_vram_0x5880_msgbox: [u16; 896];
        pub field_800: [u8; 2048];
        pub backup_of_0x3e00_in_kraid_pause: [u8; 1024];
    }

    pub struct Ram7800_Kraid {
        pub kraid_next, field_2, field_4, kraid_thinking, kraid_min_y_pos_eject, kraid_mouth_flags:
            u16;
        pub kraid_healths_8ths: [u16; 8];
        pub field_1c, kraid_target_x: u16;
        pub kraid_healths_4ths: [u16; 4];
        pub field_28, kraid_hurt_frame, kraid_hurt_frame_timer: u16;
    }

    pub struct RoomDefHeader {
        pub semiunique_room_number, area_index_, x_coordinate_on_map, y_coordinate_on_map, width,
            height, up_scroller_, down_scroller_, cre_bitset_: u8;
        pub ptr_to_doorout: VoidP;
    }

    pub struct RoomDefRoomstate {
        pub compressed_room_map_ptr: LongPtr;
        pub graphics_set, music_track, music_control: u8;
        pub room_layer3_fx_ptr, enemy_population_ptr_, enemy_tilesets_ptr: VoidP;
        pub vertical_screen_nudge_limit: u16;
        pub rdf_scroll_ptr, xray_special_casing_ptr, main_code_ptr, room_plm_header_ptr,
            bg_data_ptr, room_setup_code: VoidP;
    }

    pub struct SamusPoseParams {
        pub pose_x_dir, movement_type, new_pose_unless_buttons, direction_shots_fired,
            y_offset_to_gfx, field_5, y_radius, field_7: u8;
    }

    pub struct TileTable {
        pub top_left, top_right, bottom_left, bottom_right: u16;
    }

    pub struct TileTables {
        pub tables: [TileTable; 1024];
    }

    pub struct Vulnerability {
        pub power, wave, ice, ice_wave, spazer, spazer_wave, spazer_ice, spazer_ice_wave, plasma,
            plasma_wave, plasma_ice, plasma_ice_wave, missile, super_missile, bomb, power_bomb,
            speed_booster, shinespark, screw_attack, charged_beam, pseudo_screw_attack, field_15:
            u8;
    }
}

crate::typed_struct! {
    [
        #[derive(Copy, Clone)]
        #[repr(C)]
    ];
    pub struct OamEnt {
        pub xcoord, ycoord, charnum, flags: u8;
    }

    pub struct CheckSolidEnemyColl_Result {
        pub collision: u16;
        pub amt: i32;
    }

    pub union ExtraEnemyRam7800 {
        pub kraid: Ram7800_Kraid;
        pub pad: [u8; 64];
    }

    pub union ExtraEnemyRam8000 {
        pub pad: [u8; 64];
    }

    pub struct PairU16 {
        pub k, j: u16;
    }

    pub struct Pair_Bool_Amt {
        pub flag: bool;
        pub amt: i32;
    }

    pub struct Point16U {
        pub x, y: u16;
    }

    pub union Ram3000 {
        pub pause_menu_map_tilemap: [u16; 1024];
        pub cinematic_bg_tilemap: [u16; 1024];
        pub msgbox: Ram3000_MsgBox;
        pub msgbox_y_scroll_hdma: [u8; 240];
        pub misc: Ram3000_Misc;
        pub menu: Ram3000_Menu;
    }

    pub union Ram3800 {
        pub cinematic_bg_tilemap: [u16; 1024];
        pub equipment_screen_bg1_tilemap: [u16; 1024];
        pub debug_game_over_tilemap: [u16; 1024];
        pub cleared_message_box_bg3_tilemap: [u16; 1024];
    }

    pub union Ram4000 {
        pub xray_tilemaps: [u16; 6144];
        pub bg2_tilemap: [u16; 2048];
        pub decomp_buffer_kraid: [u16; 1024];
        pub bg2_room_select_map_tilemap: [u16; 1024];
        pub intro_japanese_text_tiles: [u16; 768];
        pub backups: Ram4000_Backups;
    }

    pub struct Ram8000_Default {
        pub var_20, var_21, var_22, var_23, var_24, var_25, var_26, var_27, var_28, var_29, var_2a,
            var_2b, var_2c, var_2d, var_2e, var_2f, var_30, var_31, var_32, var_33, var_34, var_35,
            var_36, var_37, var_38, var_39, var_3a, var_3b, var_3c, var_3d, var_3e, var_3f: u16;
    }

    pub struct Rect16U {
        pub x, y, w, h: u16;
    }

    pub struct SpawnHardcodedPlmArgs {
        pub block_x, block_y: u8;
        pub header: VoidP;
    }

}

// These detached records have typed zero/false/null power-on values.
crate::typed_struct! {
    initial; [#[derive(Copy, Clone)] #[repr(C)]];
    pub struct ApuRegisters {
        pub romReadable: bool;
        pub dspAdr: u8;
        pub snapshot_padding0: [u8; 2];
        pub cycles: u32;
        pub inPorts: [u8; 6];
        pub outPorts: [u8; 4];
        pub timer: [Timer; 3];
        pub cpuCyclesLeft: u8;
        pub pad: [u8; 6];
        pub history: DspRegWriteHistory;
    }

    pub struct BgLayer {
        pub hScroll, vScroll: u16;
        pub tilemapWider, tilemapHigher: bool;
        pub tilemapAdr, tileAdr: u16;
        pub bigTiles, mosaicEnabled: bool;
    }


    pub struct Channel {
        pub pattern_order_ptr_for_chan: u16;
        pub index, note_ticks_left, note_keyoff_ticks_left, subroutine_num_loops, volume_fade_ticks,
            pan_num_ticks, pitch_slide_length, pitch_slide_delay_left, vibrato_hold_count,
            vib_depth, tremolo_hold_count, tremolo_depth, vibrato_change_count, note_length,
            note_gate_off_fixedpt, channel_volume_master, instrument_id: u8;
        pub instrument_pitch_base, saved_pattern_ptr, pattern_start_ptr: u16;
        pub pitch_envelope_num_ticks, pitch_envelope_delay, pitch_envelope_direction,
            pitch_envelope_slide_value, vibrato_count, vibrato_rate, vibrato_delay_ticks,
            vibrato_fade_num_ticks, vibrato_fade_add_per_tick, vibrato_depth_target, tremolo_count,
            tremolo_rate, tremolo_delay_ticks, channel_transposition: u8;
        pub channel_volume, volume_fade_addpertick: u16;
        pub volume_fade_target, final_volume: u8;
        pub pan_value, pan_add_per_tick: u16;
        pub pan_target_value, pan_flag_with_phase_invert: u8;
        pub pitch, pitch_add_per_tick: u16;
        pub pitch_target, fine_tune, cutk: u8;
    }

    pub struct Cpu {
        pub memType: u8;
        pub a, x, y, sp, pc, dp: u16;
        pub k, db: u8;
        pub c, z, v, n, i, d, xf, mf, e, irqWanted, nmiWanted, waiting, stopped: bool;
        pub cyclesUsed: u8;
        pub spBreakpoint: u16;
        pub in_emu: bool;
    }

    pub struct Dma {
        pub channel: [DmaChannel; 8];
        pub hdmaTimer: u16;
        pub snapshot_padding0: [u8; 2];
        pub dmaTimer: u32;
        pub dmaBusy: bool;
        pub pad: [u8; 7];
    }

    pub struct DmaChannel {
        pub bAdr: u8;
        pub snapshot_padding0: [u8; 1];
        pub aAdr: u16;
        pub aBank: u8;
        pub snapshot_padding1: [u8; 1];
        pub size: u16;
        pub indBank: u8;
        pub snapshot_padding2: [u8; 1];
        pub tableAdr: u16;
        pub repCount, unusedByte: u8;
        pub dmaActive, hdmaActive: bool;
        pub mode: u8;
        pub fixed, decrement, indirect, fromB, unusedBit, doTransfer, terminated: bool;
        pub offIndex: u8;
        pub snapshot_padding3: [u8; 1];
    }

    pub struct DspState {
        pub apu_ram: *mut u8;
        pub ram: [u8; 128];
        pub channel: [DspChannel; 8];
        pub dirPage: u16;
        pub evenCycle, mute, reset: bool;
        pub masterVolumeL, masterVolumeR: i8;
        pub snapshot_padding0: [u8; 1];
        pub noiseSample: i16;
        pub noiseRate, noiseCounter: u16;
        pub echoWrites: bool;
        pub echoVolumeL, echoVolumeR, feedbackVolume: i8;
        pub echoBufferAdr, echoDelay, echoRemain, echoBufferIndex: u16;
        pub firBufferIndex: u8;
        pub firValues: [i8; 8];
        pub snapshot_padding1: [u8; 1];
        pub firBufferL, firBufferR: [i16; 8];
        pub sampleBuffer: [i16; 1068];
        pub sampleOffset: u16;
        pub snapshot_padding2: [u8; 2];
    }

    pub struct DspChannel {
        pub pitch, pitchCounter: u16;
        pub pitchModulation: bool;
        pub snapshot_padding0: [u8; 1];
        pub decodeBuffer: [i16; 19];
        pub srcn: u8;
        pub snapshot_padding1: [u8; 1];
        pub decodeOffset: u16;
        pub previousFlags: u8;
        pub snapshot_padding2: [u8; 1];
        pub old, older: i16;
        pub useNoise: bool;
        pub snapshot_padding3: [u8; 1];
        pub adsrRates: [u16; 4];
        pub rateCounter: u16;
        pub adsrState: u8;
        pub snapshot_padding4: [u8; 1];
        pub sustainLevel: u16;
        pub useGain: bool;
        pub gainMode: u8;
        pub directGain: bool;
        pub snapshot_padding5: [u8; 1];
        pub gainValue, gain: u16;
        pub keyOn, keyOff: bool;
        pub sampleOut: i16;
        pub volumeL, volumeR: i8;
        pub echoEnable: bool;
        pub snapshot_padding6: [u8; 1];
    }

    pub struct DspRegWriteHistory {
        pub count: u32;
        pub addr, val: [u8; 256];
    }

    pub struct Input {
        pub type_0: u8;
        pub currentState: u16;
    }

    pub struct Layer {
        pub mainScreenEnabled, subScreenEnabled, mainScreenWindowed, subScreenWindowed: bool;
    }

    pub struct Ppu {
        pub snes: *mut Snes;
        pub vram: [u16; 32768];
        pub vramPointer: u16;
        pub vramIncrementOnHigh: bool;
        pub snapshot_padding0: [u8; 1];
        pub vramIncrement: u16;
        pub vramRemapMode: u8;
        pub snapshot_padding1: [u8; 1];
        pub vramReadBuffer: u16;
        pub cgram: [u16; 256];
        pub cgramPointer: u8;
        pub cgramSecondWrite: bool;
        pub cgramBuffer: u8;
        pub snapshot_padding2: [u8; 1];
        pub oam: [u16; 256];
        pub highOam: [u8; 32];
        pub oamAdr, oamAdrWritten: u8;
        pub oamInHigh, oamInHighWritten, oamSecondWrite: bool;
        pub oamBuffer: u8;
        pub objPriority: bool;
        pub snapshot_padding3: [u8; 1];
        pub objTileAdr1, objTileAdr2: u16;
        pub objSize: u8;
        pub objPixelBufferXX, objPriorityBufferXX: [u8; 256];
        pub timeOver, rangeOver, objInterlace: bool;
        pub bgLayer: [BgLayer; 4];
        pub scrollPrev, scrollPrev2, mosaicSize, mosaicStartLine: u8;
        pub layer: [Layer; 5];
        pub m7matrix: [i16; 8];
        pub m7prev: u8;
        pub m7largeField, m7charFill, m7xFlip, m7yFlip, m7extBg: bool;
        pub m7startX, m7startY: i32;
        pub windowLayer: [WindowLayer; 6];
        pub window1left, window1right, window2left, window2right, clipMode, preventMathMode: u8;
        pub addSubscreen, subtractColor, halfColor: bool;
        pub mathEnabled: [bool; 6];
        pub fixedColorR, fixedColorG, fixedColorB: u8;
        pub forcedBlank: bool;
        pub brightness, mode: u8;
        pub bg3priority, evenFrame, pseudoHires, overscan, frameOverscan, interlace, frameInterlace,
            directColor: bool;
        pub snapshot_padding4: [u8; 1];
        pub hCount, vCount: u16;
        pub hCountSecond, vCountSecond, countersLatched: bool;
        pub ppu1openBus, ppu2openBus, pixelbuffer_placeholder: u8;
        pub windowsel: u32;
        pub extraLeftCur, extraRightCur, extraLeftRight: u8;
        pub screenEnabled, screenWindowed: [u8; 2];
        pub mosaicEnabled, lastBrightnessMult: u8;
        pub lineHasSprites: bool;
        pub bgBuffers: [PpuPixelPrioBufs; 2];
        pub objBuffer: PpuPixelPrioBufs;
        pub renderPitch: u32;
        pub renderBuffer: *mut u8;
        pub brightnessMult: [u8; 63];
        pub brightnessMultHalf: [u8; 64];
        pub mosaicModulo: [u8; 256];
    }

    pub struct PpuPixelPrioBufs {
        pub data: [PpuZbufType; 256];
    }

    pub struct SfxChan {
        pub voice_bitset, instr_timer, subnote, repeat_pt, note, ptr_idx, voice_index,
            voice_index_mult8, release_timer, volume, target_note, phase_invert: u8;
        pub ptr: u16;
        pub repeat_ctr, enable_pitch_slide, release_flag, enable_pitch_slide_legato: u8;
        pub adsr_settings: u16;
        pub channel_mask, legato, disable, subnote_delta, update_adsr: u8;
    }

    pub struct SfxRegs {
        pub priority, chan_idx_x2: u8;
        pub chan_voice_bitset_ptr: u16;
        pub cur_sound: u8;
        pub chan_voice_mask_ptr, chan_voice_index_ptr: u16;
        pub some_value: u8;
        pub cur_ptr: u16;
        pub enabled_voices, voices_to_setup, chan_idx, voices_remaining, cur_sound_index, voice_id,
            init_flag, voice_index: u8;
    }

    pub struct Snes {
        pub cpu: *mut Cpu;
        pub apu: *mut Apu;
        pub ppu, snes_ppu, my_ppu: *mut Ppu;
        pub cart: *mut Cart;
        pub debug_cycles, debug_apu_cycles, disableRender: bool;
        pub runningWhichVersion: u8;
        pub ramAdr: u32;
        pub padx: [u8; 4];
        pub hPos, vPos: u16;
        pub frames: u32;
        pub cpuCyclesLeft, cpuMemOps: u8;
        pub padpad: [u8; 2];
        pub apuCatchupCycles: c_double;
        pub hIrqEnabled, vIrqEnabled, nmiEnabled: bool;
        pub snapshot_padding0: [u8; 1];
        pub hTimer, vTimer: u16;
        pub inNmi, inIrq, inVblank: bool;
        pub snapshot_padding1: [u8; 1];
        pub portAutoReadX: [u16; 4];
        pub autoJoyRead: bool;
        pub snapshot_padding2: [u8; 1];
        pub autoJoyTimer: u16;
        pub ppuLatch: bool;
        pub multiplyA: u8;
        pub multiplyResult, divideA, divideResult: u16;
        pub fastMem: bool;
        pub openBus: u8;
    }

    pub struct Spc {
        pub apu: *mut Apu;
        pub a, x, y, sp: u8;
        pub pc: u16;
        pub c, z, v, n, i, h, p, b, stopped: bool;
        pub cyclesUsed: u8;
    }

    pub struct NativePlayerState {
        pub reg_write_history: *mut DspRegWriteHistory;
        pub timer_cycles: u8;
        pub dsp: *mut Dsp;
        pub port_to_snes, last_value_from_snes: [u8; 4];
        pub counter_sf0c, always_zero, temp_accum: u16;
        pub ttt, did_affect_volumepitch_flag: u8;
        pub addr0, addr1, lfsr_value: u16;
        pub is_chan_on, fast_forward: u8;
        pub word_20, music_ptr_toplevel: u16;
        pub block_count, sfx_timer_accum, chn, key_ON, key_OFF, cur_chan_bit, reg_FLG, reg_NON,
            reg_EON, echo_stored_time, echo_parameter_EDL, reg_EFB, global_transposition,
            main_tempo_accum: u8;
        pub tempo: u16;
        pub tempo_fade_num_ticks, tempo_fade_final: u8;
        pub tempo_fade_add, master_volume: u16;
        pub master_volume_fade_ticks, master_volume_fade_target: u8;
        pub master_volume_fade_add_per_tick: u16;
        pub vol_dirty, percussion_base_id: u8;
        pub echo_volume_left, echo_volume_right, echo_volume_fade_add_left,
            echo_volume_fade_add_right: u16;
        pub echo_volume_fade_ticks, echo_volume_fade_target_left, echo_volume_fade_target_right: u8;
        pub p_echoBuffer, memset_ptr: u16;
        pub memset_ctr, disable_sfx2, byte_4B1: u8;
        pub channel: [Channel; 8];
        pub sfx1, sfx2, sfx3: SfxRegs;
        pub sfx_chans_1: [SfxChan; 4];
        pub sfx_chans_2, sfx_chans_3: [SfxChan; 2];
        pub last_written_edl: u8;
        pub input_ports: [u8; 4];
        pub ram: *mut u8;
    }

    pub struct Timer {
        pub cycles, divider, target, counter: u8;
        pub enabled: bool;
    }

    pub struct WindowLayer {
        pub window1enabled, window2enabled, window1inversed, window2inversed: bool;
        pub maskLogic: u8;
    }

}

crate::typed_struct! {
    [#[derive(Copy, Clone)] #[repr(C)]];
    pub struct Config {
        pub window_width, window_height: i32;
        pub enhanced_mode7, new_renderer, ignore_aspect_ratio: bool;
        pub fullscreen, window_scale: u8;
        pub enable_audio, linear_filtering: bool;
        pub output_method: u8;
        pub audio_freq: u16;
        pub audio_channels: u8;
        pub audio_samples: u16;
        pub autosave: bool;
        pub extended_aspect_ratio: u8;
        pub extend_y, no_sprite_limits, display_perf_title: bool;
        pub enable_msu: u8;
        pub resume_msu, disable_frame_delay: bool;
        pub msuvolume: u8;
        pub features0: u32;
        pub link_graphics: *const c_char;
        pub memory_buffer: *mut c_char;
        pub shader, msu_path: *const c_char;
    }

    pub struct GlTextureWithSize {
        pub gl_texture: u32;
        pub width, height: u16;
    }

    pub struct RendererFuncs {
        pub Initialize: Option<unsafe extern "C-unwind" fn(*mut SDL_Window) -> bool>;
        pub Destroy: Option<unsafe extern "C-unwind" fn() -> ()>;
        pub BeginDraw: Option<unsafe extern "C-unwind" fn(i32, i32, *mut *mut u8, *mut i32) -> ()>;
        pub EndDraw: Option<unsafe extern "C-unwind" fn() -> ()>;
    }
}

pub use crate::game::constants::{
    Buttons, Consts8B, Consts8C, Consts8D, Consts8F, Consts9B, Consts60, Consts81, Consts82,
    Consts83, Consts84, Consts86, Consts87, Consts88, Consts89, Consts90, Consts91, Consts94,
    ConstsA0, ConstsA2, ConstsA3, ConstsA4, ConstsA5, ConstsA6, ConstsA7, ConstsA8, ConstsA9,
    ConstsAA, ConstsAD, ConstsB2, ConstsB3, ConstsB4, ConstsB7, EnemyProps, GameState,
    LoadingGameState, ProjectileType, SamusMovementType, SamusPose,
};

crate::type_aliases! {
    pub DspReg, EnemyAiBits, GLSLScaleType, GLbitfield, GLenum,
        GLuint, LiquidPhysicsType, RunMode, SDL_GLattr,
        SDL_HitTestResult, SDL_Keymod, SDL_Scancode, SDL_bool, SnesRegs, Uint32, __mode_t, ogl_LoadStatus, stbi__uint32, KeyAction, OutputMethod,
        SaveLoadOperation = u32;
}

crate::type_aliases! {
    pub CoroutineRet, Uint8 = u8;
}

crate::type_aliases! {
    pub FuncXY_V, Func_CalcHdma, Func_SpazerSba_V = unsafe fn(u16, u16, &mut Game) -> ();
}

crate::type_aliases! {
    pub Func_U8, Func_V_A = unsafe fn(&mut Game) -> u8;
}

crate::type_aliases! {
    pub Func_V, HandlerFunc, SyncAllFunc = unsafe fn(&mut Game) -> ();
}

pub type Func_Y_V = unsafe fn(u16, &mut Game) -> ();

pub type Func_Y_Y = unsafe fn(u16, &mut Game) -> u16;

crate::type_aliases! {
    pub GLboolean, GLubyte, stbi_uc = c_uchar;
}

crate::type_aliases! {
    pub GLint, GLsizei, SDL_JoystickPowerLevel, Sint32, GamepadButton = i32;
}

crate::type_aliases! {
    pub GLint64, Sint64 = i64;
}

crate::type_aliases! {
    pub GLintptr, GLsizeiptr = ptrdiff_t;
}

crate::type_aliases! {
    pub GLushort, stbi_us = c_ushort;
}

pub type RunFrameFunc = unsafe fn(u16, i32, &mut Game) -> ();

crate::type_aliases! {
    pub SDL_FingerID, SDL_GestureID, SDL_TouchID = Sint64;
}

pub type SDL_GLContext = *mut c_void;

crate::type_aliases! {
    pub SDL_JoystickID, SDL_Keycode = Sint32;
}

pub type Sint16 = i16;

crate::type_aliases! {
    pub __off64_t, __off_t, __time_t = c_long;
}

pub type ptrdiff_t = isize;

pub type size_t = usize;

pub type time_t = __time_t;

pub type CheckEnemyColl_Result = CheckSolidEnemyColl_Result;

pub type Enemy_Botwoon = sm_game::actors::layouts::EnemyLayout104;

impl SpawnHardcodedPlmArgs {
    pub const fn new(block_x: u8, block_y: u8, header: VoidP) -> Self {
        Self {
            block_x,
            block_y,
            header,
        }
    }
}

pub use sm_game::display::hdma_spawn::HdmaObjectDefinition;

impl TileTables {}

#[repr(C)]
pub struct _IO_FILE {
    _private: [u8; 0],
}

pub type FILE = _IO_FILE;

#[cfg(all(feature = "frontend", not(sm_verify_dll)))]
pub use sdl2_sys::SDL_Window;

#[cfg(any(not(feature = "frontend"), sm_verify_dll))]
#[repr(C)]
pub struct SDL_Window {
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

// Declare immutable string-pointer tables, retaining literal bytes and null slots.
macro_rules! immutable_c_strings {
    (@pointer None) => { ::core::ptr::null::<::core::ffi::c_char>() };
    (@pointer $bytes:literal) => { $bytes as *const u8 as *const ::core::ffi::c_char };
    ($($entry:tt),* $(,)?) => {
        unsafe {
            $crate::types::ImmutableCStringPointers::new([
                $($crate::types::immutable_c_strings!(@pointer $entry)),*
            ])
        }
    };
}
pub(crate) use immutable_c_strings;

crate::typed_struct! {
    #[derive(Clone)]
    #[repr(C)]
    pub struct Cart {
        pub type_0: u8;
        pub rom: crate::rom::RomImage;
        pub romSize: u32;
        pub ram: crate::save_ram::SaveRam;
        pub ramSize: u32;
    }
}

// The SPC dependency treats this pointer as opaque and uses the Rust bus calls.
// RAM has its own stable allocation; register snapshots never copy its owner.
pub struct Apu {
    pub spc: *mut Spc,
    pub dsp: *mut Dsp,
    pub ram: crate::apu_ram::ApuRam,
    pub registers: ApuRegisters,
}

// The C music interface sees only the first field. RAM ownership stays in Rust.
#[repr(C)]
pub struct SpcPlayer {
    pub state: NativePlayerState,
    pub ram: crate::apu_ram::ApuRam,
}
const _: () = assert!(core::mem::offset_of!(SpcPlayer, state) == 0);

// The backend sees the state prefix; its RAM allocation stays owned in Rust.
#[repr(C)]
pub struct Dsp {
    pub state: DspState,
    pub(crate) apu_memory: crate::apu_ram::ApuRam,
}
const _: () = assert!(core::mem::offset_of!(Dsp, state) == 0);

// EnemyBase, now in sm_game, must remain the prefix of EnemyData.
macro_rules! assert_enemy_base_prefix {
    ($($field:ident),+) => {
        $(const _: () = assert!(
            ::core::mem::offset_of!(EnemyBase, $field) == ::core::mem::offset_of!(EnemyData, $field)
        );)+
        const _: () = assert!(::core::mem::size_of::<EnemyBase>() == ::core::mem::offset_of!(EnemyData, ai_var_a));
    };
}
assert_enemy_base_prefix!(
    enemy_ptr,
    x_pos,
    x_subpos,
    y_pos,
    y_subpos,
    x_width,
    y_height,
    properties,
    extra_properties,
    ai_handler_bits,
    health,
    spritemap_pointer,
    timer,
    current_instruction,
    instruction_timer,
    palette_index,
    vram_tiles_index,
    layer,
    flash_timer,
    frozen_timer,
    invincibility_timer,
    shake_timer,
    frame_counter,
    bank,
    field_2f
);
