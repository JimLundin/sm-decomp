use crate::game::sm_80::Vector_IRQ;
use crate::game::sm_82::RunOneFrameOfGame;
use crate::game_state::Game;
use crate::ram_map::ram_ptr;
use crate::ram_map::word;
use crate::rtl::words;
pub use crate::snes::registers::*;
use crate::types::SpcPlayer;
use ::core::ffi::{c_char, c_void};
use ::core::mem::size_of;
use ::core::ptr::null_mut;

use crate::c_library::{
    __assert_fail, fprintf, free, memcmp, memcpy, memset, printf, puts, snprintf, stderr, time,
};
use crate::rtl::{RomPtr, RtlSaveSnapshot, RtlSetupEmuCallbacks, WriteReg};
use crate::snes::apu::apu_cycle;
use crate::snes::cpu::{cpu_getFlags, cpu_runOpcode, cpu_setFlags};
use crate::snes::dma::dma_doDma;
use crate::snes::snes::{snes_handle_pos_stuff, snes_init};
use crate::snes::snes_other::snes_loadRom;
use crate::tracing::getProcessorStateCpu;
pub use crate::types::{_IO_codecvt, _IO_marker, _IO_wide_data};
pub use crate::types::{
    _IO_lock_t, __off64_t, __off_t, __time_t, size_t, time_t, Apu, ApuHistory as C2RustUnnamed,
    BgLayer, Cart, Cpu, Dma, DmaChannel, Dsp, DspChannel, DspRegWriteHistory, EnemyBase,
    Enemy_Botwoon, Input, Layer, Ppu, PpuPixelPrioBufs, PpuZbufType, Snes, Spc, Timer, VoidP,
    WindowLayer, _IO_FILE, FILE,
};
pub use crate::types::{ProjectileInstr, SyncAllFunc};
use crate::util::ReadWholeFile;
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
crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C)]
    pub struct Snapshot {
        pub audio: crate::audio_snapshot::AudioSnapshot;
        pub player: crate::audio_snapshot::PlayerSnapshot;
        pub a, x, y, sp, dp, pc: u16;
        pub k, db, flags: u8;
        pub cpu_waiting, cpu_emulation, cpu_stopped, cpu_irq_pending, cpu_nmi_pending: bool;
        pub video_clock: (u16, u16, u32);
        pub interrupt_latches, interrupt_enables: (bool, bool, bool);
        pub h_timer: u16;
        pub snes_open_bus: u8;
        pub input_control: (bool, bool, u16);
        pub wram_port_address: u32;
        pub dma_channels: [crate::types::DmaChannel; 8];
        pub dma_timing: (u16, u32, bool);
        pub arithmetic: (u8, u16, u16, u16);
        pub vTimer: u16;
        pub ram: [u8; 131072];
        pub vram: [u16; 32768];
        pub sram: [u8; 8192];
        pub oam: [u16; 256];
        pub high_oam: [u8; 32];
        pub sprite: SpriteSnapshot;
        pub counters: CounterSnapshot;
        pub status_flags: [bool; 3];
        pub mode7_matrix: [i16; 8];
        pub mode7_flags: [bool; 5];
        pub mosaic: ([bool; 4], u8, u8);
        pub bgmode: (u8, bool, [bool; 4]);
        pub bg_scroll: [(u16, u16); 4];
        pub bg_address: [(bool, bool, u16, u16); 4];
        pub screen_routing: ([u8; 2], [u8; 2], [[bool; 4]; 5]);
        pub window_select: (u32, [(bool, bool, bool, bool, u8); 6]);
        pub window_bounds: [u8; 4];
        pub fixed_color: [u8; 3];
        pub color_math: (bool, bool, [bool; 6]);
        pub color_selection: (u8, u8, bool, bool);
        pub display_geometry: [bool; 6];
        pub brightness: u8;
        pub forced_blank: bool;
        pub cgram: [u16; 256];
        pub write_latches: [u8; 3];
        pub cgram_pointer: u8;
        pub cgram_second_write: bool;
        pub cgram_buffer, ppu1_open_bus, ppu2_open_bus: u8;
        pub vram_read_buffer, vram_pointer, vram_increment: u16;
        pub vram_increment_on_high: bool;
        pub vram_remap_mode: u8;
        // Per-run observations, not emulated or serialized audio state.
        pub apu_upload_count: u32;
        pub apu_startup_upload: bool;
    }
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
    RomPtr((0x930000 | a as i32) as u32, game) as *mut ProjectileInstr
}
crate::ram_map::ram_pointer_functions! {
    Get_Botwoon(o: i32) -> Enemy_Botwoon = [0xf78, o as isize];
}

pub unsafe fn Call(addr: u32, game: &mut Game) {
    crate::native_assertion! {
        '_c2rust_label: addr & 0x8000 != 0;
        b"addr & 0x8000\0", b"src/sm_cpu_infra.c\0", 54, b"void Call(uint32)\0"
    };
    RunAsmCode(addr, 0, 0, 0, 0, game);
}
pub unsafe fn SnesRomPtr(v: u32, game: &mut Game) -> *mut u8 {
    RomPtr(v, game) as *mut u8
}
pub unsafe fn FixBugHook(addr: u32, game: &mut Game) -> bool {
    match game.hookmode {
        1 => {
            let rombyte = SnesRomPtr(addr, game);
            crate::update_place! { previous fresh0 = game.hookcnt => game.hookcnt.wrapping_add(1); }
            game.hook_fixbug_orgbyte[fresh0 as usize] = *rombyte;
            *rombyte = 0;
            return false_0 != 0;
        }
        2 => {
            if addr == game.hookadr {
                game.hookmode = 3;
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
    game.hookcnt = 0;
    if FixBugHook(0x86ef35, game) {
        (*game.g_cpu).x = (*game.g_cpu).y;
    } else if FixBugHook(0x86ef45, game) {
        (*game.g_cpu).z = (*game.g_cpu).a as i32 == 0;
    } else if FixBugHook(0x818ab8, game) {
        if (*game.g_cpu).y as i32 == 0 {
            (*game.g_cpu).pc = 0x8b1f;
        }
    } else if FixBugHook(0xa794ba, game) {
        (*game.g_cpu).y = (*game.g_cpu).x;
    } else if FixBugHook(0xa7b968, game) {
        (*game.g_cpu).x = word!(game.cur_enemy_index);
    } else if FixBugHook(0xa7b963, game) {
        (*game.g_cpu).x = word!(game.cur_enemy_index);
    } else if FixBugHook(0xa496c8, game) {
        (*game.g_cpu).a = 0;
    } else if FixBugHook(0x9085aa, game) {
        (*game.g_cpu).a = word!(game.speed_boost_counter);
    } else if FixBugHook(0xa29044, game) as i32 != 0 || FixBugHook(0xa2905d, game) as i32 != 0 {
        (*game.g_cpu).c = (*game.g_cpu).a as i32 == 0;
    } else if FixBugHook(0xa29051, game) {
        (*game.g_cpu).a = (*game.g_cpu).a.wrapping_sub(1);
    } else if FixBugHook(0xa5931c, game) {
        (*game.g_cpu).x = word!(game.cur_enemy_index);
    } else if FixBugHook(0x80ada4, game) {
        (*game.g_cpu).a = word!(game.layer2_y_pos);
    } else if FixBugHook(0x80add9, game) {
        (*game.g_cpu).a = word!(game.layer2_y_pos);
    } else if FixBugHook(0x80ad4d, game) {
        (*game.g_cpu).a = word!(game.layer2_x_pos);
    } else if FixBugHook(0x80ad77, game) {
        (*game.g_cpu).a = word!(game.layer2_x_pos);
    } else if FixBugHook(0x9381db, game) {
        let k: i32 = (*game.g_cpu).x as i32;
        let ip: i32 = word!(game.projectile_bomb_instruction_ptr[(k >> 1)]) as i32;
        let delta: i32 = if word!(game.projectile_bomb_instruction_timers[(k >> 1)]) as i32 == 1
            && (*get_ProjectileInstr(ip as u16, game)).timer & 0x8000 == 0
        {
            0
        } else {
            -8i32
        };
        (*game.g_cpu).a = ((*game.g_cpu).a as i32 + (8 + delta)) as u16;
    } else if FixBugHook(0x86b701, game) {
        (*game.g_cpu).x = (*game.g_cpu).y;
    } else if FixBugHook(0x8fc1b0, game) {
        (*game.g_cpu).x = (*game.g_cpu).a;
    } else if !FixBugHook(0x80804f, game) {
        if FixBugHook(0x829325, game) {
            (*game.g_cpu).db = 0x82;
        } else if FixBugHook(0x848acd, game) {
            (*game.g_cpu).a = 0;
        } else if FixBugHook(0xa7ceb2, game) {
            (*game.g_cpu).x = word!(game.cur_enemy_index);
        } else if FixBugHook(0x91cd44, game) {
            if (*game.g_cpu).x as i32 == 0 {
                (*game.g_cpu).pc = 0xcd52;
            }
        } else if FixBugHook(0x82afd3, game) {
            if word!(game.pausemenu_equipment_category_item) as u8 as i32 != 1 {
                return 0x82afd9;
            }
        } else if FixBugHook(0x82b0cd, game) {
            if word!(game.pausemenu_equipment_category_item) as u8 as i32 != 2 {
                return 0x82afd9;
            }
        } else if FixBugHook(0x82b15b, game) {
            if word!(game.pausemenu_equipment_category_item) as u8 as i32 != 3 {
                return 0x82afd9;
            }
        } else if FixBugHook(0xa2d38c, game) {
            (*game.g_cpu).x = word!(game.cur_enemy_index);
        } else if FixBugHook(0xa4970f, game) {
            (*game.g_cpu).a = ((*game.g_cpu).a & 0xff);
            (*game.g_cpu).y = ((*game.g_cpu).x & 0x7);
        } else if FixBugHook(0xa496e0, game) {
            if (*game.g_cpu).x as i32 > 48 {
                word!(game.croco_cur_vline_idx = (*game.g_cpu).x);
                (*game.g_cpu).mf = 0 != 0;
                return 0xa497ce;
            }
        } else if FixBugHook(0x91da89, game) {
            if word!(game.special_samus_palette_frame) as i32 > 6 {
                word!(game.special_samus_palette_frame = 6);
            }
        } else if FixBugHook(0x828d56, game) {
            WriteReg(VMAIN as i32 as u16, 0x80, game);
        } else if FixBugHook(0x88afcf, game) {
            if (*game.g_cpu).a & 0x8000 != 0 {
                (*game.g_cpu).a = 0;
            }
        } else if FixBugHook(0x88aff2, game) {
            if ((*game.g_cpu).a as i32) < 256 {
                (*game.g_cpu).a = 256;
            }
        } else if FixBugHook(0x8189bd, game) {
            if (*game.g_cpu).y as i32 == 0 {
                return 0x818a35;
            }
        } else if FixBugHook(0xa29bc1, game) {
            (*game.g_cpu).a = 1;
        } else if FixBugHook(0x82e910, game) {
            *(ram_ptr!(game, u16; 22)) = 0;
        } else if FixBugHook(0x90a4c8, game) {
            *(ram_ptr!(game, u16; 18)) = 0;
        } else if FixBugHook(0xa99f60, game) {
            *(ram_ptr!(game, u16; 22)) = 1;
        } else if FixBugHook(0x94a85b, game) {
            memset(ram_ptr!(game, c_void; 0xd82), 0, 8 as size_t);
        } else if FixBugHook(0xa0a35c, game) {
            word!(
                game.REMOVED_R18 = *(ram_ptr!(game, u16; 0xcea) as *mut u8).offset(
                    (size_of::<u16>() as usize)
                        .wrapping_div(size_of::<u8>() as usize)
                        .wrapping_sub(1_) as isize,
                ) as u16
            );
            word!(
                game.REMOVED_R20 = (word!(game.REMOVED_R18) as i32
                    + (word!(game.REMOVED_R18) as i32 >> 1)
                    >> 1) as u16
            );
        } else if FixBugHook(0xa7b049, game) {
            (*game.g_cpu).x = 0;
        } else if FixBugHook(0xa5a018, game) {
            word!(game.REMOVED_varE24 = 0);
        } else if FixBugHook(0xb39ddb, game) {
            let E = Get_Botwoon(word!(game.cur_enemy_index) as i32, game);
            word!(game.REMOVED_R18 = ((*E).base.x_pos as i32 - (*E).var_56 as i32) as u16);
            word!(game.REMOVED_R20 = ((*E).base.y_pos as i32 - (*E).var_57 as i32) as u16);
        } else if FixBugHook(0xb39e13, game) {
            let E_0 = Get_Botwoon(word!(game.cur_enemy_index) as i32, game);
            (*E_0).var_45 = ((*E_0).var_45 as i32
                + (word!(game.REMOVED_R22) as i32 - (*E_0).var_45 as i32) as i8 as i32 * 3 / 4)
                as u8 as u16;
            word!(game.REMOVED_R22 = (*E_0).var_45);
        }
    }
    0
}
pub unsafe fn RunPatchBugHook(addr: u32, game: &mut Game) -> i32 {
    let new_pc: u32 = PatchBugs(2, addr, game);
    if game.hookmode == 3 {
        if new_pc == 0 {
            return game.hook_fixbug_orgbyte[game.hookcnt as usize] as i32;
        } else {
            (*game.g_cpu).k = (new_pc >> 16) as u8;
            (*game.g_cpu).pc = (new_pc & 0xffffu32).wrapping_add(1) as u16;
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
    if i_0 >= 0 {
        return i_0;
    }
    crate::native_assertion! {
        '_c2rust_label;
        b"0\0", b"src/sm_cpu_infra.c\0", 455, b"int CpuOpcodeHook(uint32)\0"
    };
    0
}
pub unsafe fn HookedFunctionRts(is_long: i32, game: &mut Game) -> bool {
    if game.g_calling_asm_from_c {
        game.g_calling_asm_from_c_ret = is_long;
        game.g_calling_asm_from_c = false_0 != 0;
        return false_0 != 0;
    }
    crate::native_assertion! {
        '_c2rust_label;
        b"0\0", b"src/sm_cpu_infra.c\0", 465, b"_Bool HookedFunctionRts(int)\0"
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
// Preserve the existing comparison exceptions in their original copy order.
// These ranges belong to disjoint snapshot RAM fields, not live RAM.
fn normalize_snapshot_ram(mine: &mut [u8; 0x20000], theirs: &mut [u8; 0x20000]) {
    enum Source {
        Mine,
        Theirs,
    }
    use Source::{Mine, Theirs};
    for (source, range) in [
        (Theirs, 0x0000..0x0051),
        (Theirs, 0x1f5b..0x2000),
        (Theirs, 0x00ad..0x00b1),
        (Theirs, 0x05ed..0x05f5),
        (Theirs, 0x05bc..0x05c5),
        (Mine, 0x060b..0x0611),
        (Mine, 0x0611..0x0617),
        (Theirs, 0x0641..0x0643),
        (Mine, 0x077e..0x0783),
        (Mine, 0x078f..0x0791),
        (Mine, 0x07b7..0x07b9),
        (Mine, 0x0933..0x093d),
        (Theirs, 0x0b24..0x0b28),
        (Mine, 0x0d1e..0x0d20),
        (Mine, 0x0d82..0x0d8a),
        (Mine, 0x0d9c..0x0d9e),
        (Mine, 0x0dd2..0x0dd8),
        (Mine, 0x0d8a..0x0d90),
        (Mine, 0x0e20..0x0e46),
        (Mine, 0x0e54..0x0e56),
        (Mine, 0x0e02..0x0e04),
        (Mine, 0x0e4a..0x0e4c),
        (Mine, 0x0e56..0x0e5a),
        (Mine, 0x1784..0x178c),
        (Mine, 0x1790..0x1794),
        (Mine, 0x17a8..0x17ac),
        (Mine, 0x1834..0x183c),
        (Mine, 0x184a..0x185c),
        (Mine, 0x186e..0x1886),
        (Mine, 0x18a6..0x18a8),
        (Mine, 0x189a..0x18a6),
        (Theirs, 0x1966..0x196c),
        (Mine, 0xef74..0xef78),
        (Mine, 0xf37a..0xf380),
    ] {
        match source {
            Mine => theirs[range.clone()].copy_from_slice(&mine[range]),
            Theirs => mine[range.clone()].copy_from_slice(&theirs[range]),
        }
    }
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
    // Keep every field comparison and diagnostic in its original order.
    macro_rules! compare_snapshot_fields {
        ($($field:ident => $message:literal),* $(,)?) => {
            $(if (*b).$field != (*a).$field {
                game.g_fail = true;
                eprintln!($message, game.snes_frame_counter, (*b).$field, (*a).$field);
            })*
        };
    }
    compare_snapshot_fields! {
        status_flags => "@{}: PPU status flags (rangeOver, timeOver, evenFrame) compare failed: mine {:?} != theirs {:?}",
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
    compare_snapshot_fields! {
        display_geometry => "@{}: Display geometry (interlace, OBJ interlace, overscan, pseudo-hires, frame interlace, frame overscan) compare failed: mine {:?} != theirs {:?}",
        color_selection => "@{}: CGWSEL (clip, prevent math, subscreen, direct color) compare failed: mine {:?} != theirs {:?}",
        color_math => "@{}: CGADSUB (subtract, half, layer enables) compare failed: mine {:?} != theirs {:?}",
        fixed_color => "@{}: Fixed color (R, G, B) compare failed: mine {:?} != theirs {:?}",
        window_bounds => "@{}: Window boundaries (WH0, WH1, WH2, WH3) compare failed: mine {:?} != theirs {:?}",
        window_select => "@{}: Window selection (raw, enables, inversion, logic) compare failed: mine {:?} != theirs {:?}",
        screen_routing => "@{}: Screen routing (enables, windows, layer flags) compare failed: mine {:?} != theirs {:?}",
        bg_address => "@{}: BG address (wide, high, tilemap, tile data) compare failed: mine {:?} != theirs {:?}",
        bg_scroll => "@{}: BG scroll (horizontal, vertical) compare failed: mine {:?} != theirs {:?}",
        bgmode => "@{}: BGMODE (mode, BG3 priority, tile sizes) compare failed: mine {:?} != theirs {:?}",
        mosaic => "@{}: Mosaic (BG enables, size, start line) compare failed: mine {:?} != theirs {:?}",
        mode7_flags => "@{}: Mode 7 flags (large field, character fill, X flip, Y flip, EXTBG) compare failed: mine {:?} != theirs {:?}",
        mode7_matrix => "@{}: Mode 7 matrix compare failed: mine {:?} != theirs {:?}",
        counters => "@{}: PPU counter compare failed: mine {:?} != theirs {:?}",
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
    compare_snapshot_fields! {
        vram_read_buffer => "@{}: VRAM read buffer compare failed: mine ${:04X} != theirs ${:04X}",
        write_latches => "@{}: PPU write latch compare failed (scrollPrev, scrollPrev2, m7prev): mine {:?} != theirs {:?}",
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
    normalize_snapshot_ram(&mut (*b).ram, &mut (*a).ram);
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
        let mut j: i32 = 0;
        let mut i: size_t = 0 as size_t;
        while i < 0x20000i32 as size_t {
            if (*a).ram[i as usize] as i32 != (*b).ram[i as usize] as i32 {
                j += 1;
                if j < 256 {
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
        let mut j_0: i32 = 0;
        let mut i_0: size_t = 0 as size_t;
        while i_0 < 0x2000 as size_t {
            if (*a).sram[i_0 as usize] as i32 != (*b).sram[i_0 as usize] as i32 {
                j_0 += 1;
                if j_0 < 128 {
                    if i_0 & 1 as size_t == 0 as size_t
                        && (*a).sram[i_0.wrapping_add(1 as size_t) as usize] as i32
                            != (*b).sram[i_0.wrapping_add(1 as size_t) as usize] as i32
                    {
                        fprintf(
                            stderr,
                            b"0x%.6X: %.4X != %.4X (%.4X)\n\0" as *const u8 as *const c_char,
                            i_0 as i32,
                            *((&raw mut (*b).sram as *mut u8).offset(i_0 as isize) as *mut u16)
                                as i32,
                            *((&raw mut (*a).sram as *mut u8).offset(i_0 as isize) as *mut u16)
                                as i32,
                            *((&raw mut (*prev).sram as *mut u8).offset(i_0 as isize) as *mut u16)
                                as i32,
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
    compare_snapshot_fields! {
        sprite => "@{}: Sprite state compare failed: mine {:?} != theirs {:?}",
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
    (*s).a = (*c).a;
    (*s).x = (*c).x;
    (*s).y = (*c).y;
    (*s).sp = (*c).sp;
    (*s).dp = (*c).dp;
    (*s).db = (*c).db;
    (*s).pc = (*c).pc;
    (*s).k = (*c).k;
    (*s).flags = cpu_getFlags(&*c);
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
    (*c).a = (*s).a;
    (*c).x = (*s).x;
    (*c).y = (*s).y;
    (*c).sp = (*s).sp;
    (*c).dp = (*s).dp;
    (*c).db = (*s).db;
    (*c).pc = (*s).pc;
    (*c).k = (*s).k;
    (*snes).vTimer = (*s).vTimer;
    (*c).e = (*s).cpu_emulation;
    (*c).stopped = (*s).cpu_stopped;
    (*c).irqWanted = (*s).cpu_irq_pending;
    (*c).nmiWanted = (*s).cpu_nmi_pending;
    cpu_setFlags((*s).flags, &mut *c);
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
    let org_sp: u16 = (*game.g_cpu).sp;
    let org_pc: u16 = (*game.g_cpu).pc;
    let org_b: u8 = (*game.g_cpu).db;
    let org_dp: u16 = (*game.g_cpu).dp;
    printf(b"RunAsmCode!\n\0" as *const u8 as *const c_char);
    game.g_ram[0x1ffffi32 as usize] = 1;
    let dc: bool = (*game.g_snes).debug_cycles;
    (*game.g_cpu).db = (pc >> 16) as u8;
    (*game.g_cpu).a = a;
    (*game.g_cpu).x = x;
    (*game.g_cpu).y = y;
    (*game.g_cpu).spBreakpoint = (*game.g_cpu).sp;
    (*game.g_cpu).k = (pc >> 16) as u8;
    (*game.g_cpu).pc = (pc & 0xffff) as u16;
    (*game.g_cpu).mf = flags & 1 != 0;
    (*game.g_cpu).xf = (flags & 2) >> 1 != 0;
    game.g_calling_asm_from_c = true_0 != 0;
    while game.g_calling_asm_from_c {
        let pc_0: u32 = (words!(i32, (*(*game.g_snes).cpu), k, pc)) as u32;
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
        if flags & 1 != 0 {
            let mut i: i32 = 0;
            while i < 10 {
                apu_cycle((*game.g_snes).apu, game);
                i += 1;
            }
        }
    }
    (*game.g_cpu).dp = org_dp;
    (*game.g_cpu).sp = org_sp;
    (*game.g_cpu).db = org_b;
    (*game.g_cpu).pc = org_pc;
    (*game.g_snes).debug_cycles = dc;
    game.g_calling_asm_from_c_ret
}
unsafe fn loadRom(name: *const c_char, snes: *mut Snes, game: &mut Game) -> bool {
    let mut length: size_t = 0 as size_t;
    let mut file: *mut u8 = ReadWholeFile(name, &raw mut length);
    if file.is_null() {
        puts(b"Failed to read file\0" as *const u8 as *const c_char);
        return false_0 != 0;
    }
    let result: bool = snes_loadRom(snes, file, length as i32, game);
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
    *SnesRomPtr(addr, game) = 0;
}

pub unsafe fn RtlUpdateSnesPatchForBugfix(game: &mut Game) {
    game.currently_installed_bug_fix_counter = word!(game.bug_fix_counter);
    PatchBytes(0x8584a3, &[0x20, 0x50, 0x96, 0x60], game);
    PatchBytes(
        0x859650,
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
            (if (word!(game.bug_fix_counter) as i32) < 1 {
                0x8f
            } else {
                0x8b
            }) as u8,
            0xf0,
            0xf3,
            0x60,
        ],
        game,
    );
    PatchBytes(0x8584cc, &[0x18, 0x18], game);
}
pub unsafe fn SnesInit(filename: *const c_char, game: &mut Game) -> *mut Snes {
    game.g_snes = snes_init(&raw mut game.g_ram as *mut u8);
    game.g_cpu = (*game.g_snes).cpu;
    let loaded: bool = loadRom(filename, game.g_snes, game);
    if !loaded {
        return null_mut::<Snes>();
    }
    game.g_sram = (*(*game.g_snes).cart).ram;
    game.g_rom = (*(*game.g_snes).cart).rom;
    game.register_rom(game.g_rom, (*(*game.g_snes).cart).romSize as usize);
    RtlSetupEmuCallbacks(
        null_mut::<u8>(),
        Some(RtlRunFrameCompare as unsafe fn(u16, i32, &mut Game) -> ()),
        None,
        game,
    );
    *ram_ptr!(game; 0x611) = 1;
    PatchBytes(0x82896b, &[0x20, 0xf, 0xf7], game);
    PatchBytes(0x82f70f, &[0x7c, 0x81, 0x89], game);
    PatchBytes(0x8ba362, &[0x5f, 0xf7], game);
    PatchBytes(0x8bf760, &[0xc2, 0x20, 0x4c, 0x67, 0xa3], game);
    PatchBytes(0x8584b2, &[0xa, 0xa], game);
    PatchBytes(
        0x84efd3,
        &[0xc0, 0, 0, 0xf0, 0x3, 0x20, 0x64, 0x87, 0x60],
        game,
    );
    PatchBytes(0x848243, &[0xd3, 0xef], game);
    PatchBytes(0x86f4a6, &[0xac, 0x91, 0x19, 0x4c, 0x76, 0x86], game);
    PatchBytes(0x8685bd, &[0xa6, 0xf4], game);
    PatchBytes(
        0x82f713,
        &[
            0xc2, 0x30, 0x22, 0x59, 0x94, 0x80, 0x20, 0x48, 0x89, 0x22, 0x38, 0x83, 0x80, 0x4c,
            0x13, 0xf7,
        ],
        game,
    );
    PatchBytes(0x828944, &[0x58, 0x4c, 0x13, 0xf7], game);
    PatchBytes(0x82897a, &[0x28, 0x60], game);
    PatchBytes(0x80945c, &[0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18], game);
    PatchBytes(
        0xa8b237,
        &[0xbc, 0xaa, 0xf, 0xc9, 0x6c, 0, 0x10, 0x1a],
        game,
    );
    PatchBytes(0xa2904b, &[0x49, 0xff, 0xff, 0x69, 0, 0], game);
    PatchBytes(0xa29065, &[0x49, 0xff, 0xff, 0x69, 0, 0], game);
    PatchBytes(0xa0896f, &[0x6b], game);
    PatchBytes(0xa98c12, &[0x18, 0x18, 0x18], game);
    PatchBytes(0x8085f6, &[0x60], game);
    PatchBytes(
        0x80843c,
        &[0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18],
        game,
    );
    PatchBytes(
        0x808475,
        &[0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18],
        game,
    );
    PatchBytes(
        0x808525,
        &[0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18],
        game,
    );
    PatchBytes(0x8882a1, &[0x18, 0x18, 0x18, 0x18], game);
    PatchBytes(0x828533, &[0x18, 0x18, 0x18, 0x18], game);
    PatchBytes(0x80837b, &[0x18, 0x18, 0x18, 0x18], game);
    PatchBytes(0x80838e, &[0x18, 0x18, 0x18, 0x18], game);
    PatchBytes(0x82df6c, &[0x18, 0x18, 0x18, 0x18], game);
    PatchBytes(0x80d000, &[0x20, 0x32, 0x96, 0x6b], game);
    PatchBytes(0x82e02c, &[0x22, 0, 0xd0, 0x80, 0x18], game);
    PatchBytes(0x82e06b, &[0x22, 0, 0xd0, 0x80, 0x18], game);
    PatchBytes(0x82e50d, &[0x22, 0, 0xd0, 0x80, 0x18], game);
    PatchBytes(0x82e609, &[0x22, 0, 0xd0, 0x80, 0x18], game);
    PatchBytes(0x82e526, &[0x22, 0x4, 0xd0, 0x80, 0x18], game);
    PatchBytes(
        0x80d004,
        &[0x22, 0x38, 0x83, 0x80, 0xad, 0x31, 0x9, 0x10, 0xf7, 0x6b],
        game,
    );
    PatchBytes(0x82e540, &[0x18, 0x18, 0x18, 0x18], game);
    PatchBytes(0x8bc11e, &[0x18, 0x18, 0x18, 0x18], game);
    PatchBytes(0x8bd487, &[0x18, 0x18, 0x18, 0x18], game);
    PatchBytes(0x858148, &[0x18, 0x18, 0x18], game);
    PatchBytes(0x8581b2, &[0x18, 0x18, 0x18], game);
    PatchBytes(
        0x8581ea,
        &[0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18],
        game,
    );
    PatchBytes(0x858203, &[0x18, 0x18, 0x18], game);
    PatchBytes(
        0x858236,
        &[0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18],
        game,
    );
    PatchBytes(
        0x8582b8,
        &[
            0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
        ],
        game,
    );
    PatchBytes(0x858321, &[0x18, 0x18, 0x18], game);
    PatchBytes(
        0x85835a,
        &[0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18],
        game,
    );
    PatchBytes(0x858532, &[0x18, 0x18, 0x18], game);
    PatchBytes(
        0x85856b,
        &[0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18],
        game,
    );
    PatchBytes(0x858096, &[0x18, 0x18, 0x18], game);
    PatchBytes(0x8580b4, &[0x18, 0x18, 0x18], game);
    PatchBytes(0x8580dc, &[0x18, 0x18, 0x18], game);
    PatchBytes(0x8580f2, &[0x18, 0x18, 0x18], game);
    PatchBytes(0x85861c, &[0x18, 0x18, 0x18], game);
    PatchBytes(0x858651, &[0x18, 0x18, 0x18], game);
    PatchBytes(0x858692, &[0x18, 0x18, 0x18, 0x18], game);
    PatchBytes(0x858696, &[0x18, 0x18, 0x18, 0x18], game);
    PatchBytes(0x859660, &[0x8, 0xc2, 0x30, 0x4c, 0xa9, 0x85], game);
    PatchBytes(0x8583ba, &[0x20, 0x60, 0x96], game);
    PatchBytes(
        0x859670,
        &[
            0x22, 0x81, 0x96, 0x85, 0xc9, 0xff, 0xff, 0xd0, 0x4, 0x5c, 0x5f, 0xab, 0xa2, 0x5c,
            0x26, 0xab, 0xa2,
        ],
        game,
    );
    PatchBytes(
        0x859681,
        &[
            0xcd, 0x1f, 0x1c, 0xd0, 0x8, 0x9c, 0x1f, 0x1c, 0xad, 0xf9, 0x5, 0x6b, 0xff, 0x8d, 0xc8,
            0xd, 0xa9, 0xff, 0xff, 0x6b,
        ],
        game,
    );
    PatchBytes(0xa2ab22, &[0x5c, 0x70, 0x96, 0x85], game);
    PatchBytes(
        0x8596a0,
        &[
            0x22, 0xd4, 0x8f, 0xa0, 0xad, 0xc8, 0xd, 0xf0, 0x7, 0x22, 0x95, 0x96, 0x85, 0x9c, 0xc8,
            0xd, 0x6b,
        ],
        game,
    );
    PatchBytes(0x828b65, &[0x22, 0xa0, 0x96, 0x85], game);
    PatchBytes(
        0x8596c0,
        &[0x20, 0x89, 0x85, 0xa9, 0x1c, 0, 0x8d, 0x1f, 0x1c, 0x60],
        game,
    );
    PatchBytes(0x8580e5, &[0x20, 0xc0, 0x96], game);
    PatchBytes(
        0x84efdc,
        &[
            0xad, 0xc8, 0xd, 0xf0, 0x11, 0x98, 0x9d, 0x27, 0x1d, 0xad, 0xc8, 0xd, 0x22, 0x95, 0x96,
            0x85, 0x9c, 0xc8, 0xd, 0xbc, 0x27, 0x1d, 0x4c, 0xee, 0x85,
        ],
        game,
    );
    PatchBytes(0x8485f7, &[0xf4, 0xdb, 0xef], game);
    PatchBytes(
        0x859695,
        &[0x8, 0x8b, 0xda, 0x5a, 0x5c, 0x84, 0x80, 0x85],
        game,
    );
    PatchBytes(0x858080, &[0x8d, 0xc8, 0xd, 0x6b], game);
    PatchBytes(
        0x84f000,
        &[
            0x22, 0x81, 0x96, 0x85, 0xc9, 0xff, 0xff, 0xf0, 0x4, 0x5c, 0xfa, 0x8c, 0x84, 0x7a,
            0xfa, 0x88, 0x88, 0x60,
        ],
        game,
    );
    PatchBytes(0x848cf6, &[0x5c, 0, 0xf0, 0x84], game);
    PatchBytes(0x81f000, &[0xa9, 0xff, 0xff, 0x8d, 0x98, 0x9, 0x60], game);
    PatchBytes(0x819027, &[0x5c, 0, 0xf0, 0x81], game);
    PatchBytes(0x819112, &[0x5c, 0, 0xf0, 0x81], game);
    PatchBytes(0x8194e9, &[0x5c, 0, 0xf0, 0x81], game);
    PatchBytes(0x8095e1, &[0x18, 0x18, 0x18, 0x18], game);
    if game.g_use_my_apu_code {
        PatchBytes(0x808028, &[0x60], game);
    }
    PatchBytes(0x828a59, &[0x18, 0x18, 0x18, 0x80], game);
    PatchBytes(0x828a72, &[0x18, 0x18, 0x18], game);
    PatchBytes(0x828a80, &[0x18, 0x18, 0x18, 0x80], game);
    PatchBytes(0x828a67, &[0x6], game);
    PatchBytes(0x82831e, &[0, 0x2e], game);
    PatchBytes(0x91c234, &[0xa5, 0x25], game);
    PatchBytes(0x809af3, &[0x18, 0x18, 0x18, 0x18], game);
    PatchBytes(
        0xa7ff82,
        &[
            0xa5, 0x12, 0x48, 0xa5, 0x14, 0x48, 0xa5, 0x16, 0x48, 0x22, 0xa7, 0xa6, 0xa0, 0x68,
            0x85, 0x16, 0x68, 0x85, 0x14, 0x68, 0x85, 0x12, 0x6b,
        ],
        game,
    );
    PatchBytes(0xa7b03a, &[0x22, 0x82, 0xff, 0xa7], game);
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
    PatchBugs(1, 0, game);
    game.g_snes
}
pub unsafe fn DebugGameOverMenu() {
    crate::native_assertion! {
        '_c2rust_label;
        b"0\0", b"src/sm_cpu_infra.c\0", 914, b"void DebugGameOverMenu(void)\0"
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
        let addr: u32 = (words!(i32, (*(*game.g_snes).cpu), k, pc)) as u32;
        if addr == pc1 || addr == pc2 {
            return addr;
        }
    }
}
pub unsafe fn RunOneFrameOfGame_Emulated(game: &mut Game) {
    let bug_fix_bak: u16 = word!(game.bug_fix_counter);
    let cpu = (*game.g_snes).cpu;
    if (*cpu).k == 0x80 && (*cpu).pc == 0x8459 {
        RunCpuUntilPC(0x808460, 0x808460, game);
        (*cpu).pc = 0x8462;
    }
    RunCpuUntilPC(0x808343, 0x85813c, game);
    (*(*game.g_snes).cpu).nmiWanted = true_0 != 0;
    RunCpuUntilPC(0x80834a, 0x858142, game);
    word!(game.bug_fix_counter = bug_fix_bak);
}
pub unsafe fn DrawFrameToPpu(game: &mut Game) {
    (*game.g_snes).vPos = 0;
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
    if !game.g_debug_flag && game.g_got_mismatch_count == 0 {
        let mut buffer: [c_char; 64] = [0; 64];
        snprintf(
            &raw mut buffer as *mut c_char,
            size_of::<[c_char; 64]>() as size_t,
            b"saves/bug-%d.sav\0" as *const u8 as *const c_char,
            time(null_mut::<time_t>()) as i32,
        );
        RtlSaveSnapshot(&raw mut buffer as *mut c_char, true_0 != 0, game);
    }
    game.g_got_mismatch_count = 5 * 60;
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
        (*game.g_snes).runningWhichVersion = 1;
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
        (*game.g_snes).runningWhichVersion = 2;
        RunOneFrameOfGame(game);
        DrawFrameToPpu(game);
        let queue_mine = crate::audio_snapshot::ApuQueueSnapshot::capture(game);
        MakeSnapshot(
            &mut game.g_snapshot_mine,
            &*game.g_snes,
            &*game.g_cpu,
            game.g_spc_player,
        );
        (*game.g_snes).runningWhichVersion = 0xff;
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
    (*game.g_snes).runningWhichVersion = 0;
    if word!(game.game_state) as i32 == 0xffff {
        (*(*game.g_snes).cpu).k = 0x80;
        (*(*game.g_snes).cpu).pc = 0x8459;
        *ram_ptr!(game; 0x611) = 3;
    }
    if word!(game.menu_index) & 0xff00 != 0 {
        printf(b"MENU INDEX TOO BIG!\n\0" as *const u8 as *const c_char);
        SaveBugSnapshot(game);
        word!(game.menu_index, |fresh1| (fresh1 & 0xff));
    }
    if game.g_got_mismatch_count != 0 {
        game.g_got_mismatch_count -= 1;
    }
}
pub unsafe fn RtlRunFrameCompare(input: u16, run_what: i32, game: &mut Game) {
    (*(*game.g_snes).input1).currentState = input;
    if game.g_runmode as i32 == RM_THEIRS as i32 {
        RunOneFrameOfGame_Emulated(game);
        DrawFrameToPpu(game);
    } else if game.g_runmode as i32 == RM_MINE as i32 {
        (*game.g_snes).runningWhichVersion = 0xff;
        RunOneFrameOfGame(game);
        DrawFrameToPpu(game);
        (*game.g_snes).runningWhichVersion = 0;
    } else {
        RunOneFrameOfGame_Both(game);
    };
}
