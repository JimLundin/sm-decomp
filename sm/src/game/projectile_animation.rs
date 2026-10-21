//! Player projectile animation data, instruction playback and drawing.
pub use crate::game::prelude::*;

use crate::game::samus_animation::Samus_DrawShinesparkCrashEchoProjectiles;
use crate::game::sm_80::{InvalidInterrupt_Crash, QueueSfx2_Max6};
use crate::game::sm_81::DrawProjectileSpritemap;
use crate::game::sm_8b::CalcExplosion_Mode7;
use crate::game::sm_90::{ClearProjectile, HandleProjectileTrails};

crate::typed_struct! {
    [
        #[derive(Copy, Clone)]
        #[repr(C, packed)]
    ];
    pub struct ProjectileDamagesAndInstrPtr {
        pub damages: u32;
        pub instr_ptr: VoidP;
    }

    pub struct ProjectileDataTable {
        pub damage: u16;
        pub instr_ptrs: [u16; 10];
    }
}

pub use crate::types::ProjectileInstr;
const RomPtr_93: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x93>;
#[inline]
unsafe fn get_ProjectileDataTable(a: u16, game: &mut Game) -> *mut ProjectileDataTable {
    RomPtr((0x930000u32) | ((a) as u32), game) as *mut ProjectileDataTable
}
#[inline]
unsafe fn get_ProjectileInstr(a: u16, game: &mut Game) -> *mut ProjectileInstr {
    RomPtr((0x930000u32) | ((a) as u32), game) as *mut ProjectileInstr
}
unsafe fn initialize_fixed(k: u16, table: u32, type_shift: u32, game: &mut Game) {
    let slot = (k >> 1) as usize;
    let selector = (word!(game.projectile_type[slot]) >> type_shift) & 0xf;
    let definition = game
        .g_rom
        .offset(crate::rtl::lorom_offset(table))
        .cast::<u16>()
        .add(selector as usize)
        .read_unaligned();
    let data = RomPtr_93(definition, game);
    let damage = *(data as *const u16);
    word!(game.projectile_damage[slot] = damage);
    if damage & 0x8000 != 0 {
        InvalidInterrupt_Crash(game);
    }
    word!(game.projectile_bomb_instruction_ptr[slot] = *(data.add(2) as *const u16));
    word!(game.projectile_bomb_instruction_timers[slot] = 1);
}

pub unsafe fn initialize_super_missile(k: u16, game: &mut Game) {
    initialize_fixed(k, 0x93842b, 8, game);
}
pub unsafe fn initialize_bomb(k: u16, game: &mut Game) {
    initialize_fixed(k, 0x9383f1, 8, game);
}
pub unsafe fn initialize_sba(k: u16, game: &mut Game) {
    initialize_fixed(k, 0x938413, 0, game);
}

crate::prefixed_callbacks! {
    pub unsafe fn (k: u16, game: &mut Game);
    @prefix { let v1: i32 = k as i32 >> 1; };
    initialize_projectile [] {
        let r18: i32 = word!(game.projectile_dir[v1]) as i32 & 0xf;
        let mut v2: u16 = word!(game.projectile_type[v1]);
        let mut v3: u16 = if (v2 & 0xf00) != 0 {
            crate::rtl::rom_table_word!(
                game,
                0x9383f1,
                (*(&raw mut v2 as *mut u8).offset(1) as i32 & 0xf) as isize
            )
        } else if (v2 & 0x10) != 0 {
            crate::rtl::rom_table_word!(
                game,
                0x9383d9,
                (word!(game.projectile_type[v1]) as i32 & 0xf) as isize
            )
        } else {
            crate::rtl::rom_table_word!(
                game,
                0x9383c1,
                (word!(game.projectile_type[v1]) as i32 & 0xf) as isize
            )
        };
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
    };
    start_impact [] {
        if (word!(game.projectile_type[v1]) & 0xf00) != 0 {
            if word!(game.cinematic_function) == 0 {
                QueueSfx2_Max6(7, game);
            }
            let v2: u16 = word!(game.projectile_type[v1]);
            word!(game.projectile_type[v1] = (v2 & 0xf0ff | 0x800));
            if (v2 & 0x200) != 0 {
                word!(
                    game.projectile_bomb_instruction_ptr[v1] =
                        *(&raw mut (*(rom_ptr!(game, ProjectileDamagesAndInstrPtr; 0x938691))).damages
                            as *mut u16)
                            .offset(1,)
                );
                word!(game.earthquake_type = 20);
                word!(game.earthquake_timer = 30);
            } else {
                word!(
                    game.projectile_bomb_instruction_ptr[v1] =
                        *(&raw mut (*(rom_ptr!(game, ProjectileDamagesAndInstrPtr; 0x938679)))
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
                    *(&raw mut (*(rom_ptr!(game, ProjectileDamagesAndInstrPtr; 0x938679))).damages
                        as *mut u16)
                        .offset(1,)
            );
            QueueSfx2_Max6(0xc, game);
        }
        word! {game; projectile_bomb_instruction_timers[v1] = 1; projectile_damage[v1] = 8; }
    };
    start_bomb_explosion [] {
        word!(
            game.projectile_bomb_instruction_ptr[v1] =
                *(&raw mut (*(rom_ptr!(game, ProjectileDamagesAndInstrPtr; 0x938681))).damages
                    as *mut u16)
                    .offset(1,)
        );
        word!(game.projectile_bomb_instruction_timers[v1] = 1);
    };
    initialize_echo [] {
        let r18: i32 = word!(game.projectile_dir[v1]) as i32 & 0xf;
        let PD = get_ProjectileDataTable(
            crate::rtl::rom_table_word!(
                game,
                0x938403,
                (*((ram_ptr!(game, u16; 0xc18)).offset(v1 as isize) as *mut u8) as i32 - 34i32)
                    as isize,
            ),
            game,
        );
        word!(game.projectile_damage[v1] = (*PD).damage);
        if (*PD).damage & 0x8000 != 0 {
            InvalidInterrupt_Crash(game);
        }
        word!(game.projectile_bomb_instruction_ptr[v1] = (*PD).instr_ptrs[r18 as usize]);
        word!(game.projectile_bomb_instruction_timers[v1] = 1);
    };

}
pub unsafe fn trail_frame(k: u16, game: &mut Game) -> u16 {
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
crate::game::callback_dispatch! {
    pub unsafe fn dispatch_instruction(ea: u32, j: u16, k: u16, game: &mut Game) -> u16 => ea;
    call (j, k, game); default Unreachable(game) as u16;
    @functions [pub unsafe fn (k: u16, j: u16, game: &mut Game) -> u16];
    [9667119] instruction_delete [0] { ClearProjectile(k, game); };
    [9667129] instruction_goto [*(RomPtr_93(j, game) as *mut u16)] { };
}
pub unsafe fn advance(game: &mut Game) {
    let mut PI = null_mut::<ProjectileInstr>();
    let v0: u16 = word!(game.projectile_index);
    let v1: i32 = word!(game.projectile_index) as i32 >> 1;
    crate::update_place! { unaligned_word fresh0 [fresh1] = &raw mut (*(ram_ptr!(game, u16; 0xc54)).offset(v1 as isize)) => (u16::from_le(fresh0.read_unaligned())).wrapping_sub(1); }
    if fresh1 == 1 {
        let mut v3: u16 = word!(game.projectile_bomb_instruction_ptr[v1]);
        loop {
            PI = get_ProjectileInstr(v3, game);
            if ((*PI).timer & 0x8000) == 0 {
                break;
            }
            v3 = dispatch_instruction(
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
unsafe fn draw_slots<const EXPLOSIONS: bool>(game: &mut Game) {
    let mut index: u16 = if EXPLOSIONS { 18 } else { 8 };
    let (mut x, mut y) = (0u16, 0u16);
    word!(game.projectile_index = index);
    loop {
        let slot = (index >> 1) as usize;
        if word!(game.projectile_bomb_instruction_ptr[slot]) != 0 {
            let kind = word!(game.projectile_type[slot]);
            let visible = if EXPLOSIONS {
                kind & 0xf00 >= 0x300
                    && (kind & 0xf00 != 0x300 || word!(game.projectile_variables[slot]) != 0)
            } else if kind & 0xf10 != 0 {
                kind & 0xf00 < 0x300
            } else if kind & 0xc != 0 {
                (index & 2 != 0) == (word!(game.nmi_frame_counter_word) & 2 != 0)
            } else {
                (index & 2 == 0) == (word!(game.nmi_frame_counter_word) & 1 != 0)
            };
            if visible {
                'draw: {
                    let mode7 = word!(game.ceres_status) & 0x8000 != 0
                        && (!EXPLOSIONS || (kind & 0xf00 != 0x300 && kind & 0xf00 != 0x500));
                    let clip_y = if mode7 {
                        let point = CalcExplosion_Mode7(index, game);
                        // Ceres projectiles clip transformed Y but draw with the previous Y.
                        // Ceres explosions discard both transformed coordinates.
                        if EXPLOSIONS {
                            y
                        } else {
                            x = point.x;
                            point.y
                        }
                    } else {
                        x = word!(game.projectile_x_pos[slot])
                            .wrapping_sub(word!(game.layer1_x_pos));
                        if EXPLOSIONS
                            && (x as i32 - 304 & 0x8000 == 0 || x as i32 + 48 & 0x8000 != 0)
                        {
                            break 'draw;
                        }
                        y = word!(game.projectile_y_pos[slot])
                            .wrapping_sub(word!(game.layer1_y_pos));
                        y
                    };
                    if clip_y & 0xff00 == 0
                        && (EXPLOSIONS
                            || word!(game.projectile_spritemap_pointers[slot]) & 0x8000 != 0)
                    {
                        DrawProjectileSpritemap(index, x, y, game);
                    }
                    index = word!(game.projectile_index);
                }
            }
        }
        index = index.wrapping_sub(2);
        word!(game.projectile_index = index);
        if index & 0x8000 != 0 {
            break;
        }
    }
}

pub unsafe fn draw_projectiles(game: &mut Game) {
    draw_slots::<false>(game);
    Samus_DrawShinesparkCrashEchoProjectiles(game);
    HandleProjectileTrails(game);
}
pub unsafe fn draw_explosions(game: &mut Game) {
    draw_slots::<true>(game);
}
