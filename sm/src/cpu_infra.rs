extern "C" {
    pub type _IO_wide_data;
    pub type _IO_codecvt;
    pub type _IO_marker;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    static mut stderr: *mut FILE;
    fn fprintf(
        __stream: *mut FILE,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn printf(__format: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn puts(__s: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
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
    fn memcmp(
        __s1: *const ::core::ffi::c_void,
        __s2: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn cpu_runOpcode(cpu: *mut Cpu) -> ::core::ffi::c_int;
    fn cpu_getFlags(cpu: *mut Cpu) -> uint8_t;
    fn cpu_setFlags(cpu: *mut Cpu, val: uint8_t);
    fn apu_cycle(apu: *mut Apu);
    fn dma_doDma(dma: *mut Dma);
    fn snes_init(ram: *mut uint8_t) -> *mut Snes;
    fn snes_handle_pos_stuff(snes: *mut Snes);
    fn snes_loadRom(snes: *mut Snes, data: *const uint8_t, length: ::core::ffi::c_int) -> bool;
    static mut snes_frame_counter: ::core::ffi::c_int;
    fn RunOneFrameOfGame();
    fn getProcessorStateCpu(snes: *mut Snes, line: *mut ::core::ffi::c_char);
    static mut g_ram: [uint8; 131072];
    static mut g_sram: *mut uint8;
    static mut g_rom: *const uint8;
    static mut g_debug_flag: bool;
    fn RomPtr(addr: uint32_t) -> *const uint8;
    fn WriteReg(reg: uint16, value: uint8);
    fn RtlSetupEmuCallbacks(
        emu_ram: *mut uint8,
        func: Option<RunFrameFunc>,
        sync_all: Option<SyncAllFunc>,
    );
    fn RtlSaveSnapshot(filename: *const ::core::ffi::c_char, saving_with_bug: bool);
    fn Vector_IRQ();
    fn ReadWholeFile(name: *const ::core::ffi::c_char, length: *mut size_t) -> *mut uint8;
    fn time(__timer: *mut time_t) -> time_t;
    static mut g_got_mismatch_count: ::core::ffi::c_int;
    static mut g_other_image: bool;
}
pub use crate::types::{
    __int16_t, __int32_t, __int8_t, __off64_t, __off_t, __time_t, __uint16_t, __uint32_t,
    __uint8_t, int16_t, int32_t, int8, int8_t, size_t, time_t, uint16, uint16_t, uint32, uint32_t,
    uint8, uint8_t, VoidP,
};
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
pub use crate::types::_IO_lock_t;
pub type FILE = _IO_FILE;
pub use crate::types::{
    Apu, ApuHistory as C2RustUnnamed, BgLayer, Cart, Cpu, Dma, DmaChannel, Dsp, DspChannel,
    DspRegWriteHistory, EnemyBase, Enemy_Botwoon, Input, Layer, Ppu, PpuPixelPrioBufs, PpuZbufType,
    Snes, Spc, Timer, WindowLayer,
};
pub const VMAIN: SnesRegs = 8469;
pub use crate::types::{ProjectileInstr, SyncAllFunc};
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Snapshot {
    pub a: uint16,
    pub x: uint16,
    pub y: uint16,
    pub sp: uint16,
    pub dp: uint16,
    pub pc: uint16,
    pub k: uint8,
    pub db: uint8,
    pub flags: uint8,
    pub vTimer: uint16_t,
    pub ram: [uint8; 131072],
    pub vram: [uint16; 32768],
    pub sram: [uint8; 8192],
    pub oam: [uint16; 288],
}
pub const RM_MINE: RunMode = 1;
pub const RM_BOTH: RunMode = 0;
pub const RM_THEIRS: RunMode = 2;
pub use crate::types::{RunFrameFunc, SnesRegs};
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
pub const APUI03: SnesRegs = 8515;
pub const APUI02: SnesRegs = 8514;
pub const APUI01: SnesRegs = 8513;
pub const APUI00: SnesRegs = 8512;
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
pub use crate::types::RunMode;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn get_ProjectileInstr(mut a: uint16) -> *mut ProjectileInstr {
    return RomPtr((0x930000 as ::core::ffi::c_int | a as ::core::ffi::c_int) as uint32_t)
        as *mut ProjectileInstr;
}
#[inline]
unsafe extern "C" fn Get_Botwoon(mut o: ::core::ffi::c_int) -> *mut Enemy_Botwoon {
    return (&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_Botwoon;
}
#[no_mangle]
pub static mut g_runmode: uint8 = RM_BOTH as ::core::ffi::c_int as uint8;
#[no_mangle]
pub static mut g_snes: *mut Snes = ::core::ptr::null::<Snes>() as *mut Snes;
#[no_mangle]
pub static mut g_cpu: *mut Cpu = ::core::ptr::null::<Cpu>() as *mut Cpu;
#[no_mangle]
pub static mut g_calling_asm_from_c: bool = false;
#[no_mangle]
pub static mut g_calling_asm_from_c_ret: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut g_fail: bool = false;
#[no_mangle]
pub static mut g_use_my_apu_code: bool = true_0 != 0;
static mut g_snapshot_mine: Snapshot = Snapshot {
    a: 0,
    x: 0,
    y: 0,
    sp: 0,
    dp: 0,
    pc: 0,
    k: 0,
    db: 0,
    flags: 0,
    vTimer: 0,
    ram: [0; 131072],
    vram: [0; 32768],
    sram: [0; 8192],
    oam: [0; 288],
};
static mut g_snapshot_theirs: Snapshot = Snapshot {
    a: 0,
    x: 0,
    y: 0,
    sp: 0,
    dp: 0,
    pc: 0,
    k: 0,
    db: 0,
    flags: 0,
    vTimer: 0,
    ram: [0; 131072],
    vram: [0; 32768],
    sram: [0; 8192],
    oam: [0; 288],
};
static mut g_snapshot_before: Snapshot = Snapshot {
    a: 0,
    x: 0,
    y: 0,
    sp: 0,
    dp: 0,
    pc: 0,
    k: 0,
    db: 0,
    flags: 0,
    vTimer: 0,
    ram: [0; 131072],
    vram: [0; 32768],
    sram: [0; 8192],
    oam: [0; 288],
};
static mut hookmode: uint32 = 0;
static mut hookcnt: uint32 = 0;
static mut hookadr: uint32 = 0;
static mut hook_orgbyte: [uint8; 1024] = [0; 1024];
static mut hook_fixbug_orgbyte: [uint8; 1024] = [0; 1024];
#[no_mangle]
pub unsafe extern "C" fn Call(mut addr: uint32) {
    '_c2rust_label: {
        if addr & 0x8000 as uint32 != 0 {
        } else {
            __assert_fail(
                b"addr & 0x8000\0" as *const u8 as *const ::core::ffi::c_char,
                b"src/sm_cpu_infra.c\0" as *const u8 as *const ::core::ffi::c_char,
                54 as ::core::ffi::c_uint,
                b"void Call(uint32)\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
    };
    RunAsmCode(
        addr,
        0 as uint16,
        0 as uint16,
        0 as uint16,
        0 as ::core::ffi::c_int,
    );
}
#[no_mangle]
pub unsafe extern "C" fn SnesRomPtr(mut v: uint32) -> *mut uint8_t {
    return RomPtr(v as uint32_t) as *mut uint8_t;
}
#[no_mangle]
pub unsafe extern "C" fn ProcessHook(mut v: uint32) -> bool {
    let mut rombyte: *mut uint8_t = SnesRomPtr(v);
    match hookmode {
        0 => {
            let fresh2 = hookcnt;
            hookcnt = hookcnt.wrapping_add(1);
            *rombyte = hook_orgbyte[fresh2 as usize] as uint8_t;
            return false_0 != 0;
        }
        1 => {
            let fresh3 = hookcnt;
            hookcnt = hookcnt.wrapping_add(1);
            hook_orgbyte[fresh3 as usize] = *rombyte as uint8;
            *rombyte = 0 as uint8_t;
            return false_0 != 0;
        }
        2 => {
            if v == hookadr {
                hookmode = 3 as uint32;
                return true_0 != 0;
            }
            return false_0 != 0;
        }
        _ => {}
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn FixBugHook(mut addr: uint32) -> bool {
    match hookmode {
        1 => {
            let mut rombyte: *mut uint8_t = SnesRomPtr(addr);
            let fresh0 = hookcnt;
            hookcnt = hookcnt.wrapping_add(1);
            hook_fixbug_orgbyte[fresh0 as usize] = *rombyte as uint8;
            *rombyte = 0 as uint8_t;
            return false_0 != 0;
        }
        2 => {
            if addr == hookadr {
                hookmode = 3 as uint32;
                return true_0 != 0;
            }
            hookcnt = hookcnt.wrapping_add(1);
            return false_0 != 0;
        }
        _ => {}
    }
    return false_0 != 0;
}
static mut kPatchedCarrys: [uint32; 155] = [
    0xa7ac33 as ::core::ffi::c_int as uint32,
    0xa7ac36 as ::core::ffi::c_int as uint32,
    0xa7ac39 as ::core::ffi::c_int as uint32,
    0xa7ac42 as ::core::ffi::c_int as uint32,
    0xa7ac45 as ::core::ffi::c_int as uint32,
    0xa6d6d1 as ::core::ffi::c_int as uint32,
    0xa6d6d3 as ::core::ffi::c_int as uint32,
    0xa6d6d5 as ::core::ffi::c_int as uint32,
    0xa6d700 as ::core::ffi::c_int as uint32,
    0xa6d702 as ::core::ffi::c_int as uint32,
    0xa6d704 as ::core::ffi::c_int as uint32,
    0xa6d665 as ::core::ffi::c_int as uint32,
    0xa6d667 as ::core::ffi::c_int as uint32,
    0xa6d669 as ::core::ffi::c_int as uint32,
    0xa6d694 as ::core::ffi::c_int as uint32,
    0xa6d696 as ::core::ffi::c_int as uint32,
    0xa6d698 as ::core::ffi::c_int as uint32,
    0x818b65 as ::core::ffi::c_int as uint32,
    0x818b6b as ::core::ffi::c_int as uint32,
    0x818ba7 as ::core::ffi::c_int as uint32,
    0x818bd9 as ::core::ffi::c_int as uint32,
    0x818bdf as ::core::ffi::c_int as uint32,
    0x86a671 as ::core::ffi::c_int as uint32,
    0x86a680 as ::core::ffi::c_int as uint32,
    0x86a6a9 as ::core::ffi::c_int as uint32,
    0x86a6ba as ::core::ffi::c_int as uint32,
    0x88b245 as ::core::ffi::c_int as uint32,
    0xa6d565 as ::core::ffi::c_int as uint32,
    0xa6d567 as ::core::ffi::c_int as uint32,
    0xa6d599 as ::core::ffi::c_int as uint32,
    0xa6d59b as ::core::ffi::c_int as uint32,
    0xa6d5db as ::core::ffi::c_int as uint32,
    0xa6d5dd as ::core::ffi::c_int as uint32,
    0xa6d60f as ::core::ffi::c_int as uint32,
    0xa6d611 as ::core::ffi::c_int as uint32,
    0xa6ceff as ::core::ffi::c_int as uint32,
    0xa9f4a5 as ::core::ffi::c_int as uint32,
    0xa9f4a7 as ::core::ffi::c_int as uint32,
    0xa9f4d6 as ::core::ffi::c_int as uint32,
    0xa9f4d8 as ::core::ffi::c_int as uint32,
    0xa9f519 as ::core::ffi::c_int as uint32,
    0xa9f51f as ::core::ffi::c_int as uint32,
    0xa9f521 as ::core::ffi::c_int as uint32,
    0xa9f554 as ::core::ffi::c_int as uint32,
    0xa9f55a as ::core::ffi::c_int as uint32,
    0xa9f55c as ::core::ffi::c_int as uint32,
    0xa9f24d as ::core::ffi::c_int as uint32,
    0x80aa6a as ::core::ffi::c_int as uint32,
    0x80a592 as ::core::ffi::c_int as uint32,
    0x80a720 as ::core::ffi::c_int as uint32,
    0x80a7d6 as ::core::ffi::c_int as uint32,
    0x80a99b as ::core::ffi::c_int as uint32,
    0x818aa7 as ::core::ffi::c_int as uint32,
    0x94b176 as ::core::ffi::c_int as uint32,
    0x94b156 as ::core::ffi::c_int as uint32,
    0xa99413 as ::core::ffi::c_int as uint32,
    0x80ab5d as ::core::ffi::c_int as uint32,
    0x84865c as ::core::ffi::c_int as uint32,
    0x848d90 as ::core::ffi::c_int as uint32,
    0x84ab60 as ::core::ffi::c_int as uint32,
    0x84b567 as ::core::ffi::c_int as uint32,
    0x84b588 as ::core::ffi::c_int as uint32,
    0x84b606 as ::core::ffi::c_int as uint32,
    0x84b615 as ::core::ffi::c_int as uint32,
    0x84b624 as ::core::ffi::c_int as uint32,
    0x84b9d3 as ::core::ffi::c_int as uint32,
    0x84b9e2 as ::core::ffi::c_int as uint32,
    0x84ba07 as ::core::ffi::c_int as uint32,
    0x84ba1e as ::core::ffi::c_int as uint32,
    0x84ba35 as ::core::ffi::c_int as uint32,
    0x84d6ae as ::core::ffi::c_int as uint32,
    0x84d6bf as ::core::ffi::c_int as uint32,
    0x84d812 as ::core::ffi::c_int as uint32,
    0x84daae as ::core::ffi::c_int as uint32,
    0x84dbaa as ::core::ffi::c_int as uint32,
    0x84dbe1 as ::core::ffi::c_int as uint32,
    0x84dc20 as ::core::ffi::c_int as uint32,
    0x84dc52 as ::core::ffi::c_int as uint32,
    0x84dc89 as ::core::ffi::c_int as uint32,
    0x84dcc8 as ::core::ffi::c_int as uint32,
    0x84deae as ::core::ffi::c_int as uint32,
    0x84dedd as ::core::ffi::c_int as uint32,
    0x84df0a as ::core::ffi::c_int as uint32,
    0x84df39 as ::core::ffi::c_int as uint32,
    0x86893a as ::core::ffi::c_int as uint32,
    0x9483a7 as ::core::ffi::c_int as uint32,
    0x948405 as ::core::ffi::c_int as uint32,
    0x949592 as ::core::ffi::c_int as uint32,
    0x94a13f as ::core::ffi::c_int as uint32,
    0x94a2b2 as ::core::ffi::c_int as uint32,
    0x94a3d8 as ::core::ffi::c_int as uint32,
    0xa0bc33 as ::core::ffi::c_int as uint32,
    0xa0bdac as ::core::ffi::c_int as uint32,
    0xa0bf45 as ::core::ffi::c_int as uint32,
    0xa0c725 as ::core::ffi::c_int as uint32,
    0x88b486 as ::core::ffi::c_int as uint32,
    0x88c578 as ::core::ffi::c_int as uint32,
    0xa292e8 as ::core::ffi::c_int as uint32,
    0x86f18e as ::core::ffi::c_int as uint32,
    0x888cb6 as ::core::ffi::c_int as uint32,
    0x888faa as ::core::ffi::c_int as uint32,
    0x88a483 as ::core::ffi::c_int as uint32,
    0x91cc35 as ::core::ffi::c_int as uint32,
    0x91cbff as ::core::ffi::c_int as uint32,
    0xa09541 as ::core::ffi::c_int as uint32,
    0xa09552 as ::core::ffi::c_int as uint32,
    0xa49ae8 as ::core::ffi::c_int as uint32,
    0xa6c297 as ::core::ffi::c_int as uint32,
    0xa6c3ad as ::core::ffi::c_int as uint32,
    0xa9c5ec as ::core::ffi::c_int as uint32,
    0xa9d500 as ::core::ffi::c_int as uint32,
    0xa9d537 as ::core::ffi::c_int as uint32,
    0xa9dcdb as ::core::ffi::c_int as uint32,
    0xa0a31b as ::core::ffi::c_int as uint32,
    0x91d064 as ::core::ffi::c_int as uint32,
    0x91d07a as ::core::ffi::c_int as uint32,
    0x90c719 as ::core::ffi::c_int as uint32,
    0xa6a80e as ::core::ffi::c_int as uint32,
    0xa6a816 as ::core::ffi::c_int as uint32,
    0xa4906e as ::core::ffi::c_int as uint32,
    0xa49071 as ::core::ffi::c_int as uint32,
    0x90bc75 as ::core::ffi::c_int as uint32,
    0x90bc93 as ::core::ffi::c_int as uint32,
    0xa8a459 as ::core::ffi::c_int as uint32,
    0xa8a45f as ::core::ffi::c_int as uint32,
    0xa8a465 as ::core::ffi::c_int as uint32,
    0xa8a46b as ::core::ffi::c_int as uint32,
    0xa8a477 as ::core::ffi::c_int as uint32,
    0xa8a47d as ::core::ffi::c_int as uint32,
    0xa8a483 as ::core::ffi::c_int as uint32,
    0xa8a489 as ::core::ffi::c_int as uint32,
    0xa8a543 as ::core::ffi::c_int as uint32,
    0xa8a54f as ::core::ffi::c_int as uint32,
    0xa8a55b as ::core::ffi::c_int as uint32,
    0xa8a567 as ::core::ffi::c_int as uint32,
    0xa8a573 as ::core::ffi::c_int as uint32,
    0xa8a57f as ::core::ffi::c_int as uint32,
    0xa8a58b as ::core::ffi::c_int as uint32,
    0x84d7cb as ::core::ffi::c_int as uint32,
    0x84d7e2 as ::core::ffi::c_int as uint32,
    0x84d7f4 as ::core::ffi::c_int as uint32,
    0x84d803 as ::core::ffi::c_int as uint32,
    0x8888cd as ::core::ffi::c_int as uint32,
    0x8888f0 as ::core::ffi::c_int as uint32,
    0x8888e3 as ::core::ffi::c_int as uint32,
    0x80a5f3 as ::core::ffi::c_int as uint32,
    0x80a845 as ::core::ffi::c_int as uint32,
    0x80a925 as ::core::ffi::c_int as uint32,
    0x80a6aa as ::core::ffi::c_int as uint32,
    0x948d94 as ::core::ffi::c_int as uint32,
    0x948e25 as ::core::ffi::c_int as uint32,
    0x9082a8 as ::core::ffi::c_int as uint32,
    0x9082ae as ::core::ffi::c_int as uint32,
    0xa48ca1 as ::core::ffi::c_int as uint32,
    0xa48ca4 as ::core::ffi::c_int as uint32,
];
static mut kPatchedCarrysOrg: [uint8; 155] = [0; 155];
#[no_mangle]
pub unsafe extern "C" fn PatchBugs(mut mode: uint32, mut addr: uint32) -> uint32 {
    hookmode = mode;
    hookadr = addr;
    hookcnt = 0 as uint32;
    if FixBugHook(0x86ef35 as uint32) {
        (*g_cpu).x = (*g_cpu).y;
    } else if FixBugHook(0x86ef45 as uint32) {
        (*g_cpu).z = (*g_cpu).a as ::core::ffi::c_int == 0 as ::core::ffi::c_int;
    } else if FixBugHook(0x818ab8 as uint32) {
        if (*g_cpu).y as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            (*g_cpu).pc = 0x8b1f as uint16_t;
        }
    } else if FixBugHook(0xa794ba as uint32) {
        (*g_cpu).y = (*g_cpu).x;
    } else if FixBugHook(0xa7b968 as uint32) {
        (*g_cpu).x = *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint16_t;
    } else if FixBugHook(0xa7b963 as uint32) {
        (*g_cpu).x = *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint16_t;
    } else if FixBugHook(0xa496c8 as uint32) {
        (*g_cpu).a = 0 as uint16_t;
    } else if FixBugHook(0x9085aa as uint32) {
        (*g_cpu).a = *((&raw mut g_ram as *mut uint8).offset(0xb3e as ::core::ffi::c_int as isize)
            as *mut uint16) as uint16_t;
    } else if FixBugHook(0xa29044 as uint32) as ::core::ffi::c_int != 0
        || FixBugHook(0xa2905d as uint32) as ::core::ffi::c_int != 0
    {
        (*g_cpu).c = (*g_cpu).a as ::core::ffi::c_int == 0 as ::core::ffi::c_int;
    } else if FixBugHook(0xa29051 as uint32) {
        (*g_cpu).a = (*g_cpu).a.wrapping_sub(1);
    } else if FixBugHook(0xa5931c as uint32) {
        (*g_cpu).x = *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint16_t;
    } else if FixBugHook(0x80ada4 as uint32) {
        (*g_cpu).a = *((&raw mut g_ram as *mut uint8).offset(0x919 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint16_t;
    } else if FixBugHook(0x80add9 as uint32) {
        (*g_cpu).a = *((&raw mut g_ram as *mut uint8).offset(0x919 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint16_t;
    } else if FixBugHook(0x80ad4d as uint32) {
        (*g_cpu).a = *((&raw mut g_ram as *mut uint8).offset(0x917 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint16_t;
    } else if FixBugHook(0x80ad77 as uint32) {
        (*g_cpu).a = *((&raw mut g_ram as *mut uint8).offset(0x917 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint16_t;
    } else if FixBugHook(0x9381db as uint32) {
        let mut k: ::core::ffi::c_int = (*g_cpu).x as ::core::ffi::c_int;
        let mut ip: ::core::ffi::c_int =
            *((&raw mut g_ram as *mut uint8).offset(0xc40 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((k >> 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int;
        let mut delta: ::core::ffi::c_int = if *((&raw mut g_ram as *mut uint8)
            .offset(0xc54 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            == 1 as ::core::ffi::c_int
            && (*get_ProjectileInstr(ip as uint16)).timer as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
        {
            0 as ::core::ffi::c_int
        } else {
            -(8 as ::core::ffi::c_int)
        };
        (*g_cpu).a =
            ((*g_cpu).a as ::core::ffi::c_int + (8 as ::core::ffi::c_int + delta)) as uint16_t;
    } else if FixBugHook(0x86b701 as uint32) {
        (*g_cpu).x = (*g_cpu).y;
    } else if FixBugHook(0x8fc1b0 as uint32) {
        (*g_cpu).x = (*g_cpu).a;
    } else if !FixBugHook(0x80804f as uint32) {
        if FixBugHook(0x829325 as uint32) {
            (*g_cpu).db = 0x82 as uint8_t;
        } else if FixBugHook(0x848acd as uint32) {
            (*g_cpu).a = 0 as uint16_t;
        } else if FixBugHook(0xa7ceb2 as uint32) {
            (*g_cpu).x = *((&raw mut g_ram as *mut uint8)
                .offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint16_t;
        } else if FixBugHook(0x91cd44 as uint32) {
            if (*g_cpu).x as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                (*g_cpu).pc = 0xcd52 as uint16_t;
            }
        } else if FixBugHook(0x82afd3 as uint32) {
            if *((&raw mut g_ram as *mut uint8).offset(0x755 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint8 as ::core::ffi::c_int
                != 1 as ::core::ffi::c_int
            {
                return 0x82afd9 as uint32;
            }
        } else if FixBugHook(0x82b0cd as uint32) {
            if *((&raw mut g_ram as *mut uint8).offset(0x755 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint8 as ::core::ffi::c_int
                != 2 as ::core::ffi::c_int
            {
                return 0x82afd9 as uint32;
            }
        } else if FixBugHook(0x82b15b as uint32) {
            if *((&raw mut g_ram as *mut uint8).offset(0x755 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint8 as ::core::ffi::c_int
                != 3 as ::core::ffi::c_int
            {
                return 0x82afd9 as uint32;
            }
        } else if FixBugHook(0xa2d38c as uint32) {
            (*g_cpu).x = *((&raw mut g_ram as *mut uint8)
                .offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint16_t;
        } else if FixBugHook(0xa4970f as uint32) {
            (*g_cpu).a =
                ((*g_cpu).a as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as uint16_t;
            (*g_cpu).y = ((*g_cpu).x as ::core::ffi::c_int & 0x7 as ::core::ffi::c_int) as uint16_t;
        } else if FixBugHook(0xa496e0 as uint32) {
            if (*g_cpu).x as ::core::ffi::c_int > 48 as ::core::ffi::c_int {
                *((&raw mut g_ram as *mut uint8).offset(0x690 as ::core::ffi::c_int as isize)
                    as *mut uint16) = (*g_cpu).x as uint16;
                (*g_cpu).mf = 0 as ::core::ffi::c_int != 0;
                return 0xa497ce as uint32;
            }
        } else if FixBugHook(0x91da89 as uint32) {
            if *((&raw mut g_ram as *mut uint8).offset(0xace as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                > 6 as ::core::ffi::c_int
            {
                *((&raw mut g_ram as *mut uint8).offset(0xace as ::core::ffi::c_int as isize)
                    as *mut uint16) = 6 as uint16;
            }
        } else if FixBugHook(0x828d56 as uint32) {
            WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0x80 as uint8);
        } else if FixBugHook(0x88afcf as uint32) {
            if (*g_cpu).a as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
                (*g_cpu).a = 0 as uint16_t;
            }
        } else if FixBugHook(0x88aff2 as uint32) {
            if ((*g_cpu).a as ::core::ffi::c_int) < 256 as ::core::ffi::c_int {
                (*g_cpu).a = 256 as uint16_t;
            }
        } else if FixBugHook(0x8189bd as uint32) {
            if (*g_cpu).y as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                return 0x818a35 as uint32;
            }
        } else if FixBugHook(0xa29bc1 as uint32) {
            (*g_cpu).a = 1 as uint16_t;
        } else if FixBugHook(0x82e910 as uint32) {
            *((&raw mut g_ram as *mut uint8).offset(22 as ::core::ffi::c_int as isize) as *mut uint8
                as *mut uint16) = 0 as uint16;
        } else if FixBugHook(0x90a4c8 as uint32) {
            *((&raw mut g_ram as *mut uint8).offset(18 as ::core::ffi::c_int as isize) as *mut uint8
                as *mut uint16) = 0 as uint16;
        } else if FixBugHook(0xa99f60 as uint32) {
            *((&raw mut g_ram as *mut uint8).offset(22 as ::core::ffi::c_int as isize) as *mut uint8
                as *mut uint16) = 1 as uint16;
        } else if FixBugHook(0x94a85b as uint32) {
            memset(
                (&raw mut g_ram as *mut uint8).offset(0xd82 as ::core::ffi::c_int as isize)
                    as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                8 as size_t,
            );
        } else if FixBugHook(0xa0a35c as uint32) {
            *((&raw mut g_ram as *mut uint8).offset(18 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut g_ram as *mut uint8)
                .offset(0xcea as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as uint16;
            *((&raw mut g_ram as *mut uint8).offset(20 as ::core::ffi::c_int as isize)
                as *mut uint16) = (*((&raw mut g_ram as *mut uint8)
                .offset(18 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + (*((&raw mut g_ram as *mut uint8).offset(18 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int)
                >> 1 as ::core::ffi::c_int) as uint16;
        } else if FixBugHook(0xa7b049 as uint32) {
            (*g_cpu).x = 0 as uint16_t;
        } else if FixBugHook(0xa5a018 as uint32) {
            *((&raw mut g_ram as *mut uint8).offset(0xe24 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
        } else if FixBugHook(0xb39ddb as uint32) {
            let mut E: *mut Enemy_Botwoon = Get_Botwoon(
                *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int,
            );
            *((&raw mut g_ram as *mut uint8).offset(18 as ::core::ffi::c_int as isize)
                as *mut uint16) = ((*E).base.x_pos as ::core::ffi::c_int
                - (*E).botwoon_var_56 as ::core::ffi::c_int)
                as uint16;
            *((&raw mut g_ram as *mut uint8).offset(20 as ::core::ffi::c_int as isize)
                as *mut uint16) = ((*E).base.y_pos as ::core::ffi::c_int
                - (*E).botwoon_var_57 as ::core::ffi::c_int)
                as uint16;
        } else if FixBugHook(0xb39e13 as uint32) {
            let mut E_0: *mut Enemy_Botwoon = Get_Botwoon(
                *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int,
            );
            (*E_0).botwoon_var_45 = ((*E_0).botwoon_var_45 as ::core::ffi::c_int
                + (*((&raw mut g_ram as *mut uint8).offset(22 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - (*E_0).botwoon_var_45 as ::core::ffi::c_int) as int8
                    as ::core::ffi::c_int
                    * 3 as ::core::ffi::c_int
                    / 4 as ::core::ffi::c_int) as uint8
                as uint16;
            *((&raw mut g_ram as *mut uint8).offset(22 as ::core::ffi::c_int as isize)
                as *mut uint16) = (*E_0).botwoon_var_45;
        }
    }
    return 0 as uint32;
}
#[no_mangle]
pub unsafe extern "C" fn RunPatchBugHook(mut addr: uint32) -> ::core::ffi::c_int {
    let mut new_pc: uint32 = PatchBugs(2 as uint32, addr);
    if hookmode == 3 as uint32 {
        if new_pc == 0 as uint32 {
            return hook_fixbug_orgbyte[hookcnt as usize] as ::core::ffi::c_int;
        } else {
            (*g_cpu).k = (new_pc >> 16 as ::core::ffi::c_int) as uint8_t;
            (*g_cpu).pc = (new_pc & 0xffff as uint32).wrapping_add(1 as uint32) as uint16_t;
            return *SnesRomPtr(new_pc) as ::core::ffi::c_int;
        }
    }
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn CpuOpcodeHook(mut addr: uint32) -> ::core::ffi::c_int {
    let mut i: size_t = 0 as size_t;
    while i
        != (::core::mem::size_of::<[uint32; 155]>() as usize)
            .wrapping_div(::core::mem::size_of::<uint32>() as usize)
    {
        if addr == kPatchedCarrys[i as usize] {
            return kPatchedCarrysOrg[i as usize] as ::core::ffi::c_int;
        }
        i = i.wrapping_add(1);
    }
    let mut i_0: ::core::ffi::c_int = RunPatchBugHook(addr);
    if i_0 >= 0 as ::core::ffi::c_int {
        return i_0;
    }
    '_c2rust_label: {
        __assert_fail(
            b"0\0" as *const u8 as *const ::core::ffi::c_char,
            b"src/sm_cpu_infra.c\0" as *const u8 as *const ::core::ffi::c_char,
            455 as ::core::ffi::c_uint,
            b"int CpuOpcodeHook(uint32)\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn HookedFunctionRts(mut is_long: ::core::ffi::c_int) -> bool {
    if g_calling_asm_from_c {
        g_calling_asm_from_c_ret = is_long;
        g_calling_asm_from_c = false_0 != 0;
        return false_0 != 0;
    }
    '_c2rust_label: {
        __assert_fail(
            b"0\0" as *const u8 as *const ::core::ffi::c_char,
            b"src/sm_cpu_infra.c\0" as *const u8 as *const ::core::ffi::c_char,
            465 as ::core::ffi::c_uint,
            b"_Bool HookedFunctionRts(int)\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    return false_0 != 0;
}
unsafe extern "C" fn VerifySnapshotsEq(
    mut b: *mut Snapshot,
    mut a: *mut Snapshot,
    mut prev: *mut Snapshot,
) {
    // Keep checking every frame, but bound headless logs to the first failure.
    let report_details = !VERIFY_ACTIVE || VERIFY_FAILURES == 0;
    memcpy(
        (&raw mut (*b).ram as *mut uint8).offset(0 as ::core::ffi::c_int as isize) as *mut uint8
            as *mut ::core::ffi::c_void,
        (&raw mut (*a).ram as *mut uint8).offset(0 as ::core::ffi::c_int as isize) as *mut uint8
            as *const ::core::ffi::c_void,
        0x51 as size_t,
    );
    memcpy(
        (&raw mut (*b).ram as *mut uint8).offset(0x1f5b as ::core::ffi::c_int as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*a).ram as *mut uint8).offset(0x1f5b as ::core::ffi::c_int as isize)
            as *mut uint8 as *const ::core::ffi::c_void,
        (0x100 as ::core::ffi::c_int - 0x5b as ::core::ffi::c_int) as size_t,
    );
    memcpy(
        (&raw mut (*b).ram as *mut uint8).offset(0xad as ::core::ffi::c_int as isize) as *mut uint8
            as *mut ::core::ffi::c_void,
        (&raw mut (*a).ram as *mut uint8).offset(0xad as ::core::ffi::c_int as isize) as *mut uint8
            as *const ::core::ffi::c_void,
        4 as size_t,
    );
    memcpy(
        (&raw mut (*b).ram as *mut uint8).offset(0x5e7 as ::core::ffi::c_int as isize) as *mut uint8
            as *mut ::core::ffi::c_void,
        (&raw mut (*a).ram as *mut uint8).offset(0x5e7 as ::core::ffi::c_int as isize) as *mut uint8
            as *const ::core::ffi::c_void,
        14 as size_t,
    );
    memcpy(
        (&raw mut (*b).ram as *mut uint8).offset(0x5bc as ::core::ffi::c_int as isize) as *mut uint8
            as *mut ::core::ffi::c_void,
        (&raw mut (*a).ram as *mut uint8).offset(0x5bc as ::core::ffi::c_int as isize) as *mut uint8
            as *const ::core::ffi::c_void,
        9 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0x60b as ::core::ffi::c_int as isize) as *mut uint8
            as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0x60b as ::core::ffi::c_int as isize) as *mut uint8
            as *const ::core::ffi::c_void,
        6 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0x611 as ::core::ffi::c_int as isize) as *mut uint8
            as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0x611 as ::core::ffi::c_int as isize) as *mut uint8
            as *const ::core::ffi::c_void,
        6 as size_t,
    );
    memcpy(
        (&raw mut (*b).ram as *mut uint8).offset(0x641 as ::core::ffi::c_int as isize) as *mut uint8
            as *mut ::core::ffi::c_void,
        (&raw mut (*a).ram as *mut uint8).offset(0x641 as ::core::ffi::c_int as isize) as *mut uint8
            as *const ::core::ffi::c_void,
        2 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0x77e as ::core::ffi::c_int as isize) as *mut uint8
            as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0x77e as ::core::ffi::c_int as isize) as *mut uint8
            as *const ::core::ffi::c_void,
        5 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0x78f as ::core::ffi::c_int as isize) as *mut uint8
            as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0x78f as ::core::ffi::c_int as isize) as *mut uint8
            as *const ::core::ffi::c_void,
        2 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0x7b7 as ::core::ffi::c_int as isize) as *mut uint8
            as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0x7b7 as ::core::ffi::c_int as isize) as *mut uint8
            as *const ::core::ffi::c_void,
        2 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0x933 as ::core::ffi::c_int as isize) as *mut uint8
            as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0x933 as ::core::ffi::c_int as isize) as *mut uint8
            as *const ::core::ffi::c_void,
        10 as size_t,
    );
    memcpy(
        (&raw mut (*b).ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize) as *mut uint8
            as *mut ::core::ffi::c_void,
        (&raw mut (*a).ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize) as *mut uint8
            as *const ::core::ffi::c_void,
        2 as size_t,
    );
    memcpy(
        (&raw mut (*b).ram as *mut uint8).offset(0xb24 as ::core::ffi::c_int as isize) as *mut uint8
            as *mut ::core::ffi::c_void,
        (&raw mut (*a).ram as *mut uint8).offset(0xb24 as ::core::ffi::c_int as isize) as *mut uint8
            as *const ::core::ffi::c_void,
        4 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0xd1e as ::core::ffi::c_int as isize) as *mut uint8
            as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0xd1e as ::core::ffi::c_int as isize) as *mut uint8
            as *const ::core::ffi::c_void,
        2 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0xd82 as ::core::ffi::c_int as isize) as *mut uint8
            as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0xd82 as ::core::ffi::c_int as isize) as *mut uint8
            as *const ::core::ffi::c_void,
        8 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0xd9c as ::core::ffi::c_int as isize) as *mut uint8
            as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0xd9c as ::core::ffi::c_int as isize) as *mut uint8
            as *const ::core::ffi::c_void,
        2 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0xdd2 as ::core::ffi::c_int as isize) as *mut uint8
            as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0xdd2 as ::core::ffi::c_int as isize) as *mut uint8
            as *const ::core::ffi::c_void,
        6 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0xd8a as ::core::ffi::c_int as isize) as *mut uint8
            as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0xd8a as ::core::ffi::c_int as isize) as *mut uint8
            as *const ::core::ffi::c_void,
        6 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0xe20 as ::core::ffi::c_int as isize) as *mut uint8
            as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0xe20 as ::core::ffi::c_int as isize) as *mut uint8
            as *const ::core::ffi::c_void,
        (0xe46 as ::core::ffi::c_int - 0xe20 as ::core::ffi::c_int) as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize) as *mut uint8
            as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize) as *mut uint8
            as *const ::core::ffi::c_void,
        2 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0xe02 as ::core::ffi::c_int as isize) as *mut uint8
            as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0xe02 as ::core::ffi::c_int as isize) as *mut uint8
            as *const ::core::ffi::c_void,
        2 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0xe4a as ::core::ffi::c_int as isize) as *mut uint8
            as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0xe4a as ::core::ffi::c_int as isize) as *mut uint8
            as *const ::core::ffi::c_void,
        2 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0xe56 as ::core::ffi::c_int as isize) as *mut uint8
            as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0xe56 as ::core::ffi::c_int as isize) as *mut uint8
            as *const ::core::ffi::c_void,
        4 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0x1784 as ::core::ffi::c_int as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0x1784 as ::core::ffi::c_int as isize)
            as *mut uint8 as *const ::core::ffi::c_void,
        8 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0x1790 as ::core::ffi::c_int as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0x1790 as ::core::ffi::c_int as isize)
            as *mut uint8 as *const ::core::ffi::c_void,
        4 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0x17a8 as ::core::ffi::c_int as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0x17a8 as ::core::ffi::c_int as isize)
            as *mut uint8 as *const ::core::ffi::c_void,
        4 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0x1834 as ::core::ffi::c_int as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0x1834 as ::core::ffi::c_int as isize)
            as *mut uint8 as *const ::core::ffi::c_void,
        8 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0x184a as ::core::ffi::c_int as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0x184a as ::core::ffi::c_int as isize)
            as *mut uint8 as *const ::core::ffi::c_void,
        18 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0x186e as ::core::ffi::c_int as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0x186e as ::core::ffi::c_int as isize)
            as *mut uint8 as *const ::core::ffi::c_void,
        (16 as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0x18a6 as ::core::ffi::c_int as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0x18a6 as ::core::ffi::c_int as isize)
            as *mut uint8 as *const ::core::ffi::c_void,
        2 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0x189a as ::core::ffi::c_int as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0x189a as ::core::ffi::c_int as isize)
            as *mut uint8 as *const ::core::ffi::c_void,
        12 as size_t,
    );
    memcpy(
        (&raw mut (*b).ram as *mut uint8).offset(0x1966 as ::core::ffi::c_int as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*a).ram as *mut uint8).offset(0x1966 as ::core::ffi::c_int as isize)
            as *mut uint8 as *const ::core::ffi::c_void,
        6 as size_t,
    );
    memcpy(
        (&raw mut (*b).ram as *mut uint8).offset(0x1993 as ::core::ffi::c_int as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*a).ram as *mut uint8).offset(0x1993 as ::core::ffi::c_int as isize)
            as *mut uint8 as *const ::core::ffi::c_void,
        2 as size_t,
    );
    memcpy(
        (&raw mut (*b).ram as *mut uint8).offset(0x19b3 as ::core::ffi::c_int as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*a).ram as *mut uint8).offset(0x19b3 as ::core::ffi::c_int as isize)
            as *mut uint8 as *const ::core::ffi::c_void,
        2 as size_t,
    );
    memcpy(
        (&raw mut (*b).ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*a).ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint8 as *const ::core::ffi::c_void,
        2 as size_t,
    );
    memcpy(
        (&raw mut (*b).ram as *mut uint8).offset(0x1b9d as ::core::ffi::c_int as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*a).ram as *mut uint8).offset(0x1b9d as ::core::ffi::c_int as isize)
            as *mut uint8 as *const ::core::ffi::c_void,
        2 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0x1e77 as ::core::ffi::c_int as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0x1e77 as ::core::ffi::c_int as isize)
            as *mut uint8 as *const ::core::ffi::c_void,
        2 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0x9100 as ::core::ffi::c_int as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0x9100 as ::core::ffi::c_int as isize)
            as *mut uint8 as *const ::core::ffi::c_void,
        (0x1cc as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0x9800 as ::core::ffi::c_int as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0x9800 as ::core::ffi::c_int as isize)
            as *mut uint8 as *const ::core::ffi::c_void,
        (0x1cc as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0x99cc as ::core::ffi::c_int as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0x99cc as ::core::ffi::c_int as isize)
            as *mut uint8 as *const ::core::ffi::c_void,
        2 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0xef74 as ::core::ffi::c_int as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0xef74 as ::core::ffi::c_int as isize)
            as *mut uint8 as *const ::core::ffi::c_void,
        4 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0xf37a as ::core::ffi::c_int as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0xf37a as ::core::ffi::c_int as isize)
            as *mut uint8 as *const ::core::ffi::c_void,
        6 as size_t,
    );
    if memcmp(
        &raw mut (*b).ram as *mut uint8 as *const ::core::ffi::c_void,
        &raw mut (*a).ram as *mut uint8 as *const ::core::ffi::c_void,
        0x20000 as ::core::ffi::c_int as size_t,
    ) != 0
    {
        g_fail = true_0 != 0;
        if report_details {
            fprintf(
                stderr,
                b"@%d: Memory compare failed (mine != theirs, prev):\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                snes_frame_counter,
            );
            let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut i: size_t = 0 as size_t;
            while i < 0x20000 as ::core::ffi::c_int as size_t {
                if (*a).ram[i as usize] as ::core::ffi::c_int
                    != (*b).ram[i as usize] as ::core::ffi::c_int
                {
                    j += 1;
                    if j < 256 as ::core::ffi::c_int {
                        if (i & 1 as size_t == 0 as size_t
                            || i < 0x10000 as ::core::ffi::c_int as size_t)
                            && (*a).ram[i.wrapping_add(1 as size_t) as usize] as ::core::ffi::c_int
                                != (*b).ram[i.wrapping_add(1 as size_t) as usize]
                                    as ::core::ffi::c_int
                        {
                            fprintf(
                                stderr,
                                b"0x%.6X: %.4X != %.4X (%.4X)\n\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                i as ::core::ffi::c_int,
                                *((&raw mut (*b).ram as *mut uint8).offset(i as isize) as *mut uint8
                                    as *mut uint16)
                                    as ::core::ffi::c_int,
                                *((&raw mut (*a).ram as *mut uint8).offset(i as isize) as *mut uint8
                                    as *mut uint16)
                                    as ::core::ffi::c_int,
                                *((&raw mut (*prev).ram as *mut uint8).offset(i as isize)
                                    as *mut uint8 as *mut uint16)
                                    as ::core::ffi::c_int,
                            );
                            i = i.wrapping_add(1);
                            j += 1;
                        } else {
                            fprintf(
                                stderr,
                                b"0x%.6X: %.2X != %.2X (%.2X)\n\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                i as ::core::ffi::c_int,
                                (*b).ram[i as usize] as ::core::ffi::c_int,
                                (*a).ram[i as usize] as ::core::ffi::c_int,
                                (*prev).ram[i as usize] as ::core::ffi::c_int,
                            );
                        }
                    }
                }
                i = i.wrapping_add(1);
            }
            if j != 0 {
                g_fail = true_0 != 0;
            }
            fprintf(
                stderr,
                b"  total of %d failed bytes\n\0" as *const u8 as *const ::core::ffi::c_char,
                j,
            );
        }
    }
    if memcmp(
        &raw mut (*b).sram as *mut uint8 as *const ::core::ffi::c_void,
        &raw mut (*a).sram as *mut uint8 as *const ::core::ffi::c_void,
        0x2000 as size_t,
    ) != 0
    {
        g_fail = true_0 != 0;
        if report_details {
            fprintf(
                stderr,
                b"@%d: SRAM compare failed (mine != theirs, prev):\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                snes_frame_counter,
            );
            let mut j_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut i_0: size_t = 0 as size_t;
            while i_0 < 0x2000 as size_t {
                if (*a).sram[i_0 as usize] as ::core::ffi::c_int
                    != (*b).sram[i_0 as usize] as ::core::ffi::c_int
                {
                    j_0 += 1;
                    if j_0 < 128 as ::core::ffi::c_int {
                        if i_0 & 1 as size_t == 0 as size_t
                            && (*a).sram[i_0.wrapping_add(1 as size_t) as usize]
                                as ::core::ffi::c_int
                                != (*b).sram[i_0.wrapping_add(1 as size_t) as usize]
                                    as ::core::ffi::c_int
                        {
                            fprintf(
                                stderr,
                                b"0x%.6X: %.4X != %.4X (%.4X)\n\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                i_0 as ::core::ffi::c_int,
                                *((&raw mut (*b).sram as *mut uint8).offset(i_0 as isize)
                                    as *mut uint8 as *mut uint16)
                                    as ::core::ffi::c_int,
                                *((&raw mut (*a).sram as *mut uint8).offset(i_0 as isize)
                                    as *mut uint8 as *mut uint16)
                                    as ::core::ffi::c_int,
                                *((&raw mut (*prev).sram as *mut uint8).offset(i_0 as isize)
                                    as *mut uint8 as *mut uint16)
                                    as ::core::ffi::c_int,
                            );
                            i_0 = i_0.wrapping_add(1);
                            j_0 += 1;
                        } else {
                            fprintf(
                                stderr,
                                b"0x%.6X: %.2X != %.2X (%.2X)\n\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                i_0 as ::core::ffi::c_int,
                                (*b).sram[i_0 as usize] as ::core::ffi::c_int,
                                (*a).sram[i_0 as usize] as ::core::ffi::c_int,
                                (*prev).sram[i_0 as usize] as ::core::ffi::c_int,
                            );
                        }
                    }
                }
                i_0 = i_0.wrapping_add(1);
            }
            if j_0 != 0 {
                g_fail = true_0 != 0;
            }
            fprintf(
                stderr,
                b"  total of %d failed bytes\n\0" as *const u8 as *const ::core::ffi::c_char,
                j_0,
            );
        }
    }
    if memcmp(
        &raw mut (*b).vram as *mut uint16 as *const ::core::ffi::c_void,
        &raw mut (*a).vram as *mut uint16 as *const ::core::ffi::c_void,
        (::core::mem::size_of::<uint16>() as size_t).wrapping_mul(0x8000 as size_t),
    ) != 0
    {
        g_fail = true_0 != 0;
        if report_details {
            fprintf(
                stderr,
                b"@%d: VRAM compare failed (mine != theirs, prev):\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                snes_frame_counter,
            );
            let mut i_1: size_t = 0 as size_t;
            let mut j_1: size_t = 0 as size_t;
            while i_1 < 0x8000 as size_t {
                if (*a).vram[i_1 as usize] as ::core::ffi::c_int
                    != (*b).vram[i_1 as usize] as ::core::ffi::c_int
                {
                    fprintf(
                        stderr,
                        b"0x%.6X: %.4X != %.4X (%.4X)\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        i_1 as ::core::ffi::c_int,
                        (*b).vram[i_1 as usize] as ::core::ffi::c_int,
                        (*a).vram[i_1 as usize] as ::core::ffi::c_int,
                        (*prev).vram[i_1 as usize] as ::core::ffi::c_int,
                    );
                    g_fail = true_0 != 0;
                    j_1 = j_1.wrapping_add(1);
                    if j_1 >= 32 as size_t {
                        break;
                    }
                }
                i_1 = i_1.wrapping_add(1);
            }
        }
    }
    if memcmp(
        &raw mut (*b).oam as *mut uint16 as *const ::core::ffi::c_void,
        &raw mut (*a).oam as *mut uint16 as *const ::core::ffi::c_void,
        (::core::mem::size_of::<uint16>() as size_t).wrapping_mul(0x120 as size_t),
    ) != 0
    {
        g_fail = true_0 != 0;
        if report_details {
            fprintf(
                stderr,
                b"@%d: VRAM OAM compare failed (mine != theirs, prev):\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                snes_frame_counter,
            );
            let mut i_2: size_t = 0 as size_t;
            let mut j_2: size_t = 0 as size_t;
            while i_2 < 0x120 as size_t {
                if (*a).oam[i_2 as usize] as ::core::ffi::c_int
                    != (*b).oam[i_2 as usize] as ::core::ffi::c_int
                {
                    fprintf(
                        stderr,
                        b"0x%.6X: %.4X != %.4X (%.4X)\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        i_2 as ::core::ffi::c_int,
                        (*b).oam[i_2 as usize] as ::core::ffi::c_int,
                        (*a).oam[i_2 as usize] as ::core::ffi::c_int,
                        (*prev).oam[i_2 as usize] as ::core::ffi::c_int,
                    );
                    g_fail = true_0 != 0;
                    j_2 = j_2.wrapping_add(1);
                    if j_2 >= 16 as size_t {
                        break;
                    }
                }
                i_2 = i_2.wrapping_add(1);
            }
        }
    }
}
unsafe extern "C" fn MakeSnapshot(mut s: *mut Snapshot) {
    let mut c: *mut Cpu = g_cpu;
    (*s).a = (*c).a as uint16;
    (*s).x = (*c).x as uint16;
    (*s).y = (*c).y as uint16;
    (*s).sp = (*c).sp as uint16;
    (*s).dp = (*c).dp as uint16;
    (*s).db = (*c).db as uint8;
    (*s).pc = (*c).pc as uint16;
    (*s).k = (*c).k as uint8;
    (*s).flags = cpu_getFlags(c) as uint8;
    (*s).vTimer = (*g_snes).vTimer;
    memcpy(
        &raw mut (*s).ram as *mut uint8 as *mut ::core::ffi::c_void,
        (*g_snes).ram as *const ::core::ffi::c_void,
        0x20000 as ::core::ffi::c_int as size_t,
    );
    memcpy(
        &raw mut (*s).sram as *mut uint8 as *mut ::core::ffi::c_void,
        (*(*g_snes).cart).ram as *const ::core::ffi::c_void,
        (*(*g_snes).cart).ramSize as size_t,
    );
    memcpy(
        &raw mut (*s).vram as *mut uint16 as *mut ::core::ffi::c_void,
        &raw mut (*(*g_snes).ppu).vram as *mut uint16_t as *const ::core::ffi::c_void,
        (::core::mem::size_of::<uint16>() as size_t).wrapping_mul(0x8000 as size_t),
    );
    memcpy(
        &raw mut (*s).oam as *mut uint16 as *mut ::core::ffi::c_void,
        &raw mut (*(*g_snes).ppu).oam as *mut uint16_t as *const ::core::ffi::c_void,
        (::core::mem::size_of::<uint16>() as size_t).wrapping_mul(0x120 as size_t),
    );
}
unsafe extern "C" fn RestoreSnapshot(mut s: *mut Snapshot) {
    let mut c: *mut Cpu = g_cpu;
    (*c).a = (*s).a as uint16_t;
    (*c).x = (*s).x as uint16_t;
    (*c).y = (*s).y as uint16_t;
    (*c).sp = (*s).sp as uint16_t;
    (*c).dp = (*s).dp as uint16_t;
    (*c).db = (*s).db as uint8_t;
    (*c).pc = (*s).pc as uint16_t;
    (*c).k = (*s).k as uint8_t;
    (*g_snes).vTimer = (*s).vTimer;
    cpu_setFlags(c, (*s).flags as uint8_t);
    memcpy(
        (*g_snes).ram as *mut ::core::ffi::c_void,
        &raw mut (*s).ram as *mut uint8 as *const ::core::ffi::c_void,
        0x20000 as ::core::ffi::c_int as size_t,
    );
    memcpy(
        (*(*g_snes).cart).ram as *mut ::core::ffi::c_void,
        &raw mut (*s).sram as *mut uint8 as *const ::core::ffi::c_void,
        0x2000 as size_t,
    );
    memcpy(
        &raw mut (*(*g_snes).ppu).vram as *mut uint16_t as *mut ::core::ffi::c_void,
        &raw mut (*s).vram as *mut uint16 as *const ::core::ffi::c_void,
        (::core::mem::size_of::<uint16>() as size_t).wrapping_mul(0x8000 as size_t),
    );
    memcpy(
        &raw mut (*(*g_snes).ppu).oam as *mut uint16_t as *mut ::core::ffi::c_void,
        &raw mut (*s).oam as *mut uint16 as *const ::core::ffi::c_void,
        (::core::mem::size_of::<uint16>() as size_t).wrapping_mul(0x120 as size_t),
    );
}
#[no_mangle]
pub unsafe extern "C" fn RunAsmCode(
    mut pc: uint32,
    mut a: uint16,
    mut x: uint16,
    mut y: uint16,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut org_sp: uint16 = (*g_cpu).sp as uint16;
    let mut org_pc: uint16 = (*g_cpu).pc as uint16;
    let mut org_b: uint8 = (*g_cpu).db as uint8;
    let mut org_dp: uint16 = (*g_cpu).dp as uint16;
    printf(b"RunAsmCode!\n\0" as *const u8 as *const ::core::ffi::c_char);
    g_ram[0x1ffff as ::core::ffi::c_int as usize] = 1 as uint8;
    let mut dc: bool = (*g_snes).debug_cycles;
    (*g_cpu).db = (pc >> 16 as ::core::ffi::c_int) as uint8_t;
    (*g_cpu).a = a as uint16_t;
    (*g_cpu).x = x as uint16_t;
    (*g_cpu).y = y as uint16_t;
    (*g_cpu).spBreakpoint = (*g_cpu).sp;
    (*g_cpu).k = (pc >> 16 as ::core::ffi::c_int) as uint8_t;
    (*g_cpu).pc = (pc & 0xffff as uint32) as uint16_t;
    (*g_cpu).mf = flags & 1 as ::core::ffi::c_int != 0;
    (*g_cpu).xf = (flags & 2 as ::core::ffi::c_int) >> 1 as ::core::ffi::c_int != 0;
    g_calling_asm_from_c = true_0 != 0;
    while g_calling_asm_from_c {
        let mut pc_0: uint32 = (((*(*g_snes).cpu).k as ::core::ffi::c_int)
            << 16 as ::core::ffi::c_int
            | (*(*g_snes).cpu).pc as ::core::ffi::c_int) as uint32;
        if (*g_snes).debug_cycles {
            let mut line: [::core::ffi::c_char; 80] = [0; 80];
            getProcessorStateCpu(g_snes, &raw mut line as *mut ::core::ffi::c_char);
            puts(&raw mut line as *mut ::core::ffi::c_char);
            line[0 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_char;
        }
        cpu_runOpcode(g_cpu);
        while (*(*g_snes).dma).dmaBusy {
            dma_doDma((*g_snes).dma);
        }
        if flags & 1 as ::core::ffi::c_int != 0 {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < 10 as ::core::ffi::c_int {
                apu_cycle((*g_snes).apu);
                i += 1;
            }
        }
    }
    (*g_cpu).dp = org_dp as uint16_t;
    (*g_cpu).sp = org_sp as uint16_t;
    (*g_cpu).db = org_b as uint8_t;
    (*g_cpu).pc = org_pc as uint16_t;
    (*g_snes).debug_cycles = dc;
    return g_calling_asm_from_c_ret;
}
unsafe extern "C" fn loadRom(mut name: *const ::core::ffi::c_char, mut snes: *mut Snes) -> bool {
    let mut length: size_t = 0 as size_t;
    let mut file: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    file = ReadWholeFile(name, &raw mut length) as *mut uint8_t;
    if file.is_null() {
        puts(b"Failed to read file\0" as *const u8 as *const ::core::ffi::c_char);
        return false_0 != 0;
    }
    let mut result: bool = snes_loadRom(snes as *mut Snes, file, length as ::core::ffi::c_int);
    free(file as *mut ::core::ffi::c_void);
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn PatchBytes(mut addr: uint32, mut value: *const uint8, mut n: size_t) {
    let mut i: size_t = 0 as size_t;
    while i != n {
        *SnesRomPtr(addr).offset(i as isize) = *value.offset(i as isize) as uint8_t;
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn FixupCarry(mut addr: uint32) {
    *SnesRomPtr(addr) = 0 as uint8_t;
}
#[no_mangle]
pub static mut currently_installed_bug_fix_counter: uint16 = 0;
#[no_mangle]
pub unsafe extern "C" fn RtlUpdateSnesPatchForBugfix() {
    currently_installed_bug_fix_counter = *((&raw mut g_ram as *mut uint8)
        .offset(0xc2 as ::core::ffi::c_int as isize)
        as *mut uint16);
    let mut t: [uint8; 4] = [
        0x20 as ::core::ffi::c_int as uint8,
        0x50 as ::core::ffi::c_int as uint8,
        0x96 as ::core::ffi::c_int as uint8,
        0x60 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x8584a3 as uint32,
        &raw mut t as *mut uint8,
        ::core::mem::size_of::<[uint8; 4]>() as size_t,
    );
    let mut t_0: [uint8; 14] = [
        0x20 as ::core::ffi::c_int as uint8,
        0x36 as ::core::ffi::c_int as uint8,
        0x81 as ::core::ffi::c_int as uint8,
        0x22 as ::core::ffi::c_int as uint8,
        0x59 as ::core::ffi::c_int as uint8,
        0x94 as ::core::ffi::c_int as uint8,
        0x80 as ::core::ffi::c_int as uint8,
        0xc2 as ::core::ffi::c_int as uint8,
        0x30 as ::core::ffi::c_int as uint8,
        0xa5 as ::core::ffi::c_int as uint8,
        (if (*((&raw mut g_ram as *mut uint8).offset(0xc2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int)
            < 1 as ::core::ffi::c_int
        {
            0x8f as ::core::ffi::c_int
        } else {
            0x8b as ::core::ffi::c_int
        }) as uint8,
        0xf0 as ::core::ffi::c_int as uint8,
        0xf3 as ::core::ffi::c_int as uint8,
        0x60 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x859650 as uint32,
        &raw mut t_0 as *mut uint8,
        ::core::mem::size_of::<[uint8; 14]>() as size_t,
    );
    let mut t_1: [uint8; 2] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x8584cc as uint32,
        &raw mut t_1 as *mut uint8,
        ::core::mem::size_of::<[uint8; 2]>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn SnesInit(mut filename: *const ::core::ffi::c_char) -> *mut Snes {
    g_snes = snes_init(&raw mut g_ram as *mut uint8_t) as *mut Snes;
    g_cpu = (*g_snes).cpu;
    let mut loaded: bool = loadRom(filename, g_snes);
    if !loaded {
        return ::core::ptr::null_mut::<Snes>();
    }
    g_sram = (*(*g_snes).cart).ram as *mut uint8;
    g_rom = (*(*g_snes).cart).rom;
    RtlSetupEmuCallbacks(
        ::core::ptr::null_mut::<uint8>(),
        Some(RtlRunFrameCompare as unsafe extern "C" fn(uint16, ::core::ffi::c_int) -> ()),
        None,
    );
    *(&raw mut g_ram as *mut uint8).offset(0x611 as ::core::ffi::c_int as isize) = 1 as uint8;
    let mut t: [uint8; 3] = [
        0x20 as ::core::ffi::c_int as uint8,
        0xf as ::core::ffi::c_int as uint8,
        0xf7 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(0x82896b as uint32, &raw mut t as *mut uint8, 3 as size_t);
    let mut t_0: [uint8; 3] = [
        0x7c as ::core::ffi::c_int as uint8,
        0x81 as ::core::ffi::c_int as uint8,
        0x89 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(0x82f70f as uint32, &raw mut t_0 as *mut uint8, 3 as size_t);
    let mut t_1: [uint8; 2] = [
        0x5f as ::core::ffi::c_int as uint8,
        0xf7 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x8ba362 as uint32,
        &raw mut t_1 as *mut uint8,
        ::core::mem::size_of::<[uint8; 2]>() as size_t,
    );
    let mut t_2: [uint8; 5] = [
        0xc2 as ::core::ffi::c_int as uint8,
        0x20 as ::core::ffi::c_int as uint8,
        0x4c as ::core::ffi::c_int as uint8,
        0x67 as ::core::ffi::c_int as uint8,
        0xa3 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x8bf760 as uint32,
        &raw mut t_2 as *mut uint8,
        ::core::mem::size_of::<[uint8; 5]>() as size_t,
    );
    let mut t_3: [uint8; 2] = [
        0xa as ::core::ffi::c_int as uint8,
        0xa as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(0x8584b2 as uint32, &raw mut t_3 as *mut uint8, 2 as size_t);
    let mut t_4: [uint8; 9] = [
        0xc0 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0xf0 as ::core::ffi::c_int as uint8,
        0x3 as ::core::ffi::c_int as uint8,
        0x20 as ::core::ffi::c_int as uint8,
        0x64 as ::core::ffi::c_int as uint8,
        0x87 as ::core::ffi::c_int as uint8,
        0x60 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x84efd3 as uint32,
        &raw mut t_4 as *mut uint8,
        ::core::mem::size_of::<[uint8; 9]>() as size_t,
    );
    let mut t_5: [uint8; 2] = [
        0xd3 as ::core::ffi::c_int as uint8,
        0xef as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x848243 as uint32,
        &raw mut t_5 as *mut uint8,
        ::core::mem::size_of::<[uint8; 2]>() as size_t,
    );
    let mut t_6: [uint8; 6] = [
        0xac as ::core::ffi::c_int as uint8,
        0x91 as ::core::ffi::c_int as uint8,
        0x19 as ::core::ffi::c_int as uint8,
        0x4c as ::core::ffi::c_int as uint8,
        0x76 as ::core::ffi::c_int as uint8,
        0x86 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x86f4a6 as uint32,
        &raw mut t_6 as *mut uint8,
        ::core::mem::size_of::<[uint8; 6]>() as size_t,
    );
    let mut t_7: [uint8; 2] = [
        0xa6 as ::core::ffi::c_int as uint8,
        0xf4 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x8685bd as uint32,
        &raw mut t_7 as *mut uint8,
        ::core::mem::size_of::<[uint8; 2]>() as size_t,
    );
    let mut t_8: [uint8; 16] = [
        0xc2 as ::core::ffi::c_int as uint8,
        0x30 as ::core::ffi::c_int as uint8,
        0x22 as ::core::ffi::c_int as uint8,
        0x59 as ::core::ffi::c_int as uint8,
        0x94 as ::core::ffi::c_int as uint8,
        0x80 as ::core::ffi::c_int as uint8,
        0x20 as ::core::ffi::c_int as uint8,
        0x48 as ::core::ffi::c_int as uint8,
        0x89 as ::core::ffi::c_int as uint8,
        0x22 as ::core::ffi::c_int as uint8,
        0x38 as ::core::ffi::c_int as uint8,
        0x83 as ::core::ffi::c_int as uint8,
        0x80 as ::core::ffi::c_int as uint8,
        0x4c as ::core::ffi::c_int as uint8,
        0x13 as ::core::ffi::c_int as uint8,
        0xf7 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x82f713 as uint32,
        &raw mut t_8 as *mut uint8,
        ::core::mem::size_of::<[uint8; 16]>() as size_t,
    );
    let mut t_9: [uint8; 4] = [
        0x58 as ::core::ffi::c_int as uint8,
        0x4c as ::core::ffi::c_int as uint8,
        0x13 as ::core::ffi::c_int as uint8,
        0xf7 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x828944 as uint32,
        &raw mut t_9 as *mut uint8,
        ::core::mem::size_of::<[uint8; 4]>() as size_t,
    );
    let mut t_10: [uint8; 2] = [
        0x28 as ::core::ffi::c_int as uint8,
        0x60 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x82897a as uint32,
        &raw mut t_10 as *mut uint8,
        ::core::mem::size_of::<[uint8; 2]>() as size_t,
    );
    let mut t_11: [uint8; 7] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x80945c as uint32,
        &raw mut t_11 as *mut uint8,
        ::core::mem::size_of::<[uint8; 7]>() as size_t,
    );
    let mut t_12: [uint8; 8] = [
        0xbc as ::core::ffi::c_int as uint8,
        0xaa as ::core::ffi::c_int as uint8,
        0xf as ::core::ffi::c_int as uint8,
        0xc9 as ::core::ffi::c_int as uint8,
        0x6c as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0x10 as ::core::ffi::c_int as uint8,
        0x1a as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0xa8b237 as uint32,
        &raw mut t_12 as *mut uint8,
        ::core::mem::size_of::<[uint8; 8]>() as size_t,
    );
    let mut t_13: [uint8; 6] = [
        0x49 as ::core::ffi::c_int as uint8,
        0xff as ::core::ffi::c_int as uint8,
        0xff as ::core::ffi::c_int as uint8,
        0x69 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0xa2904b as uint32,
        &raw mut t_13 as *mut uint8,
        ::core::mem::size_of::<[uint8; 6]>() as size_t,
    );
    let mut t_14: [uint8; 6] = [
        0x49 as ::core::ffi::c_int as uint8,
        0xff as ::core::ffi::c_int as uint8,
        0xff as ::core::ffi::c_int as uint8,
        0x69 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0xa29065 as uint32,
        &raw mut t_14 as *mut uint8,
        ::core::mem::size_of::<[uint8; 6]>() as size_t,
    );
    let mut t_15: [uint8; 1] = [0x6b as ::core::ffi::c_int as uint8];
    PatchBytes(
        0xa0896f as uint32,
        &raw mut t_15 as *mut uint8,
        ::core::mem::size_of::<[uint8; 1]>() as size_t,
    );
    let mut t_16: [uint8; 3] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0xa98c12 as uint32,
        &raw mut t_16 as *mut uint8,
        ::core::mem::size_of::<[uint8; 3]>() as size_t,
    );
    let mut t_17: [uint8; 1] = [0x60 as ::core::ffi::c_int as uint8];
    PatchBytes(
        0x8085f6 as uint32,
        &raw mut t_17 as *mut uint8,
        ::core::mem::size_of::<[uint8; 1]>() as size_t,
    );
    let mut t_18: [uint8; 10] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x80843c as uint32,
        &raw mut t_18 as *mut uint8,
        ::core::mem::size_of::<[uint8; 10]>() as size_t,
    );
    let mut t_19: [uint8; 10] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x808475 as uint32,
        &raw mut t_19 as *mut uint8,
        ::core::mem::size_of::<[uint8; 10]>() as size_t,
    );
    let mut t_20: [uint8; 10] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x808525 as uint32,
        &raw mut t_20 as *mut uint8,
        ::core::mem::size_of::<[uint8; 10]>() as size_t,
    );
    let mut t_21: [uint8; 4] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x8882a1 as uint32,
        &raw mut t_21 as *mut uint8,
        ::core::mem::size_of::<[uint8; 4]>() as size_t,
    );
    let mut t_22: [uint8; 4] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x828533 as uint32,
        &raw mut t_22 as *mut uint8,
        ::core::mem::size_of::<[uint8; 4]>() as size_t,
    );
    let mut t_23: [uint8; 4] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x80837b as uint32,
        &raw mut t_23 as *mut uint8,
        ::core::mem::size_of::<[uint8; 4]>() as size_t,
    );
    let mut t_24: [uint8; 4] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x80838e as uint32,
        &raw mut t_24 as *mut uint8,
        ::core::mem::size_of::<[uint8; 4]>() as size_t,
    );
    let mut t_25: [uint8; 4] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x82df6c as uint32,
        &raw mut t_25 as *mut uint8,
        ::core::mem::size_of::<[uint8; 4]>() as size_t,
    );
    let mut t_26: [uint8; 4] = [
        0x20 as ::core::ffi::c_int as uint8,
        0x32 as ::core::ffi::c_int as uint8,
        0x96 as ::core::ffi::c_int as uint8,
        0x6b as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x80d000 as uint32,
        &raw mut t_26 as *mut uint8,
        ::core::mem::size_of::<[uint8; 4]>() as size_t,
    );
    let mut t_27: [uint8; 5] = [
        0x22 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0xd0 as ::core::ffi::c_int as uint8,
        0x80 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x82e02c as uint32,
        &raw mut t_27 as *mut uint8,
        ::core::mem::size_of::<[uint8; 5]>() as size_t,
    );
    let mut t_28: [uint8; 5] = [
        0x22 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0xd0 as ::core::ffi::c_int as uint8,
        0x80 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x82e06b as uint32,
        &raw mut t_28 as *mut uint8,
        ::core::mem::size_of::<[uint8; 5]>() as size_t,
    );
    let mut t_29: [uint8; 5] = [
        0x22 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0xd0 as ::core::ffi::c_int as uint8,
        0x80 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x82e50d as uint32,
        &raw mut t_29 as *mut uint8,
        ::core::mem::size_of::<[uint8; 5]>() as size_t,
    );
    let mut t_30: [uint8; 5] = [
        0x22 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0xd0 as ::core::ffi::c_int as uint8,
        0x80 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x82e609 as uint32,
        &raw mut t_30 as *mut uint8,
        ::core::mem::size_of::<[uint8; 5]>() as size_t,
    );
    let mut t_31: [uint8; 5] = [
        0x22 as ::core::ffi::c_int as uint8,
        0x4 as ::core::ffi::c_int as uint8,
        0xd0 as ::core::ffi::c_int as uint8,
        0x80 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x82e526 as uint32,
        &raw mut t_31 as *mut uint8,
        ::core::mem::size_of::<[uint8; 5]>() as size_t,
    );
    let mut t_32: [uint8; 10] = [
        0x22 as ::core::ffi::c_int as uint8,
        0x38 as ::core::ffi::c_int as uint8,
        0x83 as ::core::ffi::c_int as uint8,
        0x80 as ::core::ffi::c_int as uint8,
        0xad as ::core::ffi::c_int as uint8,
        0x31 as ::core::ffi::c_int as uint8,
        0x9 as ::core::ffi::c_int as uint8,
        0x10 as ::core::ffi::c_int as uint8,
        0xf7 as ::core::ffi::c_int as uint8,
        0x6b as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x80d004 as uint32,
        &raw mut t_32 as *mut uint8,
        ::core::mem::size_of::<[uint8; 10]>() as size_t,
    );
    let mut t_33: [uint8; 4] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x82e540 as uint32,
        &raw mut t_33 as *mut uint8,
        ::core::mem::size_of::<[uint8; 4]>() as size_t,
    );
    let mut t_34: [uint8; 4] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x8bc11e as uint32,
        &raw mut t_34 as *mut uint8,
        ::core::mem::size_of::<[uint8; 4]>() as size_t,
    );
    let mut t_35: [uint8; 4] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x8bd487 as uint32,
        &raw mut t_35 as *mut uint8,
        ::core::mem::size_of::<[uint8; 4]>() as size_t,
    );
    let mut t_36: [uint8; 3] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x858148 as uint32,
        &raw mut t_36 as *mut uint8,
        ::core::mem::size_of::<[uint8; 3]>() as size_t,
    );
    let mut t_37: [uint8; 3] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x8581b2 as uint32,
        &raw mut t_37 as *mut uint8,
        ::core::mem::size_of::<[uint8; 3]>() as size_t,
    );
    let mut t_38: [uint8; 8] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x8581ea as uint32,
        &raw mut t_38 as *mut uint8,
        ::core::mem::size_of::<[uint8; 8]>() as size_t,
    );
    let mut t_39: [uint8; 3] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x858203 as uint32,
        &raw mut t_39 as *mut uint8,
        ::core::mem::size_of::<[uint8; 3]>() as size_t,
    );
    let mut t_40: [uint8; 8] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x858236 as uint32,
        &raw mut t_40 as *mut uint8,
        ::core::mem::size_of::<[uint8; 8]>() as size_t,
    );
    let mut t_41: [uint8; 11] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x8582b8 as uint32,
        &raw mut t_41 as *mut uint8,
        ::core::mem::size_of::<[uint8; 11]>() as size_t,
    );
    let mut t_42: [uint8; 3] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x858321 as uint32,
        &raw mut t_42 as *mut uint8,
        ::core::mem::size_of::<[uint8; 3]>() as size_t,
    );
    let mut t_43: [uint8; 8] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x85835a as uint32,
        &raw mut t_43 as *mut uint8,
        ::core::mem::size_of::<[uint8; 8]>() as size_t,
    );
    let mut t_44: [uint8; 3] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x858532 as uint32,
        &raw mut t_44 as *mut uint8,
        ::core::mem::size_of::<[uint8; 3]>() as size_t,
    );
    let mut t_45: [uint8; 8] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x85856b as uint32,
        &raw mut t_45 as *mut uint8,
        ::core::mem::size_of::<[uint8; 8]>() as size_t,
    );
    let mut t_46: [uint8; 3] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x858096 as uint32,
        &raw mut t_46 as *mut uint8,
        ::core::mem::size_of::<[uint8; 3]>() as size_t,
    );
    let mut t_47: [uint8; 3] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x8580b4 as uint32,
        &raw mut t_47 as *mut uint8,
        ::core::mem::size_of::<[uint8; 3]>() as size_t,
    );
    let mut t_48: [uint8; 3] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x8580dc as uint32,
        &raw mut t_48 as *mut uint8,
        ::core::mem::size_of::<[uint8; 3]>() as size_t,
    );
    let mut t_49: [uint8; 3] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x8580f2 as uint32,
        &raw mut t_49 as *mut uint8,
        ::core::mem::size_of::<[uint8; 3]>() as size_t,
    );
    let mut t_50: [uint8; 3] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x85861c as uint32,
        &raw mut t_50 as *mut uint8,
        ::core::mem::size_of::<[uint8; 3]>() as size_t,
    );
    let mut t_51: [uint8; 3] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x858651 as uint32,
        &raw mut t_51 as *mut uint8,
        ::core::mem::size_of::<[uint8; 3]>() as size_t,
    );
    let mut t_52: [uint8; 4] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x858692 as uint32,
        &raw mut t_52 as *mut uint8,
        ::core::mem::size_of::<[uint8; 4]>() as size_t,
    );
    let mut t_53: [uint8; 4] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x858696 as uint32,
        &raw mut t_53 as *mut uint8,
        ::core::mem::size_of::<[uint8; 4]>() as size_t,
    );
    let mut t_54: [uint8; 6] = [
        0x8 as ::core::ffi::c_int as uint8,
        0xc2 as ::core::ffi::c_int as uint8,
        0x30 as ::core::ffi::c_int as uint8,
        0x4c as ::core::ffi::c_int as uint8,
        0xa9 as ::core::ffi::c_int as uint8,
        0x85 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x859660 as uint32,
        &raw mut t_54 as *mut uint8,
        ::core::mem::size_of::<[uint8; 6]>() as size_t,
    );
    let mut t_55: [uint8; 3] = [
        0x20 as ::core::ffi::c_int as uint8,
        0x60 as ::core::ffi::c_int as uint8,
        0x96 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x8583ba as uint32,
        &raw mut t_55 as *mut uint8,
        ::core::mem::size_of::<[uint8; 3]>() as size_t,
    );
    let mut t_56: [uint8; 17] = [
        0x22 as ::core::ffi::c_int as uint8,
        0x81 as ::core::ffi::c_int as uint8,
        0x96 as ::core::ffi::c_int as uint8,
        0x85 as ::core::ffi::c_int as uint8,
        0xc9 as ::core::ffi::c_int as uint8,
        0xff as ::core::ffi::c_int as uint8,
        0xff as ::core::ffi::c_int as uint8,
        0xd0 as ::core::ffi::c_int as uint8,
        0x4 as ::core::ffi::c_int as uint8,
        0x5c as ::core::ffi::c_int as uint8,
        0x5f as ::core::ffi::c_int as uint8,
        0xab as ::core::ffi::c_int as uint8,
        0xa2 as ::core::ffi::c_int as uint8,
        0x5c as ::core::ffi::c_int as uint8,
        0x26 as ::core::ffi::c_int as uint8,
        0xab as ::core::ffi::c_int as uint8,
        0xa2 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x859670 as uint32,
        &raw mut t_56 as *mut uint8,
        ::core::mem::size_of::<[uint8; 17]>() as size_t,
    );
    let mut t_57: [uint8; 20] = [
        0xcd as ::core::ffi::c_int as uint8,
        0x1f as ::core::ffi::c_int as uint8,
        0x1c as ::core::ffi::c_int as uint8,
        0xd0 as ::core::ffi::c_int as uint8,
        0x8 as ::core::ffi::c_int as uint8,
        0x9c as ::core::ffi::c_int as uint8,
        0x1f as ::core::ffi::c_int as uint8,
        0x1c as ::core::ffi::c_int as uint8,
        0xad as ::core::ffi::c_int as uint8,
        0xf9 as ::core::ffi::c_int as uint8,
        0x5 as ::core::ffi::c_int as uint8,
        0x6b as ::core::ffi::c_int as uint8,
        0xff as ::core::ffi::c_int as uint8,
        0x8d as ::core::ffi::c_int as uint8,
        0xc8 as ::core::ffi::c_int as uint8,
        0xd as ::core::ffi::c_int as uint8,
        0xa9 as ::core::ffi::c_int as uint8,
        0xff as ::core::ffi::c_int as uint8,
        0xff as ::core::ffi::c_int as uint8,
        0x6b as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x859681 as uint32,
        &raw mut t_57 as *mut uint8,
        ::core::mem::size_of::<[uint8; 20]>() as size_t,
    );
    let mut t_58: [uint8; 4] = [
        0x5c as ::core::ffi::c_int as uint8,
        0x70 as ::core::ffi::c_int as uint8,
        0x96 as ::core::ffi::c_int as uint8,
        0x85 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0xa2ab22 as uint32,
        &raw mut t_58 as *mut uint8,
        ::core::mem::size_of::<[uint8; 4]>() as size_t,
    );
    let mut t_59: [uint8; 17] = [
        0x22 as ::core::ffi::c_int as uint8,
        0xd4 as ::core::ffi::c_int as uint8,
        0x8f as ::core::ffi::c_int as uint8,
        0xa0 as ::core::ffi::c_int as uint8,
        0xad as ::core::ffi::c_int as uint8,
        0xc8 as ::core::ffi::c_int as uint8,
        0xd as ::core::ffi::c_int as uint8,
        0xf0 as ::core::ffi::c_int as uint8,
        0x7 as ::core::ffi::c_int as uint8,
        0x22 as ::core::ffi::c_int as uint8,
        0x95 as ::core::ffi::c_int as uint8,
        0x96 as ::core::ffi::c_int as uint8,
        0x85 as ::core::ffi::c_int as uint8,
        0x9c as ::core::ffi::c_int as uint8,
        0xc8 as ::core::ffi::c_int as uint8,
        0xd as ::core::ffi::c_int as uint8,
        0x6b as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x8596a0 as uint32,
        &raw mut t_59 as *mut uint8,
        ::core::mem::size_of::<[uint8; 17]>() as size_t,
    );
    let mut t_60: [uint8; 4] = [
        0x22 as ::core::ffi::c_int as uint8,
        0xa0 as ::core::ffi::c_int as uint8,
        0x96 as ::core::ffi::c_int as uint8,
        0x85 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x828b65 as uint32,
        &raw mut t_60 as *mut uint8,
        ::core::mem::size_of::<[uint8; 4]>() as size_t,
    );
    let mut t_61: [uint8; 10] = [
        0x20 as ::core::ffi::c_int as uint8,
        0x89 as ::core::ffi::c_int as uint8,
        0x85 as ::core::ffi::c_int as uint8,
        0xa9 as ::core::ffi::c_int as uint8,
        0x1c as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0x8d as ::core::ffi::c_int as uint8,
        0x1f as ::core::ffi::c_int as uint8,
        0x1c as ::core::ffi::c_int as uint8,
        0x60 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x8596c0 as uint32,
        &raw mut t_61 as *mut uint8,
        ::core::mem::size_of::<[uint8; 10]>() as size_t,
    );
    let mut t_62: [uint8; 3] = [
        0x20 as ::core::ffi::c_int as uint8,
        0xc0 as ::core::ffi::c_int as uint8,
        0x96 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x8580e5 as uint32,
        &raw mut t_62 as *mut uint8,
        ::core::mem::size_of::<[uint8; 3]>() as size_t,
    );
    let mut t_63: [uint8; 25] = [
        0xad as ::core::ffi::c_int as uint8,
        0xc8 as ::core::ffi::c_int as uint8,
        0xd as ::core::ffi::c_int as uint8,
        0xf0 as ::core::ffi::c_int as uint8,
        0x11 as ::core::ffi::c_int as uint8,
        0x98 as ::core::ffi::c_int as uint8,
        0x9d as ::core::ffi::c_int as uint8,
        0x27 as ::core::ffi::c_int as uint8,
        0x1d as ::core::ffi::c_int as uint8,
        0xad as ::core::ffi::c_int as uint8,
        0xc8 as ::core::ffi::c_int as uint8,
        0xd as ::core::ffi::c_int as uint8,
        0x22 as ::core::ffi::c_int as uint8,
        0x95 as ::core::ffi::c_int as uint8,
        0x96 as ::core::ffi::c_int as uint8,
        0x85 as ::core::ffi::c_int as uint8,
        0x9c as ::core::ffi::c_int as uint8,
        0xc8 as ::core::ffi::c_int as uint8,
        0xd as ::core::ffi::c_int as uint8,
        0xbc as ::core::ffi::c_int as uint8,
        0x27 as ::core::ffi::c_int as uint8,
        0x1d as ::core::ffi::c_int as uint8,
        0x4c as ::core::ffi::c_int as uint8,
        0xee as ::core::ffi::c_int as uint8,
        0x85 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x84efdc as uint32,
        &raw mut t_63 as *mut uint8,
        ::core::mem::size_of::<[uint8; 25]>() as size_t,
    );
    let mut t_64: [uint8; 3] = [
        0xf4 as ::core::ffi::c_int as uint8,
        0xdb as ::core::ffi::c_int as uint8,
        0xef as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x8485f7 as uint32,
        &raw mut t_64 as *mut uint8,
        ::core::mem::size_of::<[uint8; 3]>() as size_t,
    );
    let mut t_65: [uint8; 8] = [
        0x8 as ::core::ffi::c_int as uint8,
        0x8b as ::core::ffi::c_int as uint8,
        0xda as ::core::ffi::c_int as uint8,
        0x5a as ::core::ffi::c_int as uint8,
        0x5c as ::core::ffi::c_int as uint8,
        0x84 as ::core::ffi::c_int as uint8,
        0x80 as ::core::ffi::c_int as uint8,
        0x85 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x859695 as uint32,
        &raw mut t_65 as *mut uint8,
        ::core::mem::size_of::<[uint8; 8]>() as size_t,
    );
    let mut t_66: [uint8; 4] = [
        0x8d as ::core::ffi::c_int as uint8,
        0xc8 as ::core::ffi::c_int as uint8,
        0xd as ::core::ffi::c_int as uint8,
        0x6b as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x858080 as uint32,
        &raw mut t_66 as *mut uint8,
        ::core::mem::size_of::<[uint8; 4]>() as size_t,
    );
    let mut t_67: [uint8; 18] = [
        0x22 as ::core::ffi::c_int as uint8,
        0x81 as ::core::ffi::c_int as uint8,
        0x96 as ::core::ffi::c_int as uint8,
        0x85 as ::core::ffi::c_int as uint8,
        0xc9 as ::core::ffi::c_int as uint8,
        0xff as ::core::ffi::c_int as uint8,
        0xff as ::core::ffi::c_int as uint8,
        0xf0 as ::core::ffi::c_int as uint8,
        0x4 as ::core::ffi::c_int as uint8,
        0x5c as ::core::ffi::c_int as uint8,
        0xfa as ::core::ffi::c_int as uint8,
        0x8c as ::core::ffi::c_int as uint8,
        0x84 as ::core::ffi::c_int as uint8,
        0x7a as ::core::ffi::c_int as uint8,
        0xfa as ::core::ffi::c_int as uint8,
        0x88 as ::core::ffi::c_int as uint8,
        0x88 as ::core::ffi::c_int as uint8,
        0x60 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x84f000 as uint32,
        &raw mut t_67 as *mut uint8,
        ::core::mem::size_of::<[uint8; 18]>() as size_t,
    );
    let mut t_68: [uint8; 4] = [
        0x5c as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0xf0 as ::core::ffi::c_int as uint8,
        0x84 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x848cf6 as uint32,
        &raw mut t_68 as *mut uint8,
        ::core::mem::size_of::<[uint8; 4]>() as size_t,
    );
    let mut t_69: [uint8; 7] = [
        0xa9 as ::core::ffi::c_int as uint8,
        0xff as ::core::ffi::c_int as uint8,
        0xff as ::core::ffi::c_int as uint8,
        0x8d as ::core::ffi::c_int as uint8,
        0x98 as ::core::ffi::c_int as uint8,
        0x9 as ::core::ffi::c_int as uint8,
        0x60 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x81f000 as uint32,
        &raw mut t_69 as *mut uint8,
        ::core::mem::size_of::<[uint8; 7]>() as size_t,
    );
    let mut t_70: [uint8; 4] = [
        0x5c as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0xf0 as ::core::ffi::c_int as uint8,
        0x81 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x819027 as uint32,
        &raw mut t_70 as *mut uint8,
        ::core::mem::size_of::<[uint8; 4]>() as size_t,
    );
    let mut t_71: [uint8; 4] = [
        0x5c as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0xf0 as ::core::ffi::c_int as uint8,
        0x81 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x819112 as uint32,
        &raw mut t_71 as *mut uint8,
        ::core::mem::size_of::<[uint8; 4]>() as size_t,
    );
    let mut t_72: [uint8; 4] = [
        0x5c as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0xf0 as ::core::ffi::c_int as uint8,
        0x81 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x8194e9 as uint32,
        &raw mut t_72 as *mut uint8,
        ::core::mem::size_of::<[uint8; 4]>() as size_t,
    );
    let mut t_73: [uint8; 4] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x8095e1 as uint32,
        &raw mut t_73 as *mut uint8,
        ::core::mem::size_of::<[uint8; 4]>() as size_t,
    );
    if g_use_my_apu_code {
        let mut t_74: [uint8; 1] = [0x60 as ::core::ffi::c_int as uint8];
        PatchBytes(
            0x808028 as uint32,
            &raw mut t_74 as *mut uint8,
            ::core::mem::size_of::<[uint8; 1]>() as size_t,
        );
    }
    let mut t_75: [uint8; 4] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x80 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x828a59 as uint32,
        &raw mut t_75 as *mut uint8,
        ::core::mem::size_of::<[uint8; 4]>() as size_t,
    );
    let mut t_76: [uint8; 3] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x828a72 as uint32,
        &raw mut t_76 as *mut uint8,
        ::core::mem::size_of::<[uint8; 3]>() as size_t,
    );
    let mut t_77: [uint8; 4] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x80 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x828a80 as uint32,
        &raw mut t_77 as *mut uint8,
        ::core::mem::size_of::<[uint8; 4]>() as size_t,
    );
    let mut t_78: [uint8; 1] = [0x6 as ::core::ffi::c_int as uint8];
    PatchBytes(
        0x828a67 as uint32,
        &raw mut t_78 as *mut uint8,
        ::core::mem::size_of::<[uint8; 1]>() as size_t,
    );
    let mut t_79: [uint8; 2] = [
        0 as ::core::ffi::c_int as uint8,
        0x2e as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x82831e as uint32,
        &raw mut t_79 as *mut uint8,
        ::core::mem::size_of::<[uint8; 2]>() as size_t,
    );
    let mut t_80: [uint8; 2] = [
        0xa5 as ::core::ffi::c_int as uint8,
        0x25 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x91c234 as uint32,
        &raw mut t_80 as *mut uint8,
        ::core::mem::size_of::<[uint8; 2]>() as size_t,
    );
    let mut t_81: [uint8; 4] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x809af3 as uint32,
        &raw mut t_81 as *mut uint8,
        ::core::mem::size_of::<[uint8; 4]>() as size_t,
    );
    let mut t_82: [uint8; 23] = [
        0xa5 as ::core::ffi::c_int as uint8,
        0x12 as ::core::ffi::c_int as uint8,
        0x48 as ::core::ffi::c_int as uint8,
        0xa5 as ::core::ffi::c_int as uint8,
        0x14 as ::core::ffi::c_int as uint8,
        0x48 as ::core::ffi::c_int as uint8,
        0xa5 as ::core::ffi::c_int as uint8,
        0x16 as ::core::ffi::c_int as uint8,
        0x48 as ::core::ffi::c_int as uint8,
        0x22 as ::core::ffi::c_int as uint8,
        0xa7 as ::core::ffi::c_int as uint8,
        0xa6 as ::core::ffi::c_int as uint8,
        0xa0 as ::core::ffi::c_int as uint8,
        0x68 as ::core::ffi::c_int as uint8,
        0x85 as ::core::ffi::c_int as uint8,
        0x16 as ::core::ffi::c_int as uint8,
        0x68 as ::core::ffi::c_int as uint8,
        0x85 as ::core::ffi::c_int as uint8,
        0x14 as ::core::ffi::c_int as uint8,
        0x68 as ::core::ffi::c_int as uint8,
        0x85 as ::core::ffi::c_int as uint8,
        0x12 as ::core::ffi::c_int as uint8,
        0x6b as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0xa7ff82 as uint32,
        &raw mut t_82 as *mut uint8,
        ::core::mem::size_of::<[uint8; 23]>() as size_t,
    );
    let mut t_83: [uint8; 4] = [
        0x22 as ::core::ffi::c_int as uint8,
        0x82 as ::core::ffi::c_int as uint8,
        0xff as ::core::ffi::c_int as uint8,
        0xa7 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0xa7b03a as uint32,
        &raw mut t_83 as *mut uint8,
        ::core::mem::size_of::<[uint8; 4]>() as size_t,
    );
    RtlUpdateSnesPatchForBugfix();
    let mut i: size_t = 0 as size_t;
    while i
        != (::core::mem::size_of::<[uint32; 155]>() as usize)
            .wrapping_div(::core::mem::size_of::<uint32>() as usize)
    {
        let mut t_84: uint8 = *SnesRomPtr(kPatchedCarrys[i as usize]);
        if t_84 != 0 {
            kPatchedCarrysOrg[i as usize] = t_84;
            FixupCarry(kPatchedCarrys[i as usize]);
        } else {
            printf(
                b"0x%x double patched!\n\0" as *const u8 as *const ::core::ffi::c_char,
                kPatchedCarrys[i as usize],
            );
        }
        i = i.wrapping_add(1);
    }
    PatchBugs(1 as uint32, 0 as uint32);
    return g_snes;
}
#[no_mangle]
pub unsafe extern "C" fn DebugGameOverMenu() {
    '_c2rust_label: {
        __assert_fail(
            b"0\0" as *const u8 as *const ::core::ffi::c_char,
            b"src/sm_cpu_infra.c\0" as *const u8 as *const ::core::ffi::c_char,
            914 as ::core::ffi::c_uint,
            b"void DebugGameOverMenu(void)\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn RunCpuUntilPC(mut pc1: uint32, mut pc2: uint32) -> uint32 {
    loop {
        if (*g_snes).debug_cycles {
            let mut line: [::core::ffi::c_char; 80] = [0; 80];
            getProcessorStateCpu(g_snes, &raw mut line as *mut ::core::ffi::c_char);
            puts(&raw mut line as *mut ::core::ffi::c_char);
        }
        cpu_runOpcode(g_cpu);
        let mut addr: uint32 = (((*(*g_snes).cpu).k as ::core::ffi::c_int)
            << 16 as ::core::ffi::c_int
            | (*(*g_snes).cpu).pc as ::core::ffi::c_int) as uint32;
        if addr == pc1 || addr == pc2 {
            return addr;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn RunOneFrameOfGame_Emulated() {
    let mut bug_fix_bak: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0xc2 as ::core::ffi::c_int as isize)
        as *mut uint16);
    RunCpuUntilPC(0x808343 as uint32, 0x85813c as uint32);
    (*(*g_snes).cpu).nmiWanted = true_0 != 0;
    RunCpuUntilPC(0x80834a as uint32, 0x858142 as uint32);
    *((&raw mut g_ram as *mut uint8).offset(0xc2 as ::core::ffi::c_int as isize) as *mut uint16) =
        bug_fix_bak;
}
#[no_mangle]
pub unsafe extern "C" fn DrawFrameToPpu() {
    (*g_snes).vPos = 0 as uint16_t;
    (*g_snes).hPos = (*g_snes).vPos;
    while !(*(*g_snes).cpu).nmiWanted {
        loop {
            snes_handle_pos_stuff(g_snes as *mut Snes);
            if !((*g_snes).hPos as ::core::ffi::c_int != 0 as ::core::ffi::c_int) {
                break;
            }
        }
        if (*g_snes).vIrqEnabled as ::core::ffi::c_int != 0
            && (*g_snes).vPos as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                == (*g_snes).vTimer as ::core::ffi::c_int
        {
            Vector_IRQ();
        }
    }
    (*(*g_snes).cpu).nmiWanted = false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn SaveBugSnapshot() {
    if VERIFY_ACTIVE {
        return;
    }
    if !g_debug_flag && g_got_mismatch_count == 0 as ::core::ffi::c_int {
        let mut buffer: [::core::ffi::c_char; 64] = [0; 64];
        snprintf(
            &raw mut buffer as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
            b"saves/bug-%d.sav\0" as *const u8 as *const ::core::ffi::c_char,
            time(::core::ptr::null_mut::<time_t>()) as ::core::ffi::c_int,
        );
        RtlSaveSnapshot(&raw mut buffer as *mut ::core::ffi::c_char, true_0 != 0);
    }
    g_got_mismatch_count = 5 as ::core::ffi::c_int * 60 as ::core::ffi::c_int;
}
// Set only by the single-threaded headless runner. The interactive oracle
// retains its existing recovery and snapshot behaviour.
pub static mut VERIFY_ACTIVE: bool = false;
pub static mut VERIFY_FAILURES: u64 = 0;

unsafe fn report_first_verify_failure() {
    eprintln!("First failing frame: {} (zero-based)", snes_frame_counter);
    let mine = &*(&raw const g_snapshot_mine);
    let theirs = &*(&raw const g_snapshot_theirs);
    let mut differing_bytes = 0usize;
    for (addr, (&actual, &expected)) in mine.ram.iter().zip(&theirs.ram).enumerate() {
        if actual == expected {
            continue;
        }
        differing_bytes += 1;
        if differing_bytes > 64 {
            continue;
        }
        let var = crate::ram_map::RAM_VARS
            .iter()
            .rev()
            .find(|var| var.addr <= addr as u32);
        if let Some(var) = var {
            eprintln!(
                "RAM ${:06X} ({}+${:X}): ${actual:02X} != ${expected:02X}",
                0x7e0000 + addr,
                var.name,
                addr as u32 - var.addr
            );
        } else {
            eprintln!(
                "RAM ${:06X}: ${actual:02X} != ${expected:02X}",
                0x7e0000 + addr
            );
        }
    }
    eprintln!("RAM: {differing_bytes} differing bytes (showing at most 64)");
}
#[no_mangle]
pub unsafe extern "C" fn RunOneFrameOfGame_Both() {
    (*g_snes).ppu = (*g_snes).snes_ppu;
    MakeSnapshot(&raw mut g_snapshot_before);
    loop {
        (*g_snes).runningWhichVersion = 1 as uint8_t;
        RunOneFrameOfGame_Emulated();
        DrawFrameToPpu();
        MakeSnapshot(&raw mut g_snapshot_theirs);
        (*g_snes).ppu = (*g_snes).my_ppu;
        RestoreSnapshot(&raw mut g_snapshot_before);
        (*g_snes).runningWhichVersion = 2 as uint8_t;
        RunOneFrameOfGame();
        DrawFrameToPpu();
        MakeSnapshot(&raw mut g_snapshot_mine);
        (*g_snes).runningWhichVersion = 0xff as uint8_t;
        VerifySnapshotsEq(
            &raw mut g_snapshot_mine,
            &raw mut g_snapshot_theirs,
            &raw mut g_snapshot_before,
        );
        if g_fail {
            if VERIFY_ACTIVE {
                VERIFY_FAILURES += 1;
                if VERIFY_FAILURES == 1 {
                    report_first_verify_failure();
                }
            }
            g_fail = false_0 != 0;
            if !VERIFY_ACTIVE || VERIFY_FAILURES == 1 {
                printf(b"Verify failure!\n\0" as *const u8 as *const ::core::ffi::c_char);
            }
            (*g_snes).ppu = (*g_snes).snes_ppu;
            RestoreSnapshot(&raw mut g_snapshot_before);
            if g_debug_flag {
                continue;
            }
            SaveBugSnapshot();
            RunOneFrameOfGame_Emulated();
            break;
        } else {
            (*g_snes).ppu = (*g_snes).snes_ppu;
            RestoreSnapshot(&raw mut g_snapshot_theirs);
            break;
        }
    }
    (*g_snes).ppu = if g_other_image as ::core::ffi::c_int != 0 {
        (*g_snes).my_ppu
    } else {
        (*g_snes).snes_ppu
    };
    (*g_snes).runningWhichVersion = 0 as uint8_t;
    if *((&raw mut g_ram as *mut uint8).offset(0x998 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == 0xffff as ::core::ffi::c_int
    {
        (*(*g_snes).cpu).k = 0x80 as uint8_t;
        (*(*g_snes).cpu).pc = 0x8462 as uint16_t;
        *(&raw mut g_ram as *mut uint8).offset(0x611 as ::core::ffi::c_int as isize) = 3 as uint8;
    }
    if *((&raw mut g_ram as *mut uint8).offset(0x727 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int
        != 0
    {
        printf(b"MENU INDEX TOO BIG!\n\0" as *const u8 as *const ::core::ffi::c_char);
        SaveBugSnapshot();
        let ref mut fresh1 = *((&raw mut g_ram as *mut uint8)
            .offset(0x727 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh1 = (*fresh1 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as uint16;
    }
    if g_got_mismatch_count != 0 {
        g_got_mismatch_count -= 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn RtlRunFrameCompare(mut input: uint16, mut run_what: ::core::ffi::c_int) {
    (*(*g_snes).input1).currentState = input as uint16_t;
    if g_runmode as ::core::ffi::c_int == RM_THEIRS as ::core::ffi::c_int {
        RunOneFrameOfGame_Emulated();
        DrawFrameToPpu();
    } else if g_runmode as ::core::ffi::c_int == RM_MINE as ::core::ffi::c_int {
        g_use_my_apu_code = true_0 != 0;
        (*g_snes).runningWhichVersion = 0xff as uint8_t;
        RunOneFrameOfGame();
        DrawFrameToPpu();
        (*g_snes).runningWhichVersion = 0 as uint8_t;
    } else {
        g_use_my_apu_code = true_0 != 0;
        RunOneFrameOfGame_Both();
    };
}
