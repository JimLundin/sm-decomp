//! Actual Frame Health and HUD phase observations and independent native comparisons.

use super::{FinalHealth, Observation, data_failure, read_expected, word};
use crate::lock::Sha256;
use crate::replay::result::Outcome;
use data::timing::ChannelSlot;
use game::production::frame::health::Admission;
use game::production::{Production, frame, timing};
use game::state::owned::OwnedState;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use types::Parts;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::replay) enum Arm {
    Stock,
    PendingLoss,
}

impl Arm {
    const ROOT: &'static str = "connected-health-v1";
    const ORIGIN_PIN: &'static str =
        "bfee6ac13da413a36ad42194a36aa3224af6a84e818092b1e7040ed4782f03db";

    pub(in crate::replay) fn from_case(case: &crate::replay::index::Case) -> Option<Self> {
        match case.id.0.as_str() {
            "frame/health-hud-stock" => Some(Self::Stock),
            "frame/health-hud-pending-loss" => Some(Self::PendingLoss),
            _ => None,
        }
    }

    pub(in crate::replay) fn configure(
        self,
        root: &Path,
        wram: &[u8],
        row: &str,
        health: &mut Admission,
    ) -> Result<(), Outcome> {
        let fault = || data_failure(None, "native pending Health input provenance");
        let source = SourceMember {
            path: Path::new(Self::ROOT).join("initial-owner.toml"),
            sha256: "ad001822b4ebdadbfe2495aabad3d288196b5c4ba48afc77d8a9ad956ad0f223".to_owned(),
        };
        let bytes = source.read(root, 1)?;
        let source: Setup = toml::from_str(std::str::from_utf8(&bytes).map_err(|_| fault())?)
            .map_err(|_| fault())?;
        if source.format != 1
            || source.basis.is_empty()
            || source.row != row
            || source.seed != "any-boundary:3"
            || (source.origin, source.end) != (9232, 9233)
            || source.stock_origin_sha256 != Self::ORIGIN_PIN
            || Sha256::of(wram) != Sha256::try_from(Self::ORIGIN_PIN).map_err(|_| fault())?
            || source.stock.pending_whole_override
            || source.stock.pending_whole_energy.is_some()
            || !source.single_pending_loss.pending_whole_override
            || source.single_pending_loss.pending_whole_energy != Some(1)
            || health.initial.0.pending_loss != samus_health::EnergyLoss::ZERO
        {
            return Err(fault());
        }
        if self == Self::PendingLoss {
            health.initial.0.pending_loss.whole = samus_health::WholeEnergy::new(
                source
                    .single_pending_loss
                    .pending_whole_energy
                    .ok_or_else(fault)?,
            )
            .ok_or_else(fault)?;
        }
        Ok(())
    }

    pub(in crate::replay) fn profiles(root: &Path) -> Result<String, Outcome> {
        let source = SourceMember {
            path: Path::new(Self::ROOT).join("profiles.toml"),
            sha256: "c56c5031fe69303359fde4aec6176572ea1562fef0d30df1b9112efe16bd13df".to_owned(),
        };
        String::from_utf8(source.read(root, 1)?)
            .map_err(|_| data_failure(None, "native Health and HUD profile encoding"))
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Setup {
    format: u8,
    basis: String,
    row: String,
    seed: String,
    origin: u64,
    #[serde(rename = "return")]
    end: u64,
    stock_origin_sha256: String,
    stock: PendingLoss,
    single_pending_loss: PendingLoss,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PendingLoss {
    pending_whole_override: bool,
    pending_whole_energy: Option<u16>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceMember {
    path: PathBuf,
    sha256: String,
}

impl SourceMember {
    fn read(&self, root: &Path, cut: u64) -> Result<Vec<u8>, Outcome> {
        let pin = Sha256::try_from(self.sha256.as_str())
            .map_err(|_| data_failure(Some(cut), "Health and HUD native phase pin"))?;
        read_expected(&root.join(&self.path), pin, cut)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    before_map: SourceMember,
    after_map: SourceMember,
    animation: SourceMember,
    maintenance: SourceMember,
    drawing_before_history: SourceMember,
    drawing_return: SourceMember,
    status_read: SourceMember,
    queue: SourceMember,
    vram: SourceMember,
}

pub(in crate::replay) struct Source {
    pub path: PathBuf,
    pub sha256: Sha256,
}

impl Source {
    pub(super) fn read(&self, cut: u64) -> Result<Vec<Observation>, Outcome> {
        let fault = || data_failure(Some(cut), "Health and HUD original phase observations");
        let bytes = read_expected(&self.path, self.sha256, cut)?;
        let manifest: Manifest = toml::from_str(std::str::from_utf8(&bytes).map_err(|_| fault())?)
            .map_err(|_| fault())?;
        let root = self.path.parent().ok_or_else(fault)?;
        let before_map = manifest.before_map.read(root, cut)?;
        let after_map = manifest.after_map.read(root, cut)?;
        let animation = manifest.animation.read(root, cut)?;
        let maintenance = manifest.maintenance.read(root, cut)?;
        let drawing_read = manifest.drawing_before_history.read(root, cut)?;
        let drawing_return = manifest.drawing_return.read(root, cut)?;
        let status_read = manifest.status_read.read(root, cut)?;
        let queue = manifest.queue.read(root, cut)?;
        let vram = manifest.vram.read(root, cut)?;
        let bytes = |image: &[u8], start: usize, amount: usize| {
            image
                .get(start..start.saturating_add(amount))
                .map(<[u8]>::to_vec)
                .ok_or_else(fault)
        };
        let scalar = |image: &[u8], start| -> Result<u16, Outcome> {
            let bytes = bytes(image, start, 2)?;
            Ok(u16::from_le_bytes(bytes.try_into().map_err(|_| fault())?))
        };
        let map_fields_from = |phase: &str, image: &[u8]| -> Result<_, Outcome> {
            Ok(map_fields(
                phase,
                &bytes(image, 0xc608, 192)?,
                bytes(image, 0x07f7, 256)?,
                scalar(image, 0x0a06)?,
            ))
        };
        let transfer = bytes(&queue, 0xd0, 7)?;
        let target = usize::from(scalar(&queue, 0xd5)?).saturating_mul(2);
        let mut fields = Vec::from(map_fields_from("before", &before_map)?);
        fields.push(words(
            "frame.health_hud.map.position",
            [scalar(&after_map, 0x0af6)?, scalar(&after_map, 0x0afa)?],
        ));
        fields.extend(map_fields_from("after", &after_map)?);
        let remaining: [Observation; PHASE_TAIL_FIELDS] = [
            word(
                "frame.health_hud.animation.energy",
                scalar(&animation, 0x09c2)?,
            ),
            word(
                "frame.health_hud.animation.pending",
                scalar(&animation, 0x0a50)?,
            ),
            words(
                "frame.health_hud.maintenance",
                [
                    scalar(&maintenance, 0x09c2)?,
                    scalar(&maintenance, 0x0a4c)?,
                    scalar(&maintenance, 0x0a50)?,
                    scalar(&maintenance, 0x0a4e)?,
                    scalar(&maintenance, 0x0a6a)?,
                ],
            ),
            words(
                "frame.health_hud.drawing.compare",
                [
                    scalar(&drawing_read, 0x09c2)?,
                    scalar(&drawing_read, 0x0a12)?,
                ],
            ),
            word(
                "frame.health_hud.drawing.history",
                scalar(&drawing_return, 0x0a12)?,
            ),
            words(
                "frame.health_hud.status.read",
                [scalar(&status_read, 0x09c2)?, scalar(&status_read, 0x0a06)?],
            ),
            block(
                "frame.health_hud.status.before_tiles",
                bytes(&status_read, 0xc608, 192)?,
            ),
            word("frame.health_hud.status.shown", scalar(&queue, 0x0a06)?),
            block("frame.health_hud.queue.descriptor", transfer),
            block(
                "frame.health_hud.queue.tilemap",
                bytes(&queue, 0xc608, 192)?,
            ),
            block("frame.health_hud.vram", bytes(&vram, target, 192)?),
        ];
        fields.extend(remaining);
        Ok(fields)
    }
}

fn block(field: &str, bytes: Vec<u8>) -> Observation {
    Observation {
        field: field.to_owned(),
        bytes,
    }
}

fn words<const N: usize>(field: &str, values: [u16; N]) -> Observation {
    block(
        field,
        values.into_iter().flat_map(u16::to_le_bytes).collect(),
    )
}

fn maintenance(health: &FinalHealth) -> Observation {
    words(
        "frame.health_hud.maintenance",
        [
            health.energy_whole,
            health.energy_fraction,
            health.pending_whole,
            health.pending_fraction,
            health.warning,
        ],
    )
}

fn map_fields(
    phase: &str,
    tilemap: &[u8],
    explored: Vec<u8>,
    shown: u16,
) -> [Observation; MAP_FIELDS] {
    let window = tilemap
        .as_chunks::<64>()
        .0
        .iter()
        .flat_map(|row| row.iter().skip(52).take(10).copied())
        .collect();
    let gauge = [2..16, 66..80, 140..144]
        .into_iter()
        .filter_map(|span| tilemap.get(span))
        .flatten()
        .copied()
        .collect();
    [
        block(&format!("frame.health_hud.map.{phase}_window"), window),
        block(&format!("frame.health_hud.map.{phase}_explored"), explored),
        word(&format!("frame.health_hud.map.{phase}_shown"), shown),
        block(&format!("frame.health_hud.map.{phase}_gauge"), gauge),
    ]
}

pub(in crate::replay) fn null(production: &Production) -> Result<Vec<Observation>, Outcome> {
    let fault = || data_failure(None, "frozen Health and HUD origin");
    let OwnedState::Frame(state) = production.state() else {
        return Err(fault());
    };
    let hud = state.hud.state();
    let tiles = frame::hud::tilemap(hud).map_err(|_| fault())?;
    let explored = super::super::hud::explored_bitmap(hud);
    let health = FinalHealth::from(state.health.as_ref().ok_or_else(fault)?.state());
    let prior = state
        .frame_history
        .as_ref()
        .ok_or_else(fault)?
        .previous_health();
    let position = state.movement.state().position;
    let mut fields = Vec::from(map_fields(
        "before",
        &tiles,
        explored.clone(),
        hud.shown.energy,
    ));
    fields.push(words(
        "frame.health_hud.map.position",
        [position.x.pixel().0, position.y.pixel().0],
    ));
    fields.extend(map_fields("after", &tiles, explored, hud.shown.energy));
    fields.extend([
        word("frame.health_hud.animation.energy", health.energy_whole),
        word("frame.health_hud.animation.pending", health.pending_whole),
        maintenance(&health),
        words(
            "frame.health_hud.drawing.compare",
            [health.energy_whole, prior],
        ),
        word("frame.health_hud.drawing.history", prior),
        words(
            "frame.health_hud.status.read",
            [health.energy_whole, hud.shown.energy],
        ),
        block("frame.health_hud.status.before_tiles", tiles),
        word("frame.health_hud.status.shown", hud.shown.energy),
    ]);
    Ok(fields)
}

const MAP_FIELDS: usize = 4;
const PHASE_TAIL_FIELDS: usize = 11;

/// Independently observed phase fields in a joined return.
pub(in crate::replay) const FIELDS_PER_CUT: usize = MAP_FIELDS
    .saturating_mul(2)
    .saturating_add(1)
    .saturating_add(PHASE_TAIL_FIELDS);

/// Mid-frame checkpoints captured before the terminal return checkpoint.
pub(in crate::replay) const PHASE_RESTORES_PER_CUT: usize = CHECKPOINT_PHASES.len();

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Phase {
    #[default]
    BeforeMap,
    AfterMap,
    Animation,
    Maintained,
    Drawing,
    Drawn,
    Status,
    Queued,
    Retired,
}

const CHECKPOINT_PHASES: [Phase; 8] = [
    Phase::BeforeMap,
    Phase::AfterMap,
    Phase::Animation,
    Phase::Maintained,
    Phase::Drawing,
    Phase::Drawn,
    Phase::Status,
    Phase::Queued,
];

#[derive(Clone, Default, PartialEq, Eq)]
pub(super) struct Trace {
    next: Phase,
    observations: Vec<Observation>,
    publication: Option<(usize, frame::suffix_caller::HudTransferTemplate)>,
}

impl Trace {
    pub(super) fn observe(
        &mut self,
        production: &Production,
        events: &[timing::Event],
        cut: u64,
    ) -> Result<bool, Outcome> {
        let OwnedState::Frame(state) = production.state() else {
            return Err(data_failure(Some(cut), "Health and HUD frame owner"));
        };
        let phase = self.next;
        let previous = self.observations.len();
        while let Some(next) = self.observe_phase(state, cut)? {
            self.next = next;
        }
        if self.next == Phase::Queued {
            self.observe_queue(state, events.len(), cut)?;
        }
        Ok(CHECKPOINT_PHASES.contains(&phase) && self.observations.len() != previous)
    }

    fn observe_phase(&mut self, state: &frame::State, cut: u64) -> Result<Option<Phase>, Outcome> {
        let fault = || data_failure(Some(cut), "Health and HUD phase owner");
        let fields = &mut self.observations;
        let hud = state.hud.state();
        let tilemap = || frame::hud::tilemap(hud).map_err(|_| fault());
        let phase = match self.next {
            Phase::BeforeMap
                if state.hud_map.identity() == Some(frame::hud_camera_caller::HUD_REFRESH) =>
            {
                fields.extend(map_fields(
                    "before",
                    &tilemap()?,
                    super::super::hud::explored_bitmap(hud),
                    hud.shown.energy,
                ));
                Phase::AfterMap
            }
            Phase::AfterMap => {
                let Some(map) = state.hud.applied_map() else {
                    return Ok(None);
                };
                let position = map.inputs.position;
                fields.push(words(
                    "frame.health_hud.map.position",
                    [position.x, position.y],
                ));
                fields.extend(map_fields(
                    "after",
                    &tilemap()?,
                    super::super::hud::explored_bitmap(hud),
                    hud.shown.energy,
                ));
                Phase::Animation
            }
            Phase::Animation => {
                let frame::animation_caller::Stage::Charging(animation) = &state.animation else {
                    return Ok(None);
                };
                let energy = animation
                    .receipt()
                    .and_then(|receipt| receipt.chosen.model.0.pending.as_ref())
                    .ok_or_else(fault)?
                    .inputs
                    .energy;
                fields.push(word("frame.health_hud.animation.energy", energy));
                let health = state.health.as_ref().ok_or_else(fault)?.state();
                fields.push(word(
                    "frame.health_hud.animation.pending",
                    health.pending_loss.whole.get(),
                ));
                Phase::Maintained
            }
            Phase::Maintained if state.maintenance.energy().is_some() => {
                let health = state.health.as_ref().ok_or_else(fault)?;
                fields.push(maintenance(&FinalHealth::from(health.state())));
                Phase::Drawing
            }
            Phase::Drawing => {
                let Some(read) = state.drawing_energy_read() else {
                    return Ok(None);
                };
                fields.push(words(
                    "frame.health_hud.drawing.compare",
                    [read.current, read.previous],
                ));
                Phase::Drawn
            }
            Phase::Drawn if state.s6_drawing.builder().is_some() => {
                let history = state.frame_history.as_ref().ok_or_else(fault)?;
                fields.push(word(
                    "frame.health_hud.drawing.history",
                    history.previous_health(),
                ));
                Phase::Status
            }
            Phase::Status => {
                let Some(energy) = state.late_hud.in_flight_energy() else {
                    return Ok(None);
                };
                fields.push(words(
                    "frame.health_hud.status.read",
                    [energy, hud.shown.energy],
                ));
                fields.push(block("frame.health_hud.status.before_tiles", tilemap()?));
                Phase::Queued
            }
            Phase::BeforeMap
            | Phase::Maintained
            | Phase::Drawn
            | Phase::Queued
            | Phase::Retired => {
                return Ok(None);
            }
        };
        Ok(Some(phase))
    }

    fn observe_queue(
        &mut self,
        state: &frame::State,
        event_count: usize,
        cut: u64,
    ) -> Result<(), Outcome> {
        let fault = || data_failure(Some(cut), "Health and HUD queue payload");
        let fields = &mut self.observations;
        let hud = state.hud.state();
        let frame::late_hud::Stage::Completed(frame::suffix_caller::HudQueue::Queued(request)) =
            &state.late_hud
        else {
            return Ok(());
        };
        if self.publication.is_none() {
            let descriptor = &request.template;
            let region = state
                .content
                .regions()
                .into_iter()
                .find(|region| region.start == descriptor.source)
                .ok_or_else(fault)?;
            fields.push(word("frame.health_hud.status.shown", hud.shown.energy));
            fields.push(block(
                "frame.health_hud.queue.descriptor",
                descriptor
                    .length
                    .to_le_bytes()
                    .into_iter()
                    .chain(descriptor.source.offset.to_le_bytes())
                    .chain([descriptor.source.bank])
                    .chain(descriptor.target.0.to_le_bytes())
                    .collect(),
            ));
            fields.push(block("frame.health_hud.queue.tilemap", region.bytes));
            self.publication = Some((event_count, descriptor.clone()));
        }
        Ok(())
    }

    pub(super) fn compare_prefix(&self, expected: &Source, cut: u64) -> Result<(), Outcome> {
        if self.observations.is_empty() {
            return Ok(());
        }
        let fields = expected.read(cut)?;
        let prefix = fields
            .get(..self.observations.len())
            .ok_or_else(|| data_failure(Some(cut), "Health and HUD phase prefix"))?;
        super::super::compared(cut, prefix, &self.observations)
    }

    pub(super) fn finish(
        &mut self,
        production: &Production,
        events: &[timing::Event],
        cut: u64,
    ) -> Result<Vec<Observation>, Outcome> {
        let fault = || data_failure(Some(cut), "Health and HUD queue or native consumer");
        let OwnedState::Frame(state) = production.state() else {
            return Err(fault());
        };
        let Some((enqueue, descriptor)) = self.publication.take() else {
            return Err(fault());
        };
        if !matches!(state.late_hud, frame::late_hud::Stage::Retired) {
            return Err(fault());
        }
        let bytes = hud_payload(
            events.get(enqueue..).ok_or_else(fault)?,
            usize::from(descriptor.length),
        )
        .ok_or_else(fault)?;
        self.observations
            .push(block("frame.health_hud.vram", bytes));
        self.next = Phase::Retired;
        Ok(std::mem::take(&mut self.observations))
    }
}

fn hud_payload(events: &[timing::Event], length: usize) -> Option<Vec<u8>> {
    use timing::EventKind;
    let mut matches = events.iter().enumerate().filter_map(|(ordinal, event)| {
        if !matches!(event.kind, EventKind::BulkService(ChannelSlot::Two)) {
            return None;
        }
        let body = events
            .get(ordinal.saturating_add(1)..)?
            .iter()
            .take_while(|event| {
                !matches!(
                    event.kind,
                    EventKind::Publication(_)
                        | EventKind::BulkService(_)
                        | EventKind::LineService(_)
                )
            });
        let bytes: Vec<_> = body
            .filter_map(|event| {
                let EventKind::Payload(payload) = &event.kind else {
                    return None;
                };
                matches!(payload.target.0, 0x18 | 0x19).then_some(payload.value)
            })
            .collect();
        (bytes.len() == length).then_some(bytes)
    });
    let selected = matches.next()?;
    matches.next().is_none().then_some(selected)
}
