//! Compatibility encoding only; effects consume the model before touching RAM.
#![forbid(unsafe_code)]
use crate::game::dialogue::{Continuation, Cycle, Dialogue, Message, Phase};
use crate::{game_state::Game, ram_map::words as ram};

pub(crate) fn message(id: u16) -> Message<u16> {
    match id {
        20 => Message::MapAcquired,
        24 => Message::Saved,
        28 => Message::SavePrompt,
        id => Message::Content(id),
    }
}
fn read(game: &Game, at: usize) -> u16 {
    u16::from_le_bytes([game.g_ram[at], game.g_ram[at + 1]])
}
fn write(game: &mut Game, at: usize, value: u16) {
    game.g_ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
pub(crate) fn import(game: &Game) -> Dialogue<u16> {
    use Continuation::*;
    use Phase::*;
    Dialogue {
        message: message(read(game, ram::message_box_index)),
        continuation: match game.g_ram[0x614] {
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
        feedback_remaining: read(game, ram::my_counter),
    }
}
pub(crate) fn publish(state: Dialogue<u16>, game: &mut Game) {
    use Continuation::*;
    use Phase::*;
    write(
        game,
        ram::message_box_index,
        match state.message {
            Message::MapAcquired => 20,
            Message::Saved => 24,
            Message::SavePrompt => 28,
            Message::Content(id) => id,
        },
    );
    write(game, ram::my_counter, state.feedback_remaining);
    game.g_ram[0x614] = match state.continuation {
        Start => 0,
        Waiting(Opening(Cycle::Message)) => 2,
        Waiting(Interacting(Cycle::Message)) => 3,
        Waiting(Closing(Cycle::Message)) => 4,
        Waiting(Opening(Cycle::SaveFeedback)) => 5,
        Waiting(Interacting(Cycle::SaveFeedback)) => 6,
        Waiting(Closing(Cycle::SaveFeedback)) => 7,
        Waiting(FeedbackDelay) => 8,
        Invalid(value) => value,
    };
}
pub(crate) fn save_declined(game: &Game) -> bool {
    read(game, ram::save_confirmation_selection) == 2
}
pub(crate) fn select_pause(game: &mut Game) {
    write(game, ram::game_state, 12);
}
