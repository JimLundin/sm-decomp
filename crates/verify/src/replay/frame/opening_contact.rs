//! Independent true return clocks and CPU audio events from one opening admission.

use super::{Observation, data_failure, opening_audio, read_expected};
use crate::{lock::Sha256, replay::readers::ExpectedCut, replay::result::Outcome};
use audio_transport::SenderProgress;
use canonical::Canonical;
use data::timing::MasterTicks;
use game::production::{Production, frame};
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub(in crate::replay) const LAST_RETURN: u64 = 8;

pub(in crate::replay) struct Source(PathBuf);

#[derive(Deserialize)]
struct Capture {
    format: u16,
    scope: String,
    #[serde(rename = "return")]
    returns: [Return; 8],
    contact_transition: [Transition; 6],
    cpu_port: Vec<opening_audio::Recorded>,
}

#[derive(Deserialize)]
struct Return {
    cut: u64,
    post_rti_master_ticks: u64,
    contact_incoming: Option<u8>,
    contact_outgoing: Option<u8>,
}

#[derive(Deserialize)]
struct Transition {
    cut: u64,
    phase: Phase,
    delay_turns: u8,
}

#[derive(Canonical, Clone, Copy, Deserialize)]
enum Phase {
    ClearDelay,
    AwaitClear,
    Ready,
}

#[derive(Canonical)]
struct PhaseState {
    phase: Phase,
    delay_turns: u8,
}

impl Source {
    pub(in crate::replay) fn new(root: &Path) -> Self {
        Self(root.join(
            "opening-clock-through-cut7-expected-v1/opening-through-cut7-expected.private.toml",
        ))
    }

    pub(super) fn read(&self, cut: u64) -> Result<ExpectedCut, Outcome> {
        let fault = || data_failure(Some(cut), "opening Contact return comparison");
        let pin =
            Sha256::try_from("db1ba02927211fb289f5b8f8fa690935c2700af5aca12143b8b40364c781d986")
                .map_err(|_| fault())?;
        let bytes = read_expected(&self.0, pin, cut)?;
        let capture: Capture = toml::from_str(std::str::from_utf8(&bytes).map_err(|_| fault())?)
            .map_err(|_| fault())?;
        if capture.format != 1
            || capture.scope != "LockedOpeningThroughCut7"
            || !opening_audio::ordered(&capture.cpu_port)
        {
            return Err(fault());
        }
        let native_cut = cut.checked_sub(1).ok_or_else(fault)?;
        let endpoint = capture
            .returns
            .get(usize::try_from(native_cut).map_err(|_| fault())?)
            .filter(|endpoint| endpoint.cut == native_cut)
            .ok_or_else(fault)?;
        let mut semantics = vec![clock(MasterTicks(endpoint.post_rti_master_ticks))];
        let transition = capture
            .contact_transition
            .iter()
            .find(|transition| transition.cut == native_cut);
        match (
            native_cut,
            endpoint.contact_incoming,
            endpoint.contact_outgoing,
            transition,
        ) {
            (0 | 1, None, None, None) => {}
            (2..=7, Some(incoming), Some(outgoing), Some(transition)) => {
                semantics.extend(contact(
                    transition.phase,
                    transition.delay_turns,
                    [incoming, outgoing],
                ));
            }
            _ => return Err(fault()),
        }
        let (ports, replies) = opening_audio::recorded(capture.cpu_port, cut).ok_or_else(fault)?;
        let mut fields = vec![ports];
        fields.extend(semantics.iter().cloned());
        Ok(ExpectedCut {
            fields,
            semantic_count: semantics.len(),
            semantics,
            output_count: 0,
            replies,
        })
    }
}

pub(super) fn clock(at: MasterTicks) -> Observation {
    observation("frame.time", &at)
}

pub(super) fn actual(production: &Production, state: &frame::State) -> Vec<Observation> {
    let mut fields = vec![clock(state.timing.hardware.now)];
    let phase = match state.senders.contact.0.progress {
        SenderProgress::ClearDelay(turns) => Some((Phase::ClearDelay, turns.get())),
        SenderProgress::AwaitClear => Some((Phase::AwaitClear, 0)),
        SenderProgress::Ready => Some((Phase::Ready, 0)),
        SenderProgress::AwaitCue(_) | SenderProgress::ResetDelay(_) => None,
    };
    if let Some(((phase, turns), latches)) = phase.zip(production.frame_contact_latches()) {
        fields.extend(contact(phase, turns, latches));
    }
    fields
}

fn contact(phase: Phase, turns: u8, [incoming, outgoing]: [u8; 2]) -> [Observation; 3] {
    [
        observation(
            "frame.contact.phase",
            &PhaseState {
                phase,
                delay_turns: turns,
            },
        ),
        observation("frame.contact.incoming", &incoming),
        observation("frame.contact.outgoing", &outgoing),
    ]
}

fn observation(field: &str, value: &impl Canonical) -> Observation {
    Observation {
        field: field.to_owned(),
        bytes: canonical::to_bytes(value),
    }
}
