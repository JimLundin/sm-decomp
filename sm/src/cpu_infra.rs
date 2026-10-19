use crate::game::sm_80::Vector_IRQ;
use crate::game::sm_82::RunOneFrameOfGame;
use crate::game_state::Game;
use crate::ram_map::{ram_word, set_ram_word, update_ram_word};
pub use crate::snes::registers::*;
use crate::types::SpcPlayer;
use ::core::ffi::{c_char, c_void};
use ::core::mem::size_of;
use ::core::ptr::null_mut;

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

    fn free(__ptr: *mut c_void);
    fn __assert_fail(
        __assertion: *const c_char,
        __file: *const c_char,
        __line: u32,
        __function: *const c_char,
    ) -> !;
    static mut stderr: *mut FILE;
    fn fprintf(__stream: *mut FILE, __format: *const c_char, ...) -> i32;
    fn printf(__format: *const c_char, ...) -> i32;
    fn snprintf(__s: *mut c_char, __maxlen: size_t, __format: *const c_char, ...) -> i32;
    fn puts(__s: *const c_char) -> i32;
    fn memcpy(__dest: *mut c_void, __src: *const c_void, __n: size_t) -> *mut c_void;
    fn memset(__s: *mut c_void, __c: i32, __n: size_t) -> *mut c_void;
    fn memcmp(__s1: *const c_void, __s2: *const c_void, __n: size_t) -> i32;

    fn time(__timer: *mut time_t) -> time_t;
}
pub use crate::types::{
    _IO_lock_t, __off64_t, __off_t, __time_t, size_t, time_t, Apu, ApuHistory as C2RustUnnamed,
    BgLayer, Cart, Cpu, Dma, DmaChannel, Dsp, DspChannel, DspRegWriteHistory, EnemyBase,
    Enemy_Botwoon, Input, Layer, Ppu, PpuPixelPrioBufs, PpuZbufType, Snes, Spc, Timer, VoidP,
    WindowLayer, _IO_FILE, FILE,
};
pub use crate::types::{ProjectileInstr, SyncAllFunc};
// Explicit state previously included by the oversized OAM memory copy.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SpriteSnapshot {
    pub address: u8,
    pub written_address: u8,
    pub high_table: bool,
    pub written_high_table: bool,
    pub second_byte: bool,
    pub buffer: u8,
    pub priority_rotation: bool,
    pub tile_address_1: u16,
    pub tile_address_2: u16,
    pub size: u8,
    // Preserve the inherited cache coverage without depending on Ppu layout.
    pub legacy_pixel_prefix: [u8; 19],
}
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CounterSnapshot {
    pub horizontal: u16,
    pub vertical: u16,
    pub horizontal_second_byte: bool,
    pub vertical_second_byte: bool,
    pub latched: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Snapshot {
    pub audio: crate::audio_snapshot::AudioSnapshot,
    pub player: crate::audio_snapshot::PlayerSnapshot,
    pub a: u16,
    pub x: u16,
    pub y: u16,
    pub sp: u16,
    pub dp: u16,
    pub pc: u16,
    pub k: u8,
    pub db: u8,
    pub flags: u8,
    pub cpu_waiting: bool,
    pub cpu_emulation: bool,
    pub cpu_stopped: bool,
    pub cpu_irq_pending: bool,
    pub cpu_nmi_pending: bool,
    pub video_clock: (u16, u16, u32),
    pub interrupt_latches: (bool, bool, bool),
    pub interrupt_enables: (bool, bool, bool),
    pub h_timer: u16,
    pub snes_open_bus: u8,
    pub input_control: (bool, bool, u16),
    pub wram_port_address: u32,
    pub dma_channels: [crate::types::DmaChannel; 8],
    pub dma_timing: (u16, u32, bool),
    pub arithmetic: (u8, u16, u16, u16),
    pub vTimer: u16,
    pub ram: [u8; 131072],
    pub vram: [u16; 32768],
    pub sram: [u8; 8192],
    pub oam: [u16; 256],
    pub high_oam: [u8; 32],
    pub sprite: SpriteSnapshot,
    pub counters: CounterSnapshot,
    pub status_flags: [bool; 3],
    pub mode7_matrix: [i16; 8],
    pub mode7_flags: [bool; 5],
    pub mosaic: ([bool; 4], u8, u8),
    pub bgmode: (u8, bool, [bool; 4]),
    pub bg_scroll: [(u16, u16); 4],
    pub bg_address: [(bool, bool, u16, u16); 4],
    pub screen_routing: ([u8; 2], [u8; 2], [[bool; 4]; 5]),
    pub window_select: (u32, [(bool, bool, bool, bool, u8); 6]),
    pub window_bounds: [u8; 4],
    pub fixed_color: [u8; 3],
    pub color_math: (bool, bool, [bool; 6]),
    pub color_selection: (u8, u8, bool, bool),
    pub display_geometry: [bool; 6],
    pub brightness: u8,
    pub forced_blank: bool,
    pub cgram: [u16; 256],
    pub write_latches: [u8; 3],
    pub cgram_pointer: u8,
    pub cgram_second_write: bool,
    pub cgram_buffer: u8,
    pub ppu1_open_bus: u8,
    pub ppu2_open_bus: u8,
    pub vram_read_buffer: u16,
    pub vram_pointer: u16,
    pub vram_increment: u16,
    pub vram_increment_on_high: bool,
    pub vram_remap_mode: u8,
    // Per-run observations, not emulated or serialized audio state.
    pub apu_upload_count: u32,
    pub apu_startup_upload: bool,
}
pub const RM_MINE: RunMode = 1;
pub const RM_BOTH: RunMode = 0;
pub const RM_THEIRS: RunMode = 2;
pub use crate::types::RunMode;
pub use crate::types::{RunFrameFunc, SnesRegs};
pub const NULL: *mut c_void = null_mut::<c_void>();
pub const true_0: i32 = 1;
pub const false_0: i32 = 0;
#[inline]
unsafe fn get_ProjectileInstr(a: u16, game: &mut Game) -> *mut ProjectileInstr {
    RomPtr((0x930000i32 | a as i32) as u32, game) as *mut ProjectileInstr
}
#[inline]
unsafe fn Get_Botwoon(o: i32, game: &mut Game) -> *mut Enemy_Botwoon {
    (&raw mut game.g_ram as *mut u8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Botwoon
}

pub unsafe fn Call(addr: u32, game: &mut Game) {
    '_c2rust_label: {
        if addr & 0x8000u32 != 0 {
        } else {
            __assert_fail(
                b"addr & 0x8000\0" as *const u8 as *const c_char,
                b"src/sm_cpu_infra.c\0" as *const u8 as *const c_char,
                54u32,
                b"void Call(uint32)\0" as *const u8 as *const c_char,
            );
        }
    };
    RunAsmCode(addr, 0u16, 0u16, 0u16, 0i32, game);
}
pub unsafe fn SnesRomPtr(v: u32, game: &mut Game) -> *mut u8 {
    RomPtr(v as u32, game) as *mut u8
}
pub unsafe fn ProcessHook(v: u32, game: &mut Game) -> bool {
    let rombyte: *mut u8 = SnesRomPtr(v, game);
    match game.hookmode {
        0 => {
            let fresh2 = game.hookcnt;
            game.hookcnt = game.hookcnt.wrapping_add(1);
            *rombyte = game.hook_orgbyte[fresh2 as usize] as u8;
            return false_0 != 0;
        }
        1 => {
            let fresh3 = game.hookcnt;
            game.hookcnt = game.hookcnt.wrapping_add(1);
            game.hook_orgbyte[fresh3 as usize] = *rombyte as u8;
            *rombyte = 0u8;
            return false_0 != 0;
        }
        2 => {
            if v == game.hookadr {
                game.hookmode = 3u32;
                return true_0 != 0;
            }
            return false_0 != 0;
        }
        _ => {}
    }
    false_0 != 0
}
pub unsafe fn FixBugHook(addr: u32, game: &mut Game) -> bool {
    match game.hookmode {
        1 => {
            let rombyte: *mut u8 = SnesRomPtr(addr, game);
            let fresh0 = game.hookcnt;
            game.hookcnt = game.hookcnt.wrapping_add(1);
            game.hook_fixbug_orgbyte[fresh0 as usize] = *rombyte as u8;
            *rombyte = 0u8;
            return false_0 != 0;
        }
        2 => {
            if addr == game.hookadr {
                game.hookmode = 3u32;
                return true_0 != 0;
            }
            game.hookcnt = game.hookcnt.wrapping_add(1);
            return false_0 != 0;
        }
        _ => {}
    }
    false_0 != 0
}
static kPatchedCarrys: [u32; 155] = [
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

pub unsafe fn PatchBugs(mode: u32, addr: u32, game: &mut Game) -> u32 {
    game.hookmode = mode;
    game.hookadr = addr;
    game.hookcnt = 0u32;
    if FixBugHook(0x86ef35u32, game) {
        (*game.g_cpu).x = (*game.g_cpu).y;
    } else if FixBugHook(0x86ef45u32, game) {
        (*game.g_cpu).z = (*game.g_cpu).a as i32 == 0i32;
    } else if FixBugHook(0x818ab8u32, game) {
        if (*game.g_cpu).y as i32 == 0i32 {
            (*game.g_cpu).pc = 0x8b1fu16;
        }
    } else if FixBugHook(0xa794bau32, game) {
        (*game.g_cpu).y = (*game.g_cpu).x;
    } else if FixBugHook(0xa7b968u32, game) {
        (*game.g_cpu).x = ram_word!(game, cur_enemy_index) as u16;
    } else if FixBugHook(0xa7b963u32, game) {
        (*game.g_cpu).x = ram_word!(game, cur_enemy_index) as u16;
    } else if FixBugHook(0xa496c8u32, game) {
        (*game.g_cpu).a = 0u16;
    } else if FixBugHook(0x9085aau32, game) {
        (*game.g_cpu).a = ram_word!(game, speed_boost_counter) as u16;
    } else if FixBugHook(0xa29044u32, game) as i32 != 0 || FixBugHook(0xa2905du32, game) as i32 != 0
    {
        (*game.g_cpu).c = (*game.g_cpu).a as i32 == 0i32;
    } else if FixBugHook(0xa29051u32, game) {
        (*game.g_cpu).a = (*game.g_cpu).a.wrapping_sub(1);
    } else if FixBugHook(0xa5931cu32, game) {
        (*game.g_cpu).x = ram_word!(game, cur_enemy_index) as u16;
    } else if FixBugHook(0x80ada4u32, game) {
        (*game.g_cpu).a = ram_word!(game, layer2_y_pos) as u16;
    } else if FixBugHook(0x80add9u32, game) {
        (*game.g_cpu).a = ram_word!(game, layer2_y_pos) as u16;
    } else if FixBugHook(0x80ad4du32, game) {
        (*game.g_cpu).a = ram_word!(game, layer2_x_pos) as u16;
    } else if FixBugHook(0x80ad77u32, game) {
        (*game.g_cpu).a = ram_word!(game, layer2_x_pos) as u16;
    } else if FixBugHook(0x9381dbu32, game) {
        let k: i32 = (*game.g_cpu).x as i32;
        let ip: i32 = ram_word!(game, projectile_bomb_instruction_ptr[(k >> 1i32)]) as i32;
        let delta: i32 = if ram_word!(game, projectile_bomb_instruction_timers[(k >> 1i32)]) as i32
            == 1i32
            && (*get_ProjectileInstr(ip as u16, game)).timer as i32 & 0x8000i32 == 0
        {
            0i32
        } else {
            -8i32
        };
        (*game.g_cpu).a = ((*game.g_cpu).a as i32 + (8i32 + delta)) as u16;
    } else if FixBugHook(0x86b701u32, game) {
        (*game.g_cpu).x = (*game.g_cpu).y;
    } else if FixBugHook(0x8fc1b0u32, game) {
        (*game.g_cpu).x = (*game.g_cpu).a;
    } else if !FixBugHook(0x80804fu32, game) {
        if FixBugHook(0x829325u32, game) {
            (*game.g_cpu).db = 0x82u8;
        } else if FixBugHook(0x848acdu32, game) {
            (*game.g_cpu).a = 0u16;
        } else if FixBugHook(0xa7ceb2u32, game) {
            (*game.g_cpu).x = ram_word!(game, cur_enemy_index) as u16;
        } else if FixBugHook(0x91cd44u32, game) {
            if (*game.g_cpu).x as i32 == 0i32 {
                (*game.g_cpu).pc = 0xcd52u16;
            }
        } else if FixBugHook(0x82afd3u32, game) {
            if ram_word!(game, pausemenu_equipment_category_item) as u8 as i32 != 1i32 {
                return 0x82afd9u32;
            }
        } else if FixBugHook(0x82b0cdu32, game) {
            if ram_word!(game, pausemenu_equipment_category_item) as u8 as i32 != 2i32 {
                return 0x82afd9u32;
            }
        } else if FixBugHook(0x82b15bu32, game) {
            if ram_word!(game, pausemenu_equipment_category_item) as u8 as i32 != 3i32 {
                return 0x82afd9u32;
            }
        } else if FixBugHook(0xa2d38cu32, game) {
            (*game.g_cpu).x = ram_word!(game, cur_enemy_index) as u16;
        } else if FixBugHook(0xa4970fu32, game) {
            (*game.g_cpu).a = ((*game.g_cpu).a as i32 & 0xffi32) as u16;
            (*game.g_cpu).y = ((*game.g_cpu).x as i32 & 0x7i32) as u16;
        } else if FixBugHook(0xa496e0u32, game) {
            if (*game.g_cpu).x as i32 > 48i32 {
                set_ram_word!(game.croco_cur_vline_idx = (*game.g_cpu).x as u16);
                (*game.g_cpu).mf = 0i32 != 0;
                return 0xa497ceu32;
            }
        } else if FixBugHook(0x91da89u32, game) {
            if ram_word!(game, special_samus_palette_frame) as i32 > 6i32 {
                set_ram_word!(game.special_samus_palette_frame = 6u16);
            }
        } else if FixBugHook(0x828d56u32, game) {
            WriteReg(VMAIN as i32 as u16, 0x80u8, game);
        } else if FixBugHook(0x88afcfu32, game) {
            if (*game.g_cpu).a as i32 & 0x8000i32 != 0 {
                (*game.g_cpu).a = 0u16;
            }
        } else if FixBugHook(0x88aff2u32, game) {
            if ((*game.g_cpu).a as i32) < 256i32 {
                (*game.g_cpu).a = 256u16;
            }
        } else if FixBugHook(0x8189bdu32, game) {
            if (*game.g_cpu).y as i32 == 0i32 {
                return 0x818a35u32;
            }
        } else if FixBugHook(0xa29bc1u32, game) {
            (*game.g_cpu).a = 1u16;
        } else if FixBugHook(0x82e910u32, game) {
            *((&raw mut game.g_ram as *mut u8).offset(22) as *mut u8 as *mut u16) = 0u16;
        } else if FixBugHook(0x90a4c8u32, game) {
            *((&raw mut game.g_ram as *mut u8).offset(18) as *mut u8 as *mut u16) = 0u16;
        } else if FixBugHook(0xa99f60u32, game) {
            *((&raw mut game.g_ram as *mut u8).offset(22) as *mut u8 as *mut u16) = 1u16;
        } else if FixBugHook(0x94a85bu32, game) {
            memset(
                (&raw mut game.g_ram as *mut u8).offset(0xd82) as *mut c_void,
                0i32,
                8 as size_t,
            );
        } else if FixBugHook(0xa0a35cu32, game) {
            set_ram_word!(
                game.REMOVED_R18 = *((&raw mut game.g_ram as *mut u8).offset(0xcea) as *mut u16
                    as *mut u8)
                    .offset(
                        (size_of::<u16>() as usize)
                            .wrapping_div(size_of::<u8>() as usize)
                            .wrapping_sub(1_usize) as isize,
                    ) as u16
            );
            set_ram_word!(
                game.REMOVED_R20 = (ram_word!(game, REMOVED_R18) as i32
                    + (ram_word!(game, REMOVED_R18) as i32 >> 1i32)
                    >> 1i32) as u16
            );
        } else if FixBugHook(0xa7b049u32, game) {
            (*game.g_cpu).x = 0u16;
        } else if FixBugHook(0xa5a018u32, game) {
            set_ram_word!(game.REMOVED_varE24 = 0u16);
        } else if FixBugHook(0xb39ddbu32, game) {
            let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
            set_ram_word!(
                game.REMOVED_R18 = ((*E).base.x_pos as i32 - (*E).botwoon_var_56 as i32) as u16
            );
            set_ram_word!(
                game.REMOVED_R20 = ((*E).base.y_pos as i32 - (*E).botwoon_var_57 as i32) as u16
            );
        } else if FixBugHook(0xb39e13u32, game) {
            let E_0: *mut Enemy_Botwoon =
                Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
            (*E_0).botwoon_var_45 = ((*E_0).botwoon_var_45 as i32
                + (ram_word!(game, REMOVED_R22) as i32 - (*E_0).botwoon_var_45 as i32) as i8 as i32
                    * 3i32
                    / 4i32) as u8 as u16;
            set_ram_word!(game.REMOVED_R22 = (*E_0).botwoon_var_45);
        }
    }
    0u32
}
pub unsafe fn RunPatchBugHook(addr: u32, game: &mut Game) -> i32 {
    let new_pc: u32 = PatchBugs(2u32, addr, game);
    if game.hookmode == 3u32 {
        if new_pc == 0u32 {
            return game.hook_fixbug_orgbyte[game.hookcnt as usize] as i32;
        } else {
            (*game.g_cpu).k = (new_pc >> 16i32) as u8;
            (*game.g_cpu).pc = (new_pc & 0xffffu32).wrapping_add(1u32) as u16;
            return *SnesRomPtr(new_pc, game) as i32;
        }
    }
    -1i32
}
pub unsafe fn CpuOpcodeHook(addr: u32, game: &mut Game) -> i32 {
    let mut i: size_t = 0 as size_t;
    while i != (size_of::<[u32; 155]>() as usize).wrapping_div(size_of::<u32>() as usize) {
        if addr == kPatchedCarrys[i as usize] {
            return game.kPatchedCarrysOrg[i as usize] as i32;
        }
        i = i.wrapping_add(1);
    }
    let i_0: i32 = RunPatchBugHook(addr, game);
    if i_0 >= 0i32 {
        return i_0;
    }
    '_c2rust_label: {
        __assert_fail(
            b"0\0" as *const u8 as *const c_char,
            b"src/sm_cpu_infra.c\0" as *const u8 as *const c_char,
            455u32,
            b"int CpuOpcodeHook(uint32)\0" as *const u8 as *const c_char,
        );
    };
    0i32
}
pub unsafe fn HookedFunctionRts(is_long: i32, game: &mut Game) -> bool {
    if game.g_calling_asm_from_c {
        game.g_calling_asm_from_c_ret = is_long;
        game.g_calling_asm_from_c = false_0 != 0;
        return false_0 != 0;
    }
    '_c2rust_label: {
        __assert_fail(
            b"0\0" as *const u8 as *const c_char,
            b"src/sm_cpu_infra.c\0" as *const u8 as *const c_char,
            465u32,
            b"_Bool HookedFunctionRts(int)\0" as *const u8 as *const c_char,
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
    b: *mut Snapshot,
    a: *mut Snapshot,
    prev: *mut Snapshot,
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
        (&raw mut (*b).ram as *mut u8).offset(0) as *mut u8 as *mut c_void,
        (&raw mut (*a).ram as *mut u8).offset(0) as *mut u8 as *const c_void,
        0x51 as size_t,
    );
    memcpy(
        (&raw mut (*b).ram as *mut u8).offset(0x1f5b) as *mut u8 as *mut c_void,
        (&raw mut (*a).ram as *mut u8).offset(0x1f5b) as *mut u8 as *const c_void,
        (0x100i32 - 0x5bi32) as size_t,
    );
    memcpy(
        (&raw mut (*b).ram as *mut u8).offset(0xad) as *mut u8 as *mut c_void,
        (&raw mut (*a).ram as *mut u8).offset(0xad) as *mut u8 as *const c_void,
        4 as size_t,
    );
    memcpy(
        (&raw mut (*b).ram as *mut u8).offset(0x5ed) as *mut u8 as *mut c_void,
        (&raw mut (*a).ram as *mut u8).offset(0x5ed) as *mut u8 as *const c_void,
        8 as size_t,
    );
    memcpy(
        (&raw mut (*b).ram as *mut u8).offset(0x5bc) as *mut u8 as *mut c_void,
        (&raw mut (*a).ram as *mut u8).offset(0x5bc) as *mut u8 as *const c_void,
        9 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut u8).offset(0x60b) as *mut u8 as *mut c_void,
        (&raw mut (*b).ram as *mut u8).offset(0x60b) as *mut u8 as *const c_void,
        6 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut u8).offset(0x611) as *mut u8 as *mut c_void,
        (&raw mut (*b).ram as *mut u8).offset(0x611) as *mut u8 as *const c_void,
        6 as size_t,
    );
    memcpy(
        (&raw mut (*b).ram as *mut u8).offset(0x641) as *mut u8 as *mut c_void,
        (&raw mut (*a).ram as *mut u8).offset(0x641) as *mut u8 as *const c_void,
        2 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut u8).offset(0x77e) as *mut u8 as *mut c_void,
        (&raw mut (*b).ram as *mut u8).offset(0x77e) as *mut u8 as *const c_void,
        5 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut u8).offset(0x78f) as *mut u8 as *mut c_void,
        (&raw mut (*b).ram as *mut u8).offset(0x78f) as *mut u8 as *const c_void,
        2 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut u8).offset(0x7b7) as *mut u8 as *mut c_void,
        (&raw mut (*b).ram as *mut u8).offset(0x7b7) as *mut u8 as *const c_void,
        2 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut u8).offset(0x933) as *mut u8 as *mut c_void,
        (&raw mut (*b).ram as *mut u8).offset(0x933) as *mut u8 as *const c_void,
        10 as size_t,
    );
    memcpy(
        (&raw mut (*b).ram as *mut u8).offset(0xb24) as *mut u8 as *mut c_void,
        (&raw mut (*a).ram as *mut u8).offset(0xb24) as *mut u8 as *const c_void,
        4 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut u8).offset(0xd1e) as *mut u8 as *mut c_void,
        (&raw mut (*b).ram as *mut u8).offset(0xd1e) as *mut u8 as *const c_void,
        2 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut u8).offset(0xd82) as *mut u8 as *mut c_void,
        (&raw mut (*b).ram as *mut u8).offset(0xd82) as *mut u8 as *const c_void,
        8 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut u8).offset(0xd9c) as *mut u8 as *mut c_void,
        (&raw mut (*b).ram as *mut u8).offset(0xd9c) as *mut u8 as *const c_void,
        2 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut u8).offset(0xdd2) as *mut u8 as *mut c_void,
        (&raw mut (*b).ram as *mut u8).offset(0xdd2) as *mut u8 as *const c_void,
        6 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut u8).offset(0xd8a) as *mut u8 as *mut c_void,
        (&raw mut (*b).ram as *mut u8).offset(0xd8a) as *mut u8 as *const c_void,
        6 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut u8).offset(0xe20) as *mut u8 as *mut c_void,
        (&raw mut (*b).ram as *mut u8).offset(0xe20) as *mut u8 as *const c_void,
        (0xe46i32 - 0xe20i32) as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut u8).offset(0xe54) as *mut u8 as *mut c_void,
        (&raw mut (*b).ram as *mut u8).offset(0xe54) as *mut u8 as *const c_void,
        2 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut u8).offset(0xe02) as *mut u8 as *mut c_void,
        (&raw mut (*b).ram as *mut u8).offset(0xe02) as *mut u8 as *const c_void,
        2 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut u8).offset(0xe4a) as *mut u8 as *mut c_void,
        (&raw mut (*b).ram as *mut u8).offset(0xe4a) as *mut u8 as *const c_void,
        2 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut u8).offset(0xe56) as *mut u8 as *mut c_void,
        (&raw mut (*b).ram as *mut u8).offset(0xe56) as *mut u8 as *const c_void,
        4 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut u8).offset(0x1784) as *mut u8 as *mut c_void,
        (&raw mut (*b).ram as *mut u8).offset(0x1784) as *mut u8 as *const c_void,
        8 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut u8).offset(0x1790) as *mut u8 as *mut c_void,
        (&raw mut (*b).ram as *mut u8).offset(0x1790) as *mut u8 as *const c_void,
        4 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut u8).offset(0x17a8) as *mut u8 as *mut c_void,
        (&raw mut (*b).ram as *mut u8).offset(0x17a8) as *mut u8 as *const c_void,
        4 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut u8).offset(0x1834) as *mut u8 as *mut c_void,
        (&raw mut (*b).ram as *mut u8).offset(0x1834) as *mut u8 as *const c_void,
        8 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut u8).offset(0x184a) as *mut u8 as *mut c_void,
        (&raw mut (*b).ram as *mut u8).offset(0x184a) as *mut u8 as *const c_void,
        18 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut u8).offset(0x186e) as *mut u8 as *mut c_void,
        (&raw mut (*b).ram as *mut u8).offset(0x186e) as *mut u8 as *const c_void,
        (16i32 + 8i32) as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut u8).offset(0x18a6) as *mut u8 as *mut c_void,
        (&raw mut (*b).ram as *mut u8).offset(0x18a6) as *mut u8 as *const c_void,
        2 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut u8).offset(0x189a) as *mut u8 as *mut c_void,
        (&raw mut (*b).ram as *mut u8).offset(0x189a) as *mut u8 as *const c_void,
        12 as size_t,
    );
    memcpy(
        (&raw mut (*b).ram as *mut u8).offset(0x1966) as *mut u8 as *mut c_void,
        (&raw mut (*a).ram as *mut u8).offset(0x1966) as *mut u8 as *const c_void,
        6 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut u8).offset(0xef74) as *mut u8 as *mut c_void,
        (&raw mut (*b).ram as *mut u8).offset(0xef74) as *mut u8 as *const c_void,
        4 as size_t,
    );
    memcpy(
        (&raw mut (*a).ram as *mut u8).offset(0xf37a) as *mut u8 as *mut c_void,
        (&raw mut (*b).ram as *mut u8).offset(0xf37a) as *mut u8 as *const c_void,
        6 as size_t,
    );
    if memcmp(
        &raw mut (*b).ram as *mut u8 as *const c_void,
        &raw mut (*a).ram as *mut u8 as *const c_void,
        0x20000i32 as size_t,
    ) != 0
    {
        game.g_fail = true_0 != 0;
        fprintf(
            stderr,
            b"@%d: Memory compare failed (mine != theirs, prev):\n\0" as *const u8 as *const c_char,
            game.snes_frame_counter,
        );
        let mut j: i32 = 0i32;
        let mut i: size_t = 0 as size_t;
        while i < 0x20000i32 as size_t {
            if (*a).ram[i as usize] as i32 != (*b).ram[i as usize] as i32 {
                j += 1;
                if j < 256i32 {
                    if (i & 1 as size_t == 0 as size_t || i < 0x10000i32 as size_t)
                        && (*a).ram[i.wrapping_add(1 as size_t) as usize] as i32
                            != (*b).ram[i.wrapping_add(1 as size_t) as usize] as i32
                    {
                        let read_word = |ram: &[u8; 0x20000]| {
                            i32::from(u16::from_le_bytes([ram[i as usize], ram[i as usize + 1]]))
                        };
                        fprintf(
                            stderr,
                            b"0x%.6X: %.4X != %.4X (%.4X)\n\0" as *const u8 as *const c_char,
                            i as i32,
                            read_word(&(*b).ram),
                            read_word(&(*a).ram),
                            read_word(&(*prev).ram),
                        );
                        i = i.wrapping_add(1);
                        j += 1;
                    } else {
                        fprintf(
                            stderr,
                            b"0x%.6X: %.2X != %.2X (%.2X)\n\0" as *const u8 as *const c_char,
                            i as i32,
                            (*b).ram[i as usize] as i32,
                            (*a).ram[i as usize] as i32,
                            (*prev).ram[i as usize] as i32,
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
            b"  total of %d failed bytes\n\0" as *const u8 as *const c_char,
            j,
        );
    }
    if memcmp(
        &raw mut (*b).sram as *mut u8 as *const c_void,
        &raw mut (*a).sram as *mut u8 as *const c_void,
        0x2000 as size_t,
    ) != 0
    {
        game.g_fail = true_0 != 0;
        fprintf(
            stderr,
            b"@%d: SRAM compare failed (mine != theirs, prev):\n\0" as *const u8 as *const c_char,
            game.snes_frame_counter,
        );
        let mut j_0: i32 = 0i32;
        let mut i_0: size_t = 0 as size_t;
        while i_0 < 0x2000 as size_t {
            if (*a).sram[i_0 as usize] as i32 != (*b).sram[i_0 as usize] as i32 {
                j_0 += 1;
                if j_0 < 128i32 {
                    if i_0 & 1 as size_t == 0 as size_t
                        && (*a).sram[i_0.wrapping_add(1 as size_t) as usize] as i32
                            != (*b).sram[i_0.wrapping_add(1 as size_t) as usize] as i32
                    {
                        fprintf(
                            stderr,
                            b"0x%.6X: %.4X != %.4X (%.4X)\n\0" as *const u8 as *const c_char,
                            i_0 as i32,
                            *((&raw mut (*b).sram as *mut u8).offset(i_0 as isize) as *mut u8
                                as *mut u16) as i32,
                            *((&raw mut (*a).sram as *mut u8).offset(i_0 as isize) as *mut u8
                                as *mut u16) as i32,
                            *((&raw mut (*prev).sram as *mut u8).offset(i_0 as isize) as *mut u8
                                as *mut u16) as i32,
                        );
                        i_0 = i_0.wrapping_add(1);
                        j_0 += 1;
                    } else {
                        fprintf(
                            stderr,
                            b"0x%.6X: %.2X != %.2X (%.2X)\n\0" as *const u8 as *const c_char,
                            i_0 as i32,
                            (*b).sram[i_0 as usize] as i32,
                            (*a).sram[i_0 as usize] as i32,
                            (*prev).sram[i_0 as usize] as i32,
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
            b"  total of %d failed bytes\n\0" as *const u8 as *const c_char,
            j_0,
        );
    }
    if memcmp(
        &raw mut (*b).vram as *mut u16 as *const c_void,
        &raw mut (*a).vram as *mut u16 as *const c_void,
        (size_of::<u16>() as size_t).wrapping_mul(0x8000 as size_t),
    ) != 0
    {
        game.g_fail = true_0 != 0;
        fprintf(
            stderr,
            b"@%d: VRAM compare failed (mine != theirs, prev):\n\0" as *const u8 as *const c_char,
            game.snes_frame_counter,
        );
        let mut i_1: size_t = 0 as size_t;
        let mut j_1: size_t = 0 as size_t;
        while i_1 < 0x8000 as size_t {
            if (*a).vram[i_1 as usize] as i32 != (*b).vram[i_1 as usize] as i32 {
                fprintf(
                    stderr,
                    b"0x%.6X: %.4X != %.4X (%.4X)\n\0" as *const u8 as *const c_char,
                    i_1 as i32,
                    (*b).vram[i_1 as usize] as i32,
                    (*a).vram[i_1 as usize] as i32,
                    (*prev).vram[i_1 as usize] as i32,
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
    (*s).a = (*c).a as u16;
    (*s).x = (*c).x as u16;
    (*s).y = (*c).y as u16;
    (*s).sp = (*c).sp as u16;
    (*s).dp = (*c).dp as u16;
    (*s).db = (*c).db as u8;
    (*s).pc = (*c).pc as u16;
    (*s).k = (*c).k as u8;
    (*s).flags = cpu_getFlags(&*c) as u8;
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
        &raw mut (*s).ram as *mut u8 as *mut c_void,
        (*snes).ram as *const c_void,
        0x20000i32 as size_t,
    );
    memcpy(
        &raw mut (*s).sram as *mut u8 as *mut c_void,
        (*(*snes).cart).ram as *const c_void,
        (*(*snes).cart).ramSize as size_t,
    );
    memcpy(
        &raw mut (*s).vram as *mut u16 as *mut c_void,
        &raw mut (*(*snes).ppu).vram as *mut u16 as *const c_void,
        (size_of::<u16>() as size_t).wrapping_mul(0x8000 as size_t),
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
    (*c).a = (*s).a as u16;
    (*c).x = (*s).x as u16;
    (*c).y = (*s).y as u16;
    (*c).sp = (*s).sp as u16;
    (*c).dp = (*s).dp as u16;
    (*c).db = (*s).db as u8;
    (*c).pc = (*s).pc as u16;
    (*c).k = (*s).k as u8;
    (*snes).vTimer = (*s).vTimer;
    (*c).e = (*s).cpu_emulation;
    (*c).stopped = (*s).cpu_stopped;
    (*c).irqWanted = (*s).cpu_irq_pending;
    (*c).nmiWanted = (*s).cpu_nmi_pending;
    cpu_setFlags((*s).flags as u8, &mut *c);
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
        (*snes).ram as *mut c_void,
        &raw const (*s).ram as *const u8 as *const c_void,
        0x20000i32 as size_t,
    );
    memcpy(
        (*(*snes).cart).ram as *mut c_void,
        &raw const (*s).sram as *const u8 as *const c_void,
        0x2000 as size_t,
    );
    memcpy(
        &raw mut (*(*snes).ppu).vram as *mut u16 as *mut c_void,
        &raw const (*s).vram as *const u16 as *const c_void,
        (size_of::<u16>() as size_t).wrapping_mul(0x8000 as size_t),
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
pub unsafe fn RunAsmCode(pc: u32, a: u16, x: u16, y: u16, flags: i32, game: &mut Game) -> i32 {
    let org_sp: u16 = (*game.g_cpu).sp as u16;
    let org_pc: u16 = (*game.g_cpu).pc as u16;
    let org_b: u8 = (*game.g_cpu).db as u8;
    let org_dp: u16 = (*game.g_cpu).dp as u16;
    printf(b"RunAsmCode!\n\0" as *const u8 as *const c_char);
    game.g_ram[0x1ffffi32 as usize] = 1u8;
    let dc: bool = (*game.g_snes).debug_cycles;
    (*game.g_cpu).db = (pc >> 16i32) as u8;
    (*game.g_cpu).a = a as u16;
    (*game.g_cpu).x = x as u16;
    (*game.g_cpu).y = y as u16;
    (*game.g_cpu).spBreakpoint = (*game.g_cpu).sp;
    (*game.g_cpu).k = (pc >> 16i32) as u8;
    (*game.g_cpu).pc = (pc & 0xffffu32) as u16;
    (*game.g_cpu).mf = flags & 1i32 != 0;
    (*game.g_cpu).xf = (flags & 2i32) >> 1i32 != 0;
    game.g_calling_asm_from_c = true_0 != 0;
    while game.g_calling_asm_from_c {
        let pc_0: u32 =
            (((*(*game.g_snes).cpu).k as i32) << 16i32 | (*(*game.g_snes).cpu).pc as i32) as u32;
        if (*game.g_snes).debug_cycles {
            let mut line: [c_char; 80] = [0; 80];
            getProcessorStateCpu(game.g_snes, &raw mut line as *mut c_char, game);
            puts(&raw mut line as *mut c_char);
            line[0i32 as usize] = 0 as c_char;
        }
        cpu_runOpcode(game.g_cpu, game);
        while (*(*game.g_snes).dma).dmaBusy {
            dma_doDma((*game.g_snes).dma, game);
        }
        if flags & 1i32 != 0 {
            let mut i: i32 = 0i32;
            while i < 10i32 {
                apu_cycle((*game.g_snes).apu, game);
                i += 1;
            }
        }
    }
    (*game.g_cpu).dp = org_dp as u16;
    (*game.g_cpu).sp = org_sp as u16;
    (*game.g_cpu).db = org_b as u8;
    (*game.g_cpu).pc = org_pc as u16;
    (*game.g_snes).debug_cycles = dc;
    game.g_calling_asm_from_c_ret
}
unsafe fn loadRom(name: *const c_char, snes: *mut Snes, game: &mut Game) -> bool {
    let mut length: size_t = 0 as size_t;
    let mut file: *mut u8 = null_mut::<u8>();
    file = ReadWholeFile(name, &raw mut length) as *mut u8;
    if file.is_null() {
        puts(b"Failed to read file\0" as *const u8 as *const c_char);
        return false_0 != 0;
    }
    let result: bool = snes_loadRom(snes as *mut Snes, file, length as i32, game);
    free(file as *mut c_void);
    result
}
pub unsafe fn PatchBytes(addr: u32, bytes: &[u8], game: &mut Game) {
    // Resolve the original base for every byte; RomPtr can update g_fail.
    for (i, &byte) in bytes.iter().enumerate() {
        *SnesRomPtr(addr, game).offset(i as isize) = byte;
    }
}
pub unsafe fn FixupCarry(addr: u32, game: &mut Game) {
    *SnesRomPtr(addr, game) = 0u8;
}

pub unsafe fn RtlUpdateSnesPatchForBugfix(game: &mut Game) {
    game.currently_installed_bug_fix_counter = ram_word!(game, bug_fix_counter);
    PatchBytes(0x8584a3u32, &[0x20, 0x50, 0x96, 0x60], game);
    PatchBytes(
        0x859650u32,
        &[
            0x20,
            0x36,
            0x81,
            0x22,
            0x59,
            0x94,
            0x80,
            0xc2,
            0x30,
            0xa5,
            (if (ram_word!(game, bug_fix_counter) as i32) < 1i32 {
                0x8fi32
            } else {
                0x8bi32
            }) as u8,
            0xf0,
            0xf3,
            0x60,
        ],
        game,
    );
    PatchBytes(0x8584ccu32, &[0x18, 0x18], game);
}
pub unsafe fn SnesInit(filename: *const c_char, game: &mut Game) -> *mut Snes {
    game.g_snes = snes_init(&raw mut game.g_ram as *mut u8) as *mut Snes;
    game.g_cpu = (*game.g_snes).cpu;
    let loaded: bool = loadRom(filename, game.g_snes, game);
    if !loaded {
        return null_mut::<Snes>();
    }
    game.g_sram = (*(*game.g_snes).cart).ram as *mut u8;
    game.g_rom = (*(*game.g_snes).cart).rom;
    game.register_rom(game.g_rom, (*(*game.g_snes).cart).romSize as usize);
    RtlSetupEmuCallbacks(
        null_mut::<u8>(),
        Some(RtlRunFrameCompare as unsafe fn(u16, i32, &mut Game) -> ()),
        None,
        game,
    );
    *(&raw mut game.g_ram as *mut u8).offset(0x611) = 1u8;
    PatchBytes(0x82896bu32, &[0x20, 0xf, 0xf7], game);
    PatchBytes(0x82f70fu32, &[0x7c, 0x81, 0x89], game);
    PatchBytes(0x8ba362u32, &[0x5f, 0xf7], game);
    PatchBytes(0x8bf760u32, &[0xc2, 0x20, 0x4c, 0x67, 0xa3], game);
    PatchBytes(0x8584b2u32, &[0xa, 0xa], game);
    PatchBytes(
        0x84efd3u32,
        &[0xc0, 0, 0, 0xf0, 0x3, 0x20, 0x64, 0x87, 0x60],
        game,
    );
    PatchBytes(0x848243u32, &[0xd3, 0xef], game);
    PatchBytes(0x86f4a6u32, &[0xac, 0x91, 0x19, 0x4c, 0x76, 0x86], game);
    PatchBytes(0x8685bdu32, &[0xa6, 0xf4], game);
    PatchBytes(
        0x82f713u32,
        &[
            0xc2, 0x30, 0x22, 0x59, 0x94, 0x80, 0x20, 0x48, 0x89, 0x22, 0x38, 0x83, 0x80, 0x4c,
            0x13, 0xf7,
        ],
        game,
    );
    PatchBytes(0x828944u32, &[0x58, 0x4c, 0x13, 0xf7], game);
    PatchBytes(0x82897au32, &[0x28, 0x60], game);
    PatchBytes(
        0x80945cu32,
        &[0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18],
        game,
    );
    PatchBytes(
        0xa8b237u32,
        &[0xbc, 0xaa, 0xf, 0xc9, 0x6c, 0, 0x10, 0x1a],
        game,
    );
    PatchBytes(0xa2904bu32, &[0x49, 0xff, 0xff, 0x69, 0, 0], game);
    PatchBytes(0xa29065u32, &[0x49, 0xff, 0xff, 0x69, 0, 0], game);
    PatchBytes(0xa0896fu32, &[0x6b], game);
    PatchBytes(0xa98c12u32, &[0x18, 0x18, 0x18], game);
    PatchBytes(0x8085f6u32, &[0x60], game);
    PatchBytes(
        0x80843cu32,
        &[0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18],
        game,
    );
    PatchBytes(
        0x808475u32,
        &[0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18],
        game,
    );
    PatchBytes(
        0x808525u32,
        &[0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18],
        game,
    );
    PatchBytes(0x8882a1u32, &[0x18, 0x18, 0x18, 0x18], game);
    PatchBytes(0x828533u32, &[0x18, 0x18, 0x18, 0x18], game);
    PatchBytes(0x80837bu32, &[0x18, 0x18, 0x18, 0x18], game);
    PatchBytes(0x80838eu32, &[0x18, 0x18, 0x18, 0x18], game);
    PatchBytes(0x82df6cu32, &[0x18, 0x18, 0x18, 0x18], game);
    PatchBytes(0x80d000u32, &[0x20, 0x32, 0x96, 0x6b], game);
    PatchBytes(0x82e02cu32, &[0x22, 0, 0xd0, 0x80, 0x18], game);
    PatchBytes(0x82e06bu32, &[0x22, 0, 0xd0, 0x80, 0x18], game);
    PatchBytes(0x82e50du32, &[0x22, 0, 0xd0, 0x80, 0x18], game);
    PatchBytes(0x82e609u32, &[0x22, 0, 0xd0, 0x80, 0x18], game);
    PatchBytes(0x82e526u32, &[0x22, 0x4, 0xd0, 0x80, 0x18], game);
    PatchBytes(
        0x80d004u32,
        &[0x22, 0x38, 0x83, 0x80, 0xad, 0x31, 0x9, 0x10, 0xf7, 0x6b],
        game,
    );
    PatchBytes(0x82e540u32, &[0x18, 0x18, 0x18, 0x18], game);
    PatchBytes(0x8bc11eu32, &[0x18, 0x18, 0x18, 0x18], game);
    PatchBytes(0x8bd487u32, &[0x18, 0x18, 0x18, 0x18], game);
    PatchBytes(0x858148u32, &[0x18, 0x18, 0x18], game);
    PatchBytes(0x8581b2u32, &[0x18, 0x18, 0x18], game);
    PatchBytes(
        0x8581eau32,
        &[0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18],
        game,
    );
    PatchBytes(0x858203u32, &[0x18, 0x18, 0x18], game);
    PatchBytes(
        0x858236u32,
        &[0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18],
        game,
    );
    PatchBytes(
        0x8582b8u32,
        &[
            0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
        ],
        game,
    );
    PatchBytes(0x858321u32, &[0x18, 0x18, 0x18], game);
    PatchBytes(
        0x85835au32,
        &[0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18],
        game,
    );
    PatchBytes(0x858532u32, &[0x18, 0x18, 0x18], game);
    PatchBytes(
        0x85856bu32,
        &[0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18],
        game,
    );
    PatchBytes(0x858096u32, &[0x18, 0x18, 0x18], game);
    PatchBytes(0x8580b4u32, &[0x18, 0x18, 0x18], game);
    PatchBytes(0x8580dcu32, &[0x18, 0x18, 0x18], game);
    PatchBytes(0x8580f2u32, &[0x18, 0x18, 0x18], game);
    PatchBytes(0x85861cu32, &[0x18, 0x18, 0x18], game);
    PatchBytes(0x858651u32, &[0x18, 0x18, 0x18], game);
    PatchBytes(0x858692u32, &[0x18, 0x18, 0x18, 0x18], game);
    PatchBytes(0x858696u32, &[0x18, 0x18, 0x18, 0x18], game);
    PatchBytes(0x859660u32, &[0x8, 0xc2, 0x30, 0x4c, 0xa9, 0x85], game);
    PatchBytes(0x8583bau32, &[0x20, 0x60, 0x96], game);
    PatchBytes(
        0x859670u32,
        &[
            0x22, 0x81, 0x96, 0x85, 0xc9, 0xff, 0xff, 0xd0, 0x4, 0x5c, 0x5f, 0xab, 0xa2, 0x5c,
            0x26, 0xab, 0xa2,
        ],
        game,
    );
    PatchBytes(
        0x859681u32,
        &[
            0xcd, 0x1f, 0x1c, 0xd0, 0x8, 0x9c, 0x1f, 0x1c, 0xad, 0xf9, 0x5, 0x6b, 0xff, 0x8d, 0xc8,
            0xd, 0xa9, 0xff, 0xff, 0x6b,
        ],
        game,
    );
    PatchBytes(0xa2ab22u32, &[0x5c, 0x70, 0x96, 0x85], game);
    PatchBytes(
        0x8596a0u32,
        &[
            0x22, 0xd4, 0x8f, 0xa0, 0xad, 0xc8, 0xd, 0xf0, 0x7, 0x22, 0x95, 0x96, 0x85, 0x9c, 0xc8,
            0xd, 0x6b,
        ],
        game,
    );
    PatchBytes(0x828b65u32, &[0x22, 0xa0, 0x96, 0x85], game);
    PatchBytes(
        0x8596c0u32,
        &[0x20, 0x89, 0x85, 0xa9, 0x1c, 0, 0x8d, 0x1f, 0x1c, 0x60],
        game,
    );
    PatchBytes(0x8580e5u32, &[0x20, 0xc0, 0x96], game);
    PatchBytes(
        0x84efdcu32,
        &[
            0xad, 0xc8, 0xd, 0xf0, 0x11, 0x98, 0x9d, 0x27, 0x1d, 0xad, 0xc8, 0xd, 0x22, 0x95, 0x96,
            0x85, 0x9c, 0xc8, 0xd, 0xbc, 0x27, 0x1d, 0x4c, 0xee, 0x85,
        ],
        game,
    );
    PatchBytes(0x8485f7u32, &[0xf4, 0xdb, 0xef], game);
    PatchBytes(
        0x859695u32,
        &[0x8, 0x8b, 0xda, 0x5a, 0x5c, 0x84, 0x80, 0x85],
        game,
    );
    PatchBytes(0x858080u32, &[0x8d, 0xc8, 0xd, 0x6b], game);
    PatchBytes(
        0x84f000u32,
        &[
            0x22, 0x81, 0x96, 0x85, 0xc9, 0xff, 0xff, 0xf0, 0x4, 0x5c, 0xfa, 0x8c, 0x84, 0x7a,
            0xfa, 0x88, 0x88, 0x60,
        ],
        game,
    );
    PatchBytes(0x848cf6u32, &[0x5c, 0, 0xf0, 0x84], game);
    PatchBytes(
        0x81f000u32,
        &[0xa9, 0xff, 0xff, 0x8d, 0x98, 0x9, 0x60],
        game,
    );
    PatchBytes(0x819027u32, &[0x5c, 0, 0xf0, 0x81], game);
    PatchBytes(0x819112u32, &[0x5c, 0, 0xf0, 0x81], game);
    PatchBytes(0x8194e9u32, &[0x5c, 0, 0xf0, 0x81], game);
    PatchBytes(0x8095e1u32, &[0x18, 0x18, 0x18, 0x18], game);
    if game.g_use_my_apu_code {
        PatchBytes(0x808028u32, &[0x60], game);
    }
    PatchBytes(0x828a59u32, &[0x18, 0x18, 0x18, 0x80], game);
    PatchBytes(0x828a72u32, &[0x18, 0x18, 0x18], game);
    PatchBytes(0x828a80u32, &[0x18, 0x18, 0x18, 0x80], game);
    PatchBytes(0x828a67u32, &[0x6], game);
    PatchBytes(0x82831eu32, &[0, 0x2e], game);
    PatchBytes(0x91c234u32, &[0xa5, 0x25], game);
    PatchBytes(0x809af3u32, &[0x18, 0x18, 0x18, 0x18], game);
    PatchBytes(
        0xa7ff82u32,
        &[
            0xa5, 0x12, 0x48, 0xa5, 0x14, 0x48, 0xa5, 0x16, 0x48, 0x22, 0xa7, 0xa6, 0xa0, 0x68,
            0x85, 0x16, 0x68, 0x85, 0x14, 0x68, 0x85, 0x12, 0x6b,
        ],
        game,
    );
    PatchBytes(0xa7b03au32, &[0x22, 0x82, 0xff, 0xa7], game);
    RtlUpdateSnesPatchForBugfix(game);
    let mut i: size_t = 0 as size_t;
    while i != (size_of::<[u32; 155]>() as usize).wrapping_div(size_of::<u32>() as usize) {
        let t_84: u8 = *SnesRomPtr(kPatchedCarrys[i as usize], game);
        if t_84 != 0 {
            game.kPatchedCarrysOrg[i as usize] = t_84;
            FixupCarry(kPatchedCarrys[i as usize], game);
        } else {
            printf(
                b"0x%x double patched!\n\0" as *const u8 as *const c_char,
                kPatchedCarrys[i as usize],
            );
        }
        i = i.wrapping_add(1);
    }
    PatchBugs(1u32, 0u32, game);
    game.g_snes
}
pub unsafe fn DebugGameOverMenu() {
    '_c2rust_label: {
        __assert_fail(
            b"0\0" as *const u8 as *const c_char,
            b"src/sm_cpu_infra.c\0" as *const u8 as *const c_char,
            914u32,
            b"void DebugGameOverMenu(void)\0" as *const u8 as *const c_char,
        );
    };
}
pub unsafe fn RunCpuUntilPC(pc1: u32, pc2: u32, game: &mut Game) -> u32 {
    loop {
        if (*game.g_snes).debug_cycles {
            let mut line: [c_char; 80] = [0; 80];
            getProcessorStateCpu(game.g_snes, &raw mut line as *mut c_char, game);
            puts(&raw mut line as *mut c_char);
        }
        cpu_runOpcode(game.g_cpu, game);
        let addr: u32 =
            (((*(*game.g_snes).cpu).k as i32) << 16i32 | (*(*game.g_snes).cpu).pc as i32) as u32;
        if addr == pc1 || addr == pc2 {
            return addr;
        }
    }
}
pub unsafe fn RunOneFrameOfGame_Emulated(game: &mut Game) {
    let bug_fix_bak: u16 = ram_word!(game, bug_fix_counter);
    let cpu = (*game.g_snes).cpu;
    if (*cpu).k == 0x80 && (*cpu).pc == 0x8459 {
        RunCpuUntilPC(0x808460, 0x808460, game);
        (*cpu).pc = 0x8462;
    }
    RunCpuUntilPC(0x808343u32, 0x85813cu32, game);
    (*(*game.g_snes).cpu).nmiWanted = true_0 != 0;
    RunCpuUntilPC(0x80834au32, 0x858142u32, game);
    set_ram_word!(game.bug_fix_counter = bug_fix_bak);
}
pub unsafe fn DrawFrameToPpu(game: &mut Game) {
    (*game.g_snes).vPos = 0u16;
    (*game.g_snes).hPos = (*game.g_snes).vPos;
    while !(*(*game.g_snes).cpu).nmiWanted {
        // No CPU or APU advances between these calls. Only scanline start,
        // drawing and HDMA positions have events; the last step wraps the row.
        for position in [0, 512, 1024, 1362] {
            (*game.g_snes).hPos = position;
            snes_handle_pos_stuff(game.g_snes, game);
        }
        if (*game.g_snes).vIrqEnabled as i32 != 0
            && (*game.g_snes).vPos as i32 - 1i32 == (*game.g_snes).vTimer as i32
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
    if !game.g_debug_flag && game.g_got_mismatch_count == 0i32 {
        let mut buffer: [c_char; 64] = [0; 64];
        snprintf(
            &raw mut buffer as *mut c_char,
            size_of::<[c_char; 64]>() as size_t,
            b"saves/bug-%d.sav\0" as *const u8 as *const c_char,
            time(null_mut::<time_t>()) as i32,
        );
        RtlSaveSnapshot(&raw mut buffer as *mut c_char, true_0 != 0, game);
    }
    game.g_got_mismatch_count = 5i32 * 60i32;
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
        (*game.g_snes).runningWhichVersion = 1u8;
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
        (*game.g_snes).runningWhichVersion = 2u8;
        RunOneFrameOfGame(game);
        DrawFrameToPpu(game);
        let queue_mine = crate::audio_snapshot::ApuQueueSnapshot::capture(game);
        MakeSnapshot(
            &mut game.g_snapshot_mine,
            &*game.g_snes,
            &*game.g_cpu,
            game.g_spc_player,
        );
        (*game.g_snes).runningWhichVersion = 0xffu8;
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
                printf(b"Verify failure!\n\0" as *const u8 as *const c_char);
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
    (*game.g_snes).ppu = if game.g_other_image as i32 != 0 {
        (*game.g_snes).my_ppu
    } else {
        (*game.g_snes).snes_ppu
    };
    (*game.g_snes).runningWhichVersion = 0u8;
    if ram_word!(game, game_state) as i32 == 0xffffi32 {
        (*(*game.g_snes).cpu).k = 0x80u8;
        (*(*game.g_snes).cpu).pc = 0x8459u16;
        *(&raw mut game.g_ram as *mut u8).offset(0x611) = 3u8;
    }
    if ram_word!(game, menu_index) as i32 & 0xff00i32 != 0 {
        printf(b"MENU INDEX TOO BIG!\n\0" as *const u8 as *const c_char);
        SaveBugSnapshot(game);
        update_ram_word!(game.menu_index, |fresh1| (fresh1 as i32 & 0xffi32) as u16);
    }
    if game.g_got_mismatch_count != 0 {
        game.g_got_mismatch_count -= 1;
    }
}
pub unsafe fn RtlRunFrameCompare(input: u16, run_what: i32, game: &mut Game) {
    (*(*game.g_snes).input1).currentState = input as u16;
    if game.g_runmode as i32 == RM_THEIRS as i32 {
        RunOneFrameOfGame_Emulated(game);
        DrawFrameToPpu(game);
    } else if game.g_runmode as i32 == RM_MINE as i32 {
        (*game.g_snes).runningWhichVersion = 0xffu8;
        RunOneFrameOfGame(game);
        DrawFrameToPpu(game);
        (*game.g_snes).runningWhichVersion = 0u8;
    } else {
        RunOneFrameOfGame_Both(game);
    };
}
