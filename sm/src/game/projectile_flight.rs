//! Launch speed and flight of ordinary beams, missiles and their linked extensions.
use crate::game::projectile_animation::initialize_super_missile;
use crate::game::sm_90::{ClearProjectile, InitProjectilePositionDirection};
use crate::game::sm_94::{
    BlockCollMissileHoriz, BlockCollMissileVert, BlockCollNoWaveBeamHoriz, BlockCollNoWaveBeamVert,
    BlockCollWaveBeamHoriz, BlockCollWaveBeamVert,
};
use crate::game_state::Game;
use crate::ram_map::{checked_word, ram_ptr, word};
use crate::rtl::{abs16, Unreachable};

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Flight {
    NonWave,
    Wave,
    Missile,
    SuperMissile,
}
const AXES: [u8; 10] = [2, 3, 1, 3, 2, 2, 3, 1, 3, 2];
const X_ACCELERATION: [i16; 10] = [0, 16, 16, 16, 0, 0, -16, -16, -16, 0];
const Y_ACCELERATION: [i16; 10] = [-16, -16, 0, 16, 16, 16, 16, 0, -16, -16];

pub(crate) unsafe fn update(mut k: u16, kind: Flight, trail_interval: u16, game: &mut Game) {
    let slot = k as i32 >> 1;
    if word!(game.projectile_dir[slot]) & 0xf0 != 0 {
        ClearProjectile(k, game);
        if kind == Flight::SuperMissile {
            clear_extensions(game);
        }
        return;
    }
    if trail_interval != 0 {
        let timer = (ram_ptr!(game, u16; 0xc90)).offset(slot as isize);
        let previous = *timer;
        *timer = previous.wrapping_sub(1);
        if previous == 1 {
            word!(game.projectile_timers[slot] = trail_interval);
            crate::game::projectile_trails::emit(k, game);
            k = word!(game.projectile_index);
        }
    }
    advance(k, kind, game);
}

pub(crate) unsafe fn advance(k: u16, kind: Flight, game: &mut Game) {
    let slot = k as i32 >> 1;
    let direction = (word!(game.projectile_dir[slot]) & 0xf) as usize;
    if kind != Flight::SuperMissile {
        word!(game.projectile_bomb_x_speed[slot] += X_ACCELERATION[direction] as u16);
        word!(game.projectile_bomb_y_speed[slot] += Y_ACCELERATION[direction] as u16);
    }
    if matches!(kind, Flight::Missile | Flight::SuperMissile) {
        accelerate_missile(k, game);
    }
    collide(AXES[direction], kind, game);
    let removed = delete_offscreen(game);
    if kind == Flight::SuperMissile && removed & 1 != 0 {
        clear_extensions(game);
    }
}

// Bit 0 selects horizontal movement; bit 1 selects vertical movement.
// Diagonals retain their original slot across horizontal collision callbacks.
pub(crate) unsafe fn collide(axes: u8, kind: Flight, game: &mut Game) {
    let k = word!(game.projectile_index);
    if axes & 1 != 0 {
        let hit = match kind {
            Flight::NonWave => BlockCollNoWaveBeamHoriz(k, game) & 1 != 0,
            Flight::Wave => BlockCollWaveBeamHoriz(k, game) & 1 != 0,
            Flight::Missile | Flight::SuperMissile => BlockCollMissileHoriz(k, game) != 0,
        };
        if kind == Flight::SuperMissile {
            collide_extension_x(game);
        }
        if hit || axes & 2 == 0 {
            return;
        }
    }
    match kind {
        Flight::NonWave => {
            BlockCollNoWaveBeamVert(k, game);
        }
        Flight::Wave => {
            BlockCollWaveBeamVert(k, game);
        }
        Flight::Missile | Flight::SuperMissile => {
            BlockCollMissileVert(k, game);
        }
    }
    if kind == Flight::SuperMissile {
        collide_extension_y(game);
    }
}

unsafe fn clear_extensions(game: &mut Game) {
    for slot in (0..5).rev() {
        let k = slot * 2;
        if word!(game.projectile_type[k >> 1]) & 0xfff == 512 {
            ClearProjectile(k as u16, game);
        }
    }
}
pub(crate) unsafe fn update_extension(k: u16, game: &mut Game) {
    if word!(game.projectile_dir[k >> 1]) & 0xf0 != 0 {
        ClearProjectile(k, game);
        clear_extensions(game);
    }
}

pub(crate) unsafe fn initialize_speed(k: u16, speed: u16, game: &mut Game) {
    if !crate::game::launch_motion_ram::initialize(k, speed, game) {
        Unreachable(game);
    }
}

const RomPtr_90: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x90>;
use crate::game::constants::bank_90::{
    addr_kSuperMissileAccelerations, addr_kSuperMissileAccelerations2,
};

unsafe fn delete_offscreen(game: &mut Game) -> u8 {
    let mut v0: i16 = ((*(ram_ptr!(game, u16; 0xb64))
        .offset((word!(game.projectile_index) >> 1) as isize)) as i16)
        .wrapping_sub((word!(game.layer1_x_pos)) as i16);
    if v0 as i32 + 64 & 0x8000 == 0 && v0 as i32 - 320i32 & 0x8000 != 0 {
        let mut v2: i16 = ((*(ram_ptr!(game, u16; 0xb78))
            .offset((word!(game.projectile_index) >> 1) as isize))
            as i16)
            .wrapping_sub((word!(game.layer1_y_pos)) as i16);
        if v2 as i32 + 64 & 0x8000 == 0 && v2 as i32 - 320i32 & 0x8000 != 0 {
            return 0;
        }
    }
    ClearProjectile(word!(game.projectile_index), game);
    1
}

static kInitializeProjectileSpeed_XY_Diag: [u16; 24] = [
    0x400, 0x2ab, 0x400, 0x2ab, 0x400, 0x2ab, 0x400, 0x2ab, 0x400, 0x2ab, 0x400, 0x2ab, 0x400,
    0x2ab, 0x400, 0x2ab, 0x400, 0x2ab, 0x400, 0x2ab, 0x400, 0x2ab, 0x400, 0x2ab,
];
pub(crate) unsafe fn initialize_beam_speed(r20: u16, game: &mut Game) {
    let v0: i32 = r20 as i32 >> 1;
    let v1: u16 = (4 * (word!(game.projectile_type[v0]) as i32 & 0xf)) as u16;
    let v2: u16 = (2 * (word!(game.projectile_dir[v0]) as i32 & 0xf)) as u16;
    let mut r22: u16 = if v2 == 0 || v2 == 4 || v2 == 8 || v2 == 10 || v2 == 14 || v2 == 18 {
        kInitializeProjectileSpeed_XY_Diag[(v1 >> 1) as usize]
    } else {
        if v2 != 2 && v2 != 6 && v2 != 12 && v2 != 16 {
            Unreachable(game);
        }
        kInitializeProjectileSpeed_XY_Diag[((v1 as i32 >> 1) + 1) as usize]
    };
    initialize_speed(r20, r22, game);
}

unsafe fn accelerate_missile(k: u16, game: &mut Game) {
    static word_90C301: u16 = 0x100;
    let mut v3: u16 = 0;
    let v1: i32 = k as i32 >> 1;
    if (word!(game.projectile_variables[v1]) & 0xff00) != 0 {
        if (word!(game.projectile_type[v1]) & 0x200) != 0 {
            v3 = addr_kSuperMissileAccelerations2 as u16;
        } else {
            v3 = addr_kSuperMissileAccelerations as u16;
        }
        let v4 = RomPtr_90(
            (v3 as i32 + 4 * (word!(game.projectile_dir[v1]) as i32 & 0xf)) as u16,
            game,
        );
        word!(game.projectile_bomb_x_speed[v1] += *(v4 as *mut u16));
        word!(game.projectile_bomb_y_speed[v1] += *(v4.offset(2) as *mut u16));
    } else {
        let v2: u16 = (word_90C301).wrapping_add(word!(game.projectile_variables[v1]));
        word!(game.projectile_variables[v1] = v2);
        if (v2 & 0xff00) != 0 {
            initialize_speed(k, word!(game.projectile_variables[v1]), game);
            if (word!(game.projectile_type[v1]) & 0x200) != 0 {
                spawn_extension(game);
            }
        }
    };
}

macro_rules! super_missile_extension_collision {
    (
        $game:ident, $projectile_slot:ident, $speed_field:ident, $position_field:ident,
        $collide_axis:path, $position_ram:literal $(,)?
    ) => {
        if (word!($game.projectile_type[$projectile_slot]) & 0xf00) == 512
            || (word!($game.projectile_type[$projectile_slot]) & 0xf00) == 2048
        {
            let v5: u8 = word!($game.projectile_variables[$projectile_slot]) as u8;
            if *ram_ptr!($game; 0xc7c, word!($game.projectile_index) as i32 as isize, 1)
                != 0
            {
                let v1: u16 = (abs16(word!($game.$speed_field[$projectile_slot])))
                    & (0xff00u16);
                if v1 as i32 - 2816i32 & 0x8000 != 0 {
                    let v4: i32 = word!($game.projectile_index) as i32 >> 1;
                    if (word!($game.projectile_type[v4]) & 0xf00) != 2048 {
                        word!(
                            $game.$position_field[(v5 >> 1)] =
                                word!($game.$position_field[v4])
                        );
                        return;
                    }
                } else {
                    let r18: u16 = ((v1 as i32 >> 8) - 10i32) as u16;
                    if (word!($game.$speed_field[$projectile_slot]) & 0x8000) == 0 {
                        let v6: u16 = word!($game.projectile_index);
                        word!(
                            $game.$position_field[(v5 >> 1)] =
                                (word!($game.$position_field[$projectile_slot]))
                                    .wrapping_sub(r18)
                        );
                        word!($game.projectile_index = v5 as u16);
                        $collide_axis(v5 as u16, $game);
                        word!($game.projectile_index = v6);
                        if (word!($game.projectile_type[(v6 >> 1)]) & 0xf00) == 2048 {
                            ClearProjectile(v5 as u16, $game);
                        }
                        return;
                    }
                    let v7: u16 = word!($game.projectile_index);
                    word!(
                        $game.$position_field[(v5 >> 1)] =
                            (r18).wrapping_add(
                                *(ram_ptr!($game, u16; $position_ram)).offset(
                                    (word!($game.projectile_index) >> 1) as isize,
                                ),
                            )
                    );
                    word!($game.projectile_index = v5 as u16);
                    $collide_axis(v5 as u16, $game);
                    word!($game.projectile_index = v7);
                    if (word!($game.projectile_type[(v7 >> 1)]) & 0xf00) != 2048 {
                        return;
                    }
                }
                ClearProjectile(v5 as u16, $game);
            }
        }
    };
}

unsafe fn collide_extension_y(game: &mut Game) {
    let v0: i32 = word!(game.projectile_index) as i32 >> 1;
    super_missile_extension_collision!(
        game,
        v0,
        projectile_bomb_y_speed,
        projectile_y_pos,
        BlockCollMissileVert,
        0xb78
    );
}
unsafe fn collide_extension_x(game: &mut Game) {
    let v0: i32 = word!(game.projectile_index) as i32 >> 1;
    super_missile_extension_collision!(
        game,
        v0,
        projectile_bomb_x_speed,
        projectile_x_pos,
        BlockCollMissileHoriz,
        0xb64
    );
}

#[forbid(unsafe_code)]
pub(crate) fn link_extension(slot: u16, game: &mut Game) {
    let address = crate::ram_map::word_arrays::projectile_variables
        + usize::from(checked_word!(game.projectile_index) & !1);
    // This field used native raw loads/stores; the selector uses little endian.
    let value = u16::from_ne_bytes([game.g_ram[address], game.g_ram[address + 1]]);
    game.g_ram[address..address + 2]
        .copy_from_slice(&(value & 0xff00).wrapping_add(slot).to_ne_bytes());
}

unsafe fn spawn_extension(game: &mut Game) {
    let mut v0: u16 = 0;
    while word!(game.projectile_damage[(v0 >> 1)]) != 0 {
        v0 = (v0).wrapping_add(2);
        if ((v0) as i16).wrapping_sub(10) >= 0 {
            return;
        }
    }
    let r20: u16 = v0;
    let v1: i32 = v0 as i32 >> 1;
    word!(game.projectile_type[v1] |= (0x8200u16));
    let v2: i32 = word!(game.projectile_index) as i32 >> 1;
    word!(game.projectile_x_pos[v1] = word!(game.projectile_x_pos[v2]));
    word!(game.projectile_y_pos[v1] = word!(game.projectile_y_pos[v2]));
    word!(game.projectile_dir[v1] = word!(game.projectile_dir[v2]));
    InitProjectilePositionDirection(r20, game);
    initialize_super_missile(v0, game);
    word!(game.projectile_bomb_pre_instructions[v1] = ((0x90b075i32) as u16));
    link_extension(v0, game);
    word!(game.projectile_counter += 1);
}
