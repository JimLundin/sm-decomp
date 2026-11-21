//! Drawing through Background, including the late status producer and upload request.

use super::{Buffer, Content, Dependency, Rules, State, hud, s6_drawing, suffix_caller};
use crate::production::{ProductionFault, Unsupported, timing};
use canonical::Canonical;
use data::timing::Profile;
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
            ENEMY_BACKGROUND | STATUS => self.complete_late_hud(id, content)?,
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
                self.validate_late_hud_profile(ENEMY_BACKGROUND, &contract.enemy_background)?;
            }
            Stage::StatusReady => {
                let display = &self.hud_display_current.as_ref().ok_or_else(missing)?.0;
                let mut status = display.status;
                status.energy.now = energy;
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
                let mut chosen = self.hud.clone();
                if !chosen.choose_late_status(rules.hud.get(), &content.hud, inputs) {
                    return Err(missing());
                }
                self.timing
                    .begin(STATUS, contract.status.clone())
                    .map_err(ProductionFault::FrameHardware)?;
                self.late_hud = Stage::Charging { chosen, energy };
            }
            Stage::Charging {
                energy: retained, ..
            } => {
                if *retained != energy {
                    return Err(missing());
                }
                self.validate_late_hud_profile(STATUS, &contract.status)?;
            }
            Stage::Completed(_) | Stage::Retired => return Err(missing()),
        }
        Ok(())
    }

    fn validate_late_hud_profile(
        &self,
        id: timing::WorkId,
        profile: &Profile,
    ) -> Result<(), ProductionFault> {
        let mut active = self
            .timing
            .work
            .iter()
            .filter(|work| work.kind == timing::WorkKind::Semantic(id));
        if active.next().is_some_and(|work| &work.profile == profile) && active.next().is_none() {
            Ok(())
        } else {
            Err(missing())
        }
    }

    fn complete_late_hud(
        &mut self,
        id: timing::WorkId,
        content: &Content,
    ) -> Result<(), ProductionFault> {
        let contract = content.late_hud.as_ref().ok_or_else(missing)?;
        match (&self.late_hud, id) {
            (Stage::EnemyBackground, ENEMY_BACKGROUND) => self.late_hud = Stage::StatusReady,
            (Stage::Charging { chosen, energy }, STATUS) => {
                if self.maintenance.energy() != Some(*energy) {
                    return Err(missing());
                }
                let mut hud = chosen.clone();
                if !hud.apply_late_status() || hud.events() != self.hud.events() {
                    return Err(missing());
                }
                let (status, _) = hud.publication().ok_or_else(missing)?;
                let tilemap = hud::tilemap(status).map_err(|_| missing())?;
                let ready = self
                    .content
                    .ready_buffer(Buffer::HudTilemap)
                    .ok_or_else(missing)?;
                if ready.region.bytes != tilemap {
                    return Err(missing());
                }
                let mut queue = suffix_caller::HudQueue::Empty;
                contract
                    .queue
                    .append_hud(&ready.region, &mut queue)
                    .map_err(|_| missing())?;
                self.hud = hud;
                self.late_hud = Stage::Completed(queue);
            }
            _ => return Err(missing()),
        }
        Ok(())
    }
}
