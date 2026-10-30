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
    PrepareDisplay,
    ClearDisplay,
    PrepareMessage,
    SaveSound,
    AdvanceAudio,
    RestoreDisplay,
    ResumeMovementSound,
    SelectPause,
    DrawRadius,
    DrawSelection,
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
pub fn run<C: Context>(
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
        let (next, outcome) = match phase {
            Opening(_) => animate(state, true, context),
            Closing(_) => animate(state, false, context),
            Interacting(_) => interact(state, context),
            FeedbackDelay => context.wait(state),
        };
        state = next;
        if let Outcome::Suspended(_) = outcome {
            state.continuation = Continuation::Waiting(phase);
            return (state, outcome);
        }
        phase = match phase {
            Opening(cycle) => Interacting(cycle),
            Interacting(cycle) => Closing(cycle),
            Closing(Cycle::Message) => {
                if !matches!(state.message, Message::SavePrompt)
                    || state.selection == Selection::Decline
                {
                    break;
                }
                state.message = Message::Saved;
                state = context.effect(state, ClearDisplay);
                state = context.effect(state, SaveSound);
                state.delay_remaining = 160;
                state = context.effect(state, AdvanceAudio);
                FeedbackDelay
            }
            Closing(Cycle::SaveFeedback) => {
                state.message = Message::SavePrompt;
                break;
            }
            FeedbackDelay => {
                state.delay_remaining = state.delay_remaining.wrapping_sub(1);
                if state.delay_remaining == 0 {
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
        Waiting | ReadingDelay | Dismissing => state.child,
        Invalid(_) => {
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
        if phase != Dismissing {
            state = context.effect(state, Effect::AdvanceAudio);
        }
        match phase {
            Waiting => {
                let (next, input) = context.input(state);
                state = next;
                if input.confirm {
                    break;
                }
                if input.cancel {
                    state.selection = Selection::Decline;
                    break;
                }
                if input.toggle {
                    state.selection = state.selection.toggle();
                    state = context.effect(state, Effect::DrawSelection);
                    state = context.effect(state, Effect::SelectionSound);
                }
            }
            ReadingDelay => {
                state.delay_remaining = state.delay_remaining.wrapping_sub(1);
                if state.delay_remaining == 0 {
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
