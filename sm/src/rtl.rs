use crate::cpu_infra::RtlUpdateSnesPatchForBugfix;
use crate::game_state::Game;
use crate::host::{Die, RtlApuLock, RtlApuUnlock};
use crate::snes::apu::apu_cycle;
use crate::snes::dsp::dsp_getSamples;
use crate::snes::ppu::ppu_copy;
use crate::snes::snes::{snes_catchupApu, snes_read, snes_reset, snes_saveload, snes_write};
use crate::spc_player::{
    SpcPlayer_CopyVariablesFromRam, SpcPlayer_CopyVariablesToRam, SpcPlayer_GenerateSamples,
    SpcPlayer_Initialize, SpcPlayer_Upload,
};
pub use crate::types::{_IO_codecvt, _IO_marker, _IO_wide_data};
use crate::util::{
    ByteArray_AppendByte, ByteArray_AppendData, ByteArray_Destroy, ByteArray_Resize,
};
extern "C" {

    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
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

}
pub use crate::types::{
    _IO_lock_t, __int16_t, __int32_t, __int8_t, __off64_t, __off_t, __uint16_t, __uint32_t,
    __uint8_t, int16, int16_t, int32_t, int8_t, size_t, uint16, uint16_t, uint32, uint32_t, uint8,
    uint8_t, LongPtr, PairU16, VoidP, _IO_FILE, FILE,
};
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ApuWriteEnt {
    pub ports: [uint8; 4],
}
pub use crate::types::{
    Apu, ApuHistory as C2RustUnnamed, BgLayer, Cart, Cpu, Dma, DmaChannel, Dsp, DspChannel,
    DspRegWriteHistory, Input, Layer, Ppu, PpuPixelPrioBufs, PpuZbufType, Snes, Spc, Timer,
    WindowLayer,
};
pub const APUI03: SnesRegs = 8515;
pub const APUI00: SnesRegs = 8512;
pub use crate::types::{RunFrameFunc, SyncAllFunc};
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
    pub has_base_audio_queue: bool,
    pub base_audio_queue: [u8; 71],
}
pub use crate::types::{
    ByteArray, Channel, SaveLoadFunc, SaveLoadOperation as C2RustUnnamed_0, SfxChan, SfxRegs,
    SpcPlayer,
};
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
pub use crate::types::SnesRegs;
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

pub unsafe fn RtlSetupEmuCallbacks(
    mut emu_ram: *mut uint8,
    mut func: Option<RunFrameFunc>,
    mut sync_all: Option<SyncAllFunc>,
    game: &mut Game,
) {
    game.g_rtl_memory_ptr = emu_ram;
    game.g_rtl_runframe = func;
    game.g_rtl_syncall = sync_all;
}
unsafe fn RtlSynchronizeWholeState(game: &mut Game) {
    if game.g_rtl_syncall.is_some() {
        game.g_rtl_syncall.expect("non-null function pointer")(game);
    }
}
unsafe fn RtlSyncMemoryRegion(mut ptr: *mut ::core::ffi::c_void, mut n: size_t, game: &mut Game) {
    let mut data: *mut uint8 = ptr as *mut uint8;
    '_c2rust_label: {
        if data >= &raw mut game.g_ram as *mut uint8
            && data
                < (&raw mut game.g_ram as *mut uint8).offset(0x20000 as ::core::ffi::c_int as isize)
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
    if !game.g_rtl_memory_ptr.is_null() {
        memcpy(
            game.g_rtl_memory_ptr.offset(
                data.offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long as isize
            ) as *mut ::core::ffi::c_void,
            data as *const ::core::ffi::c_void,
            n,
        );
    }
}
pub unsafe fn ByteArray_AppendVl(mut arr: *mut ByteArray, mut v: uint32) {
    while v >= 255 as uint32 {
        ByteArray_AppendByte(arr, 255 as uint8);
        v = v.wrapping_sub(255 as uint32);
    }
    ByteArray_AppendByte(arr, v as uint8);
}
pub unsafe fn saveFunc(
    mut ctx_in: *mut ::core::ffi::c_void,
    mut data: *mut ::core::ffi::c_void,
    mut data_size: size_t,
    game: &mut Game,
) {
    ByteArray_AppendData(ctx_in as *mut ByteArray, data as *mut uint8, data_size);
}
pub unsafe fn loadFunc(
    mut ctx: *mut ::core::ffi::c_void,
    mut data: *mut ::core::ffi::c_void,
    mut data_size: size_t,
    game: &mut Game,
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
unsafe fn LoadSnesState(
    mut func: Option<SaveLoadFunc>,
    mut ctx: *mut ::core::ffi::c_void,
    game: &mut Game,
) {
    snes_saveload(game.g_snes as *mut Snes, func, ctx, game);
    crate::snes::ppu::PpuRestoreRenderRouting(&mut *(*game.g_snes).ppu);
    (*(*game.g_snes).cpu).e = false_0 != 0;
    let mut next: uint32 = ((*(*game.g_snes)
        .ram
        .offset(((*(*game.g_snes).cpu).sp as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        | (*(*game.g_snes).ram.offset(
            ((*(*game.g_snes).cpu).sp as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as isize,
        ) as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int
        | (*(*game.g_snes).ram.offset(
            ((*(*game.g_snes).cpu).sp as ::core::ffi::c_int + 5 as ::core::ffi::c_int) as isize,
        ) as ::core::ffi::c_int)
            << 16 as ::core::ffi::c_int)
        + 1 as ::core::ffi::c_int) as uint32;
    if next == 0x82897e as uint32 {
        *(*game.g_snes).ram.offset(
            ((*(*game.g_snes).cpu).sp as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize,
        ) = (0xf71b as ::core::ffi::c_int - 1 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int)
            as uint8_t;
        *(*game.g_snes).ram.offset(
            ((*(*game.g_snes).cpu).sp as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as isize,
        ) = (0xf71b as ::core::ffi::c_int - 1 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int)
            as uint8_t;
    }
    RtlSynchronizeWholeState(game);
}
unsafe fn SaveSnesState(
    mut func: Option<SaveLoadFunc>,
    mut ctx: *mut ::core::ffi::c_void,
    game: &mut Game,
) {
    snes_saveload(game.g_snes as *mut Snes, func, ctx, game);
}

pub unsafe fn StateRecorder_Init(mut sr: *mut StateRecorder) {
    ByteArray_Destroy(&raw mut (*sr).log);
    ByteArray_Destroy(&raw mut (*sr).base_snapshot);
    memset(
        sr as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<StateRecorder>() as size_t,
    );
}
pub unsafe fn StateRecorder_RecordCmd(mut sr: *mut StateRecorder, mut cmd: uint8) {
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
pub unsafe fn StateRecorder_Record(mut sr: *mut StateRecorder, mut inputs: uint16) {
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
pub unsafe fn StateRecorder_RecordPatchByte(
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
pub unsafe fn ReadFromFile(mut f: *mut FILE, mut data: *mut ::core::ffi::c_void, mut n: size_t) {
    if fread(data, 1 as size_t, n, f) as size_t != n {
        Die(b"fread failed\n\0" as *const u8 as *const ::core::ffi::c_char);
    }
}
pub unsafe fn RtlReset(mut mode: ::core::ffi::c_int, game: &mut Game) {
    game.snes_frame_counter = 0 as ::core::ffi::c_int;
    snes_reset(game.g_snes as *mut Snes, true_0 != 0, game);
    if mode & 1 as ::core::ffi::c_int == 0 {
        memset(
            game.g_sram as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            0x2000 as size_t,
        );
    }
    *(&raw mut game.g_ram as *mut uint8).offset(0x611 as ::core::ffi::c_int as isize) = 1 as uint8;
    RtlApuLock();
    RtlRestoreMusicAfterLoad_Locked(true_0 != 0, game);
    RtlApuUnlock();
    RtlSynchronizeWholeState(game);
    if mode & 2 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        StateRecorder_Init(&raw mut game.state_recorder);
    }
}
pub unsafe fn GetFileSize(mut f: *mut FILE) -> ::core::ffi::c_int {
    fseek(f, 0 as ::core::ffi::c_long, SEEK_END);
    let mut r: ::core::ffi::c_int = ftell(f) as ::core::ffi::c_int;
    fseek(f, 0 as ::core::ffi::c_long, SEEK_SET);
    return r;
}
pub unsafe fn StateRecorder_Load(
    mut sr: *mut StateRecorder,
    mut f: *mut FILE,
    mut replay_mode: bool,
    game: &mut Game,
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
    (*sr).has_base_audio_queue = false;
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
                        as unsafe fn(
                            *mut ::core::ffi::c_void,
                            *mut ::core::ffi::c_void,
                            size_t,
                            &mut Game,
                        ) -> (),
                ),
                &raw mut state as *mut ::core::ffi::c_void,
                game,
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
            RtlReset(2 as ::core::ffi::c_int, game);
            if (*sr).base_snapshot.size == 8192 as size_t {
                memcpy(
                    game.g_sram as *mut ::core::ffi::c_void,
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
                    as unsafe fn(
                        *mut ::core::ffi::c_void,
                        *mut ::core::ffi::c_void,
                        size_t,
                        &mut Game,
                    ) -> (),
            ),
            &raw mut state_0 as *mut ::core::ffi::c_void,
            game,
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
            RtlClearKeyLog(game);
        }
    }
    if !is_reset {
        RtlRestoreMusicAfterLoad_Locked(false_0 != 0, game);
    }
    // The extension describes the final snapshot, not a replay's base state.
    if !replay_mode && hdr[10] == u32::from_le_bytes(*b"APQ1") {
        assert_eq!(hdr[11], 71, "invalid saved APU queue length");
        let mut bytes = [0u8; 71];
        ReadFromFile(f, bytes.as_mut_ptr().cast(), bytes.len());
        if !game.g_use_my_apu_code {
            crate::audio_snapshot::ApuQueueSnapshot::from_bytes(&bytes).restore(game);
        }
    }
    if hdr[12] == u32::from_le_bytes(*b"ABQ1") {
        assert_eq!(hdr[13], 71, "invalid replay base APU queue length");
        if replay_mode {
            // Replay read the base snapshot, leaving the final snapshot unread.
            let final_queue_size = if hdr[10] == u32::from_le_bytes(*b"APQ1") {
                assert_eq!(hdr[11], 71);
                71
            } else {
                0
            };
            let skip = ::core::ffi::c_long::try_from(hdr[6])
                .unwrap()
                .checked_add(final_queue_size)
                .unwrap();
            assert_eq!(fseek(f, skip, 1), 0);
        }
        ReadFromFile(f, (*sr).base_audio_queue.as_mut_ptr().cast(), 71);
        (*sr).has_base_audio_queue = (*sr).base_snapshot.size > 8192;
        if replay_mode && !is_reset {
            crate::audio_snapshot::ApuQueueSnapshot::from_bytes(&(*sr).base_audio_queue)
                .restore(game);
        }
    }
    RtlUpdateSnesPatchForBugfix(game);
}
pub unsafe fn StateRecorder_Save(
    mut sr: *mut StateRecorder,
    mut f: *mut FILE,
    mut saving_with_bug: bool,
    game: &mut Game,
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
                as unsafe fn(
                    *mut ::core::ffi::c_void,
                    *mut ::core::ffi::c_void,
                    size_t,
                    &mut Game,
                ) -> (),
        ),
        &raw mut arr as *mut ::core::ffi::c_void,
        game,
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
    // Reserved version-2 header words identify a final physical queue trailer.
    // Replay bases retain the host queue for either audio backend.
    let audio_queue = (!game.g_use_my_apu_code)
        .then(|| crate::audio_snapshot::ApuQueueSnapshot::capture(game).to_bytes());
    if audio_queue.is_some() {
        hdr[10] = u32::from_le_bytes(*b"APQ1");
        hdr[11] = 71;
    }
    if (*sr).has_base_audio_queue {
        hdr[12] = u32::from_le_bytes(*b"ABQ1");
        hdr[13] = 71;
    }
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
    if let Some(bytes) = audio_queue {
        fwrite(bytes.as_ptr().cast(), 1, bytes.len(), f);
    }
    if (*sr).has_base_audio_queue {
        fwrite((*sr).base_audio_queue.as_ptr().cast(), 1, 71, f);
    }
    ByteArray_Destroy(&raw mut arr);
}
pub unsafe fn StateRecorder_ClearKeyLog(mut sr: *mut StateRecorder, game: &mut Game) {
    printf(b"Clearing key log!\n\0" as *const u8 as *const ::core::ffi::c_char);
    (*sr).base_snapshot.size = 0 as size_t;
    (*sr).has_base_audio_queue = true;
    (*sr).base_audio_queue = crate::audio_snapshot::ApuQueueSnapshot::capture(game).to_bytes();
    let live_audio = if game.g_use_my_apu_code {
        // These snapshots contain only zero-valid scalars, arrays and pointers.
        // Initialize on the heap to avoid large stack temporaries.
        let mut storage = Box::<(
            crate::audio_snapshot::AudioSnapshot,
            crate::audio_snapshot::PlayerSnapshot,
        )>::new_uninit();
        core::ptr::write_bytes(storage.as_mut_ptr(), 0, 1);
        let mut snapshots = storage.assume_init();
        snapshots.0.capture(game.g_snes);
        snapshots.1.capture(game.g_spc_player);
        // Replay restores the queue separately, so do not deliver it early.
        RtlSaveMusicStateToRam_Locked(game, false);
        Some(snapshots)
    } else {
        None
    };
    SaveSnesState(
        Some(
            saveFunc
                as unsafe fn(
                    *mut ::core::ffi::c_void,
                    *mut ::core::ffi::c_void,
                    size_t,
                    &mut Game,
                ) -> (),
        ),
        &raw mut (*sr).base_snapshot as *mut ::core::ffi::c_void,
        game,
    );
    if let Some(snapshots) = live_audio {
        snapshots.0.restore(game.g_snes);
        snapshots.1.restore(game.g_spc_player);
    }
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
pub unsafe fn StateRecorder_ReadNextReplayState(
    mut sr: *mut StateRecorder,
    game: &mut Game,
) -> uint16 {
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
                    game.g_ram[(addr & 0x1ffff as uint32) as usize] =
                        *(*sr).log.data.offset(fresh6 as isize);
                    RtlSyncMemoryRegion(
                        (&raw mut game.g_ram as *mut uint8)
                            .offset((addr & 0x1ffff as uint32) as isize)
                            as *mut uint8 as *mut ::core::ffi::c_void,
                        1 as size_t,
                        game,
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
pub unsafe fn StateRecorder_StopReplay(mut sr: *mut StateRecorder) {
    if !(*sr).replay_mode {
        return;
    }
    (*sr).replay_mode = false_0 != 0;
    (*sr).total_frames = (*sr).replay_frame_counter;
    (*sr).log.size = (*sr).replay_pos_last_complete as size_t;
}
pub unsafe fn RtlClearKeyLog(game: &mut Game) {
    StateRecorder_ClearKeyLog(&raw mut game.state_recorder, game);
}
pub unsafe fn RtlStopReplay(game: &mut Game) {
    StateRecorder_StopReplay(&raw mut game.state_recorder);
}
pub unsafe fn RtlRunFrame(mut inputs: ::core::ffi::c_int, game: &mut Game) -> bool {
    // TAS verification preserves every recorded button, including opposites.
    if !game.VERIFY_ACTIVE {
        if inputs & 0x30 as ::core::ffi::c_int == 0x30 as ::core::ffi::c_int {
            inputs ^= 0x30 as ::core::ffi::c_int;
        }
        if inputs & 0xc0 as ::core::ffi::c_int == 0xc0 as ::core::ffi::c_int {
            inputs ^= 0xc0 as ::core::ffi::c_int;
        }
    }
    let mut is_replay: bool = game.state_recorder.replay_mode;
    if is_replay {
        inputs = StateRecorder_ReadNextReplayState(&raw mut game.state_recorder, game)
            as ::core::ffi::c_int;
    } else {
        if game.state_recorder.snapshot_flags & 1 as uint32 != 0 {
            game.state_recorder.snapshot_flags &= !(1 as ::core::ffi::c_int) as uint32;
            inputs = game.state_recorder.last_inputs as ::core::ffi::c_int;
        } else if *((&raw mut game.g_ram as *mut uint8).offset(0xc2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != kCurrentBugFixCounter as ::core::ffi::c_int
        {
            if game.g_debug_flag {
                printf(
                    b"bug_fix_counter %d => %d\n\0" as *const u8 as *const ::core::ffi::c_char,
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xc2 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int,
                    kCurrentBugFixCounter as ::core::ffi::c_int,
                );
            }
            if (*((&raw mut game.g_ram as *mut uint8).offset(0xc2 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int)
                < kCurrentBugFixCounter as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xc2 as ::core::ffi::c_int as isize)
                    as *mut uint16) = kCurrentBugFixCounter as ::core::ffi::c_int as uint16;
                StateRecorder_RecordPatchByte(
                    &raw mut game.state_recorder,
                    ((&raw mut game.g_ram as *mut uint8).offset(0xc2 as ::core::ffi::c_int as isize)
                        as *mut uint16 as *mut uint8)
                        .offset_from(&raw mut game.g_ram as *mut uint8)
                        as ::core::ffi::c_long as uint32,
                    (&raw mut game.g_ram as *mut uint8).offset(0xc2 as ::core::ffi::c_int as isize)
                        as *mut uint16 as *mut uint8,
                    2 as ::core::ffi::c_int,
                );
            }
        }
        StateRecorder_Record(&raw mut game.state_recorder, inputs as uint16);
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0xc2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != game.currently_installed_bug_fix_counter as ::core::ffi::c_int
    {
        RtlUpdateSnesPatchForBugfix(game);
    }
    game.g_rtl_runframe.expect("non-null function pointer")(
        inputs as uint16,
        0 as ::core::ffi::c_int,
        game,
    );
    game.snes_frame_counter += 1;
    RtlPushApuState(game);
    return is_replay;
}
pub unsafe fn RtlSaveSnapshot(
    mut filename: *const ::core::ffi::c_char,
    mut saving_with_bug: bool,
    game: &mut Game,
) {
    let mut f: *mut FILE =
        fopen(filename, b"wb\0" as *const u8 as *const ::core::ffi::c_char) as *mut FILE;
    RtlApuLock();
    RtlSaveMusicStateToRam_Locked(game, true);
    StateRecorder_Save(&raw mut game.state_recorder, f, saving_with_bug, game);
    RtlApuUnlock();
    fclose(f);
}
static kBugSaves: crate::types::ImmutableCStringPointers<7> = unsafe {
    crate::types::ImmutableCStringPointers::new([
        b"Before Kraid\0" as *const u8 as *const ::core::ffi::c_char,
        b"Before Golden Torizo\0" as *const u8 as *const ::core::ffi::c_char,
        b"After Crocomire\0" as *const u8 as *const ::core::ffi::c_char,
        b"Baby Metroid\0" as *const u8 as *const ::core::ffi::c_char,
        b"Tourian Statue\0" as *const u8 as *const ::core::ffi::c_char,
        b"Before Ridley\0" as *const u8 as *const ::core::ffi::c_char,
        b"Enter Mother Brain\0" as *const u8 as *const ::core::ffi::c_char,
    ])
};
pub unsafe fn RtlSaveLoad(
    mut cmd: ::core::ffi::c_int,
    mut slot: ::core::ffi::c_int,
    game: &mut Game,
) {
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
            &raw mut game.state_recorder,
            f,
            cmd == kSaveLoad_Replay as ::core::ffi::c_int,
            game,
        );
        ppu_copy((*game.g_snes).my_ppu, (*game.g_snes).ppu);
        RtlApuUnlock();
        RtlSynchronizeWholeState(game);
        fclose(f);
        if *(&raw mut game.g_ram as *mut uint8).offset(0x611 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            | *(&raw mut game.g_ram as *mut uint8).offset(0x612 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
            | *(&raw mut game.g_ram as *mut uint8).offset(0x613 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
            | *(&raw mut game.g_ram as *mut uint8).offset(0x614 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
            | *(&raw mut game.g_ram as *mut uint8).offset(0x615 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
            != 0
        {
            printf(
                b"Coroutine state: %d, %d, %d, %d, %d\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                *(&raw mut game.g_ram as *mut uint8).offset(0x611 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int,
                *(&raw mut game.g_ram as *mut uint8).offset(0x612 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int,
                *(&raw mut game.g_ram as *mut uint8).offset(0x613 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int,
                *(&raw mut game.g_ram as *mut uint8).offset(0x614 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int,
                *(&raw mut game.g_ram as *mut uint8).offset(0x615 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int,
            );
        }
        if *(&raw mut game.g_ram as *mut uint8).offset(0x611 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == 4 as ::core::ffi::c_int
        {
            *(&raw mut game.g_ram as *mut uint8).offset(0x611 as ::core::ffi::c_int as isize) = (10
                as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x998 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int)
                as uint8;
        }
        if *((&raw mut game.g_ram as *mut uint8).offset(0xc2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc2 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x1ff00 as ::core::ffi::c_int as isize)
                as *mut uint16);
        }
    } else {
        RtlSaveSnapshot(
            &raw mut name as *mut ::core::ffi::c_char,
            false_0 != 0,
            game,
        );
    };
}
pub unsafe fn MemCpy(
    mut dst: *mut ::core::ffi::c_void,
    mut src: *const ::core::ffi::c_void,
    mut size: ::core::ffi::c_int,
) {
    memcpy(dst, src, size as size_t);
}
pub unsafe fn MakePairU16(mut k: uint16, mut j: uint16) -> PairU16 {
    let mut r: PairU16 = PairU16 { k: k, j: j };
    return r;
}
pub unsafe fn mov24(mut a: *mut LongPtr, mut d: uint32) {
    (*a).addr = (d & 0xffff as uint32) as VoidP;
    (*a).bank = (d >> 16 as ::core::ffi::c_int) as uint8;
}
pub unsafe fn Load24(mut src: *const LongPtr) -> uint32 {
    return *(src as *mut uint32) & 0xffffff as uint32;
}
pub unsafe fn Unreachable(game: &mut Game) -> bool {
    printf(b"Unreachable!\n\0" as *const u8 as *const ::core::ffi::c_char);
    '_c2rust_label: {
        __assert_fail(
            b"0\0" as *const u8 as *const ::core::ffi::c_char,
            b"src/sm_rtl.c\0" as *const u8 as *const ::core::ffi::c_char,
            512 as ::core::ffi::c_uint,
            b"_Bool Unreachable(void)\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    game.g_ram[0x1ffff as ::core::ffi::c_int as usize] = 1 as uint8;
    return false_0 != 0;
}
pub unsafe fn RomPtr(mut addr: uint32_t, game: &mut Game) -> *const uint8 {
    if addr & 0x8000 as uint32_t == 0 {
        printf(
            b"RomPtr - Invalid access 0x%x!\n\0" as *const u8 as *const ::core::ffi::c_char,
            addr,
        );
        game.g_fail = true_0 != 0;
    }
    return game.g_rom.offset(
        (((addr >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | addr & 0x7fff as uint32_t)
            & 0x3fffff as uint32_t) as isize,
    ) as *const uint8;
}
pub unsafe fn WriteReg(mut reg: uint16, mut value: uint8, game: &mut Game) {
    snes_write(
        game.g_snes as *mut Snes,
        reg as uint32_t,
        value as uint8_t,
        game,
    );
}
pub unsafe fn Mult8x8(mut a: uint8, mut b: uint8) -> uint16 {
    return (a as ::core::ffi::c_int * b as ::core::ffi::c_int) as uint16;
}
pub unsafe fn SnesDivide(mut a: uint16, mut b: uint8) -> uint16 {
    return (if b as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        0xffff as ::core::ffi::c_int
    } else {
        a as ::core::ffi::c_int / b as ::core::ffi::c_int
    }) as uint16;
}
pub unsafe fn SnesModulus(mut a: uint16, mut b: uint8) -> uint16 {
    return (if b as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        a as ::core::ffi::c_int
    } else {
        a as ::core::ffi::c_int % b as ::core::ffi::c_int
    }) as uint16;
}
pub unsafe fn ReadReg(mut reg: uint16, game: &mut Game) -> uint8 {
    return snes_read(game.g_snes as *mut Snes, reg as uint32_t, game) as uint8;
}
pub unsafe fn ReadRegWord(mut reg: uint16, game: &mut Game) -> uint16 {
    let mut rv: uint16_t = ReadReg(reg, game) as uint16_t;
    rv = (rv as ::core::ffi::c_int
        | (ReadReg(
            (reg as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16,
            game,
        ) as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint16_t;
    return rv as uint16;
}
pub unsafe fn WriteRegWord(mut reg: uint16, mut value: uint16, game: &mut Game) {
    WriteReg(reg, value as uint8, game);
    WriteReg(
        (reg as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16,
        (value as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint8,
        game,
    );
}

pub unsafe fn RtlApuWrite(mut adr: uint32, mut val: uint8, game: &mut Game) {
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
    if *((&raw mut game.g_ram as *mut uint8).offset(0x617 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        snes_catchupApu(game.g_snes as *mut Snes, game);
        (*(*game.g_snes).apu).inPorts[(adr & 0x3 as uint32) as usize] = val as uint8_t;
        return;
    }
    game.g_apu_write.ports[(adr & 0x3 as uint32) as usize] = val;
}
unsafe fn IsFrameEmpty(mut w: *mut ApuWriteEnt) -> bool {
    return (*w).ports[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
        == 255 as ::core::ffi::c_int
        && (*w).ports[1 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
            == 255 as ::core::ffi::c_int
        && (*w).ports[2 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
            == 255 as ::core::ffi::c_int
        && (*w).ports[3 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
            == 255 as ::core::ffi::c_int;
}
pub unsafe fn RtlPushApuState(game: &mut Game) {
    RtlApuLock();
    if *((&raw mut game.g_ram as *mut uint8).offset(0x617 as ::core::ffi::c_int as isize)
        as *mut uint16)
        == 0
    {
        if game.g_apu_queue_size as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            game.g_apu_time_since_empty = 0 as uint8;
        } else {
            if game.g_apu_time_since_empty as ::core::ffi::c_int >= 32 as ::core::ffi::c_int
                && IsFrameEmpty(&raw mut game.g_apu_write) as ::core::ffi::c_int != 0
            {
                game.g_apu_time_since_empty = (game.g_apu_time_since_empty as ::core::ffi::c_int
                    - 4 as ::core::ffi::c_int)
                    as uint8;
                RtlApuUnlock();
                return;
            }
            game.g_apu_time_since_empty = game.g_apu_time_since_empty.wrapping_add(1);
        }
        let fresh2 = game.g_apu_write_ent_pos;
        game.g_apu_write_ent_pos = game.g_apu_write_ent_pos.wrapping_add(1);
        let mut w0: *mut ApuWriteEnt = (&raw mut game.g_apu_write_ents as *mut ApuWriteEnt).offset(
            (fresh2 as ::core::ffi::c_int
                & kApuMaxQueueSize as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                as isize,
        ) as *mut ApuWriteEnt;
        if game.g_apu_queue_size as ::core::ffi::c_int == kApuMaxQueueSize as ::core::ffi::c_int {
            let mut w1: *mut ApuWriteEnt = (&raw mut game.g_apu_write_ents as *mut ApuWriteEnt)
                .offset(
                    (game.g_apu_write_ent_pos as ::core::ffi::c_int
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
            game.g_apu_queue_size = game.g_apu_queue_size.wrapping_add(1);
        }
        *w0 = game.g_apu_write as ApuWriteEnt;
        memset(
            &raw mut game.g_apu_write as *mut ::core::ffi::c_void,
            0xff as ::core::ffi::c_int,
            ::core::mem::size_of::<ApuWriteEnt>() as size_t,
        );
    } else {
        game.g_apu_queue_size = 0 as uint8;
    }
    RtlApuUnlock();
}
unsafe fn RtlPopApuState_Locked(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x617 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        return;
    }
    let mut input_ports: *mut uint8 = if game.g_use_my_apu_code as ::core::ffi::c_int != 0 {
        &raw mut (*game.g_spc_player).input_ports as *mut uint8
    } else {
        &raw mut (*(*game.g_snes).apu).inPorts as *mut uint8
    };
    if game.g_apu_queue_size as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        let fresh1 = game.g_apu_queue_size;
        game.g_apu_queue_size = game.g_apu_queue_size.wrapping_sub(1);
        let mut w: *mut ApuWriteEnt = (&raw mut game.g_apu_write_ents as *mut ApuWriteEnt).offset(
            (game.g_apu_write_ent_pos as ::core::ffi::c_int - fresh1 as ::core::ffi::c_int
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
unsafe fn RtlResetApuQueue(game: &mut Game) {
    game.g_apu_queue_size = 0 as uint8;
    game.g_apu_time_since_empty = game.g_apu_queue_size;
    game.g_apu_write_ent_pos = game.g_apu_time_since_empty;
    memset(
        &raw mut game.g_apu_write as *mut ::core::ffi::c_void,
        0xff as ::core::ffi::c_int,
        ::core::mem::size_of::<ApuWriteEnt>() as size_t,
    );
}
pub unsafe fn RtlApuUpload(mut p: *const uint8, game: &mut Game) {
    let version = if game.g_snes.is_null() {
        0
    } else {
        (*game.g_snes).runningWhichVersion
    };
    if version == 1 || version == 2 {
        let startup = p == RomPtr(0xcf8000, game);
        let snapshot = if version == 1 {
            &mut game.g_snapshot_theirs
        } else {
            &mut game.g_snapshot_mine
        };
        snapshot.apu_upload_count = snapshot.apu_upload_count.saturating_add(1);
        snapshot.apu_startup_upload = startup;
    }
    RtlApuLock();
    RtlResetApuQueue(game);
    SpcPlayer_Upload(game.g_spc_player as *mut SpcPlayer, p as *const uint8_t);
    RtlApuUnlock();
}
unsafe fn RtlRestoreMusicAfterLoad_Locked(mut is_reset: bool, game: &mut Game) {
    if game.g_use_my_apu_code {
        memcpy(
            &raw mut (*game.g_spc_player).ram as *mut uint8 as *mut ::core::ffi::c_void,
            &raw mut (*(*game.g_snes).apu).ram as *mut uint8_t as *const ::core::ffi::c_void,
            65536 as ::core::ffi::c_int as size_t,
        );
        memcpy(
            &raw mut (*(*game.g_spc_player).dsp).ram as *mut uint8_t as *mut ::core::ffi::c_void,
            &raw mut (*(*(*game.g_snes).apu).dsp).ram as *mut uint8_t as *const ::core::ffi::c_void,
            (::core::mem::size_of::<Dsp>() as size_t).wrapping_sub(8 as size_t),
        );
        SpcPlayer_CopyVariablesFromRam(game.g_spc_player as *mut SpcPlayer);
    }
    if is_reset {
        SpcPlayer_Initialize(game.g_spc_player as *mut SpcPlayer);
    }
    RtlResetApuQueue(game);
}
unsafe fn RtlSaveMusicStateToRam_Locked(game: &mut Game, include_queued_commands: bool) {
    if game.g_use_my_apu_code {
        let mut spc_player: *mut SpcPlayer = game.g_spc_player as *mut SpcPlayer;
        let mut i: ::core::ffi::c_int = if include_queued_commands {
            game.g_apu_queue_size as ::core::ffi::c_int
        } else {
            0
        };
        while i != 0 {
            let mut we: *mut ApuWriteEnt = (&raw mut game.g_apu_write_ents as *mut ApuWriteEnt)
                .offset(
                    (game.g_apu_write_ent_pos as ::core::ffi::c_int - i
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
        SpcPlayer_CopyVariablesToRam(game.g_spc_player as *mut SpcPlayer);
        memcpy(
            &raw mut (*(*(*game.g_snes).apu).dsp).ram as *mut uint8_t as *mut ::core::ffi::c_void,
            &raw mut (*(*game.g_spc_player).dsp).ram as *mut uint8_t as *const ::core::ffi::c_void,
            (::core::mem::size_of::<Dsp>() as size_t).wrapping_sub(8 as size_t),
        );
        memcpy(
            &raw mut (*(*game.g_snes).apu).ram as *mut uint8_t as *mut ::core::ffi::c_void,
            &raw mut (*game.g_spc_player).ram as *mut uint8 as *const ::core::ffi::c_void,
            65536 as ::core::ffi::c_int as size_t,
        );
    }
}
pub unsafe fn RtlRenderAudio(
    mut audio_buffer: *mut int16,
    mut samples: ::core::ffi::c_int,
    mut channels: ::core::ffi::c_int,
    game: &mut Game,
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
    RtlPopApuState_Locked(game);
    if !game.g_use_my_apu_code {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x617 as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
        {
            while ((*(*(*game.g_snes).apu).dsp).sampleOffset as ::core::ffi::c_int)
                < 534 as ::core::ffi::c_int
            {
                apu_cycle((*game.g_snes).apu, game);
            }
            dsp_getSamples(
                (*(*game.g_snes).apu).dsp,
                audio_buffer as *mut int16_t,
                samples,
            );
        }
    } else {
        SpcPlayer_GenerateSamples(game.g_spc_player as *mut SpcPlayer);
        dsp_getSamples(
            (*game.g_spc_player).dsp,
            audio_buffer as *mut int16_t,
            samples,
        );
    }
    RtlApuUnlock();
}
pub unsafe fn RtlCheat(mut c: ::core::ffi::c_char, game: &mut Game) {
    if c as ::core::ffi::c_int == 'w' as i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x9c4 as ::core::ffi::c_int as isize)
            as *mut uint16);
        StateRecorder_RecordPatchByte(
            &raw mut game.state_recorder,
            0x9c2 as uint32,
            (&raw mut game.g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8,
            2 as ::core::ffi::c_int,
        );
    } else if c as ::core::ffi::c_int == 'q' as i32 {
        let ref mut fresh0 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh0 = (*fresh0 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
            as *mut uint16) = -(8 as ::core::ffi::c_int) as uint16;
        StateRecorder_RecordPatchByte(
            &raw mut game.state_recorder,
            0xafa as uint32,
            (&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8,
            2 as ::core::ffi::c_int,
        );
        StateRecorder_RecordPatchByte(
            &raw mut game.state_recorder,
            0xb2e as uint32,
            (&raw mut game.g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8,
            2 as ::core::ffi::c_int,
        );
        *((&raw mut game.g_ram as *mut uint8).offset(0x727 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    } else if c as ::core::ffi::c_int == 'q' as i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9e2 as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0x9e2 as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0) as ::core::ffi::c_int as uint16;
        StateRecorder_RecordPatchByte(
            &raw mut game.state_recorder,
            0x9e2 as uint32,
            (&raw mut game.g_ram as *mut uint8).offset(0x9e2 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8,
            2 as ::core::ffi::c_int,
        );
    }
}
pub unsafe fn RtlReadSram(game: &mut Game) {
    let mut f: *mut FILE = fopen(
        b"saves/sm.srm\0" as *const u8 as *const ::core::ffi::c_char,
        b"rb\0" as *const u8 as *const ::core::ffi::c_char,
    ) as *mut FILE;
    if !f.is_null() {
        if fread(
            game.g_sram as *mut ::core::ffi::c_void,
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
        RtlSynchronizeWholeState(game);
        ByteArray_Resize(&raw mut game.state_recorder.base_snapshot, 8192 as size_t);
        game.state_recorder.has_base_audio_queue = false;
        memcpy(
            game.state_recorder.base_snapshot.data as *mut ::core::ffi::c_void,
            game.g_sram as *const ::core::ffi::c_void,
            8192 as size_t,
        );
    }
}
pub unsafe fn RtlWriteSram(game: &mut Game) {
    if game.VERIFY_ACTIVE {
        return;
    }
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
            game.g_sram as *const ::core::ffi::c_void,
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
