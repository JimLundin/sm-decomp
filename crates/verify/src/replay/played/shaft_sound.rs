//! Native CPU-side Ambient sender comparisons. Reply reads are borrowed at the service boundary.

use super::{Run, open_rows, polled, read_lock, seed_controller, word};
use crate::replay::{
    Sources, appearance,
    compare::Observation,
    compared, connected, data_failure,
    index::{
        Case, CaseKind, ContentProvider, ExpectedProvider, ObservationId as O, Responsibility,
        Start,
    },
    movement,
    readers::ExpectedCut,
    result::{CaseResult, Outcome},
    runner, sender_offer,
};
use crate::source::WramWord;
use canonical::model::{Model, Pinned};
use game::production::{played, sender::Sounds};
use std::{collections::VecDeque, num::NonZeroU8};
use types::audio::{CueAllowance, CueId, OfferPermission, Request, SenderProgress, SenderState};

const STATE: &[O] = &[O::SenderPermission, O::SenderQueue, O::SenderProgress];
const OUTPUTS: &[O] = &[O::ShaftSoundOffer, O::SenderRequest];

pub(in crate::replay) fn valid(case: &Case) -> bool {
    case.kind == CaseKind::Integrated
        && matches!(case.start, Start::PlayedContinuation { .. })
        && case.responsibilities == [Responsibility::Ceres, Responsibility::AudioSender]
        && case.content == ContentProvider::PlayerRomImport
        && case.expectations.provider == ExpectedProvider::PinnedRecordCache
        && case.expectations.fields == STATE
        && case.expectations.outputs == OUTPUTS
        && case.expectations.output_cut.is_none()
        && case.checkpoints
}

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
        output_count: OUTPUTS.len(),
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

pub(in crate::replay) fn run(
    case: &Case,
    sources: &Sources,
    result: &mut CaseResult,
) -> Result<(), Outcome> {
    let lock = read_lock(sources)?;
    let mut rows = open_rows(sources, &lock, &case.row)?;
    let Start::PlayedContinuation { seed, frame } = case.start else {
        return Err(data_failure(None, "shaft sound continuation start"));
    };
    let last = case.end.cut_inclusive;
    let mut previous = rows.record(seed)?;
    let mut controller = seed_controller(&previous)?;
    let content = connected::Content {
        movement: movement::Content::load(&sources.rom)?,
        appearance: appearance::Content::load(&sources.rom)?,
    };
    let mut live = Run::for_case(&previous, &content, sources, case)?;
    let mut pending = None;
    compared(
        seed,
        &expected(&previous.wram, false, seed)?.fields,
        &actual(&live, None).fields,
    )?;
    let mut digest = Vec::new();
    let replayed = (|| {
        for cut in seed.saturating_add(1)..=last {
            let current = rows.record(cut)?;
            if current.video_frame != previous.video_frame.wrapping_add(1) {
                return Err(data_failure(Some(cut), "played rows join"));
            }
            let (row, local) = rows.place(cut)?;
            let place = (row.as_str(), local);
            let step = Step {
                controller,
                previous: &previous,
                current: &current,
                cut,
                place,
            };
            let mut restored = (cut > frame).then(|| restore(&live, cut)).transpose()?;
            let null = actual(&live, None);
            let outputs = step.advance(&mut live)?;
            let mut final_offer = false;
            if let Some(restored) = restored.as_mut() {
                if live.witnesses.shaft()?.sound(place).is_none() {
                    return Err(data_failure(Some(cut), "shaft sender reply trace"));
                }
                let visit = super::shaft_native::sound_visit(&previous, &current)?;
                final_offer = visit == super::shaft_native::Visit::FinalBurst;
                let observed = expected(
                    &current.wram,
                    visit != super::shaft_native::Visit::None,
                    cut,
                )?;
                compared(
                    cut,
                    &observed.fields,
                    &actual(&live, outputs.1.shaft_sound_offer).fields,
                )?;
                step.check_continuation(restored, &live, &outputs)?;
                result.counts.restorations = result.counts.restorations.saturating_add(1);
                runner::record_cut(result, &mut digest, &observed, &null.fields, cut)?;
            }
            if let Some(restored) = pending.as_mut() {
                step.check_continuation(restored, &live, &outputs)?;
                if cut == last {
                    result.counts.pending_restorations = 1;
                }
            }
            // The cut leaves two cues queued while the sender waits for a clear reply.
            // Resume across both queued presentations and the final clear.
            if final_offer {
                let sender = &live.state.shaft_sound.sender;
                if sender.queue.cues().len() != 2 || sender.progress != SenderProgress::AwaitClear {
                    return Err(data_failure(Some(cut), "shaft pending sender"));
                }
                pending = Some(restore(&live, cut)?);
                result.counts.pending_boundaries = 1;
            }
            controller = controller.read(current.pad.0, polled(&current));
            previous = current;
        }
        if result.counts.pending_restorations != 1
            || live.state.shaft_sound.sender.progress != SenderProgress::Ready
            || !live.state.shaft_sound.sender.queue.cues().is_empty()
        {
            return Err(data_failure(
                Some(last),
                "shaft sender release after pending offer",
            ));
        }
        Ok(())
    })();
    if !digest.is_empty() {
        result.observation_sha256 = crate::lock::Sha256::of(&digest).to_string();
    }
    replayed
}

struct Step<'a> {
    controller: super::Controller,
    previous: &'a oracle_record::FrameRecord,
    current: &'a oracle_record::FrameRecord,
    cut: u64,
    place: (&'a str, u64),
}

impl Step<'_> {
    fn advance(&self, run: &mut Run<'_>) -> Result<(Vec<Observation>, played::Outputs), Outcome> {
        run.step_with_outputs(
            self.controller,
            self.previous,
            self.current,
            self.cut,
            self.place,
        )
    }

    fn check_continuation(
        &self,
        restored: &mut Run<'_>,
        live: &Run<'_>,
        outputs: &(Vec<Observation>, played::Outputs),
    ) -> Result<(), Outcome> {
        if self.advance(restored).as_ref() == Ok(outputs)
            && restored.state == live.state
            && restored.return_timing == live.return_timing
            && restored.warning_picture == live.warning_picture
        {
            Ok(())
        } else {
            Err(Outcome::CheckpointFailure { cut: self.cut })
        }
    }
}

#[cfg(test)]
#[test]
fn shaft_sender_checkpoint_pin_holds() {
    assert_eq!(
        canonical::model::trace_shape::<SenderPin>().unwrap(),
        SenderPin::SHAPE
    );
}
