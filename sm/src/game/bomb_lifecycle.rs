//! Bomb fuse transitions and spread motion; legacy callbacks still own their effects.
use crate::game::explosion_window::spawn_power_bomb;
use crate::game::projectile_animation::start_bomb_explosion;
use crate::game::sm_80::QueueSfx2_Max6;
use crate::game::sm_88::EnableHdmaObjects;
use crate::game::sm_90::ClearProjectile;
use crate::game::sm_94::{BlockCollSpreadBomb, BombOrPowerBomb_Func1};
use crate::game_state::Game;
use crate::ram_map::{checked_word as word, word_arrays as arrays};

#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
    Bomb,
    PowerBomb,
    Spread,
}

pub(crate) const SPREAD_FRACTIONS: [u16; 5] = [0, 0, 0x8000, 0, 0];
const X: (usize, usize) = (arrays::projectile_x_pos, arrays::projectile_bomb_x_subpos);
const Y: (usize, usize) = (arrays::projectile_y_pos, arrays::projectile_bomb_y_subpos);
const Y_SPEED: (usize, usize) = (arrays::projectile_bomb_y_speed, arrays::projectile_timers);

pub unsafe fn advance_fuse(power: bool, game: &mut Game) {
    let selected = word!(game.projectile_index);
    let slot = usize::from(word!(game.projectile_index) >> 1);
    let remaining = word!(game.projectile_variables[slot]);
    if remaining == 0 {
        if power && word!(game.power_bomb_flag) == 0 {
            unsafe { ClearProjectile(word!(game.projectile_index), game) };
        }
        return;
    }
    let remaining = remaining.wrapping_sub(1);
    word!(game.projectile_variables[slot] = remaining);
    if remaining == 15 {
        word!(game.projectile_bomb_instruction_ptr[slot] += 28);
    } else if remaining == 0 {
        if power {
            word!(game.power_bomb_explosion_x_pos = word!(game.projectile_x_pos[slot]));
            word!(game.power_bomb_explosion_y_pos = word!(game.projectile_y_pos[slot]));
            unsafe {
                EnableHdmaObjects(game);
                spawn_power_bomb(game);
            }
            // Effects may change the live selector; the armed slot stays captured.
            word!(game.projectile_variables[(selected >> 1)] = 0xffff);
        } else {
            unsafe { QueueSfx2_Max6(8, game) };
            start_bomb_explosion(selected, game);
        }
    }
}

pub unsafe fn update(selected: u16, kind: Kind, game: &mut Game) {
    let slot = usize::from(selected >> 1);
    if word!(game.projectile_dir[slot]) & 0xf0 != 0 {
        unsafe { ClearProjectile(selected, game) };
        return;
    }
    // The fuse follows the live selector, while motion/contact retain the caller's slot.
    unsafe { advance_fuse(kind == Kind::PowerBomb, game) };
    if kind != Kind::Spread {
        unsafe { BombOrPowerBomb_Func1(selected, game) };
        return;
    }
    if word!(game.projectile_variables[slot]) != 0 {
        let gravity =
            (u32::from(word!(game.samus_y_accel)) << 16) | u32::from(word!(game.samus_y_subaccel));
        add_pair(game, slot, Y_SPEED, gravity);
        let velocity = vertical_step(slot, game);
        add_pair(game, slot, Y, velocity);
        if unsafe { BlockCollSpreadBomb(selected, game) } & 1 != 0 {
            let reverse = vertical_step(slot, game).wrapping_neg();
            add_pair(game, slot, Y, reverse);
            if word!(game.projectile_bomb_y_speed[slot]) & 0x8000 != 0 {
                word!(game.projectile_bomb_y_speed[slot] = 0);
                word!(game.projectile_y_radius[slot] = 0);
            } else {
                word!(game.projectile_timers[slot] = SPREAD_FRACTIONS[slot.wrapping_sub(5)]);
                word!(game.projectile_bomb_y_speed[slot] = word!(game.projectile_unk_A[slot]));
            }
            return;
        }
        let step = horizontal_step(word!(game.projectile_bomb_x_speed[slot]));
        add_pair(game, slot, X, step);
    }
    if unsafe { BlockCollSpreadBomb(selected, game) } & 1 != 0 {
        let speed = word!(game.projectile_bomb_x_speed[slot]);
        word!(game.projectile_bomb_x_speed[slot] ^= 0x8000);
        add_pair(game, slot, X, horizontal_step(speed).wrapping_neg());
    }
}

#[forbid(unsafe_code)]
fn vertical_step(slot: usize, game: &Game) -> u32 {
    (u32::from(word!(game.projectile_bomb_y_speed[slot])) << 16)
        | u32::from(word!(game.projectile_timers[slot]))
}

#[forbid(unsafe_code)]
fn horizontal_step(speed: u16) -> u32 {
    let magnitude = u32::from(speed & 0x7fff) << 8;
    if speed & 0x8000 != 0 {
        magnitude.wrapping_neg()
    } else {
        magnitude
    }
}

#[forbid(unsafe_code)]
fn add_pair(game: &mut Game, slot: usize, pair: (usize, usize), delta: u32) {
    let (high, low) = pair;
    let low = low + 2 * slot;
    let high = high + 2 * slot;
    // These pairs used native unaligned words, with low-before-high reads and writes.
    let fraction = u32::from(u16::from_ne_bytes([game.g_ram[low], game.g_ram[low + 1]]));
    let whole = u32::from(u16::from_ne_bytes([game.g_ram[high], game.g_ram[high + 1]]));
    let sum = (fraction | (whole << 16)).wrapping_add(delta);
    game.g_ram[low..low + 2].copy_from_slice(&(sum as u16).to_ne_bytes());
    game.g_ram[high..high + 2].copy_from_slice(&((sum >> 16) as u16).to_ne_bytes());
}
