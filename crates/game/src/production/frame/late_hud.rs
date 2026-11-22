//! Drawing through Background, including the late status producer and upload request.

use super::{Buffer, Content, Dependency, Rules, State, hud, s6_drawing, suffix_caller};
use crate::production::{ProductionFault, Unsupported, timing};
use canonical::Canonical;
use data::timing::{Profile, Template};

mod gauge;
use types::Parts;

const ENEMY_BACKGROUND: timing::WorkId = timing::WorkId(1323);
const STATUS: timing::WorkId = timing::WorkId(1324);

/// Immutable complete bounded late HUD and queue work.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Contract {
    /// Quiet enemy background request before status.
    pub enemy_background: Profile,
    /// Actual whole status through the descriptor append tail.
    pub status: Profile,
    /// Checked actual HUD transfer layout and materialization binding.
    pub queue: suffix_caller::HudTransferTemplate,
    gauge: Option<gauge::Contract>,
}

impl Contract {
    /// Bind the quiet status envelope and immutable upload descriptor.
    #[must_use]
    pub const fn new(
        enemy_background: Profile,
        status: Profile,
        queue: suffix_caller::HudTransferTemplate,
    ) -> Self {
        Self {
            enemy_background,
            status,
            queue,
            gauge: None,
        }
    }

    /// Add the reached physical redraw envelope for a changed gauge.
    /// # Errors
    /// The template must expose both tank operations and the dependent decimal operation.
    pub fn with_gauge(mut self, template: &Template) -> Result<Self, ProductionFault> {
        self.gauge = Some(gauge::Contract::new(template).ok_or_else(missing)?);
        Ok(self)
    }
}

/// Active redraw with its captured chosen HUD and live physical arithmetic.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Gauge {
    /// Complete held HUD choice, revalidated against the current owners on admission.
    pub chosen: hud::Hud,
    progress: gauge::Progress,
    first_event: u64,
    math_origin: timing::Math,
}

/// Canonical current late opportunity and its actual queue.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub enum Stage {
    /// Before the distinct enemy-background operation.
    #[default]
    Pending,
    /// Whole current enemy-background request is charging.
    EnemyBackground,
    /// Actual Drawing and background request returned.
    StatusReady,
    /// Actual chosen HUD result held through the physical opportunity.
    Charging {
        /// Complete actual clean chosen HUD.
        chosen: hud::Hud,
        /// Whole energy from actual Maintenance.
        energy: u16,
    },
    /// Actual late HUD and its queued live upload request.
    Completed(suffix_caller::HudQueue),
    /// Actual native consumer has retired the one appended request.
    Retired,
    /// Changed gauge is charging with its own device transactions.
    Gauging(Box<Gauge>),
}

impl Stage {
    /// Whole current Health energy retained by the timed Status choice.
    #[must_use]
    pub fn in_flight_energy(&self) -> Option<u16> {
        match self {
            Self::Charging { chosen, .. } => chosen.chosen_status_energy(),
            Self::Gauging(gauge) => gauge.chosen.chosen_status_energy(),
            Self::Pending
            | Self::EnemyBackground
            | Self::StatusReady
            | Self::Completed(_)
            | Self::Retired => None,
        }
    }
}

const fn missing() -> ProductionFault {
    ProductionFault::Unsupported(Unsupported::Frame(Dependency::LateHudPreparation))
}

impl State {
    pub(super) fn advance_drawing_tail(
        &mut self,
        content: &Content,
        rules: &Rules,
    ) -> Result<bool, ProductionFault> {
        if self.s6_drawing.builder().is_none() {
            self.advance_live_drawing(content)?;
        } else if !matches!(self.late_hud, Stage::Completed(_)) {
            self.advance_late_hud(content, rules)?;
        } else if !matches!(
            self.background,
            suffix_caller::BackgroundScroll::Complete(_)
        ) {
            self.advance_background(content)?;
        } else {
            return Ok(false);
        }
        Ok(true)
    }

    pub(super) fn sample_drawing_tail(
        &mut self,
        active: Option<timing::WorkId>,
        events: &[timing::Event],
    ) -> Result<(), ProductionFault> {
        if active == Some(suffix_caller::BACKGROUND_SCROLL) {
            self.sample_background(events)?;
        } else if let Some(work @ (gauge::TANKS | gauge::DIGITS)) = active {
            let Stage::Gauging(gauge) = &mut self.late_hud else {
                return Err(missing());
            };
            gauge.progress.sample(work, events).ok_or_else(missing)?;
        }
        Ok(())
    }

    pub(super) fn complete_drawing_tail(
        &mut self,
        id: timing::WorkId,
        content: &Content,
    ) -> Result<bool, ProductionFault> {
        match id {
            s6_drawing::BRIDGE | s6_drawing::DRAWING => self.complete_drawing_work(id)?,
            ENEMY_BACKGROUND | STATUS | gauge::TANKS | gauge::DIGITS => {
                self.complete_late_hud(id, content)?;
            }
            suffix_caller::BACKGROUND_SCROLL => self.complete_background(content)?,
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn advance_late_hud(
        &mut self,
        content: &Content,
        rules: &Rules,
    ) -> Result<(), ProductionFault> {
        let contract = content.late_hud.as_ref().ok_or_else(missing)?;
        let energy = self.maintenance.energy().ok_or_else(missing)?;
        let origins = self.drawing_origins.as_ref().ok_or_else(missing)?;
        if origins.actors.background_upload
            || origins.actors.background_hook
            || !self.hud.map_ready()
            || !self.hud_map.is_complete()
        {
            return Err(missing());
        }
        match &self.late_hud {
            Stage::Pending => {
                self.timing
                    .begin(ENEMY_BACKGROUND, contract.enemy_background.clone())
                    .map_err(ProductionFault::FrameHardware)?;
                self.late_hud = Stage::EnemyBackground;
            }
            Stage::EnemyBackground => {
                if !self.has_physical_bill(ENEMY_BACKGROUND, &contract.enemy_background) {
                    return Err(missing());
                }
            }
            Stage::StatusReady => {
                let chosen = self.choose_late_status_from(&self.hud, content, rules)?;
                if self.hud.state().shown.energy == energy {
                    self.timing
                        .begin(STATUS, contract.status.clone())
                        .map_err(ProductionFault::FrameHardware)?;
                    self.late_hud = Stage::Charging { chosen, energy };
                } else {
                    if !self.publications.original_interval() {
                        return Err(missing());
                    }
                    let gauge = contract.gauge.as_ref().ok_or_else(missing)?;
                    let profile = gauge
                        .tanks(
                            energy,
                            chosen.chosen_status_input().ok_or_else(missing)?.capacity,
                        )
                        .ok_or_else(missing)?;
                    self.timing
                        .begin(gauge::TANKS, profile)
                        .map_err(ProductionFault::FrameHardware)?;
                    self.late_hud = Stage::Gauging(Box::new(Gauge {
                        chosen,
                        progress: gauge::Progress::Tanks(Vec::new()),
                        first_event: u64::try_from(self.publications.events.len())
                            .map_err(|_| missing())?,
                        math_origin: self.timing.hardware.math,
                    }));
                }
            }
            Stage::Gauging(_) => self.advance_status_digits(contract)?,
            Stage::Charging { .. } => {}
            Stage::Completed(_) | Stage::Retired => return Err(missing()),
        }
        Ok(())
    }

    fn choose_late_status_from(
        &self,
        before: &hud::Hud,
        content: &Content,
        rules: &Rules,
    ) -> Result<hud::Hud, ProductionFault> {
        let display = &self.hud_display_current.as_ref().ok_or_else(missing)?.0;
        if self.publications.original_interval()
            && content
                .hud_refresh
                .as_ref()
                .is_none_or(|admitted| &admitted.source != display.as_ref())
        {
            return Err(missing());
        }
        let mut status = display.status;
        status.energy.now = self.maintenance.energy().ok_or_else(missing)?;
        let samus = self.movement.state();
        let inputs = display
            .premises
            .inputs(
                status,
                types::Pair {
                    x: samus.position.x.pixel().0,
                    y: samus.position.y.pixel().0,
                },
                samus.pose.action,
            )
            .status_inputs();
        let mut chosen = before.clone();
        if !chosen.choose_late_status(rules.hud.get(), &content.hud, inputs) {
            return Err(missing());
        }
        Ok(chosen)
    }

    fn advance_status_digits(&mut self, contract: &Contract) -> Result<(), ProductionFault> {
        let Stage::Gauging(gauge) = &mut self.late_hud else {
            return Err(missing());
        };
        if let gauge::Progress::DigitsReady(remainder) = gauge.progress {
            let profile = contract
                .gauge
                .as_ref()
                .and_then(|gauge| gauge.digits(remainder))
                .ok_or_else(missing)?;
            self.timing
                .begin(gauge::DIGITS, profile)
                .map_err(ProductionFault::FrameHardware)?;
            gauge.progress = gauge::Progress::Digits {
                remainder,
                samples: Vec::new(),
            };
        }
        Ok(())
    }

    pub(super) fn validate_late_status_work(
        &self,
        content: &Content,
    ) -> Result<(), ProductionFault> {
        let work = self.active_status_work()?;
        match &self.late_hud {
            Stage::Charging { chosen, energy } => {
                let contract = content.late_hud.as_ref().ok_or_else(missing)?;
                if self.maintenance.energy() != Some(*energy)
                    || self.hud.state().shown.energy != *energy
                    || chosen.chosen_status_energy() != Some(*energy)
                    || work.is_none_or(|work| {
                        work.kind != timing::WorkKind::Semantic(STATUS)
                            || work.profile != contract.status
                    })
                {
                    return Err(missing());
                }
            }
            Stage::Gauging(gauge) => {
                let live = gauge.chosen.chosen_status_input().ok_or_else(missing)?;
                let expected_work = gauge.progress.work().map(timing::WorkKind::Semantic);
                if !self.publications.original_interval()
                    || self.maintenance.energy() != Some(live.now)
                    || self.hud.state().shown.energy == live.now
                    || expected_work.as_ref() != work.map(|work| &work.kind)
                {
                    return Err(missing());
                }
            }
            Stage::Pending
            | Stage::EnemyBackground
            | Stage::StatusReady
            | Stage::Completed(_)
            | Stage::Retired
                if work.is_some() =>
            {
                return Err(missing());
            }
            Stage::Pending
            | Stage::EnemyBackground
            | Stage::StatusReady
            | Stage::Completed(_)
            | Stage::Retired => {}
        }
        Ok(())
    }

    fn active_status_work(&self) -> Result<Option<&timing::Work>, ProductionFault> {
        let mut active = self.timing.work.iter().filter(|work| {
            matches!(
                work.kind,
                timing::WorkKind::Semantic(STATUS | gauge::TANKS | gauge::DIGITS)
            )
        });
        let work = active.next();
        if active.next().is_some() {
            return Err(missing());
        }
        Ok(work)
    }

    pub(super) fn validate_late_status_checkpoint(
        &self,
        content: &Content,
        rules: &Rules,
    ) -> Result<(), ProductionFault> {
        self.validate_late_status_work(content)?;
        match &self.late_hud {
            Stage::Charging { chosen, .. }
                if self.choose_late_status_from(&self.hud, content, rules)? != *chosen =>
            {
                Err(missing())
            }
            Stage::Gauging(gauge) => {
                self.validate_gauge_work(gauge, self.active_status_work()?, content, rules)
            }
            Stage::Pending
            | Stage::EnemyBackground
            | Stage::StatusReady
            | Stage::Charging { .. }
            | Stage::Completed(_)
            | Stage::Retired => Ok(()),
        }
    }

    fn validate_gauge_work(
        &self,
        gauge: &Gauge,
        work: Option<&timing::Work>,
        content: &Content,
        rules: &Rules,
    ) -> Result<(), ProductionFault> {
        let live = gauge.chosen.chosen_status_input().ok_or_else(missing)?;
        let contract = content
            .late_hud
            .as_ref()
            .and_then(|contract| contract.gauge.as_ref())
            .ok_or_else(missing)?;
        let events = usize::try_from(gauge.first_event)
            .ok()
            .and_then(|ordinal| self.publications.events.get(ordinal..))
            .ok_or_else(missing)?;
        if self.choose_late_status_from(&self.hud, content, rules)? != gauge.chosen
            || gauge.first_event < self.publications.interval_start()
            || !gauge.progress.holds(live.now, live.capacity, events)
        {
            return Err(missing());
        }
        contract
            .prefix_holds(
                &gauge.progress,
                live,
                work,
                &self.timing.hardware,
                gauge.math_origin,
                events,
            )
            .ok_or_else(missing)
    }

    pub(super) fn completed_late_hud_queue(
        &self,
        content: &Content,
    ) -> Result<Option<suffix_caller::HudQueue>, ProductionFault> {
        Ok(self
            .completed_late_hud_transfer(content)?
            .map(|(queue, _)| queue.clone()))
    }

    pub(super) fn validate_late_hud_publication(
        &self,
        content: &Content,
        rules: &Rules,
    ) -> Result<(), ProductionFault> {
        let acknowledged = matches!(
            self.return_progress,
            super::frame_return::Stage::Ready {
                suffix: suffix_caller::Suffix::Acknowledged { .. },
                ..
            }
        );
        if acknowledged != matches!(self.late_hud, Stage::Retired) {
            return Err(missing());
        }
        if !matches!(self.late_hud, Stage::Completed(_) | Stage::Retired) {
            return Ok(());
        }
        let contract = content.late_hud.as_ref().ok_or_else(missing)?;
        let before = self
            .hud_map
            .applied_ordinary()
            .and_then(|map| self.hud.before_late_status(map))
            .ok_or_else(missing)?;
        let mut expected = self.choose_late_status_from(&before, content, rules)?;
        if !expected.apply_late_status()
            || expected.events() != before.events()
            || expected != self.hud
        {
            return Err(missing());
        }
        let buffer = self.late_hud_materialization(&expected)?;
        if self.content.ready_buffer(Buffer::HudTilemap) != Some(&buffer) {
            return Err(missing());
        }
        let queue = Self::late_hud_queue_from(&buffer, contract)?;
        if let Stage::Completed(retained) = &self.late_hud
            && *retained != queue
        {
            return Err(missing());
        }
        Ok(())
    }

    fn completed_late_hud_transfer<'a>(
        &'a self,
        content: &'a Content,
    ) -> Result<Option<(&'a suffix_caller::HudQueue, &'a Contract)>, ProductionFault> {
        let Stage::Completed(retained) = &self.late_hud else {
            return Ok(None);
        };
        let contract = content.late_hud.as_ref().ok_or_else(missing)?;
        let suffix_caller::HudQueue::Queued(request) = retained else {
            return Err(missing());
        };
        if request.template != contract.queue {
            return Err(missing());
        }
        Ok(Some((retained, contract)))
    }

    fn late_hud_materialization(
        &self,
        hud: &hud::Hud,
    ) -> Result<super::Materialization, ProductionFault> {
        let (status, _) = hud.publication().ok_or_else(missing)?;
        let tilemap = hud::tilemap(status).map_err(|_| missing())?;
        let ready = self
            .content
            .ready_buffer(Buffer::HudTilemap)
            .ok_or_else(missing)?;
        Ok(super::Materialization {
            buffer: Buffer::HudTilemap,
            region: data::timing::ContentRegion {
                start: ready.region.start,
                bytes: tilemap,
            },
        })
    }

    fn late_hud_queue_from(
        output: &super::Materialization,
        contract: &Contract,
    ) -> Result<suffix_caller::HudQueue, ProductionFault> {
        let mut queue = suffix_caller::HudQueue::Empty;
        contract
            .queue
            .append_hud(&output.region, &mut queue)
            .map_err(|_| missing())?;
        Ok(queue)
    }

    fn complete_late_hud(
        &mut self,
        id: timing::WorkId,
        content: &Content,
    ) -> Result<(), ProductionFault> {
        let contract = content.late_hud.as_ref().ok_or_else(missing)?;
        match (&mut self.late_hud, id) {
            (Stage::EnemyBackground, ENEMY_BACKGROUND) => self.late_hud = Stage::StatusReady,
            (Stage::Gauging(gauge), gauge::TANKS) => {
                let energy = gauge.chosen.chosen_status_input().ok_or_else(missing)?;
                gauge
                    .progress
                    .complete_tanks(energy.now, energy.capacity)
                    .ok_or_else(missing)?;
            }
            (Stage::Charging { chosen, .. }, STATUS) => {
                let hud = chosen.clone();
                self.publish_late_hud(hud, contract)?;
            }
            (Stage::Gauging(gauge), gauge::DIGITS) if gauge.progress.digits_complete() => {
                let hud = gauge.chosen.clone();
                self.publish_late_hud(hud, contract)?;
            }
            _ => return Err(missing()),
        }
        Ok(())
    }

    fn publish_late_hud(
        &mut self,
        mut hud: hud::Hud,
        contract: &Contract,
    ) -> Result<(), ProductionFault> {
        if !hud.apply_late_status() || hud.events() != self.hud.events() {
            return Err(missing());
        }
        let output = self.late_hud_materialization(&hud)?;
        if self.hud.state().shown.energy == hud.state().shown.energy
            && self.content.ready_buffer(Buffer::HudTilemap) != Some(&output)
        {
            return Err(missing());
        }
        let queue = Self::late_hud_queue_from(&output, contract)?;
        self.content.refresh_hud(output).map_err(|_| missing())?;
        self.hud = hud;
        self.late_hud = Stage::Completed(queue);
        Ok(())
    }
}
