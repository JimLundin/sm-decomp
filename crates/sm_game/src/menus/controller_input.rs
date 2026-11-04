//! Controller sampling, repeat edges, menu filtering and action bindings.
use super::host::Controller;
use super::options_objects::objects::{self as bindings, Array};
use super::prelude::*;
use crate::constants::buttons::{
    K_BUTTON_A, K_BUTTON_B, K_BUTTON_DOWN, K_BUTTON_L, K_BUTTON_LEFT, K_BUTTON_R, K_BUTTON_RIGHT,
    K_BUTTON_SELECT, K_BUTTON_START, K_BUTTON_UP, K_BUTTON_X, K_BUTTON_Y,
};

/// What a controller sample reaches: the gameplay context's input and debug
/// words, the projected words, the observed inputs, the display mirrors, the
/// sequence words, the soft reset, and the controllers as the auto-joypad read
/// holds them. Every menu host is one; a Session lease is one too.
pub trait SampleHost:
    crate::gameplay_context::Access
    + crate::projection::Views
    + crate::ram::CompatibilityRam
    + crate::display::preparation::Access
    + crate::sequence_state::Access
    + super::host::ResetRequest
{
    /// `controller`'s word as the auto-joypad read holds it now.
    fn sampled_controller_word(&mut self, controller: Controller) -> u16;
}
impl<T: Host + crate::runtime_access::RuntimeAccess + ?Sized> SampleHost for T {
    fn sampled_controller_word(&mut self, controller: Controller) -> u16 {
        self.controller_word(controller)
    }
}

fn repeat<H: SampleHost + ?Sized>(offset: usize, sampled: u16, game: &mut H) {
    use crate::{controller_state::Word as Input, gameplay_context::Word};
    let [last, pressed, repeated, previous, counter] = match offset {
        0 => [
            Input::Joypad1Lastkeys,
            Input::Joypad1Newkeys,
            Input::Joypad1Newkeys2Unused,
            Input::Joypad1Prev,
            Input::Joypad1KeyrepeatCtr,
        ],
        2 => [
            Input::Joypad2Last,
            Input::Joypad2NewKeys,
            Input::Joypad2Newkeys2,
            Input::Joypad2Prev,
            Input::Joypad2KeyrepeatCtr,
        ],
        _ => unreachable!("controller repeat has exactly two fixed callers"),
    }
    .map(Word::Input);
    game.set_gameplay_word(last, sampled);
    game.set_gameplay_word(
        pressed,
        crate::controller_state::pressed(sampled, game.gameplay_word(previous)),
    );
    game.set_gameplay_word(
        repeated,
        crate::controller_state::pressed(sampled, game.gameplay_word(previous)),
    );
    if sampled != 0 && sampled == game.gameplay_word(previous) {
        game.set_gameplay_word(counter, game.gameplay_word(counter).wrapping_sub(1));
        if game.gameplay_word(counter) == 0 {
            game.set_gameplay_word(repeated, game.gameplay_word(last));
            game.set_gameplay_word(counter, crate::gameplay_word!(game.JOYPAD_CTR_REPEAT_NEXT));
        }
    } else {
        game.set_gameplay_word(counter, crate::gameplay_word!(game.JOYPAD_CTR_REPEAT_FIRST));
    }
    game.set_gameplay_word(previous, game.gameplay_word(last));
}

pub fn filter<H: Host + ?Sized>(delay: u16, game: &mut H) {
    crate::gameplay_word!(game.TIMED_HELD_INPUT_TIMER_RESET = delay);
    let held = !game.observed_inputs().pressed & game.observed_inputs().held;
    let unchanged = held == crate::gameplay_word!(game.JOYPAD_RELEASED_KEYS);
    crate::gameplay_word!(game.JOYPAD_RELEASED_KEYS = held);
    if !unchanged {
        crate::gameplay_word!(
            game.TIMED_HELD_INPUT_TIMER = crate::gameplay_word!(game.TIMED_HELD_INPUT_TIMER_RESET)
        );
        crate::gameplay_word!(game.TIMED_HELD_INPUT = 0);
    } else if crate::gameplay_word!(game.TIMED_HELD_INPUT_TIMER -= 1, reread) & 0x8000 == 0 {
        crate::gameplay_word!(game.TIMED_HELD_INPUT = 0);
    } else {
        crate::gameplay_word! { game;
            TIMED_HELD_INPUT_TIMER = 0;
            PREVIOUS_TIMED_HELD_INPUT = crate::gameplay_word!(game.TIMED_HELD_INPUT);
            TIMED_HELD_INPUT = held;
        }
    }
    crate::gameplay_word!(
        game.NEWLY_HELD_DOWN_TIMED_HELD_INPUT = crate::gameplay_word!(game.TIMED_HELD_INPUT)
            & (crate::gameplay_word!(game.PREVIOUS_TIMED_HELD_INPUT)
                ^ crate::gameplay_word!(game.TIMED_HELD_INPUT))
    );
}

/// Requires a live, exclusively accessed SNES backend for controller register reads.
pub fn sample<H: SampleHost + ?Sized>(game: &mut H) {
    let first = game.sampled_controller_word(Controller::One);
    sample_word(first, game);
}

/// Sample the controller as a requested NMI does: from the word auto-read
/// latched at this vblank, which the next frame's logic reads.
pub fn sample_latched<H: Host + ?Sized>(game: &mut H) {
    let first = game.nmi_controller_word();
    sample_word(first, game);
}

fn sample_word<H: SampleHost + ?Sized>(first: u16, game: &mut H) {
    repeat(0, first, game);
    if crate::gameplay_word!(game.ENABLE_DEBUG) != 0 {
        let second = game.sampled_controller_word(Controller::Two);
        repeat(2, second, game);
        debug_controls(game);
    }
}

fn debug_controls<H: SampleHost + ?Sized>(game: &mut H) {
    let reset = (K_BUTTON_SELECT | K_BUTTON_START | K_BUTTON_L | K_BUTTON_R) as u16;
    if game.view_word(crate::projection::Field::ApuUploading) != 0
        || game.observed_inputs().held != reset
    {
        if crate::gameplay_word!(game.ENABLE_DEBUG) != 0 {
            crate::gameplay_word! { game, 0; JOYPAD_DBG_1, JOYPAD_DBG_2 }
            if (crate::gameplay_word!(game.JOYPAD_DBG_FLAGS) & 0x4000) == 0 {
                for (button, target) in [
                    (
                        K_BUTTON_L,
                        crate::gameplay_context::Word::Loop(crate::game_loop::Word::JoypadDbg1),
                    ),
                    (
                        K_BUTTON_R,
                        crate::gameplay_context::Word::Loop(crate::game_loop::Word::JoypadDbg2),
                    ),
                ] {
                    let chord = (K_BUTTON_SELECT | button) as u16;
                    if game.observed_inputs().held & chord == chord {
                        let value = game.observed_inputs().pressed;
                        crate::gameplay_context::Access::set_gameplay_word(game, target, value);
                        crate::gameplay_word! { game, 0; JOYPAD1_LASTKEYS, JOYPAD1_NEWKEYS }
                    }
                }
                if (crate::gameplay_word!(game.JOYPAD_DBG_2) & 0x80) != 0 {
                    // The original toggles the interrupt enables with a word
                    // store, rewriting the HDMA enable mirror after them.
                    use crate::display::preparation::{Field, Store};
                    let enables = game.display_control_byte(Field::NmiEnable) ^ 0x30;
                    let hdma = game.display_control_byte(Field::HdmaEnable);
                    game.store_display_control_byte(Field::NmiEnable, enables, Store::Named);
                    game.store_display_control_byte(Field::HdmaEnable, hdma, Store::Named);
                }
                if (crate::gameplay_word!(game.JOYPAD_DBG_2) & 0x8000) != 0 {
                    let entering_swap =
                        (!(crate::gameplay_word!(game.JOYPAD_DBG_FLAGS)) & 0x8000) != 0;
                    crate::gameplay_word!(game.JOYPAD_DBG_FLAGS ^= (0x8000u16));
                    if entering_swap {
                        let [missiles, supers, bombs] = crate::gameplay_context::inventory(game)
                            .ammunition
                            .map(|supply| supply.current);
                        crate::gameplay_word! { game;
                            JOYPAD_DBG_MISSILES_SWAP = missiles;
                            JOYPAD_DBG_SUPER_MISSILES_SWAP = supers;
                            JOYPAD_DBG_POWER_BOMBS_SWAP = bombs;
                        }
                        crate::gameplay_context::with_inventory(
                            game,
                            |_, inventory: &mut crate::inventory::Inventory| {
                                for supply in &mut inventory.ammunition {
                                    supply.current = 0;
                                }
                            },
                        );
                    } else {
                        let swapped = [
                            crate::gameplay_word!(game.JOYPAD_DBG_MISSILES_SWAP),
                            crate::gameplay_word!(game.JOYPAD_DBG_SUPER_MISSILES_SWAP),
                            crate::gameplay_word!(game.JOYPAD_DBG_POWER_BOMBS_SWAP),
                        ];
                        crate::gameplay_context::with_inventory(
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
                if (crate::gameplay_word!(game.JOYPAD_DBG_2) & 0x40) != 0 {
                    crate::gameplay_word!(game.JOYPAD_DBG_FLAGS ^= (0x2000u16));
                }
            }
        } else {
            crate::gameplay_word! { game, 0; JOYPAD_DBG_1, JOYPAD_DBG_2 }
            crate::gameplay_word!(game.JOYPAD2_LAST &= (0x10u16));
            crate::gameplay_word!(game.JOYPAD2_NEW_KEYS &= (0x10u16));
        }
    } else {
        crate::sequence_state::with_words(
            game,
            |_, words: &mut crate::session_words::SessionWords| {
                words.debug_disable_sounds = 0;
            },
        );
        game.request_reset();
    }
}

// Intro playback saves only action bindings; its directional defaults remain after restoration.
pub fn use_cinematic_bindings<H: crate::gameplay_context::Access + ?Sized>(game: &mut H) {
    for (current, saved) in [
        (
            crate::gameplay_context::Word::Input(crate::controller_state::Word::ButtonConfigShootX),
            crate::gameplay_context::Word::Loop(crate::game_loop::Word::ButtonConfigShootXSaved),
        ),
        (
            crate::gameplay_context::Word::Input(crate::controller_state::Word::ButtonConfigJumpA),
            crate::gameplay_context::Word::Loop(crate::game_loop::Word::ButtonConfigJumpASaved),
        ),
        (
            crate::gameplay_context::Word::Input(crate::controller_state::Word::ButtonConfigRunB),
            crate::gameplay_context::Word::Loop(crate::game_loop::Word::ButtonConfigRunBSaved),
        ),
        (
            crate::gameplay_context::Word::Input(
                crate::controller_state::Word::ButtonConfigItemcancelY,
            ),
            crate::gameplay_context::Word::Loop(
                crate::game_loop::Word::ButtonConfigItemcancelYSaved,
            ),
        ),
        (
            crate::gameplay_context::Word::Input(
                crate::controller_state::Word::ButtonConfigItemswitch,
            ),
            crate::gameplay_context::Word::Loop(
                crate::game_loop::Word::ButtonConfigItemswitchSaved,
            ),
        ),
        (
            crate::gameplay_context::Word::Input(
                crate::controller_state::Word::ButtonConfigAimDownL,
            ),
            crate::gameplay_context::Word::Loop(crate::game_loop::Word::ButtonConfigAimDownLSaved),
        ),
        (
            crate::gameplay_context::Word::Input(crate::controller_state::Word::ButtonConfigAimUpR),
            crate::gameplay_context::Word::Loop(crate::game_loop::Word::ButtonConfigAimUpRSaved),
        ),
    ] {
        let value = crate::gameplay_context::Access::gameplay_word(game, current);
        crate::gameplay_context::Access::set_gameplay_word(game, saved, value);
    }

    crate::gameplay_word! {game; BUTTON_CONFIG_UP = K_BUTTON_UP as u16; BUTTON_CONFIG_DOWN = K_BUTTON_DOWN as u16;
    BUTTON_CONFIG_LEFT = K_BUTTON_LEFT as u16; BUTTON_CONFIG_RIGHT = K_BUTTON_RIGHT as u16;
    BUTTON_CONFIG_SHOOT_X = K_BUTTON_X as u16; BUTTON_CONFIG_JUMP_A = K_BUTTON_A as u16;
    BUTTON_CONFIG_RUN_B = K_BUTTON_B as u16; BUTTON_CONFIG_ITEMCANCEL_Y = K_BUTTON_Y as u16;
    BUTTON_CONFIG_ITEMSWITCH = K_BUTTON_SELECT as u16;
    BUTTON_CONFIG_AIM_DOWN_L = K_BUTTON_L as u16; BUTTON_CONFIG_AIM_UP_R = K_BUTTON_R as u16; }
}

pub fn restore_cinematic_bindings<H: crate::gameplay_context::Access + ?Sized>(game: &mut H) {
    for (current, saved) in [
        (
            crate::gameplay_context::Word::Input(crate::controller_state::Word::ButtonConfigShootX),
            crate::gameplay_context::Word::Loop(crate::game_loop::Word::ButtonConfigShootXSaved),
        ),
        (
            crate::gameplay_context::Word::Input(crate::controller_state::Word::ButtonConfigJumpA),
            crate::gameplay_context::Word::Loop(crate::game_loop::Word::ButtonConfigJumpASaved),
        ),
        (
            crate::gameplay_context::Word::Input(crate::controller_state::Word::ButtonConfigRunB),
            crate::gameplay_context::Word::Loop(crate::game_loop::Word::ButtonConfigRunBSaved),
        ),
        (
            crate::gameplay_context::Word::Input(
                crate::controller_state::Word::ButtonConfigItemcancelY,
            ),
            crate::gameplay_context::Word::Loop(
                crate::game_loop::Word::ButtonConfigItemcancelYSaved,
            ),
        ),
        (
            crate::gameplay_context::Word::Input(
                crate::controller_state::Word::ButtonConfigItemswitch,
            ),
            crate::gameplay_context::Word::Loop(
                crate::game_loop::Word::ButtonConfigItemswitchSaved,
            ),
        ),
        (
            crate::gameplay_context::Word::Input(
                crate::controller_state::Word::ButtonConfigAimDownL,
            ),
            crate::gameplay_context::Word::Loop(crate::game_loop::Word::ButtonConfigAimDownLSaved),
        ),
        (
            crate::gameplay_context::Word::Input(crate::controller_state::Word::ButtonConfigAimUpR),
            crate::gameplay_context::Word::Loop(crate::game_loop::Word::ButtonConfigAimUpRSaved),
        ),
    ] {
        let value = crate::gameplay_context::Access::gameplay_word(game, saved);
        crate::gameplay_context::Access::set_gameplay_word(game, current, value);
    }
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
        let address = crate::menu_lists::binding_address(game, index);
        let binding = super::named_player_word(game, address);
        let choice = BINDING_BUTTONS[..7]
            .iter()
            .position(|button| binding & button != 0)
            .unwrap_or(0);
        bindings::set_at(game, Array::ControllerBindings, index, choice as u16);
    }
}

pub fn reset_binding_choices<H: Host + ?Sized>(game: &mut H) {
    crate::gameplay_word! {game; BUTTON_CONFIG_SHOOT_X = K_BUTTON_X as u16;
    BUTTON_CONFIG_JUMP_A = K_BUTTON_A as u16; BUTTON_CONFIG_RUN_B = K_BUTTON_B as u16;
    BUTTON_CONFIG_ITEMCANCEL_Y = K_BUTTON_Y as u16; BUTTON_CONFIG_ITEMSWITCH = K_BUTTON_SELECT as u16;
    BUTTON_CONFIG_AIM_UP_R = K_BUTTON_R as u16; BUTTON_CONFIG_AIM_DOWN_L = K_BUTTON_L as u16; }
    load_binding_choices(game);
}

/// The controller binding word at `address`, an entry of the binding
/// address table ($82:F54A).
fn binding_word(address: usize) -> crate::controller_state::Word {
    use crate::controller_state::Word;
    [
        Word::ButtonConfigShootX,
        Word::ButtonConfigJumpA,
        Word::ButtonConfigRunB,
        Word::ButtonConfigItemcancelY,
        Word::ButtonConfigItemswitch,
        Word::ButtonConfigAimDownL,
        Word::ButtonConfigAimUpR,
    ]
    .into_iter()
    .find(|word| word.address() == address)
    .expect("binding address names a controller binding word")
}

pub fn store_binding_choices<H: Host + ?Sized>(game: &mut H) -> u8 {
    for index in 0..7 {
        let binding =
            BINDING_BUTTONS[bindings::get_at(game, Array::ControllerBindings, index) as usize];
        let address = usize::from(crate::menu_lists::binding_address(game, index));
        crate::gameplay_context::Access::set_gameplay_word(
            game,
            crate::gameplay_context::Word::Input(binding_word(address)),
            binding,
        );
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
