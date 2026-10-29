use crate::execution::{Execution, HostContext};
use crate::game::gameplay::{self, Entry};
use crate::game_state::{Game, PlayerSnapshotSource, WorkRam};
use crate::ram_map::{slot, word};
use crate::rom_patches::RomPatches;
use crate::rtl::words;
pub use crate::snes::registers::*;
use ::core::ffi::{c_char, c_void};
use ::core::mem::{MaybeUninit, size_of};
use ::core::ptr::null_mut;

use crate::c_library::{
    fopen, fprintf, fread, fseek, ftell, memcmp, printf, puts, rewind, snprintf, stderr, time,
};
use crate::rtl::{RtlSaveSnapshot, RtlSetupEmuCallbacks};

#[path = "game/session_native_adoption.rs"]
pub(crate) mod session_native_adoption;
use crate::game_state::persistent_session::SessionSnapshotPayload;
use crate::snes::cpu::{cpu_getFlags, cpu_setFlags};
use crate::snes::snes_other::snes_loadRom;
use crate::tracing::getProcessorStateCpu;
pub use crate::types::ProjectileInstr;
pub use crate::types::{Cpu, Dma, Enemy_Botwoon, Ppu, Snes, size_t, time_t};
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
unsafe fn get_ProjectileInstr(a: u16, game: &mut HookContext<'_, '_>) -> *mut ProjectileInstr {
    unsafe {
        crate::rtl::rom_ptr_parts((0x930000 | a as i32) as u32, game.host) as *mut ProjectileInstr
    }
}
pub unsafe fn Call(addr: u32, game: &mut Game) {
    unsafe {
        assert!(
            game.supports_legacy_persistence(),
            "Call: CPU authority is unavailable for a prepared world"
        );
        crate::native_assertion! {
            '_c2rust_label: addr & 0x8000 != 0;
            b"addr & 0x8000\0", b"src/sm_cpu_infra.c\0", 54, b"void Call(uint32)\0"
        };
        RunAsmCode(addr, 0, 0, 0, 0, game);
    }
}
// The native diagnostic is the remaining unsafe boundary; ROM access is checked.
unsafe fn patch_image(address: u32, host: &mut HostContext<'_>) -> RomPatches {
    unsafe {
        if address & 0x8000 == 0 {
            printf(c"RomPtr - Invalid access 0x%x!\n".as_ptr(), address);
            *host.g_fail = true;
        }
        RomPatches::new(
            host.loaded_rom
                .as_ref()
                .expect("ROM must be loaded")
                .clone(),
        )
    }
}
// Startup installs host-only ROM patches without acquiring backend execution.
unsafe fn install_startup_bug_hooks(host: &mut HostContext<'_>) {
    unsafe {
        patch_bugs_parts(
            1,
            0,
            &mut HookContext {
                backend: None,
                host,
            },
        );
    }
}

unsafe fn fix_bug_hook(addr: u32, host: &mut HostContext<'_>) -> bool {
    unsafe {
        match *host.hookmode {
            1 => {
                let image = patch_image(addr, host);
                crate::update_place! { previous fresh0 = *host.hookcnt => (*host.hookcnt).wrapping_add(1); }
                host.hook_fixbug_orgbyte[fresh0 as usize] = image.byte(addr);
                image.write(addr, 0, 0);
                return false_0 != 0;
            }
            2 => {
                if addr == *host.hookadr {
                    *host.hookmode = 3;
                    return true_0 != 0;
                }
                *host.hookcnt = (*host.hookcnt).wrapping_add(1);
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

// Compatibility hooks acquire a backend only when the selected branch uses it.
struct HookContext<'a, 'host> {
    backend: Option<&'a mut crate::snes::snes::Backend>,
    host: &'a mut HostContext<'host>,
}
impl HookContext<'_, '_> {
    fn ram(&self) -> &WorkRam {
        self.host.ram
    }
    // Hooks keep the native RAM capability of the enclosing suspended execution.
    fn compatibility_byte(&self, at: usize) -> u8 {
        self.host.ram.read_byte(at)
    }
    fn compatibility_write_byte(&self, at: usize, byte: u8) {
        self.host.ram.write_byte(at, byte);
    }
    fn cpu(&self) -> *mut Cpu {
        self.backend
            .as_ref()
            .expect("CPU hook action requires an owned backend")
            .views()
            .1
    }
    unsafe fn write_reg(&mut self, reg: u16, value: u8) {
        unsafe {
            let mut execution = Execution {
                backend: self
                    .backend
                    .as_deref_mut()
                    .expect("register hook action requires an owned backend"),
                host: self.host.reborrow(),
            };
            crate::rtl::write_reg_execution(reg, value, &mut execution);
        }
    }
}

unsafe fn patch_bugs_parts(mode: u32, addr: u32, game: &mut HookContext<'_, '_>) -> u32 {
    unsafe {
        *game.host.hookmode = mode;
        *game.host.hookadr = addr;
        *game.host.hookcnt = 0;
        if fix_bug_hook(0x86ef35, &mut game.host) {
            (*game.cpu()).x = (*game.cpu()).y;
        } else if fix_bug_hook(0x86ef45, &mut game.host) {
            (*game.cpu()).z = (*game.cpu()).a as i32 == 0;
        } else if fix_bug_hook(0x818ab8, &mut game.host) {
            if (*game.cpu()).y as i32 == 0 {
                (*game.cpu()).pc = 0x8b1f;
            }
        } else if fix_bug_hook(0xa794ba, &mut game.host) {
            (*game.cpu()).y = (*game.cpu()).x;
        } else if fix_bug_hook(0xa7b968, &mut game.host) {
            (*game.cpu()).x = word!(game.cur_enemy_index);
        } else if fix_bug_hook(0xa7b963, &mut game.host) {
            (*game.cpu()).x = word!(game.cur_enemy_index);
        } else if fix_bug_hook(0xa496c8, &mut game.host) {
            (*game.cpu()).a = 0;
        } else if fix_bug_hook(0x9085aa, &mut game.host) {
            (*game.cpu()).a = word!(game.speed_boost_counter);
        } else if fix_bug_hook(0xa29044, &mut game.host) as i32 != 0
            || fix_bug_hook(0xa2905d, &mut game.host) as i32 != 0
        {
            (*game.cpu()).c = (*game.cpu()).a as i32 == 0;
        } else if fix_bug_hook(0xa29051, &mut game.host) {
            (*game.cpu()).a = (*game.cpu()).a.wrapping_sub(1);
        } else if fix_bug_hook(0xa5931c, &mut game.host) {
            (*game.cpu()).x = word!(game.cur_enemy_index);
        } else if fix_bug_hook(0x80ada4, &mut game.host) {
            (*game.cpu()).a = word!(game.layer2_y_pos);
        } else if fix_bug_hook(0x80add9, &mut game.host) {
            (*game.cpu()).a = word!(game.layer2_y_pos);
        } else if fix_bug_hook(0x80ad4d, &mut game.host) {
            (*game.cpu()).a = word!(game.layer2_x_pos);
        } else if fix_bug_hook(0x80ad77, &mut game.host) {
            (*game.cpu()).a = word!(game.layer2_x_pos);
        } else if fix_bug_hook(0x9381db, &mut game.host) {
            let k: i32 = (*game.cpu()).x as i32;
            let ip: i32 = word!(game.projectile_bomb_instruction_ptr[(k >> 1)]) as i32;
            let delta: i32 = if word!(game.projectile_bomb_instruction_timers[(k >> 1)]) as i32 == 1
                && (*get_ProjectileInstr(ip as u16, game)).timer & 0x8000 == 0
            {
                0
            } else {
                -8i32
            };
            (*game.cpu()).a = ((*game.cpu()).a as i32 + (8 + delta)) as u16;
        } else if fix_bug_hook(0x86b701, &mut game.host) {
            (*game.cpu()).x = (*game.cpu()).y;
        } else if fix_bug_hook(0x8fc1b0, &mut game.host) {
            (*game.cpu()).x = (*game.cpu()).a;
        } else if !fix_bug_hook(0x80804f, &mut game.host) {
            if fix_bug_hook(0x829325, &mut game.host) {
                (*game.cpu()).db = 0x82;
            } else if fix_bug_hook(0x848acd, &mut game.host) {
                (*game.cpu()).a = 0;
            } else if fix_bug_hook(0xa7ceb2, &mut game.host) {
                (*game.cpu()).x = word!(game.cur_enemy_index);
            } else if fix_bug_hook(0x91cd44, &mut game.host) {
                if (*game.cpu()).x as i32 == 0 {
                    (*game.cpu()).pc = 0xcd52;
                }
            } else if fix_bug_hook(0x82afd3, &mut game.host) {
                if word!(game.pausemenu_equipment_category_item) as u8 as i32 != 1 {
                    return 0x82afd9;
                }
            } else if fix_bug_hook(0x82b0cd, &mut game.host) {
                if word!(game.pausemenu_equipment_category_item) as u8 as i32 != 2 {
                    return 0x82afd9;
                }
            } else if fix_bug_hook(0x82b15b, &mut game.host) {
                if word!(game.pausemenu_equipment_category_item) as u8 as i32 != 3 {
                    return 0x82afd9;
                }
            } else if fix_bug_hook(0xa2d38c, &mut game.host) {
                (*game.cpu()).x = word!(game.cur_enemy_index);
            } else if fix_bug_hook(0xa4970f, &mut game.host) {
                (*game.cpu()).a = (*game.cpu()).a & 0xff;
                (*game.cpu()).y = (*game.cpu()).x & 0x7;
            } else if fix_bug_hook(0xa496e0, &mut game.host) {
                if (*game.cpu()).x as i32 > 48 {
                    word!(game.croco_cur_vline_idx = (*game.cpu()).x);
                    (*game.cpu()).mf = 0 != 0;
                    return 0xa497ce;
                }
            } else if fix_bug_hook(0x91da89, &mut game.host) {
                if word!(game.special_samus_palette_frame) as i32 > 6 {
                    word!(game.special_samus_palette_frame = 6);
                }
            } else if fix_bug_hook(0x828d56, &mut game.host) {
                game.write_reg(VMAIN as i32 as u16, 0x80);
            } else if fix_bug_hook(0x88afcf, &mut game.host) {
                if (*game.cpu()).a & 0x8000 != 0 {
                    (*game.cpu()).a = 0;
                }
            } else if fix_bug_hook(0x88aff2, &mut game.host) {
                if ((*game.cpu()).a as i32) < 256 {
                    (*game.cpu()).a = 256;
                }
            } else if fix_bug_hook(0x8189bd, &mut game.host) {
                if (*game.cpu()).y as i32 == 0 {
                    return 0x818a35;
                }
            } else if fix_bug_hook(0xa29bc1, &mut game.host) {
                (*game.cpu()).a = 1;
            } else if fix_bug_hook(0x82e910, &mut game.host) {
                game.ram().copy_in(22, &0u16.to_ne_bytes());
            } else if fix_bug_hook(0x90a4c8, &mut game.host) {
                game.ram().copy_in(18, &0u16.to_ne_bytes());
            } else if fix_bug_hook(0xa99f60, &mut game.host) {
                game.ram().copy_in(22, &1u16.to_ne_bytes());
            } else if fix_bug_hook(0x94a85b, &mut game.host) {
                game.ram().fill(0xd82..0xd8a, 0);
            } else if fix_bug_hook(0xa0a35c, &mut game.host) {
                word! { game;
                    REMOVED_R18 = game.ram().read_byte(0xceb) as u16;
                    REMOVED_R20 = (word!(game.REMOVED_R18) as i32
                        + (word!(game.REMOVED_R18) as i32 >> 1)
                        >> 1) as u16;
                }
            } else if fix_bug_hook(0xa7b049, &mut game.host) {
                (*game.cpu()).x = 0;
            } else if fix_bug_hook(0xa5a018, &mut game.host) {
                word!(game.REMOVED_varE24 = 0);
            } else if fix_bug_hook(0xb39ddb, &mut game.host) {
                let E = word!(game.cur_enemy_index) as i32;
                word!(
                    game.REMOVED_R18 = (slot!(game, Enemy_Botwoon[E].base.x_pos) as i32
                        - slot!(game, Enemy_Botwoon[E].var_56) as i32)
                        as u16
                );
                word!(
                    game.REMOVED_R20 = (slot!(game, Enemy_Botwoon[E].base.y_pos) as i32
                        - slot!(game, Enemy_Botwoon[E].var_57) as i32)
                        as u16
                );
            } else if fix_bug_hook(0xb39e13, &mut game.host) {
                let E_0 = word!(game.cur_enemy_index) as i32;
                slot!(
                    game,
                    Enemy_Botwoon[E_0].var_45 = (slot!(game, Enemy_Botwoon[E_0].var_45) as i32
                        + (word!(game.REMOVED_R22) as i32
                            - slot!(game, Enemy_Botwoon[E_0].var_45) as i32)
                            as i8 as i32
                            * 3
                            / 4) as u8 as u16
                );
                word!(game.REMOVED_R22 = slot!(game, Enemy_Botwoon[E_0].var_45));
            }
        }
        0
    }
}
unsafe fn run_patch_bug_hook_parts(addr: u32, game: &mut HookContext<'_, '_>) -> i32 {
    unsafe {
        let new_pc: u32 = patch_bugs_parts(2, addr, game);
        if *game.host.hookmode == 3 {
            if new_pc == 0 {
                return game.host.hook_fixbug_orgbyte[*game.host.hookcnt as usize] as i32;
            } else {
                (*game.cpu()).k = (new_pc >> 16) as u8;
                (*game.cpu()).pc = (new_pc & 0xffffu32).wrapping_add(1) as u16;
                return patch_image(new_pc, &mut game.host).byte(new_pc) as i32;
            }
        }
        -1i32
    }
}
pub(crate) unsafe fn opcode_hook(game: &mut Execution<'_>, addr: u32) -> i32 {
    unsafe {
        opcode_hook_parts(
            addr,
            &mut HookContext {
                backend: Some(&mut *game.backend),
                host: &mut game.host,
            },
        )
    }
}
unsafe fn opcode_hook_parts(addr: u32, game: &mut HookContext<'_, '_>) -> i32 {
    unsafe {
        let mut i: size_t = 0 as size_t;
        while i != (size_of::<[u32; 155]>() as usize).wrapping_div(size_of::<u32>() as usize) {
            if addr == kPatchedCarrys[i as usize] {
                return game.host.kPatchedCarrysOrg[i as usize] as i32;
            }
            i = i.wrapping_add(1);
        }
        let i_0: i32 = run_patch_bug_hook_parts(addr, game);
        if i_0 >= 0 {
            return i_0;
        }
        crate::native_assertion! {
            '_c2rust_label;
            b"0\0", b"src/sm_cpu_infra.c\0", 455, b"int CpuOpcodeHook(uint32)\0"
        };
    }
}
pub(crate) unsafe fn hooked_return(game: &mut Execution<'_>, is_long: i32) -> bool {
    unsafe { hooked_return_parts(is_long, &mut game.host) }
}
unsafe fn hooked_return_parts(is_long: i32, host: &mut HostContext<'_>) -> bool {
    unsafe {
        if *host.g_calling_asm_from_c {
            *host.g_calling_asm_from_c_ret = is_long;
            *host.g_calling_asm_from_c = false_0 != 0;
            return false_0 != 0;
        }
        crate::native_assertion! {
            '_c2rust_label;
            b"0\0", b"src/sm_cpu_infra.c\0", 465, b"_Bool HookedFunctionRts(int)\0"
        };
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
fn normalize_snapshot_ram(
    mine: &mut [u8; 0x20000],
    theirs: &mut [u8; 0x20000],
    session_mine: &mut SessionSnapshotPayload,
    session_theirs: &mut SessionSnapshotPayload,
) -> Result<(), String> {
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
            Mine => {
                theirs[range.clone()].copy_from_slice(&mine[range.clone()]);
                session_theirs
                    .normalize_from(session_mine, range)
                    .map_err(|error| format!("{error:?}"))?;
            }
            Theirs => {
                mine[range.clone()].copy_from_slice(&theirs[range.clone()]);
                session_mine
                    .normalize_from(session_theirs, range)
                    .map_err(|error| format!("{error:?}"))?;
            }
        }
    }
    Ok(())
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
    session_mine: &mut SessionSnapshotPayload,
    session_theirs: &mut SessionSnapshotPayload,
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
        if let Err(error) =
            normalize_snapshot_ram(&mut (*b).ram, &mut (*a).ram, session_mine, session_theirs)
        {
            game.revoke_session_authority();
            panic!("Session comparison normalization rejected: {error}");
        }
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
// RAM must be the selected backend's stable binding at both capture and restore.
unsafe fn MakeSnapshotWithPlayerOwner(
    s: &mut Snapshot,
    snes: &Snes,
    c: &Cpu,
    player: *mut crate::types::SpcPlayer,
    owner: Option<&mut crate::spc_player::NativePlayer>,
    ram: &WorkRam,
    backend: &crate::snes::snes::Backend,
) {
    unsafe {
        s.player.capture_with_owner(owner, player);
        s.audio.capture(snes);
        s.ppu = *snes.ppu;
        s.cpu = *c;
        s.snes = *snes;
        // DMA rewind retains its four historical fields, excluding the outer padding.
        let dma = backend.dma_state();
        s.dma.channel = dma.channel;
        s.dma.hdmaTimer = dma.hdmaTimer;
        s.dma.dmaTimer = dma.dmaTimer;
        s.dma.dmaBusy = dma.dmaBusy;
        ram.copy_out(0, &mut s.ram);
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
unsafe fn RestoreSnapshot(
    s: &Snapshot,
    snes: &mut Snes,
    c: &mut Cpu,
    ram: &WorkRam,
    backend: &crate::snes::snes::Backend,
) {
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
        let mut dma = backend.dma_state();
        copy_snapshot_fields! {dma, s.dma; channel, hdmaTimer, dmaTimer, dmaBusy}
        backend.set_dma_state(dma);
        copy_snapshot_fields! {snes, s.snes; multiplyA, multiplyResult, divideA, divideResult}
        ram.copy_in(0, &s.ram);
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
pub unsafe fn RunAsmCode(
    pc: u32,
    a: u16,
    x: u16,
    y: u16,
    flags: i32,
    native_game: &mut Game,
) -> i32 {
    let game = &mut *native_game;
    unsafe {
        assert!(
            game.supports_legacy_persistence(),
            "RunAsmCode: CPU authority is unavailable for a prepared world"
        );
        let actor_transition = game.prepare_actor_canonical();
        game.preflight_samus_runtime()
            .expect("Samus CPU entry rejected");
        let mut mutation = game
            .begin_samus_mutation()
            .expect("Samus mutation rejected");
        let game = mutation.game();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let org_sp: u16 = (*game.g_cpu).sp;
            let org_pc: u16 = (*game.g_cpu).pc;
            let org_b: u8 = (*game.g_cpu).db;
            let org_dp: u16 = (*game.g_cpu).dp;
            printf(c"RunAsmCode!\n".as_ptr());
            game.ram().write_byte(0x1ffffi32 as usize, 1);
            let dc: bool = (*game.g_snes).debug_cycles;
            crate::set_fields! {[(*game.g_cpu)]; db = (pc >> 16) as u8, a = a, x = x, y = y,
            spBreakpoint = (*game.g_cpu).sp, k = (pc >> 16) as u8, pc = (pc & 0xffff) as u16,
            mf = flags & 1 != 0, xf = (flags & 2) >> 1 != 0, };
            game.g_calling_asm_from_c = true_0 != 0;
            while game.g_calling_asm_from_c {
                let _pc_0: u32 = (words!(i32, (*(*game.g_snes).cpu), k, pc)) as u32;
                if (*game.g_snes).debug_cycles {
                    let mut line = [0u8; 80];
                    getProcessorStateCpu(&mut game.execution(), &mut line);
                    puts(line.as_ptr().cast());
                    line[0] = 0;
                }
                game.execution().step_cpu();
                while game.dma_busy() {
                    game.execution().do_dma();
                }
                if flags & 1 != 0 {
                    let mut i: i32 = 0;
                    while i < 10 {
                        game.execution().cycle_apu();
                        i += 1;
                    }
                }
            }
            crate::set_fields! {[(*game.g_cpu)]; dp = org_dp, sp = org_sp, db = org_b, pc = org_pc, };
            (*game.g_snes).debug_cycles = dc;
            game.g_calling_asm_from_c_ret
        }));
        game.commit_actor_canonical(actor_transition);
        match result {
            Ok(result) => {
                mutation
                    .finish_current()
                    .expect("Samus CPU adoption rejected");
                session_native_adoption::reconcile_current(native_game)
                    .expect("Session native result adoption rejected");
                result
            }
            Err(panic) => std::panic::resume_unwind(panic),
        }
    }
}
unsafe fn loadRom(name: *const c_char, game: &mut Game) -> bool {
    unsafe {
        let file = fopen(name, c"rb".as_ptr());
        if file.is_null() {
            puts(c"Failed to read file".as_ptr());
            return false_0 != 0;
        }
        let file = crate::util::InputFile(file);
        fseek(file.0, 0, crate::util::SEEK_END);
        let length: size_t = ftell(file.0) as size_t;
        rewind(file.0);
        // A returning Die may leave unread bytes; only spare capacity is promised.
        let mut bytes = Vec::<MaybeUninit<u8>>::new();
        if bytes.try_reserve_exact(length.wrapping_add(1)).is_err() {
            crate::host::Die(c"malloc failed".as_ptr());
        }
        bytes.spare_capacity_mut()[length].write(MaybeUninit::new(0));
        let data = bytes.as_mut_ptr().cast::<u8>();
        if fread(data.cast(), 1, length, file.0) as u32 as size_t != length {
            crate::host::Die(c"fread failed".as_ptr());
        }
        drop(file);
        snes_loadRom(data, length as i32, game)
    }
}
pub unsafe fn PatchBytes(addr: u32, bytes: &[u8], game: &mut Game) {
    unsafe {
        if !bytes.is_empty() {
            game.note_samus_external_mutation();
        }
        // Resolve the original base for every byte; RomPtr can update g_fail.
        for (i, &byte) in bytes.iter().enumerate() {
            patch_image(addr, &mut game.execution_parts().1).write(addr, i, byte);
        }
    }
}
pub unsafe fn FixupCarry(addr: u32, game: &mut Game) {
    unsafe {
        game.note_samus_external_mutation();
        patch_image(addr, &mut game.execution_parts().1).write(addr, 0, 0);
    }
}

pub fn RtlUpdateSnesPatchForBugfix(game: &mut Game) {
    game.note_samus_external_mutation();
    let counter = u16::from_le_bytes([game.ram().read_byte(0xc2), game.ram().read_byte(0xc3)]);
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
        game.note_samus_external_mutation();
        game.initialize_backend();
        let loaded: bool = loadRom(filename, game);
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
        game.ram().write_byte(0x611, 1);
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
            let t_84: u8 = patch_image(kPatchedCarrys[i as usize], &mut game.execution_parts().1)
                .byte(kPatchedCarrys[i as usize]);
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
        install_startup_bug_hooks(&mut game.execution_parts().1);
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
        game.note_samus_external_mutation();
        loop {
            if (*game.g_snes).debug_cycles {
                let mut line = [0u8; 80];
                getProcessorStateCpu(&mut game.execution(), &mut line);
                puts(line.as_ptr().cast());
            }
            game.execution().step_cpu();
            let addr: u32 = (words!(i32, (*(*game.g_snes).cpu), k, pc)) as u32;
            if addr == pc1 || addr == pc2 {
                return addr;
            }
        }
    }
}
pub unsafe fn RunOneFrameOfGame_Emulated(native_game: &mut Game) {
    let game = &mut *native_game;
    unsafe {
        if !game.supports_legacy_persistence() {
            eprintln!(
                "RunOneFrameOfGame_Emulated: CPU authority is unavailable for a prepared world"
            );
            return;
        }
        let actor_transition = game.prepare_actor_canonical();
        game.preflight_samus_runtime()
            .expect("Samus CPU entry rejected");
        let mut mutation = game
            .begin_samus_mutation()
            .expect("Samus mutation rejected");
        let game = mutation.game();
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
        mutation
            .finish_current()
            .expect("Samus CPU adoption rejected");
        session_native_adoption::reconcile_current(native_game)
            .expect("Session native result adoption rejected");
    }
}
pub unsafe fn DrawFrameToPpu(native_game: &mut Game) {
    let game = &mut *native_game;
    // This local capability is constructed only under this function's existing
    // exclusive, live-backend contract. No safe caller can construct it.
    struct NativeSettlement<'a>(&'a mut Game);
    impl crate::game::scene::settle::SettlementHost for NativeSettlement<'_> {
        type Error = std::convert::Infallible;

        fn reset_beam(&mut self) {
            unsafe {
                (*self.0.g_snes).vPos = 0;
                (*self.0.g_snes).hPos = (*self.0.g_snes).vPos;
            }
        }
        fn nmi_wanted(&self) -> bool {
            unsafe { (*(*self.0.g_snes).cpu).nmiWanted }
        }
        fn advance_at(&mut self, position: u16) -> Result<(), Self::Error> {
            unsafe {
                (*self.0.g_snes).hPos = position;
                self.0.execution().advance_beam();
            }
            Ok(())
        }
        fn vertical_irq_enabled(&self) -> bool {
            unsafe { (*self.0.g_snes).vIrqEnabled }
        }
        fn vertical_position(&self) -> u16 {
            unsafe { (*self.0.g_snes).vPos }
        }
        fn vertical_timer(&self) -> u16 {
            unsafe { (*self.0.g_snes).vTimer }
        }
        fn interrupt(&mut self) -> Result<(), Self::Error> {
            unsafe { crate::game::scanline_irq::advance(self.0) };
            Ok(())
        }
        fn clear_nmi(&mut self) {
            unsafe { (*(*self.0.g_snes).cpu).nmiWanted = false_0 != 0 };
        }
    }
    game.preflight_samus_runtime()
        .expect("Samus settlement entry rejected");
    let mut mutation = game
        .begin_samus_mutation()
        .expect("Samus mutation rejected");
    match crate::game::scene::settle::settle(&mut NativeSettlement(mutation.game())) {
        Ok(()) => (),
        Err(never) => match never {},
    }
    mutation
        .finish_current()
        .expect("Samus settlement adoption rejected");
    session_native_adoption::reconcile_current(native_game)
        .expect("Session native result adoption rejected");
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
pub unsafe fn RunOneFrameOfGame_Both(native_game: &mut Game) {
    let game = &mut *native_game;
    unsafe {
        if !game.supports_legacy_persistence() {
            eprintln!("RunOneFrameOfGame_Both: CPU authority is unavailable for a prepared world");
            return;
        }
        game.preflight_samus_runtime()
            .expect("Samus paired entry rejected");
        let mut frame_mutation = game
            .begin_samus_mutation()
            .expect("Samus mutation rejected");
        let game = frame_mutation.game();
        (*game.g_snes).ppu = (*game.g_snes).snes_ppu;
        {
            let snes = game.g_snes;
            let cpu = game.g_cpu;
            let player = game.g_spc_player;
            let (backend, ram, _, _, before, owner) = game.ram_and_owned_snapshots();
            MakeSnapshotWithPlayerOwner(before, &*snes, &*cpu, player, owner, ram, backend);
        }
        game.actor_snapshots.before = Some(game.capture_actor_snapshot());
        let samus_before = game.capture_samus_snapshot();
        let session_before =
            session_native_adoption::capture(game).expect("Session before capture rejected");
        let mut selected_session = None;
        let queue_before = crate::audio_snapshot::ApuQueueSnapshot::capture(game);
        let apu_before = queue_before.pending;
        let apu_queue_before = queue_before.size;
        loop {
            queue_before.restore(game);
            game.restore_native_player_snapshot(PlayerSnapshotSource::Before);
            game.g_snapshot_theirs.apu_upload_count = 0;
            game.g_snapshot_theirs.apu_startup_upload = false;
            (*game.g_snes).runningWhichVersion = 1;
            RunOneFrameOfGame_Emulated(game);
            DrawFrameToPpu(game);
            {
                let snes = game.g_snes;
                let cpu = game.g_cpu;
                let player = game.g_spc_player;
                let (backend, ram, _, theirs, _, owner) = game.ram_and_owned_snapshots();
                MakeSnapshotWithPlayerOwner(theirs, &*snes, &*cpu, player, owner, ram, backend);
            }
            game.actor_snapshots.theirs = Some(game.capture_actor_snapshot());
            let samus_theirs = game.capture_samus_snapshot();
            let mut session_theirs = session_native_adoption::capture(game)
                .expect("Session oracle result capture rejected");
            let queue_theirs = crate::audio_snapshot::ApuQueueSnapshot::capture(game);
            let apu_theirs = queue_theirs.pending;
            queue_before.restore(game);
            game.restore_native_player_snapshot(PlayerSnapshotSource::Before);
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
            let samus_restore = game
                .prepare_samus_adoption(&samus_before)
                .expect("Samus before restore rejected");
            {
                let mut restore = game
                    .begin_samus_mutation()
                    .expect("Samus mutation rejected");
                let game = restore.game();
                let snes = game.g_snes;
                let cpu = game.g_cpu;
                let (backend, ram, _, _, before, _) = game.ram_and_owned_snapshots();
                RestoreSnapshot(before, &mut *snes, &mut *cpu, ram, backend);
                actor_restore.commit(game.actor_identity_mut());
                restore
                    .finish(samus_restore)
                    .expect("Samus before adoption rejected");
            }
            session_native_adoption::adopt(game, &session_before)
                .expect("Session before restore rejected");
            (*game.g_snes).runningWhichVersion = 2;
            gameplay::advance(Entry::Frame, game);
            DrawFrameToPpu(game);
            let queue_mine = crate::audio_snapshot::ApuQueueSnapshot::capture(game);
            {
                let snes = game.g_snes;
                let cpu = game.g_cpu;
                let player = game.g_spc_player;
                let (backend, ram, mine, _, _, owner) = game.ram_and_owned_snapshots();
                MakeSnapshotWithPlayerOwner(mine, &*snes, &*cpu, player, owner, ram, backend);
            }
            game.actor_snapshots.mine = Some(game.capture_actor_snapshot());
            let _samus_mine = game.capture_samus_snapshot();
            let mut session_mine = session_native_adoption::capture(game)
                .expect("Session Rust result capture rejected");
            (*game.g_snes).runningWhichVersion = 0xff;
            VerifySnapshotsEq(
                &raw mut *game.g_snapshot_mine,
                &raw mut *game.g_snapshot_theirs,
                &raw mut *game.g_snapshot_before,
                game,
                &mut session_mine,
                &mut session_theirs,
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
                let samus_restore = game
                    .prepare_samus_adoption(&samus_before)
                    .expect("Samus recovery restore rejected");
                {
                    let mut restore = game
                        .begin_samus_mutation()
                        .expect("Samus mutation rejected");
                    let game = restore.game();
                    let snes = game.g_snes;
                    let cpu = game.g_cpu;
                    let (backend, ram, _, _, before, _) = game.ram_and_owned_snapshots();
                    RestoreSnapshot(before, &mut *snes, &mut *cpu, ram, backend);
                    actor_restore.commit(game.actor_identity_mut());
                    restore
                        .finish(samus_restore)
                        .expect("Samus recovery adoption rejected");
                }
                session_native_adoption::adopt(game, &session_before)
                    .expect("Session recovery before restore rejected");
                if game.g_use_my_apu_code {
                    game.restore_native_player_snapshot(PlayerSnapshotSource::Mine);
                } else {
                    game.restore_native_player_snapshot(PlayerSnapshotSource::Theirs);
                }
                if game.g_debug_flag {
                    continue;
                }
                SaveBugSnapshot(game);
                {
                    let player = game.g_spc_player;
                    let (_, _, mine, _, _, owner) = game.ram_and_owned_snapshots();
                    mine.player.capture_with_owner(owner, player);
                }
                // Saving music copies the custom player's state into the physical APU.
                game.g_snapshot_before.audio.restore(game.g_snes);
                queue_before.restore(game);
                game.restore_native_player_snapshot(PlayerSnapshotSource::Before);
                RunOneFrameOfGame_Emulated(game);
                DrawFrameToPpu(game);
                let _samus_recovery = game.capture_samus_snapshot();
                let session_recovery = session_native_adoption::capture(game)
                    .expect("Session rerun result capture rejected");
                if game.g_use_my_apu_code {
                    game.restore_native_player_snapshot(PlayerSnapshotSource::Mine);
                }
                let queue_rerun = crate::audio_snapshot::ApuQueueSnapshot::capture(game);
                queue_mine.select_runtime(queue_rerun, game);
                selected_session = Some(session_recovery);
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
                let samus_restore = game
                    .prepare_samus_adoption(&samus_theirs)
                    .expect("Samus selected restore rejected");
                {
                    let mut restore = game
                        .begin_samus_mutation()
                        .expect("Samus mutation rejected");
                    let game = restore.game();
                    let snes = game.g_snes;
                    let cpu = game.g_cpu;
                    let (backend, ram, _, theirs, _, _) = game.ram_and_owned_snapshots();
                    RestoreSnapshot(theirs, &mut *snes, &mut *cpu, ram, backend);
                    actor_restore.commit(game.actor_identity_mut());
                    restore
                        .finish(samus_restore)
                        .expect("Samus selected adoption rejected");
                }
                if game.g_use_my_apu_code {
                    game.restore_native_player_snapshot(PlayerSnapshotSource::Mine);
                } else {
                    if !failed {
                        // Continue the independently verified Rust audio branch.
                        game.g_snapshot_mine.audio.restore(game.g_snes);
                        queue_mine.restore(game);
                    }
                    game.restore_native_player_snapshot(PlayerSnapshotSource::Theirs);
                }
                selected_session = Some(session_theirs);
                break;
            }
        }
        let mut selected_session = selected_session.expect("selected Session result missing");
        (*game.g_snes).ppu = if game.g_other_image as i32 != 0 {
            (*game.g_snes).my_ppu
        } else {
            (*game.g_snes).snes_ppu
        };
        (*game.g_snes).runningWhichVersion = 0;
        if word!(game.game_state) as i32 == 0xffff {
            (*(*game.g_snes).cpu).k = 0x80;
            (*(*game.g_snes).cpu).pc = 0x8459;
            game.ram().write_byte(0x611, 3);
            selected_session
                .final_continuation(0x611, 3)
                .expect("Session final continuation rejected");
        }
        if word!(game.menu_index) & 0xff00 != 0 {
            printf(c"MENU INDEX TOO BIG!\n".as_ptr());
            SaveBugSnapshot(game);
            word!(game.menu_index &= 0xff);
        }
        if game.g_got_mismatch_count != 0 {
            game.g_got_mismatch_count -= 1;
        }
        // Publish only after comparison normalization, actual branch restoration,
        // owner-specific audio selection and the final continuation/menu writes.
        frame_mutation
            .finish_current()
            .expect("Samus normalized adoption rejected");
        session_native_adoption::adopt(native_game, &selected_session)
            .expect("Session normalized selected adoption rejected");
    }
}
pub unsafe fn RtlRunFrameCompare(input: u16, _run_what: i32, native_game: &mut Game) {
    let game = &mut *native_game;
    unsafe {
        if !game.supports_legacy_persistence() && u32::from(game.g_runmode) != RM_MINE {
            eprintln!("RtlRunFrameCompare: CPU authority is unavailable for a prepared world");
            return;
        }
        game.preflight_samus_runtime()
            .expect("Samus frame entry rejected");
        let mut mutation = game
            .begin_samus_mutation()
            .expect("Samus mutation rejected");
        let game = mutation.game();
        game.input1_mut().currentState = input;
        if game.g_runmode as i32 == RM_THEIRS as i32 {
            RunOneFrameOfGame_Emulated(game);
            DrawFrameToPpu(game);
        } else if game.g_runmode as i32 == RM_MINE as i32 {
            (*game.g_snes).runningWhichVersion = 0xff;
            gameplay::advance(Entry::Frame, game);
            DrawFrameToPpu(game);
            (*game.g_snes).runningWhichVersion = 0;
        } else {
            RunOneFrameOfGame_Both(game);
        };
        mutation
            .finish_current()
            .expect("Samus frame adoption rejected");
        session_native_adoption::reconcile_current(native_game)
            .expect("Session native result adoption rejected");
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

#[cfg(sm_verify_dll)]
#[path = "verification/player_update_cases.rs"]
pub mod player_update_cases;
