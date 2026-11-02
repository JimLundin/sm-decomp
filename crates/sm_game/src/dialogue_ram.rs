//! Dialogue control encoding and access to the existing presentation owner.
use crate::dialogue::{
    ChildContinuation, Continuation, Cycle, Dialogue, Input, Message, Phase, Script, Selection,
};

/// Script steps encode above the child's named continuations.
const STEPS: u8 = 16;

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
/// The existing control continuation lanes and message state at a boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Encoded {
    pub index: u16,
    pub parent: u8,
    pub child: u8,
    pub delay: u16,
    pub radius: u16,
    pub selection: u16,
}
/// Implementations use one current Session owner and publish before effects.
pub trait Access {
    fn dialogue_state(&self) -> Dialogue<u16>;
    fn dialogue_publish(&mut self, state: Dialogue<u16>);
    fn dialogue_request(&self) -> u16;
    fn dialogue_set_request(&mut self, value: u16);
    fn dialogue_active_index(&self) -> u16;
    fn dialogue_clear_active(&mut self);
    fn dialogue_selection(&self) -> u16;
    fn dialogue_set_selection(&mut self, value: u16);
    fn dialogue_scratch(&self) -> [u16; 4];
    fn dialogue_set_scratch(&mut self, index: usize, value: u16);
    fn dialogue_radius(&self) -> u16;
    fn dialogue_set_radius(&mut self, value: u16);
    fn dialogue_input(&self) -> Input;
    fn dialogue_binding(&self, shoot: bool) -> u16;
    fn dialogue_select_pause(&mut self);
}
pub fn import<H: Access + ?Sized>(game: &H) -> Dialogue<u16> {
    game.dialogue_state()
}
pub fn publish<H: Access + ?Sized>(state: Dialogue<u16>, game: &mut H) {
    game.dialogue_publish(state);
}
pub fn decode(encoded: Encoded) -> Dialogue<u16> {
    use Continuation::*;
    use Phase::*;
    Dialogue {
        message: message(encoded.index),
        continuation: match encoded.parent {
            0 => Start,
            2 => Waiting(Opening(Cycle::Message)),
            3 => Waiting(Interacting(Cycle::Message)),
            4 => Waiting(Closing(Cycle::Message)),
            5 => Waiting(Opening(Cycle::SaveFeedback)),
            6 => Waiting(Interacting(Cycle::SaveFeedback)),
            7 => Waiting(Closing(Cycle::SaveFeedback)),
            8 => Waiting(FeedbackDelay),
            9 => Waiting(Running(Script::Open)),
            10 => Waiting(Running(Script::Saving)),
            11 => Waiting(Running(Script::Feedback)),
            12 => Waiting(Running(Script::Close)),
            value => Invalid(value),
        },
        delay_remaining: encoded.delay,
        child: match encoded.child {
            0 => ChildContinuation::Start,
            1 => ChildContinuation::Waiting,
            2 => ChildContinuation::ReadingDelay,
            3 => ChildContinuation::Dismissing,
            4 => ChildContinuation::Polling,
            5 => ChildContinuation::Toggling,
            value @ STEPS.. => ChildContinuation::Step(value - STEPS),
            value => ChildContinuation::Invalid(value),
        },
        radius: encoded.radius,
        selection: match encoded.selection {
            0 => Selection::Accept,
            2 => Selection::Decline,
            value => Selection::Other(value),
        },
    }
}
pub fn encode(state: Dialogue<u16>) -> Encoded {
    use Continuation::*;
    use Phase::*;
    Encoded {
        index: match state.message {
            Message::MapAcquired => 20,
            Message::EnergyRestored => 21,
            Message::MissilesRestored => 22,
            Message::SaveConfirmation => 23,
            Message::Saved => 24,
            Message::SavePrompt => 28,
            Message::Content(id) => id,
        },
        delay: state.delay_remaining,
        radius: state.radius,
        selection: match state.selection {
            Selection::Accept => 0,
            Selection::Decline => 2,
            Selection::Other(value) => value,
        },
        child: match state.child {
            ChildContinuation::Start => 0,
            ChildContinuation::Waiting => 1,
            ChildContinuation::ReadingDelay => 2,
            ChildContinuation::Dismissing => 3,
            ChildContinuation::Polling => 4,
            ChildContinuation::Toggling => 5,
            ChildContinuation::Step(step) => STEPS + step,
            ChildContinuation::Invalid(value) => value,
        },
        parent: match state.continuation {
            Start => 0,
            Waiting(Opening(Cycle::Message)) => 2,
            Waiting(Interacting(Cycle::Message)) => 3,
            Waiting(Closing(Cycle::Message)) => 4,
            Waiting(Opening(Cycle::SaveFeedback)) => 5,
            Waiting(Interacting(Cycle::SaveFeedback)) => 6,
            Waiting(Closing(Cycle::SaveFeedback)) => 7,
            Waiting(FeedbackDelay) => 8,
            Waiting(Running(Script::Open)) => 9,
            Waiting(Running(Script::Saving)) => 10,
            Waiting(Running(Script::Feedback)) => 11,
            Waiting(Running(Script::Close)) => 12,
            Invalid(value) => value,
        },
    }
}
pub fn input<H: Access + ?Sized>(game: &H) -> Input {
    game.dialogue_input()
}
pub fn select_pause<H: Access + ?Sized>(game: &mut H) {
    game.dialogue_select_pause();
}
