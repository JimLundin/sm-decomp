//! Controller sampling, repeat edges and the menu's held-button filter.
use crate::game::constants::buttons::{kButton_L, kButton_R, kButton_Select, kButton_Start};
use crate::game_state::Game;
use crate::ram_map::{checked_word as at, word, words as fields};
use crate::rtl::{JOY1L, JOY2L, ReadRegWord};

fn repeat(offset: usize, sampled: u16, game: &mut Game) {
    let last = fields::joypad1_lastkeys + offset;
    let pressed = fields::joypad1_newkeys + offset;
    let repeated = fields::joypad1_newkeys2_UNUSED + offset;
    let previous = fields::joypad1_prev + offset;
    let counter = fields::joypad1_keyrepeat_ctr + offset;
    at!(game, last; = sampled);
    at!(game, pressed; = sampled & (at!(game, previous;) ^ sampled));
    at!(game, repeated; = sampled & (at!(game, previous;) ^ sampled));
    if sampled != 0 && sampled == at!(game, previous;) {
        at!(game, counter; = at!(game, counter;).wrapping_sub(1));
        if at!(game, counter;) == 0 {
            at!(game, repeated; = at!(game, last;));
            at!(game, counter; = word!(game.joypad_ctr_repeat_next));
        }
    } else {
        at!(game, counter; = word!(game.joypad_ctr_repeat_first));
    }
    at!(game, previous; = at!(game, last;));
}

pub(crate) fn filter(delay: u16, game: &mut Game) {
    word!(game.timed_held_input_timer_reset = delay);
    let held = !word!(game.joypad1_newkeys) & word!(game.joypad1_lastkeys);
    let unchanged = held == word!(game.joypad_released_keys);
    word!(game.joypad_released_keys = held);
    if !unchanged {
        word!(game.timed_held_input_timer = word!(game.timed_held_input_timer_reset));
        word!(game.timed_held_input = 0);
    } else if word!(game.timed_held_input_timer -= 1, reread) & 0x8000 == 0 {
        word!(game.timed_held_input = 0);
    } else {
        word! { game;
            timed_held_input_timer = 0;
            previous_timed_held_input = word!(game.timed_held_input);
            timed_held_input = held;
        }
    }
    word!(
        game.newly_held_down_timed_held_input = word!(game.timed_held_input)
            & (word!(game.previous_timed_held_input) ^ word!(game.timed_held_input))
    );
}

/// Requires a live, exclusively accessed SNES backend for controller register reads.
pub(crate) unsafe fn sample(game: &mut Game) {
    let first = unsafe { ReadRegWord(JOY1L as u16, game) };
    repeat(0, first, game);
    if word!(game.enable_debug) != 0 {
        let second = unsafe { ReadRegWord(JOY2L as u16, game) };
        repeat(2, second, game);
        debug_controls(game);
    }
}

fn debug_controls(game: &mut Game) {
    let reset = (kButton_Select | kButton_Start | kButton_L | kButton_R) as u16;
    if word!(game.is_uploading_apu) != 0 || word!(game.joypad1_lastkeys) != reset {
        if word!(game.enable_debug) != 0 {
            word! { game, 0; joypad_dbg_1, joypad_dbg_2 }
            if (word!(game.joypad_dbg_flags) & 0x4000) == 0 {
                for (button, target) in [
                    (kButton_L, fields::joypad_dbg_1),
                    (kButton_R, fields::joypad_dbg_2),
                ] {
                    let chord = (kButton_Select | button) as u16;
                    if word!(game.joypad1_lastkeys) & chord == chord {
                        at!(game, target; = word!(game.joypad1_newkeys));
                        word! { game, 0; joypad1_lastkeys, joypad1_newkeys }
                    }
                }
                if (word!(game.joypad_dbg_2) & 0x80) != 0 {
                    at!(game, 0x84; ^= 0x30);
                }
                if (word!(game.joypad_dbg_2) & 0x8000) != 0 {
                    let v2: bool = (!(word!(game.joypad_dbg_flags)) & 0x8000) != 0;
                    word!(game.joypad_dbg_flags ^= (0x8000u16));
                    if v2 {
                        word! { game;
                            joypad_dbg_missiles_swap = word!(game.samus_missiles);
                            joypad_dbg_super_missiles_swap = word!(game.samus_super_missiles);
                            joypad_dbg_power_bombs_swap = word!(game.samus_power_bombs);
                        }
                        word! { game, 0;
                            samus_missiles, samus_super_missiles, samus_power_bombs,
                        }
                    } else {
                        word! { game;
                            samus_missiles = word!(game.joypad_dbg_missiles_swap);
                            samus_super_missiles = word!(game.joypad_dbg_super_missiles_swap);
                            samus_power_bombs = word!(game.joypad_dbg_power_bombs_swap);
                        }
                    }
                }
                if (word!(game.joypad_dbg_2) & 0x40) != 0 {
                    word!(game.joypad_dbg_flags ^= (0x2000u16));
                }
            }
        } else {
            word! { game, 0; joypad_dbg_1, joypad_dbg_2 }
            word!(game.joypad2_last &= (0x10u16));
            word!(game.joypad2_new_keys &= (0x10u16));
        }
    } else {
        word!(game.debug_disable_sounds = 0);
        game.request_reset();
    }
}
