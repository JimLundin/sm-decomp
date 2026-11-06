//! The message box (C13): the item pickup's box as a main strand frame that yields at each
//! of the original's lag-frame waits.
//!
//! Bank $85 log, display message box $85:8080. Each wait is a `JSR $8136` wait for lag frame:
//! the main thread spins until the NMI counter changes, with no NMI requested, so the NMI
//! takes its lag path (no commit, no pad latch) and the gameplay pass, which counts game time,
//! doesn't finish while the box is up. A step runs the work after one wait up to the next and
//! names that next wait as the frame's position. The video work is named here and done by the
//! video lanes; the music and sound effect handlers the lag frames call are the sound
//! domain's.

mod video;

use data::message::{Letter, MessageBoxes, Setup};

use crate::state::slots::Frames;
use crate::state::strand::{
    Frame, LagAt, Message, MessageBoxAt, MessageWrite, PpuPreparation, PpuRestoration, Side, Wait,
};
use crate::state::{Button, State};
use crate::tick::Next;

/// Frames of the open and close animations after the first: the radius moves $200 a frame
/// between 0 and $1800 (bank $85 log, $85:844C and $85:8589).
pub const ANIMATION_STEPS: Frames = Frames(12);

/// The lag frames $85:8574 plays, before the box opens and after it closes.
const TWO_LAG_FRAMES: Frames = Frames(2);

/// Open the box showing `message`: the PLM instruction's `JSL $858080` (C12 calls it).
pub const fn display(state: &mut State, message: Message) -> Next {
    state.message = message;
    Next::Call(Frame::MessageBox(MessageBoxAt::Start))
}

/// A piece of the box's work, in the original's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Work {
    /// $85:8080: cancel the sound effects.
    CancelSoundEffects,
    /// $85:814B: HDMA off, the BG3 colours, layers and scroll registers.
    PrepareRegisters,
    /// $85:81B5: back up the BG3 tilemap the box covers.
    BackUpTilemap,
    /// $85:8206: upload the cleared box tilemap to VRAM.
    UploadClearedTilemap,
    /// $85:82BB: write the message into the box's tilemap.
    WriteMessageTilemap,
    /// $85:8363: point HDMA channel 6 at the BG3 Y scroll table.
    SetUpScrollHdma,
    /// $85:859F: the music queue and sound effects, then the BG3 Y scroll HDMA table for the
    /// current radius.
    WriteScrollTable,
    /// $85:83BD: enable the BG3 Y scroll HDMA channel.
    EnableScrollHdma,
    /// $85:8324: upload the message tilemap to VRAM.
    UploadMessageTilemap,
    /// $85:857B and $85:8496: the music queue and sound effects of a lag frame.
    LagFrameSound,
    /// $85:861F: restore the BG3 tilemap from its backup.
    RestoreTilemap,
    /// $85:8654: HDMA, the BG3 tilemap base, layers, colour math and BG3 colours.
    RestoreRegisters,
    /// $85:80B0: queue Samus's movement sound effects.
    QueueSamusMovementSound,
}

/// The buttons the controls with a special button are bound to: the controller settings'
/// defaults.
// open: the controller settings are the player's options, kept in work RAM and saved; they
// join the State with the options menu.
const SHOOT_BUTTON: Button = Button::X;
const RUN_BUTTON: Button = Button::B;

/// The letter that shows `button` on a special button's tile.
const fn letter(button: Button) -> Letter {
    match button {
        Button::A => Letter::A,
        Button::B => Letter::B,
        Button::X => Letter::X,
        Button::Y => Letter::Y,
        Button::Select => Letter::Select,
        Button::L => Letter::L,
        Button::R => Letter::R,
        Button::Start | Button::Up | Button::Down | Button::Left | Button::Right => Letter::Blank,
    }
}

/// Do a piece of the box's work.
// open: the scroll table and HDMA's enable are video work still to come (B6); the music queue
// and sound effects are C11's.
fn work(work: Work, state: &mut State, boxes: &MessageBoxes) {
    match work {
        Work::PrepareRegisters => state.video.prepare_message_box(),
        Work::BackUpTilemap => state.video.back_up_message_box_tilemap(),
        Work::UploadClearedTilemap => state.video.upload_blank_message_box(),
        Work::SetUpScrollHdma => state.video.set_up_message_box_scroll(),
        Work::RestoreTilemap => state.video.restore_message_box_tilemap(),
        Work::RestoreRegisters => state.video.restore_message_box_registers(),
        Work::UploadMessageTilemap => upload_message(state, boxes),
        Work::CancelSoundEffects
        | Work::WriteMessageTilemap
        | Work::WriteScrollTable
        | Work::EnableScrollHdma
        | Work::LagFrameSound
        | Work::QueueSamusMovementSound => {}
    }
}

/// Bank $85 log, write message tilemap `$85:82B8` and set up PPU for active message box
/// `$85:8324`: the box's rows, the top border, the message and the bottom border, with the
/// special button's letter drawn in, upload to BG3's tilemap where the setup puts them.
///
/// The original builds the rows in work RAM when the message is written and uploads them one
/// lag frame later; nothing else uses the buffer in between, so they are built here.
fn upload_message(state: &mut State, boxes: &MessageBoxes) {
    let Some(message) = boxes.message(state.message.0) else {
        return;
    };
    let border = boxes.border(message.borders);
    let mut rows = Vec::with_capacity(
        border
            .len()
            .saturating_mul(2)
            .saturating_add(message.tilemap.len()),
    );
    rows.extend_from_slice(border);
    rows.extend_from_slice(&message.tilemap);
    rows.extend_from_slice(border);
    let pressed = match message.setup {
        Setup::ShootButton => Some(SHOOT_BUTTON),
        Setup::RunButton => Some(RUN_BUTTON),
        Setup::Small | Setup::Large => None,
    };
    if let Some(tile) = pressed.and_then(|button| boxes.letter(letter(button)))
        && let Some(word) = rows.get_mut(usize::from(message.button / 2))
    {
        *word = tile;
    }
    state.video.upload_message_box(&rows, message.setup.rows());
}

/// The box waits for the next lag frame at `at`.
const fn wait(at: MessageBoxAt) -> (MessageBoxAt, Next) {
    (at, Next::Yield(Wait::LagFrame))
}

/// The first of a lag countdown's frames: X loaded with `frames`, then the wait. X counts
/// the frame being waited for, so `left` is one less.
const fn countdown(at: LagAt, frames: Frames) -> (MessageBoxAt, Next) {
    wait(MessageBoxAt::Lag {
        at,
        left: Frames(frames.0.saturating_sub(1)),
    })
}

/// One step of the box from `at`: the work after that wait, up to the next wait or the
/// return.
pub fn step(at: MessageBoxAt, state: &mut State, boxes: &MessageBoxes) -> (MessageBoxAt, Next) {
    match at {
        // $85:8080: the message index stored, sound effects cancelled, then $808D JSR $8143
        // initialise PPU, whose first act is $8148 JSR $8136.
        MessageBoxAt::Start => {
            work(Work::CancelSoundEffects, state, boxes);
            wait(MessageBoxAt::PreparingPpu(PpuPreparation::Registers))
        }
        MessageBoxAt::PreparingPpu(PpuPreparation::Registers) => {
            work(Work::PrepareRegisters, state, boxes);
            // $81B2 JSR $8136.
            wait(MessageBoxAt::PreparingPpu(PpuPreparation::TilemapBackup))
        }
        MessageBoxAt::PreparingPpu(PpuPreparation::TilemapBackup) => {
            work(Work::BackUpTilemap, state, boxes);
            // RTS, then $8090 JSR $81F3 clear tilemap: $8203 JSR $8136.
            wait(MessageBoxAt::ClearingTilemap(Side::BeforeOpen))
        }
        MessageBoxAt::ClearingTilemap(side) => {
            work(Work::UploadClearedTilemap, state, boxes);
            match side {
                // $8093 JSR $8241 initialise message box: the tilemap writer reaches $82B8
                // JSR $8136.
                Side::BeforeOpen => wait(MessageBoxAt::Writing(MessageWrite::Tilemap)),
                // $80AD JSR $861A restore PPU: $861C JSR $8136.
                Side::AfterClose => wait(MessageBoxAt::RestoringPpu(PpuRestoration::Tilemap)),
            }
        }
        MessageBoxAt::Writing(MessageWrite::Tilemap) => {
            work(Work::WriteMessageTilemap, state, boxes);
            work(Work::SetUpScrollHdma, state, boxes);
            // The PPU set-up's $831E JSR $8363 scroll HDMA: $83BA JSR $859B, whose $859C JSR
            // $8136 waits before the table is written.
            wait(MessageBoxAt::Writing(MessageWrite::ScrollHdma))
        }
        MessageBoxAt::Writing(MessageWrite::ScrollHdma) => {
            work(Work::WriteScrollTable, state, boxes);
            work(Work::EnableScrollHdma, state, boxes);
            // $8321 JSR $8136.
            wait(MessageBoxAt::Writing(MessageWrite::TilemapUpload))
        }
        MessageBoxAt::Writing(MessageWrite::TilemapUpload) => {
            work(Work::UploadMessageTilemap, state, boxes);
            // $8096 JSR $8574 play 2 lag frames: LDX #$02, $8578 JSR $8136.
            countdown(LagAt::BeforeOpen, TWO_LAG_FRAMES)
        }
        MessageBoxAt::Lag { at, left } => {
            work(Work::LagFrameSound, state, boxes);
            // DEX, BNE back to the wait.
            if let Some(left) = left.0.checked_sub(1) {
                return wait(MessageBoxAt::Lag {
                    at,
                    left: Frames(left),
                });
            }
            match at {
                // $8099 JSR $844C open: STZ the radius, $8451 JSR $859B.
                LagAt::BeforeOpen => wait(MessageBoxAt::Opening {
                    left: ANIMATION_STEPS,
                }),
                // LOOP_INPUT at $84A3 tests the pad at once.
                LagAt::Shown => button_wait(state),
                // $80B7: the display routine pulls its registers and returns (RTL).
                LagAt::AfterClose => (MessageBoxAt::Lag { at, left }, Next::Return),
            }
        }
        MessageBoxAt::Opening { left } => {
            work(Work::WriteScrollTable, state, boxes);
            match left.0.checked_sub(1) {
                // The radius grows by $200, then JSR $859B again.
                Some(left) => wait(MessageBoxAt::Opening { left: Frames(left) }),
                // Open: $80A4 JSR $846D interaction, its countdown's $8493 JSR $8136.
                None => countdown(LagAt::Shown, state.message.shown_lag()),
            }
        }
        MessageBoxAt::Shown => button_wait(state),
        MessageBoxAt::Closing { left } => {
            work(Work::WriteScrollTable, state, boxes);
            // The radius shrinks by $200 and, while it stays non-negative, JSR $859B again;
            // once closed, $80AA JSR $81F3 clear tilemap, $8203 JSR $8136.
            wait(
                left.0
                    .checked_sub(1)
                    .map_or(MessageBoxAt::ClearingTilemap(Side::AfterClose), |left| {
                        MessageBoxAt::Closing { left: Frames(left) }
                    }),
            )
        }
        MessageBoxAt::RestoringPpu(PpuRestoration::Tilemap) => {
            work(Work::RestoreTilemap, state, boxes);
            // $8651 JSR $8136.
            wait(MessageBoxAt::RestoringPpu(PpuRestoration::Registers))
        }
        MessageBoxAt::RestoringPpu(PpuRestoration::Registers) => {
            work(Work::RestoreRegisters, state, boxes);
            work(Work::QueueSamusMovementSound, state, boxes);
            // $80B4 JSR $8574 play 2 lag frames.
            countdown(LagAt::AfterClose, TWO_LAG_FRAMES)
        }
    }
}

/// Bank $85 log, $85:846D's `LOOP_INPUT` ($84A3-$84B2): it reads the auto-read pad directly,
/// with no NMI requested, until any button is held; then $80A7 JSR $8589 close, whose $858B
/// JSR $859B waits at the full radius.
// open: the save confirmation's Yes/No loop (messages $17 and $1C) joins with saving.
const fn button_wait(state: &State) -> (MessageBoxAt, Next) {
    if state.controller.auto_read.is_empty() {
        wait(MessageBoxAt::Shown)
    } else {
        wait(MessageBoxAt::Closing {
            left: ANIMATION_STEPS,
        })
    }
}

#[cfg(test)]
mod tests {
    use data::message::{BorderSize, Message as Definition, MessageBoxes, Setup};

    use super::{step, upload_message};
    use crate::state::State;
    use crate::state::strand::{Message, MessageBoxAt, MessageWrite};

    fn showing(message: u16) -> State {
        State {
            message: Message(message),
            ..State::default()
        }
    }

    fn boxes() -> MessageBoxes {
        let message = |setup, borders, button| Definition {
            setup,
            borders,
            button,
            tilemap: vec![0xA, 0xB],
        };
        MessageBoxes::new(
            vec![0x10; 3],
            vec![0x20; 3],
            vec![0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37],
            vec![
                message(Setup::Small, BorderSize::Small, 0),
                message(Setup::ShootButton, BorderSize::Large, 6),
            ],
        )
    }

    #[test]
    fn a_small_box_uploads_its_borders_around_the_message_at_the_small_rows() {
        let mut state = showing(1);
        upload_message(&mut state, &boxes());
        assert_eq!(
            state.video.memory.vram[0x59C0..0x59C8],
            [0x20, 0x20, 0x20, 0xA, 0xB, 0x20, 0x20, 0x20]
        );
        assert_eq!(state.video.dma[1].source, 0x3200 + 16);
    }

    #[test]
    fn a_large_box_draws_the_shoot_buttons_default_letter() {
        let mut state = showing(2);
        upload_message(&mut state, &boxes());
        // Offset 6 bytes is the fourth word, over the message's first, where the Shoot
        // control's default button X is drawn.
        assert_eq!(
            state.video.memory.vram[0x59A0..0x59A8],
            [0x10, 0x10, 0x10, 0x32, 0xB, 0x10, 0x10, 0x10]
        );
    }

    #[test]
    fn a_message_without_a_definition_uploads_nothing() {
        let mut state = showing(0);
        upload_message(&mut state, &boxes());
        assert!(state.video.memory.vram.iter().all(|&word| word == 0));
    }

    #[test]
    fn the_upload_step_writes_the_message_and_moves_on() {
        let mut state = showing(1);
        let (at, _) = step(
            MessageBoxAt::Writing(MessageWrite::TilemapUpload),
            &mut state,
            &boxes(),
        );
        assert!(matches!(at, MessageBoxAt::Lag { .. }));
        assert_eq!(state.video.memory.vram[0x59C3], 0xA);
    }
}
