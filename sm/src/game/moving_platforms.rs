//! Moving room mechanisms: triggers, travel, endpoint waits, and Samus contact.
// All RAM access below is checked. Unsafe scopes identify the remaining ROM,
// damage and failure interfaces that still need migration.
#![deny(unsafe_op_in_unsafe_fn)]
use crate::game::enemy_combat::{apply_shot, shot_reaction};
use crate::game::enemy_layouts::{EnemyLayout72, EnemyLayout72Late};
use crate::game::sm_80::QueueSfx3_Max6;
use crate::game::sm_a0::{
    CheckIfEnemyIsOnScreen, CheckIfEnemyTouchesSamus, EnemyFunc_AC67, IsSamusWithinEnemy_X,
};
use crate::game_state::Game;
use crate::ram_map::checked_word as word;
use crate::rtl::Unreachable;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Kind {
    TimedShutter,
    Platform,
    VerticalShutter,
    HorizontalShutter,
}
#[derive(Clone, Copy)]
pub(crate) enum Event {
    Touch,
    Shot,
    PowerBomb,
}

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    WaitForTrigger,
    WaitForProximity,
    MovingNegative,
    MovingPositive,
    WaitingAtLow,
    WaitingAtHigh,
    Stopped,
}
use Phase::*;

struct Motion {
    horizontal: bool,
    phases: [u16; 7],
}
const VERTICAL: Motion = Motion {
    horizontal: false,
    phases: [0xef09, 0xef28, 0xef68, 0xefd4, 0xf040, 0xf072, 0xf099],
};
const HORIZONTAL: Motion = Motion {
    horizontal: true,
    phases: [0xf224, 0xf243, 0xf272, 0xf2e4, 0xf38c, 0xf3b0, 0xf3d4],
};
impl Motion {
    fn state(&self, phase: Phase) -> u16 {
        self.phases[phase as usize]
    }
    fn decode(&self, state: u16) -> Option<Phase> {
        [
            WaitForTrigger,
            WaitForProximity,
            MovingNegative,
            MovingPositive,
            WaitingAtLow,
            WaitingAtHigh,
            Stopped,
        ]
        .into_iter()
        .find(|&phase| self.state(phase) == state)
    }
}
impl Kind {
    fn motion(self) -> &'static Motion {
        if self == Self::HorizontalShutter {
            &HORIZONTAL
        } else {
            &VERTICAL
        }
    }
}
// A slot is an address, never a borrow or pointer retained across callbacks.
#[forbid(unsafe_code)]
fn slot(game: &Game) -> usize {
    0xf78 + usize::from(word!(game.cur_enemy_index))
}
macro_rules! field {
    ($game:ident, $slot:ident, $($field:ident).+ $(= $value:expr)?) => {
        word!($game, $slot + field!(@offset $($field).+); $(= $value)?)
    };
    ($game:ident, $slot:ident, $($field:ident).+ ^= $value:expr) => {
        word!($game, $slot + field!(@offset $($field).+); ^= $value)
    };
    (@offset var_40) => { core::mem::offset_of!(EnemyLayout72Late, var_40) };
    (@offset $($field:ident).+) => { core::mem::offset_of!(EnemyLayout72, $($field).+) };
}
#[forbid(unsafe_code)]
fn movement_sound(game: &mut Game) {
    if CheckIfEnemyIsOnScreen(game) == 0 {
        QueueSfx3_Max6(0xe, game);
    }
}

#[forbid(unsafe_code)]
pub(crate) fn initialize(kind: Kind, game: &mut Game) {
    let e = slot(game);
    if kind == Kind::TimedShutter {
        field!(
            game,
            e,
            var_A = crate::rtl::rom_table_word!(
                game,
                0xa2ea4e,
                (field!(game, e, base.current_instruction) as i32
                    + 2 * field!(game, e, base.extra_properties) as i32) as isize
            )
        );
        let step = if field!(game, e, base.extra_properties) != 0 {
            -8i16 as u16
        } else {
            8
        };
        field!(game, e, var_B = field!(game, e, base.y_pos));
        field!(game, e, var_C = field!(game, e, var_B).wrapping_add(step));
        field!(game, e, var_D = field!(game, e, var_C).wrapping_add(step));
        field!(game, e, var_E = field!(game, e, var_D).wrapping_add(step));
        field!(game, e, base.extra_properties = 0);
        field!(game, e, var_F = 0);
        field!(game, e, base.current_instruction = 0xe998);
        let i = 2 * (field!(game, e, parameter_2) as u8) as isize;
        field!(
            game,
            e,
            var_00 = crate::rtl::rom_table_word!(game, 0xa2ea56, i)
        );
        field!(
            game,
            e,
            var_01 = crate::rtl::rom_table_word!(game, 0xa2ea56, i + 1)
        );
        return;
    }
    let p = kind.motion();
    field!(
        game,
        e,
        var_00 = field!(game, e, base.current_instruction) as u8 as u16
    );
    let i = 4 * field!(game, e, var_00) as isize;
    field!(
        game,
        e,
        var_D = crate::rtl::rom_table_word!(game, 0xa28187, i)
    );
    field!(
        game,
        e,
        var_C = crate::rtl::rom_table_word!(game, 0xa28187, i + 1)
    );
    field!(
        game,
        e,
        var_F = crate::rtl::rom_table_word!(game, 0xa28187, i + 2)
    );
    field!(
        game,
        e,
        var_E = crate::rtl::rom_table_word!(game, 0xa28187, i + 3)
    );
    field!(
        game,
        e,
        var_01 = field!(game, e, base.current_instruction) >> 8
    );
    field!(
        game,
        e,
        var_20 = field!(game, e, var_01) ^ p.horizontal as u16
    );
    field!(
        game,
        e,
        var_02 = field!(game, e, base.extra_properties) as u8 as u16
    );
    field!(
        game,
        e,
        var_08 = 16u16.wrapping_mul(field!(game, e, var_02))
    );
    field!(
        game,
        e,
        var_03 = field!(game, e, base.extra_properties) >> 8
    );
    field!(
        game,
        e,
        var_09 = 16u16.wrapping_mul(field!(game, e, var_03))
    );
    field!(game, e, var_04 = field!(game, e, parameter_1) as u8 as u16);
    field!(game, e, var_07 = 2u16.wrapping_mul(field!(game, e, var_04)));
    field!(game, e, var_05 = field!(game, e, parameter_1) >> 8);
    field!(game, e, var_06 = field!(game, e, parameter_2));
    field!(game, e, var_B = field!(game, e, parameter_2));
    let pos = if p.horizontal {
        field!(game, e, base.x_pos)
    } else {
        field!(game, e, base.y_pos)
    };
    let (low, high) = if p.horizontal {
        (field!(@offset var_11), field!(@offset var_12))
    } else {
        (field!(@offset var_0F), field!(@offset var_10))
    };
    word!(game, e + low; = pos);
    word!(game, e + high; = field!(game, e, var_05).wrapping_add(pos));
    if field!(game, e, var_01) == 0 {
        word!(game, e + high; = pos);
        word!(game, e + low; = pos.wrapping_sub(field!(game, e, var_05)));
    }
    field!(game, e, var_A = p.state(WaitForTrigger));
    if p.horizontal {
        field!(game, e, var_15 = word!(game.samus_x_pos));
    }
    field!(game, e, base.extra_properties = 0);
    field!(game, e, var_0A = 0);
    if p.horizontal {
        field!(game, e, var_0B = 0);
    }
    field!(
        game,
        e,
        base.current_instruction = match kind {
            Kind::Platform => 0xede7,
            Kind::VerticalShutter => 0xe9aa,
            _ => 0xe9d4,
        }
    );
}

pub(crate) unsafe fn update(kind: Kind, game: &mut Game) {
    if kind == Kind::TimedShutter {
        unsafe { update_timed(game) };
        return;
    }
    let e = slot(game);
    // Both original entry points dispatch both axes' phase addresses.
    let p = if VERTICAL.phases.contains(&field!(game, e, var_A)) {
        &VERTICAL
    } else {
        &HORIZONTAL
    };
    let Some(phase) = p.decode(field!(game, e, var_A)) else {
        unsafe { Unreachable(game) };
        return;
    };
    match phase {
        WaitForTrigger | WaitForProximity => {
            let trigger = if phase == WaitForProximity {
                1
            } else {
                field!(game, e, var_07) >> 1
            };
            let ready = match trigger {
                0 => {
                    let old = field!(game, e, var_B);
                    field!(game, e, var_B = old.wrapping_sub(1));
                    if old == 1 {
                        field!(game, e, var_B = field!(game, e, var_06));
                    }
                    old == 1
                }
                1 => {
                    IsSamusWithinEnemy_X(word!(game.cur_enemy_index), field!(game, e, var_06), game)
                        != 0
                }
                2 => true,
                3 | 4 => false,
                _ => panic!("mechanism trigger index out of bounds"),
            };
            if ready {
                field!(
                    game,
                    e,
                    var_A = p.state(if field!(game, e, var_01) != 0 {
                        MovingPositive
                    } else {
                        MovingNegative
                    })
                );
                if !p.horizontal {
                    movement_sound(game);
                }
            }
        }
        MovingNegative | MovingPositive => travel(p, phase == MovingPositive, game),
        WaitingAtLow | WaitingAtHigh => {
            field!(game, e, var_B = field!(game, e, var_B).wrapping_sub(1));
            if field!(game, e, var_B) & 0x8000 != 0 {
                if !p.horizontal && phase == WaitingAtHigh {
                    movement_sound(game);
                }
                field!(
                    game,
                    e,
                    var_A = p.state(if phase == WaitingAtLow {
                        MovingPositive
                    } else {
                        MovingNegative
                    })
                );
                if field!(game, e, var_04) == 1
                    && (field!(game, e, var_01) != 0) == (phase == WaitingAtLow)
                {
                    field!(game, e, var_A = p.state(WaitForProximity));
                }
                if !p.horizontal
                    && phase == WaitingAtLow
                    && field!(game, e, base.enemy_ptr) != 0xd83f
                {
                    movement_sound(game);
                }
            }
        }
        _ => {}
    }
    let p = kind.motion();
    let collision = word!(game.enemy_index_colliding_dirs[3])
        & word!(game.enemy_index_colliding_dirs[2])
        & word!(game.enemy_index_colliding_dirs[1])
        & word!(game.enemy_index_colliding_dirs[0]);
    if field!(game, e, var_A) != p.state(MovingNegative)
        && field!(game, e, var_A) != p.state(MovingPositive)
        && collision != 0xffff
        && collision == word!(game.cur_enemy_index)
        && word!(game.samus_contact_damage_index) != 0
    {
        activate(p, game);
    }
    if p.horizontal {
        field!(game, e, var_15 = word!(game.samus_x_pos));
        field!(game, e, var_16 = word!(game.samus_x_subpos));
    }
}

#[forbid(unsafe_code)]
fn travel(p: &Motion, positive: bool, game: &mut Game) {
    let e = slot(game);
    let speed = if positive {
        (u32::from(field!(game, e, var_D)) << 16) | u32::from(field!(game, e, var_C))
    } else {
        (u32::from(field!(game, e, var_F)) << 16) | u32::from(field!(game, e, var_E))
    };
    let (pos, limit) = if p.horizontal {
        field!(game, e, var_0D = field!(game, e, base.x_pos));
        field!(game, e, var_13 = 0);
        if EnemyFunc_AC67(word!(game.cur_enemy_index), game) != 0
            && (((word!(game.samus_x_pos) as i16).wrapping_sub(field!(game, e, base.x_pos) as i16)
                >= 0)
                == positive)
        {
            field!(game, e, var_13 = 1);
        }
        let next = (u32::from(field!(game, e, base.x_subpos))
            | (u32::from(field!(game, e, base.x_pos)) << 16))
            .wrapping_add(speed);
        field!(game, e, base.x_subpos = next as u16);
        field!(game, e, base.x_pos = (next >> 16) as u16);
        if field!(game, e, var_13) != 0 {
            word!(game.extra_samus_x_subdisplacement = speed as u16);
            word!(game.extra_samus_x_displacement = (speed >> 16) as u16);
            push_samus(positive, game);
        }
        (
            field!(game, e, base.x_pos),
            if positive {
                field!(game, e, var_12)
            } else {
                field!(game, e, var_11)
            },
        )
    } else {
        field!(game, e, var_0E = field!(game, e, base.y_pos));
        field!(game, e, var_0A = 0);
        if CheckIfEnemyTouchesSamus(word!(game.cur_enemy_index), game) != 0 {
            field!(game, e, var_0A = 1);
        }
        let next = (u32::from(field!(game, e, base.y_subpos))
            | (u32::from(field!(game, e, base.y_pos)) << 16))
            .wrapping_add(speed);
        field!(game, e, base.y_subpos = next as u16);
        field!(game, e, base.y_pos = (next >> 16) as u16);
        if field!(game, e, var_0A) != 0 {
            word!(
                game.extra_samus_y_displacement =
                    field!(game, e, base.y_pos).wrapping_sub(field!(game, e, var_0E))
            );
        }
        (
            field!(game, e, base.y_pos),
            if positive {
                field!(game, e, var_10)
            } else {
                field!(game, e, var_0F)
            },
        )
    };
    let delta = if positive {
        pos.wrapping_sub(limit)
    } else {
        limit.wrapping_sub(pos)
    };
    if delta as i16 >= 0 {
        let delay = if positive {
            field!(game, e, var_09)
        } else {
            field!(game, e, var_08)
        };
        if delay == 4080 {
            field!(game, e, var_A = p.state(Stopped));
        } else {
            field!(game, e, var_B = delay);
            field!(
                game,
                e,
                var_A = p.state(if positive {
                    WaitingAtHigh
                } else {
                    WaitingAtLow
                })
            );
        }
    }
}

#[forbid(unsafe_code)]
fn push_samus(positive: bool, game: &mut Game) {
    let key = if positive { 0x200 } else { 0x100 };
    if word!(game.joypad1_lastkeys) & key != 0 {
        word!(game.extra_samus_x_displacement += if positive { 4 } else { -4i16 as u16 });
        word!(game.extra_samus_y_subdisplacement = 0);
    }
}

#[forbid(unsafe_code)]
fn activate(p: &Motion, game: &mut Game) {
    let e = slot(game);
    if field!(game, e, var_07).wrapping_sub(6) & 0x8000 == 0 {
        if field!(game, e, var_07) != 8 {
            if field!(game, e, var_0C) != 0 {
                return;
            }
            field!(game, e, var_0C = 1);
        }
        if field!(game, e, var_A) == p.state(MovingNegative)
            || field!(game, e, var_A) == p.state(MovingPositive)
        {
            return;
        }
        if p.horizontal {
            field!(game, e, var_20 ^= 1);
        }
        field!(
            game,
            e,
            var_A = p.state(if field!(game, e, var_20) != 0 {
                MovingPositive
            } else {
                MovingNegative
            })
        );
        if !p.horizontal {
            field!(game, e, var_20 ^= 1);
        }
    }
    if !p.horizontal {
        movement_sound(game);
    }
}

pub(crate) unsafe fn react(kind: Kind, event: Event, game: &mut Game) {
    if let Event::Shot = event {
        match kind {
            Kind::VerticalShutter => unsafe { apply_shot(game) },
            Kind::HorizontalShutter => unsafe { shot_reaction(game) },
            _ => {}
        }
    }
    if kind == Kind::HorizontalShutter && matches!(event, Event::Touch) {
        let e = slot(game);
        if field!(game, e, var_A) == HORIZONTAL.state(Stopped) {
            let positive = (word!(game.samus_x_pos) as i16)
                .wrapping_sub(field!(game, e, base.x_pos) as i16)
                >= 0;
            let key = if positive { 0x200 } else { 0x100 };
            if word!(game.joypad1_lastkeys) & key != 0 {
                word!(game.extra_samus_x_displacement = if positive { 4 } else { -4i16 as u16 });
                word!(game.extra_samus_y_subdisplacement = 0);
            }
        }
    } else {
        activate(kind.motion(), game);
    }
}

unsafe fn update_timed(game: &mut Game) {
    let e = slot(game);
    match field!(game, e, var_A) {
        0xeabd | 0xead1 | 0xeae7 | 0xeafd => {
            let proximity = field!(game, e, var_A) == 0xead1 || field!(game, e, var_A) == 0xeae7;
            let ready = if proximity {
                IsSamusWithinEnemy_X(
                    word!(game.cur_enemy_index),
                    field!(game, e, parameter_1),
                    game,
                ) != 0
            } else if field!(game, e, parameter_1) != 0 {
                field!(
                    game,
                    e,
                    parameter_1 = field!(game, e, parameter_1).wrapping_sub(1)
                );
                false
            } else {
                true
            };
            if ready {
                movement_sound(game);
                field!(
                    game,
                    e,
                    var_A = if field!(game, e, var_A) == 0xeabd || field!(game, e, var_A) == 0xead1
                    {
                        0xec13
                    } else {
                        0xeb11
                    }
                );
            }
        }
        0xeb11 | 0xec13 => {
            let opening = field!(game, e, var_A) == 0xec13;
            if opening {
                field!(game, e, var_40 = field!(game, e, base.y_pos));
            }
            let stage = (if opening {
                field!(game, e, var_F) & 0x7fff
            } else {
                field!(game, e, var_F)
            }) as usize;
            assert!(stage < 5, "timed shutter stage out of bounds");
            if stage < 4 {
                let speed =
                    (u32::from(field!(game, e, var_00)) << 16) | u32::from(field!(game, e, var_01));
                let step = if opening { speed.wrapping_neg() } else { speed };
                let next = (u32::from(field!(game, e, base.y_subpos))
                    | (u32::from(field!(game, e, base.y_pos)) << 16))
                    .wrapping_add(step);
                field!(game, e, base.y_subpos = next as u16);
                field!(game, e, base.y_pos = (next >> 16) as u16);
                let stop = [
                    field!(game, e, var_B),
                    field!(game, e, var_C),
                    field!(game, e, var_D),
                    field!(game, e, var_E),
                ][stage];
                let limit = stop.wrapping_add(if opening { -16i16 as u16 } else { 16 });
                let delta = (limit as i16).wrapping_sub(field!(game, e, base.y_pos) as i16);
                if (opening && delta >= 0) || (!opening && delta < 0) {
                    field!(
                        game,
                        e,
                        base.y_pos = if stage == 3 {
                            limit
                        } else if opening {
                            limit.wrapping_add(7)
                        } else {
                            stop.wrapping_add(9)
                        }
                    );
                    field!(game, e, var_F = field!(game, e, var_F).wrapping_add(1));
                    if stage != 3 {
                        field!(game, e, base.instruction_timer = 1);
                        field!(
                            game,
                            e,
                            base.current_instruction = [0xe99e, 0xe9a4, 0xe9aa][stage]
                        );
                        field!(game, e, base.y_height = [16, 24, 32][stage]);
                    }
                }
            }
            if opening && CheckIfEnemyTouchesSamus(word!(game.cur_enemy_index), game) != 0 {
                let displacement =
                    field!(game, e, base.y_pos).wrapping_sub(field!(game, e, var_40));
                if (displacement as i16) < 0 {
                    word!(game.extra_samus_y_displacement += displacement);
                }
            }
        }
        _ => {
            unsafe { Unreachable(game) };
        }
    }
}
