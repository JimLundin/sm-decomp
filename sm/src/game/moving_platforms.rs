//! Moving room mechanisms: triggers, travel, endpoint waits, and Samus contact.
use crate::game::enemy_layouts::{EnemyLayout72, EnemyLayout72Late};
use crate::game::sm_80::QueueSfx3_Max6;
use crate::game::sm_a0::{
    CheckIfEnemyIsOnScreen, CheckIfEnemyTouchesSamus, EnemyFunc_AC67, IsSamusWithinEnemy_X,
    NormalEnemyShotAi, NormalEnemyShotAiSkipDeathAnim_CurEnemy,
};
use crate::game_state::Game;
use crate::ram_map::{ram_ptr, word};
use crate::rtl::{words, Unreachable};

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
unsafe fn slot(game: &mut Game) -> *mut EnemyLayout72 {
    (ram_ptr!(game, u8; 0xf78))
        .add(word!(game.cur_enemy_index) as usize)
        .cast()
}
unsafe fn movement_sound(game: &mut Game) {
    if CheckIfEnemyIsOnScreen(game) == 0 {
        QueueSfx3_Max6(0xe, game);
    }
}

pub(crate) unsafe fn initialize(kind: Kind, game: &mut Game) {
    let e = slot(game);
    if kind == Kind::TimedShutter {
        let e = e.cast::<EnemyLayout72Late>();
        (*e).var_A = crate::rtl::rom_table_word!(
            game,
            0xa2ea4e,
            ((*e).base.current_instruction as i32 + 2 * (*e).base.extra_properties as i32) as isize
        );
        let step = if (*e).base.extra_properties != 0 {
            -8i16 as u16
        } else {
            8
        };
        (*e).var_B = (*e).base.y_pos;
        (*e).var_C = (*e).var_B.wrapping_add(step);
        (*e).var_D = (*e).var_C.wrapping_add(step);
        (*e).var_E = (*e).var_D.wrapping_add(step);
        (*e).base.extra_properties = 0;
        (*e).var_F = 0;
        (*e).base.current_instruction = 0xe998;
        let i = 2 * ((*e).parameter_2 as u8) as isize;
        (*e).var_00 = crate::rtl::rom_table_word!(game, 0xa2ea56, i);
        (*e).var_01 = crate::rtl::rom_table_word!(game, 0xa2ea56, i + 1);
        return;
    }
    let p = kind.motion();
    (*e).var_00 = (*e).base.current_instruction as u8 as u16;
    let i = 4 * (*e).var_00 as isize;
    crate::set_fields! {[(*e)]; var_D = crate::rtl::rom_table_word!(game, 0xa28187, i),
    var_C = crate::rtl::rom_table_word!(game, 0xa28187, i + 1),
    var_F = crate::rtl::rom_table_word!(game, 0xa28187, i + 2),
    var_E = crate::rtl::rom_table_word!(game, 0xa28187, i + 3),
    var_01 = (*e).base.current_instruction >> 8, var_20 = (*e).var_01 ^ p.horizontal as u16,
    var_02 = (*e).base.extra_properties as u8 as u16,
    var_08 = 16u16.wrapping_mul((*e).var_02), var_03 = (*e).base.extra_properties >> 8,
    var_09 = 16u16.wrapping_mul((*e).var_03), var_04 = (*e).parameter_1 as u8 as u16,
    var_07 = 2u16.wrapping_mul((*e).var_04), var_05 = (*e).parameter_1 >> 8,
    var_06 = (*e).parameter_2, var_B = (*e).parameter_2, };
    let pos = if p.horizontal {
        (*e).base.x_pos
    } else {
        (*e).base.y_pos
    };
    let (low, high) = if p.horizontal {
        (&raw mut (*e).var_11, &raw mut (*e).var_12)
    } else {
        (&raw mut (*e).var_0F, &raw mut (*e).var_10)
    };
    *low = pos;
    *high = (*e).var_05.wrapping_add(pos);
    if (*e).var_01 == 0 {
        *high = pos;
        *low = pos.wrapping_sub((*e).var_05);
    }
    (*e).var_A = p.state(WaitForTrigger);
    if p.horizontal {
        (*e).var_15 = word!(game.samus_x_pos);
    }
    (*e).base.extra_properties = 0;
    (*e).var_0A = 0;
    if p.horizontal {
        (*e).var_0B = 0;
    }
    (*e).base.current_instruction = match kind {
        Kind::Platform => 0xede7,
        Kind::VerticalShutter => 0xe9aa,
        _ => 0xe9d4,
    };
}

pub(crate) unsafe fn update(kind: Kind, game: &mut Game) {
    if kind == Kind::TimedShutter {
        update_timed(game);
        return;
    }
    let e = slot(game);
    // Both original entry points dispatch both axes' phase addresses.
    let p = if VERTICAL.phases.contains(&(*e).var_A) {
        &VERTICAL
    } else {
        &HORIZONTAL
    };
    let Some(phase) = p.decode((*e).var_A) else {
        Unreachable(game);
        return;
    };
    match phase {
        WaitForTrigger | WaitForProximity => {
            let trigger = if phase == WaitForProximity {
                1
            } else {
                (*e).var_07 >> 1
            };
            let ready = match trigger {
                0 => {
                    let old = (*e).var_B;
                    (*e).var_B = old.wrapping_sub(1);
                    if old == 1 {
                        (*e).var_B = (*e).var_06;
                    }
                    old == 1
                }
                1 => IsSamusWithinEnemy_X(word!(game.cur_enemy_index), (*e).var_06, game) != 0,
                2 => true,
                3 | 4 => false,
                _ => panic!("mechanism trigger index out of bounds"),
            };
            if ready {
                (*e).var_A = p.state(if (*e).var_01 != 0 {
                    MovingPositive
                } else {
                    MovingNegative
                });
                if !p.horizontal {
                    movement_sound(game);
                }
            }
        }
        MovingNegative | MovingPositive => travel(p, phase == MovingPositive, game),
        WaitingAtLow | WaitingAtHigh => {
            (*e).var_B = (*e).var_B.wrapping_sub(1);
            if (*e).var_B & 0x8000 != 0 {
                if !p.horizontal && phase == WaitingAtHigh {
                    movement_sound(game);
                }
                (*e).var_A = p.state(if phase == WaitingAtLow {
                    MovingPositive
                } else {
                    MovingNegative
                });
                if (*e).var_04 == 1 && ((*e).var_01 != 0) == (phase == WaitingAtLow) {
                    (*e).var_A = p.state(WaitForProximity);
                }
                if !p.horizontal && phase == WaitingAtLow && (*e).base.enemy_ptr != 0xd83f {
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
    if (*e).var_A != p.state(MovingNegative)
        && (*e).var_A != p.state(MovingPositive)
        && collision != 0xffff
        && collision == word!(game.cur_enemy_index)
        && word!(game.samus_contact_damage_index) != 0
    {
        activate(p, game);
    }
    if p.horizontal {
        (*e).var_15 = word!(game.samus_x_pos);
        (*e).var_16 = word!(game.samus_x_subpos);
    }
}

unsafe fn travel(p: &Motion, positive: bool, game: &mut Game) {
    let e = slot(game);
    let speed = if positive {
        words!((*e), var_D, var_C)
    } else {
        words!((*e), var_F, var_E)
    };
    let (pos, limit) = if p.horizontal {
        (*e).var_0D = (*e).base.x_pos;
        (*e).var_13 = 0;
        if EnemyFunc_AC67(word!(game.cur_enemy_index), game) != 0
            && (((word!(game.samus_x_pos) as i16).wrapping_sub((*e).base.x_pos as i16) >= 0)
                == positive)
        {
            (*e).var_13 = 1;
        }
        words!((*e).base, x_pos, x_subpos += speed);
        if (*e).var_13 != 0 {
            word!(game.extra_samus_x_subdisplacement = speed as u16);
            word!(game.extra_samus_x_displacement = (speed >> 16) as u16);
            push_samus(positive, game);
        }
        (
            (*e).base.x_pos,
            if positive { (*e).var_12 } else { (*e).var_11 },
        )
    } else {
        (*e).var_0E = (*e).base.y_pos;
        (*e).var_0A = 0;
        if CheckIfEnemyTouchesSamus(word!(game.cur_enemy_index), game) != 0 {
            (*e).var_0A = 1;
        }
        words!((*e).base, y_pos, y_subpos += speed);
        if (*e).var_0A != 0 {
            word!(game.extra_samus_y_displacement = (*e).base.y_pos.wrapping_sub((*e).var_0E));
        }
        (
            (*e).base.y_pos,
            if positive { (*e).var_10 } else { (*e).var_0F },
        )
    };
    let delta = if positive {
        pos.wrapping_sub(limit)
    } else {
        limit.wrapping_sub(pos)
    };
    if delta as i16 >= 0 {
        let delay = if positive { (*e).var_09 } else { (*e).var_08 };
        if delay == 4080 {
            (*e).var_A = p.state(Stopped);
        } else {
            (*e).var_B = delay;
            (*e).var_A = p.state(if positive {
                WaitingAtHigh
            } else {
                WaitingAtLow
            });
        }
    }
}

unsafe fn push_samus(positive: bool, game: &mut Game) {
    let key = if positive { 0x200 } else { 0x100 };
    if word!(game.joypad1_lastkeys) & key != 0 {
        word!(game.extra_samus_x_displacement += if positive { 4 } else { -4i16 as u16 });
        word!(game.extra_samus_y_subdisplacement = 0);
    }
}

unsafe fn activate(p: &Motion, game: &mut Game) {
    let e = slot(game);
    if (*e).var_07.wrapping_sub(6) & 0x8000 == 0 {
        if (*e).var_07 != 8 {
            if (*e).var_0C != 0 {
                return;
            }
            (*e).var_0C = 1;
        }
        if (*e).var_A == p.state(MovingNegative) || (*e).var_A == p.state(MovingPositive) {
            return;
        }
        if p.horizontal {
            (*e).var_20 ^= 1;
        }
        (*e).var_A = p.state(if (*e).var_20 != 0 {
            MovingPositive
        } else {
            MovingNegative
        });
        if !p.horizontal {
            (*e).var_20 ^= 1;
        }
    }
    if !p.horizontal {
        movement_sound(game);
    }
}

pub(crate) unsafe fn react(kind: Kind, event: Event, game: &mut Game) {
    if let Event::Shot = event {
        match kind {
            Kind::VerticalShutter => NormalEnemyShotAi(game),
            Kind::HorizontalShutter => NormalEnemyShotAiSkipDeathAnim_CurEnemy(game),
            _ => {}
        }
    }
    if kind == Kind::HorizontalShutter && matches!(event, Event::Touch) {
        let e = slot(game);
        if (*e).var_A == HORIZONTAL.state(Stopped) {
            let positive =
                (word!(game.samus_x_pos) as i16).wrapping_sub((*e).base.x_pos as i16) >= 0;
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
    let e = slot(game).cast::<EnemyLayout72Late>();
    match (*e).var_A {
        0xeabd | 0xead1 | 0xeae7 | 0xeafd => {
            let proximity = (*e).var_A == 0xead1 || (*e).var_A == 0xeae7;
            let ready = if proximity {
                IsSamusWithinEnemy_X(word!(game.cur_enemy_index), (*e).parameter_1, game) != 0
            } else if (*e).parameter_1 != 0 {
                (*e).parameter_1 = (*e).parameter_1.wrapping_sub(1);
                false
            } else {
                true
            };
            if ready {
                movement_sound(game);
                (*e).var_A = if (*e).var_A == 0xeabd || (*e).var_A == 0xead1 {
                    0xec13
                } else {
                    0xeb11
                };
            }
        }
        0xeb11 | 0xec13 => {
            let opening = (*e).var_A == 0xec13;
            if opening {
                (*e).var_40 = (*e).base.y_pos;
            }
            let stage = (if opening {
                (*e).var_F & 0x7fff
            } else {
                (*e).var_F
            }) as usize;
            assert!(stage < 5, "timed shutter stage out of bounds");
            if stage < 4 {
                let speed = words!((*e), var_00, var_01);
                words!(
                    (*e).base,
                    y_pos,
                    y_subpos += if opening { speed.wrapping_neg() } else { speed }
                );
                let stop = [(*e).var_B, (*e).var_C, (*e).var_D, (*e).var_E][stage];
                let limit = stop.wrapping_add(if opening { -16i16 as u16 } else { 16 });
                let delta = (limit as i16).wrapping_sub((*e).base.y_pos as i16);
                if (opening && delta >= 0) || (!opening && delta < 0) {
                    (*e).base.y_pos = if stage == 3 {
                        limit
                    } else if opening {
                        limit.wrapping_add(7)
                    } else {
                        stop.wrapping_add(9)
                    };
                    (*e).var_F = (*e).var_F.wrapping_add(1);
                    if stage != 3 {
                        (*e).base.instruction_timer = 1;
                        (*e).base.current_instruction = [0xe99e, 0xe9a4, 0xe9aa][stage];
                        (*e).base.y_height = [16, 24, 32][stage];
                    }
                }
            }
            if opening && CheckIfEnemyTouchesSamus(word!(game.cur_enemy_index), game) != 0 {
                let displacement = (*e).base.y_pos.wrapping_sub((*e).var_40);
                if (displacement as i16) < 0 {
                    word!(game.extra_samus_y_displacement += displacement);
                }
            }
        }
        _ => {
            Unreachable(game);
        }
    }
}
