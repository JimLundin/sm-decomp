extern "C" {
    pub type _IO_wide_data;
    pub type _IO_codecvt;
    pub type _IO_marker;
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn Die(error: *const ::core::ffi::c_char);
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    static mut stderr: *mut FILE;
    fn rename(
        __old: *const ::core::ffi::c_char,
        __new: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn fclose(__stream: *mut FILE) -> ::core::ffi::c_int;
    fn fopen(
        __filename: *const ::core::ffi::c_char,
        __modes: *const ::core::ffi::c_char,
    ) -> *mut FILE;
    fn fprintf(
        __stream: *mut FILE,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn printf(__format: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    fn sprintf(
        __s: *mut ::core::ffi::c_char,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn fread(
        __ptr: *mut ::core::ffi::c_void,
        __size: size_t,
        __n: size_t,
        __stream: *mut FILE,
    ) -> ::core::ffi::c_ulong;
    fn fwrite(
        __ptr: *const ::core::ffi::c_void,
        __size: size_t,
        __n: size_t,
        __s: *mut FILE,
    ) -> ::core::ffi::c_ulong;
    fn fseek(
        __stream: *mut FILE,
        __off: ::core::ffi::c_long,
        __whence: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn ftell(__stream: *mut FILE) -> ::core::ffi::c_long;
    static mut snes_frame_counter: ::core::ffi::c_int;
    static mut g_use_my_apu_code: bool;
    static mut g_debug_flag: bool;
    fn RtlApuLock();
    fn RtlApuUnlock();
    fn RtlUpdateSnesPatchForBugfix();
    static mut currently_installed_bug_fix_counter: uint16;
    static mut g_spc_player: *mut SpcPlayer;
    fn dsp_getSamples(dsp: *mut Dsp, sampleData: *mut int16_t, samplesPerFrame: ::core::ffi::c_int);
    fn apu_cycle(apu: *mut Apu);
    fn ppu_copy(ppu: *mut Ppu, ppu_src: *mut Ppu);
    fn snes_reset(snes: *mut Snes, hard: bool);
    fn snes_read(snes: *mut Snes, adr: uint32_t) -> uint8_t;
    fn snes_write(snes: *mut Snes, adr: uint32_t, val: uint8_t);
    fn snes_saveload(snes: *mut Snes, func: Option<SaveLoadFunc>, ctx: *mut ::core::ffi::c_void);
    fn snes_catchupApu(snes: *mut Snes);
    static mut g_snes: *mut Snes;
    static mut g_fail: bool;
    fn SpcPlayer_GenerateSamples(p: *mut SpcPlayer);
    fn SpcPlayer_Initialize(p: *mut SpcPlayer);
    fn SpcPlayer_Upload(p: *mut SpcPlayer, data: *const uint8_t);
    fn SpcPlayer_CopyVariablesFromRam(p: *mut SpcPlayer);
    fn SpcPlayer_CopyVariablesToRam(p: *mut SpcPlayer);
    fn ByteArray_Resize(arr: *mut ByteArray, new_size: size_t);
    fn ByteArray_Destroy(arr: *mut ByteArray);
    fn ByteArray_AppendData(arr: *mut ByteArray, data: *const uint8, data_size: size_t);
    fn ByteArray_AppendByte(arr: *mut ByteArray, v: uint8);
}
pub type __int8_t = i8;
pub type __uint8_t = u8;
pub type __int16_t = i16;
pub type __uint16_t = u16;
pub type __int32_t = i32;
pub type __uint32_t = u32;
pub type __off_t = ::core::ffi::c_long;
pub type __off64_t = ::core::ffi::c_long;
pub type int8_t = __int8_t;
pub type int16_t = __int16_t;
pub type int32_t = __int32_t;
pub type uint8_t = __uint8_t;
pub type uint16_t = __uint16_t;
pub type uint32_t = __uint32_t;
pub type size_t = usize;
pub type uint8 = uint8_t;
pub type uint16 = uint16_t;
pub type int16 = int16_t;
pub type uint32 = uint32_t;
pub type VoidP = uint16;
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct LongPtr {
    pub addr: VoidP,
    pub bank: uint8,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PairU16 {
    pub k: uint16,
    pub j: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _IO_FILE {
    pub _flags: ::core::ffi::c_int,
    pub _IO_read_ptr: *mut ::core::ffi::c_char,
    pub _IO_read_end: *mut ::core::ffi::c_char,
    pub _IO_read_base: *mut ::core::ffi::c_char,
    pub _IO_write_base: *mut ::core::ffi::c_char,
    pub _IO_write_ptr: *mut ::core::ffi::c_char,
    pub _IO_write_end: *mut ::core::ffi::c_char,
    pub _IO_buf_base: *mut ::core::ffi::c_char,
    pub _IO_buf_end: *mut ::core::ffi::c_char,
    pub _IO_save_base: *mut ::core::ffi::c_char,
    pub _IO_backup_base: *mut ::core::ffi::c_char,
    pub _IO_save_end: *mut ::core::ffi::c_char,
    pub _markers: *mut _IO_marker,
    pub _chain: *mut _IO_FILE,
    pub _fileno: ::core::ffi::c_int,
    pub _flags2: ::core::ffi::c_int,
    pub _old_offset: __off_t,
    pub _cur_column: ::core::ffi::c_ushort,
    pub _vtable_offset: ::core::ffi::c_schar,
    pub _shortbuf: [::core::ffi::c_char; 1],
    pub _lock: *mut ::core::ffi::c_void,
    pub _offset: __off64_t,
    pub _codecvt: *mut _IO_codecvt,
    pub _wide_data: *mut _IO_wide_data,
    pub _freeres_list: *mut _IO_FILE,
    pub _freeres_buf: *mut ::core::ffi::c_void,
    pub __pad5: size_t,
    pub _mode: ::core::ffi::c_int,
    pub _unused2: [::core::ffi::c_char; 20],
}
pub type _IO_lock_t = ();
pub type FILE = _IO_FILE;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ApuWriteEnt {
    pub ports: [uint8; 4],
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
pub struct Input {
    pub snes: *mut Snes,
    pub type_0: uint8_t,
    pub currentState: uint16_t,
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
pub type PpuZbufType = uint16_t;
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
pub struct Layer {
    pub mainScreenEnabled: bool,
    pub subScreenEnabled: bool,
    pub mainScreenWindowed: bool,
    pub subScreenWindowed: bool,
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
    pub c2rust_unnamed: C2RustUnnamed,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed {
    pub hist: DspRegWriteHistory,
    pub padpad: *mut ::core::ffi::c_void,
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
pub struct Timer {
    pub cycles: uint8_t,
    pub divider: uint8_t,
    pub target: uint8_t,
    pub counter: uint8_t,
    pub enabled: bool,
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
pub const APUI03: SnesRegs = 8515;
pub const APUI00: SnesRegs = 8512;
pub type RunFrameFunc = unsafe extern "C" fn(uint16, ::core::ffi::c_int) -> ();
pub type SyncAllFunc = unsafe extern "C" fn() -> ();
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StateRecorder {
    pub last_inputs: uint16,
    pub frames_since_last: uint32,
    pub total_frames: uint32,
    pub replay_pos: uint32,
    pub replay_pos_last_complete: uint32,
    pub replay_frame_counter: uint32,
    pub replay_next_cmd_at: uint32,
    pub snapshot_flags: uint32,
    pub replay_cmd: uint8,
    pub replay_mode: bool,
    pub log: ByteArray,
    pub base_snapshot: ByteArray,
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
pub type SaveLoadFunc =
    unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut ::core::ffi::c_void, size_t) -> ();
pub type C2RustUnnamed_0 = ::core::ffi::c_uint;
pub const kSaveLoad_Replay: C2RustUnnamed_0 = 3;
pub const kSaveLoad_Load: C2RustUnnamed_0 = 2;
pub const kSaveLoad_Save: C2RustUnnamed_0 = 1;
pub const kApuMaxQueueSize: C2RustUnnamed_2 = 16;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct LoadFuncState {
    pub p: *mut uint8,
    pub pend: *mut uint8,
}
pub const kCurrentBugFixCounter: C2RustUnnamed_1 = 1;
pub type SnesRegs = ::core::ffi::c_uint;
pub const MIRR7: SnesRegs = 17279;
pub const UNUSED7: SnesRegs = 17275;
pub const NTRL7: SnesRegs = 17274;
pub const A2A7H: SnesRegs = 17273;
pub const A2A7L: SnesRegs = 17272;
pub const DAS70: SnesRegs = 17271;
pub const DAS7H: SnesRegs = 17270;
pub const DAS7L: SnesRegs = 17269;
pub const A1B7: SnesRegs = 17268;
pub const A1T7H: SnesRegs = 17267;
pub const A1T7L: SnesRegs = 17266;
pub const BBAD7: SnesRegs = 17265;
pub const DMAP7: SnesRegs = 17264;
pub const MIRR6: SnesRegs = 17263;
pub const UNUSED6: SnesRegs = 17259;
pub const NTRL6: SnesRegs = 17258;
pub const A2A6H: SnesRegs = 17257;
pub const A2A6L: SnesRegs = 17256;
pub const DAS60: SnesRegs = 17255;
pub const DAS6H: SnesRegs = 17254;
pub const DAS6L: SnesRegs = 17253;
pub const A1B6: SnesRegs = 17252;
pub const A1T6H: SnesRegs = 17251;
pub const A1T6L: SnesRegs = 17250;
pub const BBAD6: SnesRegs = 17249;
pub const DMAP6: SnesRegs = 17248;
pub const MIRR5: SnesRegs = 17247;
pub const UNUSED5: SnesRegs = 17243;
pub const NTRL5: SnesRegs = 17242;
pub const A2A5H: SnesRegs = 17241;
pub const A2A5L: SnesRegs = 17240;
pub const DAS50: SnesRegs = 17239;
pub const DAS5H: SnesRegs = 17238;
pub const DAS5L: SnesRegs = 17237;
pub const A1B5: SnesRegs = 17236;
pub const A1T5H: SnesRegs = 17235;
pub const A1T5L: SnesRegs = 17234;
pub const BBAD5: SnesRegs = 17233;
pub const DMAP5: SnesRegs = 17232;
pub const MIRR4: SnesRegs = 17231;
pub const UNUSED4: SnesRegs = 17227;
pub const NTRL4: SnesRegs = 17226;
pub const A2A4H: SnesRegs = 17225;
pub const A2A4L: SnesRegs = 17224;
pub const DAS40: SnesRegs = 17223;
pub const DAS4H: SnesRegs = 17222;
pub const DAS4L: SnesRegs = 17221;
pub const A1B4: SnesRegs = 17220;
pub const A1T4H: SnesRegs = 17219;
pub const A1T4L: SnesRegs = 17218;
pub const BBAD4: SnesRegs = 17217;
pub const DMAP4: SnesRegs = 17216;
pub const MIRR3: SnesRegs = 17215;
pub const UNUSED3: SnesRegs = 17211;
pub const NTRL3: SnesRegs = 17210;
pub const A2A3H: SnesRegs = 17209;
pub const A2A3L: SnesRegs = 17208;
pub const DAS30: SnesRegs = 17207;
pub const DAS3H: SnesRegs = 17206;
pub const DAS3L: SnesRegs = 17205;
pub const A1B3: SnesRegs = 17204;
pub const A1T3H: SnesRegs = 17203;
pub const A1T3L: SnesRegs = 17202;
pub const BBAD3: SnesRegs = 17201;
pub const DMAP3: SnesRegs = 17200;
pub const MIRR2: SnesRegs = 17199;
pub const UNUSED2: SnesRegs = 17195;
pub const NTRL2: SnesRegs = 17194;
pub const A2A2H: SnesRegs = 17193;
pub const A2A2L: SnesRegs = 17192;
pub const DAS20: SnesRegs = 17191;
pub const DAS2H: SnesRegs = 17190;
pub const DAS2L: SnesRegs = 17189;
pub const A1B2: SnesRegs = 17188;
pub const A1T2H: SnesRegs = 17187;
pub const A1T2L: SnesRegs = 17186;
pub const BBAD2: SnesRegs = 17185;
pub const DMAP2: SnesRegs = 17184;
pub const MIRR1: SnesRegs = 17183;
pub const UNUSED1: SnesRegs = 17179;
pub const NTRL1: SnesRegs = 17178;
pub const A2A1H: SnesRegs = 17177;
pub const A2A1L: SnesRegs = 17176;
pub const DAS10: SnesRegs = 17175;
pub const DAS1H: SnesRegs = 17174;
pub const DAS1L: SnesRegs = 17173;
pub const A1B1: SnesRegs = 17172;
pub const A1T1H: SnesRegs = 17171;
pub const A1T1L: SnesRegs = 17170;
pub const BBAD1: SnesRegs = 17169;
pub const DMAP1: SnesRegs = 17168;
pub const MIRR0: SnesRegs = 17167;
pub const UNUSED0: SnesRegs = 17163;
pub const NTRL0: SnesRegs = 17162;
pub const A2A0H: SnesRegs = 17161;
pub const A2A0L: SnesRegs = 17160;
pub const DAS00: SnesRegs = 17159;
pub const DAS0H: SnesRegs = 17158;
pub const DAS0L: SnesRegs = 17157;
pub const A1B0: SnesRegs = 17156;
pub const A1T0H: SnesRegs = 17155;
pub const A1T0L: SnesRegs = 17154;
pub const BBAD0: SnesRegs = 17153;
pub const DMAP0: SnesRegs = 17152;
pub const JOY4H: SnesRegs = 16927;
pub const JOY4L: SnesRegs = 16926;
pub const JOY3H: SnesRegs = 16925;
pub const JOY3L: SnesRegs = 16924;
pub const JOY2H: SnesRegs = 16923;
pub const JOY2L: SnesRegs = 16922;
pub const JOY1H: SnesRegs = 16921;
pub const JOY1L: SnesRegs = 16920;
pub const RDMPYH: SnesRegs = 16919;
pub const RDMPYL: SnesRegs = 16918;
pub const RDDIVH: SnesRegs = 16917;
pub const RDDIVL: SnesRegs = 16916;
pub const RDIO: SnesRegs = 16915;
pub const HVBJOY: SnesRegs = 16914;
pub const TIMEUP: SnesRegs = 16913;
pub const RDNMI: SnesRegs = 16912;
pub const MEMSEL: SnesRegs = 16909;
pub const HDMAEN: SnesRegs = 16908;
pub const MDMAEN: SnesRegs = 16907;
pub const VTIMEH: SnesRegs = 16906;
pub const VTIMEL: SnesRegs = 16905;
pub const HTIMEH: SnesRegs = 16904;
pub const HTIMEL: SnesRegs = 16903;
pub const WRDIVB: SnesRegs = 16902;
pub const WRDIVH: SnesRegs = 16901;
pub const WRDIVL: SnesRegs = 16900;
pub const WRMPYB: SnesRegs = 16899;
pub const WRMPYA: SnesRegs = 16898;
pub const WRIO: SnesRegs = 16897;
pub const NMITIMEN: SnesRegs = 16896;
pub const JOYB: SnesRegs = 16407;
pub const JOYA: SnesRegs = 16406;
pub const WMADDH: SnesRegs = 8579;
pub const WMADDM: SnesRegs = 8578;
pub const WMADDL: SnesRegs = 8577;
pub const WMDATA: SnesRegs = 8576;
pub const APUI02: SnesRegs = 8514;
pub const APUI01: SnesRegs = 8513;
pub const STAT78: SnesRegs = 8511;
pub const STAT77: SnesRegs = 8510;
pub const OPVCT: SnesRegs = 8509;
pub const OPHCT: SnesRegs = 8508;
pub const RDCGRAM: SnesRegs = 8507;
pub const RDVRAMH: SnesRegs = 8506;
pub const RDVRAML: SnesRegs = 8505;
pub const RDOAM: SnesRegs = 8504;
pub const SLHV: SnesRegs = 8503;
pub const MPYH: SnesRegs = 8502;
pub const MPYM: SnesRegs = 8501;
pub const MPYL: SnesRegs = 8500;
pub const SETINI: SnesRegs = 8499;
pub const COLDATA: SnesRegs = 8498;
pub const CGADSUB: SnesRegs = 8497;
pub const CGWSEL: SnesRegs = 8496;
pub const TSW: SnesRegs = 8495;
pub const TMW: SnesRegs = 8494;
pub const TS: SnesRegs = 8493;
pub const TM: SnesRegs = 8492;
pub const WOBJLOG: SnesRegs = 8491;
pub const WBGLOG: SnesRegs = 8490;
pub const WH3: SnesRegs = 8489;
pub const WH2: SnesRegs = 8488;
pub const WH1: SnesRegs = 8487;
pub const WH0: SnesRegs = 8486;
pub const WOBJSEL: SnesRegs = 8485;
pub const W34SEL: SnesRegs = 8484;
pub const W12SEL: SnesRegs = 8483;
pub const CGDATA: SnesRegs = 8482;
pub const CGADD: SnesRegs = 8481;
pub const M7Y: SnesRegs = 8480;
pub const M7X: SnesRegs = 8479;
pub const M7D: SnesRegs = 8478;
pub const M7C: SnesRegs = 8477;
pub const M7B: SnesRegs = 8476;
pub const M7A: SnesRegs = 8475;
pub const M7SEL: SnesRegs = 8474;
pub const VMDATAH: SnesRegs = 8473;
pub const VMDATAL: SnesRegs = 8472;
pub const VMADDH: SnesRegs = 8471;
pub const VMADDL: SnesRegs = 8470;
pub const VMAIN: SnesRegs = 8469;
pub const BG4VOFS: SnesRegs = 8468;
pub const BG4HOFS: SnesRegs = 8467;
pub const BG3VOFS: SnesRegs = 8466;
pub const BG3HOFS: SnesRegs = 8465;
pub const BG2VOFS: SnesRegs = 8464;
pub const BG2HOFS: SnesRegs = 8463;
pub const BG1VOFS: SnesRegs = 8462;
pub const BG1HOFS: SnesRegs = 8461;
pub const BG34NBA: SnesRegs = 8460;
pub const BG12NBA: SnesRegs = 8459;
pub const BG4SC: SnesRegs = 8458;
pub const BG3SC: SnesRegs = 8457;
pub const BG2SC: SnesRegs = 8456;
pub const BG1SC: SnesRegs = 8455;
pub const MOSAIC: SnesRegs = 8454;
pub const BGMODE: SnesRegs = 8453;
pub const OAMDATA: SnesRegs = 8452;
pub const OAMADDH: SnesRegs = 8451;
pub const OAMADDL: SnesRegs = 8450;
pub const OBSEL: SnesRegs = 8449;
pub const INIDISP: SnesRegs = 8448;
pub type C2RustUnnamed_1 = ::core::ffi::c_uint;
pub type C2RustUnnamed_2 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SEEK_SET: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SEEK_END: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
#[no_mangle]
pub static mut g_ram: [uint8; 131072] = [0; 131072];
#[no_mangle]
pub static mut g_sram: *mut uint8 = ::core::ptr::null::<uint8>() as *mut uint8;
#[no_mangle]
pub static mut g_rom: *const uint8 = ::core::ptr::null::<uint8>();
static mut g_rtl_memory_ptr: *mut uint8 = ::core::ptr::null::<uint8>() as *mut uint8;
static mut g_rtl_runframe: Option<RunFrameFunc> = None;
static mut g_rtl_syncall: Option<SyncAllFunc> = None;
#[no_mangle]
pub unsafe extern "C" fn RtlSetupEmuCallbacks(
    mut emu_ram: *mut uint8,
    mut func: Option<RunFrameFunc>,
    mut sync_all: Option<SyncAllFunc>,
) {
    g_rtl_memory_ptr = emu_ram;
    g_rtl_runframe = func;
    g_rtl_syncall = sync_all;
}
unsafe extern "C" fn RtlSynchronizeWholeState() {
    if g_rtl_syncall.is_some() {
        ::core::mem::transmute::<_, fn()>(g_rtl_syncall.expect("non-null function pointer"))();
    }
}
unsafe extern "C" fn RtlSyncMemoryRegion(mut ptr: *mut ::core::ffi::c_void, mut n: size_t) {
    let mut data: *mut uint8 = ptr as *mut uint8;
    '_c2rust_label: {
        if data >= &raw mut g_ram as *mut uint8
            && data < (&raw mut g_ram as *mut uint8).offset(0x20000 as ::core::ffi::c_int as isize)
        {
        } else {
            __assert_fail(
                b"data >= g_ram && data < g_ram + 0x20000\0" as *const u8
                    as *const ::core::ffi::c_char,
                b"src/sm_rtl.c\0" as *const u8 as *const ::core::ffi::c_char,
                38 as ::core::ffi::c_uint,
                b"void RtlSyncMemoryRegion(void *, size_t)\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if !g_rtl_memory_ptr.is_null() {
        memcpy(
            g_rtl_memory_ptr.offset(
                data.offset_from(&raw mut g_ram as *mut uint8) as ::core::ffi::c_long as isize
            ) as *mut ::core::ffi::c_void,
            data as *const ::core::ffi::c_void,
            n,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn ByteArray_AppendVl(mut arr: *mut ByteArray, mut v: uint32) {
    while v >= 255 as uint32 {
        ByteArray_AppendByte(arr, 255 as uint8);
        v = v.wrapping_sub(255 as uint32);
    }
    ByteArray_AppendByte(arr, v as uint8);
}
#[no_mangle]
pub unsafe extern "C" fn saveFunc(
    mut ctx_in: *mut ::core::ffi::c_void,
    mut data: *mut ::core::ffi::c_void,
    mut data_size: size_t,
) {
    ByteArray_AppendData(ctx_in as *mut ByteArray, data as *mut uint8, data_size);
}
#[no_mangle]
pub unsafe extern "C" fn loadFunc(
    mut ctx: *mut ::core::ffi::c_void,
    mut data: *mut ::core::ffi::c_void,
    mut data_size: size_t,
) {
    let mut st: *mut LoadFuncState = ctx as *mut LoadFuncState;
    '_c2rust_label: {
        if (*st).pend.offset_from((*st).p) as ::core::ffi::c_long as size_t >= data_size {
        } else {
            __assert_fail(
                b"(size_t)(st->pend - st->p) >= data_size\0" as *const u8
                    as *const ::core::ffi::c_char,
                b"src/sm_rtl.c\0" as *const u8 as *const ::core::ffi::c_char,
                59 as ::core::ffi::c_uint,
                b"void loadFunc(void *, void *, size_t)\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
    };
    memcpy(data, (*st).p as *const ::core::ffi::c_void, data_size);
    (*st).p = (*st).p.offset(data_size as isize);
}
unsafe extern "C" fn LoadSnesState(
    mut func: Option<SaveLoadFunc>,
    mut ctx: *mut ::core::ffi::c_void,
) {
    snes_saveload(g_snes as *mut Snes, func, ctx);
    (*(*g_snes).cpu).e = false_0 != 0;
    let mut next: uint32 =
        ((*(*g_snes)
            .ram
            .offset(((*(*g_snes).cpu).sp as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            | (*(*g_snes).ram.offset(
                ((*(*g_snes).cpu).sp as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as isize,
            ) as ::core::ffi::c_int)
                << 8 as ::core::ffi::c_int
            | (*(*g_snes).ram.offset(
                ((*(*g_snes).cpu).sp as ::core::ffi::c_int + 5 as ::core::ffi::c_int) as isize,
            ) as ::core::ffi::c_int)
                << 16 as ::core::ffi::c_int)
            + 1 as ::core::ffi::c_int) as uint32;
    if next == 0x82897e as uint32 {
        *(*g_snes).ram.offset(
            ((*(*g_snes).cpu).sp as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize,
        ) = (0xf71b as ::core::ffi::c_int - 1 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int)
            as uint8_t;
        *(*g_snes).ram.offset(
            ((*(*g_snes).cpu).sp as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as isize,
        ) = (0xf71b as ::core::ffi::c_int - 1 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int)
            as uint8_t;
    }
    RtlSynchronizeWholeState();
}
unsafe extern "C" fn SaveSnesState(
    mut func: Option<SaveLoadFunc>,
    mut ctx: *mut ::core::ffi::c_void,
) {
    snes_saveload(g_snes as *mut Snes, func, ctx);
}
static mut state_recorder: StateRecorder = StateRecorder {
    last_inputs: 0,
    frames_since_last: 0,
    total_frames: 0,
    replay_pos: 0,
    replay_pos_last_complete: 0,
    replay_frame_counter: 0,
    replay_next_cmd_at: 0,
    snapshot_flags: 0,
    replay_cmd: 0,
    replay_mode: false,
    log: ByteArray {
        data: ::core::ptr::null::<uint8>() as *mut uint8,
        size: 0,
        capacity: 0,
    },
    base_snapshot: ByteArray {
        data: ::core::ptr::null::<uint8>() as *mut uint8,
        size: 0,
        capacity: 0,
    },
};
#[no_mangle]
pub unsafe extern "C" fn StateRecorder_Init(mut sr: *mut StateRecorder) {
    ByteArray_Destroy(&raw mut (*sr).log);
    ByteArray_Destroy(&raw mut (*sr).base_snapshot);
    memset(
        sr as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<StateRecorder>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn StateRecorder_RecordCmd(mut sr: *mut StateRecorder, mut cmd: uint8) {
    let mut frames: ::core::ffi::c_int = (*sr).frames_since_last as ::core::ffi::c_int;
    (*sr).frames_since_last = 0 as uint32;
    let mut x: ::core::ffi::c_int = if (cmd as ::core::ffi::c_int) < 0xc0 as ::core::ffi::c_int {
        0xf as ::core::ffi::c_int
    } else {
        0x1 as ::core::ffi::c_int
    };
    ByteArray_AppendByte(
        &raw mut (*sr).log,
        (cmd as ::core::ffi::c_int | (if frames < x { frames } else { x })) as uint8,
    );
    if frames >= x {
        ByteArray_AppendVl(&raw mut (*sr).log, (frames - x) as uint32);
    }
}
#[no_mangle]
pub unsafe extern "C" fn StateRecorder_Record(mut sr: *mut StateRecorder, mut inputs: uint16) {
    let mut diff: uint16 =
        (inputs as ::core::ffi::c_int ^ (*sr).last_inputs as ::core::ffi::c_int) as uint16;
    if diff as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        (*sr).last_inputs = inputs;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < 12 as ::core::ffi::c_int {
            if diff as ::core::ffi::c_int >> i & 1 as ::core::ffi::c_int != 0 {
                StateRecorder_RecordCmd(sr, (i << 4 as ::core::ffi::c_int) as uint8);
            }
            i += 1;
        }
    }
    (*sr).frames_since_last = (*sr).frames_since_last.wrapping_add(1);
    (*sr).total_frames = (*sr).total_frames.wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn StateRecorder_RecordPatchByte(
    mut sr: *mut StateRecorder,
    mut addr: uint32,
    mut value: *const uint8,
    mut num: ::core::ffi::c_int,
) {
    '_c2rust_label: {
        if addr < 0x20000 as uint32 {
        } else {
            __assert_fail(
                b"addr < 0x20000\0" as *const u8 as *const ::core::ffi::c_char,
                b"src/sm_rtl.c\0" as *const u8 as *const ::core::ffi::c_char,
                133 as ::core::ffi::c_uint,
                b"void StateRecorder_RecordPatchByte(StateRecorder *, uint32, const uint8 *, int)\0"
                    as *const u8 as *const ::core::ffi::c_char,
            );
        }
    };
    let mut lb: size_t = (*sr).log.size;
    let mut lq: ::core::ffi::c_int = if num - 1 as ::core::ffi::c_int <= 3 as ::core::ffi::c_int {
        num - 1 as ::core::ffi::c_int
    } else {
        3 as ::core::ffi::c_int
    };
    StateRecorder_RecordCmd(
        sr,
        (0xc0 as ::core::ffi::c_int
            | (if addr & 0x10000 as uint32 != 0 {
                2 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            })
            | lq << 2 as ::core::ffi::c_int) as uint8,
    );
    if lq == 3 as ::core::ffi::c_int {
        ByteArray_AppendVl(
            &raw mut (*sr).log,
            (num - 1 as ::core::ffi::c_int - 3 as ::core::ffi::c_int) as uint32,
        );
    }
    ByteArray_AppendByte(
        &raw mut (*sr).log,
        (addr >> 8 as ::core::ffi::c_int) as uint8,
    );
    ByteArray_AppendByte(&raw mut (*sr).log, addr as uint8);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < num {
        ByteArray_AppendByte(&raw mut (*sr).log, *value.offset(i as isize));
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn ReadFromFile(
    mut f: *mut FILE,
    mut data: *mut ::core::ffi::c_void,
    mut n: size_t,
) {
    if fread(data, 1 as size_t, n, f) as size_t != n {
        Die(b"fread failed\n\0" as *const u8 as *const ::core::ffi::c_char);
    }
}
#[no_mangle]
pub unsafe extern "C" fn RtlReset(mut mode: ::core::ffi::c_int) {
    snes_frame_counter = 0 as ::core::ffi::c_int;
    snes_reset(g_snes as *mut Snes, true_0 != 0);
    if mode & 1 as ::core::ffi::c_int == 0 {
        memset(
            g_sram as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            0x2000 as size_t,
        );
    }
    *(&raw mut g_ram as *mut uint8).offset(0x611 as ::core::ffi::c_int as isize) = 1 as uint8;
    RtlApuLock();
    RtlRestoreMusicAfterLoad_Locked(true_0 != 0);
    RtlApuUnlock();
    RtlSynchronizeWholeState();
    if mode & 2 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        StateRecorder_Init(&raw mut state_recorder);
    }
}
#[no_mangle]
pub unsafe extern "C" fn GetFileSize(mut f: *mut FILE) -> ::core::ffi::c_int {
    fseek(f, 0 as ::core::ffi::c_long, SEEK_END);
    let mut r: ::core::ffi::c_int = ftell(f) as ::core::ffi::c_int;
    fseek(f, 0 as ::core::ffi::c_long, SEEK_SET);
    return r;
}
#[no_mangle]
pub unsafe extern "C" fn StateRecorder_Load(
    mut sr: *mut StateRecorder,
    mut f: *mut FILE,
    mut replay_mode: bool,
) {
    let mut hdr: [uint32; 16] = [
        0 as ::core::ffi::c_int as uint32,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    ];
    let mut is_old: bool = false_0 != 0;
    let mut is_reset: bool = false_0 != 0;
    ReadFromFile(
        f,
        &raw mut hdr as *mut uint32 as *mut ::core::ffi::c_void,
        (8 as size_t).wrapping_mul(::core::mem::size_of::<uint32>() as size_t),
    );
    if hdr[0 as ::core::ffi::c_int as usize] != 2 as uint32 {
        hdr[8 as ::core::ffi::c_int as usize] = hdr[7 as ::core::ffi::c_int as usize];
        hdr[7 as ::core::ffi::c_int as usize] =
            hdr[5 as ::core::ffi::c_int as usize] >> 1 as ::core::ffi::c_int;
        hdr[5 as ::core::ffi::c_int as usize] =
            if hdr[5 as ::core::ffi::c_int as usize] & 1 as uint32 != 0 {
                hdr[6 as ::core::ffi::c_int as usize]
            } else {
                0 as uint32
            };
    } else if hdr[0 as ::core::ffi::c_int as usize] == 2 as uint32 {
        ReadFromFile(
            f,
            (&raw mut hdr as *mut uint32).offset(8 as ::core::ffi::c_int as isize)
                as *mut ::core::ffi::c_void,
            (8 as size_t).wrapping_mul(::core::mem::size_of::<uint32>() as size_t),
        );
    } else {
        '_c2rust_label: {
            __assert_fail(
                b"0\0" as *const u8 as *const ::core::ffi::c_char,
                b"src/sm_rtl.c\0" as *const u8 as *const ::core::ffi::c_char,
                195 as ::core::ffi::c_uint,
                b"void StateRecorder_Load(StateRecorder *, FILE *, _Bool)\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        };
    }
    (*sr).total_frames = hdr[1 as ::core::ffi::c_int as usize];
    ByteArray_Resize(
        &raw mut (*sr).log,
        hdr[2 as ::core::ffi::c_int as usize] as size_t,
    );
    ReadFromFile(
        f,
        (*sr).log.data as *mut ::core::ffi::c_void,
        (*sr).log.size,
    );
    (*sr).last_inputs = hdr[3 as ::core::ffi::c_int as usize] as uint16;
    (*sr).frames_since_last = hdr[4 as ::core::ffi::c_int as usize];
    ByteArray_Resize(
        &raw mut (*sr).base_snapshot,
        hdr[5 as ::core::ffi::c_int as usize] as size_t,
    );
    ReadFromFile(
        f,
        (*sr).base_snapshot.data as *mut ::core::ffi::c_void,
        (*sr).base_snapshot.size,
    );
    (*sr).snapshot_flags = hdr[9 as ::core::ffi::c_int as usize];
    (*sr).replay_next_cmd_at = 0 as uint32;
    (*sr).replay_mode = replay_mode;
    if replay_mode {
        (*sr).frames_since_last = 0 as uint32;
        (*sr).last_inputs = 0 as uint16;
        (*sr).replay_pos_last_complete = 0 as uint32;
        (*sr).replay_pos = (*sr).replay_pos_last_complete;
        (*sr).replay_frame_counter = 0 as uint32;
        if (*sr).base_snapshot.size > 8192 as size_t {
            let mut state: LoadFuncState = LoadFuncState {
                p: (*sr).base_snapshot.data,
                pend: (*sr)
                    .base_snapshot
                    .data
                    .offset((*sr).base_snapshot.size as isize),
            };
            LoadSnesState(
                Some(
                    loadFunc
                        as unsafe extern "C" fn(
                            *mut ::core::ffi::c_void,
                            *mut ::core::ffi::c_void,
                            size_t,
                        ) -> (),
                ),
                &raw mut state as *mut ::core::ffi::c_void,
            );
            '_c2rust_label_0: {
                if state.p == state.pend {
                } else {
                    __assert_fail(
                        b"state.p == state.pend\0" as *const u8 as *const ::core::ffi::c_char,
                        b"src/sm_rtl.c\0" as *const u8 as *const ::core::ffi::c_char,
                        219 as ::core::ffi::c_uint,
                        b"void StateRecorder_Load(StateRecorder *, FILE *, _Bool)\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                }
            };
        } else {
            RtlReset(2 as ::core::ffi::c_int);
            if (*sr).base_snapshot.size == 8192 as size_t {
                memcpy(
                    g_sram as *mut ::core::ffi::c_void,
                    (*sr).base_snapshot.data as *const ::core::ffi::c_void,
                    8192 as size_t,
                );
            }
            is_reset = true_0 != 0;
        }
    } else {
        (*sr).replay_pos_last_complete = hdr[7 as ::core::ffi::c_int as usize];
        (*sr).replay_pos = (*sr).replay_pos_last_complete;
        (*sr).replay_frame_counter = hdr[8 as ::core::ffi::c_int as usize];
        (*sr).replay_mode = (*sr).replay_frame_counter != 0 as uint32;
        '_c2rust_label_1: {
            if hdr[6 as ::core::ffi::c_int as usize] == 275493 as uint32 {
            } else {
                __assert_fail(
                    b"hdr[6] == 275493\0" as *const u8 as *const ::core::ffi::c_char,
                    b"src/sm_rtl.c\0" as *const u8 as *const ::core::ffi::c_char,
                    232 as ::core::ffi::c_uint,
                    b"void StateRecorder_Load(StateRecorder *, FILE *, _Bool)\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
        };
        let mut arr: ByteArray = ByteArray {
            data: ::core::ptr::null_mut::<uint8>(),
            size: 0,
            capacity: 0,
        };
        ByteArray_Resize(
            &raw mut arr,
            hdr[6 as ::core::ffi::c_int as usize] as size_t,
        );
        ReadFromFile(f, arr.data as *mut ::core::ffi::c_void, arr.size);
        let mut state_0: LoadFuncState = LoadFuncState {
            p: arr.data,
            pend: arr.data.offset(arr.size as isize),
        };
        LoadSnesState(
            Some(
                loadFunc
                    as unsafe extern "C" fn(
                        *mut ::core::ffi::c_void,
                        *mut ::core::ffi::c_void,
                        size_t,
                    ) -> (),
            ),
            &raw mut state_0 as *mut ::core::ffi::c_void,
        );
        ByteArray_Destroy(&raw mut arr);
        '_c2rust_label_2: {
            if state_0.p == state_0.pend {
            } else {
                __assert_fail(
                    b"state.p == state.pend\0" as *const u8 as *const ::core::ffi::c_char,
                    b"src/sm_rtl.c\0" as *const u8 as *const ::core::ffi::c_char,
                    240 as ::core::ffi::c_uint,
                    b"void StateRecorder_Load(StateRecorder *, FILE *, _Bool)\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
        };
        if is_old {
            RtlClearKeyLog();
        }
    }
    if !is_reset {
        RtlRestoreMusicAfterLoad_Locked(false_0 != 0);
    }
    RtlUpdateSnesPatchForBugfix();
}
#[no_mangle]
pub unsafe extern "C" fn StateRecorder_Save(
    mut sr: *mut StateRecorder,
    mut f: *mut FILE,
    mut saving_with_bug: bool,
) {
    let mut hdr: [uint32; 16] = [
        0 as ::core::ffi::c_int as uint32,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    ];
    let mut arr: ByteArray = ByteArray {
        data: ::core::ptr::null_mut::<uint8>(),
        size: 0,
        capacity: 0,
    };
    SaveSnesState(
        Some(
            saveFunc
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_void,
                    *mut ::core::ffi::c_void,
                    size_t,
                ) -> (),
        ),
        &raw mut arr as *mut ::core::ffi::c_void,
    );
    '_c2rust_label: {
        if (*sr).base_snapshot.size == 0 as size_t
            || (*sr).base_snapshot.size == arr.size
            || (*sr).base_snapshot.size == 8192 as size_t
        {
        } else {
            __assert_fail(
                b"sr->base_snapshot.size == 0 || sr->base_snapshot.size == arr.size || sr->base_snapshot.size == 8192\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"src/sm_rtl.c\0" as *const u8 as *const ::core::ffi::c_char,
                260 as ::core::ffi::c_uint,
                b"void StateRecorder_Save(StateRecorder *, FILE *, _Bool)\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
    };
    hdr[0 as ::core::ffi::c_int as usize] = 2 as uint32;
    hdr[1 as ::core::ffi::c_int as usize] = (*sr).total_frames;
    hdr[2 as ::core::ffi::c_int as usize] = (*sr).log.size as uint32;
    hdr[3 as ::core::ffi::c_int as usize] = (*sr).last_inputs as uint32;
    hdr[4 as ::core::ffi::c_int as usize] = (*sr).frames_since_last;
    hdr[5 as ::core::ffi::c_int as usize] = (*sr).base_snapshot.size as uint32;
    hdr[6 as ::core::ffi::c_int as usize] = arr.size as uint32;
    if (*sr).replay_mode {
        hdr[7 as ::core::ffi::c_int as usize] = (*sr).replay_pos_last_complete;
        hdr[8 as ::core::ffi::c_int as usize] = (*sr).replay_frame_counter;
    }
    hdr[9 as ::core::ffi::c_int as usize] =
        (saving_with_bug as ::core::ffi::c_int * 1 as ::core::ffi::c_int) as uint32;
    fwrite(
        &raw mut hdr as *mut uint32 as *const ::core::ffi::c_void,
        1 as size_t,
        ::core::mem::size_of::<[uint32; 16]>() as size_t,
        f,
    );
    fwrite(
        (*sr).log.data as *const ::core::ffi::c_void,
        1 as size_t,
        (*sr).log.size,
        f,
    );
    fwrite(
        (*sr).base_snapshot.data as *const ::core::ffi::c_void,
        1 as size_t,
        (*sr).base_snapshot.size,
        f,
    );
    fwrite(
        arr.data as *const ::core::ffi::c_void,
        1 as size_t,
        arr.size,
        f,
    );
    ByteArray_Destroy(&raw mut arr);
}
#[no_mangle]
pub unsafe extern "C" fn StateRecorder_ClearKeyLog(mut sr: *mut StateRecorder) {
    printf(b"Clearing key log!\n\0" as *const u8 as *const ::core::ffi::c_char);
    (*sr).base_snapshot.size = 0 as size_t;
    SaveSnesState(
        Some(
            saveFunc
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_void,
                    *mut ::core::ffi::c_void,
                    size_t,
                ) -> (),
        ),
        &raw mut (*sr).base_snapshot as *mut ::core::ffi::c_void,
    );
    let mut old_log: ByteArray = (*sr).log;
    let mut old_frames_since_last: ::core::ffi::c_int =
        (*sr).frames_since_last as ::core::ffi::c_int;
    memset(
        &raw mut (*sr).log as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<ByteArray>() as size_t,
    );
    (*sr).frames_since_last = 0 as uint32;
    if (*sr).last_inputs != 0 {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < 12 as ::core::ffi::c_int {
            if (*sr).last_inputs as ::core::ffi::c_int >> i & 1 as ::core::ffi::c_int != 0 {
                StateRecorder_RecordCmd(sr, (i << 4 as ::core::ffi::c_int) as uint8);
            }
            i += 1;
        }
    }
    if (*sr).replay_mode {
        if (*sr).replay_next_cmd_at != 0xffffffff as uint32 {
            (*sr).replay_next_cmd_at = (*sr)
                .replay_next_cmd_at
                .wrapping_sub(old_frames_since_last as uint32);
            (*sr).frames_since_last = (*sr).replay_next_cmd_at;
            (*sr).replay_pos_last_complete = (*sr).log.size as uint32;
            StateRecorder_RecordCmd(sr, (*sr).replay_cmd);
            let mut old_replay_pos: ::core::ffi::c_int = (*sr).replay_pos as ::core::ffi::c_int;
            (*sr).replay_pos = (*sr).log.size as uint32;
            ByteArray_AppendData(
                &raw mut (*sr).log,
                old_log.data.offset(old_replay_pos as isize),
                old_log.size.wrapping_sub(old_replay_pos as size_t),
            );
        }
        (*sr).total_frames = (*sr).total_frames.wrapping_sub((*sr).replay_frame_counter);
        (*sr).replay_frame_counter = 0 as uint32;
    } else {
        (*sr).total_frames = 0 as uint32;
    }
    ByteArray_Destroy(&raw mut old_log);
    (*sr).frames_since_last = 0 as uint32;
}
#[no_mangle]
pub unsafe extern "C" fn StateRecorder_ReadNextReplayState(mut sr: *mut StateRecorder) -> uint16 {
    '_c2rust_label: {
        if (*sr).replay_mode {
        } else {
            __assert_fail(
                b"sr->replay_mode\0" as *const u8 as *const ::core::ffi::c_char,
                b"src/sm_rtl.c\0" as *const u8 as *const ::core::ffi::c_char,
                322 as ::core::ffi::c_uint,
                b"uint16 StateRecorder_ReadNextReplayState(StateRecorder *)\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
    };
    while (*sr).frames_since_last >= (*sr).replay_next_cmd_at {
        let mut replay_pos: ::core::ffi::c_int = (*sr).replay_pos as ::core::ffi::c_int;
        if replay_pos as uint32 != (*sr).replay_pos_last_complete {
            (*sr).frames_since_last = 0 as uint32;
            if ((*sr).replay_cmd as ::core::ffi::c_int) < 0xc0 as ::core::ffi::c_int {
                (*sr).last_inputs = ((*sr).last_inputs as ::core::ffi::c_int
                    ^ (1 as ::core::ffi::c_int)
                        << ((*sr).replay_cmd as ::core::ffi::c_int >> 4 as ::core::ffi::c_int))
                    as uint16;
            } else if ((*sr).replay_cmd as ::core::ffi::c_int) < 0xd0 as ::core::ffi::c_int {
                let mut nb: ::core::ffi::c_int = 1 as ::core::ffi::c_int
                    + ((*sr).replay_cmd as ::core::ffi::c_int >> 2 as ::core::ffi::c_int
                        & 3 as ::core::ffi::c_int);
                let mut t: uint8 = 0;
                if nb == 4 as ::core::ffi::c_int {
                    loop {
                        let fresh3 = replay_pos;
                        replay_pos = replay_pos + 1;
                        t = *(*sr).log.data.offset(fresh3 as isize);
                        nb += t as ::core::ffi::c_int;
                        if !(t as ::core::ffi::c_int == 255 as ::core::ffi::c_int) {
                            break;
                        }
                    }
                }
                let mut addr: uint32 = (((*sr).replay_cmd as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int
                    & 1 as ::core::ffi::c_int)
                    << 16 as ::core::ffi::c_int) as uint32;
                let fresh4 = replay_pos;
                replay_pos = replay_pos + 1;
                addr |= ((*(*sr).log.data.offset(fresh4 as isize) as ::core::ffi::c_int)
                    << 8 as ::core::ffi::c_int) as uint32;
                let fresh5 = replay_pos;
                replay_pos = replay_pos + 1;
                addr |= *(*sr).log.data.offset(fresh5 as isize) as uint32;
                loop {
                    let fresh6 = replay_pos;
                    replay_pos = replay_pos + 1;
                    g_ram[(addr & 0x1ffff as uint32) as usize] =
                        *(*sr).log.data.offset(fresh6 as isize);
                    RtlSyncMemoryRegion(
                        (&raw mut g_ram as *mut uint8).offset((addr & 0x1ffff as uint32) as isize)
                            as *mut uint8 as *mut ::core::ffi::c_void,
                        1 as size_t,
                    );
                    addr = addr.wrapping_add(1);
                    nb -= 1;
                    if !(nb != 0) {
                        break;
                    }
                }
            } else {
                '_c2rust_label_0: {
                    __assert_fail(
                        b"0\0" as *const u8 as *const ::core::ffi::c_char,
                        b"src/sm_rtl.c\0" as *const u8 as *const ::core::ffi::c_char,
                        344 as ::core::ffi::c_uint,
                        b"uint16 StateRecorder_ReadNextReplayState(StateRecorder *)\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                };
            }
        }
        (*sr).replay_pos_last_complete = replay_pos as uint32;
        if replay_pos as size_t >= (*sr).log.size {
            (*sr).replay_pos = replay_pos as uint32;
            (*sr).replay_next_cmd_at = 0xffffffff as ::core::ffi::c_uint as uint32;
            break;
        } else {
            let fresh7 = replay_pos;
            replay_pos = replay_pos + 1;
            let mut cmd: uint8 = *(*sr).log.data.offset(fresh7 as isize);
            let mut t_0: uint8 = 0;
            let mut mask: ::core::ffi::c_int =
                if (cmd as ::core::ffi::c_int) < 0xc0 as ::core::ffi::c_int {
                    0xf as ::core::ffi::c_int
                } else {
                    0x1 as ::core::ffi::c_int
                };
            let mut frames: ::core::ffi::c_int = cmd as ::core::ffi::c_int & mask;
            if frames == mask {
                loop {
                    let fresh8 = replay_pos;
                    replay_pos = replay_pos + 1;
                    t_0 = *(*sr).log.data.offset(fresh8 as isize);
                    frames += t_0 as ::core::ffi::c_int;
                    if !(t_0 as ::core::ffi::c_int == 255 as ::core::ffi::c_int) {
                        break;
                    }
                }
            }
            (*sr).replay_next_cmd_at = frames as uint32;
            (*sr).replay_cmd = cmd;
            (*sr).replay_pos = replay_pos as uint32;
        }
    }
    (*sr).frames_since_last = (*sr).frames_since_last.wrapping_add(1);
    (*sr).replay_frame_counter = (*sr).replay_frame_counter.wrapping_add(1);
    if (*sr).replay_frame_counter >= (*sr).total_frames {
        (*sr).replay_mode = false_0 != 0;
    }
    return (*sr).last_inputs;
}
#[no_mangle]
pub unsafe extern "C" fn StateRecorder_StopReplay(mut sr: *mut StateRecorder) {
    if !(*sr).replay_mode {
        return;
    }
    (*sr).replay_mode = false_0 != 0;
    (*sr).total_frames = (*sr).replay_frame_counter;
    (*sr).log.size = (*sr).replay_pos_last_complete as size_t;
}
#[no_mangle]
pub unsafe extern "C" fn RtlClearKeyLog() {
    StateRecorder_ClearKeyLog(&raw mut state_recorder);
}
#[no_mangle]
pub unsafe extern "C" fn RtlStopReplay() {
    StateRecorder_StopReplay(&raw mut state_recorder);
}
#[no_mangle]
pub unsafe extern "C" fn RtlRunFrame(mut inputs: ::core::ffi::c_int) -> bool {
    if inputs & 0x30 as ::core::ffi::c_int == 0x30 as ::core::ffi::c_int {
        inputs ^= 0x30 as ::core::ffi::c_int;
    }
    if inputs & 0xc0 as ::core::ffi::c_int == 0xc0 as ::core::ffi::c_int {
        inputs ^= 0xc0 as ::core::ffi::c_int;
    }
    let mut is_replay: bool = state_recorder.replay_mode;
    if is_replay {
        inputs = StateRecorder_ReadNextReplayState(&raw mut state_recorder) as ::core::ffi::c_int;
    } else {
        if state_recorder.snapshot_flags & 1 as uint32 != 0 {
            state_recorder.snapshot_flags &= !(1 as ::core::ffi::c_int) as uint32;
            inputs = state_recorder.last_inputs as ::core::ffi::c_int;
        } else if *((&raw mut g_ram as *mut uint8).offset(0xc2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != kCurrentBugFixCounter as ::core::ffi::c_int
        {
            if g_debug_flag {
                printf(
                    b"bug_fix_counter %d => %d\n\0" as *const u8 as *const ::core::ffi::c_char,
                    *((&raw mut g_ram as *mut uint8).offset(0xc2 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int,
                    kCurrentBugFixCounter as ::core::ffi::c_int,
                );
            }
            if (*((&raw mut g_ram as *mut uint8).offset(0xc2 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int)
                < kCurrentBugFixCounter as ::core::ffi::c_int
            {
                *((&raw mut g_ram as *mut uint8).offset(0xc2 as ::core::ffi::c_int as isize)
                    as *mut uint16) = kCurrentBugFixCounter as ::core::ffi::c_int as uint16;
                StateRecorder_RecordPatchByte(
                    &raw mut state_recorder,
                    ((&raw mut g_ram as *mut uint8).offset(0xc2 as ::core::ffi::c_int as isize)
                        as *mut uint16 as *mut uint8)
                        .offset_from(&raw mut g_ram as *mut uint8)
                        as ::core::ffi::c_long as uint32,
                    (&raw mut g_ram as *mut uint8).offset(0xc2 as ::core::ffi::c_int as isize)
                        as *mut uint16 as *mut uint8,
                    2 as ::core::ffi::c_int,
                );
            }
        }
        StateRecorder_Record(&raw mut state_recorder, inputs as uint16);
    }
    if *((&raw mut g_ram as *mut uint8).offset(0xc2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        != currently_installed_bug_fix_counter as ::core::ffi::c_int
    {
        RtlUpdateSnesPatchForBugfix();
    }
    g_rtl_runframe.expect("non-null function pointer")(inputs as uint16, 0 as ::core::ffi::c_int);
    snes_frame_counter += 1;
    RtlPushApuState();
    return is_replay;
}
#[no_mangle]
pub unsafe extern "C" fn RtlSaveSnapshot(
    mut filename: *const ::core::ffi::c_char,
    mut saving_with_bug: bool,
) {
    let mut f: *mut FILE =
        fopen(filename, b"wb\0" as *const u8 as *const ::core::ffi::c_char) as *mut FILE;
    RtlApuLock();
    RtlSaveMusicStateToRam_Locked();
    StateRecorder_Save(&raw mut state_recorder, f, saving_with_bug);
    RtlApuUnlock();
    fclose(f);
}
static mut kBugSaves: [*const ::core::ffi::c_char; 7] = [
    b"Before Kraid\0" as *const u8 as *const ::core::ffi::c_char,
    b"Before Golden Torizo\0" as *const u8 as *const ::core::ffi::c_char,
    b"After Crocomire\0" as *const u8 as *const ::core::ffi::c_char,
    b"Baby Metroid\0" as *const u8 as *const ::core::ffi::c_char,
    b"Tourian Statue\0" as *const u8 as *const ::core::ffi::c_char,
    b"Before Ridley\0" as *const u8 as *const ::core::ffi::c_char,
    b"Enter Mother Brain\0" as *const u8 as *const ::core::ffi::c_char,
];
#[no_mangle]
pub unsafe extern "C" fn RtlSaveLoad(mut cmd: ::core::ffi::c_int, mut slot: ::core::ffi::c_int) {
    let mut name: [::core::ffi::c_char; 128] = [0; 128];
    if slot >= 256 as ::core::ffi::c_int {
        let mut i: ::core::ffi::c_int = slot - 256 as ::core::ffi::c_int;
        if cmd == kSaveLoad_Save as ::core::ffi::c_int
            || i as usize
                >= (::core::mem::size_of::<[*const ::core::ffi::c_char; 7]>() as usize)
                    .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>() as usize)
        {
            return;
        }
        sprintf(
            &raw mut name as *mut ::core::ffi::c_char,
            b"saves/%s.sav\0" as *const u8 as *const ::core::ffi::c_char,
            kBugSaves[i as usize],
        );
    } else {
        sprintf(
            &raw mut name as *mut ::core::ffi::c_char,
            b"saves/save%d.sav\0" as *const u8 as *const ::core::ffi::c_char,
            slot,
        );
    }
    printf(
        b"*** %s slot %d\n\0" as *const u8 as *const ::core::ffi::c_char,
        if cmd == kSaveLoad_Save as ::core::ffi::c_int {
            b"Saving\0" as *const u8 as *const ::core::ffi::c_char
        } else if cmd == kSaveLoad_Load as ::core::ffi::c_int {
            b"Loading\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            b"Replaying\0" as *const u8 as *const ::core::ffi::c_char
        },
        slot,
    );
    if cmd != kSaveLoad_Save as ::core::ffi::c_int {
        let mut f: *mut FILE = fopen(
            &raw mut name as *mut ::core::ffi::c_char,
            b"rb\0" as *const u8 as *const ::core::ffi::c_char,
        ) as *mut FILE;
        if f.is_null() {
            printf(
                b"Failed fopen: %s\n\0" as *const u8 as *const ::core::ffi::c_char,
                &raw mut name as *mut ::core::ffi::c_char,
            );
            return;
        }
        RtlApuLock();
        StateRecorder_Load(
            &raw mut state_recorder,
            f,
            cmd == kSaveLoad_Replay as ::core::ffi::c_int,
        );
        ppu_copy((*g_snes).my_ppu, (*g_snes).ppu);
        RtlApuUnlock();
        RtlSynchronizeWholeState();
        fclose(f);
        if *(&raw mut g_ram as *mut uint8).offset(0x611 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            | *(&raw mut g_ram as *mut uint8).offset(0x612 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
            | *(&raw mut g_ram as *mut uint8).offset(0x613 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
            | *(&raw mut g_ram as *mut uint8).offset(0x614 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
            | *(&raw mut g_ram as *mut uint8).offset(0x615 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
            != 0
        {
            printf(
                b"Coroutine state: %d, %d, %d, %d, %d\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                *(&raw mut g_ram as *mut uint8).offset(0x611 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int,
                *(&raw mut g_ram as *mut uint8).offset(0x612 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int,
                *(&raw mut g_ram as *mut uint8).offset(0x613 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int,
                *(&raw mut g_ram as *mut uint8).offset(0x614 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int,
                *(&raw mut g_ram as *mut uint8).offset(0x615 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int,
            );
        }
        if *(&raw mut g_ram as *mut uint8).offset(0x611 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == 4 as ::core::ffi::c_int
        {
            *(&raw mut g_ram as *mut uint8).offset(0x611 as ::core::ffi::c_int as isize) =
                (10 as ::core::ffi::c_int
                    + *((&raw mut g_ram as *mut uint8).offset(0x998 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as uint8;
        }
        if *((&raw mut g_ram as *mut uint8).offset(0xc2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            *((&raw mut g_ram as *mut uint8).offset(0xc2 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut g_ram as *mut uint8)
                .offset(0x1ff00 as ::core::ffi::c_int as isize)
                as *mut uint16);
        }
    } else {
        RtlSaveSnapshot(&raw mut name as *mut ::core::ffi::c_char, false_0 != 0);
    };
}
#[no_mangle]
pub unsafe extern "C" fn MemCpy(
    mut dst: *mut ::core::ffi::c_void,
    mut src: *const ::core::ffi::c_void,
    mut size: ::core::ffi::c_int,
) {
    memcpy(dst, src, size as size_t);
}
#[no_mangle]
pub unsafe extern "C" fn MakePairU16(mut k: uint16, mut j: uint16) -> PairU16 {
    let mut r: PairU16 = PairU16 { k: k, j: j };
    return r;
}
#[no_mangle]
pub unsafe extern "C" fn mov24(mut a: *mut LongPtr, mut d: uint32) {
    (*a).addr = (d & 0xffff as uint32) as VoidP;
    (*a).bank = (d >> 16 as ::core::ffi::c_int) as uint8;
}
#[no_mangle]
pub unsafe extern "C" fn Load24(mut src: *const LongPtr) -> uint32 {
    return *(src as *mut uint32) & 0xffffff as uint32;
}
#[no_mangle]
pub unsafe extern "C" fn Unreachable() -> bool {
    printf(b"Unreachable!\n\0" as *const u8 as *const ::core::ffi::c_char);
    '_c2rust_label: {
        __assert_fail(
            b"0\0" as *const u8 as *const ::core::ffi::c_char,
            b"src/sm_rtl.c\0" as *const u8 as *const ::core::ffi::c_char,
            512 as ::core::ffi::c_uint,
            b"_Bool Unreachable(void)\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    g_ram[0x1ffff as ::core::ffi::c_int as usize] = 1 as uint8;
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn RomPtr(mut addr: uint32_t) -> *const uint8 {
    if addr & 0x8000 as uint32_t == 0 {
        printf(
            b"RomPtr - Invalid access 0x%x!\n\0" as *const u8 as *const ::core::ffi::c_char,
            addr,
        );
        g_fail = true_0 != 0;
    }
    return g_rom.offset(
        (((addr >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | addr & 0x7fff as uint32_t)
            & 0x3fffff as uint32_t) as isize,
    ) as *const uint8;
}
#[no_mangle]
pub unsafe extern "C" fn WriteReg(mut reg: uint16, mut value: uint8) {
    snes_write(g_snes as *mut Snes, reg as uint32_t, value as uint8_t);
}
#[no_mangle]
pub unsafe extern "C" fn Mult8x8(mut a: uint8, mut b: uint8) -> uint16 {
    return (a as ::core::ffi::c_int * b as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn SnesDivide(mut a: uint16, mut b: uint8) -> uint16 {
    return (if b as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        0xffff as ::core::ffi::c_int
    } else {
        a as ::core::ffi::c_int / b as ::core::ffi::c_int
    }) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn SnesModulus(mut a: uint16, mut b: uint8) -> uint16 {
    return (if b as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        a as ::core::ffi::c_int
    } else {
        a as ::core::ffi::c_int % b as ::core::ffi::c_int
    }) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn ReadReg(mut reg: uint16) -> uint8 {
    return snes_read(g_snes as *mut Snes, reg as uint32_t) as uint8;
}
#[no_mangle]
pub unsafe extern "C" fn ReadRegWord(mut reg: uint16) -> uint16 {
    let mut rv: uint16_t = ReadReg(reg) as uint16_t;
    rv = (rv as ::core::ffi::c_int
        | (ReadReg((reg as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16)
            as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint16_t;
    return rv as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn WriteRegWord(mut reg: uint16, mut value: uint16) {
    WriteReg(reg, value as uint8);
    WriteReg(
        (reg as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16,
        (value as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint8,
    );
}
static mut g_apu_write: ApuWriteEnt = ApuWriteEnt { ports: [0; 4] };
static mut g_apu_write_ents: [ApuWriteEnt; 16] = [ApuWriteEnt { ports: [0; 4] }; 16];
static mut g_apu_time_since_empty: uint8 = 0;
static mut g_apu_write_ent_pos: uint8 = 0;
static mut g_apu_queue_size: uint8 = 0;
#[no_mangle]
pub unsafe extern "C" fn RtlApuWrite(mut adr: uint32, mut val: uint8) {
    '_c2rust_label: {
        if adr >= APUI00 as ::core::ffi::c_int as uint32
            && adr <= APUI03 as ::core::ffi::c_int as uint32
        {
        } else {
            __assert_fail(
                b"adr >= APUI00 && adr <= APUI03\0" as *const u8 as *const ::core::ffi::c_char,
                b"src/sm_rtl.c\0" as *const u8 as *const ::core::ffi::c_char,
                571 as ::core::ffi::c_uint,
                b"void RtlApuWrite(uint32, uint8)\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
    };
    if *((&raw mut g_ram as *mut uint8).offset(0x617 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        snes_catchupApu(g_snes as *mut Snes);
        (*(*g_snes).apu).inPorts[(adr & 0x3 as uint32) as usize] = val as uint8_t;
        return;
    }
    if (*g_snes).runningWhichVersion as ::core::ffi::c_int != 2 as ::core::ffi::c_int {
        g_apu_write.ports[(adr & 0x3 as uint32) as usize] = val;
    }
}
unsafe extern "C" fn IsFrameEmpty(mut w: *mut ApuWriteEnt) -> bool {
    return (*w).ports[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
        == 255 as ::core::ffi::c_int
        && (*w).ports[1 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
            == 255 as ::core::ffi::c_int
        && (*w).ports[2 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
            == 255 as ::core::ffi::c_int
        && (*w).ports[3 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
            == 255 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn RtlPushApuState() {
    RtlApuLock();
    if *((&raw mut g_ram as *mut uint8).offset(0x617 as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
    {
        if g_apu_queue_size as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            g_apu_time_since_empty = 0 as uint8;
        } else {
            if g_apu_time_since_empty as ::core::ffi::c_int >= 32 as ::core::ffi::c_int
                && IsFrameEmpty(&raw mut g_apu_write) as ::core::ffi::c_int != 0
            {
                g_apu_time_since_empty = (g_apu_time_since_empty as ::core::ffi::c_int
                    - 4 as ::core::ffi::c_int) as uint8;
                RtlApuUnlock();
                return;
            }
            g_apu_time_since_empty = g_apu_time_since_empty.wrapping_add(1);
        }
        let fresh2 = g_apu_write_ent_pos;
        g_apu_write_ent_pos = g_apu_write_ent_pos.wrapping_add(1);
        let mut w0: *mut ApuWriteEnt = (&raw mut g_apu_write_ents as *mut ApuWriteEnt).offset(
            (fresh2 as ::core::ffi::c_int
                & kApuMaxQueueSize as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                as isize,
        ) as *mut ApuWriteEnt;
        if g_apu_queue_size as ::core::ffi::c_int == kApuMaxQueueSize as ::core::ffi::c_int {
            let mut w1: *mut ApuWriteEnt = (&raw mut g_apu_write_ents as *mut ApuWriteEnt).offset(
                (g_apu_write_ent_pos as ::core::ffi::c_int
                    & kApuMaxQueueSize as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    as isize,
            ) as *mut ApuWriteEnt;
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < 4 as ::core::ffi::c_int {
                if (*w1).ports[i as usize] as ::core::ffi::c_int == 255 as ::core::ffi::c_int {
                    (*w1).ports[i as usize] = (*w0).ports[i as usize];
                }
                i += 1;
            }
        } else {
            g_apu_queue_size = g_apu_queue_size.wrapping_add(1);
        }
        *w0 = g_apu_write as ApuWriteEnt;
        memset(
            &raw mut g_apu_write as *mut ::core::ffi::c_void,
            0xff as ::core::ffi::c_int,
            ::core::mem::size_of::<ApuWriteEnt>() as size_t,
        );
    } else {
        g_apu_queue_size = 0 as uint8;
    }
    RtlApuUnlock();
}
unsafe extern "C" fn RtlPopApuState_Locked() {
    if *((&raw mut g_ram as *mut uint8).offset(0x617 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        return;
    }
    let mut input_ports: *mut uint8 = if g_use_my_apu_code as ::core::ffi::c_int != 0 {
        &raw mut (*g_spc_player).input_ports as *mut uint8
    } else {
        &raw mut (*(*g_snes).apu).inPorts as *mut uint8
    };
    if g_apu_queue_size as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        let fresh1 = g_apu_queue_size;
        g_apu_queue_size = g_apu_queue_size.wrapping_sub(1);
        let mut w: *mut ApuWriteEnt = (&raw mut g_apu_write_ents as *mut ApuWriteEnt).offset(
            (g_apu_write_ent_pos as ::core::ffi::c_int - fresh1 as ::core::ffi::c_int
                & kApuMaxQueueSize as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                as isize,
        ) as *mut ApuWriteEnt;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i != 4 as ::core::ffi::c_int {
            if (*w).ports[i as usize] as ::core::ffi::c_int != 255 as ::core::ffi::c_int {
                *input_ports.offset(i as isize) = (*w).ports[i as usize];
            }
            i += 1;
        }
    }
}
unsafe extern "C" fn RtlResetApuQueue() {
    g_apu_queue_size = 0 as uint8;
    g_apu_time_since_empty = g_apu_queue_size;
    g_apu_write_ent_pos = g_apu_time_since_empty;
    memset(
        &raw mut g_apu_write as *mut ::core::ffi::c_void,
        0xff as ::core::ffi::c_int,
        ::core::mem::size_of::<ApuWriteEnt>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn RtlApuUpload(mut p: *const uint8) {
    RtlApuLock();
    RtlResetApuQueue();
    SpcPlayer_Upload(g_spc_player as *mut SpcPlayer, p as *const uint8_t);
    RtlApuUnlock();
}
unsafe extern "C" fn RtlRestoreMusicAfterLoad_Locked(mut is_reset: bool) {
    if g_use_my_apu_code {
        memcpy(
            &raw mut (*g_spc_player).ram as *mut uint8 as *mut ::core::ffi::c_void,
            &raw mut (*(*g_snes).apu).ram as *mut uint8_t as *const ::core::ffi::c_void,
            65536 as ::core::ffi::c_int as size_t,
        );
        memcpy(
            &raw mut (*(*g_spc_player).dsp).ram as *mut uint8_t as *mut ::core::ffi::c_void,
            &raw mut (*(*(*g_snes).apu).dsp).ram as *mut uint8_t as *const ::core::ffi::c_void,
            (::core::mem::size_of::<Dsp>() as size_t).wrapping_sub(8 as size_t),
        );
        SpcPlayer_CopyVariablesFromRam(g_spc_player as *mut SpcPlayer);
    }
    if is_reset {
        SpcPlayer_Initialize(g_spc_player as *mut SpcPlayer);
    }
    RtlResetApuQueue();
}
unsafe extern "C" fn RtlSaveMusicStateToRam_Locked() {
    if g_use_my_apu_code {
        let mut spc_player: *mut SpcPlayer = g_spc_player as *mut SpcPlayer;
        let mut i: ::core::ffi::c_int = g_apu_queue_size as ::core::ffi::c_int;
        while i != 0 {
            let mut we: *mut ApuWriteEnt = (&raw mut g_apu_write_ents as *mut ApuWriteEnt).offset(
                (g_apu_write_ent_pos as ::core::ffi::c_int - i
                    & kApuMaxQueueSize as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    as isize,
            ) as *mut ApuWriteEnt;
            let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while j < 4 as ::core::ffi::c_int {
                if (*we).ports[j as usize] as ::core::ffi::c_int != 255 as ::core::ffi::c_int {
                    (*spc_player).input_ports[j as usize] = (*we).ports[j as usize];
                }
                j += 1;
            }
            i -= 1;
        }
        SpcPlayer_CopyVariablesToRam(g_spc_player as *mut SpcPlayer);
        memcpy(
            &raw mut (*(*(*g_snes).apu).dsp).ram as *mut uint8_t as *mut ::core::ffi::c_void,
            &raw mut (*(*g_spc_player).dsp).ram as *mut uint8_t as *const ::core::ffi::c_void,
            (::core::mem::size_of::<Dsp>() as size_t).wrapping_sub(8 as size_t),
        );
        memcpy(
            &raw mut (*(*g_snes).apu).ram as *mut uint8_t as *mut ::core::ffi::c_void,
            &raw mut (*g_spc_player).ram as *mut uint8 as *const ::core::ffi::c_void,
            65536 as ::core::ffi::c_int as size_t,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn RtlRenderAudio(
    mut audio_buffer: *mut int16,
    mut samples: ::core::ffi::c_int,
    mut channels: ::core::ffi::c_int,
) {
    '_c2rust_label: {
        if channels == 2 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"channels == 2\0" as *const u8 as *const ::core::ffi::c_char,
                b"src/sm_rtl.c\0" as *const u8 as *const ::core::ffi::c_char,
                679 as ::core::ffi::c_uint,
                b"void RtlRenderAudio(int16 *, int, int)\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
    };
    RtlApuLock();
    RtlPopApuState_Locked();
    if !g_use_my_apu_code {
        if *((&raw mut g_ram as *mut uint8).offset(0x617 as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
        {
            while ((*(*(*g_snes).apu).dsp).sampleOffset as ::core::ffi::c_int)
                < 534 as ::core::ffi::c_int
            {
                apu_cycle((*g_snes).apu);
            }
            dsp_getSamples((*(*g_snes).apu).dsp, audio_buffer as *mut int16_t, samples);
        }
    } else {
        SpcPlayer_GenerateSamples(g_spc_player as *mut SpcPlayer);
        dsp_getSamples((*g_spc_player).dsp, audio_buffer as *mut int16_t, samples);
    }
    RtlApuUnlock();
}
#[no_mangle]
pub unsafe extern "C" fn RtlCheat(mut c: ::core::ffi::c_char) {
    if c as ::core::ffi::c_int == 'w' as i32 {
        *((&raw mut g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut g_ram as *mut uint8)
            .offset(0x9c4 as ::core::ffi::c_int as isize)
            as *mut uint16);
        StateRecorder_RecordPatchByte(
            &raw mut state_recorder,
            0x9c2 as uint32,
            (&raw mut g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8,
            2 as ::core::ffi::c_int,
        );
    } else if c as ::core::ffi::c_int == 'q' as i32 {
        let ref mut fresh0 = *((&raw mut g_ram as *mut uint8)
            .offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh0 = (*fresh0 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
            as *mut uint16) = -(8 as ::core::ffi::c_int) as uint16;
        StateRecorder_RecordPatchByte(
            &raw mut state_recorder,
            0xafa as uint32,
            (&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8,
            2 as ::core::ffi::c_int,
        );
        StateRecorder_RecordPatchByte(
            &raw mut state_recorder,
            0xb2e as uint32,
            (&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8,
            2 as ::core::ffi::c_int,
        );
        *((&raw mut g_ram as *mut uint8).offset(0x727 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    } else if c as ::core::ffi::c_int == 'q' as i32 {
        *((&raw mut g_ram as *mut uint8).offset(0x9e2 as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut g_ram as *mut uint8)
            .offset(0x9e2 as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0) as ::core::ffi::c_int as uint16;
        StateRecorder_RecordPatchByte(
            &raw mut state_recorder,
            0x9e2 as uint32,
            (&raw mut g_ram as *mut uint8).offset(0x9e2 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8,
            2 as ::core::ffi::c_int,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn RtlReadSram() {
    let mut f: *mut FILE = fopen(
        b"saves/sm.srm\0" as *const u8 as *const ::core::ffi::c_char,
        b"rb\0" as *const u8 as *const ::core::ffi::c_char,
    ) as *mut FILE;
    if !f.is_null() {
        if fread(
            g_sram as *mut ::core::ffi::c_void,
            1 as size_t,
            8192 as size_t,
            f,
        ) != 8192 as ::core::ffi::c_ulong
        {
            fprintf(
                stderr,
                b"Error reading saves/sm.srm\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        fclose(f);
        RtlSynchronizeWholeState();
        ByteArray_Resize(&raw mut state_recorder.base_snapshot, 8192 as size_t);
        memcpy(
            state_recorder.base_snapshot.data as *mut ::core::ffi::c_void,
            g_sram as *const ::core::ffi::c_void,
            8192 as size_t,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn RtlWriteSram() {
    rename(
        b"saves/sm.srm\0" as *const u8 as *const ::core::ffi::c_char,
        b"saves/sm.srm.bak\0" as *const u8 as *const ::core::ffi::c_char,
    );
    let mut f: *mut FILE = fopen(
        b"saves/sm.srm\0" as *const u8 as *const ::core::ffi::c_char,
        b"wb\0" as *const u8 as *const ::core::ffi::c_char,
    ) as *mut FILE;
    if !f.is_null() {
        fwrite(
            g_sram as *const ::core::ffi::c_void,
            1 as size_t,
            8192 as size_t,
            f,
        );
        fclose(f);
    } else {
        fprintf(
            stderr,
            b"Unable to write saves/sm.srm\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
}
