pub use crate::game::constants::projectile_types::*;
use crate::game::sm_80::{InvalidInterrupt_Crash, QueueSfx2_Max6};
use crate::game::sm_81::DrawProjectileSpritemap;
use crate::game::sm_8b::CalcExplosion_Mode7;
use crate::game::sm_90::{
    ClearProjectile, HandleProjectileTrails, Samus_DrawShinesparkCrashEchoProjectiles,
};
use crate::game_state::Game;
use crate::ram_map::ram_ptr;
use crate::ram_map::word;
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
    let v1: i32 = k as i32 >> 1;
    let r18: i32 = word!(game.projectile_dir[v1]) as i32 & 0xf;
    let mut v2: u16 = word!(game.projectile_type[v1]);
    let mut v3: u16 = 0;
    if (v2 & 0xf00) != 0 {
        v3 = crate::rtl::rom_table_word!(
            game,
            0x9383f1,
            (*(&raw mut v2 as *mut u8).offset(1) as i32 & 0xf) as isize
        );
    } else if (v2 & 0x10) != 0 {
        v3 = crate::rtl::rom_table_word!(
            game,
            0x9383d9,
            (word!(game.projectile_type[v1]) as i32 & 0xf) as isize
        );
    } else {
        v3 = crate::rtl::rom_table_word!(
            game,
            0x9383c1,
            (word!(game.projectile_type[v1]) as i32 & 0xf) as isize
        );
    }
    let PD = get_ProjectileDataTable(v3, game);
    if (*PD).damage & 0x8000 != 0 {
        InvalidInterrupt_Crash(game);
    }
    word!(game.projectile_damage[v1] = (*PD).damage);
    let v7: u16 = (*PD).instr_ptrs[r18 as usize];
    word!(game.projectile_bomb_instruction_ptr[v1] = v7);
    let v8 = RomPtr_93(v7, game);
    word!(game.projectile_x_radius[v1] = *v8.offset(4) as u16);
    word!(game.projectile_y_radius[v1] = *v8.offset(5) as u16);
    word!(game.projectile_bomb_instruction_timers[v1] = 1);
}
pub unsafe fn InitializeInstrForSuperMissile(v0: u16, game: &mut Game) {
    let v1: i32 = v0 as i32 >> 1;
    let v3 = RomPtr_93(
        crate::rtl::rom_table_word!(
            game,
            0x93842b,
            (*((ram_ptr!(game, u16; 0xc18)).offset(v1 as isize) as *mut u8).offset(1) as i32 & 0xf)
                as isize,
        ),
        game,
    );
    let v4: u16 = *(v3 as *mut u16);
    word!(game.projectile_damage[v1] = v4);
    if v4 & 0x8000 != 0 {
        InvalidInterrupt_Crash(game);
    }
    word!(game.projectile_bomb_instruction_ptr[v1] = *(v3.offset(2) as *mut u16));
    word!(game.projectile_bomb_instruction_timers[v1] = 1);
}
pub unsafe fn InitializeInstrForMissile(v0: u16, game: &mut Game) {
    let v1: i32 = v0 as i32 >> 1;
    let v3 = RomPtr_93(
        crate::rtl::rom_table_word!(
            game,
            0x9383f1,
            (*((ram_ptr!(game, u16; 0xc18)).offset(v1 as isize) as *mut u8).offset(1) as i32 & 0xf)
                as isize,
        ),
        game,
    );
    let v4: u16 = *(v3 as *mut u16);
    word!(game.projectile_damage[v1] = *(v3 as *mut u16));
    if v4 & 0x8000 != 0 {
        InvalidInterrupt_Crash(game);
    }
    word!(game.projectile_bomb_instruction_ptr[v1] = *(v3.offset(2) as *mut u16));
    word!(game.projectile_bomb_instruction_timers[v1] = 1);
}
pub unsafe fn KillProjectileInner(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    if (word!(game.projectile_type[v1]) & 0xf00) != 0 {
        if word!(game.cinematic_function) == 0 {
            QueueSfx2_Max6(7, game);
        }
        let v2: u16 = word!(game.projectile_type[v1]);
        word!(game.projectile_type[v1] = (v2 & 0xf0ff | 0x800));
        if (v2 & 0x200) != 0 {
            word!(
                game.projectile_bomb_instruction_ptr[v1] =
                    *(&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x938691))
                        as *mut ProjectileDamagesAndInstrPtr))
                        .damages as *mut u16)
                        .offset(1,)
            );
            word!(game.earthquake_type = 20);
            word!(game.earthquake_timer = 30);
        } else {
            word!(
                game.projectile_bomb_instruction_ptr[v1] =
                    *(&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x938679))
                        as *mut ProjectileDamagesAndInstrPtr))
                        .instr_ptr)
                        .offset(1)
            );
        }
        if word!(game.cooldown_timer) as i32 - 21i32 & 0x8000 == 0 {
            word!(game.cooldown_timer = 20);
        }
    } else {
        word!(game.projectile_type[v1] = (word!(game.projectile_type[v1]) & 0xf0ff | 0x700));
        word!(
            game.projectile_bomb_instruction_ptr[v1] =
                *(&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x938679))
                    as *mut ProjectileDamagesAndInstrPtr))
                    .damages as *mut u16)
                    .offset(1,)
        );
        QueueSfx2_Max6(0xc, game);
    }
    word!(game.projectile_bomb_instruction_timers[v1] = 1);
    word!(game.projectile_damage[v1] = 8);
}
pub unsafe fn InitializeBombExplosion(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    word!(
        game.projectile_bomb_instruction_ptr[v1] =
            *(&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x938681))
                as *mut ProjectileDamagesAndInstrPtr))
                .damages as *mut u16)
                .offset(1,)
    );
    word!(game.projectile_bomb_instruction_timers[v1] = 1);
}
pub unsafe fn InitializeShinesparkEchoOrSpazerSba(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    let r18: i32 = word!(game.projectile_dir[v1]) as i32 & 0xf;
    let PD = get_ProjectileDataTable(
        crate::rtl::rom_table_word!(
            game,
            0x938403,
            (*((ram_ptr!(game, u16; 0xc18)).offset(v1 as isize) as *mut u8).offset(0) as i32
                - 34i32) as isize,
        ),
        game,
    );
    word!(game.projectile_damage[v1] = (*PD).damage);
    if (*PD).damage & 0x8000 != 0 {
        InvalidInterrupt_Crash(game);
    }
    word!(game.projectile_bomb_instruction_ptr[v1] = (*PD).instr_ptrs[r18 as usize]);
    word!(game.projectile_bomb_instruction_timers[v1] = 1);
}
pub unsafe fn InitializeSbaProjectile(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    let v2 = RomPtr_93(
        crate::rtl::rom_table_word!(
            game,
            0x938413,
            (word!(game.projectile_type[v1]) as i32 & 0xf) as isize
        ),
        game,
    );
    let v3: u16 = *(v2 as *mut u16);
    word!(game.projectile_damage[v1] = v3);
    if v3 & 0x8000 != 0 {
        InvalidInterrupt_Crash(game);
    }
    word!(game.projectile_bomb_instruction_ptr[v1] = *(v2.offset(2) as *mut u16));
    word!(game.projectile_bomb_instruction_timers[v1] = 1);
}
pub unsafe fn ProjectileInsts_GetValue(k: u16, game: &mut Game) -> u16 {
    let ip: i32 = word!(game.projectile_bomb_instruction_ptr[(k >> 1)]) as i32;
    let delta: i32 = if word!(game.projectile_bomb_instruction_timers[(k >> 1)]) == 1
        && (*get_ProjectileInstr(ip as u16, game)).timer & 0x8000 == 0
    {
        0
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
    let v0: u16 = word!(game.projectile_index);
    let v1: i32 = word!(game.projectile_index) as i32 >> 1;
    crate::update_place! { reference fresh0 [fresh1] = *(ram_ptr!(game, u16; 0xc54)).offset(v1 as isize) => (*fresh0).wrapping_sub(1); }
    if fresh1 == 1 {
        let mut v3: u16 = word!(game.projectile_bomb_instruction_ptr[v1]);
        loop {
            PI = get_ProjectileInstr(v3, game);
            if ((*PI).timer & 0x8000) == 0 {
                break;
            }
            v3 = CallProj93Instr(
                (((*PI).timer) as u32) | (0x930000u32),
                v0,
                (v3).wrapping_add(2),
                game,
            );
            if v3 == 0 {
                return;
            }
        }
        word!(game.projectile_bomb_instruction_timers[v1] = (*PI).timer);
        word!(game.projectile_spritemap_pointers[v1] = (*PI).spritemap_ptr);
        word!(game.projectile_x_radius[v1] = (*PI).x_radius as u16);
        word!(game.projectile_y_radius[v1] = (*PI).y_radius as u16);
        word!(game.projectile_bomb_instruction_ptr[v1] = (v3).wrapping_add(8));
    }
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, j: u16, game: &mut Game) -> u16;
    Proj93Instr_Delete [0] { ClearProjectile(k, game); };
    Proj93Instr_Goto [*(RomPtr_93(j, game) as *mut u16)] { };
}
pub unsafe fn DrawPlayerExplosions2(game: &mut Game) {
    let mut skip_explosion_draw: bool;
    let mut v0: u16 = 8;
    let mut r18: u16 = 0;
    let mut r20: u16 = 0;
    word!(game.projectile_index = 8);
    loop {
        let v1: i32 = v0 as i32 >> 1;
        if (word!(game.projectile_bomb_instruction_ptr[v1]) != 0) {
            let mut v2: u16 = 0;
            let mut v3: u16 = 0;
            v2 = word!(game.projectile_type[v1]);
            skip_explosion_draw = if v2 as i32 & (kProjectileType_TypeMask as i32 | 0x10) != 0 {
                ((v2 as i32 & 0xf00) - 768i32 & 0x8000 == 0)
            } else if (v2 & 0xc) != 0 {
                if (v0 & 2) != 0 {
                    ((word!(game.nmi_frame_counter_word) & 2) == 0)
                } else {
                    ((word!(game.nmi_frame_counter_word) & 2) != 0)
                }
            } else if (v0 & 2) != 0 {
                ((word!(game.nmi_frame_counter_word) & 1) != 0)
            } else {
                ((word!(game.nmi_frame_counter_word) & 1) == 0)
            };
            if skip_explosion_draw {
            } else {
                if (word!(game.ceres_status) & 0x8000) == 0 {
                    r20 = (word!(game.projectile_x_pos[v1])).wrapping_sub(word!(game.layer1_x_pos));
                    v3 = (word!(game.projectile_y_pos[v1])).wrapping_sub(word!(game.layer1_y_pos));
                    r18 = v3;
                } else {
                    let pt: Point16U = CalcExplosion_Mode7(v0, game);
                    v3 = pt.y;
                    r20 = pt.x;
                }
                if (v3 & 0xff00) == 0
                    && (word!(game.projectile_spritemap_pointers[v1]) & 0x8000) != 0
                {
                    DrawProjectileSpritemap(v0, r20, r18, game);
                }
                v0 = word!(game.projectile_index);
            }
        }
        v0 = (v0).wrapping_sub(2);
        word!(game.projectile_index = v0);
        if ((v0 & 0x8000) != 0) {
            break;
        }
    }
    Samus_DrawShinesparkCrashEchoProjectiles(game);
    HandleProjectileTrails(game);
}
pub unsafe fn DrawBombAndProjectileExplosions(game: &mut Game) {
    let mut v0: u16 = 18;
    let mut v2: u16 = 0;
    let mut r18: u16 = 0;
    let mut r20: u16 = 0;
    word!(game.projectile_index = 18);
    loop {
        let v1: i32 = v0 as i32 >> 1;
        if !(word!(game.projectile_bomb_instruction_ptr[v1]) == 0
            || (((word!(game.projectile_type[v1]) as i32 & 0xf00) - 768i32) as i16) < 0)
        {
            'draw: {
                let mode7 = if (word!(game.projectile_type[v1]) & 0xf00) == 768 {
                    if word!(game.projectile_variables[v1]) == 0 {
                        break 'draw;
                    }
                    false
                } else if (word!(game.projectile_type[v1]) & 0xf00) == 1280
                    || (word!(game.ceres_status) & 0x8000) == 0
                {
                    false
                } else {
                    true
                };
                if mode7 {
                    CalcExplosion_Mode7(v0, game);
                    v2 = r18;
                } else {
                    let v3: u16 =
                        (word!(game.projectile_x_pos[v1])).wrapping_sub(word!(game.layer1_x_pos));
                    r20 = v3;
                    if v3 as i32 - 304i32 & 0x8000 == 0 || v3 as i32 + 48 & 0x8000 != 0 {
                        break 'draw;
                    }
                    v2 = (word!(game.projectile_y_pos[v1])).wrapping_sub(word!(game.layer1_y_pos));
                    r18 = v2;
                }
                if ((v2 & 0xff00) == 0) {
                    DrawProjectileSpritemap(v0, r20, r18, game);
                }
                v0 = word!(game.projectile_index);
            }
        }
        v0 = (v0).wrapping_sub(2);
        word!(game.projectile_index = v0);
        if ((v0 & 0x8000) != 0) {
            break;
        }
    }
}
