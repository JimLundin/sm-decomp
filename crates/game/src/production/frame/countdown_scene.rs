//! Explicitly borrowed scene scheduling, independent of countdown and coarse gameplay.

use super::{
    Caller, Content, Dependency, State,
    countdown_caller::Stage as CountdownStage,
    interrupts::{DrawingRequest, DrawingState},
};
use crate::production::{ProductionFault, Unsupported, timing};
use canonical::{Canonical, model::Pinned};
use data::timing::{Interrupt, WRAM_DATA_PORT};
use serde::{Deserialize, Serialize};

/// Semantic service selected by the separately qualified current scene owner.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Choice {
    /// The selected service may maintain and present the countdown.
    MaintainCountdown,
    /// The selected service may present without maintaining the countdown.
    PresentCountdownOnly,
}

/// The independently qualified lifetime of the current scene slot.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum Scope {
    InitialThroughCeres,
    Adjacent(u64),
}

/// Runtime invalidation, including writes which leave the semantic value unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum Freshness {
    Preserved,
    SceneWrite,
    UnqualifiedActivity,
}

/// Actual accepted HUD interrupt history in this interval.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum Interrupts {
    Unseen,
    Begun,
    Returned,
}

/// Complete live scoped scene owner; no fixture proof flags are runtime state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Current {
    choice: Choice,
    scope: Scope,
    freshness: Freshness,
    interrupts: Interrupts,
}

/// Complete independent scoped origin, bound in immutable production content.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Admission {
    /// Choice decoded from the independently admitted initial scene slot.
    pub choice: Choice,
}

/// Actual scene-selected input offered before any clean countdown answer exists.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Offer {
    choice: Admission,
    scope: Scope,
}

/// Complete current scheduling owner, including its actual invalidation/IRQ progress.
pub struct CurrentPin;
impl Pinned for CurrentPin {
    type State = Current;
    const NAME: &'static str = "production-current-countdown-scene";
    const SHAPE: u64 = 0x5436_68C9_C31C_9BBB;
}

/// Missing, stale, unsupported or unqualified scheduling input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// The current owner does not match its admitted origin or retained offer.
    Changed,
    /// A writer or activity invalidates the admitted preservation interval.
    Activity,
    /// The presentation-only arm has no integrated qualification in this scope.
    Choice,
}

impl Current {
    /// Admit one independent initial scene choice without choosing countdown work.
    #[must_use]
    pub const fn from_origin(choice: Choice) -> Self {
        Self {
            choice,
            scope: Scope::InitialThroughCeres,
            freshness: Freshness::Preserved,
            interrupts: Interrupts::Unseen,
        }
    }

    /// Any actual scene-slot write invalidates this no-writer qualification, even same-value.
    pub const fn write(&mut self, choice: Choice) {
        self.choice = choice;
        self.freshness = Freshness::SceneWrite;
    }

    /// An activity outside the admitted interval cannot preserve the old qualification.
    pub const fn invalidate(&mut self) {
        self.freshness = Freshness::UnqualifiedActivity;
    }

    /// Observe an actual successfully qualified HUD IRQ admission, never a source flag.
    pub const fn hud_irq(&mut self, request: DrawingRequest) {
        match (self.interrupts, request) {
            (Interrupts::Unseen, DrawingRequest::Begin) => {
                self.interrupts = Interrupts::Begun;
            }
            (Interrupts::Begun, DrawingRequest::End) => {
                self.interrupts = Interrupts::Returned;
            }
            _ => self.invalidate(),
        }
    }

    pub(super) const fn begin_adjacent(mut self, start: u64) -> Self {
        self.scope = Scope::Adjacent(start);
        self.interrupts = Interrupts::Unseen;
        self
    }

    /// Validate the complete current no-writer owner against its independently admitted origin.
    /// # Errors
    /// A write/activity or changed origin rejects the current qualification.
    pub fn validate(&self, admission: Admission) -> Result<(), Fault> {
        self.validate_preserved()?;
        if self.choice != admission.choice {
            return Err(Fault::Changed);
        }
        Ok(())
    }

    /// Check actual writer/activity invalidation when restoring the complete owner.
    /// # Errors
    /// A scene write or out-of-scope activity invalidates this frame qualification.
    pub fn validate_preserved(&self) -> Result<(), Fault> {
        if self.freshness != Freshness::Preserved {
            return Err(Fault::Activity);
        }
        Ok(())
    }

    /// Select from the live current owner; a result or coarse route is not an input.
    /// # Errors
    /// Missing freshness or the unqualified presentation-only scheduling arm.
    pub fn offer(&self, admission: Admission) -> Result<Offer, Fault> {
        self.validate(admission)?;
        if self.choice != Choice::MaintainCountdown {
            return Err(Fault::Choice);
        }
        Ok(Offer {
            choice: admission,
            scope: self.scope,
        })
    }

    /// Read-only same-owner qualification for the later actual Ceres consumer.
    /// This validates scheduling only, never completes or reruns countdown work.
    /// The caller must separately prove actual Ceres opportunity and countdown completion.
    /// # Errors
    /// Changed disposition, unqualified activity or incomplete accepted interrupt history.
    pub fn revalidate_ceres_input(&self, admission: Admission, offer: Offer) -> Result<(), Fault> {
        if self.offer(admission)? != offer {
            return Err(Fault::Changed);
        }
        if self.interrupts != Interrupts::Returned {
            return Err(Fault::Activity);
        }
        Ok(())
    }
}

impl Offer {
    /// Actual semantic service choice, with no countdown effects or timing advancement.
    #[must_use]
    pub const fn choice(self) -> Choice {
        self.choice.choice
    }
}

const fn missing() -> ProductionFault {
    ProductionFault::Unsupported(Unsupported::Frame(Dependency::CountdownSceneScheduling))
}

fn scene_writer(event: &timing::Event) -> bool {
    match &event.kind {
        timing::EventKind::Publication(write) => write.port == WRAM_DATA_PORT,
        timing::EventKind::Payload(payload) => payload.target.publication_port() == WRAM_DATA_PORT,
        timing::EventKind::Sample(..)
        | timing::EventKind::BulkService(_)
        | timing::EventKind::LineService(_)
        | timing::EventKind::Interrupt(_)
        | timing::EventKind::ControllerStart
        | timing::EventKind::ControllerEnd
        | timing::EventKind::ControllerCommitted => false,
    }
}

fn contains_one_nmi(events: &[timing::Event]) -> bool {
    let mut nmis = events
        .iter()
        .filter(|event| event.kind == timing::EventKind::Interrupt(Interrupt::Nmi));
    nmis.next().is_some() && nmis.next().is_none()
}

impl State {
    pub(super) fn scene_after_native_return(
        &self,
        content: &Content,
        native_admitted: bool,
    ) -> Result<Option<Current>, ProductionFault> {
        let Some(adjacent) = content.adjacent_scene_countdown else {
            return Ok(None);
        };
        if !self.publications.original_interval() {
            return Ok(None);
        }
        let owner = self.scene_countdown.as_ref().ok_or_else(missing)?.0;
        let initial = self.scene_admission(content, &owner)?;
        if !native_admitted
            || owner.interrupts != Interrupts::Returned
            || owner.validate(initial).is_err()
            || initial != adjacent
            || self.publications.events.iter().any(scene_writer)
        {
            return Err(missing());
        }
        Ok(Some(owner))
    }

    fn scene_admission(
        &self,
        content: &Content,
        owner: &Current,
    ) -> Result<Admission, ProductionFault> {
        let start = self.publications.interval_start();
        match owner.scope {
            Scope::InitialThroughCeres if start == 0 => content.scene_countdown,
            Scope::Adjacent(opened) if opened == start && start != 0 => {
                content.adjacent_scene_countdown
            }
            Scope::InitialThroughCeres | Scope::Adjacent(_) => None,
        }
        .ok_or_else(missing)
    }

    pub(crate) fn validate_scene_checkpoint(&self, content: &Content) -> Result<(), super::Fault> {
        let offer = match &self.countdown {
            CountdownStage::Pending => None,
            CountdownStage::Charging { offer, .. } | CountdownStage::Complete { offer, .. } => {
                Some(*offer)
            }
        };
        let prior = self
            .publications
            .preceding()
            .map_err(super::Fault::Publications)?;
        let adjacent = content
            .adjacent_scene_countdown
            .filter(|_| contains_one_nmi(prior));
        let Some(owner) = self.scene_countdown.as_ref().map(|owner| &owner.0) else {
            return if offer.is_none() && adjacent.is_none() {
                Ok(())
            } else {
                Err(super::Fault::InvalidPlayerEntry)
            };
        };
        if matches!(owner.scope, Scope::Adjacent(_))
            && (adjacent.is_none()
                || content.scene_countdown != adjacent
                || prior.iter().any(scene_writer))
        {
            return Err(super::Fault::InvalidPlayerEntry);
        }
        let invalid = |_| super::Fault::InvalidPlayerEntry;
        let (gameplay, native_admitted) = self.gameplay_publications().map_err(invalid)?;
        self.validate_scene_preservation(content, gameplay, native_admitted)
            .map_err(invalid)?;
        let admission = self.scene_admission(content, owner).map_err(invalid)?;
        if offer.is_some_and(|offer| owner.offer(admission) != Ok(offer)) {
            return Err(super::Fault::InvalidPlayerEntry);
        }
        Ok(())
    }

    fn current_scene(&self, content: &Content) -> Result<(&Current, Admission), ProductionFault> {
        let (gameplay, native_admitted) = self.gameplay_publications()?;
        if native_admitted {
            return Err(missing());
        }
        self.validate_scene_preservation(content, gameplay, native_admitted)?;
        let owner = &self.scene_countdown.as_ref().ok_or_else(missing)?.0;
        if matches!(owner.scope, Scope::Adjacent(_)) {
            let irq_count = gameplay
                .iter()
                .filter(|event| event.kind == timing::EventKind::Interrupt(Interrupt::Irq))
                .count();
            // The scene records actual admissions. The drawing owner retires End on return.
            if owner.interrupts != Interrupts::Returned
                || irq_count != 2
                || !matches!(
                    self.drawing,
                    Some(DrawingState {
                        active: None,
                        recurring_due: None,
                        ..
                    })
                )
            {
                return Err(missing());
            }
        }
        Ok((owner, self.scene_admission(content, owner)?))
    }

    pub(super) fn revalidate_countdown_scene_at_ceres(
        &self,
        content: &Content,
        offer: Offer,
    ) -> Result<(), ProductionFault> {
        let (owner, admission) = self.current_scene(content)?;
        owner
            .revalidate_ceres_input(admission, offer)
            .map_err(|_| missing())
    }

    /// Revalidate actual runtime preservation before any dependent effect.
    pub(super) fn validate_scene_preservation(
        &self,
        content: &Content,
        gameplay_publications: &[timing::Event],
        native_admitted: bool,
    ) -> Result<(), ProductionFault> {
        let Some(owner) = &self.scene_countdown else {
            return Ok(());
        };
        owner
            .0
            .validate(self.scene_admission(content, &owner.0)?)
            .map_err(|_| missing())?;
        if gameplay_publications.iter().any(|event| {
            scene_writer(event) || event.kind == timing::EventKind::Interrupt(Interrupt::Nmi)
        }) {
            return Err(missing());
        }
        if native_admitted {
            return Ok(());
        }
        for work in &self.timing.work {
            match work.kind {
                timing::WorkKind::Native {
                    interrupt,
                    identity,
                    ..
                } if interrupt != data::timing::Interrupt::Irq
                    || identity != timing::WorkId(100) =>
                {
                    return Err(missing());
                }
                timing::WorkKind::Handler { identity, .. } if identity != timing::WorkId(100) => {
                    return Err(missing());
                }
                timing::WorkKind::Semantic(_)
                | timing::WorkKind::Wait(_)
                | timing::WorkKind::Native { .. }
                | timing::WorkKind::Handler { .. } => {}
            }
        }
        Ok(())
    }

    /// Derive the actual Timer scene offer from current owners at returned `HUDMap`.
    /// No selected countdown transition is claimed until its clean producer exists.
    /// # Errors
    /// Early, missing, stale or unsupported inputs reject without state/output changes.
    pub fn countdown_scene_offer(
        &self,
        content: &Content,
        rules: &super::Rules,
    ) -> Result<Offer, ProductionFault> {
        let (owner, admission) = self.current_scene(content)?;
        if !self.hud_map.is_complete()
            || !matches!(
                self.caller,
                Caller::PlayerContinuation {
                    post_down: super::post_down_caller::Stage::Complete(_),
                    charging: false,
                    ..
                }
            )
        {
            return Err(missing());
        }
        self.validate_hud_map(content, rules)?;
        owner.offer(admission).map_err(|_| missing())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use canonical::model::Model;
    #[test]
    fn complete_scene_owner_shape_remains_pinned() {
        assert_eq!(canonical::model::pin_holds::<CurrentPin>(), Ok(()));
    }
    #[test]
    fn same_value_write_and_unqualified_activity_invalidate_selection() {
        let admission = Admission {
            choice: Choice::MaintainCountdown,
        };
        let mut current = Current::from_origin(admission.choice);
        let offered = current.offer(admission).unwrap();
        assert_eq!(offered.choice(), Choice::MaintainCountdown);
        current.write(admission.choice);
        assert_eq!(current.offer(admission), Err(Fault::Activity));
        let mut current = Current::from_origin(admission.choice);
        current.invalidate();
        assert_eq!(current.offer(admission), Err(Fault::Activity));
    }
    #[test]
    fn accepted_irq_history_preserves_one_offer_without_second_countdown_call() {
        let admission = Admission {
            choice: Choice::MaintainCountdown,
        };
        let mut current = Current::from_origin(admission.choice);
        current.hud_irq(super::super::interrupts::DrawingRequest::Begin);
        let offer = current.offer(admission).unwrap();
        current.hud_irq(super::super::interrupts::DrawingRequest::End);
        assert_eq!(current.revalidate_ceres_input(admission, offer), Ok(()));
        let bytes = canonical::to_bytes(&Model::<CurrentPin>(current));
        let restored: Model<CurrentPin> = canonical::from_bytes(&bytes).unwrap();
        assert_eq!(restored.0, current);
        assert_eq!(restored.0.revalidate_ceres_input(admission, offer), Ok(()));
    }
}
