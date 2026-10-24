use crate::game::sm_80::Vector_IRQ;
use crate::game::sm_82::RunOneFrameOfGame;
use crate::game_state::Game;
use crate::ram_map::ram_ptr;
use crate::ram_map::word;
use crate::rom_patches::RomPatches;
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
pub use crate::types::ProjectileInstr;
pub use crate::types::{Cpu, Dma, Enemy_Botwoon, Ppu, Snes, size_t, time_t};
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
    initial; [#[derive(Copy, Clone)] #[repr(C)]];
    pub struct Snapshot {
        pub audio: crate::audio_snapshot::AudioSnapshot;
        pub player: crate::audio_snapshot::PlayerSnapshot;
        // Extra live fields are inert: never followed, restored or compared.
        pub cpu: Cpu;
        pub ppu: Ppu;
        pub dma: Dma;
        pub snes: Snes;
        pub ram: [u8; 131072];
        pub sram: [u8; 8192];
        // Per-run observations, not emulated or serialized audio state.
        pub apu_upload_count: u32;
        pub apu_startup_upload: bool;
    }
}
pub const RM_MINE: RunMode = 1;
pub const RM_BOTH: RunMode = 0;
pub const RM_THEIRS: RunMode = 2;
pub use crate::types::RunMode;
pub const NULL: *mut c_void = null_mut::<c_void>();
pub const true_0: i32 = 1;
pub const false_0: i32 = 0;
#[inline]
unsafe fn get_ProjectileInstr(a: u16, game: &mut Game) -> *mut ProjectileInstr {
    unsafe { RomPtr((0x930000 | a as i32) as u32, game) as *mut ProjectileInstr }
}
crate::ram_map::ram_pointer_functions! {
    Get_Botwoon(o: i32) -> Enemy_Botwoon = [0xf78, o as isize];
}

pub unsafe fn Call(addr: u32, game: &mut Game) {
    unsafe {
        crate::native_assertion! {
            '_c2rust_label: addr & 0x8000 != 0;
            b"addr & 0x8000\0", b"src/sm_cpu_infra.c\0", 54, b"void Call(uint32)\0"
        };
        RunAsmCode(addr, 0, 0, 0, 0, game);
    }
}
// The native diagnostic is the remaining unsafe boundary; ROM access is checked.
unsafe fn patch_image(address: u32, game: &mut Game) -> RomPatches {
    unsafe {
        if address & 0x8000 == 0 {
            printf(c"RomPtr - Invalid access 0x%x!\n".as_ptr(), address);
            game.g_fail = true;
        }
        RomPatches::new(
            game.loaded_rom
                .as_ref()
                .expect("ROM must be loaded")
                .clone(),
        )
    }
}
pub unsafe fn FixBugHook(addr: u32, game: &mut Game) -> bool {
    unsafe {
        match game.hookmode {
            1 => {
                let image = patch_image(addr, game);
                crate::update_place! { previous fresh0 = game.hookcnt => game.hookcnt.wrapping_add(1); }
                game.hook_fixbug_orgbyte[fresh0 as usize] = image.byte(addr);
                image.write(addr, 0, 0);
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
    unsafe {
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
                word! { game;
                    REMOVED_R18 = *ram_ptr!(game; 0xcea, (size_of::<u16>() as usize)
                            .wrapping_div(size_of::<u8>() as usize)
                            .wrapping_sub(1_) as isize) as u16;
                    REMOVED_R20 = (word!(game.REMOVED_R18) as i32
                        + (word!(game.REMOVED_R18) as i32 >> 1)
                        >> 1) as u16;
                }
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
}
pub unsafe fn RunPatchBugHook(addr: u32, game: &mut Game) -> i32 {
    unsafe {
        let new_pc: u32 = PatchBugs(2, addr, game);
        if game.hookmode == 3 {
            if new_pc == 0 {
                return game.hook_fixbug_orgbyte[game.hookcnt as usize] as i32;
            } else {
                (*game.g_cpu).k = (new_pc >> 16) as u8;
                (*game.g_cpu).pc = (new_pc & 0xffffu32).wrapping_add(1) as u16;
                return patch_image(new_pc, game).byte(new_pc) as i32;
            }
        }
        -1i32
    }
}
pub unsafe fn CpuOpcodeHook(addr: u32, game: &mut Game) -> i32 {
    unsafe {
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
}
pub unsafe fn HookedFunctionRts(is_long: i32, game: &mut Game) -> bool {
    unsafe {
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
unsafe fn compare_snapshot_bytes(
    mine: &[u8],
    theirs: &[u8],
    previous: &[u8],
    header: *const c_char,
    report_limit: i32,
    odd_word_end: usize,
    game: &mut Game,
) {
    unsafe {
        if mine == theirs {
            return;
        }
        game.g_fail = true;
        fprintf(stderr, header, game.snes_frame_counter);
        let mut failed = 0;
        let mut i = 0;
        while i < mine.len() {
            if theirs[i] != mine[i] {
                failed += 1;
                if failed < report_limit {
                    if (i & 1 == 0 || i < odd_word_end) && theirs[i + 1] != mine[i + 1] {
                        let word =
                            |bytes: &[u8]| i32::from(u16::from_le_bytes([bytes[i], bytes[i + 1]]));
                        fprintf(
                            stderr,
                            c"0x%.6X: %.4X != %.4X (%.4X)\n".as_ptr(),
                            i as i32,
                            word(mine),
                            word(theirs),
                            word(previous),
                        );
                        i += 1;
                        failed += 1;
                    } else {
                        fprintf(
                            stderr,
                            c"0x%.6X: %.2X != %.2X (%.2X)\n".as_ptr(),
                            i as i32,
                            mine[i] as i32,
                            theirs[i] as i32,
                            previous[i] as i32,
                        );
                    }
                }
            }
            i += 1;
        }
        if failed != 0 {
            game.g_fail = true;
        }
        fprintf(stderr, c"  total of %d failed bytes\n".as_ptr(), failed);
    }
}

unsafe fn VerifySnapshotsEq(
    b: *mut Snapshot,
    a: *mut Snapshot,
    prev: *mut Snapshot,
    game: &mut Game,
) {
    unsafe {
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
        ($($projection:expr => $message:literal),* $(,)?) => {
            $(let mine = $projection(&(*b).ppu);
                let theirs = $projection(&(*a).ppu);
                if mine != theirs {
                game.g_fail = true;
                eprintln!($message, game.snes_frame_counter, mine, theirs);
            })*
        };
    }
        compare_snapshot_fields! {
            |ppu: &Ppu| [ppu.rangeOver, ppu.timeOver, ppu.evenFrame] => "@{}: PPU status flags (rangeOver, timeOver, evenFrame) compare failed: mine {:?} != theirs {:?}",
        }
        compare_snapshot_fields! {
            |ppu: &Ppu| (ppu.brightness, ppu.forcedBlank) => "@{}: INIDISP (brightness, forced blank) compare failed: mine {:?} != theirs {:?}",
        }
        compare_snapshot_fields! {
            |ppu: &Ppu| [
            ppu.interlace,
            ppu.objInterlace,
            ppu.overscan,
            ppu.pseudoHires,
            ppu.frameInterlace,
            ppu.frameOverscan,
        ] => "@{}: Display geometry (interlace, OBJ interlace, overscan, pseudo-hires, frame interlace, frame overscan) compare failed: mine {:?} != theirs {:?}",
            |ppu: &Ppu| (
            ppu.clipMode,
            ppu.preventMathMode,
            ppu.addSubscreen,
            ppu.directColor,
        ) => "@{}: CGWSEL (clip, prevent math, subscreen, direct color) compare failed: mine {:?} != theirs {:?}",
            |ppu: &Ppu| (ppu.subtractColor, ppu.halfColor, ppu.mathEnabled) => "@{}: CGADSUB (subtract, half, layer enables) compare failed: mine {:?} != theirs {:?}",
            |ppu: &Ppu| [ppu.fixedColorR, ppu.fixedColorG, ppu.fixedColorB] => "@{}: Fixed color (R, G, B) compare failed: mine {:?} != theirs {:?}",
            |ppu: &Ppu| [
            ppu.window1left,
            ppu.window1right,
            ppu.window2left,
            ppu.window2right,
        ] => "@{}: Window boundaries (WH0, WH1, WH2, WH3) compare failed: mine {:?} != theirs {:?}",
            |ppu: &Ppu| (
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
        ) => "@{}: Window selection (raw, enables, inversion, logic) compare failed: mine {:?} != theirs {:?}",
            |ppu: &Ppu| (
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
        ) => "@{}: Screen routing (enables, windows, layer flags) compare failed: mine {:?} != theirs {:?}",
            |ppu: &Ppu| ppu
            .bgLayer
            .map(|bg| (bg.tilemapWider, bg.tilemapHigher, bg.tilemapAdr, bg.tileAdr)) => "@{}: BG address (wide, high, tilemap, tile data) compare failed: mine {:?} != theirs {:?}",
            |ppu: &Ppu| ppu.bgLayer.map(|bg| (bg.hScroll, bg.vScroll)) => "@{}: BG scroll (horizontal, vertical) compare failed: mine {:?} != theirs {:?}",
            |ppu: &Ppu| (ppu.mode, ppu.bg3priority, ppu.bgLayer.map(|bg| bg.bigTiles)) => "@{}: BGMODE (mode, BG3 priority, tile sizes) compare failed: mine {:?} != theirs {:?}",
            |ppu: &Ppu| (
            ppu.bgLayer.map(|bg| bg.mosaicEnabled),
            ppu.mosaicSize,
            ppu.mosaicStartLine,
        ) => "@{}: Mosaic (BG enables, size, start line) compare failed: mine {:?} != theirs {:?}",
            |ppu: &Ppu| [
            ppu.m7largeField,
            ppu.m7charFill,
            ppu.m7xFlip,
            ppu.m7yFlip,
            ppu.m7extBg,
        ] => "@{}: Mode 7 flags (large field, character fill, X flip, Y flip, EXTBG) compare failed: mine {:?} != theirs {:?}",
            |ppu: &Ppu| ppu.m7matrix => "@{}: Mode 7 matrix compare failed: mine {:?} != theirs {:?}",
            |ppu: &Ppu| CounterSnapshot {
            horizontal: ppu.hCount,
            vertical: ppu.vCount,
            horizontal_second_byte: ppu.hCountSecond,
            vertical_second_byte: ppu.vCountSecond,
            latched: ppu.countersLatched,
        } => "@{}: PPU counter compare failed: mine {:?} != theirs {:?}",
        }
        if ((*b).ppu.ppu1openBus, (*b).ppu.ppu2openBus)
            != ((*a).ppu.ppu1openBus, (*a).ppu.ppu2openBus)
        {
            game.g_fail = true;
            eprintln!(
                "@{}: PPU open bus compare failed: mine {:02X}/{:02X} != theirs {:02X}/{:02X}",
                game.snes_frame_counter,
                (*b).ppu.ppu1openBus,
                (*b).ppu.ppu2openBus,
                (*a).ppu.ppu1openBus,
                (*a).ppu.ppu2openBus
            );
        }
        compare_snapshot_fields! {
            |ppu: &Ppu| (ppu.cgramPointer, ppu.cgramSecondWrite, ppu.cgramBuffer) => "@{}: CGRAM port (address, second byte, buffer) compare failed: mine {:?} != theirs {:?}",
            |ppu: &Ppu| (ppu.vramPointer, ppu.vramIncrement, ppu.vramIncrementOnHigh, ppu.vramRemapMode) => "@{}: VRAM port (address, increment, high, remap) compare failed: mine {:?} != theirs {:?}",
        }
        compare_snapshot_fields! {
            |ppu: &Ppu| ppu.vramReadBuffer => "@{}: VRAM read buffer compare failed: mine ${:04X} != theirs ${:04X}",
            |ppu: &Ppu| [
            ppu.scrollPrev,
            ppu.scrollPrev2,
            ppu.m7prev,
        ] => "@{}: PPU write latch compare failed (scrollPrev, scrollPrev2, m7prev): mine {:?} != theirs {:?}",
        }
        if (*b).ppu.cgram != (*a).ppu.cgram {
            game.g_fail = true;
            eprintln!(
                "@{}: CGRAM compare failed (mine != theirs)",
                game.snes_frame_counter
            );
            for (index, (&mine, &theirs)) in (*b).ppu.cgram.iter().zip(&(*a).ppu.cgram).enumerate()
            {
                if mine != theirs {
                    eprintln!("CGRAM ${index:03X}: ${mine:04X} != ${theirs:04X}");
                }
            }
        }
        normalize_snapshot_ram(&mut (*b).ram, &mut (*a).ram);
        compare_snapshot_bytes(
            &(*b).ram,
            &(*a).ram,
            &(*prev).ram,
            c"@%d: Memory compare failed (mine != theirs, prev):\n".as_ptr(),
            256,
            0x10000,
            game,
        );
        compare_snapshot_bytes(
            &(*b).sram,
            &(*a).sram,
            &(*prev).sram,
            c"@%d: SRAM compare failed (mine != theirs, prev):\n".as_ptr(),
            128,
            0,
            game,
        );
        if memcmp(
            &raw mut (*b).ppu.vram as *mut u16 as *const c_void,
            &raw mut (*a).ppu.vram as *mut u16 as *const c_void,
            (size_of::<u16>() as size_t).wrapping_mul(0x8000 as size_t),
        ) != 0
        {
            game.g_fail = true_0 != 0;
            fprintf(
                stderr,
                c"@%d: VRAM compare failed (mine != theirs, prev):\n".as_ptr(),
                game.snes_frame_counter,
            );
            let mut j_1: size_t = 0 as size_t;
            for i_1 in 0usize..32768 {
                if (*a).ppu.vram[i_1 as usize] as i32 != (*b).ppu.vram[i_1 as usize] as i32 {
                    fprintf(
                        stderr,
                        c"0x%.6X: %.4X != %.4X (%.4X)\n".as_ptr(),
                        i_1 as i32,
                        (*b).ppu.vram[i_1 as usize] as i32,
                        (*a).ppu.vram[i_1 as usize] as i32,
                        (*prev).ppu.vram[i_1 as usize] as i32,
                    );
                    game.g_fail = true_0 != 0;
                    j_1 = j_1.wrapping_add(1);
                    if j_1 >= 32 as size_t {
                        break;
                    }
                }
            }
        }
        if (*b).ppu.oam != (*a).ppu.oam || (*b).ppu.highOam != (*a).ppu.highOam {
            game.g_fail = true;
            eprintln!("@{}: OAM memory compare failed", game.snes_frame_counter);
            let low = (*b)
                .ppu
                .oam
                .iter()
                .zip(&(*a).ppu.oam)
                .zip(&(*prev).ppu.oam)
                .enumerate()
                .filter(|(_, ((mine, theirs), _))| mine != theirs)
                .map(|(index, ((mine, theirs), previous))| (index * 2, *mine, *theirs, *previous));
            let high = (*b)
                .ppu
                .highOam
                .iter()
                .zip(&(*a).ppu.highOam)
                .zip(&(*prev).ppu.highOam)
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
                eprintln!(
                    "OAM byte ${offset:03X}: mine ${mine:04X} != theirs ${theirs:04X}, previous ${previous:04X}"
                );
            }
        }
        compare_snapshot_fields! {
            |ppu: &Ppu| SpriteSnapshot {
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
        } => "@{}: Sprite state compare failed: mine {:?} != theirs {:?}",
        }
    }
}
// Snapshot storage is separate from the live emulator allocations. Narrow
// borrows let the compiler retain that distinction across the bulk copies.
unsafe fn MakeSnapshot(s: &mut Snapshot, snes: &Snes, c: &Cpu, player: *mut SpcPlayer) {
    unsafe {
        s.player.capture(player);
        s.audio.capture(snes);
        s.ppu = *snes.ppu;
        s.cpu = *c;
        s.snes = *snes;
        // DMA rewind retains its four historical fields, excluding the outer padding.
        s.dma.channel = (*snes.dma).channel;
        s.dma.hdmaTimer = (*snes.dma).hdmaTimer;
        s.dma.dmaTimer = (*snes.dma).dmaTimer;
        s.dma.dmaBusy = (*snes.dma).dmaBusy;
        memcpy(s.ram.as_mut_ptr().cast(), snes.ram.cast(), 0x20000);
        (*snes.cart)
            .ram
            .read_bytes(0, &mut s.sram[..(*snes.cart).ramSize as usize]);
    }
}
// This field list preserves the historical partial restore and its phases.
// Snapshot pointers, CPU counters and renderer caches remain inert copies.
macro_rules! copy_snapshot_fields {
    ($target:expr, $source:expr; $($field:ident),* $(,)?) => {
        $($target.$field = $source.$field;)*
    };
}
unsafe fn RestoreSnapshot(s: &Snapshot, snes: &mut Snes, c: &mut Cpu) {
    unsafe {
        s.audio.restore(snes);
        let ppu = &mut *snes.ppu;
        copy_snapshot_fields! {ppu, s.ppu;
            brightness, forcedBlank, fixedColorR, fixedColorG, fixedColorB,
            subtractColor, halfColor, mathEnabled, interlace, objInterlace,
            overscan, pseudoHires, frameInterlace, frameOverscan,
            clipMode, preventMathMode, addSubscreen, directColor,
            window1left, window1right, window2left, window2right, windowsel,
        }
        for (w, saved) in ppu.windowLayer.iter_mut().zip(&s.ppu.windowLayer) {
            copy_snapshot_fields! {w, saved;
                window1enabled, window2enabled, window1inversed, window2inversed, maskLogic,
            }
        }
        copy_snapshot_fields! {ppu, s.ppu; screenEnabled, screenWindowed}
        for (layer, saved) in ppu.layer.iter_mut().zip(&s.ppu.layer) {
            copy_snapshot_fields! {layer, saved;
                mainScreenEnabled, subScreenEnabled, mainScreenWindowed, subScreenWindowed,
            }
        }
        for (bg, saved) in ppu.bgLayer.iter_mut().zip(&s.ppu.bgLayer) {
            copy_snapshot_fields! {bg, saved; tilemapWider, tilemapHigher, tilemapAdr, tileAdr}
        }
        for (bg, saved) in ppu.bgLayer.iter_mut().zip(&s.ppu.bgLayer) {
            copy_snapshot_fields! {bg, saved; hScroll, vScroll}
        }
        copy_snapshot_fields! {ppu, s.ppu; mode, bg3priority}
        for (bg, saved) in ppu.bgLayer.iter_mut().zip(&s.ppu.bgLayer) {
            bg.bigTiles = saved.bigTiles;
        }
        for (bg, saved) in ppu.bgLayer.iter_mut().zip(&s.ppu.bgLayer) {
            bg.mosaicEnabled = saved.mosaicEnabled;
        }
        copy_snapshot_fields! {ppu, s.ppu; mosaicSize, mosaicStartLine}
        if !brightness_cache_is_valid(ppu) {
            ppu.lastBrightnessMult = 255;
        }
        copy_snapshot_fields! {ppu, s.ppu;
            m7matrix, m7largeField, m7charFill, m7xFlip, m7yFlip, m7extBg,
            rangeOver, timeOver, evenFrame, scrollPrev, scrollPrev2, m7prev,
            cgram, cgramPointer, cgramSecondWrite, cgramBuffer, ppu1openBus, ppu2openBus,
            vramReadBuffer, vramPointer, vramIncrement, vramIncrementOnHigh, vramRemapMode,
        }
        copy_snapshot_fields! {c, s.cpu; a, x, y, sp, dp, db, pc, k}
        snes.vTimer = s.snes.vTimer;
        copy_snapshot_fields! {c, s.cpu; e, stopped, irqWanted, nmiWanted}
        cpu_setFlags(cpu_getFlags(&s.cpu), c);
        c.waiting = s.cpu.waiting;
        copy_snapshot_fields! {snes, s.snes;
            hPos, vPos, frames, inNmi, inIrq, inVblank,
            hIrqEnabled, vIrqEnabled, nmiEnabled, hTimer, openBus,
            ppuLatch, autoJoyRead, autoJoyTimer, ramAdr,
        }
        let dma = &mut *snes.dma;
        copy_snapshot_fields! {dma, s.dma; channel, hdmaTimer, dmaTimer, dmaBusy}
        copy_snapshot_fields! {snes, s.snes; multiplyA, multiplyResult, divideA, divideResult}
        memcpy(snes.ram.cast(), s.ram.as_ptr().cast(), 0x20000);
        (*snes.cart).ram.write_bytes(0, &s.sram[..0x2000]);
        (*snes.ppu).vram = s.ppu.vram;
        let ppu = &mut *snes.ppu;
        copy_snapshot_fields! {ppu, s.ppu;
            hCount, vCount, hCountSecond, vCountSecond, countersLatched,
            oam, highOam, oamAdr, oamAdrWritten, oamInHigh, oamInHighWritten,
            oamSecondWrite, oamBuffer, objPriority, objTileAdr1, objTileAdr2, objSize,
        }
        ppu.objPixelBufferXX[..19].copy_from_slice(&s.ppu.objPixelBufferXX[..19]);
    }
}
pub unsafe fn RunAsmCode(pc: u32, a: u16, x: u16, y: u16, flags: i32, game: &mut Game) -> i32 {
    unsafe {
        let org_sp: u16 = (*game.g_cpu).sp;
        let org_pc: u16 = (*game.g_cpu).pc;
        let org_b: u8 = (*game.g_cpu).db;
        let org_dp: u16 = (*game.g_cpu).dp;
        printf(c"RunAsmCode!\n".as_ptr());
        game.g_ram[0x1ffffi32 as usize] = 1;
        let dc: bool = (*game.g_snes).debug_cycles;
        crate::set_fields! {[(*game.g_cpu)]; db = (pc >> 16) as u8, a = a, x = x, y = y,
        spBreakpoint = (*game.g_cpu).sp, k = (pc >> 16) as u8, pc = (pc & 0xffff) as u16,
        mf = flags & 1 != 0, xf = (flags & 2) >> 1 != 0, };
        game.g_calling_asm_from_c = true_0 != 0;
        while game.g_calling_asm_from_c {
            let pc_0: u32 = (words!(i32, (*(*game.g_snes).cpu), k, pc)) as u32;
            if (*game.g_snes).debug_cycles {
                let mut line = [0u8; 80];
                getProcessorStateCpu(game.g_snes, &mut line, game);
                puts(line.as_ptr().cast());
                line[0] = 0;
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
        crate::set_fields! {[(*game.g_cpu)]; dp = org_dp, sp = org_sp, db = org_b, pc = org_pc, };
        (*game.g_snes).debug_cycles = dc;
        game.g_calling_asm_from_c_ret
    }
}
unsafe fn loadRom(name: *const c_char, snes: *mut Snes, game: &mut Game) -> bool {
    unsafe {
        let mut length: size_t = 0 as size_t;
        let mut file: *mut u8 = ReadWholeFile(name, &raw mut length);
        if file.is_null() {
            puts(c"Failed to read file".as_ptr());
            return false_0 != 0;
        }
        let result: bool = snes_loadRom(snes, file, length as i32, game);
        free(file as *mut c_void);
        result
    }
}
pub unsafe fn PatchBytes(addr: u32, bytes: &[u8], game: &mut Game) {
    unsafe {
        // Resolve the original base for every byte; RomPtr can update g_fail.
        for (i, &byte) in bytes.iter().enumerate() {
            patch_image(addr, game).write(addr, i, byte);
        }
    }
}
pub unsafe fn FixupCarry(addr: u32, game: &mut Game) {
    unsafe {
        patch_image(addr, game).write(addr, 0, 0);
    }
}

pub fn RtlUpdateSnesPatchForBugfix(game: &mut Game) {
    let counter = u16::from_le_bytes([game.g_ram[0xc2], game.g_ram[0xc3]]);
    game.currently_installed_bug_fix_counter = counter;
    RomPatches::new(
        game.loaded_rom
            .as_ref()
            .expect("ROM must be loaded")
            .clone(),
    )
    .update_bugfix(counter);
}
pub unsafe fn SnesInit(filename: *const c_char, game: &mut Game) -> *mut Snes {
    unsafe {
        game.g_snes = snes_init(&raw mut game.g_ram as *mut u8);
        game.g_cpu = (*game.g_snes).cpu;
        let loaded: bool = loadRom(filename, game.g_snes, game);
        if !loaded {
            return null_mut::<Snes>();
        }
        game.g_sram = (*(*game.g_snes).cart).ram.clone();
        game.register_native_rom((*(*game.g_snes).cart).rom.clone());
        game.register_rom((*(*game.g_snes).cart).rom.clone());
        RtlSetupEmuCallbacks(
            null_mut::<u8>(),
            Some(RtlRunFrameCompare as unsafe fn(u16, i32, &mut Game) -> ()),
            None,
            game,
        );
        *ram_ptr!(game; 0x611) = 1;
        RomPatches::new(
            game.loaded_rom
                .as_ref()
                .expect("ROM must be loaded")
                .clone(),
        )
        .install_startup(game.g_use_my_apu_code);
        RtlUpdateSnesPatchForBugfix(game);
        let mut i: size_t = 0 as size_t;
        while i != (size_of::<[u32; 155]>() as usize).wrapping_div(size_of::<u32>() as usize) {
            let t_84: u8 =
                patch_image(kPatchedCarrys[i as usize], game).byte(kPatchedCarrys[i as usize]);
            if t_84 != 0 {
                game.kPatchedCarrysOrg[i as usize] = t_84;
                FixupCarry(kPatchedCarrys[i as usize], game);
            } else {
                printf(
                    c"0x%x double patched!\n".as_ptr(),
                    kPatchedCarrys[i as usize],
                );
            }
            i = i.wrapping_add(1);
        }
        PatchBugs(1, 0, game);
        game.g_snes
    }
}
pub unsafe fn DebugGameOverMenu() {
    unsafe {
        crate::native_assertion! {
            '_c2rust_label;
            b"0\0", b"src/sm_cpu_infra.c\0", 914, b"void DebugGameOverMenu(void)\0"
        };
    }
}
pub unsafe fn RunCpuUntilPC(pc1: u32, pc2: u32, game: &mut Game) -> u32 {
    unsafe {
        loop {
            if (*game.g_snes).debug_cycles {
                let mut line = [0u8; 80];
                getProcessorStateCpu(game.g_snes, &mut line, game);
                puts(line.as_ptr().cast());
            }
            cpu_runOpcode(game.g_cpu, game);
            let addr: u32 = (words!(i32, (*(*game.g_snes).cpu), k, pc)) as u32;
            if addr == pc1 || addr == pc2 {
                return addr;
            }
        }
    }
}
pub unsafe fn RunOneFrameOfGame_Emulated(game: &mut Game) {
    unsafe {
        if !game.supports_legacy_persistence() {
            eprintln!(
                "RunOneFrameOfGame_Emulated: CPU authority is unavailable for a prepared world"
            );
            return;
        }
        let actor_transition = game.prepare_actor_canonical();
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
        game.commit_actor_canonical(actor_transition);
    }
}
pub unsafe fn DrawFrameToPpu(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn SaveBugSnapshot(game: &mut Game) {
    unsafe {
        if game.VERIFY_ACTIVE {
            return;
        }
        if !game.g_debug_flag && game.g_got_mismatch_count == 0 {
            let mut buffer: [c_char; 64] = [0; 64];
            snprintf(
                &raw mut buffer as *mut c_char,
                size_of::<[c_char; 64]>() as size_t,
                c"saves/bug-%d.sav".as_ptr(),
                time(null_mut::<time_t>()) as i32,
            );
            RtlSaveSnapshot(&raw mut buffer as *mut c_char, true_0 != 0, game);
        }
        game.g_got_mismatch_count = 5 * 60;
    }
}
// Set only by the single-threaded headless runner. The interactive oracle
// retains its existing recovery and snapshot behaviour.

unsafe fn report_first_verify_failure(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn RunOneFrameOfGame_Both(game: &mut Game) {
    unsafe {
        if !game.supports_legacy_persistence() {
            eprintln!("RunOneFrameOfGame_Both: CPU authority is unavailable for a prepared world");
            return;
        }
        (*game.g_snes).ppu = (*game.g_snes).snes_ppu;
        MakeSnapshot(
            &mut game.g_snapshot_before,
            &*game.g_snes,
            &*game.g_cpu,
            game.g_spc_player,
        );
        game.actor_snapshots.before = Some(game.capture_actor_snapshot());
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
            game.actor_snapshots.theirs = Some(game.capture_actor_snapshot());
            let queue_theirs = crate::audio_snapshot::ApuQueueSnapshot::capture(game);
            let apu_theirs = queue_theirs.pending;
            queue_before.restore(game);
            game.g_snapshot_before.player.restore(game.g_spc_player);
            game.g_snapshot_mine.apu_upload_count = 0;
            game.g_snapshot_mine.apu_startup_upload = false;
            (*game.g_snes).ppu = (*game.g_snes).my_ppu;
            let actor_restore = game
                .actor_snapshots
                .before
                .as_ref()
                .expect("paired actor snapshot missing")
                .preflight(game.actor_identity(), game.world())
                .expect("actor restore rejected");
            RestoreSnapshot(&game.g_snapshot_before, &mut *game.g_snes, &mut *game.g_cpu);
            actor_restore.commit(game.actor_identity_mut());
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
            game.actor_snapshots.mine = Some(game.capture_actor_snapshot());
            (*game.g_snes).runningWhichVersion = 0xff;
            VerifySnapshotsEq(
                &raw mut *game.g_snapshot_mine,
                &raw mut *game.g_snapshot_theirs,
                &raw mut *game.g_snapshot_before,
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
                && apu_before == [0xff; 4]
                && game.apu_queue.size == 0
                && game.apu_queue.position == 0
                && game.apu_queue.time_since_empty == 0
                && game.g_snapshot_mine.ram[0x998..0x99a] == [1, 0]
                && game.g_snapshot_mine.ram[0x79b..0x79d] == [0, 0]
                && game.apu_queue.pending == [0, 0xff, 0, 0xff]
                && apu_theirs == [0; 4];
            if game.apu_queue.pending != apu_theirs && !startup_reset {
                game.g_fail = true;
                eprintln!(
                    "@{}: APU queued ports compare failed: mine {:02X?} != theirs {:02X?}",
                    game.snes_frame_counter, game.apu_queue.pending, apu_theirs
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
                    printf(c"Verify failure!\n".as_ptr());
                }
            }
            // Headless recovery adopts the completed oracle frame, including the Rust
            // continuation bytes copied during comparison. Replaying the previous
            // frame would leave those bytes stale.
            if failed && !game.VERIFY_ACTIVE {
                (*game.g_snes).ppu = (*game.g_snes).snes_ppu;
                let actor_restore = game
                    .actor_snapshots
                    .before
                    .as_ref()
                    .expect("paired actor snapshot missing")
                    .preflight(game.actor_identity(), game.world())
                    .expect("actor restore rejected");
                RestoreSnapshot(&game.g_snapshot_before, &mut *game.g_snes, &mut *game.g_cpu);
                actor_restore.commit(game.actor_identity_mut());
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
                let actor_restore = game
                    .actor_snapshots
                    .theirs
                    .as_ref()
                    .expect("paired actor snapshot missing")
                    .preflight(game.actor_identity(), game.world())
                    .expect("actor restore rejected");
                RestoreSnapshot(&game.g_snapshot_theirs, &mut *game.g_snes, &mut *game.g_cpu);
                actor_restore.commit(game.actor_identity_mut());
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
            printf(c"MENU INDEX TOO BIG!\n".as_ptr());
            SaveBugSnapshot(game);
            word!(game.menu_index &= 0xff);
        }
        if game.g_got_mismatch_count != 0 {
            game.g_got_mismatch_count -= 1;
        }
    }
}
pub unsafe fn RtlRunFrameCompare(input: u16, run_what: i32, game: &mut Game) {
    unsafe {
        if !game.supports_legacy_persistence() && u32::from(game.g_runmode) != RM_MINE {
            eprintln!("RtlRunFrameCompare: CPU authority is unavailable for a prepared world");
            return;
        }
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
}

#[path = "verification/player_transitions_oracle.rs"]
pub mod player_transitions_oracle;
#[path = "verification/samus_palette_oracle.rs"]
pub mod samus_palette_oracle;

#[path = "verification/draygon_sprite_oracle.rs"]
pub mod draygon_sprite_oracle;

#[path = "verification/sprite_pool_oracle.rs"]
pub mod sprite_pool_oracle;

#[path = "verification/flags_save_oracle.rs"]
pub mod flags_save_oracle;

#[path = "verification/flags_save_candidate.rs"]
pub mod flags_save_candidate;

#[path = "verification/kraid_flags_candidate.rs"]
pub mod kraid_flags_candidate;
#[path = "verification/kraid_flags_oracle.rs"]
pub mod kraid_flags_oracle;

#[cfg(sm_verify_dll)]
#[path = "verification/actor_mutable_cpu_checks.rs"]
pub(crate) mod actor_mutable_cpu_checks;
#[cfg(sm_verify_dll)]
#[path = "verification/player_collision_ownership.rs"]
pub(crate) mod player_collision_ownership;

#[cfg(sm_verify_dll)]
#[path = "verification/player_transitions_rust.rs"]
pub mod player_transitions_rust;

#[cfg(sm_verify_dll)]
#[path = "verification/player_transitions_pose_families.rs"]
pub mod player_transitions_pose_families;

#[cfg(sm_verify_dll)]
#[path = "verification/player_transitions_selected.rs"]
pub mod player_transitions_selected;

#[cfg(sm_verify_dll)]
#[path = "verification/player_transitions_composed.rs"]
pub mod player_transitions_composed;

#[cfg(sm_verify_dll)]
#[path = "verification/player_transitions_callers.rs"]
pub mod player_transitions_callers;

#[cfg(sm_verify_dll)]
#[path = "verification/player_settlement.rs"]
pub mod player_settlement;

#[cfg(sm_verify_dll)]
#[path = "verification/player_locomotion.rs"]
pub mod player_locomotion;
