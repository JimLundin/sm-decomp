//! One complete current health authority and its finite actor/interrupt preservation.

use super::{
    borrowed_player_preparation as actors, interrupts::DrawingRequest,
    movement_support_contract::ActorCompletionPin,
};
use canonical::{
    Canonical,
    model::{Model, Pinned},
};

/// Complete health quantities; delivery and neighbouring qualification stay outside this pin.
pub struct HealthPin;
impl Pinned for HealthPin {
    type State = samus_health::State;
    const NAME: &'static str = "production-current-samus-health";
    const SHAPE: u64 = 0x2FD6_C95A_B0A2_9544;
}

/// Gameplay clock supplied by the independently qualified neighbouring owner.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Time {
    /// This frame admits periodic loss.
    Running,
    /// Requires an unimplemented stopped-time producer.
    Stopped,
}

/// Independently qualified suit flags; the health model implements neither reduction.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Suits {
    /// Actual Varia equipment disposition.
    pub varia: bool,
    /// Actual Gravity equipment disposition.
    pub gravity: bool,
}

/// Actual current neighbouring activity; only the inactive arm is qualified here.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Activity {
    /// No activity in this finite source envelope.
    Inactive,
    /// Requires a separately qualified producer.
    Active,
}

/// Current region of the pause opportunity, independent from scene scheduling.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Region {
    /// The source-qualified return before reading Start.
    Ceres,
    /// Another pause path is unimplemented.
    Other,
}

/// Finite original neighbour context, never a borrowed health result.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Context {
    /// Actual current gameplay clock.
    pub time: Time,
    /// Actual current damage-reducing suits.
    pub suits: Suits,
    /// Current power-bomb activity; another pause arm is unsupported.
    pub power_bomb: Activity,
    /// Current elevator activity; another pause arm is unsupported.
    pub elevator: Activity,
    /// Ceres region admits the pause return before Start is read.
    pub region: Region,
    /// Actual inactive protection shared with actor contact and Drawing.
    pub protection: Activity,
}

/// Independently admitted original quantities and context, retained as read-only provenance.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Admission {
    /// Complete initial health, including both fractions and pending loss.
    pub initial: Model<HealthPin>,
    /// Explicit finite borrowed neighbouring inputs.
    pub context: Context,
}

#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
enum Freshness {
    Preserved,
    Invalidated,
    Retired,
}
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
enum InterruptHistory {
    Unseen,
    Begun,
    Returned,
}

/// A missing lifetime join, unsupported context or writer invalidates this frame scope.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// A same-value or changing writer/activity invalidated the source qualification.
    Activity,
    /// An actual producer or original input no longer matches this authority.
    Owner,
    /// Stopped time, suits, protection or pause conditions require another producer.
    Context,
}

/// Complete live health and actual no-writer history; no maintenance runs on construction.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Health {
    current: Model<HealthPin>,
    origin: Option<Admission>,
    context: Context,
    actor: Option<Model<ActorCompletionPin>>,
    interrupts: InterruptHistory,
    freshness: Freshness,
    maintained: bool,
}

impl Context {
    /// Admit only the independently reviewed finite neighbouring arm.
    /// # Errors
    /// Wider clocks, equipment, protection and pause arms are not implemented.
    pub fn qualify(self) -> Result<(), Fault> {
        if self.time != Time::Running
            || self.suits.varia
            || self.suits.gravity
            || self.power_bomb != Activity::Inactive
            || self.elevator != Activity::Inactive
            || self.region != Region::Ceres
            || self.protection != Activity::Inactive
        {
            return Err(Fault::Context);
        }
        Ok(())
    }
}

impl Health {
    /// Construct all five quantities directly from the original owner without applying loss.
    /// # Errors
    /// Unsupported neighbouring context cannot manufacture a qualified origin.
    pub fn from_origin(admission: Admission) -> Result<Self, Fault> {
        admission.context.qualify()?;
        if admission.initial.0.protection != samus_health::ProtectionFrames::ZERO {
            return Err(Fault::Context);
        }
        Ok(Self {
            current: admission.initial,
            context: admission.context,
            origin: Some(admission),
            actor: None,
            interrupts: InterruptHistory::Unseen,
            freshness: Freshness::Preserved,
            maintained: false,
        })
    }

    /// Sole current complete health, readable without selecting periodic maintenance.
    #[must_use]
    pub const fn state(&self) -> &samus_health::State {
        &self.current.0
    }

    /// Any external health, damage, equipment, protection or clock write invalidates this scope.
    /// Same-value writes are activity, not proof that the old input remains fresh.
    pub const fn invalidate(&mut self) {
        self.freshness = Freshness::Invalidated;
    }

    /// Explicit external replacement cannot preserve the finite no-writer qualification.
    pub const fn write(&mut self, state: samus_health::State) {
        self.current = Model(state);
        self.invalidate();
    }

    /// Explicit clock, equipment or protection writes invalidate even identical inputs.
    pub const fn write_context(&mut self, context: Context) {
        self.context = context;
        self.invalidate();
    }

    /// Observe only the actual successfully admitted HUD interrupt pair.
    pub const fn hud_irq(&mut self, request: DrawingRequest) {
        match (self.interrupts, request) {
            (InterruptHistory::Unseen, DrawingRequest::Begin) => {
                self.interrupts = InterruptHistory::Begun;
            }
            (InterruptHistory::Begun, DrawingRequest::End) => {
                self.interrupts = InterruptHistory::Returned;
            }
            _ => self.invalidate(),
        }
    }

    /// Join the actual returned bounded actor operation, including the shared protection alias.
    /// # Errors
    /// Missing no-hurt qualification, changed clocks/protection or duplicate completion.
    pub fn actor_completed(
        &mut self,
        completion: &actors::ActorUpdateCompletion,
    ) -> Result<(), Fault> {
        self.validate_current()?;
        let dependency = &completion.dependency;
        if self.actor.is_some()
            || completion.solid_boxes().is_err()
            || dependency.contact.samus_invincibility != 0
            || dependency.contact.damage_mode != actors::ContactDamageDisposition::Ordinary
            || dependency.clocks.simulation != actors::ClockDisposition::Running
        {
            return Err(Fault::Owner);
        }
        self.actor = Some(Model(completion.clone()));
        Ok(())
    }

    /// Validate complete original input and retained source lifetime before a dependent effect.
    /// # Errors
    /// Source changes, writes and unsupported neighbouring inputs reject atomically.
    pub fn validate_origin(&self, admission: &Admission) -> Result<(), Fault> {
        if self.freshness != Freshness::Preserved {
            return Err(Fault::Activity);
        }
        admission.context.qualify()?;
        if self.current.0.protection != samus_health::ProtectionFrames::ZERO
            || self.origin.as_ref() != Some(admission)
            || self.context != admission.context
            || (!self.maintained && self.current != admission.initial)
        {
            return Err(Fault::Owner);
        }
        Ok(())
    }

    /// Complete owned source qualification for canonical checkpoint admission.
    /// # Errors
    /// Invalidated or changed current inputs cannot regain freshness by restoration.
    pub fn validate_current(&self) -> Result<(), Fault> {
        if self.freshness == Freshness::Retired {
            self.context.qualify()?;
            return if self
                .origin
                .as_ref()
                .is_some_and(|origin| origin.context == self.context)
                && self.state().protection == samus_health::ProtectionFrames::ZERO
                && (!self.maintained || self.interrupts == InterruptHistory::Returned)
            {
                Ok(())
            } else {
                Err(Fault::Owner)
            };
        }
        self.validate_origin(self.origin.as_ref().ok_or(Fault::Owner)?)
    }

    pub(super) fn retire_opportunity(&mut self) {
        self.actor = None;
        self.interrupts = InterruptHistory::Unseen;
        self.maintained = false;
        self.freshness = Freshness::Retired;
    }

    /// Read fresh current whole energy for the actual pre-Maintenance Animation opportunity.
    /// # Errors
    /// The same actual actor and both qualified IRQ admissions must have completed.
    pub fn at_animation(
        &self,
        admission: &Admission,
        actors: &actors::ActorUpdateCompletion,
    ) -> Result<u16, Fault> {
        self.validate_animation_origin(admission, actors)?;
        if self.maintained {
            return Err(Fault::Owner);
        }
        Ok(self.current.0.energy.whole.get())
    }

    fn validate_animation_origin(
        &self,
        admission: &Admission,
        actors: &actors::ActorUpdateCompletion,
    ) -> Result<(), Fault> {
        self.validate_origin(admission)?;
        if self.actor() != Some(actors) || self.interrupts != InterruptHistory::Returned {
            return Err(Fault::Owner);
        }
        Ok(())
    }

    pub(super) fn complete_maintenance(
        &mut self,
        before: &samus_health::State,
        result: samus_health::State,
    ) -> Result<(), Fault> {
        self.validate_current()?;
        if self.maintained
            || self.state() != before
            || self.interrupts != InterruptHistory::Returned
        {
            return Err(Fault::Owner);
        }
        self.current = Model(result);
        self.maintained = true;
        Ok(())
    }

    pub(super) const fn maintained(&self) -> bool {
        self.maintained
    }

    pub(super) fn holds_recurring_actor(
        &self,
        admission: &Admission,
        actor: &actors::ActorUpdateCompletion,
    ) -> bool {
        self.freshness == Freshness::Retired
            && self.origin.as_ref() == Some(admission)
            && self.context == admission.context
            && self.validate_current().is_ok()
            && self.actor() == Some(actor)
    }

    /// Actual actor completion retained independently from the original source declaration.
    #[must_use]
    pub fn actor(&self) -> Option<&actors::ActorUpdateCompletion> {
        self.actor.as_ref().map(|receipt| &receipt.0)
    }
}

const fn missing() -> crate::production::ProductionFault {
    crate::production::ProductionFault::Unsupported(crate::production::Unsupported::Frame(
        super::Dependency::HealthCurrentInputs,
    ))
}

impl super::State {
    pub(super) fn energy_for_animation(
        &self,
        content: &super::Content,
        actors: &actors::ActorUpdateCompletion,
    ) -> Result<u16, crate::production::ProductionFault> {
        let owner = self.health.as_ref().ok_or_else(missing)?;
        let origin = content.health.as_ref().ok_or_else(missing)?;
        if self.publications.original_interval() {
            return owner.at_animation(origin, actors).map_err(|_| missing());
        }
        let (gameplay, _) = self.gameplay_publications()?;
        if content.adjacent_health_context != Some(origin.context)
            || !self.adjacent_health_history_holds()
            || !self.drawing_irqs_returned(gameplay)
            || !owner.holds_recurring_actor(origin, actors)
        {
            return Err(missing());
        }
        self.validate_health_preservation(content, gameplay)?;
        Ok(owner.state().energy.whole.get())
    }

    fn adjacent_health_history_holds(&self) -> bool {
        use crate::production::timing::EventKind;
        use data::timing::Interrupt::{Irq, Nmi};
        self.publications.preceding().is_ok_and(|prior| {
            !prior.iter().any(super::publications::writes_wram)
                && prior
                    .iter()
                    .filter_map(|event| {
                        if let EventKind::Interrupt(interrupt) = event.kind {
                            Some(interrupt)
                        } else {
                            None
                        }
                    })
                    .eq([Irq, Irq, Nmi])
        })
    }

    pub(super) fn original_animation_energy(&self, content: &super::Content) -> Option<u16> {
        let super::Caller::PlayerContinuation { remaining, .. } = &self.caller else {
            return None;
        };
        let admission = content.health.as_ref()?;
        self.health
            .as_ref()?
            .validate_animation_origin(admission, remaining.0.actors.as_ref()?)
            .ok()?;
        // Maintenance is the first writer in this qualified interval and follows Animation.
        Some(admission.initial.0.energy.whole.get())
    }

    pub(crate) fn validate_carried_health(
        &self,
        content: &super::Content,
    ) -> Result<(), super::Fault> {
        use super::Fault::InvalidPlayerEntry;
        let Some(neighbours) = content.adjacent_health_context else {
            return Ok(());
        };
        let owner = self.health.as_ref().ok_or(InvalidPlayerEntry)?;
        let admission = content.health.as_ref().ok_or(InvalidPlayerEntry)?;
        if neighbours != admission.context || owner.context != neighbours {
            return Err(InvalidPlayerEntry);
        }
        if self.publications.original_interval() {
            owner
                .validate_origin(admission)
                .map_err(|_| InvalidPlayerEntry)?;
            if !owner.maintained {
                return match self.return_progress {
                    super::frame_return::Stage::Pending => Ok(()),
                    super::frame_return::Stage::Finalizing(_)
                    | super::frame_return::Stage::Ready { .. } => Err(InvalidPlayerEntry),
                };
            }
        } else {
            let (gameplay, _) = self
                .gameplay_publications()
                .map_err(|_| InvalidPlayerEntry)?;
            self.validate_health_preservation(content, gameplay)
                .map_err(|_| InvalidPlayerEntry)?;
            if owner.freshness != Freshness::Retired || !self.adjacent_health_history_holds() {
                return Err(InvalidPlayerEntry);
            }
        }
        // Validate the saved result of the first Maintenance on a private copy.
        // The live owner is advanced only by the scheduled Maintenance return.
        let mut maintained = admission.initial.0;
        if maintained.health_opportunity(&samus_health::Rules::REFERENCE)
            != samus_health::WarningSound::None
            || (!self.publications.original_interval()
                && owner.maintained
                && maintained.health_opportunity(&samus_health::Rules::REFERENCE)
                    != samus_health::WarningSound::None)
            || owner.state() != &maintained
        {
            return Err(InvalidPlayerEntry);
        }
        Ok(())
    }

    pub(super) fn validate_health_preservation(
        &self,
        content: &super::Content,
        gameplay_publications: &[crate::production::timing::Event],
    ) -> Result<(), crate::production::ProductionFault> {
        use crate::production::timing;
        let (Some(owner), Some(admission)) = (&self.health, &content.health) else {
            return if self.health.is_none() && content.health.is_none() {
                Ok(())
            } else {
                Err(missing())
            };
        };
        if owner.freshness == Freshness::Retired {
            owner.validate_current().map_err(|_| missing())?;
            if owner.origin.as_ref() != Some(admission) || owner.context != admission.context {
                return Err(missing());
            }
        } else {
            owner.validate_origin(admission).map_err(|_| missing())?;
            if self.scene_countdown.is_none() || content.scene_countdown.is_none() {
                return Err(missing());
            }
        }
        if self
            .drawing_origins
            .as_ref()
            .is_some_and(|drawing| drawing.invincibility.0 != 0)
        {
            return Err(missing());
        }
        if let super::Caller::PlayerContinuation { remaining, .. } = &self.caller
            && owner.actor() != remaining.0.actors.as_ref()
        {
            return Err(missing());
        }
        if gameplay_publications.iter().any(|event| {
            event.kind == timing::EventKind::Interrupt(data::timing::Interrupt::Nmi)
                || super::publications::writes_wram(event)
        }) {
            return Err(missing());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_health_shape_remains_pinned() {
        assert_eq!(canonical::model::pin_holds::<HealthPin>(), Ok(()));
    }
}
