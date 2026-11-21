//! Scoped sender admission keeps recorded inputs separate from following-entry observations.

use std::collections::VecDeque;
use std::num::NonZeroU8;
use std::path::Path;

use audio_transport::{
    CpuMasterTicks, CueId, OfferPermission, QueueAllowance, Request, SenderProgress, SenderState,
    SoundGroup,
};
use game::production::{
    AdvanceInput, Production, ProductionContent, ProductionOutputs, ProductionRules,
    ProductionSeed, SenderInput,
};
use game::state::owned::OwnedState;
use serde::Deserialize;

use super::compare::Observation;
use super::index::{Case, SenderOfferControl, Start};
use super::readers::{ActualReader, AdmittedCase, ExpectedReader, InputReader};
use super::result::Outcome;
use super::{Sources, data_failure, production_failure};
use crate::lock::Sha256;

pub(super) const SOURCE_ROW: &str = "chain-any-0003-DFD7";
pub(super) const NATIVE_WINDOWS: usize = 19;

const CAPTURE: &str = "deed3fab55b6d610429a7e74aa006d39e95189d1dfd031cf387323874ec5e629";
const INPUTS: &str = "5ba2872046c74d2f7fae52445d5ef1143a629cc53566d41f649217586c6b085c";
const EXPECTED: &str = "8dfedf3bc590683cfcaef8a0afcd9678aef23310d086b7ec8477f1224db5ceb8";

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
    fn valid(&self) -> bool {
        self.schema == 1
            && self.source_row == SOURCE_ROW
            && self.source_ciphertext_sha256 == CAPTURE
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
    permission: String,
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
    reply: ReplyKind,
    reply_cue: Option<CueId>,
}

#[derive(Deserialize)]
enum ReplyKind {
    Absent,
    Cue,
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

pub(super) struct Offer {
    cue: CueId,
    allowance: QueueAllowance,
    permission: OfferPermission,
}

impl NativeOffer {
    fn permission(&self) -> Result<OfferPermission, Outcome> {
        let (name, permission) = if self.disabled {
            ("Disabled", OfferPermission::Disabled)
        } else if self.demo {
            ("Demo", OfferPermission::Demo)
        } else if self.exploding {
            ("Explosion", OfferPermission::Explosion)
        } else {
            ("Enabled", OfferPermission::Enabled)
        };
        if self.permission != name {
            return Err(data_failure(None, "sender offer suppression priority"));
        }
        Ok(permission)
    }

    fn valid_bracket(&self, after: &Fifo) -> bool {
        let seed = &self.seed;
        let starts_at_origin = seed.boundary == StartBoundary::LockedLoad;
        let prior_service = match (
            self.previous_group_service_id,
            self.previous_group_service_end_master_ticks,
        ) {
            (Some(id), Some(time)) => id < after.group_service_id && time <= seed.now_master_ticks,
            (None, None) => true,
            _ => false,
        };
        let seed_cut = if starts_at_origin {
            self.source_id == 0 && seed.now_master_ticks == 0 && self.cut == 0 && seed.cut.is_none()
        } else {
            self.source_id != 0 && seed.cut.and_then(|cut| cut.checked_add(1)) == Some(self.cut)
        };
        prior_service
            && seed_cut
            && !seed.service_pending
            && seed.now_master_ticks < self.prefetch_master_ticks
            && self.prefetch_master_ticks.checked_add(2) == Some(self.exec_master_ticks)
            && self.exec_master_ticks < after.master_ticks
            && after.boundary == "NextSelectedGroupServiceEntry"
    }
}

struct Window {
    seed: SenderState,
    group: SoundGroup,
    offers: Vec<Offer>,
    expected: Vec<Observation>,
}

fn read_pinned<T: serde::de::DeserializeOwned>(path: &Path, pin: &str) -> Result<T, Outcome> {
    let bytes = std::fs::read(path).map_err(|_| data_failure(None, "sender offer fixture file"))?;
    if Sha256::of(&bytes).to_string() != pin {
        return Err(data_failure(None, "sender offer fixture pin"));
    }
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| data_failure(None, "sender offer fixture encoding"))?;
    toml::from_str(text).map_err(|_| data_failure(None, "sender offer fixture schema"))
}

fn native_records(
    path: &Path,
    source_id: usize,
    cut: u64,
    require_no_intervening_contact: bool,
) -> Result<(NativeOffer, Fifo), Outcome> {
    let path = path.join("sender-offers-v1");
    let inputs: Inputs = read_pinned(&path.join("inputs.toml"), INPUTS)?;
    let expected: Expected = read_pinned(&path.join("expected.toml"), EXPECTED)?;
    if !inputs.header.valid()
        || !expected.header.valid()
        || inputs.offer.len() != NATIVE_WINDOWS
        || expected.fifo.len() != NATIVE_WINDOWS
        || inputs.offer.windows(2).any(|pair| {
            matches!(pair, [first, second] if first.exec_master_ticks >= second.exec_master_ticks || first.cut > second.cut)
        })
        || !inputs.offer.iter().zip(&expected.fifo).enumerate().all(|(id, (offer, fifo))| {
            id == offer.source_id && id == fifo.source_id && offer.valid_bracket(fifo)
        })
        || !cross_group_order(&inputs.offer)
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

fn native(path: &Path, source_id: usize, cut: u64) -> Result<Window, Outcome> {
    let (offer, fifo) = native_records(path, source_id, cut, false)?;
    let request = Offer {
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
    fn decode(self, cut: u64) -> Result<SenderState, Outcome> {
        let reply = match (self.reply, self.reply_cue) {
            (ReplyKind::Absent, None) => Request::Absent,
            (ReplyKind::Cue, Some(cue)) => Request::Cue(cue),
            _ => return Err(data_failure(Some(cut), "sender seed reply payload")),
        };
        Ok(SenderState {
            now: CpuMasterTicks(self.now_master_ticks),
            reply,
            queue: self.queue,
            progress: self.progress.decode()?,
            pending: None,
        })
    }
}

pub(super) struct ContactIngress {
    pub seed: SenderState,
    pub at: CpuMasterTicks,
    pub current: game::production::frame::post_down_contract::OfferCurrent,
}

pub(super) fn contact_ingress(
    path: &Path,
    source_id: usize,
    cut: u64,
) -> Result<ContactIngress, Outcome> {
    use game::production::frame::post_down_contract::{Demo, OfferCurrent, PowerBomb, Sounds};
    let inputs: Inputs = read_pinned(&path.join("sender-offers-v1/inputs.toml"), INPUTS)?;
    if !inputs.header.valid() || inputs.offer.len() != NATIVE_WINDOWS {
        return Err(data_failure(None, "native Contact ingress header"));
    }
    let offer = inputs
        .offer
        .into_iter()
        .nth(source_id)
        .filter(|offer| {
            offer.source_id == source_id
                && offer.cut == cut
                && offer.group == SoundGroup::Contact
                && offer.seed.boundary == StartBoundary::PrecedingCut
                && offer.seed.cut.and_then(|prior| prior.checked_add(1)) == Some(cut)
                && !offer.seed.service_pending
                && offer.seed.now_master_ticks < offer.prefetch_master_ticks
                && offer.prefetch_master_ticks.checked_add(2) == Some(offer.exec_master_ticks)
        })
        .ok_or_else(|| data_failure(Some(cut), "native Contact ingress bracket"))?;
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
        at: CpuMasterTicks(offer.exec_master_ticks),
        current,
    })
}

pub(super) struct ContactFifo {
    pub ingress_prefetch: u64,
    pub at: CpuMasterTicks,
    pub queue: VecDeque<CueId>,
    pub progress: SenderProgress,
}

pub(super) fn contact_fifo(
    path: &Path,
    source_id: usize,
    cut: u64,
) -> Result<ContactFifo, Outcome> {
    let (offer, fifo) = native_records(path, source_id, cut, true)?;
    Ok(ContactFifo {
        ingress_prefetch: offer.prefetch_master_ticks,
        at: CpuMasterTicks(fifo.master_ticks),
        queue: fifo.queue,
        progress: fifo.progress.decode()?,
    })
}

fn cross_group_order(offers: &[NativeOffer]) -> bool {
    use CueId::{BackgroundSilence, Beam, Door, SamusSilence};
    use SoundGroup::{Action, Ambient, Contact};
    let at_cut = |cut| {
        offers
            .iter()
            .filter(move |offer| offer.cut == cut)
            .map(|offer| (offer.group, offer.cue))
    };
    at_cut(157).eq([(Action, Beam), (Contact, Door)])
        && at_cut(183).eq([(Action, SamusSilence), (Ambient, BackgroundSilence)])
}

fn constructed(control: SenderOfferControl) -> Window {
    use CueId::{Door, Footstep, Land};
    use OfferPermission::{Demo, Disabled, Enabled, Explosion};
    use QueueAllowance::{Six, Three};
    use SenderOfferControl::{
        DenyAtSix, DenyDemo, DenyDisabled, DenyExplosion, DenySmallerLimit, OrderedPair,
        RepeatAtFive,
    };

    let (tail, cues, allowance, permission, after): (&[_], &[_], _, _, &[_]) = match control {
        OrderedPair => (&[], &[Land, Door], Six, Enabled, &[Land, Door]),
        RepeatAtFive => (&[Land], &[Land], Six, Enabled, &[Land, Land]),
        DenyAtSix => (&[Land, Door], &[Land], Six, Enabled, &[Land, Door]),
        DenySmallerLimit => (&[Land], &[Door], Three, Enabled, &[Land]),
        DenyDisabled => (&[], &[Door], Six, Disabled, &[]),
        DenyDemo => (&[], &[Door], Six, Demo, &[]),
        DenyExplosion => (&[], &[Door], Six, Explosion, &[]),
    };
    let queue = |tail: &[CueId]| {
        [Footstep; 4]
            .into_iter()
            .chain(tail.iter().copied())
            .collect()
    };
    Window {
        seed: SenderState {
            now: CpuMasterTicks(0),
            reply: Request::Absent,
            queue: queue(tail),
            progress: SenderProgress::Ready,
            pending: None,
        },
        group: SoundGroup::Contact,
        offers: cues
            .iter()
            .map(|&cue| Offer {
                cue,
                allowance,
                permission,
            })
            .collect(),
        expected: observations(&queue(after), &SenderProgress::Ready),
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
        Start::NativeSenderOffer { source_id } => native(&sources.native, source_id, cut)?,
        Start::ConstructedSenderOffer { control } => constructed(control),
        Start::NativeOpportunity { .. }
        | Start::CacheCut { .. }
        | Start::NativeSeed { .. }
        | Start::NativeCheckpoint { .. }
        | Start::PowerOn => return Err(data_failure(Some(cut), "sender offer start contract")),
    };
    let content = ProductionContent::AudioSender(group);
    let rules = ProductionRules::AudioSender;
    let production = Production::admit(
        ProductionSeed::AudioSender(Box::new(seed)),
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
        actual: ActualReader::SenderOffer,
        schedule: vec![cut],
    })
}

pub(super) fn advance(
    production: &mut Production,
    offers: &[Offer],
    cut: u64,
) -> Result<ProductionOutputs, Outcome> {
    for &Offer {
        cue,
        allowance,
        permission,
    } in offers
    {
        let output = production
            .advance(AdvanceInput::AudioSender(SenderInput::Offer {
                cue,
                allowance,
                permission,
            }))
            .map_err(|fault| production_failure(cut, fault))?;
        if !matches!(output, ProductionOutputs::AudioSender(events) if events.is_empty()) {
            return Err(data_failure(Some(cut), "sender offer output scope"));
        }
    }
    Ok(ProductionOutputs::AudioSender(Vec::new()))
}

pub(super) fn actual(production: &Production) -> Result<Vec<Observation>, Outcome> {
    let OwnedState::AudioSender(state) = production.state() else {
        return Err(data_failure(None, "sender offer projection scope"));
    };
    Ok(observations(&state.0.queue, &state.0.progress))
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
    }
}

pub(super) fn observations(queue: &VecDeque<CueId>, progress: &SenderProgress) -> Vec<Observation> {
    let progress = match *progress {
        SenderProgress::Ready => vec![0],
        SenderProgress::AwaitCue(cue) => vec![1, cue_tag(cue)],
        SenderProgress::ClearDelay(turns) => vec![2, turns.get()],
        SenderProgress::AwaitClear => vec![3],
        SenderProgress::ResetDelay(turns) => vec![4, turns.get()],
    };
    vec![
        Observation {
            field: "sender.queue".to_owned(),
            bytes: queue.iter().copied().map(cue_tag).collect(),
        },
        Observation {
            field: "sender.progress".to_owned(),
            bytes: progress,
        },
    ]
}
