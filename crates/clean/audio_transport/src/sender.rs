//! Spec: SenderTiming. CPU service milestones observe the platform reply at the read.
//!
//! CPU time and all service deadlines are unbounded exact whole master ticks.
use crate::{
    CpuMasterTicks, CueId, OfferPermission, QueueAllowance, Request, SenderCompletion,
    SenderProgress, SenderResult, SenderState,
};
use std::num::NonZeroU8;
use std::ops::{Add, Sub};
/// Spec: OfferAcceptance. Offers occur only between services, never remove old cues.
pub fn offer(
    state: &mut SenderState,
    cue: CueId,
    allowance: QueueAllowance,
    permission: OfferPermission,
) {
    let limit = match allowance {
        QueueAllowance::One => 1,
        QueueAllowance::Three => 3,
        QueueAllowance::Six => 6,
        QueueAllowance::Nine => 9,
        QueueAllowance::Fifteen => 15,
    };
    if permission == OfferPermission::Enabled && state.queue.len() < limit {
        state.queue.push_back(cue);
    }
}
/// Spec: SenderTiming. Public ordered milestone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SenderOutput {
    /// Reply sampled at this CPU instant.
    Observation(CpuMasterTicks, Request),
    /// Request presented at this CPU instant.
    Presentation(CpuMasterTicks, Request),
    /// Result became committed here.
    Completed(CpuMasterTicks),
}
/// Spec: SenderTiming. The platform supplies this after validating execution and queue invariants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SenderProfile {
    /// Admitted CPU master-clock sender profile, while delivery is active.
    FastBus,
}
/// Spec: SenderTiming. Begin without sampling a future reply.
pub fn begin(state: &mut SenderState, profile: SenderProfile) {
    const READ: u64 = 58;
    let SenderProfile::FastBus = profile;
    if state.pending.is_some() {
        return;
    }
    let observes = matches!(
        state.progress,
        SenderProgress::AwaitCue(_) | SenderProgress::AwaitClear
    );
    let result = if observes {
        None
    } else {
        Some(select(state, &state.now))
    };
    state.pending = Some(SenderCompletion {
        origin: state.now.clone(),
        observation: observes.then(|| crate::ExactCpuMasterTicks(Add::add(&state.now.0, READ))),
        result,
    });
}
/// Spec: SenderTiming. Platform update; an update at the observation instant precedes its read.
pub const fn reply(state: &mut SenderState, value: Request) {
    state.reply = value;
}
/// Spec: SenderTiming. Platform-owned reply update on the CPU clock.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplyInput {
    /// Exact CPU update instant.
    pub at: CpuMasterTicks,
    /// Reply visible from this instant.
    pub reply: Request,
}
/// Spec: SenderTiming. Ordered updates at an observation instant precede that observation.
///
/// The platform owns these updates, elapsed CPU time and validated FastBus scheduling.
/// Inputs are ordered and fall within this elapsed interval. No future input is retained.
pub fn advance_with_inputs(
    state: &mut SenderState,
    elapsed: CpuMasterTicks,
    inputs: &[ReplyInput],
) -> Vec<SenderOutput> {
    use num_traits::Zero;
    let stop = Add::add(&state.now.0, elapsed.0);
    let mut outputs = Vec::new();
    let mut inputs = inputs.iter().peekable();
    while let Some(input) = inputs.next() {
        let before = if input.at.0.is_zero() {
            input.at.0.clone()
        } else {
            Sub::sub(&input.at.0, 1u64)
        }
        .max(state.now.0.clone());
        let elapsed = crate::ExactCpuMasterTicks(before.sub(&state.now.0));
        outputs.extend(advance(state, elapsed));
        state.now.clone_from(&input.at);
        state.reply = input.reply;
        while inputs.peek().is_some_and(|later| later.at == input.at) {
            if let Some(later) = inputs.next() {
                state.reply = later.reply;
            }
        }
        outputs.extend(advance(state, CpuMasterTicks(0)));
    }
    let remaining = crate::ExactCpuMasterTicks(stop.sub(&state.now.0));
    outputs.extend(advance(state, remaining));
    outputs
}
/// Spec: SenderTiming. Advance CPU time; each milestone is emitted only once.
pub fn advance(state: &mut SenderState, elapsed: CpuMasterTicks) -> Vec<SenderOutput> {
    let stop = Add::add(&state.now.0, elapsed.0);
    let mut outputs = Vec::new();
    let Some(mut pending) = state.pending.take() else {
        state.now = crate::ExactCpuMasterTicks(stop);
        return outputs;
    };
    if let Some(at) = pending.observation.as_ref().filter(|at| at.0 <= stop) {
        outputs.push(SenderOutput::Observation(at.clone(), state.reply));
        pending.observation = None;
        pending.result = Some(select(state, &pending.origin));
    }
    if let Some(result) = &mut pending.result {
        if let Some((at, value)) = result.presentation.as_ref().filter(|(at, _)| at.0 <= stop) {
            outputs.push(SenderOutput::Presentation(at.clone(), *value));
            result.presentation = None;
        }
        if result.completion.0 <= stop {
            state.progress = result.progress.clone();
            state.queue.clone_from(&result.queue);
            outputs.push(SenderOutput::Completed(result.completion.clone()));
            state.now = crate::ExactCpuMasterTicks(stop);
            return outputs;
        }
    }
    state.pending = Some(pending);
    state.now = crate::ExactCpuMasterTicks(stop);
    outputs
}
fn select(state: &SenderState, origin: &CpuMasterTicks) -> SenderResult {
    let mut queue = state.queue.clone();
    let (progress, present, work) = match &state.progress {
        SenderProgress::Ready => deliver(&mut queue, false),
        SenderProgress::AwaitCue(cue) => accept_cue(*cue, state.reply),
        SenderProgress::ClearDelay(n) => clear_delay(*n),
        SenderProgress::AwaitClear => accept_clear(&mut queue, state.reply),
        SenderProgress::ResetDelay(n) => reset_delay(*n),
    };
    SenderResult {
        progress,
        queue,
        presentation: present.map(|(offset, value)| {
            (
                crate::ExactCpuMasterTicks(Add::add(&origin.0, offset)),
                value,
            )
        }),
        completion: crate::ExactCpuMasterTicks(Add::add(&origin.0, work)),
    }
}
fn deliver(
    queue: &mut std::collections::VecDeque<CueId>,
    cleared: bool,
) -> (SenderProgress, Option<(u64, Request)>, u64) {
    const EMPTY: u64 = 122;
    const CLEAR_EMPTY: u64 = 252;
    const PRESENT: u64 = 212;
    const CLEAR_PRESENT: u64 = 342;
    const WORK: u64 = 398;
    const CLEAR_WORK: u64 = 528;
    queue.pop_front().map_or(
        (
            SenderProgress::Ready,
            None,
            if cleared { CLEAR_EMPTY } else { EMPTY },
        ),
        |cue| {
            (
                SenderProgress::AwaitCue(cue),
                Some((
                    if cleared { CLEAR_PRESENT } else { PRESENT },
                    Request::Cue(cue),
                )),
                if cleared { CLEAR_WORK } else { WORK },
            )
        },
    )
}
fn accept_cue(cue: CueId, reply: Request) -> (SenderProgress, Option<(u64, Request)>, u64) {
    const DELAY_TURNS: u8 = 2;
    const ACCEPT: u64 = 210;
    const PRESENT: u64 = 104;
    const RETRY: u64 = 162;
    let delay = NonZeroU8::new(DELAY_TURNS).unwrap_or(NonZeroU8::MIN);
    if reply == Request::Cue(cue) {
        (SenderProgress::ClearDelay(delay), None, ACCEPT)
    } else {
        (
            SenderProgress::AwaitCue(cue),
            Some((PRESENT, Request::Cue(cue))),
            RETRY,
        )
    }
}
fn clear_delay(n: NonZeroU8) -> (SenderProgress, Option<(u64, Request)>, u64) {
    const HOLD: u64 = 116;
    const PRESENT: u64 = 100;
    const CLEAR: u64 = 218;
    if n.get() > 1 {
        (
            SenderProgress::ClearDelay(
                NonZeroU8::new(n.get().saturating_sub(1)).unwrap_or(NonZeroU8::MIN),
            ),
            None,
            HOLD,
        )
    } else {
        (
            SenderProgress::AwaitClear,
            Some((PRESENT, Request::Absent)),
            CLEAR,
        )
    }
}
fn accept_clear(
    queue: &mut std::collections::VecDeque<CueId>,
    reply: Request,
) -> (SenderProgress, Option<(u64, Request)>, u64) {
    const PRESENT: u64 = 104;
    const RETRY: u64 = 162;
    if reply == Request::Absent {
        deliver(queue, true)
    } else {
        (
            SenderProgress::AwaitClear,
            Some((PRESENT, Request::Absent)),
            RETRY,
        )
    }
}
fn reset_delay(n: NonZeroU8) -> (SenderProgress, Option<(u64, Request)>, u64) {
    const HOLD: u64 = 116;
    const EXPIRE: u64 = 142;
    if n.get() > 1 {
        (
            SenderProgress::ResetDelay(
                NonZeroU8::new(n.get().saturating_sub(1)).unwrap_or(NonZeroU8::MIN),
            ),
            None,
            HOLD,
        )
    } else {
        (SenderProgress::Ready, None, EXPIRE)
    }
}
