//! Controller sampling, repeat edges, menu filtering and action bindings.
use super::prelude::*;
use crate::constants::buttons::{
    kButton_A, kButton_B, kButton_Down, kButton_L, kButton_Left, kButton_R, kButton_Right,
    kButton_Select, kButton_Start, kButton_Up, kButton_X, kButton_Y,
};
use crate::ram::{checked_word as at, word, words as fields};

fn repeat<H: Host + ?Sized>(offset: usize, sampled: u16, game: &mut H) {
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

pub fn filter<H: Host + ?Sized>(delay: u16, game: &mut H) {
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
pub fn sample<H: Host + ?Sized>(game: &mut H) {
    let first = { game.read_register_word(JOY1L as u16) };
    repeat(0, first, game);
    if word!(game.enable_debug) != 0 {
        let second = { game.read_register_word(JOY2L as u16) };
        repeat(2, second, game);
        debug_controls(game);
    }
}

fn debug_controls<H: Host + ?Sized>(game: &mut H) {
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
                    at!(game, crate::ram::mirror::NMITIMEN; ^= 0x30);
                }
                if (word!(game.joypad_dbg_2) & 0x8000) != 0 {
                    let entering_swap = (!(word!(game.joypad_dbg_flags)) & 0x8000) != 0;
                    word!(game.joypad_dbg_flags ^= (0x8000u16));
                    if entering_swap {
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

// Intro playback saves only action bindings; its directional defaults remain after restoration.
pub fn use_cinematic_bindings<H: Host + ?Sized>(game: &mut H) {
    crate::word_transfers!(game, ascending, 7;
        little(fields::button_config_shoot_x_saved) <- little(fields::button_config_shoot_x));
    word! {game; button_config_up = kButton_Up as u16; button_config_down = kButton_Down as u16;
    button_config_left = kButton_Left as u16; button_config_right = kButton_Right as u16;
    button_config_shoot_x = kButton_X as u16; button_config_jump_a = kButton_A as u16;
    button_config_run_b = kButton_B as u16; button_config_itemcancel_y = kButton_Y as u16;
    button_config_itemswitch = kButton_Select as u16;
    button_config_aim_down_L = kButton_L as u16; button_config_aim_up_R = kButton_R as u16; }
}

pub fn restore_cinematic_bindings<H: Host + ?Sized>(game: &mut H) {
    crate::word_transfers!(game, ascending, 7;
        little(fields::button_config_shoot_x) <- little(fields::button_config_shoot_x_saved));
}

// The editor assigns the first seven buttons; saved choices also accept Left and Right.
const BINDING_BUTTONS: [u16; 9] = [
    kButton_X as u16,
    kButton_A as u16,
    kButton_B as u16,
    kButton_Select as u16,
    kButton_Y as u16,
    kButton_L as u16,
    kButton_R as u16,
    kButton_Left as u16,
    kButton_Right as u16,
];

pub fn load_binding_choices<H: Host + ?Sized>(game: &mut H) {
    for index in 0..7 {
        let address = crate::menu_lists::binding_address(game, index) as usize;
        let binding = u16::from_ne_bytes([
            game.ram().read_byte(address),
            game.ram().read_byte(address + 1),
        ]);
        let choice = BINDING_BUTTONS[..7]
            .iter()
            .position(|button| binding & button != 0)
            .unwrap_or(0);
        word!(game.options_controller_bindings[index] = choice as u16);
    }
}

pub fn reset_binding_choices<H: Host + ?Sized>(game: &mut H) {
    word! {game; button_config_shoot_x = kButton_X as u16;
    button_config_jump_a = kButton_A as u16; button_config_run_b = kButton_B as u16;
    button_config_itemcancel_y = kButton_Y as u16; button_config_itemswitch = kButton_Select as u16;
    button_config_aim_up_R = kButton_R as u16; button_config_aim_down_L = kButton_L as u16; }
    load_binding_choices(game);
}

pub fn store_binding_choices<H: Host + ?Sized>(game: &mut H) -> u8 {
    for index in 0..7 {
        let binding = BINDING_BUTTONS[word!(game.options_controller_bindings[index]) as usize];
        let address = crate::menu_lists::binding_address(game, index);
        crate::word_transfers!(game, ascending, 1; native(address) <- value(binding));
    }
    0
}

pub fn remap_binding<H: Host + ?Sized>(game: &mut H) {
    let Some(choice) = BINDING_BUTTONS[..7]
        .iter()
        .rposition(|button| button & word!(game.joypad1_newkeys) != 0)
    else {
        return;
    };
    let mut next = (2 * word!(game.menu_option_index) as i32 + 2) as u16;
    if (2 * word!(game.menu_option_index) as i32 - 12) & 0x8000 == 0 {
        next = 0;
    }
    // Malformed choice lists retain the cursor after the sixth unsuccessful advance.
    for _ in 0..6 {
        let address = 0x1b23 + 2 * ((next as usize >> 1) + 13);
        if u16::from_ne_bytes([
            game.ram().read_byte(address),
            game.ram().read_byte(address + 1),
        ]) == choice as u16
        {
            break;
        }
        next = next.wrapping_add(2);
        if (next as i16).wrapping_sub(14) >= 0 {
            next = 0;
        }
    }
    let selected = word!(game.menu_option_index) as i32;
    let previous = word!(game.options_controller_bindings[selected]);
    word!(game.options_controller_bindings[selected] = choice as u16);
    crate::word_transfers!(game, ascending, 1;
        native(crate::ram::word_arrays::options_controller_bindings as isize + 2 * (next as isize >> 1))
            <- value(previous));
    super::menu_tilemap::controller_bindings(game);
}
