//! Scoped offer acceptance: one frame's offers to one sound queue, compared before the frame's
//! sender service.
//!
//! The recorded windows bracket each offer between a preceding cut and the group's next service
//! entry, with no service of that group between, so the frame is played with a cadence that runs
//! no audio turn and no service. The rest of the transport is a declared silent seed that such a
//! frame never reads.

use std::collections::VecDeque;
use std::num::NonZeroU8;
use std::path::Path;

use audio_transport::frame::{EffectCadence, FrameCadence, TurnCount};
use audio_transport::{
    CueId, CueQueue, GroupState, Loudness, OfferPermission, Playback, QueueAllowance,
    ReleaseMemory, Request, ResidentMusicReply, SenderProgress, SenderState, SongMix, SongState,
    SoundGroup, State,
};
use game::production::{
    AdvanceInput, Production, ProductionContent, ProductionOutputs, ProductionRules,
    ProductionSeed, audio,
};
use serde::{Deserialize, Serialize};

use super::compare::Observation;
use super::index::{Case, FixtureFile, SenderOfferControl, Start, valid_hash};
use super::readers::{ActualReader, AdmittedCase, ExpectedReader, InputReader};
use super::result::Outcome;
use super::{Sources, data_failure, production_failure};

/// Pinned causal and observation streams for native sender windows.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SenderPacket {
    /// Locked capture row recorded in both file headers.
    pub(super) row: String,
    capture_sha256: String,
    pub(super) windows: usize,
    inputs: FixtureFile,
    expected: FixtureFile,
    simultaneous: Vec<OfferOrder>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct OfferOrder {
    cut: u64,
    offers: Vec<(SoundGroup, CueId)>,
}

impl SenderPacket {
    pub(super) fn input_path(&self) -> &str {
        &self.inputs.path
    }

    pub(super) fn valid(&self) -> bool {
        !self.row.is_empty()
            && valid_hash(&self.capture_sha256)
            && self.windows > 0
            && self.inputs.valid()
            && self.expected.valid()
            && self.inputs.path != self.expected.path
            && self.inputs.sha256 != self.expected.sha256
            && self
                .simultaneous
                .is_sorted_by(|first, second| first.cut < second.cut)
            && self
                .simultaneous
                .iter()
                .all(|order| order.offers.len() >= 2)
    }

    fn cross_group_order(&self, offers: &[NativeOffer]) -> bool {
        self.simultaneous.iter().all(|order| {
            offers
                .iter()
                .filter(|offer| offer.cut == order.cut)
                .map(|offer| (offer.group, offer.cue))
                .eq(order.offers.iter().copied())
        }) && !offers.windows(2).any(|pair| {
            matches!(pair, [first, second] if first.cut == second.cut && first.group != second.group
                && !self.simultaneous.iter().any(|order| order.cut == first.cut))
        })
    }
}

#[derive(Deserialize)]
struct Header {
    schema: u8,
    source_row: String,
    source_ciphertext_sha256: String,
    time_unit: String,
    origin: String,
    scope: String,
}

impl Header {
    fn valid(&self, packet: &SenderPacket) -> bool {
        self.schema == 1
            && self.source_row == packet.row
            && self.source_ciphertext_sha256 == packet.capture_sha256
            && self.time_unit == "CpuMasterTicks"
            && self.origin == "LockedRoomEntry"
            && self.scope == "ScopedSingleOffer"
    }
}

#[derive(Deserialize)]
struct Inputs {
    #[serde(flatten)]
    header: Header,
    offer: Vec<NativeOffer>,
}

#[derive(Deserialize)]
struct Expected {
    #[serde(flatten)]
    header: Header,
    fifo: Vec<Fifo>,
}

#[derive(Deserialize)]
struct NativeOffer {
    source_id: usize,
    cut: u64,
    group: SoundGroup,
    cue: CueId,
    allowance: QueueAllowance,
    #[serde(with = "super::index::Permission")]
    permission: OfferPermission,
    prefetch_master_ticks: u64,
    exec_master_ticks: u64,
    disabled: bool,
    demo: bool,
    exploding: bool,
    previous_group_service_id: Option<u64>,
    previous_group_service_end_master_ticks: Option<u64>,
    seed: Seed,
}

#[derive(Deserialize, PartialEq)]
enum StartBoundary {
    LockedLoad,
    PrecedingCut,
}

#[derive(Deserialize)]
struct Seed {
    now_master_ticks: u64,
    boundary: StartBoundary,
    cut: Option<u64>,
    service_pending: bool,
    queue: VecDeque<CueId>,
    #[serde(flatten)]
    progress: RecordedProgress,
}

#[derive(Deserialize)]
struct Fifo {
    source_id: usize,
    boundary: String,
    group_service_id: u64,
    master_ticks: u64,
    queue: VecDeque<CueId>,
    #[serde(flatten)]
    progress: RecordedProgress,
}

#[derive(Deserialize)]
struct RecordedProgress {
    progress: ProgressKind,
    progress_cue: Option<CueId>,
    progress_remaining_turns: Option<NonZeroU8>,
}

#[derive(Deserialize)]
enum ProgressKind {
    Ready,
    AwaitCue,
    ClearDelay,
    AwaitClear,
    ResetDelay,
}

impl RecordedProgress {
    fn decode(self) -> Result<SenderProgress, Outcome> {
        use ProgressKind::{AwaitClear, AwaitCue, ClearDelay, Ready, ResetDelay};
        match (
            self.progress,
            self.progress_cue,
            self.progress_remaining_turns,
        ) {
            (Ready, None, None) => Ok(SenderProgress::Ready),
            (AwaitCue, Some(cue), None) => Ok(SenderProgress::AwaitCue(cue)),
            (ClearDelay, None, Some(turns)) => Ok(SenderProgress::ClearDelay(turns)),
            (AwaitClear, None, None) => Ok(SenderProgress::AwaitClear),
            (ResetDelay, None, Some(turns)) => Ok(SenderProgress::ResetDelay(turns)),
            _ => Err(data_failure(None, "sender progress payload")),
        }
    }
}

impl NativeOffer {
    fn permission(&self) -> Result<OfferPermission, Outcome> {
        let permission = if self.disabled {
            OfferPermission::Disabled
        } else if self.demo {
            OfferPermission::Demo
        } else if self.exploding {
            OfferPermission::Explosion
        } else {
            OfferPermission::Enabled
        };
        if self.permission != permission {
            return Err(data_failure(None, "sender offer suppression priority"));
        }
        Ok(permission)
    }

    fn valid_seed(&self) -> bool {
        let seed = &self.seed;
        let at_cut = match seed.boundary {
            StartBoundary::LockedLoad => {
                self.source_id == 0
                    && seed.now_master_ticks == 0
                    && self.cut == 0
                    && seed.cut.is_none()
            }
            StartBoundary::PrecedingCut => {
                self.source_id != 0 && seed.cut.and_then(|cut| cut.checked_add(1)) == Some(self.cut)
            }
        };
        at_cut
            && !seed.service_pending
            && seed.now_master_ticks < self.prefetch_master_ticks
            && self.prefetch_master_ticks.checked_add(2) == Some(self.exec_master_ticks)
    }

    fn valid_bracket(&self, after: &Fifo) -> bool {
        let prior_service = match (
            self.previous_group_service_id,
            self.previous_group_service_end_master_ticks,
        ) {
            (Some(id), Some(time)) => {
                id < after.group_service_id && time <= self.seed.now_master_ticks
            }
            (None, None) => true,
            _ => false,
        };
        self.valid_seed()
            && prior_service
            && self.exec_master_ticks < after.master_ticks
            && after.boundary == "NextSelectedGroupServiceEntry"
    }
}

struct Window {
    seed: SenderState,
    group: SoundGroup,
    offers: Vec<audio_transport::Offer>,
    expected: Vec<Observation>,
}

/// A frame that runs no audio turn and no sender service: it only queues its offers.
pub(super) const fn offers_only() -> FrameCadence {
    FrameCadence {
        effects: EffectCadence::WithoutService(TurnCount::ZERO),
        music: TurnCount::ZERO,
    }
}

const fn parts(group: SoundGroup) -> usize {
    match group {
        SoundGroup::Action | SoundGroup::Ambient => 1,
        SoundGroup::Contact => 2,
    }
}

/// A silent transport holding one recorded sender: the other senders ready with empty queues,
/// every line and reply absent, nothing playing and no song. A frame that runs no turn and no
/// service reads only the selected queue.
pub(super) fn silent_transport(group: SoundGroup, sender: SenderState) -> State {
    let reply = ResidentMusicReply {
        identity: "silent".to_owned(),
        original_byte: 0,
    };
    let mut sender = Some(sender);
    State {
        senders: enum_map::EnumMap::from_fn(|line| {
            sender
                .take_if(|_| line == group)
                .unwrap_or_else(|| SenderState {
                    queue: CueQueue::default(),
                    progress: SenderProgress::Ready,
                })
        }),
        request_lines: enum_map::EnumMap::from_fn(|_| Request::Absent),
        groups: enum_map::EnumMap::from_fn(|line| GroupState {
            history: vec![ReleaseMemory::Inactive; parts(line)],
            handled: Request::Absent,
            staged: Request::Absent,
            reply: Request::Absent,
            playback: Playback::Empty,
        }),
        song: SongState {
            tracks: Vec::new(),
            mix: SongMix {
                master: Loudness(0),
                percussion: 0,
                echo_voices: std::collections::BTreeSet::new(),
                echo_left: 0,
                echo_right: 0,
                echo_delay: 0,
                feedback: 0,
                filter: 0,
                echo_enabled: false,
            },
        },
        music_reply: reply.clone(),
        resident_music_reply: reply,
        reservations: std::collections::BTreeSet::new(),
    }
}

fn native_records(
    path: &Path,
    packet: &SenderPacket,
    source_id: usize,
    cut: u64,
    require_no_intervening_contact: bool,
) -> Result<(NativeOffer, Fifo), Outcome> {
    let inputs: Inputs = packet.inputs.read(path)?;
    let expected: Expected = packet.expected.read(path)?;
    if !inputs.header.valid(packet)
        || !expected.header.valid(packet)
        || inputs.offer.len() != packet.windows
        || expected.fifo.len() != packet.windows
        || inputs.offer.windows(2).any(|pair| {
            matches!(pair, [first, second] if first.exec_master_ticks >= second.exec_master_ticks || first.cut > second.cut)
        })
        || !inputs.offer.iter().zip(&expected.fifo).enumerate().all(|(id, (offer, fifo))| {
            id == offer.source_id && id == fifo.source_id && offer.valid_bracket(fifo)
        })
        || !packet.cross_group_order(&inputs.offer)
    {
        return Err(data_failure(None, "sender offer source order or bracket"));
    }
    if require_no_intervening_contact {
        let (offer, fifo) = inputs
            .offer
            .get(source_id)
            .zip(expected.fifo.get(source_id))
            .ok_or_else(|| data_failure(Some(cut), "native Contact offer bracket"))?;
        if offer.group != SoundGroup::Contact
            || inputs.offer.iter().any(|later| {
                later.group == SoundGroup::Contact
                    && later.exec_master_ticks > offer.exec_master_ticks
                    && later.prefetch_master_ticks < fifo.master_ticks
            })
        {
            return Err(data_failure(
                Some(cut),
                "intervening Contact offer before FIFO observation",
            ));
        }
    }
    inputs
        .offer
        .into_iter()
        .zip(expected.fifo)
        .nth(source_id)
        .filter(|(offer, _)| offer.cut == cut)
        .ok_or_else(|| data_failure(Some(cut), "sender offer source identity"))
}

fn native(
    path: &Path,
    packet: &SenderPacket,
    source_id: usize,
    cut: u64,
) -> Result<Window, Outcome> {
    let (offer, fifo) = native_records(path, packet, source_id, cut, false)?;
    let request = audio_transport::Offer {
        cue: offer.cue,
        allowance: offer.allowance,
        permission: offer.permission()?,
    };
    Ok(Window {
        group: offer.group,
        offers: vec![request],
        seed: offer.seed.decode(cut)?,
        expected: observations(&fifo.queue, &fifo.progress.decode()?),
    })
}

impl Seed {
    // The recorded reply and CPU time are the old service's inputs; offers read neither.
    fn decode(self, cut: u64) -> Result<SenderState, Outcome> {
        Ok(SenderState {
            queue: CueQueue::try_from(self.queue)
                .map_err(|_| data_failure(Some(cut), "sender seed queue length"))?,
            progress: self.progress.decode()?,
        })
    }
}

pub(super) struct ContactIngress {
    pub seed: SenderState,
    pub source_id: usize,
    pub current: game::production::sender::OfferCurrent,
}

pub(super) fn contact_ingress(
    path: &Path,
    packet: &SenderPacket,
    cut: u64,
    seed_master_ticks: u64,
) -> Result<ContactIngress, Outcome> {
    use game::production::sender::{Demo, OfferCurrent, PowerBomb, Sounds};
    let inputs: Inputs = packet.inputs.read(path)?;
    if !inputs.header.valid(packet) || inputs.offer.len() != packet.windows {
        return Err(data_failure(None, "native Contact ingress header"));
    }
    let mut matching = inputs.offer.into_iter().enumerate().filter(|(_, offer)| {
        offer.group == SoundGroup::Contact
            && offer.cut == cut
            && offer.seed.now_master_ticks == seed_master_ticks
    });
    let (Some((id, offer)), None) = (matching.next(), matching.next()) else {
        return Err(data_failure(Some(cut), "native Contact ingress identity"));
    };
    if id != offer.source_id
        || offer.seed.boundary != StartBoundary::PrecedingCut
        || !offer.valid_seed()
    {
        return Err(data_failure(Some(cut), "native Contact ingress bracket"));
    }
    offer.permission()?;
    let current = OfferCurrent {
        sounds: if offer.disabled {
            Sounds::Disabled
        } else {
            Sounds::Enabled
        },
        demo: if offer.demo {
            Demo::Active
        } else {
            Demo::Inactive
        },
        power_bomb: if offer.exploding {
            PowerBomb::Active
        } else {
            PowerBomb::Inactive
        },
    };
    Ok(ContactIngress {
        seed: offer.seed.decode(cut)?,
        source_id: offer.source_id,
        current,
    })
}

pub(super) struct ContactFifo {
    pub ingress_prefetch: u64,
    pub service_master_ticks: u64,
    pub queue: VecDeque<CueId>,
    pub progress: SenderProgress,
}

pub(super) fn contact_fifo(
    path: &Path,
    packet: &SenderPacket,
    source_id: usize,
    cut: u64,
) -> Result<ContactFifo, Outcome> {
    let (offer, fifo) = native_records(path, packet, source_id, cut, true)?;
    Ok(ContactFifo {
        ingress_prefetch: offer.prefetch_master_ticks,
        service_master_ticks: fifo.master_ticks,
        queue: fifo.queue,
        progress: fifo.progress.decode()?,
    })
}

fn constructed(control: &SenderOfferControl) -> Window {
    Window {
        seed: SenderState {
            queue: control.seed.clone(),
            progress: SenderProgress::Ready,
        },
        group: control.group,
        offers: control
            .cues
            .iter()
            .map(|&cue| audio_transport::Offer {
                cue,
                allowance: control.allowance,
                permission: control.permission,
            })
            .collect(),
        expected: observations(control.expected.cues(), &SenderProgress::Ready),
    }
}

pub(super) fn admit(case: &Case, sources: &Sources) -> Result<AdmittedCase, Outcome> {
    let cut = case.end.cut_inclusive;
    let Window {
        seed,
        group,
        offers,
        expected,
    } = match case.start {
        Start::NativeSenderOffer { source_id } => {
            native(&sources.native, case.sender_packet()?, source_id, cut)?
        }
        Start::ConstructedSenderOffer { ref control } => constructed(control),
        Start::NativeOpportunity { .. }
        | Start::NativeRecord { .. }
        | Start::PlayedContinuation { .. }
        | Start::CacheCut { .. }
        | Start::NativeSeed { .. }
        | Start::PowerOn => return Err(data_failure(Some(cut), "sender offer start contract")),
    };
    let content = ProductionContent::Audio(Box::new(audio::Content::silent()));
    let rules = ProductionRules::Audio;
    let production = Production::admit(
        ProductionSeed::Audio(Box::new(silent_transport(group, seed))),
        content.clone(),
        rules.clone(),
    )
    .map_err(|_| data_failure(Some(cut), "sender offer production admission"))?;
    Ok(AdmittedCase {
        production,
        content,
        rules,
        inputs: InputReader::SenderOffer(offers),
        expected: ExpectedReader::SenderOffer(expected),
        actual: ActualReader::SenderOffer(group),
        schedule: vec![cut],
    })
}

pub(super) fn advance(
    production: &mut Production,
    offers: &[audio_transport::Offer],
    cut: u64,
) -> Result<ProductionOutputs, Outcome> {
    production
        .advance(AdvanceInput::Audio {
            offers,
            cadence: offers_only(),
        })
        .map_err(|fault| production_failure(cut, fault))
}

pub(super) fn actual(
    production: &Production,
    group: SoundGroup,
) -> Result<Vec<Observation>, Outcome> {
    let state = production
        .transport()
        .ok_or_else(|| data_failure(None, "sender offer projection scope"))?;
    let sender = &state.senders[group];
    Ok(observations(sender.queue.cues(), &sender.progress))
}

pub(super) const fn cue_tag(cue: CueId) -> u8 {
    match cue {
        CueId::Footstep => 0,
        CueId::Spin => 1,
        CueId::EndSpin => 2,
        CueId::Land => 3,
        CueId::Beam => 4,
        CueId::SamusSilence => 5,
        CueId::BackgroundSilence => 6,
        CueId::Door => 7,
        CueId::ShaftExplosion => 8,
    }
}

pub(super) fn progress_bytes(progress: &SenderProgress) -> Vec<u8> {
    match *progress {
        SenderProgress::Ready => vec![0],
        SenderProgress::AwaitCue(cue) => vec![1, cue_tag(cue)],
        SenderProgress::ClearDelay(turns) => vec![2, turns.get()],
        SenderProgress::AwaitClear => vec![3],
        SenderProgress::ResetDelay(turns) => vec![4, turns.get()],
    }
}

pub(super) fn observations(queue: &VecDeque<CueId>, progress: &SenderProgress) -> Vec<Observation> {
    vec![
        Observation {
            field: "sender.queue".to_owned(),
            bytes: queue.iter().copied().map(cue_tag).collect(),
        },
        Observation {
            field: "sender.progress".to_owned(),
            bytes: progress_bytes(progress),
        },
    ]
}
