//! Independent true return clocks and CPU audio events from one opening admission.

use super::{Observation, data_failure, opening_audio, read_expected};
use crate::{lock::Sha256, replay::readers::ExpectedCut, replay::result::Outcome};
use audio_transport::SenderProgress;
use canonical::Canonical;
use data::timing::MasterTicks;
use game::production::{Production, frame};
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod corridor;

pub(in crate::replay) const LAST_RETURN: u64 = 8;
pub(in crate::replay) const GROUND_CORRIDOR_LAST_RETURN: u64 = 16;
/// The capture records no Contact transition before this native return.
const FIRST_CONTACT_RETURN: u64 = 2;
const CONTACT_FIELDS: usize = 3;

/// Fields compared over the first `cuts` returns: each return's clock, the Contact fields from
/// the first recorded transition on, and the ground corridor's owners when the case carries it.
pub(in crate::replay) fn field_count(cuts: usize, ground_corridor: bool) -> Option<usize> {
    let contact_returns = cuts.saturating_sub(usize::try_from(FIRST_CONTACT_RETURN).ok()?);
    let corridor = if ground_corridor {
        corridor::field_count(cuts)?
    } else {
        0
    };
    cuts.checked_add(contact_returns.checked_mul(CONTACT_FIELDS)?)?
        .checked_add(corridor)
}

pub(in crate::replay) struct Source {
    path: PathBuf,
    ground_corridor: Option<corridor::Source>,
}

#[derive(Deserialize)]
struct Capture {
    format: u16,
    scope: String,
    #[serde(rename = "return")]
    returns: Vec<Return>,
    contact_transition: Vec<Transition>,
    cpu_port: Vec<opening_audio::Recorded>,
}

#[derive(Deserialize)]
struct Return {
    cut: u64,
    post_rti_master_ticks: u64,
    contact_incoming: Option<u8>,
    contact_outgoing: Option<u8>,
    #[serde(flatten)]
    owners: corridor::RetainedOwners,
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
    AwaitCue,
}

#[derive(Canonical)]
struct PhaseState {
    phase: Phase,
    delay_turns: u8,
}

impl Source {
    pub(in crate::replay) fn new(root: &Path, ground_corridor: bool) -> Self {
        let member = if ground_corridor {
            "opening-clock-through-cut15-expected-v1/opening-through-cut15-expected.private.toml"
        } else {
            "opening-clock-through-cut7-expected-v1/opening-through-cut7-expected.private.toml"
        };
        Self {
            path: root.join(member),
            ground_corridor: ground_corridor.then(|| corridor::Source::new(root)),
        }
    }

    pub(super) fn read(&self, cut: u64) -> Result<ExpectedCut, Outcome> {
        let fault = || data_failure(Some(cut), "opening Contact return comparison");
        let (sha, scope, returns) = if self.ground_corridor.is_some() {
            (
                "b8456d71b47313a81f59940ff29287c75d100b934457455264c20d5e6dd0e570",
                "LockedOpeningThroughCut15",
                GROUND_CORRIDOR_LAST_RETURN,
            )
        } else {
            (
                "db1ba02927211fb289f5b8f8fa690935c2700af5aca12143b8b40364c781d986",
                "LockedOpeningThroughCut7",
                LAST_RETURN,
            )
        };
        let pin = Sha256::try_from(sha).map_err(|_| fault())?;
        let bytes = read_expected(&self.path, pin, cut)?;
        let capture: Capture = toml::from_str(std::str::from_utf8(&bytes).map_err(|_| fault())?)
            .map_err(|_| fault())?;
        if capture.format != 1
            || capture.scope != scope
            || u64::try_from(capture.returns.len()).ok() != Some(returns)
            || u64::try_from(capture.contact_transition.len()).ok() != returns.checked_sub(2)
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
            (..FIRST_CONTACT_RETURN, None, None, None) => {}
            (FIRST_CONTACT_RETURN.., Some(incoming), Some(outgoing), Some(transition)) => {
                semantics.extend(contact(
                    transition.phase,
                    transition.delay_turns,
                    [incoming, outgoing],
                ));
            }
            _ => return Err(fault()),
        }
        if let Some(source) = &self.ground_corridor {
            semantics.extend(source.read(cut, native_cut, &endpoint.owners)?);
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
        SenderProgress::AwaitCue(_) => Some((Phase::AwaitCue, 0)),
        SenderProgress::ResetDelay(_) => None,
    };
    if let Some(((phase, turns), latches)) = phase.zip(production.frame_contact_latches()) {
        fields.extend(contact(phase, turns, latches));
    }
    fields
}

pub(super) fn initial_corridor(state: &frame::State) -> Vec<Observation> {
    corridor::initial(state)
}

pub(super) fn actual_corridor(
    state: &frame::State,
    events: &[game::production::timing::Event],
    cut: u64,
) -> Result<Vec<Observation>, Outcome> {
    corridor::actual(state, events, cut)
}

fn contact(
    phase: Phase,
    turns: u8,
    [incoming, outgoing]: [u8; 2],
) -> [Observation; CONTACT_FIELDS] {
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
