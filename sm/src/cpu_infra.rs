use crate::game::sm_80::Vector_IRQ;
use crate::game::sm_82::RunOneFrameOfGame;
use crate::game_state::Game;
pub use crate::snes::registers::*;
use crate::types::SpcPlayer;

use crate::rtl::{RomPtr, RtlSaveSnapshot, RtlSetupEmuCallbacks, WriteReg};
use crate::snes::apu::apu_cycle;
use crate::snes::cpu::{cpu_getFlags, cpu_runOpcode, cpu_setFlags};
use crate::snes::dma::dma_doDma;
use crate::snes::snes::{snes_handle_pos_stuff, snes_init};
use crate::snes::snes_other::snes_loadRom;
use crate::tracing::getProcessorStateCpu;
pub use crate::types::{_IO_codecvt, _IO_marker, _IO_wide_data};
use crate::util::ReadWholeFile;
extern "C" {

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

    fn time(__timer: *mut time_t) -> time_t;
}
pub use crate::types::{
    _IO_lock_t, __int16_t, __int32_t, __int8_t, __off64_t, __off_t, __time_t, __uint16_t,
    __uint32_t, __uint8_t, int16_t, int32_t, int8, int8_t, size_t, time_t, uint16, uint16_t,
    uint32, uint32_t, uint8, uint8_t, Apu, ApuHistory as C2RustUnnamed, BgLayer, Cart, Cpu, Dma,
    DmaChannel, Dsp, DspChannel, DspRegWriteHistory, EnemyBase, Enemy_Botwoon, Input, Layer, Ppu,
    PpuPixelPrioBufs, PpuZbufType, Snes, Spc, Timer, VoidP, WindowLayer, _IO_FILE, FILE,
};
pub use crate::types::{ProjectileInstr, SyncAllFunc};
// Explicit state previously included by the oversized OAM memory copy.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SpriteSnapshot {
    pub address: uint8,
    pub written_address: uint8,
    pub high_table: bool,
    pub written_high_table: bool,
    pub second_byte: bool,
    pub buffer: uint8,
    pub priority_rotation: bool,
    pub tile_address_1: uint16,
    pub tile_address_2: uint16,
    pub size: uint8,
    // Preserve the inherited cache coverage without depending on Ppu layout.
    pub legacy_pixel_prefix: [uint8; 19],
}
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CounterSnapshot {
    pub horizontal: uint16,
    pub vertical: uint16,
    pub horizontal_second_byte: bool,
    pub vertical_second_byte: bool,
    pub latched: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Snapshot {
    pub audio: crate::audio_snapshot::AudioSnapshot,
    pub player: crate::audio_snapshot::PlayerSnapshot,
    pub a: uint16,
    pub x: uint16,
    pub y: uint16,
    pub sp: uint16,
    pub dp: uint16,
    pub pc: uint16,
    pub k: uint8,
    pub db: uint8,
    pub flags: uint8,
    pub cpu_waiting: bool,
    pub cpu_emulation: bool,
    pub cpu_stopped: bool,
    pub cpu_irq_pending: bool,
    pub cpu_nmi_pending: bool,
    pub video_clock: (uint16_t, uint16_t, uint32_t),
    pub interrupt_latches: (bool, bool, bool),
    pub interrupt_enables: (bool, bool, bool),
    pub h_timer: uint16_t,
    pub snes_open_bus: uint8_t,
    pub input_control: (bool, bool, uint16_t),
    pub wram_port_address: uint32_t,
    pub dma_channels: [crate::types::DmaChannel; 8],
    pub dma_timing: (uint16_t, uint32_t, bool),
    pub arithmetic: (uint8_t, uint16_t, uint16_t, uint16_t),
    pub vTimer: uint16_t,
    pub ram: [uint8; 131072],
    pub vram: [uint16; 32768],
    pub sram: [uint8; 8192],
    pub oam: [uint16; 256],
    pub high_oam: [uint8; 32],
    pub sprite: SpriteSnapshot,
    pub counters: CounterSnapshot,
    pub status_flags: [bool; 3],
    pub mode7_matrix: [i16; 8],
    pub mode7_flags: [bool; 5],
    pub mosaic: ([bool; 4], uint8, uint8),
    pub bgmode: (uint8, bool, [bool; 4]),
    pub bg_scroll: [(uint16, uint16); 4],
    pub bg_address: [(bool, bool, uint16, uint16); 4],
    pub screen_routing: ([uint8; 2], [uint8; 2], [[bool; 4]; 5]),
    pub window_select: (uint32, [(bool, bool, bool, bool, uint8); 6]),
    pub window_bounds: [uint8; 4],
    pub fixed_color: [uint8; 3],
    pub color_math: (bool, bool, [bool; 6]),
    pub color_selection: (uint8, uint8, bool, bool),
    pub display_geometry: [bool; 6],
    pub brightness: uint8,
    pub forced_blank: bool,
    pub cgram: [uint16; 256],
    pub write_latches: [uint8; 3],
    pub cgram_pointer: uint8,
    pub cgram_second_write: bool,
    pub cgram_buffer: uint8,
    pub ppu1_open_bus: uint8,
    pub ppu2_open_bus: uint8,
    pub vram_read_buffer: uint16,
    pub vram_pointer: uint16,
    pub vram_increment: uint16,
    pub vram_increment_on_high: bool,
    pub vram_remap_mode: uint8,
    // Per-run observations, not emulated or serialized audio state.
    pub apu_upload_count: u32,
    pub apu_startup_upload: bool,
}
pub const RM_MINE: RunMode = 1;
pub const RM_BOTH: RunMode = 0;
pub const RM_THEIRS: RunMode = 2;
pub use crate::types::RunMode;
pub use crate::types::{RunFrameFunc, SnesRegs};
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const true_0: ::core::ffi::c_int = 1;
pub const false_0: ::core::ffi::c_int = 0;
#[inline]
unsafe fn get_ProjectileInstr(mut a: uint16, game: &mut Game) -> *mut ProjectileInstr {
    RomPtr(
        (0x930000 as ::core::ffi::c_int | a as ::core::ffi::c_int) as uint32_t,
        game,
    ) as *mut ProjectileInstr
}
#[inline]
unsafe fn Get_Botwoon(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Botwoon {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Botwoon
}

pub unsafe fn Call(mut addr: uint32, game: &mut Game) {
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
        game,
    );
}
pub unsafe fn SnesRomPtr(mut v: uint32, game: &mut Game) -> *mut uint8_t {
    RomPtr(v as uint32_t, game) as *mut uint8_t
}
pub unsafe fn ProcessHook(mut v: uint32, game: &mut Game) -> bool {
    let mut rombyte: *mut uint8_t = SnesRomPtr(v, game);
    match game.hookmode {
        0 => {
            let fresh2 = game.hookcnt;
            game.hookcnt = game.hookcnt.wrapping_add(1);
            *rombyte = game.hook_orgbyte[fresh2 as usize] as uint8_t;
            return false_0 != 0;
        }
        1 => {
            let fresh3 = game.hookcnt;
            game.hookcnt = game.hookcnt.wrapping_add(1);
            game.hook_orgbyte[fresh3 as usize] = *rombyte as uint8;
            *rombyte = 0 as uint8_t;
            return false_0 != 0;
        }
        2 => {
            if v == game.hookadr {
                game.hookmode = 3 as uint32;
                return true_0 != 0;
            }
            return false_0 != 0;
        }
        _ => {}
    }
    false_0 != 0
}
pub unsafe fn FixBugHook(mut addr: uint32, game: &mut Game) -> bool {
    match game.hookmode {
        1 => {
            let mut rombyte: *mut uint8_t = SnesRomPtr(addr, game);
            let fresh0 = game.hookcnt;
            game.hookcnt = game.hookcnt.wrapping_add(1);
            game.hook_fixbug_orgbyte[fresh0 as usize] = *rombyte as uint8;
            *rombyte = 0 as uint8_t;
            return false_0 != 0;
        }
        2 => {
            if addr == game.hookadr {
                game.hookmode = 3 as uint32;
                return true_0 != 0;
            }
            game.hookcnt = game.hookcnt.wrapping_add(1);
            return false_0 != 0;
        }
        _ => {}
    }
    false_0 != 0
}
static kPatchedCarrys: [uint32; 155] = [
    0xa7ac33, 0xa7ac36, 0xa7ac39, 0xa7ac42, 0xa7ac45, 0xa6d6d1, 0xa6d6d3, 0xa6d6d5, 0xa6d700,
    0xa6d702, 0xa6d704, 0xa6d665, 0xa6d667, 0xa6d669, 0xa6d694, 0xa6d696, 0xa6d698, 0x818b65,
    0x818b6b, 0x818ba7, 0x818bd9, 0x818bdf, 0x86a671, 0x86a680, 0x86a6a9, 0x86a6ba, 0x88b245,
    0xa6d565, 0xa6d567, 0xa6d599, 0xa6d59b, 0xa6d5db, 0xa6d5dd, 0xa6d60f, 0xa6d611, 0xa6ceff,
    0xa9f4a5, 0xa9f4a7, 0xa9f4d6, 0xa9f4d8, 0xa9f519, 0xa9f51f, 0xa9f521, 0xa9f554, 0xa9f55a,
    0xa9f55c, 0xa9f24d, 0x80aa6a, 0x80a592, 0x80a720, 0x80a7d6, 0x80a99b, 0x818aa7, 0x94b176,
    0x94b156, 0xa99413, 0x80ab5d, 0x84865c, 0x848d90, 0x84ab60, 0x84b567, 0x84b588, 0x84b606,
    0x84b615, 0x84b624, 0x84b9d3, 0x84b9e2, 0x84ba07, 0x84ba1e, 0x84ba35, 0x84d6ae, 0x84d6bf,
    0x84d812, 0x84daae, 0x84dbaa, 0x84dbe1, 0x84dc20, 0x84dc52, 0x84dc89, 0x84dcc8, 0x84deae,
    0x84dedd, 0x84df0a, 0x84df39, 0x86893a, 0x9483a7, 0x948405, 0x949592, 0x94a13f, 0x94a2b2,
    0x94a3d8, 0xa0bc33, 0xa0bdac, 0xa0bf45, 0xa0c725, 0x88b486, 0x88c578, 0xa292e8, 0x86f18e,
    0x888cb6, 0x888faa, 0x88a483, 0x91cc35, 0x91cbff, 0xa09541, 0xa09552, 0xa49ae8, 0xa6c297,
    0xa6c3ad, 0xa9c5ec, 0xa9d500, 0xa9d537, 0xa9dcdb, 0xa0a31b, 0x91d064, 0x91d07a, 0x90c719,
    0xa6a80e, 0xa6a816, 0xa4906e, 0xa49071, 0x90bc75, 0x90bc93, 0xa8a459, 0xa8a45f, 0xa8a465,
    0xa8a46b, 0xa8a477, 0xa8a47d, 0xa8a483, 0xa8a489, 0xa8a543, 0xa8a54f, 0xa8a55b, 0xa8a567,
    0xa8a573, 0xa8a57f, 0xa8a58b, 0x84d7cb, 0x84d7e2, 0x84d7f4, 0x84d803, 0x8888cd, 0x8888f0,
    0x8888e3, 0x80a5f3, 0x80a845, 0x80a925, 0x80a6aa, 0x948d94, 0x948e25, 0x9082a8, 0x9082ae,
    0xa48ca1, 0xa48ca4,
];

pub unsafe fn PatchBugs(mut mode: uint32, mut addr: uint32, game: &mut Game) -> uint32 {
    game.hookmode = mode;
    game.hookadr = addr;
    game.hookcnt = 0 as uint32;
    if FixBugHook(0x86ef35 as uint32, game) {
        (*game.g_cpu).x = (*game.g_cpu).y;
    } else if FixBugHook(0x86ef45 as uint32, game) {
        (*game.g_cpu).z = (*game.g_cpu).a as ::core::ffi::c_int == 0 as ::core::ffi::c_int;
    } else if FixBugHook(0x818ab8 as uint32, game) {
        if (*game.g_cpu).y as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            (*game.g_cpu).pc = 0x8b1f as uint16_t;
        }
    } else if FixBugHook(0xa794ba as uint32, game) {
        (*game.g_cpu).y = (*game.g_cpu).x;
    } else if FixBugHook(0xa7b968 as uint32, game) {
        (*game.g_cpu).x =
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as uint16_t;
    } else if FixBugHook(0xa7b963 as uint32, game) {
        (*game.g_cpu).x =
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as uint16_t;
    } else if FixBugHook(0xa496c8 as uint32, game) {
        (*game.g_cpu).a = 0 as uint16_t;
    } else if FixBugHook(0x9085aa as uint32, game) {
        (*game.g_cpu).a =
            *((&raw mut game.g_ram as *mut uint8).offset(0xb3e) as *mut uint16) as uint16_t;
    } else if FixBugHook(0xa29044 as uint32, game) as ::core::ffi::c_int != 0
        || FixBugHook(0xa2905d as uint32, game) as ::core::ffi::c_int != 0
    {
        (*game.g_cpu).c = (*game.g_cpu).a as ::core::ffi::c_int == 0 as ::core::ffi::c_int;
    } else if FixBugHook(0xa29051 as uint32, game) {
        (*game.g_cpu).a = (*game.g_cpu).a.wrapping_sub(1);
    } else if FixBugHook(0xa5931c as uint32, game) {
        (*game.g_cpu).x =
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as uint16_t;
    } else if FixBugHook(0x80ada4 as uint32, game) {
        (*game.g_cpu).a =
            *((&raw mut game.g_ram as *mut uint8).offset(0x919) as *mut uint16) as uint16_t;
    } else if FixBugHook(0x80add9 as uint32, game) {
        (*game.g_cpu).a =
            *((&raw mut game.g_ram as *mut uint8).offset(0x919) as *mut uint16) as uint16_t;
    } else if FixBugHook(0x80ad4d as uint32, game) {
        (*game.g_cpu).a =
            *((&raw mut game.g_ram as *mut uint8).offset(0x917) as *mut uint16) as uint16_t;
    } else if FixBugHook(0x80ad77 as uint32, game) {
        (*game.g_cpu).a =
            *((&raw mut game.g_ram as *mut uint8).offset(0x917) as *mut uint16) as uint16_t;
    } else if FixBugHook(0x9381db as uint32, game) {
        let mut k: ::core::ffi::c_int = (*game.g_cpu).x as ::core::ffi::c_int;
        let mut ip: ::core::ffi::c_int =
            *((&raw mut game.g_ram as *mut uint8).offset(0xc40) as *mut uint16)
                .offset((k >> 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int;
        let mut delta: ::core::ffi::c_int = if *((&raw mut game.g_ram as *mut uint8).offset(0xc54)
            as *mut uint16)
            .offset((k >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            == 1 as ::core::ffi::c_int
            && (*get_ProjectileInstr(ip as uint16, game)).timer as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
        {
            0 as ::core::ffi::c_int
        } else {
            -(8 as ::core::ffi::c_int)
        };
        (*game.g_cpu).a =
            ((*game.g_cpu).a as ::core::ffi::c_int + (8 as ::core::ffi::c_int + delta)) as uint16_t;
    } else if FixBugHook(0x86b701 as uint32, game) {
        (*game.g_cpu).x = (*game.g_cpu).y;
    } else if FixBugHook(0x8fc1b0 as uint32, game) {
        (*game.g_cpu).x = (*game.g_cpu).a;
    } else if !FixBugHook(0x80804f as uint32, game) {
        if FixBugHook(0x829325 as uint32, game) {
            (*game.g_cpu).db = 0x82 as uint8_t;
        } else if FixBugHook(0x848acd as uint32, game) {
            (*game.g_cpu).a = 0 as uint16_t;
        } else if FixBugHook(0xa7ceb2 as uint32, game) {
            (*game.g_cpu).x =
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as uint16_t;
        } else if FixBugHook(0x91cd44 as uint32, game) {
            if (*game.g_cpu).x as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                (*game.g_cpu).pc = 0xcd52 as uint16_t;
            }
        } else if FixBugHook(0x82afd3 as uint32, game) {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16) as uint8
                as ::core::ffi::c_int
                != 1 as ::core::ffi::c_int
            {
                return 0x82afd9 as uint32;
            }
        } else if FixBugHook(0x82b0cd as uint32, game) {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16) as uint8
                as ::core::ffi::c_int
                != 2 as ::core::ffi::c_int
            {
                return 0x82afd9 as uint32;
            }
        } else if FixBugHook(0x82b15b as uint32, game) {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16) as uint8
                as ::core::ffi::c_int
                != 3 as ::core::ffi::c_int
            {
                return 0x82afd9 as uint32;
            }
        } else if FixBugHook(0xa2d38c as uint32, game) {
            (*game.g_cpu).x =
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as uint16_t;
        } else if FixBugHook(0xa4970f as uint32, game) {
            (*game.g_cpu).a =
                ((*game.g_cpu).a as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as uint16_t;
            (*game.g_cpu).y =
                ((*game.g_cpu).x as ::core::ffi::c_int & 0x7 as ::core::ffi::c_int) as uint16_t;
        } else if FixBugHook(0xa496e0 as uint32, game) {
            if (*game.g_cpu).x as ::core::ffi::c_int > 48 as ::core::ffi::c_int {
                *((&raw mut game.g_ram as *mut uint8).offset(0x690) as *mut uint16) =
                    (*game.g_cpu).x as uint16;
                (*game.g_cpu).mf = 0 as ::core::ffi::c_int != 0;
                return 0xa497ce as uint32;
            }
        } else if FixBugHook(0x91da89 as uint32, game) {
            if *((&raw mut game.g_ram as *mut uint8).offset(0xace) as *mut uint16)
                as ::core::ffi::c_int
                > 6 as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xace) as *mut uint16) = 6 as uint16;
            }
        } else if FixBugHook(0x828d56 as uint32, game) {
            WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0x80 as uint8, game);
        } else if FixBugHook(0x88afcf as uint32, game) {
            if (*game.g_cpu).a as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
                (*game.g_cpu).a = 0 as uint16_t;
            }
        } else if FixBugHook(0x88aff2 as uint32, game) {
            if ((*game.g_cpu).a as ::core::ffi::c_int) < 256 as ::core::ffi::c_int {
                (*game.g_cpu).a = 256 as uint16_t;
            }
        } else if FixBugHook(0x8189bd as uint32, game) {
            if (*game.g_cpu).y as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                return 0x818a35 as uint32;
            }
        } else if FixBugHook(0xa29bc1 as uint32, game) {
            (*game.g_cpu).a = 1 as uint16_t;
        } else if FixBugHook(0x82e910 as uint32, game) {
            *((&raw mut game.g_ram as *mut uint8).offset(22) as *mut uint8 as *mut uint16) =
                0 as uint16;
        } else if FixBugHook(0x90a4c8 as uint32, game) {
            *((&raw mut game.g_ram as *mut uint8).offset(18) as *mut uint8 as *mut uint16) =
                0 as uint16;
        } else if FixBugHook(0xa99f60 as uint32, game) {
            *((&raw mut game.g_ram as *mut uint8).offset(22) as *mut uint8 as *mut uint16) =
                1 as uint16;
        } else if FixBugHook(0x94a85b as uint32, game) {
            memset(
                (&raw mut game.g_ram as *mut uint8).offset(0xd82) as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                8 as size_t,
            );
        } else if FixBugHook(0xa0a35c as uint32, game) {
            *((&raw mut game.g_ram as *mut uint8).offset(18) as *mut uint16) =
                *((&raw mut game.g_ram as *mut uint8).offset(0xcea) as *mut uint16 as *mut uint8)
                    .offset(
                        (::core::mem::size_of::<uint16>() as usize)
                            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                            .wrapping_sub(1 as usize) as isize,
                    ) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(20) as *mut uint16) =
                (*((&raw mut game.g_ram as *mut uint8).offset(18) as *mut uint16)
                    as ::core::ffi::c_int
                    + (*((&raw mut game.g_ram as *mut uint8).offset(18) as *mut uint16)
                        as ::core::ffi::c_int
                        >> 1 as ::core::ffi::c_int)
                    >> 1 as ::core::ffi::c_int) as uint16;
        } else if FixBugHook(0xa7b049 as uint32, game) {
            (*game.g_cpu).x = 0 as uint16_t;
        } else if FixBugHook(0xa5a018 as uint32, game) {
            *((&raw mut game.g_ram as *mut uint8).offset(0xe24) as *mut uint16) = 0 as uint16;
        } else if FixBugHook(0xb39ddb as uint32, game) {
            let mut E: *mut Enemy_Botwoon = Get_Botwoon(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                    as ::core::ffi::c_int,
                game,
            );
            *((&raw mut game.g_ram as *mut uint8).offset(18) as *mut uint16) =
                ((*E).base.x_pos as ::core::ffi::c_int - (*E).botwoon_var_56 as ::core::ffi::c_int)
                    as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(20) as *mut uint16) =
                ((*E).base.y_pos as ::core::ffi::c_int - (*E).botwoon_var_57 as ::core::ffi::c_int)
                    as uint16;
        } else if FixBugHook(0xb39e13 as uint32, game) {
            let mut E_0: *mut Enemy_Botwoon = Get_Botwoon(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                    as ::core::ffi::c_int,
                game,
            );
            (*E_0).botwoon_var_45 = ((*E_0).botwoon_var_45 as ::core::ffi::c_int
                + (*((&raw mut game.g_ram as *mut uint8).offset(22) as *mut uint16)
                    as ::core::ffi::c_int
                    - (*E_0).botwoon_var_45 as ::core::ffi::c_int) as int8
                    as ::core::ffi::c_int
                    * 3 as ::core::ffi::c_int
                    / 4 as ::core::ffi::c_int) as uint8
                as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(22) as *mut uint16) =
                (*E_0).botwoon_var_45;
        }
    }
    0 as uint32
}
pub unsafe fn RunPatchBugHook(mut addr: uint32, game: &mut Game) -> ::core::ffi::c_int {
    let mut new_pc: uint32 = PatchBugs(2 as uint32, addr, game);
    if game.hookmode == 3 as uint32 {
        if new_pc == 0 as uint32 {
            return game.hook_fixbug_orgbyte[game.hookcnt as usize] as ::core::ffi::c_int;
        } else {
            (*game.g_cpu).k = (new_pc >> 16 as ::core::ffi::c_int) as uint8_t;
            (*game.g_cpu).pc = (new_pc & 0xffff as uint32).wrapping_add(1 as uint32) as uint16_t;
            return *SnesRomPtr(new_pc, game) as ::core::ffi::c_int;
        }
    }
    -(1 as ::core::ffi::c_int)
}
pub unsafe fn CpuOpcodeHook(mut addr: uint32, game: &mut Game) -> ::core::ffi::c_int {
    let mut i: size_t = 0 as size_t;
    while i
        != (::core::mem::size_of::<[uint32; 155]>() as usize)
            .wrapping_div(::core::mem::size_of::<uint32>() as usize)
    {
        if addr == kPatchedCarrys[i as usize] {
            return game.kPatchedCarrysOrg[i as usize] as ::core::ffi::c_int;
        }
        i = i.wrapping_add(1);
    }
    let mut i_0: ::core::ffi::c_int = RunPatchBugHook(addr, game);
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
    0 as ::core::ffi::c_int
}
pub unsafe fn HookedFunctionRts(mut is_long: ::core::ffi::c_int, game: &mut Game) -> bool {
    if game.g_calling_asm_from_c {
        game.g_calling_asm_from_c_ret = is_long;
        game.g_calling_asm_from_c = false_0 != 0;
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
    false_0 != 0
}
fn brightness_cache_is_valid(ppu: &Ppu) -> bool {
    let brightness = ppu.lastBrightnessMult;
    // An invalid marker forces regeneration before the next rendered row.
    if brightness > 15 {
        return true;
    }
    let scaled =
        |level: usize| (((level << 3) | (level >> 2)) * usize::from(brightness) / 15) as u8;
    ppu.brightnessMult
        .iter()
        .enumerate()
        .all(|(i, value)| *value == scaled(i.min(31)))
        && ppu
            .brightnessMultHalf
            .iter()
            .enumerate()
            .all(|(i, value)| *value == scaled(i / 2))
}
unsafe fn VerifySnapshotsEq(
    mut b: *mut Snapshot,
    mut a: *mut Snapshot,
    mut prev: *mut Snapshot,
    game: &mut Game,
) {
    // Report every mismatch so replay failure sequences can be compared.
    for (side, ppu) in [
        ("mine", &*(*game.g_snes).my_ppu),
        ("theirs", &*(*game.g_snes).snes_ppu),
    ] {
        if !brightness_cache_is_valid(ppu) {
            game.g_fail = true;
            eprintln!(
                "@{}: {side} brightness cache invalid for marker {}",
                game.snes_frame_counter, ppu.lastBrightnessMult
            );
        }
    }
    if game.VERIFY_ACTIVE {
        // Both headless hosts register initialized 256x240 BGRA buffers with
        // pitch 1024. Compare before recovery so a failed mine frame is visible.
        let mine = &*(*game.g_snes).my_ppu;
        let theirs = &*(*game.g_snes).snes_ppu;
        assert_eq!(mine.renderPitch, 1024);
        assert_eq!(theirs.renderPitch, 1024);
        let mine_pixels = core::slice::from_raw_parts(mine.renderBuffer, 256 * 240 * 4);
        let their_pixels = core::slice::from_raw_parts(theirs.renderBuffer, 256 * 240 * 4);
        if mine_pixels != their_pixels {
            game.g_fail = true;
            let byte = mine_pixels
                .iter()
                .zip(their_pixels)
                .position(|(a, b)| a != b)
                .unwrap();
            eprintln!(
                "@{}: Rendered frame compare failed at pixel ({}, {}), byte {}: ${:02X} != ${:02X}",
                game.snes_frame_counter,
                (byte / 4) % 256,
                byte / 1024,
                byte % 4,
                mine_pixels[byte],
                their_pixels[byte]
            );
        }
    }
    if !game.g_use_my_apu_code {
        if let Some(field) = (*b).audio.first_difference(&(*a).audio) {
            game.g_fail = true;
            eprintln!(
                "@{}: Physical APU state compare failed: {}",
                game.snes_frame_counter, field
            );
        }
    }
    if (*b).status_flags != (*a).status_flags {
        game.g_fail = true;
        eprintln!(
            "@{}: PPU status flags (rangeOver, timeOver, evenFrame) compare failed: mine {:?} != theirs {:?}",
            game.snes_frame_counter,
            (*b).status_flags,
            (*a).status_flags
        );
    }
    if ((*b).brightness, (*b).forced_blank) != ((*a).brightness, (*a).forced_blank) {
        game.g_fail = true;
        eprintln!(
            "@{}: INIDISP (brightness, forced blank) compare failed: mine {:?} != theirs {:?}",
            game.snes_frame_counter,
            ((*b).brightness, (*b).forced_blank),
            ((*a).brightness, (*a).forced_blank)
        );
    }
    if (*b).display_geometry != (*a).display_geometry {
        game.g_fail = true;
        eprintln!(
            "@{}: Display geometry (interlace, OBJ interlace, overscan, pseudo-hires, frame interlace, frame overscan) compare failed: mine {:?} != theirs {:?}",
            game.snes_frame_counter, (*b).display_geometry, (*a).display_geometry
        );
    }
    if (*b).color_selection != (*a).color_selection {
        game.g_fail = true;
        eprintln!(
            "@{}: CGWSEL (clip, prevent math, subscreen, direct color) compare failed: mine {:?} != theirs {:?}",
            game.snes_frame_counter, (*b).color_selection, (*a).color_selection
        );
    }
    if (*b).color_math != (*a).color_math {
        game.g_fail = true;
        eprintln!(
            "@{}: CGADSUB (subtract, half, layer enables) compare failed: mine {:?} != theirs {:?}",
            game.snes_frame_counter,
            (*b).color_math,
            (*a).color_math
        );
    }
    if (*b).fixed_color != (*a).fixed_color {
        game.g_fail = true;
        eprintln!(
            "@{}: Fixed color (R, G, B) compare failed: mine {:?} != theirs {:?}",
            game.snes_frame_counter,
            (*b).fixed_color,
            (*a).fixed_color
        );
    }
    if (*b).window_bounds != (*a).window_bounds {
        game.g_fail = true;
        eprintln!(
            "@{}: Window boundaries (WH0, WH1, WH2, WH3) compare failed: mine {:?} != theirs {:?}",
            game.snes_frame_counter,
            (*b).window_bounds,
            (*a).window_bounds
        );
    }
    if (*b).window_select != (*a).window_select {
        game.g_fail = true;
        eprintln!(
            "@{}: Window selection (raw, enables, inversion, logic) compare failed: mine {:?} != theirs {:?}",
            game.snes_frame_counter, (*b).window_select, (*a).window_select
        );
    }
    if (*b).screen_routing != (*a).screen_routing {
        game.g_fail = true;
        eprintln!(
            "@{}: Screen routing (enables, windows, layer flags) compare failed: mine {:?} != theirs {:?}",
            game.snes_frame_counter, (*b).screen_routing, (*a).screen_routing
        );
    }
    if (*b).bg_address != (*a).bg_address {
        game.g_fail = true;
        eprintln!(
            "@{}: BG address (wide, high, tilemap, tile data) compare failed: mine {:?} != theirs {:?}",
            game.snes_frame_counter, (*b).bg_address, (*a).bg_address
        );
    }
    if (*b).bg_scroll != (*a).bg_scroll {
        game.g_fail = true;
        eprintln!(
            "@{}: BG scroll (horizontal, vertical) compare failed: mine {:?} != theirs {:?}",
            game.snes_frame_counter,
            (*b).bg_scroll,
            (*a).bg_scroll
        );
    }
    if (*b).bgmode != (*a).bgmode {
        game.g_fail = true;
        eprintln!(
            "@{}: BGMODE (mode, BG3 priority, tile sizes) compare failed: mine {:?} != theirs {:?}",
            game.snes_frame_counter,
            (*b).bgmode,
            (*a).bgmode
        );
    }
    if (*b).mosaic != (*a).mosaic {
        game.g_fail = true;
        eprintln!(
            "@{}: Mosaic (BG enables, size, start line) compare failed: mine {:?} != theirs {:?}",
            game.snes_frame_counter,
            (*b).mosaic,
            (*a).mosaic
        );
    }
    if (*b).mode7_flags != (*a).mode7_flags {
        game.g_fail = true;
        eprintln!(
            "@{}: Mode 7 flags (large field, character fill, X flip, Y flip, EXTBG) compare failed: mine {:?} != theirs {:?}",
            game.snes_frame_counter,
            (*b).mode7_flags,
            (*a).mode7_flags
        );
    }
    if (*b).mode7_matrix != (*a).mode7_matrix {
        game.g_fail = true;
        eprintln!(
            "@{}: Mode 7 matrix compare failed: mine {:?} != theirs {:?}",
            game.snes_frame_counter,
            (*b).mode7_matrix,
            (*a).mode7_matrix
        );
    }
    if (*b).counters != (*a).counters {
        game.g_fail = true;
        eprintln!(
            "@{}: PPU counter compare failed: mine {:?} != theirs {:?}",
            game.snes_frame_counter,
            (*b).counters,
            (*a).counters
        );
    }
    if ((*b).ppu1_open_bus, (*b).ppu2_open_bus) != ((*a).ppu1_open_bus, (*a).ppu2_open_bus) {
        game.g_fail = true;
        eprintln!(
            "@{}: PPU open bus compare failed: mine {:02X}/{:02X} != theirs {:02X}/{:02X}",
            game.snes_frame_counter,
            (*b).ppu1_open_bus,
            (*b).ppu2_open_bus,
            (*a).ppu1_open_bus,
            (*a).ppu2_open_bus
        );
    }
    let mine_cgram_port = (
        (*b).cgram_pointer,
        (*b).cgram_second_write,
        (*b).cgram_buffer,
    );
    let their_cgram_port = (
        (*a).cgram_pointer,
        (*a).cgram_second_write,
        (*a).cgram_buffer,
    );
    if mine_cgram_port != their_cgram_port {
        game.g_fail = true;
        eprintln!(
            "@{}: CGRAM port (address, second byte, buffer) compare failed: mine {:?} != theirs {:?}",
            game.snes_frame_counter, mine_cgram_port, their_cgram_port
        );
    }
    let mine_vram_port = (
        (*b).vram_pointer,
        (*b).vram_increment,
        (*b).vram_increment_on_high,
        (*b).vram_remap_mode,
    );
    let their_vram_port = (
        (*a).vram_pointer,
        (*a).vram_increment,
        (*a).vram_increment_on_high,
        (*a).vram_remap_mode,
    );
    if mine_vram_port != their_vram_port {
        game.g_fail = true;
        eprintln!(
            "@{}: VRAM port (address, increment, high, remap) compare failed: mine {:?} != theirs {:?}",
            game.snes_frame_counter, mine_vram_port, their_vram_port
        );
    }
    if (*b).vram_read_buffer != (*a).vram_read_buffer {
        game.g_fail = true;
        eprintln!(
            "@{}: VRAM read buffer compare failed: mine ${:04X} != theirs ${:04X}",
            game.snes_frame_counter,
            (*b).vram_read_buffer,
            (*a).vram_read_buffer
        );
    }
    if (*b).write_latches != (*a).write_latches {
        game.g_fail = true;
        eprintln!(
            "@{}: PPU write latch compare failed (scrollPrev, scrollPrev2, m7prev): mine {:?} != theirs {:?}",
            game.snes_frame_counter,
            (*b).write_latches,
            (*a).write_latches
        );
    }
    if (*b).cgram != (*a).cgram {
        game.g_fail = true;
        eprintln!(
            "@{}: CGRAM compare failed (mine != theirs)",
            game.snes_frame_counter
        );
        for (index, (&mine, &theirs)) in (*b).cgram.iter().zip(&(*a).cgram).enumerate() {
            if mine != theirs {
                eprintln!("CGRAM ${index:03X}: ${mine:04X} != ${theirs:04X}");
            }
        }
    }
    memcpy(
        (&raw mut (*b).ram as *mut uint8).offset(0) as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*a).ram as *mut uint8).offset(0) as *mut uint8 as *const ::core::ffi::c_void,
        0x51 as size_t,
    );
    memcpy(
        (&raw mut (*b).ram as *mut uint8).offset(0x1f5b) as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*a).ram as *mut uint8).offset(0x1f5b) as *mut uint8
            as *const ::core::ffi::c_void,
        (0x100 as ::core::ffi::c_int - 0x5b as ::core::ffi::c_int) as size_t,
    );
    memcpy(
        (&raw mut (*b).ram as *mut uint8).offset(0xad) as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*a).ram as *mut uint8).offset(0xad) as *mut uint8 as *const ::core::ffi::c_void,
        4 as size_t,
    );
    memcpy(
        (&raw mut (*b).ram as *mut uint8).offset(0x5ed) as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*a).ram as *mut uint8).offset(0x5ed) as *mut uint8 as *const ::core::ffi::c_void,
        8 as size_t,
    );
    memcpy(
        (&raw mut (*b).ram as *mut uint8).offset(0x5bc) as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*a).ram as *mut uint8).offset(0x5bc) as *mut uint8 as *const ::core::ffi::c_void,
        9 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0x60b) as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0x60b) as *mut uint8 as *const ::core::ffi::c_void,
        6 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0x611) as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0x611) as *mut uint8 as *const ::core::ffi::c_void,
        6 as size_t,
    );
    memcpy(
        (&raw mut (*b).ram as *mut uint8).offset(0x641) as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*a).ram as *mut uint8).offset(0x641) as *mut uint8 as *const ::core::ffi::c_void,
        2 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0x77e) as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0x77e) as *mut uint8 as *const ::core::ffi::c_void,
        5 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0x78f) as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0x78f) as *mut uint8 as *const ::core::ffi::c_void,
        2 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0x7b7) as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0x7b7) as *mut uint8 as *const ::core::ffi::c_void,
        2 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0x933) as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0x933) as *mut uint8 as *const ::core::ffi::c_void,
        10 as size_t,
    );
    memcpy(
        (&raw mut (*b).ram as *mut uint8).offset(0xb24) as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*a).ram as *mut uint8).offset(0xb24) as *mut uint8 as *const ::core::ffi::c_void,
        4 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0xd1e) as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0xd1e) as *mut uint8 as *const ::core::ffi::c_void,
        2 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0xd82) as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0xd82) as *mut uint8 as *const ::core::ffi::c_void,
        8 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0xd9c) as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0xd9c) as *mut uint8 as *const ::core::ffi::c_void,
        2 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0xdd2) as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0xdd2) as *mut uint8 as *const ::core::ffi::c_void,
        6 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0xd8a) as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0xd8a) as *mut uint8 as *const ::core::ffi::c_void,
        6 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0xe20) as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0xe20) as *mut uint8 as *const ::core::ffi::c_void,
        (0xe46 as ::core::ffi::c_int - 0xe20 as ::core::ffi::c_int) as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0xe54) as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0xe54) as *mut uint8 as *const ::core::ffi::c_void,
        2 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0xe02) as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0xe02) as *mut uint8 as *const ::core::ffi::c_void,
        2 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0xe4a) as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0xe4a) as *mut uint8 as *const ::core::ffi::c_void,
        2 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0xe56) as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0xe56) as *mut uint8 as *const ::core::ffi::c_void,
        4 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0x1784) as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0x1784) as *mut uint8
            as *const ::core::ffi::c_void,
        8 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0x1790) as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0x1790) as *mut uint8
            as *const ::core::ffi::c_void,
        4 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0x17a8) as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0x17a8) as *mut uint8
            as *const ::core::ffi::c_void,
        4 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0x1834) as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0x1834) as *mut uint8
            as *const ::core::ffi::c_void,
        8 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0x184a) as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0x184a) as *mut uint8
            as *const ::core::ffi::c_void,
        18 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0x186e) as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0x186e) as *mut uint8
            as *const ::core::ffi::c_void,
        (16 as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0x18a6) as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0x18a6) as *mut uint8
            as *const ::core::ffi::c_void,
        2 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0x189a) as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0x189a) as *mut uint8
            as *const ::core::ffi::c_void,
        12 as size_t,
    );
    memcpy(
        (&raw mut (*b).ram as *mut uint8).offset(0x1966) as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*a).ram as *mut uint8).offset(0x1966) as *mut uint8
            as *const ::core::ffi::c_void,
        6 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0xef74) as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0xef74) as *mut uint8
            as *const ::core::ffi::c_void,
        4 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut uint8).offset(0xf37a) as *mut uint8 as *mut ::core::ffi::c_void,
        (&raw mut (*b).ram as *mut uint8).offset(0xf37a) as *mut uint8
            as *const ::core::ffi::c_void,
        6 as size_t,
    );
    if memcmp(
        &raw mut (*b).ram as *mut uint8 as *const ::core::ffi::c_void,
        &raw mut (*a).ram as *mut uint8 as *const ::core::ffi::c_void,
        0x20000 as ::core::ffi::c_int as size_t,
    ) != 0
    {
        game.g_fail = true_0 != 0;
        fprintf(
            stderr,
            b"@%d: Memory compare failed (mine != theirs, prev):\n\0" as *const u8
                as *const ::core::ffi::c_char,
            game.snes_frame_counter,
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
                            != (*b).ram[i.wrapping_add(1 as size_t) as usize] as ::core::ffi::c_int
                    {
                        let read_word = |ram: &[u8; 0x20000]| {
                            i32::from(u16::from_le_bytes([ram[i as usize], ram[i as usize + 1]]))
                        };
                        fprintf(
                            stderr,
                            b"0x%.6X: %.4X != %.4X (%.4X)\n\0" as *const u8
                                as *const ::core::ffi::c_char,
                            i as ::core::ffi::c_int,
                            read_word(&(*b).ram),
                            read_word(&(*a).ram),
                            read_word(&(*prev).ram),
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
            game.g_fail = true_0 != 0;
        }
        fprintf(
            stderr,
            b"  total of %d failed bytes\n\0" as *const u8 as *const ::core::ffi::c_char,
            j,
        );
    }
    if memcmp(
        &raw mut (*b).sram as *mut uint8 as *const ::core::ffi::c_void,
        &raw mut (*a).sram as *mut uint8 as *const ::core::ffi::c_void,
        0x2000 as size_t,
    ) != 0
    {
        game.g_fail = true_0 != 0;
        fprintf(
            stderr,
            b"@%d: SRAM compare failed (mine != theirs, prev):\n\0" as *const u8
                as *const ::core::ffi::c_char,
            game.snes_frame_counter,
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
                        && (*a).sram[i_0.wrapping_add(1 as size_t) as usize] as ::core::ffi::c_int
                            != (*b).sram[i_0.wrapping_add(1 as size_t) as usize]
                                as ::core::ffi::c_int
                    {
                        fprintf(
                            stderr,
                            b"0x%.6X: %.4X != %.4X (%.4X)\n\0" as *const u8
                                as *const ::core::ffi::c_char,
                            i_0 as ::core::ffi::c_int,
                            *((&raw mut (*b).sram as *mut uint8).offset(i_0 as isize) as *mut uint8
                                as *mut uint16) as ::core::ffi::c_int,
                            *((&raw mut (*a).sram as *mut uint8).offset(i_0 as isize) as *mut uint8
                                as *mut uint16) as ::core::ffi::c_int,
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
            game.g_fail = true_0 != 0;
        }
        fprintf(
            stderr,
            b"  total of %d failed bytes\n\0" as *const u8 as *const ::core::ffi::c_char,
            j_0,
        );
    }
    if memcmp(
        &raw mut (*b).vram as *mut uint16 as *const ::core::ffi::c_void,
        &raw mut (*a).vram as *mut uint16 as *const ::core::ffi::c_void,
        (::core::mem::size_of::<uint16>() as size_t).wrapping_mul(0x8000 as size_t),
    ) != 0
    {
        game.g_fail = true_0 != 0;
        fprintf(
            stderr,
            b"@%d: VRAM compare failed (mine != theirs, prev):\n\0" as *const u8
                as *const ::core::ffi::c_char,
            game.snes_frame_counter,
        );
        let mut i_1: size_t = 0 as size_t;
        let mut j_1: size_t = 0 as size_t;
        while i_1 < 0x8000 as size_t {
            if (*a).vram[i_1 as usize] as ::core::ffi::c_int
                != (*b).vram[i_1 as usize] as ::core::ffi::c_int
            {
                fprintf(
                    stderr,
                    b"0x%.6X: %.4X != %.4X (%.4X)\n\0" as *const u8 as *const ::core::ffi::c_char,
                    i_1 as ::core::ffi::c_int,
                    (*b).vram[i_1 as usize] as ::core::ffi::c_int,
                    (*a).vram[i_1 as usize] as ::core::ffi::c_int,
                    (*prev).vram[i_1 as usize] as ::core::ffi::c_int,
                );
                game.g_fail = true_0 != 0;
                j_1 = j_1.wrapping_add(1);
                if j_1 >= 32 as size_t {
                    break;
                }
            }
            i_1 = i_1.wrapping_add(1);
        }
    }
    if (*b).oam != (*a).oam || (*b).high_oam != (*a).high_oam {
        game.g_fail = true;
        eprintln!("@{}: OAM memory compare failed", game.snes_frame_counter);
        let low = (*b)
            .oam
            .iter()
            .zip(&(*a).oam)
            .zip(&(*prev).oam)
            .enumerate()
            .filter(|(_, ((mine, theirs), _))| mine != theirs)
            .map(|(index, ((mine, theirs), previous))| (index * 2, *mine, *theirs, *previous));
        let high = (*b)
            .high_oam
            .iter()
            .zip(&(*a).high_oam)
            .zip(&(*prev).high_oam)
            .enumerate()
            .filter(|(_, ((mine, theirs), _))| mine != theirs)
            .map(|(index, ((mine, theirs), previous))| {
                (
                    512 + index,
                    u16::from(*mine),
                    u16::from(*theirs),
                    u16::from(*previous),
                )
            });
        for (offset, mine, theirs, previous) in low.chain(high).take(16) {
            eprintln!("OAM byte ${offset:03X}: mine ${mine:04X} != theirs ${theirs:04X}, previous ${previous:04X}");
        }
    }
    if (*b).sprite != (*a).sprite {
        game.g_fail = true;
        eprintln!(
            "@{}: Sprite state compare failed: mine {:?} != theirs {:?}",
            game.snes_frame_counter,
            (*b).sprite,
            (*a).sprite
        );
    }
}
// Snapshot storage is separate from the live emulator allocations. Narrow
// borrows let the compiler retain that distinction across the bulk copies.
unsafe fn MakeSnapshot(s: &mut Snapshot, snes: &Snes, c: &Cpu, player: *mut SpcPlayer) {
    (*s).player.capture(player);
    (*s).audio.capture(snes);
    (*s).write_latches = [
        (*(*snes).ppu).scrollPrev,
        (*(*snes).ppu).scrollPrev2,
        (*(*snes).ppu).m7prev,
    ];
    (*s).cgram = (*(*snes).ppu).cgram;
    (*s).cgram_pointer = (*(*snes).ppu).cgramPointer;
    (*s).cgram_second_write = (*(*snes).ppu).cgramSecondWrite;
    (*s).cgram_buffer = (*(*snes).ppu).cgramBuffer;
    (*s).ppu1_open_bus = (*(*snes).ppu).ppu1openBus;
    (*s).ppu2_open_bus = (*(*snes).ppu).ppu2openBus;
    (*s).vram_read_buffer = (*(*snes).ppu).vramReadBuffer;
    (*s).vram_pointer = (*(*snes).ppu).vramPointer;
    (*s).vram_increment = (*(*snes).ppu).vramIncrement;
    (*s).vram_increment_on_high = (*(*snes).ppu).vramIncrementOnHigh;
    (*s).vram_remap_mode = (*(*snes).ppu).vramRemapMode;
    (*s).a = (*c).a as uint16;
    (*s).x = (*c).x as uint16;
    (*s).y = (*c).y as uint16;
    (*s).sp = (*c).sp as uint16;
    (*s).dp = (*c).dp as uint16;
    (*s).db = (*c).db as uint8;
    (*s).pc = (*c).pc as uint16;
    (*s).k = (*c).k as uint8;
    (*s).flags = cpu_getFlags(&*c) as uint8;
    (*s).cpu_waiting = (*c).waiting;
    (*s).cpu_emulation = (*c).e;
    (*s).cpu_stopped = (*c).stopped;
    (*s).cpu_irq_pending = (*c).irqWanted;
    (*s).cpu_nmi_pending = (*c).nmiWanted;
    (*s).video_clock = (snes.hPos, snes.vPos, snes.frames);
    (*s).interrupt_latches = (snes.inNmi, snes.inIrq, snes.inVblank);
    (*s).interrupt_enables = (snes.hIrqEnabled, snes.vIrqEnabled, snes.nmiEnabled);
    (*s).h_timer = snes.hTimer;
    (*s).snes_open_bus = snes.openBus;
    (*s).input_control = (snes.ppuLatch, snes.autoJoyRead, snes.autoJoyTimer);
    (*s).wram_port_address = snes.ramAdr;
    let dma = snes.dma;
    (*s).dma_channels = (*dma).channel;
    (*s).dma_timing = ((*dma).hdmaTimer, (*dma).dmaTimer, (*dma).dmaBusy);
    (*s).arithmetic = (
        snes.multiplyA,
        snes.multiplyResult,
        snes.divideA,
        snes.divideResult,
    );
    (*s).vTimer = (*snes).vTimer;
    memcpy(
        &raw mut (*s).ram as *mut uint8 as *mut ::core::ffi::c_void,
        (*snes).ram as *const ::core::ffi::c_void,
        0x20000 as ::core::ffi::c_int as size_t,
    );
    memcpy(
        &raw mut (*s).sram as *mut uint8 as *mut ::core::ffi::c_void,
        (*(*snes).cart).ram as *const ::core::ffi::c_void,
        (*(*snes).cart).ramSize as size_t,
    );
    memcpy(
        &raw mut (*s).vram as *mut uint16 as *mut ::core::ffi::c_void,
        &raw mut (*(*snes).ppu).vram as *mut uint16_t as *const ::core::ffi::c_void,
        (::core::mem::size_of::<uint16>() as size_t).wrapping_mul(0x8000 as size_t),
    );
    let ppu = &*(*snes).ppu;
    (*s).brightness = ppu.brightness;
    (*s).forced_blank = ppu.forcedBlank;
    (*s).fixed_color = [ppu.fixedColorR, ppu.fixedColorG, ppu.fixedColorB];
    (*s).color_math = (ppu.subtractColor, ppu.halfColor, ppu.mathEnabled);
    (*s).display_geometry = [
        ppu.interlace,
        ppu.objInterlace,
        ppu.overscan,
        ppu.pseudoHires,
        ppu.frameInterlace,
        ppu.frameOverscan,
    ];
    (*s).color_selection = (
        ppu.clipMode,
        ppu.preventMathMode,
        ppu.addSubscreen,
        ppu.directColor,
    );
    (*s).window_bounds = [
        ppu.window1left,
        ppu.window1right,
        ppu.window2left,
        ppu.window2right,
    ];
    (*s).window_select = (
        ppu.windowsel,
        ppu.windowLayer.map(|w| {
            (
                w.window1enabled,
                w.window2enabled,
                w.window1inversed,
                w.window2inversed,
                w.maskLogic,
            )
        }),
    );
    (*s).screen_routing = (
        ppu.screenEnabled,
        ppu.screenWindowed,
        ppu.layer.map(|layer| {
            [
                layer.mainScreenEnabled,
                layer.subScreenEnabled,
                layer.mainScreenWindowed,
                layer.subScreenWindowed,
            ]
        }),
    );
    (*s).bg_address = ppu
        .bgLayer
        .map(|bg| (bg.tilemapWider, bg.tilemapHigher, bg.tilemapAdr, bg.tileAdr));
    (*s).bg_scroll = ppu.bgLayer.map(|bg| (bg.hScroll, bg.vScroll));
    (*s).bgmode = (ppu.mode, ppu.bg3priority, ppu.bgLayer.map(|bg| bg.bigTiles));
    (*s).mosaic = (
        ppu.bgLayer.map(|bg| bg.mosaicEnabled),
        ppu.mosaicSize,
        ppu.mosaicStartLine,
    );
    (*s).mode7_matrix = ppu.m7matrix;
    (*s).mode7_flags = [
        ppu.m7largeField,
        ppu.m7charFill,
        ppu.m7xFlip,
        ppu.m7yFlip,
        ppu.m7extBg,
    ];
    (*s).status_flags = [ppu.rangeOver, ppu.timeOver, ppu.evenFrame];
    (*s).counters = CounterSnapshot {
        horizontal: ppu.hCount,
        vertical: ppu.vCount,
        horizontal_second_byte: ppu.hCountSecond,
        vertical_second_byte: ppu.vCountSecond,
        latched: ppu.countersLatched,
    };
    (*s).oam = ppu.oam;
    (*s).high_oam = ppu.highOam;
    (*s).sprite = SpriteSnapshot {
        address: ppu.oamAdr,
        written_address: ppu.oamAdrWritten,
        high_table: ppu.oamInHigh,
        written_high_table: ppu.oamInHighWritten,
        second_byte: ppu.oamSecondWrite,
        buffer: ppu.oamBuffer,
        priority_rotation: ppu.objPriority,
        tile_address_1: ppu.objTileAdr1,
        tile_address_2: ppu.objTileAdr2,
        size: ppu.objSize,
        legacy_pixel_prefix: ppu.objPixelBufferXX[..19].try_into().unwrap(),
    };
}
unsafe fn RestoreSnapshot(s: &Snapshot, snes: &mut Snes, c: &mut Cpu) {
    (*s).audio.restore(snes);
    let ppu = &mut *(*snes).ppu;
    ppu.brightness = (*s).brightness;
    ppu.forcedBlank = (*s).forced_blank;
    [ppu.fixedColorR, ppu.fixedColorG, ppu.fixedColorB] = (*s).fixed_color;
    (ppu.subtractColor, ppu.halfColor, ppu.mathEnabled) = (*s).color_math;
    [
        ppu.interlace,
        ppu.objInterlace,
        ppu.overscan,
        ppu.pseudoHires,
        ppu.frameInterlace,
        ppu.frameOverscan,
    ] = (*s).display_geometry;
    (
        ppu.clipMode,
        ppu.preventMathMode,
        ppu.addSubscreen,
        ppu.directColor,
    ) = (*s).color_selection;
    [
        ppu.window1left,
        ppu.window1right,
        ppu.window2left,
        ppu.window2right,
    ] = (*s).window_bounds;
    let (raw, windows) = (*s).window_select;
    ppu.windowsel = raw;
    for (w, (enabled1, enabled2, inverted1, inverted2, logic)) in
        ppu.windowLayer.iter_mut().zip(windows)
    {
        w.window1enabled = enabled1;
        w.window2enabled = enabled2;
        w.window1inversed = inverted1;
        w.window2inversed = inverted2;
        w.maskLogic = logic;
    }
    let (enabled, windowed, layers) = (*s).screen_routing;
    ppu.screenEnabled = enabled;
    ppu.screenWindowed = windowed;
    for (layer, flags) in ppu.layer.iter_mut().zip(layers) {
        [
            layer.mainScreenEnabled,
            layer.subScreenEnabled,
            layer.mainScreenWindowed,
            layer.subScreenWindowed,
        ] = flags;
    }
    for (bg, (wide, high, tilemap, tiles)) in ppu.bgLayer.iter_mut().zip((*s).bg_address) {
        bg.tilemapWider = wide;
        bg.tilemapHigher = high;
        bg.tilemapAdr = tilemap;
        bg.tileAdr = tiles;
    }
    for (bg, (horizontal, vertical)) in ppu.bgLayer.iter_mut().zip((*s).bg_scroll) {
        bg.hScroll = horizontal;
        bg.vScroll = vertical;
    }
    let (mode, bg3_priority, big_tiles) = (*s).bgmode;
    ppu.mode = mode;
    ppu.bg3priority = bg3_priority;
    for (bg, big_tiles) in ppu.bgLayer.iter_mut().zip(big_tiles) {
        bg.bigTiles = big_tiles;
    }
    let (enabled, size, start_line) = (*s).mosaic;
    for (bg, enabled) in ppu.bgLayer.iter_mut().zip(enabled) {
        bg.mosaicEnabled = enabled;
    }
    ppu.mosaicSize = size;
    ppu.mosaicStartLine = start_line;
    if !brightness_cache_is_valid(ppu) {
        ppu.lastBrightnessMult = 255;
    }
    ppu.m7matrix = (*s).mode7_matrix;
    [
        ppu.m7largeField,
        ppu.m7charFill,
        ppu.m7xFlip,
        ppu.m7yFlip,
        ppu.m7extBg,
    ] = (*s).mode7_flags;
    [ppu.rangeOver, ppu.timeOver, ppu.evenFrame] = (*s).status_flags;
    [ppu.scrollPrev, ppu.scrollPrev2, ppu.m7prev] = (*s).write_latches;
    (*(*snes).ppu).cgram = (*s).cgram;
    (*(*snes).ppu).cgramPointer = (*s).cgram_pointer;
    (*(*snes).ppu).cgramSecondWrite = (*s).cgram_second_write;
    (*(*snes).ppu).cgramBuffer = (*s).cgram_buffer;
    (*(*snes).ppu).ppu1openBus = (*s).ppu1_open_bus;
    (*(*snes).ppu).ppu2openBus = (*s).ppu2_open_bus;
    (*(*snes).ppu).vramReadBuffer = (*s).vram_read_buffer;
    (*(*snes).ppu).vramPointer = (*s).vram_pointer;
    (*(*snes).ppu).vramIncrement = (*s).vram_increment;
    (*(*snes).ppu).vramIncrementOnHigh = (*s).vram_increment_on_high;
    (*(*snes).ppu).vramRemapMode = (*s).vram_remap_mode;
    (*c).a = (*s).a as uint16_t;
    (*c).x = (*s).x as uint16_t;
    (*c).y = (*s).y as uint16_t;
    (*c).sp = (*s).sp as uint16_t;
    (*c).dp = (*s).dp as uint16_t;
    (*c).db = (*s).db as uint8_t;
    (*c).pc = (*s).pc as uint16_t;
    (*c).k = (*s).k as uint8_t;
    (*snes).vTimer = (*s).vTimer;
    (*c).e = (*s).cpu_emulation;
    (*c).stopped = (*s).cpu_stopped;
    (*c).irqWanted = (*s).cpu_irq_pending;
    (*c).nmiWanted = (*s).cpu_nmi_pending;
    cpu_setFlags((*s).flags as uint8_t, &mut *c);
    (*c).waiting = (*s).cpu_waiting;
    (snes.hPos, snes.vPos, snes.frames) = (*s).video_clock;
    (snes.inNmi, snes.inIrq, snes.inVblank) = (*s).interrupt_latches;
    (snes.hIrqEnabled, snes.vIrqEnabled, snes.nmiEnabled) = (*s).interrupt_enables;
    snes.hTimer = (*s).h_timer;
    snes.openBus = (*s).snes_open_bus;
    (snes.ppuLatch, snes.autoJoyRead, snes.autoJoyTimer) = (*s).input_control;
    snes.ramAdr = (*s).wram_port_address;
    let dma = snes.dma;
    (*dma).channel = (*s).dma_channels;
    ((*dma).hdmaTimer, (*dma).dmaTimer, (*dma).dmaBusy) = (*s).dma_timing;
    (
        snes.multiplyA,
        snes.multiplyResult,
        snes.divideA,
        snes.divideResult,
    ) = (*s).arithmetic;
    memcpy(
        (*snes).ram as *mut ::core::ffi::c_void,
        &raw const (*s).ram as *const uint8 as *const ::core::ffi::c_void,
        0x20000 as ::core::ffi::c_int as size_t,
    );
    memcpy(
        (*(*snes).cart).ram as *mut ::core::ffi::c_void,
        &raw const (*s).sram as *const uint8 as *const ::core::ffi::c_void,
        0x2000 as size_t,
    );
    memcpy(
        &raw mut (*(*snes).ppu).vram as *mut uint16_t as *mut ::core::ffi::c_void,
        &raw const (*s).vram as *const uint16 as *const ::core::ffi::c_void,
        (::core::mem::size_of::<uint16>() as size_t).wrapping_mul(0x8000 as size_t),
    );
    let ppu = &mut *(*snes).ppu;
    ppu.hCount = (*s).counters.horizontal;
    ppu.vCount = (*s).counters.vertical;
    ppu.hCountSecond = (*s).counters.horizontal_second_byte;
    ppu.vCountSecond = (*s).counters.vertical_second_byte;
    ppu.countersLatched = (*s).counters.latched;
    ppu.oam = (*s).oam;
    ppu.highOam = (*s).high_oam;
    ppu.oamAdr = (*s).sprite.address;
    ppu.oamAdrWritten = (*s).sprite.written_address;
    ppu.oamInHigh = (*s).sprite.high_table;
    ppu.oamInHighWritten = (*s).sprite.written_high_table;
    ppu.oamSecondWrite = (*s).sprite.second_byte;
    ppu.oamBuffer = (*s).sprite.buffer;
    ppu.objPriority = (*s).sprite.priority_rotation;
    ppu.objTileAdr1 = (*s).sprite.tile_address_1;
    ppu.objTileAdr2 = (*s).sprite.tile_address_2;
    ppu.objSize = (*s).sprite.size;
    ppu.objPixelBufferXX[..19].copy_from_slice(&(*s).sprite.legacy_pixel_prefix);
}
pub unsafe fn RunAsmCode(
    mut pc: uint32,
    mut a: uint16,
    mut x: uint16,
    mut y: uint16,
    mut flags: ::core::ffi::c_int,
    game: &mut Game,
) -> ::core::ffi::c_int {
    let mut org_sp: uint16 = (*game.g_cpu).sp as uint16;
    let mut org_pc: uint16 = (*game.g_cpu).pc as uint16;
    let mut org_b: uint8 = (*game.g_cpu).db as uint8;
    let mut org_dp: uint16 = (*game.g_cpu).dp as uint16;
    printf(b"RunAsmCode!\n\0" as *const u8 as *const ::core::ffi::c_char);
    game.g_ram[0x1ffff as ::core::ffi::c_int as usize] = 1 as uint8;
    let mut dc: bool = (*game.g_snes).debug_cycles;
    (*game.g_cpu).db = (pc >> 16 as ::core::ffi::c_int) as uint8_t;
    (*game.g_cpu).a = a as uint16_t;
    (*game.g_cpu).x = x as uint16_t;
    (*game.g_cpu).y = y as uint16_t;
    (*game.g_cpu).spBreakpoint = (*game.g_cpu).sp;
    (*game.g_cpu).k = (pc >> 16 as ::core::ffi::c_int) as uint8_t;
    (*game.g_cpu).pc = (pc & 0xffff as uint32) as uint16_t;
    (*game.g_cpu).mf = flags & 1 as ::core::ffi::c_int != 0;
    (*game.g_cpu).xf = (flags & 2 as ::core::ffi::c_int) >> 1 as ::core::ffi::c_int != 0;
    game.g_calling_asm_from_c = true_0 != 0;
    while game.g_calling_asm_from_c {
        let mut pc_0: uint32 =
            (((*(*game.g_snes).cpu).k as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                | (*(*game.g_snes).cpu).pc as ::core::ffi::c_int) as uint32;
        if (*game.g_snes).debug_cycles {
            let mut line: [::core::ffi::c_char; 80] = [0; 80];
            getProcessorStateCpu(game.g_snes, &raw mut line as *mut ::core::ffi::c_char, game);
            puts(&raw mut line as *mut ::core::ffi::c_char);
            line[0 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_char;
        }
        cpu_runOpcode(game.g_cpu, game);
        while (*(*game.g_snes).dma).dmaBusy {
            dma_doDma((*game.g_snes).dma, game);
        }
        if flags & 1 as ::core::ffi::c_int != 0 {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < 10 as ::core::ffi::c_int {
                apu_cycle((*game.g_snes).apu, game);
                i += 1;
            }
        }
    }
    (*game.g_cpu).dp = org_dp as uint16_t;
    (*game.g_cpu).sp = org_sp as uint16_t;
    (*game.g_cpu).db = org_b as uint8_t;
    (*game.g_cpu).pc = org_pc as uint16_t;
    (*game.g_snes).debug_cycles = dc;
    game.g_calling_asm_from_c_ret
}
unsafe fn loadRom(
    mut name: *const ::core::ffi::c_char,
    mut snes: *mut Snes,
    game: &mut Game,
) -> bool {
    let mut length: size_t = 0 as size_t;
    let mut file: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    file = ReadWholeFile(name, &raw mut length) as *mut uint8_t;
    if file.is_null() {
        puts(b"Failed to read file\0" as *const u8 as *const ::core::ffi::c_char);
        return false_0 != 0;
    }
    let mut result: bool =
        snes_loadRom(snes as *mut Snes, file, length as ::core::ffi::c_int, game);
    free(file as *mut ::core::ffi::c_void);
    result
}
pub unsafe fn PatchBytes(
    mut addr: uint32,
    mut value: *const uint8,
    mut n: size_t,
    game: &mut Game,
) {
    let mut i: size_t = 0 as size_t;
    while i != n {
        *SnesRomPtr(addr, game).offset(i as isize) = *value.offset(i as isize) as uint8_t;
        i = i.wrapping_add(1);
    }
}
pub unsafe fn FixupCarry(mut addr: uint32, game: &mut Game) {
    *SnesRomPtr(addr, game) = 0 as uint8_t;
}

pub unsafe fn RtlUpdateSnesPatchForBugfix(game: &mut Game) {
    game.currently_installed_bug_fix_counter =
        *((&raw mut game.g_ram as *mut uint8).offset(0xc2) as *mut uint16);
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
        game,
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
        (if (*((&raw mut game.g_ram as *mut uint8).offset(0xc2) as *mut uint16)
            as ::core::ffi::c_int)
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
        game,
    );
    let mut t_1: [uint8; 2] = [
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x8584cc as uint32,
        &raw mut t_1 as *mut uint8,
        ::core::mem::size_of::<[uint8; 2]>() as size_t,
        game,
    );
}
pub unsafe fn SnesInit(mut filename: *const ::core::ffi::c_char, game: &mut Game) -> *mut Snes {
    game.g_snes = snes_init(&raw mut game.g_ram as *mut uint8_t) as *mut Snes;
    game.g_cpu = (*game.g_snes).cpu;
    let mut loaded: bool = loadRom(filename, game.g_snes, game);
    if !loaded {
        return ::core::ptr::null_mut::<Snes>();
    }
    game.g_sram = (*(*game.g_snes).cart).ram as *mut uint8;
    game.g_rom = (*(*game.g_snes).cart).rom;
    game.register_rom(game.g_rom, (*(*game.g_snes).cart).romSize as usize);
    RtlSetupEmuCallbacks(
        ::core::ptr::null_mut::<uint8>(),
        Some(RtlRunFrameCompare as unsafe fn(uint16, ::core::ffi::c_int, &mut Game) -> ()),
        None,
        game,
    );
    *(&raw mut game.g_ram as *mut uint8).offset(0x611) = 1 as uint8;
    let mut t: [uint8; 3] = [
        0x20 as ::core::ffi::c_int as uint8,
        0xf as ::core::ffi::c_int as uint8,
        0xf7 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x82896b as uint32,
        &raw mut t as *mut uint8,
        3 as size_t,
        game,
    );
    let mut t_0: [uint8; 3] = [
        0x7c as ::core::ffi::c_int as uint8,
        0x81 as ::core::ffi::c_int as uint8,
        0x89 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x82f70f as uint32,
        &raw mut t_0 as *mut uint8,
        3 as size_t,
        game,
    );
    let mut t_1: [uint8; 2] = [
        0x5f as ::core::ffi::c_int as uint8,
        0xf7 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x8ba362 as uint32,
        &raw mut t_1 as *mut uint8,
        ::core::mem::size_of::<[uint8; 2]>() as size_t,
        game,
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
        game,
    );
    let mut t_3: [uint8; 2] = [
        0xa as ::core::ffi::c_int as uint8,
        0xa as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x8584b2 as uint32,
        &raw mut t_3 as *mut uint8,
        2 as size_t,
        game,
    );
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
        game,
    );
    let mut t_5: [uint8; 2] = [
        0xd3 as ::core::ffi::c_int as uint8,
        0xef as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x848243 as uint32,
        &raw mut t_5 as *mut uint8,
        ::core::mem::size_of::<[uint8; 2]>() as size_t,
        game,
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
        game,
    );
    let mut t_7: [uint8; 2] = [
        0xa6 as ::core::ffi::c_int as uint8,
        0xf4 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x8685bd as uint32,
        &raw mut t_7 as *mut uint8,
        ::core::mem::size_of::<[uint8; 2]>() as size_t,
        game,
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
        game,
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
        game,
    );
    let mut t_10: [uint8; 2] = [
        0x28 as ::core::ffi::c_int as uint8,
        0x60 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x82897a as uint32,
        &raw mut t_10 as *mut uint8,
        ::core::mem::size_of::<[uint8; 2]>() as size_t,
        game,
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
        game,
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
        game,
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
        game,
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
        game,
    );
    let mut t_15: [uint8; 1] = [0x6b as ::core::ffi::c_int as uint8];
    PatchBytes(
        0xa0896f as uint32,
        &raw mut t_15 as *mut uint8,
        ::core::mem::size_of::<[uint8; 1]>() as size_t,
        game,
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
        game,
    );
    let mut t_17: [uint8; 1] = [0x60 as ::core::ffi::c_int as uint8];
    PatchBytes(
        0x8085f6 as uint32,
        &raw mut t_17 as *mut uint8,
        ::core::mem::size_of::<[uint8; 1]>() as size_t,
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
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
        game,
    );
    if game.g_use_my_apu_code {
        let mut t_74: [uint8; 1] = [0x60 as ::core::ffi::c_int as uint8];
        PatchBytes(
            0x808028 as uint32,
            &raw mut t_74 as *mut uint8,
            ::core::mem::size_of::<[uint8; 1]>() as size_t,
            game,
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
        game,
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
        game,
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
        game,
    );
    let mut t_78: [uint8; 1] = [0x6 as ::core::ffi::c_int as uint8];
    PatchBytes(
        0x828a67 as uint32,
        &raw mut t_78 as *mut uint8,
        ::core::mem::size_of::<[uint8; 1]>() as size_t,
        game,
    );
    let mut t_79: [uint8; 2] = [
        0 as ::core::ffi::c_int as uint8,
        0x2e as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x82831e as uint32,
        &raw mut t_79 as *mut uint8,
        ::core::mem::size_of::<[uint8; 2]>() as size_t,
        game,
    );
    let mut t_80: [uint8; 2] = [
        0xa5 as ::core::ffi::c_int as uint8,
        0x25 as ::core::ffi::c_int as uint8,
    ];
    PatchBytes(
        0x91c234 as uint32,
        &raw mut t_80 as *mut uint8,
        ::core::mem::size_of::<[uint8; 2]>() as size_t,
        game,
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
        game,
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
        game,
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
        game,
    );
    RtlUpdateSnesPatchForBugfix(game);
    let mut i: size_t = 0 as size_t;
    while i
        != (::core::mem::size_of::<[uint32; 155]>() as usize)
            .wrapping_div(::core::mem::size_of::<uint32>() as usize)
    {
        let mut t_84: uint8 = *SnesRomPtr(kPatchedCarrys[i as usize], game);
        if t_84 != 0 {
            game.kPatchedCarrysOrg[i as usize] = t_84;
            FixupCarry(kPatchedCarrys[i as usize], game);
        } else {
            printf(
                b"0x%x double patched!\n\0" as *const u8 as *const ::core::ffi::c_char,
                kPatchedCarrys[i as usize],
            );
        }
        i = i.wrapping_add(1);
    }
    PatchBugs(1 as uint32, 0 as uint32, game);
    game.g_snes
}
pub unsafe fn DebugGameOverMenu() {
    '_c2rust_label: {
        __assert_fail(
            b"0\0" as *const u8 as *const ::core::ffi::c_char,
            b"src/sm_cpu_infra.c\0" as *const u8 as *const ::core::ffi::c_char,
            914 as ::core::ffi::c_uint,
            b"void DebugGameOverMenu(void)\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
}
pub unsafe fn RunCpuUntilPC(mut pc1: uint32, mut pc2: uint32, game: &mut Game) -> uint32 {
    loop {
        if (*game.g_snes).debug_cycles {
            let mut line: [::core::ffi::c_char; 80] = [0; 80];
            getProcessorStateCpu(game.g_snes, &raw mut line as *mut ::core::ffi::c_char, game);
            puts(&raw mut line as *mut ::core::ffi::c_char);
        }
        cpu_runOpcode(game.g_cpu, game);
        let mut addr: uint32 =
            (((*(*game.g_snes).cpu).k as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                | (*(*game.g_snes).cpu).pc as ::core::ffi::c_int) as uint32;
        if addr == pc1 || addr == pc2 {
            return addr;
        }
    }
}
pub unsafe fn RunOneFrameOfGame_Emulated(game: &mut Game) {
    let mut bug_fix_bak: uint16 =
        *((&raw mut game.g_ram as *mut uint8).offset(0xc2) as *mut uint16);
    let cpu = (*game.g_snes).cpu;
    if (*cpu).k == 0x80 && (*cpu).pc == 0x8459 {
        RunCpuUntilPC(0x808460, 0x808460, game);
        (*cpu).pc = 0x8462;
    }
    RunCpuUntilPC(0x808343 as uint32, 0x85813c as uint32, game);
    (*(*game.g_snes).cpu).nmiWanted = true_0 != 0;
    RunCpuUntilPC(0x80834a as uint32, 0x858142 as uint32, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc2) as *mut uint16) = bug_fix_bak;
}
pub unsafe fn DrawFrameToPpu(game: &mut Game) {
    (*game.g_snes).vPos = 0 as uint16_t;
    (*game.g_snes).hPos = (*game.g_snes).vPos;
    while !(*(*game.g_snes).cpu).nmiWanted {
        // No CPU or APU advances between these calls. Only scanline start,
        // drawing and HDMA positions have events; the last step wraps the row.
        for position in [0, 512, 1024, 1362] {
            (*game.g_snes).hPos = position;
            snes_handle_pos_stuff(game.g_snes, game);
        }
        if (*game.g_snes).vIrqEnabled as ::core::ffi::c_int != 0
            && (*game.g_snes).vPos as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                == (*game.g_snes).vTimer as ::core::ffi::c_int
        {
            Vector_IRQ(game);
        }
    }
    (*(*game.g_snes).cpu).nmiWanted = false_0 != 0;
}
pub unsafe fn SaveBugSnapshot(game: &mut Game) {
    if game.VERIFY_ACTIVE {
        return;
    }
    if !game.g_debug_flag && game.g_got_mismatch_count == 0 as ::core::ffi::c_int {
        let mut buffer: [::core::ffi::c_char; 64] = [0; 64];
        snprintf(
            &raw mut buffer as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
            b"saves/bug-%d.sav\0" as *const u8 as *const ::core::ffi::c_char,
            time(::core::ptr::null_mut::<time_t>()) as ::core::ffi::c_int,
        );
        RtlSaveSnapshot(
            &raw mut buffer as *mut ::core::ffi::c_char,
            true_0 != 0,
            game,
        );
    }
    game.g_got_mismatch_count = 5 as ::core::ffi::c_int * 60 as ::core::ffi::c_int;
}
// Set only by the single-threaded headless runner. The interactive oracle
// retains its existing recovery and snapshot behaviour.

unsafe fn report_first_verify_failure(game: &mut Game) {
    eprintln!(
        "First failing frame: {} (zero-based)",
        game.snes_frame_counter
    );
    let mine = &*(&raw const game.g_snapshot_mine);
    let theirs = &*(&raw const game.g_snapshot_theirs);
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
pub unsafe fn RunOneFrameOfGame_Both(game: &mut Game) {
    (*game.g_snes).ppu = (*game.g_snes).snes_ppu;
    MakeSnapshot(
        &mut game.g_snapshot_before,
        &*game.g_snes,
        &*game.g_cpu,
        game.g_spc_player,
    );
    let queue_before = crate::audio_snapshot::ApuQueueSnapshot::capture(game);
    let apu_before = queue_before.pending;
    let apu_queue_before = queue_before.size;
    loop {
        queue_before.restore(game);
        game.g_snapshot_before.player.restore(game.g_spc_player);
        game.g_snapshot_theirs.apu_upload_count = 0;
        game.g_snapshot_theirs.apu_startup_upload = false;
        (*game.g_snes).runningWhichVersion = 1 as uint8_t;
        RunOneFrameOfGame_Emulated(game);
        DrawFrameToPpu(game);
        MakeSnapshot(
            &mut game.g_snapshot_theirs,
            &*game.g_snes,
            &*game.g_cpu,
            game.g_spc_player,
        );
        let queue_theirs = crate::audio_snapshot::ApuQueueSnapshot::capture(game);
        let apu_theirs = queue_theirs.pending;
        queue_before.restore(game);
        game.g_snapshot_before.player.restore(game.g_spc_player);
        game.g_snapshot_mine.apu_upload_count = 0;
        game.g_snapshot_mine.apu_startup_upload = false;
        (*game.g_snes).ppu = (*game.g_snes).my_ppu;
        RestoreSnapshot(&game.g_snapshot_before, &mut *game.g_snes, &mut *game.g_cpu);
        (*game.g_snes).runningWhichVersion = 2 as uint8_t;
        RunOneFrameOfGame(game);
        DrawFrameToPpu(game);
        let queue_mine = crate::audio_snapshot::ApuQueueSnapshot::capture(game);
        MakeSnapshot(
            &mut game.g_snapshot_mine,
            &*game.g_snes,
            &*game.g_cpu,
            game.g_spc_player,
        );
        (*game.g_snes).runningWhichVersion = 0xff as uint8_t;
        VerifySnapshotsEq(
            &raw mut game.g_snapshot_mine,
            &raw mut game.g_snapshot_theirs,
            &raw mut game.g_snapshot_before,
            game,
        );
        // Check commands before queue consumption/driver latching. The patched
        // ROM skips uploads; mine supplies the single runtime upload/reset.
        // Only this independently probed startup reset permits absent SFX
        // writes instead of zero writes. FF remains distinct everywhere else.
        let startup_reset = game.g_use_my_apu_code
            && game.g_snapshot_theirs.apu_upload_count == 0
            && game.g_snapshot_mine.apu_upload_count == 1
            && game.g_snapshot_mine.apu_startup_upload
            && apu_queue_before == 0
            && apu_before.ports == [0xff; 4]
            && game.g_apu_queue_size == 0
            && game.g_apu_write_ent_pos == 0
            && game.g_apu_time_since_empty == 0
            && game.g_snapshot_mine.ram[0x998..0x99a] == [1, 0]
            && game.g_snapshot_mine.ram[0x79b..0x79d] == [0, 0]
            && game.g_apu_write.ports == [0, 0xff, 0, 0xff]
            && apu_theirs.ports == [0; 4];
        if game.g_apu_write.ports != apu_theirs.ports && !startup_reset {
            game.g_fail = true;
            eprintln!(
                "@{}: APU queued ports compare failed: mine {:02X?} != theirs {:02X?}",
                game.snes_frame_counter, game.g_apu_write.ports, apu_theirs.ports
            );
        }
        if !game.g_use_my_apu_code {
            if let Some(field) = queue_mine.first_difference(&queue_theirs) {
                game.g_fail = true;
                eprintln!(
                    "@{}: Physical APU queue compare failed: {}",
                    game.snes_frame_counter, field
                );
            }
        }
        // Select recovery commands; successful physical comparison adopts mine below.
        queue_mine.select_runtime(queue_theirs, game);
        let failed = game.g_fail;
        if failed {
            if game.VERIFY_ACTIVE {
                game.VERIFY_FAILURES += 1;
                if game.VERIFY_FAILURES == 1 {
                    report_first_verify_failure(game);
                }
            }
            game.g_fail = false_0 != 0;
            if !game.VERIFY_ACTIVE || game.VERIFY_FAILURES == 1 {
                printf(b"Verify failure!\n\0" as *const u8 as *const ::core::ffi::c_char);
            }
        }
        // Headless recovery adopts the completed oracle frame, including the Rust
        // continuation bytes copied during comparison. Replaying the previous
        // frame would leave those bytes stale.
        if failed && !game.VERIFY_ACTIVE {
            (*game.g_snes).ppu = (*game.g_snes).snes_ppu;
            RestoreSnapshot(&game.g_snapshot_before, &mut *game.g_snes, &mut *game.g_cpu);
            if game.g_use_my_apu_code {
                game.g_snapshot_mine.player.restore(game.g_spc_player);
            } else {
                game.g_snapshot_theirs.player.restore(game.g_spc_player);
            }
            if game.g_debug_flag {
                continue;
            }
            SaveBugSnapshot(game);
            game.g_snapshot_mine.player.capture(game.g_spc_player);
            // Saving music copies the custom player's state into the physical APU.
            game.g_snapshot_before.audio.restore(game.g_snes);
            queue_before.restore(game);
            game.g_snapshot_before.player.restore(game.g_spc_player);
            RunOneFrameOfGame_Emulated(game);
            DrawFrameToPpu(game);
            if game.g_use_my_apu_code {
                game.g_snapshot_mine.player.restore(game.g_spc_player);
            }
            let queue_rerun = crate::audio_snapshot::ApuQueueSnapshot::capture(game);
            queue_mine.select_runtime(queue_rerun, game);
            break;
        } else {
            (*game.g_snes).ppu = (*game.g_snes).snes_ppu;
            RestoreSnapshot(&game.g_snapshot_theirs, &mut *game.g_snes, &mut *game.g_cpu);
            if game.g_use_my_apu_code {
                game.g_snapshot_mine.player.restore(game.g_spc_player);
            } else {
                if !failed {
                    // Continue the independently verified Rust audio branch.
                    game.g_snapshot_mine.audio.restore(game.g_snes);
                    queue_mine.restore(game);
                }
                game.g_snapshot_theirs.player.restore(game.g_spc_player);
            }
            break;
        }
    }
    (*game.g_snes).ppu = if game.g_other_image as ::core::ffi::c_int != 0 {
        (*game.g_snes).my_ppu
    } else {
        (*game.g_snes).snes_ppu
    };
    (*game.g_snes).runningWhichVersion = 0 as uint8_t;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) as ::core::ffi::c_int
        == 0xffff as ::core::ffi::c_int
    {
        (*(*game.g_snes).cpu).k = 0x80 as uint8_t;
        (*(*game.g_snes).cpu).pc = 0x8459 as uint16_t;
        *(&raw mut game.g_ram as *mut uint8).offset(0x611) = 3 as uint8;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16) as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int
        != 0
    {
        printf(b"MENU INDEX TOO BIG!\n\0" as *const u8 as *const ::core::ffi::c_char);
        SaveBugSnapshot(game);
        let ref mut fresh1 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh1 = (*fresh1 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as uint16;
    }
    if game.g_got_mismatch_count != 0 {
        game.g_got_mismatch_count -= 1;
    }
}
pub unsafe fn RtlRunFrameCompare(
    mut input: uint16,
    mut run_what: ::core::ffi::c_int,
    game: &mut Game,
) {
    (*(*game.g_snes).input1).currentState = input as uint16_t;
    if game.g_runmode as ::core::ffi::c_int == RM_THEIRS as ::core::ffi::c_int {
        RunOneFrameOfGame_Emulated(game);
        DrawFrameToPpu(game);
    } else if game.g_runmode as ::core::ffi::c_int == RM_MINE as ::core::ffi::c_int {
        (*game.g_snes).runningWhichVersion = 0xff as uint8_t;
        RunOneFrameOfGame(game);
        DrawFrameToPpu(game);
        (*game.g_snes).runningWhichVersion = 0 as uint8_t;
    } else {
        RunOneFrameOfGame_Both(game);
    };
}
