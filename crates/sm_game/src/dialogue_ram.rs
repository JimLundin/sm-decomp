//! Compatibility encoding only; effects consume the model before touching RAM.
#![forbid(unsafe_code)]
use crate::dialogue::{
    ChildContinuation, Continuation, Cycle, Dialogue, Input, Message, Phase, Selection,
};
use crate::ram::RawRam as _;
use crate::ram::words as ram;

pub fn message(id: u16) -> Message<u16> {
    match id {
        20 => Message::MapAcquired,
        21 => Message::EnergyRestored,
        22 => Message::MissilesRestored,
        23 => Message::SaveConfirmation,
        24 => Message::Saved,
        28 => Message::SavePrompt,
        id => Message::Content(id),
    }
}
fn read<H: crate::ram::RamHost + ?Sized>(game: &H, at: usize) -> u16 {
    u16::from_le_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)])
}
fn write<H: crate::ram::RamHost + ?Sized>(game: &mut H, at: usize, value: u16) {
    game.ram().copy_in(at, &value.to_le_bytes());
}
pub fn import<H: crate::ram::RamHost + ?Sized>(game: &H) -> Dialogue<u16> {
    use Continuation::*;
    use Phase::*;
    Dialogue {
        message: message(read(game, ram::message_box_index)),
        continuation: match game.ram().read_byte(0x614) {
            0 => Start,
            2 => Waiting(Opening(Cycle::Message)),
            3 => Waiting(Interacting(Cycle::Message)),
            4 => Waiting(Closing(Cycle::Message)),
            5 => Waiting(Opening(Cycle::SaveFeedback)),
            6 => Waiting(Interacting(Cycle::SaveFeedback)),
            7 => Waiting(Closing(Cycle::SaveFeedback)),
            8 => Waiting(FeedbackDelay),
            value => Invalid(value),
        },
        delay_remaining: read(game, ram::my_counter),
        child: match game.ram().read_byte(0x615) {
            0 => ChildContinuation::Start,
            1 => ChildContinuation::Waiting,
            2 => ChildContinuation::ReadingDelay,
            3 => ChildContinuation::Dismissing,
            value => ChildContinuation::Invalid(value),
        },
        radius: read(game, ram::message_box_animation_y_radius),
        selection: match read(game, ram::save_confirmation_selection) {
            0 => Selection::Accept,
            2 => Selection::Decline,
            value => Selection::Other(value),
        },
    }
}
pub fn publish<H: crate::ram::RamHost + ?Sized>(state: Dialogue<u16>, game: &mut H) {
    use Continuation::*;
    use Phase::*;
    write(
        game,
        ram::message_box_index,
        match state.message {
            Message::MapAcquired => 20,
            Message::EnergyRestored => 21,
            Message::MissilesRestored => 22,
            Message::SaveConfirmation => 23,
            Message::Saved => 24,
            Message::SavePrompt => 28,
            Message::Content(id) => id,
        },
    );
    write(game, ram::my_counter, state.delay_remaining);
    write(game, ram::message_box_animation_y_radius, state.radius);
    write(
        game,
        ram::save_confirmation_selection,
        match state.selection {
            Selection::Accept => 0,
            Selection::Decline => 2,
            Selection::Other(value) => value,
        },
    );
    game.ram().write_byte(
        0x615,
        match state.child {
            ChildContinuation::Start => 0,
            ChildContinuation::Waiting => 1,
            ChildContinuation::ReadingDelay => 2,
            ChildContinuation::Dismissing => 3,
            ChildContinuation::Invalid(value) => value,
        },
    );
    game.ram().write_byte(
        0x614,
        match state.continuation {
            Start => 0,
            Waiting(Opening(Cycle::Message)) => 2,
            Waiting(Interacting(Cycle::Message)) => 3,
            Waiting(Closing(Cycle::Message)) => 4,
            Waiting(Opening(Cycle::SaveFeedback)) => 5,
            Waiting(Interacting(Cycle::SaveFeedback)) => 6,
            Waiting(Closing(Cycle::SaveFeedback)) => 7,
            Waiting(FeedbackDelay) => 8,
            Invalid(value) => value,
        },
    );
}
pub fn input<H: crate::ram::RamHost + ?Sized>(game: &H) -> Input {
    use crate::constants::buttons::*;
    let keys = read(game, ram::joypad1_newkeys);
    let dismiss = if read(game, ram::bug_fix_counter) == 0 {
        keys != 0
    } else {
        read(game, ram::joypad1_lastkeys) != 0
    };
    Input {
        confirm: keys & K_BUTTON_A as u16 != 0,
        cancel: keys & K_BUTTON_B as u16 != 0,
        toggle: keys & (K_BUTTON_SELECT | K_BUTTON_LEFT | K_BUTTON_RIGHT) as u16 != 0,
        dismiss,
    }
}
pub fn select_pause<H: crate::ram::RamHost + ?Sized>(game: &mut H) {
    write(game, ram::game_state, 12);
}
