//! Shared dialogue, animation and input transitions; external operations consume the owner.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Message<Id> {
    Content(Id),
    MapAcquired,
    EnergyRestored,
    MissilesRestored,
    SaveConfirmation,
    SavePrompt,
    Saved,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cycle {
    Message,
    SaveFeedback,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Opening(Cycle),
    Interacting(Cycle),
    Closing(Cycle),
    FeedbackDelay,
    Running(Script),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Continuation {
    Start,
    Waiting(Phase),
    Invalid(u8),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChildContinuation {
    Start,
    // Animation and confirmation share their first wait token. The invoking
    // phase supplies its meaning, independently of the live outer saved cursor.
    Waiting,
    ReadingDelay,
    Dismissing,
    /// The confirmation's second lag frame before it reads input.
    Polling,
    /// The toggled selection waits a lag frame before its upload.
    Toggling,
    /// The next step of a running script.
    Step(u8),
    Invalid(u8),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Selection {
    Accept,
    Decline,
    Other(u16),
}
impl Selection {
    fn toggle(self) -> Self {
        match self {
            Self::Accept => Self::Decline,
            Self::Decline => Self::Accept,
            Self::Other(value) => Self::Other(value ^ 2),
        }
    }
}
pub struct Input {
    pub confirm: bool,
    pub cancel: bool,
    pub toggle: bool,
    pub dismiss: bool,
}
pub struct Dialogue<Id> {
    pub message: Message<Id>,
    pub continuation: Continuation,
    pub delay_remaining: u16,
    pub child: ChildContinuation,
    pub radius: u16,
    pub selection: Selection,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    CancelSounds,
    /// Select "yes" before the first lag frame.
    ClearSelection,
    /// Save and replace the BG3, blending and scroll registers.
    PrepareDisplay,
    /// Read the BG3 tilemap the box covers back from VRAM.
    SaveBackground,
    FillBackground,
    UploadBackground,
    DrawTopBorder,
    /// The message text and bottom border; the radius starts at 0.
    WriteMessage,
    /// The button letter and the BG3 Y scroll HDMA channel.
    PrepareScroll,
    EnableScroll,
    UploadMessage,
    SaveSound,
    /// Music queue, then sound effects.
    AdvanceAudio,
    AdvanceSoundEffects,
    RestoreBackground,
    RestoreDisplay,
    /// The HDMA object handler, which also handles the music queue.
    UpdateHdma,
    ResumeMovementSound,
    SelectPause,
    DrawRadius,
    StageSelection,
    UploadSelection,
    SelectionSound,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Complete,
    Suspended(std::num::NonZeroU8),
}
pub trait Context {
    type Id;
    // Transfer the state by value: no retained snapshot survives a legacy effect.
    fn effect(&mut self, state: Dialogue<Self::Id>, effect: Effect) -> Dialogue<Self::Id>;
    fn wait(&mut self, state: Dialogue<Self::Id>) -> (Dialogue<Self::Id>, Outcome);
    fn input(&mut self, state: Dialogue<Self::Id>) -> (Dialogue<Self::Id>, Input);
}

/// The original's straight-line stretches of the message box routine ($85:8080).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Script {
    /// Initialise the PPU, clear BG3, draw the message and play 2 lag frames.
    Open,
    /// Clear BG3 and start the saving sound before the 160-frame delay.
    Saving,
    /// Draw the "save completed" message.
    Feedback,
    /// Clear BG3, restore the PPU, then play 2 lag frames.
    Close,
    /// The gunship's close, still without its 2 lag frames ($85:80F2).
    CloseGunship,
}
#[derive(Clone, Copy)]
enum Step {
    Do(Effect),
    /// Wait for a lag frame ($85:8136).
    Wait,
    /// A lag frame, then the music queue and sound effects.
    Lag,
    Delay(u16),
}
mod steps {
    use super::{Effect::*, Step::*, *};
    pub(super) const PREPARE: &[Step] = &[
        Do(CancelSounds),
        Do(ClearSelection),
        Wait,
        Do(PrepareDisplay),
        Wait,
        Do(SaveBackground),
        Do(AdvanceAudio),
    ];
    pub(super) const CLEAR: &[Step] = &[
        Do(FillBackground),
        Wait,
        Do(UploadBackground),
        Do(AdvanceAudio),
    ];
    pub(super) const MESSAGE: &[Step] = &[
        Do(DrawTopBorder),
        Lag,
        Do(WriteMessage),
        Do(PrepareScroll),
        Lag,
        Do(DrawRadius),
        Do(EnableScroll),
        Wait,
        Do(UploadMessage),
        Do(AdvanceAudio),
    ];
    pub(super) const PAUSE: &[Step] = &[Lag, Lag];
    pub(super) const SAVING: &[Step] = &[Do(SaveSound), Delay(160), Do(AdvanceAudio)];
    pub(super) const RESTORE: &[Step] = &[
        Wait,
        Do(RestoreBackground),
        Wait,
        Do(RestoreDisplay),
        Do(UpdateHdma),
        Do(AdvanceSoundEffects),
        Do(ResumeMovementSound),
    ];
}
impl Script {
    fn segments(self) -> &'static [&'static [Step]] {
        use steps::*;
        match self {
            Self::Open => &[PREPARE, CLEAR, MESSAGE, PAUSE],
            Self::Saving => &[CLEAR, SAVING],
            // The gunship's 2 lag frames before it opens ($85:80DC) are still patched.
            Self::Feedback => &[MESSAGE],
            Self::Close => &[CLEAR, RESTORE, PAUSE],
            Self::CloseGunship => &[CLEAR, RESTORE],
        }
    }
}

pub fn run<C: Context>(
    mut state: Dialogue<C::Id>,
    requested: Message<C::Id>,
    context: &mut C,
) -> (Dialogue<C::Id>, Outcome) {
    use Phase::*;
    // The invocation captures its phase. Imported writes to the saved cursor
    // remain observable, but redirect only the next invocation.
    let mut phase = match state.continuation {
        Continuation::Start => {
            state.message = requested;
            Running(Script::Open)
        }
        Continuation::Waiting(phase) => phase,
        Continuation::Invalid(_) => {
            state.continuation = Continuation::Start;
            return (state, Outcome::Complete);
        }
    };
    loop {
        let (next, outcome) = match phase {
            Opening(_) => animate(state, true, context),
            Closing(_) => animate(state, false, context),
            Interacting(_) => interact(state, context),
            FeedbackDelay => context.wait(state),
            Running(script) => run_script(state, script, context),
        };
        state = next;
        if let Outcome::Suspended(_) = outcome {
            state.continuation = Continuation::Waiting(phase);
            return (state, outcome);
        }
        phase = match phase {
            Running(Script::Open) => Opening(Cycle::Message),
            Running(Script::Saving) => FeedbackDelay,
            Running(Script::Feedback) => Opening(Cycle::SaveFeedback),
            Running(Script::Close | Script::CloseGunship) => break,
            Opening(cycle) => Interacting(cycle),
            Interacting(cycle) => Closing(cycle),
            Closing(Cycle::Message) => match state.message {
                Message::SavePrompt if state.selection != Selection::Decline => {
                    state.message = Message::Saved;
                    Running(Script::Saving)
                }
                Message::SavePrompt => Running(Script::CloseGunship),
                _ => Running(Script::Close),
            },
            Closing(Cycle::SaveFeedback) => {
                state.message = Message::SavePrompt;
                Running(Script::CloseGunship)
            }
            FeedbackDelay => {
                state.delay_remaining = state.delay_remaining.wrapping_sub(1);
                if state.delay_remaining == 0 {
                    Running(Script::Feedback)
                } else {
                    state = context.effect(state, Effect::AdvanceAudio);
                    FeedbackDelay
                }
            }
        };
    }
    if matches!(state.message, Message::MapAcquired) {
        state = context.effect(state, Effect::SelectPause);
    }
    state.continuation = Continuation::Start;
    (state, Outcome::Complete)
}

fn run_script<C: Context>(
    mut state: Dialogue<C::Id>,
    script: Script,
    context: &mut C,
) -> (Dialogue<C::Id>, Outcome) {
    let steps = || script.segments().iter().flat_map(|segment| segment.iter());
    let first = match state.child {
        ChildContinuation::Start => 0,
        ChildContinuation::Step(step) if usize::from(step) < steps().count() => usize::from(step),
        _ => {
            state.child = ChildContinuation::Start;
            return (state, Outcome::Complete);
        }
    };
    for (index, step) in steps().enumerate().skip(first) {
        match *step {
            Step::Do(effect) => state = context.effect(state, effect),
            Step::Delay(frames) => state.delay_remaining = frames,
            Step::Wait | Step::Lag => {
                let (next, outcome) = context.wait(state);
                state = next;
                if let Outcome::Suspended(_) = outcome {
                    // Resume at the wait itself: the next invocation completes it.
                    state.child = ChildContinuation::Step(
                        u8::try_from(index).expect("scripts have fewer than 256 steps"),
                    );
                    return (state, outcome);
                }
                if let Step::Lag = step {
                    state = context.effect(state, Effect::AdvanceAudio);
                }
            }
        }
    }
    state.child = ChildContinuation::Start;
    (state, Outcome::Complete)
}

pub fn animate<C: Context>(
    mut state: Dialogue<C::Id>,
    opening: bool,
    context: &mut C,
) -> (Dialogue<C::Id>, Outcome) {
    use ChildContinuation::*;
    match state.child {
        Start if opening => state.radius = 0,
        Start | Waiting => {}
        _ => {
            state.child = Start;
            return (state, Outcome::Complete);
        }
    }
    loop {
        let (next, outcome) = context.wait(state);
        state = next;
        if let Outcome::Suspended(_) = outcome {
            state.child = Waiting;
            return (state, outcome);
        }
        state = context.effect(state, Effect::AdvanceAudio);
        state = context.effect(state, Effect::DrawRadius);
        // Opening presents its last radius; closing commits its underflow.
        if opening && state.radius == 0x1800 {
            break;
        }
        state.radius = if opening {
            state.radius.wrapping_add(0x200)
        } else {
            state.radius.wrapping_sub(0x200)
        };
        if !opening && state.radius & 0x8000 != 0 {
            break;
        }
    }
    state.child = Start;
    (state, Outcome::Complete)
}

pub fn interact<C: Context>(
    mut state: Dialogue<C::Id>,
    context: &mut C,
) -> (Dialogue<C::Id>, Outcome) {
    use ChildContinuation::*;
    // As with the parent, the local phase survives callback cursor mutations.
    let mut phase = match state.child {
        Start => {
            if matches!(
                state.message,
                Message::SaveConfirmation | Message::SavePrompt
            ) {
                state.selection = Selection::Accept;
                Waiting
            } else {
                state.delay_remaining = if matches!(
                    state.message,
                    Message::MapAcquired
                        | Message::EnergyRestored
                        | Message::MissilesRestored
                        | Message::Saved
                ) {
                    10
                } else {
                    360
                };
                ReadingDelay
            }
        }
        Waiting | Polling | Toggling | ReadingDelay | Dismissing => state.child,
        Step(_) | Invalid(_) => {
            state.child = Start;
            return (state, Outcome::Complete);
        }
    };
    loop {
        let (next, outcome) = context.wait(state);
        state = next;
        if let Outcome::Suspended(_) = outcome {
            state.child = phase;
            return (state, outcome);
        }
        if !matches!(phase, Dismissing | Toggling) {
            state = context.effect(state, Effect::AdvanceAudio);
        }
        match phase {
            // The confirmation reads input every second lag frame.
            Waiting => phase = Polling,
            Polling => {
                let (next, input) = context.input(state);
                state = next;
                if input.confirm {
                    break;
                }
                if input.cancel {
                    state.selection = Selection::Decline;
                    break;
                }
                phase = Waiting;
                if input.toggle {
                    state.selection = state.selection.toggle();
                    state = context.effect(state, Effect::StageSelection);
                    phase = Toggling;
                }
            }
            Toggling => {
                for effect in [
                    Effect::UploadSelection,
                    Effect::AdvanceAudio,
                    Effect::SelectionSound,
                ] {
                    state = context.effect(state, effect);
                }
                phase = Waiting;
            }
            ReadingDelay => {
                state.delay_remaining = state.delay_remaining.wrapping_sub(1);
                if state.delay_remaining == 0 {
                    // LOOP_INPUT ($85:84A3) reads the controller in the delay's last frame, so a
                    // held button dismisses the box without another lag frame.
                    let (next, input) = context.input(state);
                    state = next;
                    if input.dismiss {
                        break;
                    }
                    phase = Dismissing;
                }
            }
            Dismissing => {
                let (next, input) = context.input(state);
                state = next;
                if input.dismiss {
                    break;
                }
            }
            _ => unreachable!(),
        }
    }
    state.child = Start;
    (state, Outcome::Complete)
}
