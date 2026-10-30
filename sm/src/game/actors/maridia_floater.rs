//! Maridia Floater body, bobbing, ascent and delayed spike burst.
//! Native actor words and little-endian drawing aliases remain authoritative.
#![forbid(unsafe_code)]
use crate::game::actors::{api, combat};
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use crate::ram_map::{slot, word};
use sm_game::actors::access::ActorKey as Key;
use sm_game::actors::layouts::EnemyLayout8 as Floater;
use {
    crate::game::enemy_projectiles::spawn_with_enemy_gfx, sm_game::enemy_projectiles::host::Effects,
};

const BOB: [u16; 12] = [0, 1, 2, 3, 2, 1, 0, 0xffff, 0xfffe, 0xfffd, 0xfffe, 0xffff];

fn spawn_spikes(k: u16, game: &mut Game) {
    for direction in (0..=7).rev() {
        spawn_with_enemy_gfx(direction, k, 0xd298, game);
    }
}

fn draw_stem(k: u16, game: &mut Game) {
    let drawing = k as i32 >> 1;
    let instruction = word!(game.ENEMY_DRAWING_QUEUE[(drawing + 103)]);
    let (bias, offsets) = if instruction as i32 + 0x3e6f & 0x8000 != 0 {
        (15997, [-12i16, -16, -20])
    } else {
        (15983, [-20i16, -16, -12])
    };
    let mut index = (instruction as i32 - 4 + bias) as u16 >> 1;
    if index as i32 - 6 & 0x8000 == 0 {
        index = 0;
    }
    let y = (offsets[(index >> 1) as usize] as u16)
        .wrapping_add(slot!(game, Floater[k as i32].base.y_pos));
    word!(game.ENEMY_DRAWING_QUEUE[(drawing + 93)] = y);
}

// Return true only when a requested center stop is reached on this bob tick.
fn bob(k: u16, stop_at_center: bool, game: &mut Game) -> bool {
    let E = k as i32;
    let expired = slot!(game, Floater[E].var_d) == 1;
    slot!(game, Floater[E].var_d, |old| old.wrapping_sub(1));
    let negative = slot!(game, Floater[E].var_d) & 0x8000 != 0;
    if !expired && !negative {
        return false;
    }
    slot!(game, Floater[E].var_d = 5);
    let drawing = k as i32 >> 1;
    let x = BOB[slot!(game, Floater[E].var_c) as usize]
        .wrapping_add(word!(game.ENEMY_DRAWING_QUEUE_SIZES[drawing]));
    word!(game.ENEMY_DRAWING_QUEUE[(drawing + 91)] = x);
    slot!(game, Floater[E].base.x_pos = x);
    if stop_at_center && centered(k, game) {
        return true;
    }
    let mut next = slot!(game, Floater[E].var_c).wrapping_add(1);
    let wrap = if stop_at_center {
        next as i32 - 12 & 0x8000 == 0
    } else {
        slot!(game, Floater[E].var_c) as i32 - 11 & 0x8000 == 0
    };
    if wrap {
        next = 0;
    }
    slot!(game, Floater[E].var_c = next);
    false
}
fn centered(k: u16, game: &mut Game) -> bool {
    slot!(game, Floater[k as i32].var_c) == 0 || slot!(game, Floater[k as i32].var_c) == 6
}
fn move_vertical(k: u16, game: &mut Game) -> bool {
    // The zero acceleration still reads and publishes the split native words.
    slot!(game, Floater[k as i32](var_a, var_b) += 0);
    let amount = (u32::from(slot!(game, Floater[k as i32].var_a)) << 16)
        | u32::from(slot!(game, Floater[k as i32].var_b));
    api::move_vertical(&mut Runtime::new(game), Key::from_legacy(k), amount as i32)
        == api::MovementResult::Blocked
}
fn native_size(index: usize, game: &Game) -> u16 {
    let at = 0xf68 + index * 2;
    u16::from_ne_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)])
}
fn begin_descent(k: u16, game: &mut Game) {
    slot!(game, Floater[k as i32]; var_f = 0xc500, var_e = 10);
    let drawing = k as i32 >> 1;
    word!(game.ENEMY_DRAWING_QUEUE[(drawing + 104)] = 1);
    word!(game.ENEMY_DRAWING_QUEUE[(drawing + 103)] = 0xc191);
}

pub(crate) fn state_address(game: &mut Game) -> u32 {
    let E = word!(game.CUR_ENEMY_INDEX) as i32;
    u32::from(slot!(game, Floater[E].var_f)) | 0xa80000
}

pub(crate) fn advance(mut state: u16, k: u16, game: &mut Game) {
    let E = k as i32;
    let drawing = k as i32 >> 1;
    if state == 0xc2cf && slot!(game, Floater[E].base.ai_handler_bits) & 1 != 0 {
        slot!(game, Floater[E].var_f = 0xc3e1);
        state = 0xc3e1;
    }
    match state {
        0xc283 | 0xc2a6 | 0xc500 | 0xc59f => {
            let expired = slot!(game, Floater[E].var_e) == 1;
            slot!(game, Floater[E].var_e, |old| old.wrapping_sub(1));
            let negative = slot!(game, Floater[E].var_e) & 0x8000 != 0;
            if expired || negative {
                match state {
                    0xc283 => {
                        word!(game.ENEMY_DRAWING_QUEUE[(drawing + 104)] = 1);
                        word!(game.ENEMY_DRAWING_QUEUE[(drawing + 103)] = 0xc183);
                        slot!(game, Floater[E]; var_f = 0xc2a6, var_e = 10);
                    }
                    0xc2a6 => {
                        slot!(game, Floater[E]; var_f = 0xc2cf, var_a = 0xffff, var_b = 0x8000);
                        slot!(game, Floater[E].base; instruction_timer = 1, current_instruction = 0xc163);
                    }
                    0xc500 => {
                        slot!(game, Floater[E]; var_f = 0xc51d, var_a = 1, var_b = 0);
                    }
                    _ => {
                        word!(game.ENEMY_DRAWING_QUEUE[(drawing + 100)] = 0);
                        word!(game.ENEMY_DRAWING_QUEUE[(drawing + 97)] |= 0x200);
                        spawn_spikes(k, game);
                        combat::die(
                            &mut Runtime::new(game),
                            api::DeathKind::Normal { animation: 0 },
                        );
                        return;
                    }
                }
            }
        }
        0xc2cf | 0xc3e1 => {
            let reacting = state == 0xc3e1;
            if reacting && slot!(game, Floater[E].base.ai_handler_bits) & 1 == 0 {
                slot!(game, Floater[E].var_f = 0xc36b);
                return;
            }
            bob(k, false, game);
            let reached = move_vertical(k, game) || {
                let home = native_size(drawing as usize + 1, game);
                let rise = if reacting {
                    native_size(drawing as usize + 3, game)
                } else {
                    0x40
                };
                (home as i32 - rise as i32 - slot!(game, Floater[E].base.y_pos) as i32) as i16 >= 0
            };
            if reached {
                if centered(k, game) {
                    if reacting {
                        slot!(game, Floater[E].var_f = 0xc4dc);
                    } else {
                        begin_descent(k, game);
                    }
                } else {
                    slot!(
                        game,
                        Floater[E].var_f = if reacting { 0xc469 } else { 0xc36b }
                    );
                }
            }
        }
        0xc36b | 0xc469 => {
            if state == 0xc469 && slot!(game, Floater[E].base.ai_handler_bits) & 1 == 0 {
                slot!(game, Floater[E].var_f = 0xc36b);
                return;
            }
            if bob(k, true, game) {
                if state == 0xc469 {
                    slot!(game, Floater[E].var_f = 0xc4dc);
                } else {
                    begin_descent(k, game);
                }
            } else {
                move_vertical(k, game);
            }
        }
        0xc4dc => {
            if slot!(game, Floater[E].base.ai_handler_bits) & 1 == 0 {
                begin_descent(k, game);
            }
        }
        0xc51d => {
            move_vertical(k, game);
            if (slot!(game, Floater[E].base.y_pos) as i16)
                .wrapping_sub(word!(game.ENEMY_DRAWING_QUEUE_SIZES[(drawing + 1)]) as i16)
                >= 0
            {
                slot!(
                    game,
                    Floater[E].base.y_pos = word!(game.ENEMY_DRAWING_QUEUE_SIZES[(drawing + 1)])
                );
                slot!(game, Floater[E]; var_f = 0xc283, var_e = 60);
                slot!(game, Floater[E].base; instruction_timer = 1, current_instruction = 0xc173);
            }
        }
        0xc569 => {
            if word!(game.ENEMY_DRAWING_QUEUE[(drawing + 103)]) as i32 + 0x3e6f & 0x8000 == 0 {
                let index = (word!(game.ENEMY_DRAWING_QUEUE[(drawing + 103)]) as i32 - 4 + 15983)
                    as u16
                    >> 1;
                if index != 0 {
                    word!(
                        game.ENEMY_DRAWING_QUEUE[(drawing + 103)] =
                            sm_game::enemy_lists::floater_spike_instruction_list(
                                game,
                                (index >> 1) as isize
                            )
                    );
                    word!(game.ENEMY_DRAWING_QUEUE[(drawing + 104)] = 1);
                }
            }
            slot!(game, Floater[E]; var_f = 0xc59f, var_e = 32);
        }
        0xc568 => return,
        _ => {
            Runtime::new(game).unreachable();
            return;
        }
    }
    draw_stem(k, game);
}

pub(crate) fn initialize(game: &mut Game) {
    let E: i32 = word!(game.CUR_ENEMY_INDEX) as i32;
    slot!(game, Floater[E].base.properties, |old| (old)
        | ((0x2000) as u16));
    slot!(game, Floater[E].base.spritemap_pointer = 0x804d as u16);
    slot!(game, Floater[E].base.instruction_timer = 1);
    slot!(game, Floater[E].base.timer = 0);
    if slot!(game, Floater[E].parameter_1) != 0 {
        slot!(game, Floater[E].var_a = slot!(game, Floater[E].base.x_pos));
        slot!(game, Floater[E].var_b = slot!(game, Floater[E].base.y_pos));
        slot!(game, Floater[E].var_f = (0xa8c568i32) as u16);
        slot!(game, Floater[E].base.current_instruction = 0xc199 as u16);
        slot!(game, Floater[E].var_d = slot!(game, Floater[E].parameter_2));
    } else {
        slot!(game, Floater[E].var_e = 60);
        slot!(game, Floater[E].var_f = (0xa8c283i32) as u16);
        slot!(game, Floater[E].base.current_instruction = 0xc173 as u16);
    };
}

pub(crate) fn touch(game: &mut Game) {
    let E: i32 = word!(game.CUR_ENEMY_INDEX) as i32;
    if slot!(game, Floater[E].parameter_2) == 0 {
        let palette_index: u16 = slot!(game, Floater[E].base.palette_index);
        combat::contact_reaction(&mut Runtime::new(game), api::DeathHandling::Finalize);
        if slot!(game, Floater[E].base.health) == 0 {
            word!(
                game.ENEMY_DRAWING_QUEUE[((word!(game.CUR_ENEMY_INDEX) as i32 >> 1) + 97)] |=
                    (0x200u16)
            );
            slot!(game, Floater[E].base.palette_index = palette_index);
            spawn_spikes(word!(game.CUR_ENEMY_INDEX), game);
            slot!(game, Floater[E].base.palette_index = 2560);
        }
    }
}

pub(crate) fn shot(game: &mut Game) {
    let E: i32 = word!(game.CUR_ENEMY_INDEX) as i32;
    if slot!(game, Floater[E].parameter_2) == 0 {
        combat::shot_reaction(&mut Runtime::new(game), api::ShotHandling::ReactionOnly);
        if (slot!(game, Floater[E].base.ai_handler_bits) & 4) != 0 {
            let v2: i32 = word!(game.CUR_ENEMY_INDEX) as i32 >> 1;
            word!(game.ENEMY_DRAWING_QUEUE[(v2 + 109)] = slot!(game, Floater[E].base.frozen_timer));
            word!(game.ENEMY_DRAWING_QUEUE[(v2 + 99)] |= (4u16));
        }
        if (slot!(game, Floater[E].base.ai_handler_bits) & 2) != 0 {
            let v3: i32 = word!(game.CUR_ENEMY_INDEX) as i32 >> 1;
            word!(game.ENEMY_DRAWING_QUEUE[(v3 + 108)] = slot!(game, Floater[E].base.flash_timer));
            word!(game.ENEMY_DRAWING_QUEUE[(v3 + 99)] |= (2u16));
        }
        if slot!(game, Floater[E].base.health) == 0 {
            slot!(game, Floater[E].var_f = (0xa8c569i32) as u16);
            slot!(game, Floater[E].parameter_2 = 1);
        }
    }
}

pub(crate) fn power_bomb(game: &mut Game) {
    combat::power_bomb_reaction(&mut Runtime::new(game), api::DeathHandling::Finalize);
    let E: i32 = word!(game.CUR_ENEMY_INDEX) as i32;
    if slot!(game, Floater[E].base.health) != 0 {
        let v1: i32 = word!(game.CUR_ENEMY_INDEX) as i32 >> 1;
        word!(game.ENEMY_DRAWING_QUEUE[(v1 + 111)] = slot!(game, Floater[E].base.shake_timer));
        word!(
            game.ENEMY_DRAWING_QUEUE[(v1 + 110)] = slot!(game, Floater[E].base.invincibility_timer)
        );
        word!(game.ENEMY_DRAWING_QUEUE[(v1 + 108)] = slot!(game, Floater[E].base.flash_timer));
        word!(game.ENEMY_DRAWING_QUEUE[(v1 + 109)] = slot!(game, Floater[E].base.frozen_timer));
        word!(game.ENEMY_DRAWING_QUEUE[(v1 + 99)] = slot!(game, Floater[E].base.ai_handler_bits));
    } else {
        word!(
            game.ENEMY_DRAWING_QUEUE[((word!(game.CUR_ENEMY_INDEX) as i32 >> 1) + 97)] |=
                (0x200u16)
        );
    };
}
