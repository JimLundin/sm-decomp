//! Native CPU-side Ambient sender comparisons. Reply reads are borrowed at the service boundary.

use super::{Run, word};
use crate::replay::{
    compare::Observation,
    compared, data_failure,
    index::{Case, Start},
    readers::ExpectedCut,
    result::Outcome,
    runner, sender_offer,
};
use crate::source::WramWord;
use canonical::model::{Model, Pinned};
use game::production::{played, sender::Sounds};
use std::{collections::VecDeque, num::NonZeroU8};
use types::audio::{CueAllowance, CueId, OfferPermission, Request, SenderProgress, SenderState};

const OUTPUTS: usize = 2;

/// Reply reads reconstructed from the acknowledged native cue and clear edges. Between edges
/// the last acknowledged value gives the same CPU comparison at the scheduled service.
pub(super) fn reply(
    trace: Option<&super::witnesses::shaft::Sound>,
    cut: u64,
    finished: bool,
) -> Option<Request> {
    let trace = trace.filter(|_| finished)?;
    Some(if trace.cue(cut) {
        Request::Cue(CueId::ShaftExplosion)
    } else {
        Request::Absent
    })
}

fn permission(wram: &[u8]) -> Result<Sounds, Outcome> {
    Ok(
        if word(
            wram,
            WramWord {
                name: "room sound disable",
                offset: 0x05f5,
            },
        )? == 0
        {
            Sounds::Enabled
        } else {
            Sounds::Disabled
        },
    )
}

pub(super) fn seed(wram: &[u8]) -> Result<played::shaft_sound::State, Outcome> {
    Ok(played::shaft_sound::State {
        permission: permission(wram)?,
        ..Default::default()
    })
}

fn observations(
    permission: Sounds,
    queue: &VecDeque<CueId>,
    progress: &SenderProgress,
    offer: Option<audio_transport::Offer>,
    request: Request,
) -> ExpectedCut {
    let mut semantics = vec![Observation::quantity(
        "sender.permission",
        i64::from(permission == Sounds::Enabled),
    )];
    semantics.extend(sender_offer::observations(queue, progress));
    let mut fields = semantics.clone();
    fields.push(Observation {
        field: "shaft-sound.offer".into(),
        bytes: offer.map_or_else(Vec::new, |offer| {
            let allowance = match offer.allowance {
                CueAllowance::One => 1,
                CueAllowance::Three => 3,
                CueAllowance::Six => 6,
                CueAllowance::Nine => 9,
                CueAllowance::Fifteen => 15,
            };
            let permission = match offer.permission {
                OfferPermission::Enabled => 1,
                OfferPermission::Disabled => 0,
                OfferPermission::Demo => 2,
                OfferPermission::Explosion => 3,
            };
            vec![sender_offer::cue_tag(offer.cue), allowance, permission]
        }),
    });
    fields.push(Observation {
        field: "sender.request".into(),
        bytes: match request {
            Request::Absent => Vec::new(),
            Request::Cue(cue) => vec![sender_offer::cue_tag(cue)],
        },
    });
    ExpectedCut {
        semantic_count: semantics.len(),
        output_count: OUTPUTS,
        semantics,
        fields,
        replies: 0,
    }
}

fn expected(wram: &[u8], offer: bool, cut: u64) -> Result<ExpectedCut, Outcome> {
    let bad = || data_failure(Some(cut), "Ambient sender RAM");
    let byte = |offset| wram.get(offset).copied().ok_or_else(bad);
    let head = byte(0x0644)?;
    let tail = byte(0x0647)?;
    if head >= 16 || tail >= 16 {
        return Err(bad());
    }
    let source_cue = |value| match value {
        0x25 => Ok(CueId::ShaftExplosion),
        _ => Err(bad()),
    };
    let queue = (0..tail.wrapping_sub(head) % 16)
        .map(|slot| {
            byte(0x0666_usize.saturating_add(usize::from(head.wrapping_add(slot) % 16)))
                .and_then(source_cue)
        })
        .collect::<Result<VecDeque<_>, _>>()?;
    let line = byte(0x064e)?;
    let request = if line == 0 {
        Request::Absent
    } else {
        Request::Cue(source_cue(line)?)
    };
    let progress = match byte(0x064a)? {
        0 => SenderProgress::Ready,
        1 => SenderProgress::AwaitCue(source_cue(line)?),
        2 => SenderProgress::ClearDelay(NonZeroU8::new(byte(0x0651)?).ok_or_else(bad)?),
        3 => SenderProgress::AwaitClear,
        _ => return Err(bad()),
    };
    let sounds = permission(wram)?;
    let offer = offer.then_some(audio_transport::Offer {
        cue: CueId::ShaftExplosion,
        allowance: CueAllowance::Six,
        permission: if sounds == Sounds::Disabled {
            OfferPermission::Disabled
        } else {
            OfferPermission::Enabled
        },
    });
    Ok(observations(sounds, &queue, &progress, offer, request))
}

fn actual(run: &Run<'_>, offer: Option<audio_transport::Offer>) -> ExpectedCut {
    let state = &run.state.shaft_sound;
    observations(
        state.permission,
        state.sender.queue.cues(),
        &state.sender.progress,
        offer,
        state.request,
    )
}

struct SenderPin;
impl Pinned for SenderPin {
    type State = (SenderState, Request);
    const NAME: &'static str = "shaft-sound-sender";
    const SHAPE: u64 = 0x696c_9db4_00a1_599f;
}

#[derive(canonical::Canonical)]
struct SavedSender {
    permission: Sounds,
    transport: Model<SenderPin>,
}

fn restore<'a>(run: &Run<'a>, cut: u64) -> Result<Run<'a>, Outcome> {
    let state = &run.state.shaft_sound;
    let bytes = canonical::to_bytes(&SavedSender {
        permission: state.permission,
        transport: Model((state.sender.clone(), state.request)),
    });
    let SavedSender {
        permission,
        transport: Model((sender, request)),
    } = canonical::from_bytes(&bytes).map_err(|_| Outcome::CheckpointFailure { cut })?;
    let mut restored = run.clone();
    restored.state.shaft_sound = played::shaft_sound::State {
        permission,
        sender,
        request,
    };
    Ok(restored)
}

pub(in crate::replay::played) fn run(
    room: crate::replay::played::indexed::AdmittedRoom<'_, '_>,
    case: &Case,
    result: &mut runner::Receipt,
) -> Result<(), Outcome> {
    let Start::PlayedContinuation { seed, frame } = case.start else {
        return Err(data_failure(None, "shaft sound continuation start"));
    };
    let last = case.end.cut_inclusive;
    let mut pending = None;
    compared(
        seed,
        &expected(&room.seed.wram, false, seed)?.fields,
        &actual(&room.run, None).fields,
    )?;
    room.replay(case, |live, turn, _| {
        let cut = turn.cut;
        let current = turn.current;
        let place = turn.place;
        let mut restored = (cut > frame).then(|| restore(live, cut)).transpose()?;
        let null = actual(live, None);
        let outputs = turn.advance(live)?;
        let mut final_offer = false;
        if let Some(restored) = restored.as_mut() {
            if live.witnesses.shaft()?.sound(place).is_none() {
                return Err(data_failure(Some(cut), "shaft sender reply trace"));
            }
            let visit = super::shaft_native::sound_visit(turn.previous, current)?;
            final_offer = visit == super::shaft_native::Visit::FinalBurst;
            let observed = expected(
                &current.wram,
                visit != super::shaft_native::Visit::None,
                cut,
            )?;
            compared(
                cut,
                &observed.fields,
                &actual(live, outputs.1.shaft_sound_offer).fields,
            )?;
            turn.check_continuation(restored, live, &outputs)
                .map_err(|_| Outcome::CheckpointFailure { cut })?;
            result.restored(1)?;
            result.record(&observed, &null.fields, cut)?;
        }
        if let Some(restored) = pending.as_mut() {
            turn.check_continuation(restored, live, &outputs)
                .map_err(|_| Outcome::CheckpointFailure { cut })?;
            if cut == last {
                result.resumed()?;
            }
        }
        // The cut leaves two cues queued while the sender waits for a clear reply.
        // Resume across both queued presentations and the final clear.
        if final_offer {
            let sender = &live.state.shaft_sound.sender;
            if sender.queue.cues().len() != 2 || sender.progress != SenderProgress::AwaitClear {
                return Err(data_failure(Some(cut), "shaft pending sender"));
            }
            pending = Some(restore(live, cut)?);
            result.pending()?;
        }
        if cut == last
            && (result.counts.pending_restorations != 1
                || live.state.shaft_sound.sender.progress != SenderProgress::Ready
                || !live.state.shaft_sound.sender.queue.cues().is_empty())
        {
            return Err(data_failure(
                Some(last),
                "shaft sender release after pending offer",
            ));
        }
        Ok(())
    })
}

#[cfg(test)]
#[test]
fn shaft_sender_checkpoint_pin_holds() {
    assert_eq!(
        canonical::model::trace_shape::<SenderPin>().unwrap(),
        SenderPin::SHAPE
    );
}
