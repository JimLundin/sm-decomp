//! Special-beam motion and retirement; RAM remains authoritative across effects.
use crate::game::projectile_animation::initialize_echo;
use crate::game::projectile_trails;
use crate::game::sm_80::QueueSfx1_Max6;
use crate::game::sm_90::{ClearProjectile, Projectile_SinLookup};
use crate::game_state::Game;
use crate::ram_map::{checked_word as word, word_arrays as arrays};
use crate::rtl::Unreachable;

#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
    Wave,
    Ice,
    IceReleased,
    Plasma,
    Spazer,
    SpazerReleased,
}

pub unsafe fn update(mut selected: u16, kind: Kind, game: &mut Game) {
    let slot = usize::from(selected >> 1);
    if word!(game.projectile_dir[slot]) & 0xf0 != 0 {
        if matches!(kind, Kind::Wave | Kind::Ice) {
            unsafe { QueueSfx1_Max6(if kind == Kind::Wave { 0x29 } else { 0x24 }, game) };
        }
        if kind == Kind::Spazer {
            let paired = [4, 6, 0, 2][slot];
            unsafe {
                ClearProjectile(selected, game);
                ClearProjectile(paired, game);
            }
        } else {
            unsafe { ClearProjectile(selected, game) };
        }
        return;
    }
    if kind == Kind::Plasma {
        hold(game);
        let point = unsafe {
            Projectile_SinLookup(
                word!(game.projectile_variables[slot]),
                word!(game.projectile_bomb_x_speed[slot]),
            )
        };
        word!(game.projectile_x_pos[slot] = point.x.wrapping_add(word!(game.samus_x_pos)));
        word!(game.projectile_y_pos[slot] = point.y.wrapping_add(word!(game.samus_y_pos)));
        orbit_step(slot, game);
        match word!(game.projectile_bomb_y_speed[slot]).wrapping_mul(2) {
            0 => plasma_radius(selected, false, game),
            2 => plasma_radius(selected, true, game),
            4 => unsafe { depart_plasma(selected, game) },
            _ => {
                unsafe { Unreachable(game) };
                loop {
                    core::hint::spin_loop();
                }
            }
        }
        return;
    }
    if kind == Kind::Wave {
        let previous = word!(game.projectile_bomb_y_speed[slot]);
        word!(game.projectile_bomb_y_speed[slot] = previous.wrapping_sub(1));
        if previous == 1 || (previous as i16).wrapping_sub(1) < 0 {
            unsafe {
                QueueSfx1_Max6(0x29, game);
                ClearProjectile(selected, game);
            }
            return;
        }
    }
    let previous_x = word!(game.projectile_bomb_x_speed[slot]);
    let previous = word!(game.projectile_timers[slot]);
    word!(game.projectile_timers[slot] = previous.wrapping_sub(1));
    if previous == 1 {
        word!(game.projectile_timers[slot] = 4);
        unsafe { projectile_trails::emit(selected, game) };
        selected = word!(game.projectile_index);
    }
    unsafe { resume(selected, kind, (2 * slot) as u16, previous_x, game) };
}

/// Continue after a trail callback. Wave motion retains the original slot;
/// every other continuation follows the selector reread by the caller.
pub unsafe fn resume(selected: u16, kind: Kind, original: u16, previous_x: u16, game: &mut Game) {
    let slot = usize::from(selected >> 1);
    if kind == Kind::Wave {
        let original = usize::from(original >> 1);
        pursue(
            original,
            arrays::projectile_x_pos,
            arrays::projectile_bomb_x_subpos,
            arrays::projectile_bomb_x_speed,
            word!(game.samus_x_pos),
            game,
        );
        pursue(
            original,
            arrays::projectile_y_pos,
            arrays::projectile_bomb_y_subpos,
            arrays::projectile_variables,
            word!(game.samus_y_pos),
            game,
        );
        if selected == 6 && (word!(game.projectile_bomb_x_speed[3]) ^ previous_x) & 0x8000 != 0 {
            unsafe { QueueSfx1_Max6(0x28, game) };
        }
    } else if kind == Kind::SpazerReleased {
        let y = word!(game.projectile_y_pos[slot]).wrapping_add(8);
        word!(game.projectile_y_pos[slot] = y);
        if y.wrapping_sub(word!(game.layer1_y_pos)).wrapping_sub(248) & 0x8000 == 0 {
            unsafe { ClearProjectile(selected, game) };
            return;
        }
    } else {
        let radius = if kind == Kind::Ice {
            32
        } else {
            word!(game.projectile_bomb_x_speed[slot])
        };
        let point = unsafe { Projectile_SinLookup(word!(game.projectile_variables[slot]), radius) };
        word!(game.projectile_x_pos[slot] = point.x.wrapping_add(word!(game.samus_x_pos)));
        if kind == Kind::IceReleased
            && outside(
                word!(game.projectile_x_pos[slot]),
                word!(game.layer1_x_pos),
                32,
                288,
            )
        {
            unsafe { ClearProjectile(selected, game) };
            return;
        }
        if kind == Kind::Spazer {
            unsafe { advance_spazer(selected, point.y, game) };
        } else {
            word!(game.projectile_y_pos[slot] = point.y.wrapping_add(word!(game.samus_y_pos)));
            if kind == Kind::IceReleased
                && outside(
                    word!(game.projectile_y_pos[slot]),
                    word!(game.layer1_y_pos),
                    0xfff0,
                    256,
                )
            {
                unsafe { ClearProjectile(selected, game) };
                return;
            }
            orbit_step(slot, game);
            if kind == Kind::IceReleased {
                word!(
                    game.projectile_bomb_x_speed[slot] = u16::from(
                        (word!(game.projectile_bomb_x_speed[slot]) as u8).wrapping_add(8)
                    )
                );
            } else {
                let previous = word!(game.projectile_bomb_y_speed[slot]);
                word!(game.projectile_bomb_y_speed[slot] = previous.wrapping_sub(1));
                if previous == 1 {
                    word!(game.projectile_bomb_pre_instructions[slot] = 0xcf7a);
                    word!(game.projectile_bomb_x_speed[slot] = 40);
                    unsafe { QueueSfx1_Max6(0x24, game) };
                }
            }
        }
    }
    hold(game);
}

#[forbid(unsafe_code)]
pub fn plasma_radius(selected: u16, contracting: bool, game: &mut Game) {
    let slot = usize::from(selected >> 1);
    let radius = word!(game.projectile_bomb_x_speed[slot]) as u8;
    let radius = if contracting {
        radius.wrapping_sub(4)
    } else {
        radius.wrapping_add(4)
    };
    word!(game.projectile_bomb_x_speed[slot] = u16::from(radius));
    if if contracting {
        radius < 45
    } else {
        radius >= 192
    } {
        word!(game.projectile_bomb_y_speed[slot] = if contracting { 2 } else { 1 });
    }
}

pub unsafe fn depart_plasma(selected: u16, game: &mut Game) {
    let slot = usize::from(selected >> 1);
    if outside(
        word!(game.projectile_x_pos[slot]),
        word!(game.layer1_x_pos),
        32,
        288,
    ) || outside(
        word!(game.projectile_y_pos[slot]),
        word!(game.layer1_y_pos),
        0xfff0,
        256,
    ) {
        unsafe { ClearProjectile(selected, game) };
    } else {
        word!(
            game.projectile_bomb_x_speed[slot] =
                u16::from((word!(game.projectile_bomb_x_speed[slot]) as u8).wrapping_add(4))
        );
    }
}

unsafe fn advance_spazer(selected: u16, y: u16, game: &mut Game) {
    let slot = usize::from(selected >> 1);
    let phase = [0, 1, 2][usize::from(word!(game.projectile_unk_A[slot]) >> 1)];
    let y = y
        .wrapping_add(word!(game.samus_y_pos))
        .wrapping_sub(if phase == 0 { 0 } else { 114 });
    word!(game.projectile_y_pos[slot] = y);
    if phase != 0 && y.wrapping_sub(word!(game.layer1_y_pos)).wrapping_sub(16) & 0x8000 != 0 {
        unsafe { finish_spazer(selected, game) };
        return;
    }
    // These are native byte lanes, even on a host with different word ordering.
    let step = game.g_ram[arrays::projectile_bomb_y_speed + 2 * slot];
    let angle = game.g_ram[arrays::projectile_variables + 2 * slot].wrapping_add(step);
    word!(game.projectile_variables[slot] = u16::from(angle));
    if phase == 0 {
        if angle == 128 {
            word!(game.projectile_bomb_x_speed[slot] = 160);
            word!(game.projectile_bomb_y_speed[slot] = [2, 0xfffe, 2, 0xfffe][slot]);
            word!(game.projectile_dir[slot] = 0);
            word!(game.projectile_unk_A[slot] = 2);
        }
    } else if phase == 1 {
        let radius = word!(game.projectile_bomb_x_speed[slot]).wrapping_sub(5);
        word!(game.projectile_bomb_x_speed[slot] = radius);
        if radius == 0 {
            word!(game.projectile_bomb_y_speed[slot] = [0xfffe, 2, 0xfffe, 2][slot]);
            word!(
                game.projectile_variables[slot] =
                    u16::from((word!(game.projectile_variables[slot]) as u8).wrapping_add(128))
            );
            word!(game.projectile_unk_A[slot] = 4);
            if selected == 0 {
                unsafe { QueueSfx1_Max6(0x26, game) };
            }
        }
    } else {
        let radius = word!(game.projectile_bomb_x_speed[slot]).wrapping_add(5);
        word!(game.projectile_bomb_x_speed[slot] = radius);
        if radius.wrapping_sub(96) & 0x8000 == 0 {
            unsafe { finish_spazer(selected, game) };
        }
    }
}

unsafe fn finish_spazer(selected: u16, game: &mut Game) {
    let slot = usize::from(selected >> 1);
    word!(game.projectile_x_pos[slot] += [16, 16, 0xfff0, 0xfff0][slot]);
    word!(game.projectile_dir[slot] = 5);
    word!(game.projectile_timers[slot] = 4);
    word!(game.projectile_bomb_pre_instructions[slot] = 0xdc9c);
    if (selected as i16).wrapping_sub(4) < 0 {
        word!(game.projectile_type[slot] = 0x8024);
        unsafe { initialize_echo(selected, game) };
    }
}

#[forbid(unsafe_code)]
fn hold(game: &mut Game) {
    word!(game.cooldown_timer = 2);
    word!(game.flare_counter = 0);
}

#[forbid(unsafe_code)]
fn orbit_step(slot: usize, game: &mut Game) {
    word!(
        game.projectile_variables[slot] = u16::from(
            (word!(game.used_for_sba_attacksB60) as u8)
                .wrapping_add(word!(game.projectile_variables[slot]) as u8)
        )
    );
}

#[forbid(unsafe_code)]
fn outside(position: u16, camera: u16, lower: u16, upper: u16) -> bool {
    let relative = position.wrapping_sub(camera);
    relative.wrapping_add(lower) & 0x8000 != 0 || relative.wrapping_sub(upper) & 0x8000 == 0
}

#[forbid(unsafe_code)]
fn pursue(
    slot: usize,
    position: usize,
    fraction: usize,
    speed: usize,
    target: u16,
    game: &mut Game,
) {
    let (position, fraction, speed) = (position + 2 * slot, fraction + 2 * slot, speed + 2 * slot);
    let previous = word!(game, speed;);
    let speed = if (target as i16).wrapping_sub(word!(game, position;) as i16) < 0 {
        if previous.wrapping_add(2047) & 0x8000 == 0 {
            word!(game, speed; = previous.wrapping_sub(64));
        }
        word!(game, speed;)
    } else {
        if previous.wrapping_sub(2048) & 0x8000 != 0 {
            word!(game, speed; = previous.wrapping_add(64));
        }
        word!(game, speed;)
    };
    // Legacy split pairs read and write native low words before high words.
    let low = u32::from(u16::from_ne_bytes([
        game.g_ram[fraction],
        game.g_ram[fraction + 1],
    ]));
    let high = u32::from(u16::from_ne_bytes([
        game.g_ram[position],
        game.g_ram[position + 1],
    ]));
    let next = (low | (high << 16)).wrapping_add((i32::from(speed as i16) << 8) as u32);
    game.g_ram[fraction..fraction + 2].copy_from_slice(&(next as u16).to_ne_bytes());
    game.g_ram[position..position + 2].copy_from_slice(&((next >> 16) as u16).to_ne_bytes());
}
