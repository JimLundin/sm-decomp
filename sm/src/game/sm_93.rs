pub use crate::game::constants::projectile_types::*;
use crate::game::sm_80::{InvalidInterrupt_Crash, QueueSfx2_Max6};
use crate::game::sm_81::DrawProjectileSpritemap;
use crate::game::sm_8b::CalcExplosion_Mode7;
use crate::game::sm_90::{
    ClearProjectile, HandleProjectileTrails, Samus_DrawShinesparkCrashEchoProjectiles,
};
use crate::game_state::Game;
use crate::ram_map::ram_ptr;
use crate::ram_map::{ram_word, set_ram_word};
use crate::rtl::{RomPtr, Unreachable};
use ::core::ptr::null_mut;

pub use crate::types::{Point16U, VoidP};
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct ProjectileDamagesAndInstrPtr {
    pub damages: u32,
    pub instr_ptr: VoidP,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct ProjectileDataTable {
    pub damage: u16,
    pub instr_ptrs: [u16; 10],
}
pub use crate::types::{ProjectileInstr, ProjectileType};
const RomPtr_93: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x93>;
#[inline]
unsafe fn get_ProjectileDataTable(a: u16, game: &mut Game) -> *mut ProjectileDataTable {
    RomPtr((0x930000u32) | ((a) as u32), game) as *mut ProjectileDataTable
}
#[inline]
unsafe fn get_ProjectileInstr(a: u16, game: &mut Game) -> *mut ProjectileInstr {
    RomPtr((0x930000u32) | ((a) as u32), game) as *mut ProjectileInstr
}
pub unsafe fn InitializeProjectile(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    let r18: i32 = ram_word!(game, projectile_dir[v1]) as i32 & 0xfi32;
    let mut v2: u16 = ram_word!(game, projectile_type[v1]);
    let mut v3: u16 = 0;
    if (v2 & 0xf00) != 0 {
        v3 = crate::rtl::rom_table_word!(
            game,
            0x9383f1,
            (*(&raw mut v2 as *mut u8).offset(1isize) as i32 & 0xfi32) as isize
        );
    } else if (v2 & 0x10) != 0 {
        v3 = crate::rtl::rom_table_word!(
            game,
            0x9383d9,
            (ram_word!(game, projectile_type[v1]) as i32 & 0xfi32) as isize
        );
    } else {
        v3 = crate::rtl::rom_table_word!(
            game,
            0x9383c1,
            (ram_word!(game, projectile_type[v1]) as i32 & 0xfi32) as isize
        );
    }
    let PD = get_ProjectileDataTable(v3, game);
    if (*PD).damage & 0x8000 != 0 {
        InvalidInterrupt_Crash(game);
    }
    set_ram_word!(game.projectile_damage[v1] = (*PD).damage);
    let v7: u16 = (*PD).instr_ptrs[r18 as usize];
    set_ram_word!(game.projectile_bomb_instruction_ptr[v1] = v7);
    let v8 = RomPtr_93(v7, game);
    set_ram_word!(game.projectile_x_radius[v1] = *v8.offset(4) as u16);
    set_ram_word!(game.projectile_y_radius[v1] = *v8.offset(5) as u16);
    set_ram_word!(game.projectile_bomb_instruction_timers[v1] = 1u16);
}
pub unsafe fn InitializeInstrForSuperMissile(v0: u16, game: &mut Game) {
    let v1: i32 = v0 as i32 >> 1i32;
    let v3 = RomPtr_93(
        crate::rtl::rom_table_word!(
            game,
            0x93842b,
            (*((ram_ptr!(game, u16; 0xc18)).offset(v1 as isize) as *mut u8).offset(1isize) as i32
                & 0xfi32) as isize,
        ),
        game,
    );
    let v4: u16 = *(v3 as *mut u16);
    set_ram_word!(game.projectile_damage[v1] = v4);
    if v4 & 0x8000 != 0 {
        InvalidInterrupt_Crash(game);
    }
    set_ram_word!(game.projectile_bomb_instruction_ptr[v1] = *(v3.offset(2) as *mut u16));
    set_ram_word!(game.projectile_bomb_instruction_timers[v1] = 1u16);
}
pub unsafe fn InitializeInstrForMissile(v0: u16, game: &mut Game) {
    let v1: i32 = v0 as i32 >> 1i32;
    let v3 = RomPtr_93(
        crate::rtl::rom_table_word!(
            game,
            0x9383f1,
            (*((ram_ptr!(game, u16; 0xc18)).offset(v1 as isize) as *mut u8).offset(1isize) as i32
                & 0xfi32) as isize,
        ),
        game,
    );
    let v4: u16 = *(v3 as *mut u16);
    set_ram_word!(game.projectile_damage[v1] = *(v3 as *mut u16));
    if v4 & 0x8000 != 0 {
        InvalidInterrupt_Crash(game);
    }
    set_ram_word!(game.projectile_bomb_instruction_ptr[v1] = *(v3.offset(2) as *mut u16));
    set_ram_word!(game.projectile_bomb_instruction_timers[v1] = 1u16);
}
pub unsafe fn KillProjectileInner(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    if (ram_word!(game, projectile_type[v1]) & 0xf00) != 0 {
        if ram_word!(game, cinematic_function) == 0 {
            QueueSfx2_Max6(7u16, game);
        }
        let v2: u16 = ram_word!(game, projectile_type[v1]);
        set_ram_word!(game.projectile_type[v1] = (v2 & 0xf0ff | 0x800));
        if (v2 & 0x200) != 0 {
            set_ram_word!(
                game.projectile_bomb_instruction_ptr[v1] =
                    *(&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x938691))
                        as *mut ProjectileDamagesAndInstrPtr))
                        .damages as *mut u16)
                        .offset(1isize,)
            );
            set_ram_word!(game.earthquake_type = 20u16);
            set_ram_word!(game.earthquake_timer = 30u16);
        } else {
            set_ram_word!(
                game.projectile_bomb_instruction_ptr[v1] =
                    *(&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x938679))
                        as *mut ProjectileDamagesAndInstrPtr))
                        .instr_ptr)
                        .offset(1)
            );
        }
        if ram_word!(game, cooldown_timer) as i32 - 21i32 & 0x8000i32 == 0 {
            set_ram_word!(game.cooldown_timer = 20u16);
        }
    } else {
        set_ram_word!(
            game.projectile_type[v1] = (ram_word!(game, projectile_type[v1]) & 0xf0ff | 0x700)
        );
        set_ram_word!(
            game.projectile_bomb_instruction_ptr[v1] =
                *(&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x938679))
                    as *mut ProjectileDamagesAndInstrPtr))
                    .damages as *mut u16)
                    .offset(1isize,)
        );
        QueueSfx2_Max6(0xcu16, game);
    }
    set_ram_word!(game.projectile_bomb_instruction_timers[v1] = 1u16);
    set_ram_word!(game.projectile_damage[v1] = 8u16);
}
pub unsafe fn InitializeBombExplosion(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    set_ram_word!(
        game.projectile_bomb_instruction_ptr[v1] =
            *(&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x938681))
                as *mut ProjectileDamagesAndInstrPtr))
                .damages as *mut u16)
                .offset(1isize,)
    );
    set_ram_word!(game.projectile_bomb_instruction_timers[v1] = 1u16);
}
pub unsafe fn InitializeShinesparkEchoOrSpazerSba(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    let r18: i32 = ram_word!(game, projectile_dir[v1]) as i32 & 0xfi32;
    let PD = get_ProjectileDataTable(
        crate::rtl::rom_table_word!(
            game,
            0x938403,
            (*((ram_ptr!(game, u16; 0xc18)).offset(v1 as isize) as *mut u8).offset(0) as i32
                - 34i32) as isize,
        ),
        game,
    );
    set_ram_word!(game.projectile_damage[v1] = (*PD).damage);
    if (*PD).damage & 0x8000 != 0 {
        InvalidInterrupt_Crash(game);
    }
    set_ram_word!(game.projectile_bomb_instruction_ptr[v1] = (*PD).instr_ptrs[r18 as usize]);
    set_ram_word!(game.projectile_bomb_instruction_timers[v1] = 1u16);
}
pub unsafe fn InitializeSbaProjectile(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    let v2 = RomPtr_93(
        crate::rtl::rom_table_word!(
            game,
            0x938413,
            (ram_word!(game, projectile_type[v1]) as i32 & 0xfi32) as isize
        ),
        game,
    );
    let v3: u16 = *(v2 as *mut u16);
    set_ram_word!(game.projectile_damage[v1] = v3);
    if v3 & 0x8000 != 0 {
        InvalidInterrupt_Crash(game);
    }
    set_ram_word!(game.projectile_bomb_instruction_ptr[v1] = *(v2.offset(2) as *mut u16));
    set_ram_word!(game.projectile_bomb_instruction_timers[v1] = 1u16);
}
pub unsafe fn ProjectileInsts_GetValue(k: u16, game: &mut Game) -> u16 {
    let ip: i32 = ram_word!(game, projectile_bomb_instruction_ptr[(k >> 1)]) as i32;
    let delta: i32 = if ram_word!(game, projectile_bomb_instruction_timers[(k >> 1)]) == 1u16
        && (*get_ProjectileInstr(ip as u16, game)).timer & 0x8000 == 0
    {
        0i32
    } else {
        -8i32
    };
    (*get_ProjectileInstr((ip + delta) as u16, game)).field_6
}
crate::game::address_dispatch! {
    pub unsafe fn CallProj93Instr(ea: u32, j: u16, k: u16, game: &mut Game) -> u16 => ea;
        call (j, k, game); default Unreachable(game) as u16;
    {
        9667119 => Proj93Instr_Delete; 9667129 => Proj93Instr_Goto;
    }
}
pub unsafe fn RunProjectileInstructions(game: &mut Game) {
    let mut PI = null_mut::<ProjectileInstr>();
    let v0: u16 = ram_word!(game, projectile_index);
    let v1: i32 = ram_word!(game, projectile_index) as i32 >> 1i32;
    crate::update_place! { reference fresh0 [fresh1] = *(ram_ptr!(game, u16; 0xc54)).offset(v1 as isize) => (*fresh0).wrapping_sub(1); }
    if fresh1 == 1u16 {
        let mut v3: u16 = ram_word!(game, projectile_bomb_instruction_ptr[v1]);
        loop {
            PI = get_ProjectileInstr(v3, game);
            if ((*PI).timer & 0x8000) == 0 {
                break;
            }
            v3 = CallProj93Instr(
                (((*PI).timer) as u32) | (0x930000u32),
                v0,
                (v3).wrapping_add(2u16),
                game,
            );
            if v3 == 0 {
                return;
            }
        }
        set_ram_word!(game.projectile_bomb_instruction_timers[v1] = (*PI).timer);
        set_ram_word!(game.projectile_spritemap_pointers[v1] = (*PI).spritemap_ptr);
        set_ram_word!(game.projectile_x_radius[v1] = (*PI).x_radius as u16);
        set_ram_word!(game.projectile_y_radius[v1] = (*PI).y_radius as u16);
        set_ram_word!(game.projectile_bomb_instruction_ptr[v1] = (v3).wrapping_add(8u16));
    }
}
pub unsafe fn Proj93Instr_Delete(k: u16, j: u16, game: &mut Game) -> u16 {
    ClearProjectile(k, game);
    0u16
}
pub unsafe fn Proj93Instr_Goto(k: u16, j: u16, game: &mut Game) -> u16 {
    *(RomPtr_93(j, game) as *mut u16)
}
pub unsafe fn DrawPlayerExplosions2(game: &mut Game) {
    let mut skip_explosion_draw: bool;
    let mut v0: u16 = 8u16;
    let mut r18: u16 = 0;
    let mut r20: u16 = 0;
    set_ram_word!(game.projectile_index = 8u16);
    loop {
        let v1: i32 = v0 as i32 >> 1i32;
        if (ram_word!(game, projectile_bomb_instruction_ptr[v1]) != 0) {
            let mut v2: u16 = 0;
            let mut v3: u16 = 0;
            v2 = ram_word!(game, projectile_type[v1]);
            skip_explosion_draw = if v2 as i32 & (kProjectileType_TypeMask as i32 | 0x10i32) != 0i32
            {
                ((v2 as i32 & 0xf00i32) - 768i32 & 0x8000i32 == 0)
            } else if (v2 & 0xc) != 0 {
                if (v0 & 2) != 0 {
                    ((ram_word!(game, nmi_frame_counter_word) & 2) == 0)
                } else {
                    ((ram_word!(game, nmi_frame_counter_word) & 2) != 0)
                }
            } else if (v0 & 2) != 0 {
                ((ram_word!(game, nmi_frame_counter_word) & 1) != 0)
            } else {
                ((ram_word!(game, nmi_frame_counter_word) & 1) == 0)
            };
            if skip_explosion_draw {
            } else {
                if (ram_word!(game, ceres_status) & 0x8000) == 0 {
                    r20 = (ram_word!(game, projectile_x_pos[v1]))
                        .wrapping_sub(ram_word!(game, layer1_x_pos));
                    v3 = (ram_word!(game, projectile_y_pos[v1]))
                        .wrapping_sub(ram_word!(game, layer1_y_pos));
                    r18 = v3;
                } else {
                    let pt: Point16U = CalcExplosion_Mode7(v0, game);
                    v3 = pt.y;
                    r20 = pt.x;
                }
                if (v3 & 0xff00) == 0
                    && (ram_word!(game, projectile_spritemap_pointers[v1]) & 0x8000) != 0
                {
                    DrawProjectileSpritemap(v0, r20, r18, game);
                }
                v0 = ram_word!(game, projectile_index);
            }
        }
        v0 = (v0).wrapping_sub(2u16);
        set_ram_word!(game.projectile_index = v0);
        if ((v0 & 0x8000) != 0) {
            break;
        }
    }
    Samus_DrawShinesparkCrashEchoProjectiles(game);
    HandleProjectileTrails(game);
}
pub unsafe fn DrawBombAndProjectileExplosions(game: &mut Game) {
    let mut current_block: u64;
    let mut v0: u16 = 18u16;
    let mut v2: u16 = 0;
    let mut r18: u16 = 0;
    let mut r20: u16 = 0;
    set_ram_word!(game.projectile_index = 18u16);
    loop {
        let v1: i32 = v0 as i32 >> 1i32;
        if !(ram_word!(game, projectile_bomb_instruction_ptr[v1]) == 0
            || (((ram_word!(game, projectile_type[v1]) as i32 & 0xf00i32) - 768i32) as i16) < 0i16)
        {
            if (ram_word!(game, projectile_type[v1]) & 0xf00) == 768 {
                if ram_word!(game, projectile_variables[v1]) == 0 {
                    current_block = 2612600445040741303;
                } else {
                    current_block = 11875828834189669668;
                }
            } else if (ram_word!(game, projectile_type[v1]) & 0xf00) == 1280
                || (ram_word!(game, ceres_status) & 0x8000) == 0
            {
                current_block = 11875828834189669668;
            } else {
                CalcExplosion_Mode7(v0, game);
                v2 = r18;
                current_block = 2311960645627505036;
            }
            if current_block == 2612600445040741303 {
            } else {
                if current_block == 11875828834189669668 {
                    let v3: u16 = (ram_word!(game, projectile_x_pos[v1]))
                        .wrapping_sub(ram_word!(game, layer1_x_pos));
                    r20 = v3;
                    if v3 as i32 - 304i32 & 0x8000i32 == 0 || v3 as i32 + 48i32 & 0x8000i32 != 0 {
                        current_block = 2612600445040741303;
                    } else {
                        v2 = (ram_word!(game, projectile_y_pos[v1]))
                            .wrapping_sub(ram_word!(game, layer1_y_pos));
                        r18 = v2;
                        current_block = 2311960645627505036;
                    }
                }
                if current_block == 2612600445040741303 {
                } else {
                    if ((v2 & 0xff00) == 0) {
                        DrawProjectileSpritemap(v0, r20, r18, game);
                    }
                    v0 = ram_word!(game, projectile_index);
                }
            }
        }
        v0 = (v0).wrapping_sub(2u16);
        set_ram_word!(game.projectile_index = v0);
        if ((v0 & 0x8000) != 0) {
            break;
        }
    }
}
