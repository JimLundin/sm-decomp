//! One admitted CPU audio sender group and its actual clean service entry.

use super::{AdmissionFault, ProductionFault, Unsupported};
use audio_transport::sender::{ReplyInput, SenderOutput, SenderProfile};
use audio_transport::{
    CpuMasterTicks, CueId, OfferPermission, QueueAllowance, Request, SenderProgress, SenderState,
    SoundGroup,
};

/// One causal offer or independently scheduled service operation.
pub enum SenderInput<'a> {
    /// An actual producer offers one cue between services.
    Offer {
        /// The semantic cue; its group must match the admitted sender.
        cue: CueId,
        /// The producer's queue allowance.
        allowance: QueueAllowance,
        /// The producer's actual sound permission.
        permission: OfferPermission,
    },
    /// Begin a service only under the independently admitted platform timing profile.
    Begin(SenderProfile),
    /// Resume through an exact CPU stop with actual physical reply updates.
    Advance {
        /// Absolute CPU master ticks, distinct from audio cycles.
        stop: CpuMasterTicks,
        /// Ordered replies strictly after the held CPU instant and through this stop.
        replies: &'a [ReplyInput],
    },
}

fn request_group(request: Request, group: SoundGroup) -> bool {
    match request {
        Request::Absent => true,
        Request::Cue(cue) => audio_transport::effects::group(cue) == group,
    }
}

fn progress_group(progress: &SenderProgress, group: SoundGroup) -> bool {
    match progress {
        SenderProgress::AwaitCue(cue) => audio_transport::effects::group(*cue) == group,
        SenderProgress::Ready
        | SenderProgress::ClearDelay(_)
        | SenderProgress::AwaitClear
        | SenderProgress::ResetDelay(_) => true,
    }
}

pub(super) fn validate(state: &SenderState, group: SoundGroup) -> Result<(), AdmissionFault> {
    let queue_valid = |queue: &std::collections::VecDeque<CueId>| {
        queue.len() <= 15
            && queue
                .iter()
                .all(|cue| audio_transport::effects::group(*cue) == group)
    };
    if !queue_valid(&state.queue)
        || !request_group(state.reply, group)
        || !progress_group(&state.progress, group)
        || state
            .pending
            .as_ref()
            .and_then(|pending| pending.result.as_ref())
            .is_some_and(|result| {
                !queue_valid(&result.queue)
                    || !progress_group(&result.progress, group)
                    || result
                        .presentation
                        .as_ref()
                        .is_some_and(|(_, request)| !request_group(*request, group))
            })
    {
        return Err(AdmissionFault::SenderQueue);
    }
    Ok(())
}

pub(super) fn advance(
    state: &mut SenderState,
    group: SoundGroup,
    input: SenderInput<'_>,
) -> Result<Vec<SenderOutput>, ProductionFault> {
    match input {
        SenderInput::Offer {
            cue,
            allowance,
            permission,
        } => {
            if state.pending.is_some() {
                return Err(ProductionFault::Unsupported(Unsupported::SenderService));
            }
            if audio_transport::effects::group(cue) != group {
                return Err(ProductionFault::Unsupported(Unsupported::SenderCueGroup));
            }
            audio_transport::sender::offer(state, cue, allowance, permission);
            Ok(Vec::new())
        }
        SenderInput::Begin(profile) => {
            audio_transport::sender::begin(state, profile);
            Ok(Vec::new())
        }
        SenderInput::Advance { stop, replies } => {
            if stop.0 < state.now.0 {
                return Err(ProductionFault::TimeReversed);
            }
            if replies
                .iter()
                .any(|input| input.at.0 <= state.now.0 || input.at.0 > stop.0)
            {
                return Err(ProductionFault::PresentationOutsideInterval);
            }
            if replies.windows(2).any(|pair| {
                pair.first()
                    .zip(pair.last())
                    .is_some_and(|(before, after)| before.at.0 > after.at.0)
            }) {
                return Err(ProductionFault::PresentationsUnordered);
            }
            if replies
                .iter()
                .any(|input| !request_group(input.reply, group))
            {
                return Err(ProductionFault::Unsupported(Unsupported::SenderCueGroup));
            }
            let elapsed =
                audio_transport::ExactCpuMasterTicks(std::ops::Sub::sub(&stop.0, &state.now.0));
            Ok(audio_transport::sender::advance_with_inputs(
                state, elapsed, replies,
            ))
        }
    }
}
