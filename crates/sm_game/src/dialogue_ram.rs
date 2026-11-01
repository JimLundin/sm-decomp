//! Compatibility encoding only; effects consume the model before touching RAM.
use crate::dialogue::{
    ChildContinuation, Continuation, Cycle, Dialogue, Input, Message, Phase, Script, Selection,
};
use crate::ram::RawRam as _;
use crate::ram::words as ram;

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
        message: message(read(game, ram::MESSAGE_BOX_INDEX)),
        continuation: match game.ram().read_byte(0x614) {
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
        delay_remaining: read(game, ram::MY_COUNTER),
        child: match game.ram().read_byte(0x615) {
            0 => ChildContinuation::Start,
            1 => ChildContinuation::Waiting,
            2 => ChildContinuation::ReadingDelay,
            3 => ChildContinuation::Dismissing,
            4 => ChildContinuation::Polling,
            5 => ChildContinuation::Toggling,
            value @ STEPS.. => ChildContinuation::Step(value - STEPS),
            value => ChildContinuation::Invalid(value),
        },
        radius: message_box::get(game, message_box::Word::Radius),
        selection: match read(game, ram::SAVE_CONFIRMATION_SELECTION) {
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
        ram::MESSAGE_BOX_INDEX,
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
    write(game, ram::MY_COUNTER, state.delay_remaining);
    message_box::set(game, message_box::Word::Radius, state.radius);
    write(
        game,
        ram::SAVE_CONFIRMATION_SELECTION,
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
            ChildContinuation::Polling => 4,
            ChildContinuation::Toggling => 5,
            ChildContinuation::Step(step) => STEPS + step,
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
            Waiting(Running(Script::Open)) => 9,
            Waiting(Running(Script::Saving)) => 10,
            Waiting(Running(Script::Feedback)) => 11,
            Waiting(Running(Script::Close)) => 12,
            Invalid(value) => value,
        },
    );
}
pub fn input<H: crate::ram::RamHost + ?Sized>(game: &H) -> Input {
    use crate::constants::buttons::*;
    let keys = read(game, ram::JOYPAD1_NEWKEYS);
    Input {
        confirm: keys & K_BUTTON_A as u16 != 0,
        cancel: keys & K_BUTTON_B as u16 != 0,
        toggle: keys & (K_BUTTON_SELECT | K_BUTTON_LEFT | K_BUTTON_RIGHT) as u16 != 0,
        // LOOP_INPUT ($85:84A3) reads the joypad's raw state ($4218/$4219): a held
        // button dismisses the box, as snesrev's bug-fix counter 1 decides too.
        dismiss: read(game, ram::JOYPAD1_LASTKEYS) != 0,
    }
}
pub fn select_pause<H: crate::ram::RamHost + ?Sized>(game: &mut H) {
    write(game, ram::GAME_STATE, 12);
}

/// The message box's animation words, which the Session holds as
/// `messages.box_radius` and `messages.box_scratch`, read and stored through
/// the host's projected bytes.
pub mod message_box {
    use crate::ram::CompatibilityRam;
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Word {
        Radius = 0x5a2,
        Scratch0 = 0x5a4,
        Scratch1 = 0x5a6,
        Scratch2 = 0x5a8,
        Scratch3 = 0x5aa,
    }
    /// The scratch words in RAM order.
    pub const SCRATCH: [Word; 4] = [
        Word::Scratch0,
        Word::Scratch1,
        Word::Scratch2,
        Word::Scratch3,
    ];
    pub fn get<H: CompatibilityRam + ?Sized>(h: &H, word: Word) -> u16 {
        crate::projection::word(h, word as usize)
    }
    pub fn set<H: CompatibilityRam + ?Sized>(h: &mut H, word: Word, value: u16) {
        crate::projection::set_word(h, word as usize, value);
    }
}
