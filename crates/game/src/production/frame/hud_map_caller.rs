//! Fresh HUD inputs, complete tilemap readiness and its separate physical return.

use super::recurring_hud_map::RecurringDisplay;

use super::{Caller, Dependency, State, hud::Hud, hud_camera_caller, hud_stage};
use crate::production::{ProductionFault, Unsupported, timing};
use canonical::{Canonical, model::Model};
use data::timing::Profile;
use types::Parts;

/// Independently admitted unchanged display owners and their complete physical contract.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Content {
    /// Actual refresh producer and separately exposed physical return.
    pub physical: hud_camera_caller::RefreshPhysical,
    /// Full checked source capability; movement position and action are supplied at use.
    pub source: hud_stage::DisplayReceipt,
    /// Checked room identity backing the same imported display room and map.
    pub room: data::id::RoomId,
    /// Exact destination of the actual completed tilemap producer.
    pub address: data::dma::BusAddress,
    /// Scoped unchanged external owners and physical branch of the adjacent opportunity.
    pub recurring: Option<RecurringDisplay>,
}

/// Read-only causal inputs and full actual HUD results retained during this consumer.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Context {
    before: Hud,
    chosen: Hud,
    result: Hud,
    produced: Option<Hud>,
    movement: super::movement::Movement,
    controls: hud_stage::ControlsCompletion,
    source: Model<hud_stage::DisplayReceiptPin>,
    room: data::id::RoomId,
}

/// Actual refresh/readiness/return progress; no future display input is frontloaded.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub enum Stage {
    /// Completed Poststep has not yet offered fresh display inputs.
    #[default]
    Pending,
    /// Full actual choice and output-consumer progress survive interruption.
    Active {
        /// Distinct actual producer readiness and physical return phases.
        phase: hud_camera_caller::Refresh,
        /// The actual current owners used by this choice.
        context: Box<Context>,
    },
}

const fn missing() -> ProductionFault {
    ProductionFault::Unsupported(Unsupported::Frame(Dependency::HudDisplayPreparation))
}

impl Stage {
    /// Actual semantic work awaiting its returned physical identity.
    #[must_use]
    pub const fn identity(&self) -> Option<timing::WorkId> {
        match self {
            Self::Active {
                phase: hud_camera_caller::Refresh::Charging,
                ..
            } => Some(hud_camera_caller::HUD_REFRESH),
            Self::Active {
                phase: hud_camera_caller::Refresh::Returning,
                ..
            } => Some(hud_camera_caller::HUD_RETURN),
            Self::Pending
            | Self::Active {
                phase:
                    hud_camera_caller::Refresh::Pending
                    | hud_camera_caller::Refresh::Ready
                    | hud_camera_caller::Refresh::Complete,
                ..
            } => None,
        }
    }

    /// A returned producer and its physical return have both completed.
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        matches!(
            self,
            Self::Active {
                phase: hud_camera_caller::Refresh::Complete,
                ..
            }
        )
    }

    pub(super) fn holds(&self, state: &State) -> bool {
        let active: Vec<_> = state
            .timing
            .work
            .iter()
            .filter_map(|work| match work.kind {
                timing::WorkKind::Semantic(id @ timing::WorkId(1200..=1201)) => Some(id),
                timing::WorkKind::Semantic(_)
                | timing::WorkKind::Wait(_)
                | timing::WorkKind::Native { .. }
                | timing::WorkKind::Handler { .. } => None,
            })
            .collect();
        if active.as_slice() != self.identity().as_slice() {
            return false;
        }
        self.owners_hold(state)
    }

    fn owners_hold(&self, state: &State) -> bool {
        let Self::Active { phase, context } = self else {
            return true;
        };
        let expected = match phase {
            hud_camera_caller::Refresh::Charging => Some(&context.chosen),
            hud_camera_caller::Refresh::Ready
            | hud_camera_caller::Refresh::Returning
            | hud_camera_caller::Refresh::Complete => context.produced.as_ref(),
            hud_camera_caller::Refresh::Pending => None,
        };
        let map_movement = match (phase, &state.interruption) {
            (
                hud_camera_caller::Refresh::Complete,
                super::interruption_caller::Stage::Complete(before),
            ) => before.as_ref(),
            _ => &state.movement,
        };
        expected.is_some_and(|hud| state.hud.retains_ordinary(hud))
            && map_movement == &context.movement
            && state.hud_display_current.as_ref() == Some(&context.source)
            && matches!(&state.caller, Caller::PlayerContinuation { controls, post_down: super::post_down_caller::Stage::Complete(_), .. } if controls == &context.controls)
            && (context.produced.is_some()
                == !matches!(phase, hud_camera_caller::Refresh::Charging))
    }
}

impl State {
    fn hud_map_prefix<'a>(
        &self,
        content: &'a super::Content,
    ) -> Result<&'a Content, ProductionFault> {
        let Caller::PlayerContinuation {
            post_down: post_down @ super::post_down_caller::Stage::Complete(_),
            charging: false,
            ..
        } = &self.caller
        else {
            return Err(missing());
        };
        if !self.animation_prefix_holds(post_down) {
            return Err(missing());
        }
        let current = content.hud_refresh.as_ref().ok_or_else(missing)?;
        if self.validate_room_owner(content).map_err(|_| missing())? != current.room {
            return Err(missing());
        }
        Ok(current)
    }

    fn current_display(
        &self,
        admitted: &Content,
        before: &Hud,
        rules: &super::Rules,
    ) -> Result<Model<hud_stage::DisplayReceiptPin>, ProductionFault> {
        if self.publications.original_interval() {
            let retained = self.hud_display_current.as_ref().ok_or_else(missing)?;
            if retained.0.as_ref() != &admitted.source {
                return Err(missing());
            }
            return Ok(retained.clone());
        }
        let receipt = admitted
            .recurring
            .as_ref()
            .and_then(|scope| {
                scope.current(self, before, rules.hud.get(), &admitted.source.premises.map)
            })
            .ok_or_else(missing)?;
        let actual = Model(Box::new(receipt));
        if self
            .hud_display_current
            .as_ref()
            .is_some_and(|retained| retained != &actual)
        {
            return Err(missing());
        }
        Ok(actual)
    }

    fn fresh_hud_choice(
        &self,
        before: &Hud,
        content: &super::Content,
        rules: &super::Rules,
        current: &hud_stage::DisplayReceipt,
    ) -> Result<(Hud, Hud, Profile), ProductionFault> {
        let Caller::PlayerContinuation { controls, .. } = &self.caller else {
            return Err(missing());
        };
        let admitted = content.hud_refresh.as_ref().ok_or_else(missing)?;
        let samus = self.movement.state();
        let mut hud = before.clone();
        let mut refresh = hud_camera_caller::Refresh::Pending;
        let profile = refresh
            .choose(
                &mut hud,
                &self.movement,
                controls,
                hud_stage::CurrentDisplay {
                    premises: &current.premises,
                    status: current.status,
                    position: types::Pair {
                        x: samus.position.x.pixel().0,
                        y: samus.position.y.pixel().0,
                    },
                    action: samus.pose.action,
                    event_owner: Some(&current.premises.events),
                },
                (rules.hud.get(), &content.hud),
                (current, &admitted.physical),
            )
            .map_err(|_| missing())?;
        let mut result = hud.clone();
        let publication = refresh
            .complete_producer(&mut result, hud_camera_caller::HUD_REFRESH)
            .map_err(|_| missing())?;
        if !publication.events.is_empty() {
            return Err(missing());
        }
        Ok((hud, result, profile))
    }

    /// Choose current display inputs once, or validate the retained producer/return.
    pub(super) fn advance_hud_map(
        &mut self,
        content: &super::Content,
        rules: &super::Rules,
    ) -> Result<(), ProductionFault> {
        if !self.hud_map.holds(self) {
            return Err(missing());
        }
        match &self.hud_map {
            Stage::Pending => {
                let admitted = self.hud_map_prefix(content)?;
                let current = self.current_display(admitted, &self.hud, rules)?;
                let (chosen, result, profile) =
                    self.fresh_hud_choice(&self.hud, content, rules, &current.0)?;
                let Caller::PlayerContinuation { controls, .. } = &self.caller else {
                    return Err(missing());
                };
                let owner = Context {
                    before: self.hud.clone(),
                    chosen: chosen.clone(),
                    result,
                    produced: None,
                    movement: self.movement.clone(),
                    controls: controls.clone(),
                    source: current.clone(),
                    room: admitted.room,
                };
                self.timing
                    .begin(hud_camera_caller::HUD_REFRESH, profile)
                    .map_err(ProductionFault::FrameHardware)?;
                self.hud_display_current = Some(current);
                self.hud = chosen;
                self.hud_map = Stage::Active {
                    phase: hud_camera_caller::Refresh::Charging,
                    context: Box::new(owner),
                };
            }
            Stage::Active {
                phase: hud_camera_caller::Refresh::Charging | hud_camera_caller::Refresh::Returning,
                ..
            } => self.validate_hud_map(content, rules)?,
            Stage::Active {
                phase: hud_camera_caller::Refresh::Ready,
                ..
            } => {
                self.validate_hud_map(content, rules)?;
                let admitted = content.hud_refresh.as_ref().ok_or_else(missing)?;
                let Stage::Active { phase, .. } = &mut self.hud_map else {
                    return Err(missing());
                };
                let profile = phase
                    .begin_return(&self.hud, &admitted.physical)
                    .map_err(|_| missing())?;
                self.timing
                    .begin(hud_camera_caller::HUD_RETURN, profile)
                    .map_err(ProductionFault::FrameHardware)?;
            }
            Stage::Active {
                phase: hud_camera_caller::Refresh::Complete,
                ..
            } => {
                self.validate_hud_map(content, rules)?;
                self.advance_countdown(content, rules)?;
            }
            Stage::Active {
                phase: hud_camera_caller::Refresh::Pending,
                ..
            } => return Err(missing()),
        }
        Ok(())
    }

    pub(crate) fn validate_hud_checkpoint(
        &self,
        content: &super::Content,
        rules: &super::Rules,
    ) -> Result<(), super::Fault> {
        if self.hud_map.identity().is_some() {
            self.validate_hud_map(content, rules)
                .map_err(|_| super::Fault::InvalidPlayerEntry)?;
        }
        self.validate_late_status_work(content)
            .map_err(|_| super::Fault::InvalidPlayerEntry)?;
        self.validate_completed_late_hud_queue(content)
            .map_err(|_| super::Fault::InvalidPlayerEntry)?;
        Ok(())
    }

    pub(super) fn validate_hud_map(
        &self,
        content: &super::Content,
        rules: &super::Rules,
    ) -> Result<(), ProductionFault> {
        let admitted = self.hud_map_prefix(content)?;
        if !self.hud_map.holds(self) {
            return Err(missing());
        }
        let Stage::Active { phase, context } = &self.hud_map else {
            return Err(missing());
        };
        let current = self.current_display(admitted, &context.before, rules)?;
        let (chosen, result, producer) =
            self.fresh_hud_choice(&context.before, content, rules, &current.0)?;
        if chosen != context.chosen || result != context.result || admitted.room != context.room {
            return Err(missing());
        }
        if let Some(identity) = self.hud_map.identity() {
            let expected = if identity == hud_camera_caller::HUD_REFRESH {
                producer
            } else {
                let [_, tail] = admitted.physical.profiles().map_err(|_| missing())?;
                tail
            };
            let retained: Vec<_> = self
                .timing
                .work
                .iter()
                .filter(|work| work.kind == timing::WorkKind::Semantic(identity))
                .collect();
            if retained.len() != 1 || retained.first().is_none_or(|work| work.profile != expected) {
                return Err(missing());
            }
        }
        if !matches!(phase, hud_camera_caller::Refresh::Charging) {
            let tilemap = super::hud::tilemap(context.result.state()).map_err(|_| missing())?;
            if context.produced.as_ref() != Some(&context.result)
                || !context.result.events().is_empty()
                || !self
                    .content
                    .ready_buffer(super::Buffer::HudTilemap)
                    .is_some_and(|output| {
                        output.region.start == admitted.address && output.region.bytes == tilemap
                    })
            {
                return Err(missing());
            }
        }
        Ok(())
    }

    /// Publish the complete actual HUD only at producer return; retain its separate tail.
    pub(super) fn complete_hud_map(
        &mut self,
        work: timing::WorkId,
        content: &super::Content,
        rules: &super::Rules,
    ) -> Result<(), ProductionFault> {
        // The returned operation is no longer in timing's retained active work.
        let admitted = self.hud_map_prefix(content)?;
        if !self.hud_map.owners_hold(self) || self.hud_map.identity() != Some(work) {
            return Err(missing());
        }
        let Stage::Active { phase, context } = &mut self.hud_map else {
            return Err(missing());
        };
        if work == hud_camera_caller::HUD_REFRESH {
            let publication = phase
                .complete_producer(&mut self.hud, work)
                .map_err(|_| missing())?;
            if !publication.events.is_empty() {
                return Err(missing());
            }
            self.content
                .complete(super::Materialization {
                    buffer: super::Buffer::HudTilemap,
                    region: data::timing::ContentRegion {
                        start: admitted.address,
                        bytes: publication.tilemap,
                    },
                })
                .map_err(|_| missing())?;
            context.produced = Some(self.hud.clone());
        } else {
            phase.complete_return(work).map_err(|_| missing())?;
        }
        self.validate_hud_map(content, rules)
    }
}
