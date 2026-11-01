//! Controller sampling, repeat edges, menu filtering and action bindings.
use super::options_objects::objects::{self as bindings, Array};
use super::prelude::*;
use crate::constants::buttons::{
    K_BUTTON_A, K_BUTTON_B, K_BUTTON_DOWN, K_BUTTON_L, K_BUTTON_LEFT, K_BUTTON_R, K_BUTTON_RIGHT,
    K_BUTTON_SELECT, K_BUTTON_START, K_BUTTON_UP, K_BUTTON_X, K_BUTTON_Y,
};
use crate::ram::{checked_word as at, word, words as fields};

fn repeat<H: Host + ?Sized>(offset: usize, sampled: u16, game: &mut H) {
    let last = fields::JOYPAD1_LASTKEYS + offset;
    let pressed = fields::JOYPAD1_NEWKEYS + offset;
    let repeated = fields::JOYPAD1_NEWKEYS2_UNUSED + offset;
    let previous = fields::JOYPAD1_PREV + offset;
    let counter = fields::JOYPAD1_KEYREPEAT_CTR + offset;
    at!(game, last; = sampled);
    at!(game, pressed; = sampled & (at!(game, previous;) ^ sampled));
    at!(game, repeated; = sampled & (at!(game, previous;) ^ sampled));
    if sampled != 0 && sampled == at!(game, previous;) {
        at!(game, counter; = at!(game, counter;).wrapping_sub(1));
        if at!(game, counter;) == 0 {
            at!(game, repeated; = at!(game, last;));
            at!(game, counter; = word!(game.JOYPAD_CTR_REPEAT_NEXT));
        }
    } else {
        at!(game, counter; = word!(game.JOYPAD_CTR_REPEAT_FIRST));
    }
    at!(game, previous; = at!(game, last;));
}

pub fn filter<H: Host + ?Sized>(delay: u16, game: &mut H) {
    word!(game.TIMED_HELD_INPUT_TIMER_RESET = delay);
    let held = !game.observed_inputs().pressed & game.observed_inputs().held;
    let unchanged = held == word!(game.JOYPAD_RELEASED_KEYS);
    word!(game.JOYPAD_RELEASED_KEYS = held);
    if !unchanged {
        word!(game.TIMED_HELD_INPUT_TIMER = word!(game.TIMED_HELD_INPUT_TIMER_RESET));
        word!(game.TIMED_HELD_INPUT = 0);
    } else if word!(game.TIMED_HELD_INPUT_TIMER -= 1, reread) & 0x8000 == 0 {
        word!(game.TIMED_HELD_INPUT = 0);
    } else {
        word! { game;
            TIMED_HELD_INPUT_TIMER = 0;
            PREVIOUS_TIMED_HELD_INPUT = word!(game.TIMED_HELD_INPUT);
            TIMED_HELD_INPUT = held;
        }
    }
    word!(
        game.NEWLY_HELD_DOWN_TIMED_HELD_INPUT = word!(game.TIMED_HELD_INPUT)
            & (word!(game.PREVIOUS_TIMED_HELD_INPUT) ^ word!(game.TIMED_HELD_INPUT))
    );
}

/// Requires a live, exclusively accessed SNES backend for controller register reads.
pub fn sample<H: Host + ?Sized>(game: &mut H) {
    let first = { game.read_register_word(JOY1L as u16) };
    sample_word(first, game);
}

/// Sample the controller as a requested NMI does: from the word auto-read
/// latched at this vblank, which the next frame's logic reads.
pub fn sample_latched<H: Host + ?Sized>(game: &mut H) {
    let first = game.nmi_controller_word();
    sample_word(first, game);
}

fn sample_word<H: Host + ?Sized>(first: u16, game: &mut H) {
    repeat(0, first, game);
    if word!(game.ENABLE_DEBUG) != 0 {
        let second = { game.read_register_word(JOY2L as u16) };
        repeat(2, second, game);
        debug_controls(game);
    }
}

fn debug_controls<H: Host + ?Sized>(game: &mut H) {
    let reset = (K_BUTTON_SELECT | K_BUTTON_START | K_BUTTON_L | K_BUTTON_R) as u16;
    if word!(game.IS_UPLOADING_APU) != 0 || game.observed_inputs().held != reset {
        if word!(game.ENABLE_DEBUG) != 0 {
            word! { game, 0; JOYPAD_DBG_1, JOYPAD_DBG_2 }
            if (word!(game.JOYPAD_DBG_FLAGS) & 0x4000) == 0 {
                for (button, target) in [
                    (K_BUTTON_L, fields::JOYPAD_DBG_1),
                    (K_BUTTON_R, fields::JOYPAD_DBG_2),
                ] {
                    let chord = (K_BUTTON_SELECT | button) as u16;
                    if game.observed_inputs().held & chord == chord {
                        at!(game, target; = game.observed_inputs().pressed);
                        word! { game, 0; JOYPAD1_LASTKEYS, JOYPAD1_NEWKEYS }
                    }
                }
                if (word!(game.JOYPAD_DBG_2) & 0x80) != 0 {
                    at!(game, crate::ram::mirror::NMITIMEN; ^= 0x30);
                }
                if (word!(game.JOYPAD_DBG_2) & 0x8000) != 0 {
                    let entering_swap = (!(word!(game.JOYPAD_DBG_FLAGS)) & 0x8000) != 0;
                    word!(game.JOYPAD_DBG_FLAGS ^= (0x8000u16));
                    if entering_swap {
                        let [missiles, supers, bombs] =
                            crate::session_cohort::load::<crate::inventory::Inventory, _>(game)
                                .ammunition
                                .map(|supply| supply.current);
                        word! { game;
                            JOYPAD_DBG_MISSILES_SWAP = missiles;
                            JOYPAD_DBG_SUPER_MISSILES_SWAP = supers;
                            JOYPAD_DBG_POWER_BOMBS_SWAP = bombs;
                        }
                        crate::session_cohort::with(
                            game,
                            |_, inventory: &mut crate::inventory::Inventory| {
                                for supply in &mut inventory.ammunition {
                                    supply.current = 0;
                                }
                            },
                        );
                    } else {
                        let swapped = [
                            word!(game.JOYPAD_DBG_MISSILES_SWAP),
                            word!(game.JOYPAD_DBG_SUPER_MISSILES_SWAP),
                            word!(game.JOYPAD_DBG_POWER_BOMBS_SWAP),
                        ];
                        crate::session_cohort::with(
                            game,
                            |_, inventory: &mut crate::inventory::Inventory| {
                                for (supply, value) in inventory.ammunition.iter_mut().zip(swapped)
                                {
                                    supply.current = value;
                                }
                            },
                        );
                    }
                }
                if (word!(game.JOYPAD_DBG_2) & 0x40) != 0 {
                    word!(game.JOYPAD_DBG_FLAGS ^= (0x2000u16));
                }
            }
        } else {
            word! { game, 0; JOYPAD_DBG_1, JOYPAD_DBG_2 }
            word!(game.JOYPAD2_LAST &= (0x10u16));
            word!(game.JOYPAD2_NEW_KEYS &= (0x10u16));
        }
    } else {
        crate::session_cohort::with(game, |_, words: &mut crate::session_words::SessionWords| {
            words.debug_disable_sounds = 0;
        });
        game.request_reset();
    }
}

// Intro playback saves only action bindings; its directional defaults remain after restoration.
pub fn use_cinematic_bindings<H: Host + ?Sized>(game: &mut H) {
    crate::word_transfers!(game, ascending, 7;
        little(fields::BUTTON_CONFIG_SHOOT_X_SAVED) <- little(fields::BUTTON_CONFIG_SHOOT_X));
    word! {game; BUTTON_CONFIG_UP = K_BUTTON_UP as u16; BUTTON_CONFIG_DOWN = K_BUTTON_DOWN as u16;
    BUTTON_CONFIG_LEFT = K_BUTTON_LEFT as u16; BUTTON_CONFIG_RIGHT = K_BUTTON_RIGHT as u16;
    BUTTON_CONFIG_SHOOT_X = K_BUTTON_X as u16; BUTTON_CONFIG_JUMP_A = K_BUTTON_A as u16;
    BUTTON_CONFIG_RUN_B = K_BUTTON_B as u16; BUTTON_CONFIG_ITEMCANCEL_Y = K_BUTTON_Y as u16;
    BUTTON_CONFIG_ITEMSWITCH = K_BUTTON_SELECT as u16;
    BUTTON_CONFIG_AIM_DOWN_L = K_BUTTON_L as u16; BUTTON_CONFIG_AIM_UP_R = K_BUTTON_R as u16; }
}

pub fn restore_cinematic_bindings<H: Host + ?Sized>(game: &mut H) {
    crate::word_transfers!(game, ascending, 7;
        little(fields::BUTTON_CONFIG_SHOOT_X) <- little(fields::BUTTON_CONFIG_SHOOT_X_SAVED));
}

// The editor assigns the first seven buttons; saved choices also accept Left and Right.
const BINDING_BUTTONS: [u16; 9] = [
    K_BUTTON_X as u16,
    K_BUTTON_A as u16,
    K_BUTTON_B as u16,
    K_BUTTON_SELECT as u16,
    K_BUTTON_Y as u16,
    K_BUTTON_L as u16,
    K_BUTTON_R as u16,
    K_BUTTON_LEFT as u16,
    K_BUTTON_RIGHT as u16,
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
        bindings::set_at(game, Array::ControllerBindings, index, choice as u16);
    }
}

pub fn reset_binding_choices<H: Host + ?Sized>(game: &mut H) {
    word! {game; BUTTON_CONFIG_SHOOT_X = K_BUTTON_X as u16;
    BUTTON_CONFIG_JUMP_A = K_BUTTON_A as u16; BUTTON_CONFIG_RUN_B = K_BUTTON_B as u16;
    BUTTON_CONFIG_ITEMCANCEL_Y = K_BUTTON_Y as u16; BUTTON_CONFIG_ITEMSWITCH = K_BUTTON_SELECT as u16;
    BUTTON_CONFIG_AIM_UP_R = K_BUTTON_R as u16; BUTTON_CONFIG_AIM_DOWN_L = K_BUTTON_L as u16; }
    load_binding_choices(game);
}

pub fn store_binding_choices<H: Host + ?Sized>(game: &mut H) -> u8 {
    for index in 0..7 {
        let binding =
            BINDING_BUTTONS[bindings::get_at(game, Array::ControllerBindings, index) as usize];
        let address = crate::menu_lists::binding_address(game, index);
        crate::word_transfers!(game, ascending, 1; native(address) <- value(binding));
    }
    0
}

pub fn remap_binding<H: Host + ?Sized>(selected: u16, game: &mut H) {
    crate::probe::hit("controller_input.remap_binding");
    let Some(choice) = BINDING_BUTTONS[..7]
        .iter()
        .rposition(|button| button & game.observed_inputs().pressed != 0)
    else {
        return;
    };
    let mut next = (2 * i32::from(selected) + 2) as u16;
    if (2 * i32::from(selected) - 12) & 0x8000 == 0 {
        next = 0;
    }
    // Malformed choice lists retain the cursor after the sixth unsuccessful advance.
    for _ in 0..6 {
        if bindings::get_at(game, Array::ControllerBindings, next >> 1) == choice as u16 {
            break;
        }
        next = next.wrapping_add(2);
        if (next as i16).wrapping_sub(14) >= 0 {
            next = 0;
        }
    }
    let selected = i32::from(selected);
    let previous = bindings::get_at(game, Array::ControllerBindings, selected);
    bindings::set_at(game, Array::ControllerBindings, selected, choice as u16);
    bindings::set_at(game, Array::ControllerBindings, next >> 1, previous);
    super::menu_tilemap::controller_bindings(game);
}
