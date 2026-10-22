//! Shared dialogue control; presentation and input retain their own child waits.
#![forbid(unsafe_code)]

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Message<Id> {
    Content(Id),
    MapAcquired,
    SavePrompt,
    Saved,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Cycle {
    Message,
    SaveFeedback,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Phase {
    Opening(Cycle),
    Interacting(Cycle),
    Closing(Cycle),
    FeedbackDelay,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Continuation {
    Start,
    Waiting(Phase),
    Invalid(u8),
}
pub(crate) struct Dialogue<Id> {
    pub message: Message<Id>,
    pub continuation: Continuation,
    pub feedback_remaining: u16,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Effect {
    CancelSounds,
    PrepareDisplay,
    ClearDisplay,
    PrepareMessage,
    SaveSound,
    AdvanceAudio,
    RestoreDisplay,
    ResumeMovementSound,
    SelectPause,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Outcome {
    Complete,
    Suspended(std::num::NonZeroU8),
}
pub(crate) trait Context {
    type Id;
    // Transfer the state by value: no retained snapshot survives a legacy effect.
    fn effect(&mut self, state: Dialogue<Self::Id>, effect: Effect) -> Dialogue<Self::Id>;
    fn poll(&mut self, state: Dialogue<Self::Id>, phase: Phase) -> (Dialogue<Self::Id>, Outcome);
    // Selection is currently owned by the interaction child.
    fn save_declined(&self) -> bool;
}
pub(crate) fn run<C: Context>(
    mut state: Dialogue<C::Id>,
    requested: Message<C::Id>,
    context: &mut C,
) -> (Dialogue<C::Id>, Outcome) {
    use Effect::*;
    use Phase::*;
    // The invocation captures its phase. Imported writes to the saved cursor
    // remain observable, but redirect only the next invocation.
    let mut phase = match state.continuation {
        Continuation::Start => {
            state.message = requested;
            for effect in [CancelSounds, PrepareDisplay, ClearDisplay, PrepareMessage] {
                state = context.effect(state, effect);
            }
            Opening(Cycle::Message)
        }
        Continuation::Waiting(phase) => phase,
        Continuation::Invalid(_) => {
            state.continuation = Continuation::Start;
            return (state, Outcome::Complete);
        }
    };
    loop {
        let (next, outcome) = context.poll(state, phase);
        state = next;
        if let Outcome::Suspended(_) = outcome {
            state.continuation = Continuation::Waiting(phase);
            return (state, outcome);
        }
        phase = match phase {
            Opening(cycle) => Interacting(cycle),
            Interacting(cycle) => Closing(cycle),
            Closing(Cycle::Message) => {
                if !matches!(state.message, Message::SavePrompt) || context.save_declined() {
                    break;
                }
                state.message = Message::Saved;
                state = context.effect(state, ClearDisplay);
                state = context.effect(state, SaveSound);
                state.feedback_remaining = 160;
                state = context.effect(state, AdvanceAudio);
                FeedbackDelay
            }
            Closing(Cycle::SaveFeedback) => {
                state.message = Message::SavePrompt;
                break;
            }
            FeedbackDelay => {
                state.feedback_remaining = state.feedback_remaining.wrapping_sub(1);
                if state.feedback_remaining == 0 {
                    state = context.effect(state, PrepareMessage);
                    Opening(Cycle::SaveFeedback)
                } else {
                    state = context.effect(state, AdvanceAudio);
                    FeedbackDelay
                }
            }
        };
    }
    for effect in [ClearDisplay, RestoreDisplay, ResumeMovementSound] {
        state = context.effect(state, effect);
    }
    if matches!(state.message, Message::MapAcquired) {
        state = context.effect(state, SelectPause);
    }
    state.continuation = Continuation::Start;
    (state, Outcome::Complete)
}
